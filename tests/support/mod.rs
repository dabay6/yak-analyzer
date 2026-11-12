//! What the tests that hand the binary a hand-off file share.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs the built binary with the arguments given.
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .output()
        .expect("the binary starts")
}

/// A new, empty folder of one test's own, under cargo's folder for the files tests make. It holds an
/// empty git repository, `checkout`, which the test's hand-off files name as their root, since the
/// analyzer reads its root through git.
pub fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let checkout = folder.join("checkout");

    // The folder is left over from an earlier run, or is not there: either way it is made new.
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&checkout).expect("the test's folder can be made");

    let made = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&checkout)
        .output()
        .expect("git starts");

    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );

    folder
}

/// A hand-off file of format 1 that the analyzer can use, for the checkout in the folder given.
pub fn document(folder: &Path) -> Value {
    let text = |path: &Path| {
        path.to_str()
            .expect("the test's folder is valid UTF-8")
            .to_owned()
    };

    json!({
        "formatVersion": 1,
        "root": text(&folder.join("checkout")),
        "detection": { "folders": [".", "packages/app"], "truncated": false },
        "stacks": [
            {
                "name": "typescript",
                "interfaceVersion": 1,
                "pack": {
                    "data": ["**/fixtures/**"],
                    "cyclomaticLimit": 20,
                    "configEntries": [{ "file": "runner.config.*", "property": "include" }]
                },
                "project": { "generated": ["src/generated/**"], "internal": ["packages/core"] }
            },
            { "name": "bare", "pack": {}, "project": {} }
        ],
        "siblings": text(folder)
    })
}

/// Writes a document as the hand-off file in the folder, beside the checkout, and returns the file's
/// path as text.
pub fn written(folder: &Path, document: &Value) -> String {
    let file = folder.join("hand-off.json");

    fs::write(&file, document.to_string()).expect("the hand-off file can be written");

    file.to_str()
        .expect("the test's folder is valid UTF-8")
        .to_owned()
}
