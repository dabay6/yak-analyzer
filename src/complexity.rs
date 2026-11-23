//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint of its
//! finding, and the document the command prints (complexity spec LIMIT-1, LIMIT-2, CPX-4, SOLO-1 to
//! SOLO-5).

use std::path::Path;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::files::{self, Kind};
use crate::findings::{self, Rule};
use crate::functions::{self, Function};
use crate::hand_off::{HandOff, Stack};
use crate::notice::{self, Notice};
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::timings::Stages;
use crate::tree::Tree;

/// The kind of the notice for a measure no declaration gives a limit (spec LIMIT-2).
const NOT_CHECKED: &str = "complexity-not-checked";

/// The name of the cyclomatic measure in what is printed (spec CPX-3).
const CYCLOMATIC: &str = "cyclomatic";

/// The name of the cognitive measure in what is printed (spec CPX-3).
const COGNITIVE: &str = "cognitive";

/// The limit of each measure, where one is declared (spec LIMIT-1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    /// The cyclomatic limit, from `cyclomaticLimit`.
    pub cyclomatic: Option<u64>,
    /// The cognitive limit, from `cognitiveLimit`.
    pub cognitive: Option<u64>,
}

impl Limits {
    /// The limits the hand-off file declares, one value of each for the whole run: the project's over
    /// the pack's, and of values from the same source, the later stack's (spec LIMIT-1; design ENT-4).
    pub fn of(handed: Option<&HandOff>) -> Self {
        let stacks = handed.map_or(&[][..], |handed| handed.stacks.as_slice());

        Self {
            cyclomatic: declared(stacks, "cyclomaticLimit"),
            cognitive: declared(stacks, "cognitiveLimit"),
        }
    }

    /// The notice for the measures with no limit, which are not checked, where there are some (spec
    /// LIMIT-2).
    pub fn notice(self) -> Option<Notice> {
        let sentence = match (self.cyclomatic, self.cognitive) {
            (Some(_), Some(_)) => return None,
            (None, None) => {
                "No `cyclomaticLimit` or `cognitiveLimit` is declared, so no function's complexity is \
                 checked and no complexity finding is reported."
            }
            (None, Some(_)) => {
                "No `cyclomaticLimit` is declared, so functions' cyclomatic complexity is not checked."
            }
            (Some(_), None) => {
                "No `cognitiveLimit` is declared, so functions' cognitive complexity is not checked."
            }
        };

        Some(Notice::new(NOT_CHECKED, None, sentence.to_owned()))
    }

    /// The measures a function is over, cyclomatic first: a score greater than its limit, and not one
    /// equal to it (spec CPX-3; the spec's terms).
    pub fn over(self, function: &Function) -> Vec<&'static str> {
        [
            (CYCLOMATIC, self.cyclomatic, function.scored.cyclomatic),
            (COGNITIVE, self.cognitive, function.scored.cognitive),
        ]
        .into_iter()
        .filter(|(_, limit, score)| limit.is_some_and(|limit| *score > limit))
        .map(|(measure, _, _)| measure)
        .collect()
    }
}

/// The value of a limit's key that wins: the last stack's that the project states it for, or else the
/// last stack's that the pack states it for.
fn declared(stacks: &[Stack], key: &str) -> Option<u64> {
    let last = |project: bool| {
        stacks.iter().rev().find_map(|stack| {
            let declarations = if project { &stack.project } else { &stack.pack };

            declarations.get(key).and_then(Value::as_u64)
        })
    };

    last(true).or_else(|| last(false))
}

/// One function of a file, with the measures it is over and the fingerprint of its finding.
pub struct Measured {
    /// The file, by its path from the root.
    pub path: String,
    /// The function, with its name path and scores.
    pub function: Function,
    /// The measures it is over, cyclomatic first.
    pub over: Vec<&'static str>,
    /// The fingerprint `audit` gives, or would give, its finding (spec CPX-4).
    pub fingerprint: String,
}

/// Every function of the reading's own scripts and Vue components, sorted by path, line and column.
/// A file declared data or generated, and a type declaration file, gives none (spec FN-2, CPX-5,
/// SOLO-2).
pub fn measured(reading: &Reading, limits: Limits) -> Vec<Measured> {
    let mut measured = Vec::new();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || reading.marks.generated.contains(&file.path)
        {
            continue;
        }

        let Some(read) = reading.scripts.read.get(&file.path) else {
            continue;
        };

        for function in functions::named(&read.units) {
            measured.push(Measured {
                over: limits.over(&function),
                fingerprint: fingerprint(&file.path, &function),
                path: file.path.clone(),
                function,
            });
        }
    }

    measured.sort_by(|a, b| {
        (&a.path, a.function.scored.line, a.function.scored.column).cmp(&(
            &b.path,
            b.function.scored.line,
            b.function.scored.column,
        ))
    });

    measured
}

/// The fingerprint of a function's finding: its rule, its file's path, and for each part of its name
/// path `n` for a named part or `u` for an unnamed one, its name, and its number in decimal digits
/// (spec CPX-4; design IF-4).
pub fn fingerprint(path: &str, function: &Function) -> String {
    let mut parts = vec![path.to_owned()];

    for part in &function.path {
        parts.push(if part.name.is_some() { "n" } else { "u" }.to_owned());
        parts.push(part.name.clone().unwrap_or_default());
        parts.push(part.number.to_string());
    }

    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

    findings::fingerprint(Rule::Complexity, &parts)
}

/// Reads the working tree as `audit` does, with the hand-off file given or with none, and returns the
/// document `complexity` prints: the limits, the functions over a limit, or with `all` every function,
/// and the notices (spec SOLO-1 to SOLO-3).
pub fn run(hand_off: Option<&Path>, all: bool) -> Result<String, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = reading::start(hand_off)?;
    let listing = files::list(&root, &mut notices)?;
    let limits = Limits::of(handed.as_ref());
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
    let functions: Vec<Value> = measured(&reading, limits)
        .iter()
        .filter(|measured| all || !measured.over.is_empty())
        .map(entry)
        .collect();

    reading.notices.extend(limits.notice());

    let mut document = json!({
        "functions": functions,
        "interfaceVersion": INTERFACE_VERSION,
        "limits": { CYCLOMATIC: limits.cyclomatic, COGNITIVE: limits.cognitive },
        "notices": notice::printed(reading.notices),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// One function as the document lists it (spec SOLO-2).
fn entry(measured: &Measured) -> Value {
    json!({
        "column": measured.function.scored.column,
        "cognitive": measured.function.scored.cognitive,
        "cyclomatic": measured.function.scored.cyclomatic,
        "fingerprint": measured.fingerprint,
        "function": measured.function.readable(),
        "line": measured.function.scored.line,
        "over": measured.over,
        "path": measured.path,
    })
}
