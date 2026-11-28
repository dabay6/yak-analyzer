//! Runs the built binary on made-up projects whose hand-off file states the `unreported` declaration,
//! and checks the files it names: that only a project may state it, that `graph` flags them, and that
//! no command reports a finding in them, each saying how many it left out (ignoring spec KEY-6 to
//! KEY-8, READ-1 to READ-3, DEAD-1, CPLX-1, DUPE-1, CYCLE-1, COUNT-1 to COUNT-5, TEST-1).

#[expect(
    dead_code,
    reason = "the shared project helpers include running `graph` with no hand-off file and reading a document's paths and notice kinds, which these tests do not need"
)]
#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, printed, stopped};

/// One stack named `typescript` whose project declares what is given.
fn stack(declared: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": {}, "project": declared }])
}

/// A hand-off file whose project declares what is given.
fn declaring(project: &Project, declared: Value) -> String {
    project.hand_off(&project.root, &["."], stack(declared))
}

/// The sentences of a document's notices of the kind given, in its order.
fn said<'d>(document: &'d Value, kind: &str) -> Vec<&'d str> {
    document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .filter(|notice| notice["kind"] == json!(kind))
        .map(|notice| notice["sentence"].as_str().expect("a sentence"))
        .collect()
}

#[test]
fn only_a_project_may_state_the_files_no_finding_is_reported_in() {
    let project = Project::new("unreported-pack");

    project.write("src/index.ts", "export {};\n").commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([{
            "name": "typescript",
            "interfaceVersion": 1,
            "pack": { "unreported": ["scripts/**"] },
            "project": {}
        }]),
    );

    // A pack's glob could not be taken back by a project, so a pack may not state the key (ignoring
    // spec KEY-7).
    assert_eq!(
        stopped(&project.run(|binary| {
            binary.args(["--hand-off", &hand_off]);
        })),
        "yak-analyzer: cannot use the declaration `unreported` that the pack states for the stack \
         `typescript`: only a project may state this key; change it in the pack"
    );
}

#[test]
fn graph_flags_each_file_an_unreported_glob_matches() {
    let project = Project::new("unreported-flags");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .write("scripts/tool.ts", "export {};\n")
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "unreported": ["scripts/**", "nothing/**"] }),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &hand_off]);
    }));
    let flagged: Vec<&str> = document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .filter(|file| file["unreported"] == json!(true))
        .map(|file| file["path"].as_str().expect("a path"))
        .collect();

    // Each file a glob matches is flagged, beside its `data` and `generated` flags (ignoring spec
    // KEY-6, COUNT-5).
    assert_eq!(flagged, ["scripts/tool.ts"]);
    // A glob that matches no file gives the notice a `data` or `generated` glob gives (ignoring spec
    // KEY-6).
    assert_eq!(
        said(&document, "glob-matches-nothing"),
        [
            "The `unreported` glob `nothing/**` that the project states for the stack `typescript` \
          matches no file."
        ]
    );
}
