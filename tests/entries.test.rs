//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! and the packages whose entry points are likely incomplete (spec ENTRY-1 to ENTRY-8, CMD-4, FIX-1,
//! FIX-2).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// A source that a file of the project names.
fn named(from: &str, file: &str) -> Value {
    json!({ "by": null, "file": file, "from": from, "stack": null })
}

/// A source that a declaration of the stack `typescript` names, the pack's or the project's.
fn declared(by: &str) -> Value {
    json!({ "by": by, "file": null, "from": "entries", "stack": "typescript" })
}

/// One stack named `typescript` that holds the declarations given, the pack's and the project's.
fn stack(pack: Value, project: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": project }])
}

/// The entry points a document lists, each with its sources.
fn entries(document: &Value) -> Vec<(&str, Vec<Value>)> {
    document["entries"]
        .as_array()
        .expect("a list of entry points")
        .iter()
        .map(|entry| {
            (
                entry["path"].as_str().expect("a path"),
                entry["sources"]
                    .as_array()
                    .expect("a list of sources")
                    .clone(),
            )
        })
        .collect()
}

/// The packages a document marks as likely missing entry points, each with why.
fn faults(document: &Value) -> Vec<(&str, &str)> {
    document["entryFaults"]
        .as_array()
        .expect("a list of faults")
        .iter()
        .map(|fault| {
            (
                fault["package"].as_str().expect("a folder"),
                fault["fault"].as_str().expect("a fault"),
            )
        })
        .collect()
}

#[test]
fn a_package_jsons_main_module_types_and_bin_name_entry_points_mapped_back_to_their_source() {
    let project = Project::new("entries-package-fields");

    project
        .write(
            "package.json",
            r#"{
                "name": "lib",
                "main": "./dist/index.js",
                "module": "./dist/index.mjs",
                "types": "./dist/index.d.ts",
                "bin": { "lib": "./bin/cli.js", "lib-admin": "./dist/admin.js" }
            }"#,
        )
        .write(
            "tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "dist", "rootDir": "src" } }"#,
        )
        .write("src/index.ts", "")
        .write("src/admin.ts", "")
        .write("src/other.ts", "")
        .write("bin/cli.js", "")
        .commit();

    let document = project.graph();

    assert_eq!(
        entries(&document),
        [
            ("bin/cli.js", vec![named("bin", "package.json")]),
            ("src/admin.ts", vec![named("bin", "package.json")]),
            (
                "src/index.ts",
                vec![
                    named("main", "package.json"),
                    named("module", "package.json"),
                    named("types", "package.json"),
                ]
            ),
        ]
    );
    assert_eq!(faults(&document), []);
}

#[test]
fn exports_patterns_fill_one_star_twice_hide_what_a_null_key_matches_and_add_nothing_when_internal()
{
    let project = Project::new("entries-exports");

    project
        .write(
            "package.json",
            r#"{ "private": true, "workspaces": ["packages/*"] }"#,
        )
        .write(
            "packages/ui/package.json",
            r#"{
                "name": "ui",
                "exports": {
                    ".": { "types": "./dist/index.d.ts", "default": "./dist/index.js" },
                    "./*": "./dist/*/*.js",
                    "./secret": null
                }
            }"#,
        )
        .write(
            "packages/ui/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "dist", "rootDir": "src" } }"#,
        )
        .write("packages/ui/src/index.ts", "")
        .write("packages/ui/src/button/button.ts", "")
        .write("packages/ui/src/card/card.tsx", "")
        .write("packages/ui/src/secret/secret.ts", "")
        .write("packages/ui/src/button/helper.ts", "")
        .commit();

    let document = project.graph();
    let exported = || vec![named("exports", "packages/ui/package.json")];

    // `./*` reaches `button/button` and `card/card`, never `button/helper`; `./secret` is hidden.
    assert_eq!(
        entries(&document),
        [
            ("packages/ui/src/button/button.ts", exported()),
            ("packages/ui/src/card/card.tsx", exported()),
            ("packages/ui/src/index.ts", exported()),
        ]
    );
    assert_eq!(faults(&document), [(".", "no-entry-point")]);

    // Declared internal, the package's `exports` adds no entry points (spec ENTRY-5).
    let file = project.hand_off(
        &project.root,
        &[".", "packages/ui"],
        stack(json!({}), json!({ "internal": ["packages/ui"] })),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(entries(&document), []);
    assert_eq!(
        faults(&document),
        [(".", "no-entry-point"), ("packages/ui", "no-entry-point")]
    );
}

#[test]
fn a_bin_or_exports_target_with_no_source_marks_its_package() {
    let project = Project::new("entries-unmapped");

    project
        .write(
            "package.json",
            r#"{
                "name": "generated",
                "bin": "./bin/run.js",
                "exports": { "./schema": "./generated/schema.js" }
            }"#,
        )
        .write("lib/x.ts", "")
        .commit();

    let document = project.graph();

    assert_eq!(entries(&document), []);
    assert_eq!(faults(&document), [(".", "unmapped-target")]);
}

#[test]
fn a_tsconfigs_files_and_a_pages_scripts_are_entry_points() {
    let project = Project::new("entries-files-and-pages");

    project
        .write("package.json", r#"{ "name": "app" }"#)
        .write("tsconfig.json", r#"{ "files": ["src/setup.ts"] }"#)
        .write(
            "public/admin.html",
            r#"<script type="module" src="../src/main.ts"></script>"#,
        )
        .write("src/setup.ts", "")
        .write("src/main.ts", "")
        .write("src/other.ts", "")
        .commit();

    let document = project.graph();

    assert_eq!(
        entries(&document),
        [
            ("src/main.ts", vec![named("page", "public/admin.html")]),
            ("src/setup.ts", vec![named("files", "tsconfig.json")]),
        ]
    );
    assert_eq!(faults(&document), []);
}

#[test]
fn entries_globs_apply_a_packs_in_each_folder_and_a_projects_from_the_root_mapped_from_build_output()
 {
    let project = Project::new("entries-declared");

    project
        .write(
            "package.json",
            r#"{ "private": true, "workspaces": ["packages/*"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write(
            "packages/a/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "build", "rootDir": "src" } }"#,
        )
        .write("packages/a/src/x.ts", "")
        .write("packages/a/src/x.test.ts", "")
        .write("packages/a/src/deep/y.test.tsx", "")
        .write("packages/b/package.json", r#"{ "name": "b" }"#)
        .write("packages/b/test/z.test.js", "")
        .write("scripts/release.mjs", "")
        .commit();

    let declarations = stack(
        json!({ "entries": ["build/**/*.test.js", "test/**/*.test.js", "nothing/**"] }),
        json!({ "entries": ["scripts/*.mjs"] }),
    );
    let file = project.hand_off(
        &project.root,
        &[".", "packages/a", "packages/b"],
        declarations,
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // In `packages/a` the built tests map back to their sources; in `packages/b` they are committed
    // JavaScript and match as written (spec ENTRY-3, ENTRY-4).
    assert_eq!(
        entries(&document),
        [
            ("packages/a/src/deep/y.test.tsx", vec![declared("pack")]),
            ("packages/a/src/x.test.ts", vec![declared("pack")]),
            ("packages/b/test/z.test.js", vec![declared("pack")]),
            ("scripts/release.mjs", vec![declared("project")]),
        ]
    );
    assert_eq!(notices(&document), ["glob-matches-nothing"]);
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The `entries` glob `nothing/**` that the pack states for the stack `typescript` matches no file, \
             nor any source mapped back from build output."
        )
    );
}

#[test]
fn a_bin_of_a_form_npm_does_not_give_it_stops_the_run() {
    let project = Project::new("entries-bin-form");

    project
        .write("package.json", r#"{ "name": "tool", "bin": ["./cli.js"] }"#)
        .write("cli.js", "")
        .commit();

    // The field is read in every form npm gives it, and a value of another form stops the run at
    // RUN-4's step 11 (spec ENTRY-1, PACK-6).
    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `bin` in package.json holds a value of a form it does not have; it must be a \
         path, or an object that maps each command to a path; fix it"
    );

    project
        .write("package.json", r#"{ "name": "tool", "bin": "./cli.js" }"#)
        .commit();

    let document = project.graph();

    assert_eq!(paths(&document), ["cli.js", "package.json"]);
    assert_eq!(
        entries(&document),
        [("cli.js", vec![named("bin", "package.json")])]
    );
}

/// A source that a stack's `configEntries` declaration names: the configuration file itself, with no
/// file, or a file the configuration file's value names.
fn configured(file: Option<&str>) -> Value {
    json!({ "by": "pack", "file": file, "from": "configEntries", "stack": "typescript" })
}

#[test]
fn a_scripts_words_after_node_or_a_declared_program_are_entry_points() {
    let project = Project::new("entries-scripts");

    project
        .write(
            "package.json",
            r#"{
                "name": "app",
                "scripts": {
                    "test": "node scripts/run-tests.mjs",
                    "dev": "NODE_ENV=dev node --watch './src/server.ts' && echo done",
                    "bundle": "cross-env MODE=prod esbuild src/cli.ts --outfile=dist/cli.js",
                    "lint": "eslint src/other.ts",
                    "spaced": "node \"scripts/a b.mjs\"; node scripts/missing.mjs"
                }
            }"#,
        )
        .write("scripts/run-tests.mjs", "")
        .write("scripts/a b.mjs", "")
        .write("src/server.ts", "")
        .write("src/cli.ts", "")
        .write("src/other.ts", "")
        .commit();

    let scripts = || vec![named("scripts", "package.json")];

    // Without a hand-off file only `node` runs files: `cross-env` is no program the analyzer knows.
    assert_eq!(
        entries(&project.graph()),
        [
            ("scripts/a b.mjs", scripts()),
            ("scripts/run-tests.mjs", scripts()),
            ("src/server.ts", scripts()),
        ]
    );

    // Declared, every later word of `cross-env`'s command is tried, the option's value mapped back from
    // build output (spec ENTRY-7).
    let file = project.hand_off(
        &project.root,
        &["."],
        stack(json!({ "scriptCommands": ["cross-env"] }), json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(
        entries(&document),
        [
            ("scripts/a b.mjs", scripts()),
            ("scripts/run-tests.mjs", scripts()),
            ("src/cli.ts", scripts()),
            ("src/server.ts", scripts()),
        ]
    );
}

#[test]
fn scripts_of_a_form_npm_does_not_give_them_stop_the_run() {
    let project = Project::new("entries-scripts-form");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "scripts": ["node x.js"] }"#,
        )
        .commit();

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `scripts` in package.json holds a value of a form it does not have; it must \
         be an object that maps each script's name to its command; fix it"
    );
}

#[test]
fn a_configuration_file_names_entry_points_by_a_literal_a_folder_or_its_default() {
    let project = Project::new("entries-configured");

    project
        .write(
            "packages/app/vitest.config.ts",
            "import { defineConfig } from 'vitest/config';\n\
             import './test/helper';\n\
             export default defineConfig({\n\
               test: { include: ['test/**/*.spec.ts', '!skip/**', 'odd/**/*.?(c|m)ts'] },\n\
             });\n",
        )
        .write("packages/app/test/a.spec.ts", "")
        .write("packages/app/test/helper.ts", "")
        .write(
            "packages/app/tools/jest.config.js",
            "module.exports = { roots: [`tools/tests`] };\n",
        )
        .write("packages/app/tools/tests/x.test.js", "")
        .write(
            "runner.config.js",
            "const folders = ['x'];\nexport default { include: folders };\n",
        )
        .write("checks/a.check.js", "")
        .commit();

    let declarations = json!({
        "configEntries": [
            {
                "file": "vitest.config.{ts,mts,js,mjs}",
                "property": ["test", "include"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["**/*.test.ts"]
            },
            {
                "file": "**/jest.config.js",
                "property": ["roots"],
                "value": "folder",
                "pattern": "**/*.test.js",
                "relativeTo": "detection",
                "default": ["**/*.test.js"]
            },
            {
                "file": "runner.config.js",
                "property": ["include"],
                "value": "globs",
                "relativeTo": "detection",
                "default": ["checks/*.check.js"]
            }
        ]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/app"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // The literal inside `defineConfig`, from the configuration file's folder; the folder `roots` names,
    // from the detection folder, joined to its pattern; and the default where the value is a variable
    // (spec ENTRY-6). Each configuration file is itself an entry point, from the deepest detection folder.
    assert_eq!(
        entries(&document),
        [
            (
                "checks/a.check.js",
                vec![configured(Some("runner.config.js"))]
            ),
            (
                "packages/app/test/a.spec.ts",
                vec![configured(Some("packages/app/vitest.config.ts"))]
            ),
            ("packages/app/tools/jest.config.js", vec![configured(None)]),
            (
                "packages/app/tools/tests/x.test.js",
                vec![configured(Some("packages/app/tools/jest.config.js"))]
            ),
            ("packages/app/vitest.config.ts", vec![configured(None)]),
            ("runner.config.js", vec![configured(None)]),
        ]
    );
    assert_eq!(
        notices(&document),
        ["config-glob-not-read", "config-glob-not-read"]
    );
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The glob `!skip/**` in packages/app/vitest.config.ts is not one the analyzer reads, so it adds no \
             entry point."
        )
    );
}
