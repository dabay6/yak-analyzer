//! Checks the hand-off files the tests use against the copy of yakshaver's schema, and that the analyzer
//! and the schema agree on files made wrong on purpose (spec HO-1).

mod support;

use serde_json::{Value, json};

use support::{document, folder, run, written};

/// The copy of the schema of hand-off format 1. `tests/hand-off/README.md` says where it came from.
const SCHEMA: &str = include_str!("hand-off/hand-off-1.schema.json");

/// Whether the schema accepts a document.
fn accepted(document: &Value) -> bool {
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema's copy is JSON");

    jsonschema::validator_for(&schema)
        .expect("the schema's copy is a JSON Schema")
        .is_valid(document)
}

#[test]
fn the_schema_accepts_every_hand_off_file_the_tests_use_and_so_does_the_analyzer() {
    let place = folder("schema-valid");
    let plain = document(&place);
    let mut with_base = plain.clone();
    let mut cut_short = plain.clone();
    let mut with_more = plain.clone();

    with_base["base"] = json!("0123456789abcdef0123456789abcdef01234567");
    cut_short["detection"]["truncated"] = json!(true);
    with_more["cache"] = json!({ "folder": "somewhere" });

    for (name, handed) in [
        ("plain", plain),
        ("with a base", with_base),
        ("cut short", cut_short),
        ("with a field the format does not know", with_more),
    ] {
        let output = run(&["graph", "--hand-off", &written(&place, &handed)]);

        assert!(accepted(&handed), "{name}");
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn the_schema_and_the_analyzer_both_refuse_a_file_made_wrong_on_purpose() {
    let place = folder("schema-invalid");
    let plain = document(&place);
    let mut no_root = plain.clone();
    let mut other_format = plain.clone();
    let mut more_in_detection = plain.clone();
    let mut no_folders = plain.clone();
    let mut more_in_a_stack = plain.clone();
    let mut odd_project_value = plain.clone();
    let mut short_base = plain.clone();

    no_root.as_object_mut().expect("an object").remove("root");
    other_format["formatVersion"] = json!(2);
    more_in_detection["detection"]["limit"] = json!(64);
    no_folders["detection"]["folders"] = json!([]);
    more_in_a_stack["stacks"][0]["tool"] = json!("analyzer");
    odd_project_value["stacks"][0]["project"]["data"] = json!({ "globs": [] });
    short_base["base"] = json!("abc123");

    for (name, handed) in [
        ("no root", no_root),
        ("another format", other_format),
        (
            "a field the format does not allow under detection",
            more_in_detection,
        ),
        ("no folders", no_folders),
        (
            "a field the format does not allow in a stack",
            more_in_a_stack,
        ),
        (
            "a project value that is neither a list nor a number",
            odd_project_value,
        ),
        ("a base that is not a commit's full name", short_base),
    ] {
        let output = run(&["graph", "--hand-off", &written(&place, &handed)]);

        assert!(!accepted(&handed), "{name}");
        assert_eq!(output.status.code(), Some(1), "{name}");
    }
}
