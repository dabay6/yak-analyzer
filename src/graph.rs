//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file and nothing of the project: the lists of packages and files are
//! empty until the parts that fill them are built.

use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::hand_off;
use crate::stop::Stop;

/// Something a person should know about a run that went on (spec RUN-5).
struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
}

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
    let mut notices = Vec::new();

    match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice {
                    kind: "detection-cut-short",
                    file: None,
                    sentence: "yak's detection stopped at its limit of folders, so the folders the hand-off \
                               file lists are not the whole workspace."
                        .to_owned(),
                });
            }
        }
        None => notices.push(Notice {
            kind: "no-hand-off-file",
            file: None,
            sentence:
                "No hand-off file was given, so the project is read from its own standard files, \
                       with no declarations."
                    .to_owned(),
        }),
    }

    // Notices about no file come first, then by file, by kind and by sentence (spec RUN-8).
    notices.sort_by(|a, b| (&a.file, a.kind, &a.sentence).cmp(&(&b.file, b.kind, &b.sentence)));

    let document = json!({
        "files": [],
        "handOff": hand_off_path.is_some(),
        "notices": notices.iter().map(notice).collect::<Vec<Value>>(),
        "packages": [],
    });

    Ok(format!("{document:#}"))
}

/// One notice as it is printed.
fn notice(notice: &Notice) -> Value {
    json!({ "file": notice.file, "kind": notice.kind, "sentence": notice.sentence })
}
