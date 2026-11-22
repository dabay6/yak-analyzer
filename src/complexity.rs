//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint and message
//! of its finding, and the document the command prints (complexity spec LIMIT-1, LIMIT-2, CPX-2 to
//! CPX-4, SOLO-1 to SOLO-5).

use std::fmt::Write as _;
use std::path::Path;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::files::{self, Kind};
use crate::findings::{self, Rule};
use crate::functions::{self, Function, Scored};
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

/// The measures, in the order a finding names them.
const MEASURES: [&str; 2] = [CYCLOMATIC, COGNITIVE];

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

    /// The limit of the measure named, where one is declared.
    pub fn limit(self, measure: &str) -> Option<u64> {
        if measure == CYCLOMATIC {
            self.cyclomatic
        } else {
            self.cognitive
        }
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

/// A function's two scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scores {
    /// Its cyclomatic complexity.
    pub cyclomatic: u64,
    /// Its cognitive complexity.
    pub cognitive: u64,
}

impl Scores {
    /// The score of the measure named.
    pub fn score(self, measure: &str) -> u64 {
        if measure == CYCLOMATIC {
            self.cyclomatic
        } else {
            self.cognitive
        }
    }
}

impl From<Scored> for Scores {
    fn from(scored: Scored) -> Self {
        Self {
            cyclomatic: scored.cyclomatic,
            cognitive: scored.cognitive,
        }
    }
}

/// What a complexity finding carries in its property bag (spec CPX-3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detail {
    /// The function's readable name path.
    pub function: String,
    /// Its scores.
    pub scores: Scores,
    /// The limits in force.
    pub limits: Limits,
    /// The measures it is over, cyclomatic first.
    pub over: Vec<&'static str>,
    /// Its scores at the base: none without a base, and `Some(None)` where the base does not have
    /// the function.
    pub base: Option<Option<Scores>>,
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

/// What a function's fingerprint is made from after its rule: its file's path, and for each part of
/// its name path `n` for a named part or `u` for an unnamed one, its name, and its number in decimal
/// digits (spec CPX-4; design IF-4).
pub fn parts(path: &str, function: &Function) -> Vec<String> {
    let mut parts = vec![path.to_owned()];

    for part in &function.path {
        parts.push(if part.name.is_some() { "n" } else { "u" }.to_owned());
        parts.push(part.name.clone().unwrap_or_default());
        parts.push(part.number.to_string());
    }

    parts
}

/// The fingerprint of a function's finding (spec CPX-4).
pub fn fingerprint(path: &str, function: &Function) -> String {
    let parts = parts(path, function);
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

    findings::fingerprint(Rule::Complexity, &parts)
}

/// The message of a function's finding: its readable name path, each measure it is over with its
/// score and limit, then the other measure's score, as in "`Parser > read` has cognitive complexity
/// 23, over the limit of 15 (cyclomatic 12)." (spec CPX-2).
pub fn message(measured: &Measured, limits: Limits) -> String {
    let scores = Scores::from(measured.function.scored);
    let over: Vec<String> = measured
        .over
        .iter()
        .map(|measure| {
            format!(
                "{measure} complexity {}, over the limit of {}",
                scores.score(measure),
                limits.limit(measure).unwrap_or_default()
            )
        })
        .collect();
    let others: Vec<String> = MEASURES
        .iter()
        .filter(|measure| !measured.over.contains(measure))
        .map(|measure| format!("{measure} {}", scores.score(measure)))
        .collect();
    let mut message = format!(
        "`{}` has {}",
        measured.function.readable(),
        over.join(", and ")
    );

    if !others.is_empty() {
        let _ = write!(message, " ({})", others.join(", "));
    }

    message.push('.');
    message
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
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
    let limits = reading.limits;
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
