//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file and the file list: the list of packages is empty until the part
//! that fills it is built.

use std::env;
use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::files::{self, File};
use crate::hand_off;
use crate::notice::{self, Notice};
use crate::stop::Stop;

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
    let mut notices = Vec::new();

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice::new(
                    "detection-cut-short",
                    None,
                    "yak's detection stopped at its limit of folders, so the folders the hand-off file lists \
                     are not the whole workspace."
                        .to_owned(),
                ));
            }

            handed.root
        }
        None => {
            notices.push(Notice::new(
                "no-hand-off-file",
                None,
                "No hand-off file was given, so the project is read from its own standard files, with no \
                 declarations."
                    .to_owned(),
            ));

            env::current_dir().map_err(|error| {
                Stop::new(format!(
                    "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
                ))
            })?
        }
    };
    let files = files::list(&root, &mut notices)?;
    let document = json!({
        "files": files.iter().map(file).collect::<Vec<Value>>(),
        "handOff": hand_off_path.is_some(),
        "notices": notice::printed(notices),
        "packages": [],
    });

    Ok(format!("{document:#}"))
}

/// One file as it is printed.
fn file(file: &File) -> Value {
    json!({ "kind": file.kind.name(), "path": file.path, "tracked": file.tracked })
}
