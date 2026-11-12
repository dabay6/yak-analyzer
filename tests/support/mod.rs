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

/// A new, empty folder of one test's own, under cargo's folder for the files tests make.
pub fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);

    // The folder is left over from an earlier run, or is not there: either way it is made new.
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the test's folder can be made");

    folder
}

/// A hand-off file of format 1 that the analyzer can use, for a checkout in the folder given.
pub fn document(folder: &Path) -> Value {
    let place = folder.to_str().expect("the test's folder is valid UTF-8");

    json!({
        "formatVersion": 1,
        "root": place,
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
        "siblings": place
    })
}

/// Writes a document as the hand-off file in the folder, and returns the file's path as text.
pub fn written(folder: &Path, document: &Value) -> String {
    let file = folder.join("hand-off.json");

    fs::write(&file, document.to_string()).expect("the hand-off file can be written");

    file.to_str()
        .expect("the test's folder is valid UTF-8")
        .to_owned()
}
