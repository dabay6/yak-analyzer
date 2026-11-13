//! The declaration keys the analyzer knows, and the check that it can use every declaration a hand-off
//! file holds (spec KEY-1, KEY-2, KEY-5).
//!
//! The keys are drafts until the design spec's stage 2 ends. This part checks them; the parts that read a
//! project use them.

use serde_json::Value;

use crate::glob::Glob;
use crate::hand_off::{Stack, leaves_its_folder};
use crate::stop::Stop;

/// What kind of value a key takes.
#[derive(Clone, Copy)]
enum Kind {
    /// A list of globs.
    Globs,
    /// A list of paths.
    Paths,
    /// A positive whole number.
    Number,
    /// Any JSON, which only a pack may state.
    Opaque,
}

/// Every key the analyzer knows, with the kind of its value (spec KEY-1).
const KEYS: [(&str, Kind); 9] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("internal", Kind::Paths),
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("configEntries", Kind::Opaque),
    ("scriptCommands", Kind::Opaque),
];

/// Who states a declaration: the hand-off file keeps the two apart.
#[derive(Clone, Copy)]
enum Source {
    /// The pack that carries the stack.
    Pack,
    /// The project itself.
    Project,
}

impl Source {
    /// The word for the line.
    const fn word(self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::Project => "project",
        }
    }

    /// Where a person changes the declaration. The line names no `yak` command: those names are yak's.
    const fn fix(self) -> &'static str {
        match self {
            Self::Pack => "change it in the pack",
            Self::Project => "change it in the project's own declarations",
        }
    }
}

/// Checks every declaration of every stack, the stacks in the file's order and the pack's before the
/// project's. The first that cannot be used stops the run (spec KEY-2).
pub fn check(stacks: &[Stack]) -> Result<(), Stop> {
    for stack in stacks {
        for (source, declarations) in [
            (Source::Pack, &stack.pack),
            (Source::Project, &stack.project),
        ] {
            for (key, value) in declarations {
                if let Some(why) = problem(key, value, source) {
                    return Err(Stop::new(format!(
                        "cannot use the declaration `{key}` that the {} states for the stack `{}`: {why}; {}",
                        source.word(),
                        stack.name,
                        source.fix()
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Why a declaration cannot be used, or nothing when it can.
fn problem(key: &str, value: &Value, source: Source) -> Option<String> {
    let Some((_, kind)) = KEYS.iter().find(|(known, _)| *known == key) else {
        let known = KEYS.map(|(known, _)| known).join(", ");

        return Some(format!("the analyzer knows no such key, only {known}"));
    };

    match kind {
        Kind::Globs => match texts(value) {
            Some(globs) => globs.iter().find_map(|glob| glob_problem(glob)),
            None => Some("its value must be a list of globs".to_owned()),
        },
        Kind::Paths => match texts(value) {
            Some(paths) => paths.iter().find_map(|path| path_problem(path)),
            None => Some("its value must be a list of paths".to_owned()),
        },
        Kind::Number => match value.as_u64() {
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
        Kind::Opaque => match source {
            Source::Pack => None,
            Source::Project => Some("only a pack may state this key".to_owned()),
        },
    }
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
}

/// Why a path cannot be used, or nothing when it can.
fn path_problem(path: &str) -> Option<String> {
    if path.is_empty() {
        Some("a path in it is empty".to_owned())
    } else if leaves_its_folder(path) {
        Some(format!(
            "the path `{path}` is absolute or steps out of its folder with `..`"
        ))
    } else {
        None
    }
}

/// Why a glob cannot be used, or nothing when it can (spec KEY-5).
fn glob_problem(glob: &str) -> Option<String> {
    if glob.is_empty() {
        return Some("a glob in it is empty".to_owned());
    }

    if leaves_its_folder(glob) {
        return Some(format!(
            "the glob `{glob}` is absolute or steps out of its folder with `..`"
        ));
    }

    Glob::new(glob).err()
}
