//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads nothing of the project: the lists of packages and files are empty until the parts that
//! fill them are built.

use serde_json::{Value, json};

/// Something a person should know about a run that went on (spec RUN-5).
struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
}

/// Reads the project and returns the document to print.
pub fn read() -> String {
    let notices = [Notice {
        kind: "no-hand-off-file",
        file: None,
        sentence: "No hand-off file was given, so the project is read from its own standard files, with no \
                   declarations."
            .to_owned(),
    }];
    let document = json!({
        "files": [],
        "handOff": false,
        "notices": notices.iter().map(notice).collect::<Vec<Value>>(),
        "packages": [],
    });

    format!("{document:#}")
}

/// One notice as it is printed.
fn notice(notice: &Notice) -> Value {
    json!({ "file": notice.file, "kind": notice.kind, "sentence": notice.sentence })
}
