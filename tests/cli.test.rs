//! Starts the built binary and checks what it answers (spec BIN-1, BIN-2).

use std::process::{Command, Output};

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
fn refuses_anything_else_with_one_line_on_standard_error() {
    let cases: [&[&str]; 3] = [&["analyze"], &["--version", "extra"], &["--help"]];

    for args in cases {
        let output = run(args);
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(error.lines().count(), 1, "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(error.contains("--version"), "{args:?}");
    }
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version"));
}
