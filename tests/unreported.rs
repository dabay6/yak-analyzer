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

/// The one run of the report `audit` prints for the checkout with the options given. The run must
/// succeed and write no line.
fn audited(project: &Project, options: &[&str]) -> Value {
    printed(&project.start("audit", |binary| {
        binary.args(options);
    }))["runs"][0]
        .clone()
}

/// The one run of the report `audit` prints for the checkout with the hand-off file given.
fn audit(project: &Project, hand_off: &str) -> Value {
    audited(project, &["--hand-off", hand_off])
}

/// The document `graph` prints for the checkout with the hand-off file given.
fn graphed(project: &Project, hand_off: &str) -> Value {
    printed(&project.run(|binary| {
        binary.args(["--hand-off", hand_off]);
    }))
}

/// The results of a run, in its order.
fn results(run: &Value) -> &[Value] {
    run["results"].as_array().expect("a list of results")
}

/// Where a result is: its file as the report writes it, and its line where it has one.
fn place(result: &Value) -> (&str, Option<u64>) {
    let place = &result["locations"][0]["physicalLocation"];

    (
        place["artifactLocation"]["uri"].as_str().expect("a file"),
        place["region"]["startLine"].as_u64(),
    )
}

/// The findings of a run, in its order: each with its rule, its file and its line.
fn found(run: &Value) -> Vec<(&str, &str, Option<u64>)> {
    results(run)
        .iter()
        .map(|result| {
            let (file, line) = place(result);

            (result["ruleId"].as_str().expect("a rule"), file, line)
        })
        .collect()
}

/// The findings of the rule given, in the run's order: each with its file, its line, and whether it is
/// new or inherited where the run says.
fn states<'r>(run: &'r Value, rule: &str) -> Vec<(&'r str, Option<u64>, Option<&'r str>)> {
    results(run)
        .iter()
        .filter(|result| result["ruleId"] == json!(rule))
        .map(|result| {
            let (file, line) = place(result);

            (file, line, result["baselineState"].as_str())
        })
        .collect()
}

/// The finding of the rule given at the file given.
fn at<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r Value {
    results(run)
        .iter()
        .find(|result| result["ruleId"] == json!(rule) && place(result).0 == uri)
        .unwrap_or_else(|| panic!("a finding of {rule} at {uri}"))
}

/// The message of the finding of the rule given at the file given.
fn message<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r str {
    at(run, rule, uri)["message"]["text"]
        .as_str()
        .expect("a message")
}

/// The fingerprint of the finding of the rule given at the file given.
fn fingerprint<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r str {
    at(run, rule, uri)["partialFingerprints"]["yakAnalyzer/v1"]
        .as_str()
        .expect("a fingerprint")
}

/// The sentences of a run's notifications of the kind given, in its order.
fn sentences<'r>(run: &'r Value, kind: &str) -> Vec<&'r str> {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .filter(|notification| notification["descriptor"]["id"] == json!(kind))
        .map(|notification| {
            notification["message"]["text"]
                .as_str()
                .expect("a sentence")
        })
        .collect()
}

/// The reasons `graph` gives for the entry point at the path given; null where it is no entry point.
fn reasons(document: &Value, path: &str) -> Value {
    document["entries"]
        .as_array()
        .expect("a list of entry points")
        .iter()
        .find(|entry| entry["path"] == json!(path))
        .map_or(Value::Null, |entry| entry["sources"].clone())
}

/// The one reason the `unreported` declaration of the stack `typescript` gives an entry point.
fn unreported_reason() -> Value {
    json!([{ "by": "project", "file": null, "from": "unreported", "stack": "typescript" }])
}

#[test]
fn an_unreported_file_is_read_and_reaches_what_it_imports_but_nothing_is_reported_in_it() {
    let project = Project::new("unreported-dead-code");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import { used } from './lib';\n\nused();\n")
        .write(
            "src/lib.ts",
            "export const used = (): void => {};\nexport const helper = (): void => {};\n",
        )
        .write(
            "scripts/report.ts",
            "import { helper } from '../src/lib';\nimport { only } from './only';\nimport { gone } from \
             './missing';\n\nhelper();\nonly();\ngone();\n\nexport const spare = 1;\n",
        )
        .write(
            "scripts/only.ts",
            "export const only = (): void => {};\nexport const extra = 2;\n",
        )
        .commit();

    let hand_off = declaring(&project, json!({ "unreported": ["scripts/report.ts"] }));
    let run = audit(&project, &hand_off);

    // `helper` and `only` are used by the unreported script, an entry point; `extra`, which nothing
    // imports, is now reported, and nothing in the script is (ignoring spec READ-2, DEAD-1).
    assert_eq!(found(&run), [("unused-export", "scripts/only.ts", Some(2))]);
    // Without the declaration the run would report the script and `only.ts` as unused files, and the
    // script's import of `./missing`; `helper` is used even then, since an import in any file read is a
    // use. The two in the script count against its glob, and `only.ts` in the notice for the rest
    // (ignoring spec COUNT-1, COUNT-2).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "1 more finding(s) outside the `unreported` files are not reported, since only those files \
             use what they name.",
            "The `unreported` glob `scripts/report.ts` that the project states for the stack `typescript` \
             left out 2 finding(s).",
        ]
    );
    // `graph` lists the script as an entry point with the reason `unreported` (ignoring spec COUNT-5).
    assert_eq!(
        reasons(&graphed(&project, &hand_off), "scripts/report.ts"),
        unreported_reason()
    );
}

#[test]
fn an_unreported_files_re_exports_pass_no_use_on_unless_something_else_starts_it() {
    let project = Project::new("unreported-re-exports");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export { kept } from './kept';\n")
        .write("src/kept.ts", "export const kept = 1;\n")
        .write("scripts/barrel.ts", "export { lone } from '../src/lone';\n")
        .write("src/lone.ts", "export const lone = 1;\n")
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "unreported": ["src/index.ts", "scripts/**"] }),
    );

    // The package's `main` still uses every export of `src/index.ts`, so `kept` is used through it; the
    // unreported script alone starts nothing, so `lone`, which only it re-exports, is unused (ignoring
    // spec READ-2).
    assert_eq!(
        found(&audit(&project, &hand_off)),
        [("unused-export", "src/lone.ts", Some(1))]
    );
}

#[test]
fn a_cycle_is_left_out_only_when_every_file_in_it_is_unreported() {
    let project = Project::new("unreported-cycles");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import './c';\nimport '../scripts/x';\n")
        .write("src/c.ts", "import '../scripts/z';\nexport const c = 1;\n")
        .write("scripts/z.ts", "import '../src/c';\nexport const z = 1;\n")
        .write("scripts/x.ts", "import './y';\nexport const x = 1;\n")
        .write("scripts/y.ts", "import './x';\nexport const y = 1;\n")
        .write("gen/p.ts", "import './q';\n")
        .write("gen/q.ts", "import './p';\n")
        .commit();

    let plain = audit(
        &project,
        &declaring(&project, json!({ "generated": ["gen/**"] })),
    );
    let hand_off = declaring(
        &project,
        json!({ "generated": ["gen/**"], "unreported": ["scripts/**", "gen/**"] }),
    );
    let run = audit(&project, &hand_off);

    // The cycle of `src/c.ts` and `scripts/z.ts` is reported at its first file that is not unreported,
    // with the fingerprint it has without the declaration; the cycles of unreported files alone are
    // left out, generated ones included (ignoring spec CYCLE-1, KEY-8).
    assert_eq!(
        found(&run),
        [
            ("unused-export", "src/c.ts", Some(2)),
            ("circular-dependency", "src/c.ts", Some(1)),
        ]
    );
    assert_eq!(
        message(&run, "circular-dependency", "src/c.ts"),
        "These files import each other in a loop: `src/c.ts` → `scripts/z.ts` → `src/c.ts`."
    );
    assert_eq!(
        fingerprint(&run, "circular-dependency", "src/c.ts"),
        fingerprint(&plain, "circular-dependency", "scripts/z.ts")
    );
    // `scripts/**` left out the exports of `x`, `y` and `z` and the cycle of `x` and `y`; `gen/**` the
    // cycle of `p` and `q`, which `generated` alone does not leave out (ignoring spec COUNT-2).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "The `unreported` glob `gen/**` that the project states for the stack `typescript` left out \
             1 finding(s).",
            "The `unreported` glob `scripts/**` that the project states for the stack `typescript` left \
             out 4 finding(s).",
        ]
    );
    // A file both generated and unreported is an entry point (ignoring spec KEY-8).
    assert_eq!(
        reasons(&graphed(&project, &hand_off), "gen/p.ts"),
        unreported_reason()
    );
}

#[test]
fn each_glob_counts_the_findings_it_left_out_once_however_many_stacks_state_it() {
    let project = Project::new("unreported-counts");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import { a } from './a';\n\na();\n")
        .write(
            "src/a.ts",
            "export const a = (): void => {};\nexport const spare = 1;\n",
        )
        .write("docs/note.ts", "export const note = 1;\n")
        .write("fixtures/sample.ts", "export const sample = 1;\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([
            {
                "name": "typescript",
                "interfaceVersion": 1,
                "pack": { "data": ["fixtures/**"] },
                "project": {
                    "unreported": [
                        "src/a.ts",
                        "src/*.ts",
                        "src/index.ts",
                        "docs/**",
                        "fixtures/**",
                        "nothing/**"
                    ]
                }
            },
            {
                "name": "javascript",
                "interfaceVersion": 1,
                "pack": {},
                "project": { "unreported": ["src/a.ts"] }
            }
        ]),
    );
    let run = audit(&project, &hand_off);

    // `spare` and the unused `docs/note.ts` are left out, and nothing is reported (ignoring spec
    // DEAD-1).
    assert!(found(&run).is_empty());
    // Two globs that match one file each count its finding; a glob that two stacks state is one notice;
    // a glob with no effect, or matching only a data file, says 0; a glob that matches nothing gives
    // its own notice instead (ignoring spec COUNT-1, COUNT-2, KEY-8).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "The `unreported` glob `docs/**` that the project states for the stack `typescript` left out \
             1 finding(s).",
            "The `unreported` glob `fixtures/**` that the project states for the stack `typescript` left \
             out 0 finding(s).",
            "The `unreported` glob `src/*.ts` that the project states for the stack `typescript` left out \
             1 finding(s).",
            "The `unreported` glob `src/a.ts` that the project states for the stacks `typescript`, \
             `javascript` left out 1 finding(s).",
            "The `unreported` glob `src/index.ts` that the project states for the stack `typescript` left \
             out 0 finding(s).",
        ]
    );
    assert_eq!(
        sentences(&run, "glob-matches-nothing"),
        [
            "The `unreported` glob `nothing/**` that the project states for the stack `typescript` \
          matches no file."
        ]
    );
    // A data file is not read, so it is no entry point (ignoring spec KEY-8).
    assert_eq!(
        reasons(&graphed(&project, &hand_off), "fixtures/sample.ts"),
        Value::Null
    );
}

#[test]
fn an_unreported_entry_point_leaves_a_packages_fault_and_its_files_out_of_the_notices() {
    let project = Project::new("unreported-packages");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .write("tools/package.json", r#"{ "name": "tools" }"#)
        .write("tools/run.ts", "export {};\n")
        .write("tools/left.ts", "export {};\n")
        .write("extra/package.json", r#"{ "name": "extra" }"#)
        .write("extra/only.ts", "export {};\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &[".", "tools"],
        stack(json!({ "unreported": ["tools/run.ts", "extra/**"] })),
    );
    let run = audit(&project, &hand_off);

    // `tools/run.ts` is the package's only entry point, by the reason `unreported` alone, so the package
    // keeps its fault and the unused file its hint (ignoring spec READ-2).
    assert_eq!(found(&run), [("unused-file", "tools/left.ts", None)]);
    assert_eq!(
        message(&run, "unused-file", "tools/left.ts"),
        "No entry point reaches this file through any import, so it can be removed. Its package `tools` \
         has no entry point at all, so the likely fault is a missing `entries` declaration."
    );
    // The package `extra` holds only unreported files, so it gives no notice that its kinds are not
    // reported (ignoring spec READ-3).
    assert!(sentences(&run, "kinds-not-reported").is_empty());
}

#[test]
fn with_a_base_a_new_cycle_moves_only_to_an_added_import_in_a_file_that_is_not_unreported() {
    let project = Project::new("unreported-base");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import './c';\nimport './d';\n")
        .write("src/c.ts", "import '../scripts/z';\nexport const c = 1;\n")
        .write("scripts/z.ts", "import '../src/e';\nexport const z = 1;\n")
        .write("src/e.ts", "export const e = 1;\n")
        .write("src/d.ts", "import '../scripts/w';\nexport const d = 1;\n")
        .write("scripts/w.ts", "export const w = 1;\n")
        .write("docs/old.ts", "export const old = 1;\n")
        .commit();
    project.git(&["switch", "--quiet", "-c", "branch"]);
    project
        .write("src/e.ts", "import './c';\nexport const e = 1;\n")
        .write(
            "scripts/z.ts",
            "import '../src/e';\nimport '../src/c';\nexport const z = 1;\n",
        )
        .write("scripts/w.ts", "import '../src/d';\nexport const w = 1;\n");

    let hand_off = declaring(&project, json!({ "unreported": ["scripts/**", "docs/**"] }));
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // The branch added an import in `scripts/z.ts`, first by path, and one in `src/e.ts`: the new cycle
    // moves to the second. The other's only added import is in `scripts/w.ts`, so it keeps its place
    // at `src/d.ts` (ignoring spec CYCLE-1).
    assert_eq!(
        states(&run, "circular-dependency"),
        [
            ("src/d.ts", Some(1), Some("new")),
            ("src/e.ts", Some(1), Some("new")),
        ]
    );
    assert!(message(&run, "circular-dependency", "src/e.ts").ends_with(
        " The branch added the import of `./c` on line 1 of `src/e.ts`, which leads to `src/c.ts`."
    ));
    assert!(message(&run, "circular-dependency", "src/d.ts").ends_with(
        " The branch added the import of `../src/d` on line 1 of `scripts/w.ts`, which leads to \
             `src/d.ts`."
    ));
    // `docs/old.ts` is left out on both sides, and only the branch's finding is counted (ignoring spec
    // COUNT-3).
    assert!(sentences(&run, "findings-unreported").contains(
        &"The `unreported` glob `docs/**` that the project states for the stack `typescript` left out 1 \
          finding(s)."
    ));
}
