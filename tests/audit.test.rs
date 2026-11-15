//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The copy of SARIF 2.1.0's schema. `tests/sarif/README.md` says where it came from.
const SCHEMA: &str = include_str!("sarif/sarif-schema-2.1.0.json");

/// The one run of the report `audit` prints for the checkout, with the hand-off file given or with
/// none. The run must succeed and write no line, and the schema must accept the report (spec FIX-3).
fn audit(project: &Project, hand_off: Option<&str>) -> Value {
    let report = printed(&project.start("audit", |binary| {
        if let Some(file) = hand_off {
            binary.args(["--hand-off", file]);
        }
    }));
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema's copy is JSON");
    let faults: Vec<String> = jsonschema::validator_for(&schema)
        .expect("the schema's copy is a JSON Schema")
        .iter_errors(&report)
        .map(|fault| format!("{}: {fault}", fault.instance_path()))
        .collect();

    assert_eq!(faults, Vec::<String>::new());
    assert_eq!(report["version"], json!("2.1.0"));
    assert_eq!(report["runs"].as_array().map(Vec::len), Some(1));

    report["runs"][0].clone()
}

/// The notifications of a run, in its order: each with its kind and the file it is about, as the
/// report writes it.
fn notifications(run: &Value) -> Vec<(&str, Option<&str>)> {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .map(|notification| {
            (
                notification["descriptor"]["id"].as_str().expect("a kind"),
                notification["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str(),
            )
        })
        .collect()
}

#[test]
fn prints_one_report_that_names_the_tool_the_interface_version_and_the_five_rules() {
    let project = Project::new("audit-report");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { helper } from './helper';\n\nhelper();\n",
        )
        .write("src/helper.ts", "export const helper = (): void => {};\n")
        .commit();

    let run = audit(&project, None);
    let driver = &run["tool"]["driver"];
    let rules: Vec<&str> = driver["rules"]
        .as_array()
        .expect("a list of rules")
        .iter()
        .map(|rule| rule["id"].as_str().expect("a name"))
        .collect();

    assert_eq!(driver["name"], json!("yak-analyzer"));
    assert_eq!(driver["version"], json!(env!("CARGO_PKG_VERSION")));
    assert_eq!(run["properties"]["interfaceVersion"], json!(1));
    assert_eq!(
        rules,
        [
            "unused-file",
            "unused-export",
            "unused-type",
            "unresolved-import",
            "circular-dependency",
        ]
    );
    assert_eq!(run["invocations"][0]["executionSuccessful"], json!(true));
    // Every file here is reached, every export is used, and every import leads to a file.
    assert_eq!(run["results"], json!([]));
}

#[test]
fn the_notices_of_a_run_are_the_notifications_of_its_invocation() {
    let project = Project::new("audit-notices");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .commit()
        // An untracked script that does not parse is skipped, with a notice about the file.
        .write("src/not done.ts", "const = ;\n");

    let run = audit(&project, None);
    let document = project.graph();

    // The notices are the ones `graph` prints, in its order, and a file is named by its path from the
    // root, written as a URI (spec REPORT-1, REPORT-4).
    assert_eq!(
        notifications(&run),
        [
            ("no-hand-off-file", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
    assert_eq!(
        notifications(&run)
            .iter()
            .map(|(kind, _)| *kind)
            .collect::<Vec<&str>>(),
        notices(&document)
    );
    assert!(paths(&document).contains(&"src/not done.ts"));

    // With a hand-off file, its declarations are read, and what they give notice of is in the report.
    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([{
            "name": "typescript",
            "interfaceVersion": 1,
            "pack": { "data": ["**/fixtures/**"] },
            "project": {}
        }]),
    );

    assert_eq!(
        notifications(&audit(&project, Some(&hand_off))),
        [
            ("glob-matches-nothing", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
}

#[test]
fn stops_where_graph_stops_with_the_same_line() {
    let project = Project::new("audit-stops");

    project.write("src/broken.ts", "const = ;\n").commit();

    // A run that cannot go on prints one line on standard error and exits 1, as `graph` does (spec
    // CMD-3).
    let line = stopped(&project.start("audit", |_| {}));

    assert!(line.contains("src/broken.ts"), "{line}");
    assert_eq!(line, stopped(&project.run(|_| {})));
}
