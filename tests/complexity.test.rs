//! Runs the built binary's `complexity` on made-up projects and checks the document it prints: the
//! limits it reads, the functions it finds and how it names them, and how it stops (complexity spec
//! FN-1 to FN-5, LIMIT-1, LIMIT-2, CPX-4, CPX-5, SOLO-1 to SOLO-4, CASE-1).

#[expect(
    dead_code,
    reason = "the shared project helpers include running `graph` and reading a document's paths, which these tests do not need"
)]
#[path = "support/project.rs"]
mod project;

use std::collections::BTreeMap;

use serde_json::{Value, json};

use project::{Project, notices, printed, stopped};

/// The document `complexity` prints for the checkout with the options given. The run must succeed
/// and write no line.
fn complexity(project: &Project, options: &[&str]) -> Value {
    printed(&project.start("complexity", |binary| {
        binary.args(options);
    }))
}

/// A hand-off file for the checkout, with one stack whose pack and project declare what is given.
fn declaring(project: &Project, pack: &Value, own: &Value) -> String {
    project.hand_off(
        &project.root,
        &["."],
        json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": own }]),
    )
}

/// The functions a document lists, in its order: each one's file and readable name path.
fn listed(document: &Value) -> Vec<(&str, &str)> {
    document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .map(|function| {
            (
                function["path"].as_str().expect("a path"),
                function["function"].as_str().expect("a name path"),
            )
        })
        .collect()
}

/// The readable name paths of the functions a document lists in the file given, in its order.
fn names<'d>(document: &'d Value, path: &str) -> Vec<&'d str> {
    listed(document)
        .into_iter()
        .filter(|(file, _)| *file == path)
        .map(|(_, name)| name)
        .collect()
}

/// The fingerprints of the functions a document lists in the file given, by their readable name
/// paths.
fn fingerprints<'d>(document: &'d Value, path: &str) -> BTreeMap<&'d str, &'d str> {
    document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .filter(|function| function["path"] == json!(path))
        .map(|function| {
            (
                function["function"].as_str().expect("a name path"),
                function["fingerprint"].as_str().expect("a fingerprint"),
            )
        })
        .collect()
}

/// The sentence of the first notice of the kind given that a document holds.
fn sentence<'d>(document: &'d Value, kind: &str) -> &'d str {
    document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .find(|notice| notice["kind"] == json!(kind))
        .and_then(|notice| notice["sentence"].as_str())
        .unwrap_or_default()
}

#[test]
fn with_no_limit_declared_it_lists_no_function_and_says_so() {
    let project = Project::new("complexity-no-limit");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let document = complexity(&project, &[]);

    assert_eq!(document["interfaceVersion"], json!(1));
    assert_eq!(
        document["limits"],
        json!({ "cyclomatic": null, "cognitive": null })
    );
    assert_eq!(document["functions"], json!([]));
    assert_eq!(
        notices(&document),
        ["complexity-not-checked", "no-hand-off-file"]
    );
    assert!(
        sentence(&document, "complexity-not-checked")
            .contains("`cyclomaticLimit` or `cognitiveLimit`")
    );

    // With `--all`, every function is listed, over nothing (spec SOLO-3).
    let document = complexity(&project, &["--all"]);
    let function = &document["functions"][0];
    let fingerprint = function["fingerprint"].as_str().expect("a fingerprint");

    assert_eq!(fingerprint.len(), 32);
    assert!(fingerprint.chars().all(|digit| digit.is_ascii_hexdigit()));
    assert_eq!(
        document["functions"],
        json!([{
            "path": "src/index.ts",
            "line": 1,
            "column": 20,
            "function": "run",
            "fingerprint": fingerprint,
            "cyclomatic": 1,
            "cognitive": 0,
            "over": [],
        }])
    );
}

#[test]
fn the_project_s_limit_wins_over_the_pack_s_and_a_later_stack_s_over_an_earlier_one() {
    let project = Project::new("complexity-limits");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let stack = |name: &str, pack: Value, own: Value| json!({ "name": name, "interfaceVersion": 1, "pack": pack, "project": own });
    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([
            stack(
                "first",
                json!({ "cyclomaticLimit": 5 }),
                json!({ "cognitiveLimit": 9 })
            ),
            stack(
                "second",
                json!({ "cyclomaticLimit": 7, "cognitiveLimit": 3 }),
                json!({})
            ),
        ]),
    );
    let document = complexity(&project, &["--hand-off", &hand_off]);

    assert_eq!(
        document["limits"],
        json!({ "cyclomatic": 7, "cognitive": 9 })
    );
    assert!(!notices(&document).contains(&"complexity-not-checked"));
}

#[test]
fn with_one_limit_declared_only_its_measure_is_checked_with_a_notice() {
    let project = Project::new("complexity-one-limit");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let hand_off = declaring(&project, &json!({ "cognitiveLimit": 15 }), &json!({}));
    let document = complexity(&project, &["--hand-off", &hand_off]);

    assert_eq!(
        document["limits"],
        json!({ "cyclomatic": null, "cognitive": 15 })
    );
    assert!(
        sentence(&document, "complexity-not-checked")
            .starts_with("No `cyclomaticLimit` is declared")
    );
}

#[test]
fn a_score_equal_to_its_limit_is_not_over_it() {
    let project = Project::new("complexity-equal");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let hand_off = declaring(
        &project,
        &json!({ "cyclomaticLimit": 1, "cognitiveLimit": 1 }),
        &json!({}),
    );

    assert_eq!(
        complexity(&project, &["--hand-off", &hand_off])["functions"],
        json!([])
    );
    assert_eq!(
        complexity(&project, &["--all", "--hand-off", &hand_off])["functions"][0]["over"],
        json!([])
    );
}

#[test]
fn names_each_function_by_what_holds_it_and_numbers_the_repeats() {
    let project = Project::new("complexity-names");

    project
        .write(
            "src/names.ts",
            r#"export function read() {}
export function load(a: string): void;
export function load(a: number): void;
export function load(a: unknown): void {}
const parse = () => {};
const object = {
  get value() { return 1; },
  set value(v: number) {},
  run() {},
  "": () => 0,
  'read #2': () => 0,
  [Symbol.iterator]: () => 0,
  data: { size: 1 },
};
const first = { go() {} };
const second = { go() {} };
a.b.read = function () {};
c.b.read = function () {};
a.b.read = function () {};
{ const local = () => {}; }
{ const local = () => {}; }
const Parser = class { read() {} };
items.map((item) => item);
items.map((item) => item);
it("reads a file", () => {});
it(`reads a file`, () => {});
promise.then(() => 1).catch(() => 2);
describe.each(rows)("x", () => {});
new Promise((resolve) => resolve(1));
"#,
        )
        .write(
            "src/component.ts",
            "export default defineComponent({ setup() {} });\n",
        )
        .write("src/anonymous.ts", "export default function () {}\n")
        .write("src/settings.ts", "export default { run() {} };\n")
        .commit();

    let document = complexity(&project, &["--all"]);

    // Overload signatures take no place, and an object literal with no function in it is passed
    // over (spec FN-1, FN-5).
    assert_eq!(
        names(&document, "src/names.ts"),
        [
            "read",
            "load",
            "parse",
            "object > get value",
            "object > set value",
            "object > run",
            "object > ",
            "object > read #2",
            "object > <anonymous 1>",
            "first > go",
            "second > go",
            "a.b.read",
            "c.b.read",
            "a.b.read #2",
            "local",
            "local #2",
            "Parser > read",
            "items.map()",
            "items.map() #2",
            "it(\"reads a file\")",
            "it(\"reads a file\") #2",
            "promise.then()",
            "promise.then().catch()",
            "describe.each()(\"x\")",
            "new Promise()",
        ]
    );
    assert_eq!(
        names(&document, "src/component.ts"),
        ["defineComponent() > setup"]
    );
    assert_eq!(names(&document, "src/anonymous.ts"), ["default"]);
    assert_eq!(names(&document, "src/settings.ts"), ["default > run"]);
}

#[test]
fn places_a_function_at_its_own_name_or_else_at_its_first_character() {
    let project = Project::new("complexity-places");

    project
        .write(
            "src/places.ts",
            "export function own() {}\nconst arrow = () => {};\nclass Holder {\n  method() {}\n}\nconst wrapped = (() => {});\n",
        )
        .commit();

    let document = complexity(&project, &["--all"]);
    let places: Vec<(&str, u64, u64)> = document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .map(|function| {
            (
                function["function"].as_str().expect("a name path"),
                function["line"].as_u64().expect("a line"),
                function["column"].as_u64().expect("a column"),
            )
        })
        .collect();

    assert_eq!(
        places,
        [
            ("own", 1, 17),
            ("arrow", 2, 15),
            ("Holder > method", 4, 3),
            ("wrapped", 6, 18),
        ]
    );
}

#[test]
fn a_vue_component_s_script_blocks_are_one_file_and_an_mdx_document_is_not_scored() {
    let project = Project::new("complexity-vue");

    project
        .write(
            "src/App.vue",
            "<script lang=\"ts\">\nexport function shared() {}\n</script>\n\n<script setup lang=\"ts\">\nfunction shared() {}\nconst count = computed(() => 1);\n</script>\n<template><p>{{ count }}</p></template>\n",
        )
        .write(
            "src/Inline.vue",
            "<script>export const inline = () => 1;</script>\n",
        )
        .write("docs/page.mdx", "export function shown() { return 1; }\n\n# Title\n")
        .commit();

    let document = complexity(&project, &["--all"]);
    let places: Vec<(&str, &str, u64, u64)> = document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .map(|function| {
            (
                function["path"].as_str().expect("a path"),
                function["function"].as_str().expect("a name path"),
                function["line"].as_u64().expect("a line"),
                function["column"].as_u64().expect("a column"),
            )
        })
        .collect();

    assert_eq!(
        places,
        [
            ("src/App.vue", "shared", 2, 17),
            ("src/App.vue", "shared #2", 6, 10),
            ("src/App.vue", "computed()", 7, 24),
            ("src/Inline.vue", "inline", 1, 31),
        ]
    );
}

#[test]
fn leaves_out_data_generated_and_type_declaration_files() {
    let project = Project::new("complexity-left-out");

    project
        .write("src/index.ts", "export function kept() {}\n")
        .write("src/generated/made.ts", "export function made() {}\n")
        .write("src/fixtures/sample.ts", "export function sample() {}\n")
        .write("src/types.d.ts", "export declare function typed(): void;\n")
        .commit();

    let hand_off = declaring(
        &project,
        &json!({ "data": ["src/fixtures/**"], "generated": ["src/generated/**"] }),
        &json!({}),
    );
    let document = complexity(&project, &["--hand-off", &hand_off, "--all"]);

    assert_eq!(listed(&document), [("src/index.ts", "kept")]);
}

#[test]
fn a_fingerprint_holds_no_line_and_no_formatting_and_tells_names_apart() {
    let project = Project::new("complexity-fingerprints");

    project
        .write(
            "src/calls.ts",
            "items.map((item) => item);\nit('reads a file', () => {});\na.b.read = function () {};\n",
        )
        .write(
            "src/holder.js",
            "export const holder = { read() {}, read: () => 0, 'read #2': () => 0 };\n",
        )
        .commit();

    let before = complexity(&project, &["--all"]);

    // A formatter's rewrap, a comment, a quote change and optional chaining move no fingerprint (spec
    // FN-4).
    project.write(
        "src/calls.ts",
        "// A comment moves every line.\nitems\n  ?.map((item) => item);\nit(\"reads a file\", () => {});\na\n  .b /* a comment */ .read = function () {};\n",
    );

    let after = complexity(&project, &["--all"]);

    assert_eq!(
        fingerprints(&before, "src/calls.ts"),
        fingerprints(&after, "src/calls.ts")
    );
    assert_eq!(fingerprints(&after, "src/calls.ts").len(), 3);

    // The second `read` and the key `'read #2'` read alike and are told apart (spec CPX-4).
    let holder: Vec<&str> = after["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .filter(|function| function["path"] == json!("src/holder.js"))
        .map(|function| function["fingerprint"].as_str().expect("a fingerprint"))
        .collect();

    assert_eq!(
        names(&after, "src/holder.js"),
        ["holder > read", "holder > read #2", "holder > read #2"]
    );
    assert_ne!(holder[1], holder[2]);
}

#[test]
fn refuses_a_command_line_it_does_not_answer_and_stops_on_a_hand_off_file_it_cannot_read() {
    let project = Project::new("complexity-refusals");

    project.write("src/index.ts", "export {};\n").commit();

    for options in [
        &["--all", "--all"][..],
        &["--hand-off"],
        &["--base", "main"],
        &["--hand-off", "a.json", "--hand-off", "b.json"],
    ] {
        let output = project.start("complexity", |binary| {
            binary.args(options);
        });
        let line = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{options:?}");
        assert!(output.stdout.is_empty());
        assert!(
            line.contains("complexity [--hand-off <file>] [--all]"),
            "{line}"
        );
    }

    let missing = project.home.join("missing.json");
    let missing = missing.to_str().expect("the test's folder is valid UTF-8");
    let line = stopped(&project.start("complexity", |binary| {
        binary.args(["--hand-off", missing]);
    }));

    assert!(line.contains("cannot be read"), "{line}");
}
