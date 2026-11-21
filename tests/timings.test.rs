//! Runs `graph` with the timing switch set and unset, and checks that it changes nothing but the one
//! line it adds on a run that succeeds (spec TIME-1 to TIME-5, TEST-1, TEST-2).

#[expect(
    dead_code,
    reason = "the shared project helpers include reading a document's paths and notices and writing a hand-off file, which these tests do not need"
)]
#[path = "support/project.rs"]
mod project;

use serde_json::Value;

use project::{Project, printed, stopped};

/// The variable that turns the timings on.
const VARIABLE: &str = "YAK_ANALYZER_TIMINGS";

/// The keys of a reading's stages (spec TIME-2, TIME-2b).
const READING: [&str; 11] = [
    "handOff",
    "files",
    "marks",
    "packages",
    "tsconfig",
    "scriptsRead",
    "scriptsParse",
    "resolve",
    "entries",
    "uses",
    "build",
];

/// A small project: a package, a tsconfig, and two scripts that import each other.
fn project(name: &str) -> Project {
    let project = Project::new(name);

    project
        .write("package.json", r#"{ "name": "app", "main": "src/a.ts" }"#)
        .write("tsconfig.json", r#"{ "compilerOptions": {} }"#)
        .write(
            "src/a.ts",
            "import { b } from './b';\n\nexport const a = b;\n",
        )
        .write(
            "src/b.ts",
            "import type { a } from './a';\n\nexport const b = 1;\n",
        )
        .commit();

    project
}

#[test]
fn set_to_1_it_adds_one_line_with_every_stage_and_changes_nothing_else() {
    let project = project("timings-set");
    let unset = project.run(|_| {});
    let set = project.run(|binary| {
        binary.env(VARIABLE, "1");
    });

    printed(&unset);
    assert_eq!(set.status.code(), unset.status.code());
    assert_eq!(set.stdout, unset.stdout);

    let error = String::from_utf8_lossy(&set.stderr);

    assert_eq!(error.lines().count(), 1, "{error}");

    let line: Value = serde_json::from_str(error.trim_end()).expect("the line is JSON");
    let timings = &line["timings"];

    for stage in READING {
        assert!(timings["reading"][stage].is_u64(), "{stage}: {line}");
    }

    assert_eq!(
        timings["reading"].as_object().map(serde_json::Map::len),
        Some(READING.len()),
        "{line}"
    );
    assert!(timings["print"].is_u64(), "{line}");
    assert!(timings["total"].is_u64(), "{line}");
    assert_eq!(
        line.as_object().map(serde_json::Map::len),
        Some(1),
        "{line}"
    );
}

#[test]
fn any_other_value_leaves_standard_error_empty() {
    let project = project("timings-other-value");

    for value in ["0", "true", "yes", ""] {
        printed(&project.run(|binary| {
            binary.env(VARIABLE, value);
        }));
    }
}

#[test]
fn a_run_that_stops_writes_its_one_stop_line_and_no_timings() {
    // The folder's name stays clear of the word the line is checked for, since the stop line names the path.
    let project = project("switch-on-stopped");
    let missing = project.home.join("no-such-hand-off.json");
    let output = project.run(|binary| {
        binary.arg("--hand-off").arg(&missing).env(VARIABLE, "1");
    });
    let line = stopped(&output);

    assert!(line.starts_with("yak-analyzer: "), "{line}");
    assert!(!line.contains("timings"), "{line}");
}
