//! Starts the built binary and checks its commands and how it refuses (spec RUN-1, RUN-2, RUN-3, RUN-7).

use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs the built binary with the arguments given.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .output()
        .expect("the binary starts")
}

#[test]
fn prints_its_version_on_standard_output_and_exits_zero() {
    let output = run(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("yak-analyzer {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn prints_its_interface_version_and_the_hand_off_formats_it_reads_as_one_line_of_json() {
    let output = run(&["interface"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"handOffFormats\":[1],\"interfaceVersion\":1}\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn graph_without_a_hand_off_file_prints_one_document_and_says_so_in_a_notice() {
    let output = run(&["graph"]);
    let document: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(document["handOff"], json!(false));
    assert_eq!(document["files"], json!([]));
    assert_eq!(document["packages"], json!([]));
    assert_eq!(document["notices"].as_array().map(Vec::len), Some(1));
    assert_eq!(document["notices"][0]["kind"], json!("no-hand-off-file"));
    assert_eq!(document["notices"][0]["file"], Value::Null);
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    let cases: [&[&str]; 6] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
        &["interface", "extra"],
        &["graph", "extra"],
        &["graph", "--base", "main"],
    ];

    for args in cases {
        let output = run(args);
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(error.lines().count(), 1, "{args:?}");
        assert!(error.starts_with("yak-analyzer: cannot do `"), "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(
            error.contains("--version, interface, and graph"),
            "{args:?}"
        );
    }
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version, interface, and graph"));
}

#[test]
fn keeps_a_refusal_on_one_line_when_an_argument_holds_a_line_break() {
    let output = run(&["first\nsecond"]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("first\\nsecond"));
}
