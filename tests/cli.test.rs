//! Starts the built binary and checks its commands and how it refuses (spec RUN-1, RUN-2, RUN-3, RUN-7;
//! dead-code spec CMD-1, CMD-3, CMD-5).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs the built binary with the arguments given, without the timing switch a developer's environment
/// may set (spec TEST-1a).
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .env_remove("YAK_ANALYZER_TIMINGS")
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
    // Without a hand-off file the root is the folder the binary is started in: an empty repository here.
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-graph");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("the test's folder can be made");
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git starts")
            .success()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .arg("graph")
        .env_remove("YAK_ANALYZER_TIMINGS")
        .current_dir(&root)
        .output()
        .expect("the binary starts");
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
fn audit_without_a_hand_off_file_prints_one_sarif_report_and_exits_zero() {
    // An empty repository holds nothing to find: the report still names the tool and lists the rules.
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-audit");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("the test's folder can be made");
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git starts")
            .success()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .arg("audit")
        .current_dir(&root)
        .output()
        .expect("the binary starts");
    let report: Value = serde_json::from_slice(&output.stdout).expect("audit prints JSON");
    let run = &report["runs"][0];
    let notifications = &run["invocations"][0]["toolExecutionNotifications"];

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(report["version"], json!("2.1.0"));
    assert_eq!(report["runs"].as_array().map(Vec::len), Some(1));
    assert_eq!(run["tool"]["driver"]["name"], json!("yak-analyzer"));
    assert_eq!(
        run["tool"]["driver"]["rules"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(run["results"], json!([]));
    assert_eq!(notifications.as_array().map(Vec::len), Some(1));
    assert_eq!(
        notifications[0]["descriptor"]["id"],
        json!("no-hand-off-file")
    );
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    // `graph` takes no base, and `audit` takes each option once, with its value.
    let cases: [&[&str]; 10] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
        &["interface", "extra"],
        &["graph", "extra"],
        &["graph", "--base", "main"],
        &["audit", "extra"],
        &["audit", "--base"],
        &["audit", "--base", "main", "--base", "main"],
        &["audit", "--hand-off", "a.json", "--hand-off", "b.json"],
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
            error.contains("--version, interface, graph [--hand-off <file>], and audit"),
            "{args:?}"
        );
    }
}

#[test]
fn refuses_a_base_git_would_read_as_an_option_before_git_sees_it() {
    // A base that begins with `-`, such as one of git's options that writes a file, or that holds a
    // control character, or that is empty, is refused as a wrong option is, before any git starts
    // (dead-code spec CMD-2).
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-base-refused");
    let _ = fs::remove_dir_all(&folder);

    fs::create_dir_all(&folder).expect("the test's folder can be made");

    for base in ["--output=written.txt", "-", "main\nother", ""] {
        let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
            .args(["audit", "--base", base])
            .current_dir(&folder)
            .output()
            .expect("the binary starts");
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{base:?}");
        assert!(output.stdout.is_empty(), "{base:?}");
        assert_eq!(error.lines().count(), 1, "{base:?}");
        assert!(error.starts_with("yak-analyzer: the base `"), "{error}");
        assert!(error.contains("name the commit another way"), "{error}");
    }

    assert_eq!(
        fs::read_dir(&folder)
            .expect("the folder can be read")
            .count(),
        0
    );
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version, interface, graph [--hand-off <file>], and audit"));
}

#[test]
fn keeps_a_refusal_on_one_line_when_an_argument_holds_a_line_break() {
    let output = run(&["first\nsecond"]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("first\\nsecond"));
}
