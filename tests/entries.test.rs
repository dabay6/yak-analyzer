//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! the packages whose entry points are likely incomplete, the uses an entry point makes of its own
//! exports, and the files the entry points reach (spec ENTRY-1 to ENTRY-8, USE-3, USE-6, CMD-4, FIX-1,
//! FIX-2; comparison spec CONF-1 to CONF-3, CI-1 to CI-4).

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

#[test]
fn a_configuration_files_globs_are_relative_as_the_tool_reads_them_dot_and_dot_dot_included() {
    let project = Project::new("entries-configured-relative");

    project
        .write(
            "apps/book/.storybook/main.ts",
            "export default {\n\
               stories: ['../src/**/*.stories.ts', '../../../../far/*.ts', 'docs/**/../*.mdx'],\n\
             };\n",
        )
        .write("apps/book/src/a.stories.ts", "")
        .write("apps/book/src/deep/b.stories.ts", "")
        .write(
            "packages/a/vitest.config.ts",
            "export default {\n\
               test: { setupFiles: ['./setup.ts'], globalSetup: ['../../shared.mjs'] },\n\
             };\n",
        )
        .write("packages/a/setup.ts", "")
        .write("shared.mjs", "")
        .commit();

    let declarations = json!({
        "configEntries": [
            {
                "file": ".storybook/main.{ts,js}",
                "property": ["stories"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            },
            {
                "file": "vitest.config.{ts,js}",
                "property": ["test", "setupFiles"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            },
            {
                "file": "vitest.config.{ts,js}",
                "property": ["test", "globalSetup"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            }
        ]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "apps/book", "packages/a"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // `./` names the folder the value is relative to, and a `..` climbs from it while it stays inside
    // the root (spec ENTRY-6b, ENTRY-6d).
    assert_eq!(
        entries(&document),
        [
            ("apps/book/.storybook/main.ts", vec![configured(None)]),
            (
                "apps/book/src/a.stories.ts",
                vec![configured(Some("apps/book/.storybook/main.ts"))]
            ),
            (
                "apps/book/src/deep/b.stories.ts",
                vec![configured(Some("apps/book/.storybook/main.ts"))]
            ),
            (
                "packages/a/setup.ts",
                vec![configured(Some("packages/a/vitest.config.ts"))]
            ),
            ("packages/a/vitest.config.ts", vec![configured(None)]),
            (
                "shared.mjs",
                vec![configured(Some("packages/a/vitest.config.ts"))]
            ),
        ]
    );

    // A glob that would leave the root, and one with a `..` after a wildcard, are passed over.
    assert_eq!(
        notices(&document),
        ["config-glob-not-read", "config-glob-not-read"]
    );
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The glob `../../../../far/*.ts` in apps/book/.storybook/main.ts is not one the analyzer reads, so \
             it adds no entry point."
        )
    );
}

#[test]
fn a_configuration_files_glob_of_build_output_names_the_source_it_is_built_from() {
    let project = Project::new("entries-configured-built");

    project
        .write("packages/ui/package.json", r#"{ "name": "ui" }"#)
        .write(
            "packages/ui/tsconfig.json",
            r#"{ "compilerOptions": { "rootDir": "src", "outDir": "out" } }"#,
        )
        .write(
            "packages/ui/runner.config.mjs",
            "export default { files: 'out/**/*.test.js' };\n",
        )
        .write("packages/ui/src/a.test.ts", "")
        .write("packages/ui/src/deep/b.test.ts", "")
        .write("packages/ui/src/c.ts", "")
        .commit();

    let declarations = json!({
        "configEntries": [
            {
                "file": "runner.config.{js,mjs}",
                "property": ["files"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            },
            {
                "file": "runner.config.{js,mjs}",
                "property": ["unstated"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["checks/**/*.check.js"]
            }
        ]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/ui"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // The tests the runner runs once built are their sources, mapped back through the package's
    // `outDir`; a default that matches nothing adds nothing and says nothing (spec ENTRY-6c).
    assert_eq!(
        entries(&document),
        [
            ("packages/ui/runner.config.mjs", vec![configured(None)]),
            (
                "packages/ui/src/a.test.ts",
                vec![configured(Some("packages/ui/runner.config.mjs"))]
            ),
            (
                "packages/ui/src/deep/b.test.ts",
                vec![configured(Some("packages/ui/runner.config.mjs"))]
            ),
        ]
    );
    assert_eq!(notices(&document), Vec::<&str>::new());
}

#[test]
fn a_configuration_file_a_script_names_by_an_option_is_read_as_a_matched_one_is() {
    let project = Project::new("entries-configured-option");

    project
        .write(
            "package.json",
            r#"{
                "name": "root",
                "private": true,
                "workspaces": ["packages/*"],
                "scripts": { "unit": "vitest --config packages/app/vitest.config.ts" }
            }"#,
        )
        .write(
            "packages/app/package.json",
            r#"{
                "name": "app",
                "scripts": {
                    "test": "vitest run",
                    "e2e": "npx vitest --config vitest.e2e.config.ts",
                    "int": "cross-env CI=1 ./node_modules/.bin/vitest.cmd -c=config/int.config.mjs",
                    "gone": "vitest --config missing.config.ts"
                }
            }"#,
        )
        .write(
            "packages/app/vitest.config.ts",
            "export default { test: { include: ['test/**/*.unit.ts'] } };\n",
        )
        .write(
            "packages/app/vitest.e2e.config.ts",
            "export default { test: { include: ['e2e/**/*.e2e.ts'] } };\n",
        )
        .write(
            "packages/app/config/int.config.mjs",
            "export default { test: { include: ['int/**/*.int.ts'] } };\n",
        )
        .write("packages/app/test/a.unit.ts", "")
        .write("packages/app/e2e/b.e2e.ts", "")
        .write("packages/app/int/c.int.ts", "")
        .write("packages/app/config/int/d.int.ts", "")
        .write("test/e.unit.ts", "")
        .commit();

    let declarations = json!({
        "configEntries": [{
            "file": "vitest.config.{ts,mts}",
            "property": ["test", "include"],
            "value": "globs",
            "relativeTo": "config",
            "default": ["**/*.test.ts"],
            "options": { "program": "vitest", "names": ["--config", "-c"], "relativeTo": "script" }
        }]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/app"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // A file a script names after `--config` or `-c=`, behind `npx` or `cross-env`, is an entry point,
    // and its value is read from the folder of the script's `package.json`, not the file's own. A file
    // both matched and named is read from its own folder and from the naming script's (comparison spec
    // CONF-1 to CONF-3).
    assert_eq!(
        entries(&document),
        [
            ("packages/app/config/int.config.mjs", vec![configured(None)]),
            (
                "packages/app/e2e/b.e2e.ts",
                vec![configured(Some("packages/app/vitest.e2e.config.ts"))]
            ),
            (
                "packages/app/int/c.int.ts",
                vec![configured(Some("packages/app/config/int.config.mjs"))]
            ),
            (
                "packages/app/test/a.unit.ts",
                vec![configured(Some("packages/app/vitest.config.ts"))]
            ),
            ("packages/app/vitest.config.ts", vec![configured(None)]),
            ("packages/app/vitest.e2e.config.ts", vec![configured(None)]),
            (
                "test/e.unit.ts",
                vec![configured(Some("packages/app/vitest.config.ts"))]
            ),
        ]
    );

    // A word that names no listed file adds nothing, with a notice (comparison spec CONF-2).
    assert_eq!(notices(&document), ["script-config-not-listed"]);
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The script `vitest --config missing.config.ts` in packages/app/package.json names \
             `missing.config.ts` after `--config` as a configuration file of `vitest`, and no listed file is \
             there, so it adds no entry point."
        )
    );
}

#[test]
fn a_root_script_that_runs_another_packages_build_output_names_that_packages_source() {
    let project = Project::new("entries-scripts-elsewhere");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "scripts": { "docs": "node packages/a/out/run.js" } }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write(
            "packages/a/tsconfig.json",
            r#"{ "compilerOptions": { "rootDir": "lib", "outDir": "out" } }"#,
        )
        .write("packages/a/lib/run.ts", "")
        .commit();

    // The word is mapped back by the `outDir` of the package it names, not by the root's, which has
    // none (spec ENTRY-7a).
    assert_eq!(
        entries(&project.graph()),
        [(
            "packages/a/lib/run.ts",
            vec![named("scripts", "package.json")]
        )]
    );
}

/// The source a command file's command gives each file it starts (comparison spec CI-4).
fn commanded(file: &str) -> Value {
    json!({ "by": "pack", "file": file, "from": "commandFiles", "stack": "typescript" })
}

#[test]
fn a_ci_workflows_commands_start_files_as_a_scripts_do() {
    let project = Project::new("entries-command-files");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "private": true, "workspaces": ["packages/*"] }"#,
        )
        .write("packages/web/package.json", r#"{ "name": "web" }"#)
        .write(
            ".github/workflows/ci.yml",
            r#"on: push
defaults:
  run:
    working-directory: tools
jobs:
  build:
    defaults:
      run:
        working-directory: ./packages/web/
    steps:
      - uses: actions/checkout@v4
      - run: node scripts/build.mjs
      - run: node scripts/mapped.mjs
        working-directory:
          - not a text
      - name: Several
        working-directory: .
        run: |
          node scripts/a.mjs # node scripts/commented.mjs
          node \
            scripts/b.mjs --flag
          echo "the next line is quoted
          node scripts/hidden.mjs"
          # don't run the next file
          node scripts/c.mjs
          echo it's here
          node scripts/after.mjs
          node scripts/missing.mjs $RUNNER_TEMP/x.mjs
      - run: vitest --config vitest.e2e.config.ts
        working-directory: packages/web
      - run: node scripts/never.mjs
        working-directory: ${{ matrix.dir }}
  lint:
    steps:
      - run:
          - node list-a.mjs
          - node list-b.mjs
      - run: node check.mjs
"#,
        )
        .write(
            ".github/workflows/two.yml",
            "jobs:\n  one:\n    steps:\n      - run: node first.mjs\n---\n\
             jobs:\n  two:\n    steps:\n      - run: node second.mjs\n",
        )
        .write(".github/workflows/broken.yml", "jobs: [unclosed\n")
        .write(
            ".github/workflows/bomb.yaml",
            "a: &a [x, x, x, x, x, x, x, x, x, x]\n\
             b: &b [*a, *a, *a, *a, *a, *a, *a, *a, *a, *a]\n\
             c: &c [*b, *b, *b, *b, *b, *b, *b, *b, *b, *b]\n\
             d: &d [*c, *c, *c, *c, *c, *c, *c, *c, *c, *c]\n\
             e: &e [*d, *d, *d, *d, *d, *d, *d, *d, *d, *d]\n\
             f: &f [*e, *e, *e, *e, *e, *e, *e, *e, *e, *e]\n\
             g: &g [*f, *f, *f, *f, *f, *f, *f, *f, *f, *f]\n\
             h: &h [*g, *g, *g, *g, *g, *g, *g, *g, *g, *g]\n\
             i: [*h, *h, *h, *h, *h, *h, *h, *h, *h, *h]\n",
        )
        .write(
            "packages/web/vitest.e2e.config.ts",
            "export default { test: { include: ['e2e/**/*.e2e.ts'] } };\n",
        )
        .write("packages/web/e2e/x.e2e.ts", "")
        .write("packages/web/scripts/build.mjs", "")
        .write("packages/web/scripts/mapped.mjs", "")
        .write("scripts/a.mjs", "")
        .write("scripts/b.mjs", "")
        .write("scripts/c.mjs", "")
        .write("scripts/after.mjs", "")
        .write("scripts/commented.mjs", "")
        .write("scripts/hidden.mjs", "")
        .write("scripts/never.mjs", "")
        .write("tools/check.mjs", "")
        .write("tools/list-a.mjs", "")
        .write("tools/list-b.mjs", "")
        .write("first.mjs", "")
        .write("second.mjs", "")
        .commit();

    let declarations = json!({
        "commandFiles": [{
            "file": ".github/workflows/*.{yml,yaml}",
            "commands": [["jobs", "*", "steps", "*", "run"]],
            "folder": [["working-directory"], ["defaults", "run", "working-directory"]]
        }],
        "configEntries": [{
            "file": "vitest.config.{ts,mts}",
            "property": ["test", "include"],
            "value": "globs",
            "relativeTo": "config",
            "default": [],
            "options": { "program": "vitest", "names": ["--config", "-c"], "relativeTo": "script" }
        }]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/web"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let ci = commanded(".github/workflows/ci.yml");
    let two = commanded(".github/workflows/two.yml");

    // Each file a step starts with `node` is an entry point, found from the step's own folder, else its
    // job's default, else the workflow's; a comment, a quoted text spanning a line end and a word holding
    // `$` start nothing, a continued line is one command, and a stray quote loses only its own line. A
    // configuration file named by `--config` in a step is read from the step's folder (comparison spec
    // CI-1, CI-2).
    assert_eq!(
        entries(&document),
        [
            ("first.mjs", vec![two.clone()]),
            (
                "packages/web/e2e/x.e2e.ts",
                vec![configured(Some("packages/web/vitest.e2e.config.ts"))]
            ),
            ("packages/web/scripts/build.mjs", vec![ci.clone()]),
            ("packages/web/scripts/mapped.mjs", vec![ci.clone()]),
            ("packages/web/vitest.e2e.config.ts", vec![configured(None)]),
            ("scripts/a.mjs", vec![ci.clone()]),
            ("scripts/after.mjs", vec![ci.clone()]),
            ("scripts/b.mjs", vec![ci.clone()]),
            ("scripts/c.mjs", vec![ci.clone()]),
            ("second.mjs", vec![two]),
            ("tools/check.mjs", vec![ci.clone()]),
            ("tools/list-a.mjs", vec![ci.clone()]),
            ("tools/list-b.mjs", vec![ci]),
        ]
    );

    // A file past the bound, one that is not valid YAML, and a folder the analyzer cannot know each give
    // one notice and add nothing; the run goes on (comparison spec CI-3).
    let said: Vec<(&str, &str)> = document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .map(|notice| {
            (
                notice["file"].as_str().expect("a file"),
                notice["kind"].as_str().expect("a kind"),
            )
        })
        .collect();

    assert_eq!(
        said,
        [
            (".github/workflows/bomb.yaml", "command-file-not-read"),
            (".github/workflows/broken.yml", "command-file-not-read"),
            (".github/workflows/ci.yml", "command-file-not-read"),
        ]
    );
    assert_eq!(
        document["notices"][2]["sentence"],
        json!(
            "The folder `${{ matrix.dir }}` that a command of .github/workflows/ci.yml runs in is not one the \
             analyzer can know, so the commands there add no entry point."
        )
    );
}

/// The uses a document holds that count from an entry point: each with the file whose exports are
/// used, the export's name, `*` for every export, and the field or declaration that made the entry
/// point one.
fn entry_uses(document: &Value) -> Vec<(&str, &str, &str)> {
    document["uses"]
        .as_array()
        .expect("a list of uses")
        .iter()
        .filter(|used| !used["entry"].is_null())
        .map(|used| {
            (
                used["file"].as_str().expect("a file"),
                used["name"].as_str().expect("a name"),
                used["entry"]["from"].as_str().expect("a source"),
            )
        })
        .collect()
}

/// The files a document says the entry points reach, each with whether it is reached only as text.
fn reachable(document: &Value) -> Vec<(&str, bool)> {
    document["reachable"]
        .as_array()
        .expect("a list of files")
        .iter()
        .map(|file| {
            (
                file["path"].as_str().expect("a path"),
                file["text"].as_bool().expect("a mark"),
            )
        })
        .collect()
}

#[test]
fn an_entry_points_exports_are_all_used_through_its_re_exports_too() {
    let project = Project::new("entries-uses");

    project
        .write(
            "package.json",
            r#"{ "name": "lib", "main": "src/index.ts", "bin": "src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "export { helper } from './helper';\nexport * from './all';\nexport const own = 1;\n",
        )
        .write(
            "src/helper.ts",
            "export const helper = 1;\nexport const internal = 2;\n",
        )
        .write("src/all.ts", "export const a = 1;\nexport default 2;\n")
        .write("src/unused.ts", "export const dead = 1;\n")
        .commit();

    let document = project.graph();

    // Whatever starts an entry point consumes its exports, once for each source that made it one, and
    // that use passes on through its re-exports: to the one name of `./helper`, and to every name of
    // `./all` but `default` (spec USE-3).
    assert_eq!(
        entry_uses(&document),
        [
            ("src/all.ts", "a", "bin"),
            ("src/all.ts", "a", "main"),
            ("src/helper.ts", "helper", "bin"),
            ("src/helper.ts", "helper", "main"),
            ("src/index.ts", "*", "bin"),
            ("src/index.ts", "*", "main"),
        ]
    );
    assert_eq!(
        reachable(&document),
        [
            ("src/all.ts", false),
            ("src/helper.ts", false),
            ("src/index.ts", false),
        ]
    );
}

#[test]
fn reaches_files_through_every_form_of_import_and_reads_one_imported_with_a_query_as_text() {
    let project = Project::new("entries-reach");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "src/main.ts" }"#,
        )
        .write(
            "src/main.ts",
            "import type { T } from './types';\n\
             import raw from './shader.ts?raw';\n\
             import both from './both.ts?raw';\n\
             import './both';\n\
             import App from './App.vue';\n\
             const lazy = () => import('./lazy');\n\
             const old = require('./old.cjs');\n\
             export * from './again';\n\
             export const value: T = 'x';\n\
             console.log(raw, both, App, lazy, old);\n",
        )
        .write("src/types.ts", "export type T = string;\n")
        .write(
            "src/shader.ts",
            "export * from './never';\nexport const shade = 1;\n",
        )
        .write("src/never.ts", "export const never = 1;\n")
        .write("src/both.ts", "import './deep';\nexport const both = 1;\n")
        .write("src/deep.ts", "")
        .write(
            "src/App.vue",
            "<script src=\"./logic.ts\"></script>\n<template><p /></template>\n",
        )
        .write("src/logic.ts", "export default {};\n")
        .write("src/lazy.ts", "")
        .write("src/old.cjs", "")
        .write("src/again.ts", "")
        .write(
            "index.html",
            "<script type=\"module\" src=\"./src/page.ts\"></script>\n",
        )
        .write("src/page.ts", "")
        .write("src/orphan.ts", "import './island';\n")
        .write("src/island.ts", "")
        .commit();

    let document = project.graph();

    // Every import that leads to files is followed, of every form: a type import, a call, a re-export,
    // a component's `src` and a page's script. A file only imports with a query reach is read as text,
    // so the file it imports is not reached; one an import without a query also reaches is read as
    // code. A file no entry point reaches is not here (spec USE-6).
    assert_eq!(
        reachable(&document),
        [
            ("src/App.vue", false),
            ("src/again.ts", false),
            ("src/both.ts", false),
            ("src/deep.ts", false),
            ("src/lazy.ts", false),
            ("src/logic.ts", false),
            ("src/main.ts", false),
            ("src/old.cjs", false),
            ("src/page.ts", false),
            ("src/shader.ts", true),
            ("src/types.ts", false),
        ]
    );

    let uses = |path: &str| -> Vec<Value> {
        document["uses"]
            .as_array()
            .expect("a list of uses")
            .iter()
            .filter(|used| used["file"] == json!(path))
            .map(|used| json!([used["name"], used["by"], used["line"]]))
            .collect()
    };

    // Read as text, a file's exports are all used, and it passes nothing on.
    assert_eq!(uses("src/shader.ts"), [json!(["*", "src/main.ts", 2])]);
    assert_eq!(uses("src/never.ts"), Vec::<Value>::new());
    // The script a component's block names is the component's own.
    assert_eq!(uses("src/logic.ts"), [json!(["*", "src/App.vue", 1])]);
}
