//! The command line: what the binary was asked to do (spec RUN-1, RUN-2; dead-code spec CMD-1, CMD-2).

use std::ffi::OsString;
use std::path::PathBuf;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The command that prints the interface version, the hand-off formats the analyzer reads and the
/// declaration keys it knows.
const INTERFACE_COMMAND: &str = "interface";

/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// The command that reads a project and prints a report of what it found (spec CMD-1).
const AUDIT_COMMAND: &str = "audit";

/// The option of `graph` and of `audit` that names the hand-off file.
const HAND_OFF_OPTION: &str = "--hand-off";

/// The option of `audit` that names the commit to compare with (dead-code spec CMD-1).
const BASE_OPTION: &str = "--base";

/// The command that reads a project and prints each function's complexity (complexity spec SOLO-1).
const COMPLEXITY_COMMAND: &str = "complexity";

/// The command that reads a project and prints its groups of copies (duplication spec SHOW-1).
const DUPLICATION_COMMAND: &str = "duplication";

/// The option of `complexity` that lists every function, not only those over a limit, and of
/// `duplication` that lists every group, the accepted ones too (complexity spec SOLO-3; duplication
/// spec SHOW-3).
const ALL_OPTION: &str = "--all";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, graph [--hand-off <file>], audit [--hand-off <file>] \
                        [--base <commit>], complexity [--hand-off <file>] [--all], and duplication \
                        [--hand-off <file>] [--all]";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the version.
    Version,
    /// Print the interface version, the hand-off formats and the declaration keys.
    Interface,
    /// Read the project and print what was read, with a hand-off file or without one.
    Graph { hand_off: Option<PathBuf> },
    /// Read the project and print a report of what was found, with a hand-off file or without one, and
    /// compared with a base commit or with none.
    Audit {
        hand_off: Option<PathBuf>,
        base: Option<String>,
    },
    /// Read the project and print each function's complexity, with a hand-off file or without one: the
    /// functions over a limit, or with `all` every function (complexity spec SOLO-1).
    Complexity {
        hand_off: Option<PathBuf>,
        all: bool,
    },
    /// Read the project and print its groups of copies, with a hand-off file or without one: those a
    /// person has not accepted, or with `all` every group (duplication spec SHOW-1).
    Duplication {
        hand_off: Option<PathBuf>,
        all: bool,
    },
}

/// Reads the arguments after the program's name. The error is the text of the refusal.
pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let words: Vec<&str> = args.iter().map(|arg| arg.to_str().unwrap_or("")).collect();

    match words.as_slice() {
        [VERSION_OPTION] => Ok(Command::Version),
        [INTERFACE_COMMAND] => Ok(Command::Interface),
        [GRAPH_COMMAND] => Ok(Command::Graph { hand_off: None }),
        [GRAPH_COMMAND, HAND_OFF_OPTION, _] => Ok(Command::Graph {
            hand_off: Some(PathBuf::from(&args[2])),
        }),
        [AUDIT_COMMAND, ..] => audit(args),
        [COMPLEXITY_COMMAND, ..] => {
            listing(args).map(|(hand_off, all)| Command::Complexity { hand_off, all })
        }
        [DUPLICATION_COMMAND, ..] => {
            listing(args).map(|(hand_off, all)| Command::Duplication { hand_off, all })
        }
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => Err(refused(args)),
    }
}

/// `audit` and its options, each at most once, in either order. A base that begins with `-`, or holds
/// a control character, is refused before git sees it (dead-code spec CMD-2).
fn audit(args: &[OsString]) -> Result<Command, String> {
    let mut hand_off = None;
    let mut base = None;

    for pair in args[1..].chunks(2) {
        let [option, value] = pair else {
            return Err(refused(args));
        };

        match option.to_str() {
            Some(HAND_OFF_OPTION) if hand_off.is_none() => hand_off = Some(PathBuf::from(value)),
            Some(BASE_OPTION) if base.is_none() => {
                let named = value.to_string_lossy();

                if named.is_empty() || named.starts_with('-') || named.chars().any(char::is_control)
                {
                    return Err(format!(
                        "the base `{named}` cannot be a commit's name: it is empty, begins with `-` or holds a \
                         control character, which git would not read as a name; name the commit another way"
                    ));
                }

                base = Some(named.into_owned());
            }
            _ => return Err(refused(args)),
        }
    }

    Ok(Command::Audit { hand_off, base })
}

/// The options of `complexity` and of `duplication`, each at most once, in either order: the hand-off
/// file, and whether to list everything (complexity spec SOLO-1; duplication spec SHOW-1).
fn listing(args: &[OsString]) -> Result<(Option<PathBuf>, bool), String> {
    let mut hand_off = None;
    let mut all = false;
    let mut words = args[1..].iter();

    while let Some(option) = words.next() {
        match option.to_str() {
            Some(HAND_OFF_OPTION) if hand_off.is_none() => {
                let Some(value) = words.next() else {
                    return Err(refused(args));
                };

                hand_off = Some(PathBuf::from(value));
            }
            Some(ALL_OPTION) if !all => all = true,
            _ => return Err(refused(args)),
        }
    }

    Ok((hand_off, all))
}

/// The refusal of a command line the binary does not answer.
fn refused(args: &[OsString]) -> String {
    let given = args
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    format!("cannot do `{given}`; it answers {ACCEPTED}")
}
