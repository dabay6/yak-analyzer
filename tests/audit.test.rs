//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4).

#[path = "support/project.rs"]
mod project;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The copy of SARIF 2.1.0's schema. `tests/sarif/README.md` says where it came from.
const SCHEMA: &str = include_str!("sarif/sarif-schema-2.1.0.json");

/// The one run of the report `audit` prints for the checkout, with the hand-off file given or with
/// none.
fn audit(project: &Project, hand_off: Option<&str>) -> Value {
    match hand_off {
        Some(file) => audited(project, &["--hand-off", file]),
        None => audited(project, &[]),
    }
}

/// The one run of the report `audit` prints for the checkout with the options given. The run must
/// succeed and write no line, and the schema must accept the report (spec FIX-3).
fn audited(project: &Project, options: &[&str]) -> Value {
    let report = printed(&project.start("audit", |binary| {
        binary.args(options);
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

#[test]
fn reports_each_import_that_leads_to_no_file_in_any_package() {
    let project = Project::new("audit-unresolved");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts", "dependencies": { "left-pad": "1.0.0" } }"#,
        )
        .write(
            "src/index.ts",
            "import './missing';\nimport 'left-pad';\nimport 'node:fs';\nimport './missing';\n\
             const name = 'x';\nawait import(name);\n",
        )
        .write("src/env.d.ts", "import './nowhere';\n")
        .write("src/generated/api.ts", "import './nowhere';\n")
        .write("tools/package.json", r#"{ "name": "tools" }"#)
        .write("tools/run.ts", "require('./gone');\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(json!({}), json!({ "generated": ["src/generated/**"] })),
    );
    let run = audit(&project, Some(&hand_off));

    // Each import that leads nowhere is one finding at its line, in a package detection looked in or
    // not; one of a package, a built-in module or a computed specifier is none, and nor is one in a type
    // declaration file or a generated file. Two of one specifier in one file share a fingerprint (spec
    // KIND-3, REPORT-3).
    let unresolved: Vec<_> = found(&run)
        .into_iter()
        .filter(|(rule, _, _)| *rule == "unresolved-import")
        .collect();

    assert_eq!(
        unresolved,
        [
            ("unresolved-import", "src/index.ts", Some(1)),
            ("unresolved-import", "src/index.ts", Some(4)),
            ("unresolved-import", "tools/run.ts", Some(1)),
        ]
    );
    assert_eq!(
        message(&run, "unresolved-import", "tools/run.ts"),
        "The import of `./gone` leads to no file, no package and no built-in module."
    );

    let printed = fingerprints(&run);
    let first = found(&run)
        .iter()
        .position(|(rule, _, _)| *rule == "unresolved-import")
        .expect("an unresolved import");

    assert_eq!(printed[first], printed[first + 1]);
    assert_ne!(printed[first], printed[first + 2]);
}

#[test]
fn reports_each_set_of_files_that_load_each_other_once_over_static_imports_alone() {
    let project = Project::new("audit-cycles");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import './a';\nimport './lazy';\nimport './typed';\nimport './self';\nimport './one';\n",
        )
        // A static cycle of three, with a shorter way back from `b`.
        .write(
            "src/a.ts",
            "export const a = 1;\nimport { b } from './b';\nconsole.log(b);\n",
        )
        .write(
            "src/b.ts",
            "import './c';\nexport { a } from './a';\nexport const b = 2;\n",
        )
        .write(
            "src/c.ts",
            "const loaded = require('./b');\nconsole.log(loaded);\n",
        )
        // A loop closed only by `import()`, or only by imports of types, loads nothing in a loop.
        .write("src/lazy.ts", "import './lazy-back';\n")
        .write("src/lazy-back.ts", "await import('./lazy');\n")
        .write("src/typed.ts", "import './typed-back';\nexport type T = 1;\n")
        .write(
            "src/typed-back.ts",
            "import type { T } from './typed';\nimport { type T as U } from './typed';\n\
             export type { T } from './typed';\nexport type V = [T, U];\n",
        )
        // A file that imports itself.
        .write("src/self.ts", "import './self';\n")
        // A loop through a type declaration file is none.
        .write("src/one.ts", "import './two';\n")
        .write("src/two.d.ts", "import './one';\n")
        .commit();

    let run = audit(&project, None);
    let cycles: Vec<&Value> = run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .filter(|result| result["ruleId"] == json!("circular-dependency"))
        .collect();
    let related = |result: &Value| -> Vec<(String, u64)> {
        result["relatedLocations"]
            .as_array()
            .map(|locations| {
                locations
                    .iter()
                    .map(|location| {
                        let place = &location["physicalLocation"];

                        (
                            place["artifactLocation"]["uri"]
                                .as_str()
                                .expect("a file")
                                .to_owned(),
                            place["region"]["startLine"].as_u64().expect("a line"),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    // One finding for each set, at the first import of its first file by path that loads another file
    // of the set; the set's other files are related, each at its first such import. The message names
    // the shortest loop through the first file (spec KIND-4, FIX-1 pattern 18).
    assert_eq!(
        found(&run)
            .into_iter()
            .filter(|(rule, _, _)| *rule == "circular-dependency")
            .collect::<Vec<_>>(),
        [
            ("circular-dependency", "src/a.ts", Some(2)),
            ("circular-dependency", "src/self.ts", Some(1)),
        ]
    );
    assert_eq!(
        related(cycles[0]),
        [("src/b.ts".to_owned(), 1), ("src/c.ts".to_owned(), 1)]
    );
    assert_eq!(
        cycles[0]["message"]["text"],
        json!(
            "These files import each other in a loop: `src/a.ts` → `src/b.ts` → `src/a.ts`. The loop is part \
             of a set of 3 files that each reach all the others."
        )
    );
    assert_eq!(related(cycles[1]), []);
    assert_eq!(
        cycles[1]["message"]["text"],
        json!("These files import each other in a loop: `src/self.ts` → `src/self.ts`.")
    );

    // An import all of whose names are marked `type` brings in types alone, as `graph` prints it.
    let document = project.graph();
    let typed_back = document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!("src/typed-back.ts"))
        .expect("the file is listed");

    assert_eq!(
        typed_back["imports"]
            .as_array()
            .expect("a list of imports")
            .iter()
            .map(|import| import["typeOnly"].as_bool().expect("a flag"))
            .collect::<Vec<bool>>(),
        [true, true, true]
    );
}

#[test]
fn names_ten_files_of_a_long_loop_and_how_many_more() {
    let project = Project::new("audit-long-cycle");

    project.write(
        "package.json",
        r#"{ "name": "app", "main": "./src/m00.ts" }"#,
    );

    for link in 0..12 {
        project.write(
            &format!("src/m{link:02}.ts"),
            &format!("import './m{:02}';\n", (link + 1) % 12),
        );
    }

    project.commit();

    let run = audit(&project, None);

    // A loop of twelve files names the first ten, then how many more, then the first again (spec
    // KIND-4).
    assert_eq!(
        run["results"][0]["message"]["text"],
        json!(
            "These files import each other in a loop: `src/m00.ts` → `src/m01.ts` → `src/m02.ts` → \
             `src/m03.ts` → `src/m04.ts` → `src/m05.ts` → `src/m06.ts` → `src/m07.ts` → `src/m08.ts` → \
             `src/m09.ts` → 2 more → `src/m00.ts`."
        )
    );
    assert_eq!(
        run["results"][0]["relatedLocations"]
            .as_array()
            .map(Vec::len),
        Some(11)
    );
}

/// The findings of a run, in its order: each with its rule, its file as the report writes it, its line
/// where it has one, and whether it is new or inherited where the run says.
fn states(run: &Value) -> Vec<(&str, &str, Option<u64>, Option<&str>)> {
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
                result["baselineState"].as_str(),
            )
        })
        .collect()
}

/// The notifications of a run, in its order: each with its kind, the file it is about as the report
/// writes it, and whether the base's reading gave it.
fn marked(run: &Value) -> Vec<(&str, Option<&str>, bool)> {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .map(|notification| {
            (
                notification["descriptor"]["id"].as_str().expect("a kind"),
                notification["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str(),
                notification["properties"]["base"] == json!(true),
            )
        })
        .collect()
}

/// The sentence of the notification of the kind given.
fn sentence<'r>(run: &'r Value, kind: &str) -> &'r str {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .find(|notification| notification["descriptor"]["id"] == json!(kind))
        .and_then(|notification| notification["message"]["text"].as_str())
        .unwrap_or_else(|| panic!("a notification of the kind {kind}"))
}

/// Every path under a folder, `.git` included, with the size of each file and when it was last
/// changed. A folder's own time is left out: the system may record a change made in it a little
/// later, and a change shows in its files anyway.
fn snapshot(folder: &Path) -> BTreeMap<String, Option<(u64, Option<SystemTime>)>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![folder.to_path_buf()];

    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).expect("the folder can be read") {
            let entry = entry.expect("an entry can be read");
            let facts = entry.metadata().expect("an entry can be looked at");
            let path = entry.path();

            if facts.is_dir() {
                found.insert(path.display().to_string(), None);
                waiting.push(path);
            } else {
                found.insert(
                    path.display().to_string(),
                    Some((facts.len(), facts.modified().ok())),
                );
            }
        }
    }

    found
}

/// A project of one package whose base is the commit on `main`, with the branch `branch` checked out.
fn branched(name: &str, files: &[(&str, &str)]) -> Project {
    let project = Project::new(name);

    for (path, content) in files {
        project.write(path, content);
    }

    project.commit();
    project.git(&["switch", "--quiet", "-c", "branch"]);

    project
}

#[test]
fn a_finding_the_base_has_too_is_unchanged_and_one_the_branch_made_is_new() {
    let project = branched(
        "base-new-and-inherited",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            (
                "src/index.ts",
                "import { a } from './a';\nimport './missing';\na();\n",
            ),
            (
                "src/a.ts",
                "export const a = (): void => {};\nexport const old = 1;\nexport const dropped = 2;\n",
            ),
        ],
    );

    // The branch drops one unused export and adds another in a commit, and adds unresolved imports and
    // an untracked file without committing them (spec FIX-4).
    project
        .write(
            "src/a.ts",
            "export const a = (): void => {};\nexport const old = 1;\nexport const fresh = 3;\n",
        )
        .commit()
        .write(
            "src/index.ts",
            "import { a } from './a';\nimport './missing';\nimport './missing';\nimport './elsewhere';\na();\n",
        )
        .write("src/scratch.ts", "export const scratch = 1;\n");

    let run = audited(&project, &["--base", "main"]);

    // A finding the base has too is unchanged; one the branch made is new; one only the base has is not
    // printed. Of two findings with one fingerprint, the base's one is the first by line (spec INH-1,
    // REPORT-1, FIX-1 pattern 22).
    assert_eq!(
        states(&run),
        [
            ("unused-file", "src/scratch.ts", None, Some("new")),
            ("unused-export", "src/a.ts", Some(2), Some("unchanged")),
            ("unused-export", "src/a.ts", Some(3), Some("new")),
            (
                "unresolved-import",
                "src/index.ts",
                Some(2),
                Some("unchanged")
            ),
            ("unresolved-import", "src/index.ts", Some(3), Some("new")),
            ("unresolved-import", "src/index.ts", Some(4), Some("new")),
        ]
    );
    // The same project gives the same bytes every run (spec INH-4, RUN-8).
    assert_eq!(run, audited(&project, &["--base", "main"]));
    // Without a base, no finding is new or inherited (spec CMD-1).
    assert!(
        states(&audited(&project, &[]))
            .iter()
            .all(|(_, _, _, state)| state.is_none())
    );
}

#[test]
fn the_base_is_read_from_the_commit_and_nothing_is_written() {
    let project = branched(
        "base-from-the-commit",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            ("src/index.ts", "import { used } from './util';\nused();\n"),
            (
                "src/util.ts",
                "export const used = (): void => {};\nexport const spare = 1;\n",
            ),
            ("src/old.ts", "export const old = 1;\n"),
        ],
    );

    // The working tree changes a file and deletes another, and commits neither change: the base still
    // reads both as the commit stores them.
    project.write(
        "src/util.ts",
        "export const used = (): void => {};\nexport const extra = 1;\n",
    );
    fs::remove_file(project.root.join("src/old.ts")).expect("the file can be removed");

    // git's own `status` may write its index, so it is asked before the snapshot is taken.
    let status = project.git(&["status", "--porcelain"]);
    let before = snapshot(&project.root);
    let run = audited(&project, &["--base", "branch"]);

    // The base's `spare` and its unused file are the commit's, so neither is printed, and `extra` is
    // the working tree's alone (spec COMMIT-1, COMMIT-2).
    assert_eq!(
        states(&run),
        [("unused-export", "src/util.ts", Some(2), Some("new"))]
    );
    // Nothing is checked out and nothing is written, `.git` included (spec COMMIT-1, RUN-6).
    assert_eq!(snapshot(&project.root), before);
    assert_eq!(project.git(&["status", "--porcelain"]), status);
}

#[test]
fn names_the_base_commit_and_marks_the_notices_of_the_bases_reading() {
    let project = branched(
        "base-notifications",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            ("src/index.ts", "export {};\n"),
            // A tsconfig no script depends on, which extends a file that is not there: left out, with a
            // notice.
            ("other/tsconfig.json", r#"{ "extends": "./missing.json" }"#),
        ],
    );

    project.git(&["rm", "--quiet", "other/tsconfig.json"]);
    project.commit();

    let full = project.git(&["rev-parse", "main"]).trim().to_owned();
    let run = audited(&project, &["--base", "main"]);

    // One notification names the base by its full name, whatever name it was given by; each notice of
    // the base's reading is marked as the base's (spec REPORT-4, CMD-2).
    assert_eq!(
        marked(&run),
        [
            ("base", None, false),
            ("no-hand-off-file", None, false),
            (
                "tsconfig-extends-left-out",
                Some("other/tsconfig.json"),
                true
            ),
        ]
    );
    assert!(sentence(&run, "base").contains(&full), "{full}");
}

#[test]
fn a_base_that_is_not_a_commit_stops_the_run_after_the_file_list() {
    let project = Project::new("base-not-a-commit");

    project
        .write("package.json", r#"{ "name": "app" }"#)
        .write("src/index.ts", "export {};\n")
        .commit();

    // A name git does not know, and a name of a file rather than a commit, stop the run with the name
    // and the fix (spec CMD-2, CMD-3 stop 7b).
    for name in ["nosuch", "HEAD:package.json"] {
        let line = stopped(&project.start("audit", |binary| {
            binary.args(["--base", name]);
        }));

        assert_eq!(
            line,
            format!(
                "yak-analyzer: the base `{name}` is not a commit in this repository; fetch it with `git \
                 fetch`, or name another, and run again"
            )
        );
    }

    // It is asked before any `package.json` is read: a broken one is reported after it (spec RUN-4).
    project.write("package.json", "{ broken");

    let line = stopped(&project.start("audit", |binary| {
        binary.args(["--base", "nosuch"]);
    }));

    assert!(line.contains("`nosuch` is not a commit"), "{line}");
}

#[test]
fn a_base_whose_files_cannot_be_read_skips_them_and_marks_every_finding_new() {
    let project = branched(
        "base-skipped-files",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            // A tsconfig the scripts depend on, and a script, that do not parse at the base: on the
            // working tree each would stop the run.
            ("tsconfig.json", "{ broken"),
            ("src/index.ts", "import { a } from './a';\na();\n"),
            (
                "src/a.ts",
                "export const a = (): void => {};\nexport const old = 1;\n",
            ),
            ("src/broken.ts", "const = ;\n"),
        ],
    );

    project
        .write("tsconfig.json", "{}")
        .write("src/broken.ts", "export {};\n")
        .commit();

    let run = audited(&project, &["--base", "main"]);

    // At the base each is skipped with a notice marked as the base's, and since a skipped file can make
    // a finding look inherited, every finding is new, with one notification that says why (spec
    // COMMIT-4, FIX-2).
    assert_eq!(
        states(&run),
        [
            ("unused-file", "src/broken.ts", None, Some("new")),
            ("unused-export", "src/a.ts", Some(2), Some("new")),
        ]
    );
    assert_eq!(
        marked(&run),
        [
            ("base", None, false),
            ("base-files-skipped", None, false),
            ("no-hand-off-file", None, false),
            ("file-skipped", Some("src/broken.ts"), true),
            ("file-skipped", Some("tsconfig.json"), true),
        ]
    );
    assert_eq!(
        sentence(&run, "base-files-skipped"),
        "2 file(s) were skipped at the base, so no finding is marked unchanged: `tsconfig.json`, \
         `src/broken.ts`."
    );
}

#[test]
fn the_base_is_read_with_the_hand_off_files_declarations_and_the_working_trees_ignore_rules() {
    let project = branched(
        "base-declarations",
        &[
            ("package.json", r#"{ "name": "app" }"#),
            (".gitignore", "out/\n"),
            (
                "src/tool.ts",
                "import { used } from './helper';\nimport './out/built.js';\nused();\n",
            ),
            (
                "src/helper.ts",
                "export const used = (): void => {};\nexport const extra = 1;\n",
            ),
        ],
    );

    // The branch drops the ignore rule, so the import of the folder it ignored leads nowhere now.
    project.git(&["rm", "--quiet", ".gitignore"]);
    project.commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([{
            "name": "typescript",
            "interfaceVersion": 1,
            "pack": {},
            "project": { "entries": ["src/tool.ts"] }
        }]),
    );
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // The declaration makes `src/tool.ts` an entry point at the base too, so `extra` is unused on both
    // sides rather than in a file nothing reaches; and the base asks the working tree's ignore rules,
    // so the import was unresolved there too (spec COMMIT-3, COMMIT-5).
    assert_eq!(
        states(&run),
        [
            ("unused-export", "src/helper.ts", Some(2), Some("unchanged")),
            (
                "unresolved-import",
                "src/tool.ts",
                Some(2),
                Some("unchanged")
            ),
        ]
    );
}

#[test]
fn a_partial_clone_reads_its_base_with_a_git_that_never_fetches_and_stops_with_an_older_one() {
    let project = branched(
        "base-partial-clone",
        &[
            ("package.json", r#"{ "name": "app" }"#),
            ("src/index.ts", "export {};\n"),
        ],
    );

    // A promisor remote is what makes a repository a partial clone.
    project.git(&["config", "remote.origin.promisor", "true"]);

    let version = Command::new("git")
        .arg("--version")
        .output()
        .expect("git starts");
    let version = String::from_utf8_lossy(&version.stdout);
    let numbers: Vec<u32> = version
        .trim()
        .trim_start_matches("git version ")
        .split(['.', ' '])
        .take(2)
        .map(|number| number.parse().expect("a number"))
        .collect();
    let output = project.start("audit", |binary| {
        binary.args(["--base", "main"]);
    });

    // git 2.44 is the first to honour the switch that turns lazy fetching off (spec COMMIT-2, CMD-3
    // stop 7c).
    if numbers.as_slice() >= [2, 44].as_slice() {
        assert_eq!(printed(&output)["version"], json!("2.1.0"));
    } else {
        let line = stopped(&output);

        assert!(line.contains("partial clone"), "{line}");
        assert!(line.contains("2.44"), "{line}");
    }
}
