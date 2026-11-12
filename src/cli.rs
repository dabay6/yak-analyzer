//! The command line: what the binary was asked to do (spec RUN-1, RUN-2).

use std::ffi::OsString;
use std::path::PathBuf;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The command that prints the interface version and the hand-off formats the analyzer reads.
const INTERFACE_COMMAND: &str = "interface";

/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// The option of `graph` that names the hand-off file.
const HAND_OFF_OPTION: &str = "--hand-off";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, and graph [--hand-off <file>]";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the version.
    Version,
    /// Print the interface version and the hand-off formats.
    Interface,
    /// Read the project and print what was read, with a hand-off file or without one.
    Graph { hand_off: Option<PathBuf> },
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
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => {
            let given = args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            Err(format!("cannot do `{given}`; it answers {ACCEPTED}"))
        }
    }
}
