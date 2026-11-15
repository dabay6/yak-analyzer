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

/// One stack named `typescript` that holds the declarations given, the pack's and the project's.
fn stack(pack: Value, project: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": project }])
}

/// The findings of a run, in its order: each with its rule, its file as the report writes it, and its
/// line where it has one.
fn found(run: &Value) -> Vec<(&str, &str, Option<u64>)> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .map(|result| {
            let place = &result["locations"][0]["physicalLocation"];

            (
                result["ruleId"].as_str().expect("a rule"),
                place["artifactLocation"]["uri"].as_str().expect("a file"),
                place["region"]["startLine"].as_u64(),
            )
        })
        .collect()
}

/// The message of the finding of the rule given at the file given.
fn message<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r str {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .find(|result| {
            result["ruleId"] == json!(rule)
                && result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    == json!(uri)
        })
        .and_then(|result| result["message"]["text"].as_str())
        .unwrap_or_else(|| panic!("a finding of {rule} at {uri}"))
}

/// The fingerprints of a run's findings, in its order.
fn fingerprints(run: &Value) -> Vec<&str> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .map(|result| {
            result["partialFingerprints"]["yakAnalyzer/v1"]
                .as_str()
                .expect("a fingerprint")
        })
        .collect()
}

#[test]
fn every_way_a_project_names_its_entry_points_leaves_only_the_file_nothing_reaches() {
    let project = Project::new("audit-entry-patterns");

    project
        .write(
            "package.json",
            r#"{
                "name": "app",
                "main": "./dist/index.js",
                "bin": { "app": "./bundle/app.js" },
                "exports": { ".": "./dist/index.js", "./features/*": "./dist/features/*.js" },
                "scripts": { "test": "node scripts/run-tests.mjs" }
            }"#,
        )
        .write(
            "tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "dist", "rootDir": "src" }, "files": ["src/global.ts"] }"#,
        )
        .write("src/index.ts", "export const start = 1;\n")
        .write("src/features/search.ts", "export const search = 1;\n")
        .write("src/global.ts", "export {};\n")
        .write("src/cli.ts", "export {};\n")
        .write("src/math.test.ts", "export {};\n")
        .write("scripts/run-tests.mjs", "export {};\n")
        .write("tools/program.mjs", "export {};\n")
        .write(
            "runner.config.ts",
            "import { helper } from './config-helper';\n\
             export default { include: ['e2e/**/*.e2e.ts'], setup: helper };\n",
        )
        .write("config-helper.ts", "export const helper = 1;\n")
        .write("e2e/login.e2e.ts", "export {};\n")
        .write("src/orphan.ts", "export const orphan = 1;\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(
            json!({
                "entries": ["dist/**/*.test.js"],
                "configEntries": [{
                    "file": "runner.config.ts",
                    "property": ["include"],
                    "value": "globs",
                    "relativeTo": "config",
                    "default": []
                }]
            }),
            json!({ "entries": ["src/cli.ts", "tools/program.mjs"] }),
        ),
    );
    let run = audit(&project, Some(&hand_off));

    // A `package.json`'s fields and scripts, a tsconfig's `files`, a glob of built tests mapped back, a
    // configuration file and the helper it imports, an `exports` pattern, a bundled `bin` settled by an
    // `entries` declaration, and a program a test starts by its path: each is an entry point, and
    // their exports are all used. Only the file nothing reaches is reported, and not its export (spec
    // KIND-1, KIND-2, FIX-1 patterns 1 to 4, 6, 7, 11 and 25).
    assert_eq!(found(&run), [("unused-file", "src/orphan.ts", None)]);
    // Its package has a `bin` target that maps to no source, so the message says what is likely missing
    // (spec ENTRY-8, FIX-1 pattern 3).
    assert_eq!(
        message(&run, "unused-file", "src/orphan.ts"),
        "No entry point reaches this file through any import, so it can be removed. Its package `.` has a \
         `bin` or `exports` target that maps to no source, so the likely fault is a missing `entries` \
         declaration."
    );
}

#[test]
fn reports_an_export_no_other_file_uses_as_a_value_or_a_type_once_where_it_is_declared() {
    let project = Project::new("audit-unused-exports");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { used, type Shape } from './shapes';\n\
             import { spare } from './spare';\n\
             import * as tools from './tools';\n\
             import { relayed } from './relay';\n\
             const local = (shape: Shape) => used(shape) + tools.measure() + relayed;\n\
             local({ size: 1 });\n",
        )
        .write(
            "src/shapes.ts",
            "export interface Shape { size: number }\n\
             export type Unseen = string;\n\
             export const used = (shape: Shape): number => inner(shape);\n\
             export function inner(shape: Shape): number { return shape.size; }\n\
             export const nowhere = 1;\n",
        )
        .write("src/spare.ts", "export const spare = 1;\n")
        .write(
            "src/tools.ts",
            "export const measure = (): number => 1;\nexport const cut = (): number => 2;\n",
        )
        .write(
            "src/other.ts",
            "const nowhere = 2;\nexport const elsewhere = nowhere;\n",
        )
        .write("src/relay.ts", "export * from './deep';\n")
        .write(
            "src/deep.ts",
            "export const relayed = 1;\nexport const stranded = 2;\n",
        )
        .commit();

    let run = audit(&project, None);

    // An export used only inside its file and one used nowhere; an import whose binding is never
    // referenced uses nothing; a local of the same name in another file is not a use; a namespace uses
    // the member read off it; and a name passed on through `export *` is reported once, where it is
    // declared (spec KIND-2, USE-2, USE-2a, FIX-1 pattern 17, FIX-2).
    assert_eq!(
        found(&run),
        [
            ("unused-file", "src/other.ts", None),
            ("unused-export", "src/deep.ts", Some(2)),
            ("unused-export", "src/shapes.ts", Some(4)),
            ("unused-export", "src/shapes.ts", Some(5)),
            ("unused-export", "src/spare.ts", Some(1)),
            ("unused-export", "src/tools.ts", Some(2)),
            ("unused-type", "src/shapes.ts", Some(2)),
        ]
    );
    assert_eq!(
        run["results"][2]["message"]["text"],
        json!(
            "The export `inner` is used only inside its own file, so its `export` can be dropped."
        )
    );
    assert_eq!(
        run["results"][3]["message"]["text"],
        json!("The export `nowhere` is used nowhere, so it can be removed.")
    );
    assert_eq!(
        run["results"][6]["message"]["text"],
        json!("The type `Unseen` is used nowhere, so it can be removed.")
    );
    // An export's location is its name, the column counted from one.
    assert_eq!(
        run["results"][2]["locations"][0]["physicalLocation"]["region"],
        json!({ "startLine": 4, "startColumn": 17 })
    );
}

#[test]
fn reports_no_file_of_a_kind_that_is_never_an_unused_file() {
    let project = Project::new("audit-never-unused");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .write("src/env.d.ts", "declare const flag: boolean;\n")
        .write("src/fixtures/broken.ts", "const = ;\n")
        .write("src/generated/api.ts", "export const api = 1;\n")
        .write("docs/guide.mdx", "# Guide\n")
        .write("public/index.html", "<p>page</p>\n")
        .write("notes.txt", "notes\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(
            json!({ "data": ["**/fixtures/**"] }),
            json!({ "generated": ["src/generated/**"] }),
        ),
    );

    // A type declaration file, a data file, a generated file, an MDX document, an HTML page and a file
    // of another kind are never reported unused (spec KIND-1).
    assert_eq!(found(&audit(&project, Some(&hand_off))), []);
}

#[test]
fn a_vue_template_and_an_mdx_document_keep_what_they_name_used() {
    let project = Project::new("audit-vue-mdx");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/main.ts" }"#,
        )
        .write(
            "src/main.ts",
            "import App from './App.vue';\nconsole.log(App);\n",
        )
        .write(
            "src/App.vue",
            "<template>\n  <Card />\n</template>\n\n\
             <script setup lang=\"ts\">\n\
             import Card from './Card';\n\
             import Unused from './Unused';\n\
             </script>\n",
        )
        .write("src/Card.ts", "export default {};\n")
        .write("src/Unused.ts", "export default {};\n")
        .write(
            "docs/guide.mdx",
            "import { Chart, Table } from '../src/widgets'\n\n# Guide\n\n<Chart />\n",
        )
        .write(
            "src/widgets.ts",
            "export const Chart = 1;\nexport const Table = 2;\n",
        )
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(json!({ "entries": ["docs/**/*.mdx"] }), json!({})),
    );

    // A component the template names is used, and one a `<script setup>` block only imports is not,
    // though the import still reaches its file; a document a declaration makes an entry point uses the
    // export its text names, and not the one it only imports (spec KIND-1, KIND-2, USE-4, USE-5, FIX-1
    // pattern 16).
    assert_eq!(
        found(&audit(&project, Some(&hand_off))),
        [
            ("unused-export", "src/Unused.ts", Some(1)),
            ("unused-export", "src/widgets.ts", Some(2)),
        ]
    );
}

#[test]
fn the_kinds_that_depend_on_entry_points_wait_for_a_detection_folder() {
    let project = Project::new("audit-not-detected");

    project
        .write(
            "packages/a/package.json",
            r#"{ "name": "a", "main": "./index.ts" }"#,
        )
        .write("packages/a/index.ts", "export {};\n")
        .write("packages/a/orphan.ts", "export const orphan = 1;\n")
        .write(
            "packages/b/package.json",
            r#"{ "name": "b", "main": "./index.ts" }"#,
        )
        .write("packages/b/index.ts", "import './missing';\n")
        .write("packages/b/orphan.ts", "export const orphan = 1;\n")
        .write("scripts/loose.ts", "export const loose = 1;\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &[".", "packages/a"],
        stack(json!({}), json!({})),
    );
    let run = audit(&project, Some(&hand_off));

    // Only the package whose folder yak's detection looked in has its unused files reported. The others
    // get a notice each, and so do the files in no package (spec ENTRY-9, FIX-2).
    assert_eq!(
        found(&run)
            .into_iter()
            .filter(|(rule, _, _)| rule.starts_with("unused"))
            .collect::<Vec<_>>(),
        [("unused-file", "packages/a/orphan.ts", None)]
    );
    assert_eq!(
        notifications(&run),
        [("kinds-not-reported", None), ("kinds-not-reported", None),]
    );

    let sentences: Vec<&str> = run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .map(|notification| {
            notification["message"]["text"]
                .as_str()
                .expect("a sentence")
        })
        .collect();

    assert_eq!(
        sentences,
        [
            "1 file(s) the analyzer reads are in no package, so their unused files, exports and types are \
             not reported.",
            "The package `packages/b` is not in a folder yak's detection looked in, so its unused files, \
             exports and types are not reported.",
        ]
    );
}

#[test]
fn a_computed_import_gives_a_notice_to_declare_what_it_loads() {
    let project = Project::new("audit-computed");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "const name = 'a';\nawait import(`./plugins/${name}`);\nrequire(name);\n",
        )
        .write("src/plugins/a.ts", "export {};\n")
        .commit();

    let run = audit(&project, None);

    // What a computed import loads looks unused until it is declared (spec ENTRY-10, FIX-1 pattern 19).
    assert_eq!(found(&run), [("unused-file", "src/plugins/a.ts", None)]);
    assert_eq!(
        notifications(&run),
        [
            ("no-hand-off-file", None),
            ("computed-import", Some("src/index.ts")),
        ]
    );
    assert_eq!(
        run["invocations"][0]["toolExecutionNotifications"][1]["message"]["text"],
        json!(
            "2 call(s) of `import()` or `require()` load a file the analyzer cannot work out, so what they \
             load must be declared in `entries`."
        )
    );
}

#[test]
fn a_fingerprint_names_a_finding_by_its_parts_and_never_by_its_line() {
    let project = Project::new("audit-fingerprints");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import './a';\n")
        .write(
            "src/a.ts",
            "export const first = 1;\nexport const second = 2;\n",
        )
        .write("src/b.ts", "export {};\n")
        .commit();

    let before = audit(&project, None);

    // Each fingerprint is 32 hexadecimal digits, and each finding has its own (spec REPORT-3).
    let printed = fingerprints(&before);

    assert_eq!(printed.len(), 3);
    assert!(printed.iter().all(|each| {
        each.len() == 32
            && each
                .chars()
                .all(|digit| digit.is_ascii_digit() || ('a'..='f').contains(&digit))
    }));
    assert_eq!(
        printed
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );

    // Moved down a line, the same findings keep their fingerprints.
    project
        .write(
            "src/a.ts",
            "\nexport const first = 1;\nexport const second = 2;\n",
        )
        .commit();

    let after = audit(&project, None);

    assert_eq!(found(&after)[1], ("unused-export", "src/a.ts", Some(2)));
    assert_eq!(fingerprints(&after), printed);
}
