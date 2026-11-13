//! Runs the built binary on made-up projects and checks how tsconfig files are read and what they do to
//! where imports lead: the config that governs a file, `paths`, `baseUrl`, `extends` above the root and
//! outside the project, and how a run stops on one it cannot read (spec CFG-1 to CFG-4, RES-2, RES-4,
//! MAP-1, MAP-3, LIST-4, LIST-8, RUN-4, FIX-2, FIX-3, FIX-4).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// What an import leads to: its outcome and rule, and the files, the place or the ignored path that go
/// with it.
fn led(document: &Value, path: &str, specifier: &str) -> Value {
    let import = document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .and_then(|file| file["imports"].as_array())
        .and_then(|imports| {
            imports
                .iter()
                .find(|import| import["specifier"] == json!(specifier))
        })
        .unwrap_or_else(|| panic!("{path} imports {specifier}"));
    let mut led = json!({ "outcome": import["outcome"], "rule": import["rule"] });

    for key in ["files", "outside", "package", "listed", "ignored", "from"] {
        if let Some(value) = import.get(key) {
            led[key] = value.clone();
        }
    }

    led
}

/// An import that leads to the files given, by the rule given.
fn files(rule: &str, found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": rule, "files": found })
}

/// An import by the rule given that leads to a package outside the project.
fn package(rule: &str, name: &str, listed: bool) -> Value {
    json!({ "outcome": "outside", "rule": rule, "outside": "package", "package": name, "listed": listed })
}

#[test]
fn governs_a_file_by_the_nearest_config_that_covers_it_or_one_it_references() {
    let project = Project::new("tsconfig-governs");

    project
        .write(
            "tsconfig.json",
            r#"{
                "files": [],
                "references": [
                    { "path": "./tsconfig.app.json" },
                    { "path": "./tsconfig.node.json" },
                    { "path": "./tsconfig.missing.json" },
                ],
            }"#,
        )
        .write(
            "tsconfig.app.json",
            "// The app's own config.\n{ \"include\": [\"src\"], /* aliases */ \"compilerOptions\": { \"paths\": { \"@/*\": [\"./src/*\"] } } }\n",
        )
        .write(
            "tsconfig.node.json",
            r#"{ "include": ["scripts"], "compilerOptions": { "baseUrl": "." } }"#,
        )
        .write("src/lib/util.ts", "")
        .write("src/main.ts", "import '@/lib/util';\n")
        .write("src/legacy.js", "import '@/lib/util';\n")
        .write("scripts/build.ts", "import 'src/lib/util';\nimport 'src/lib/missing';\n")
        .write("other/x.ts", "import '@/lib/util';\n")
        .write("web/jsconfig.json", r#"{ "compilerOptions": { "paths": { "~/*": ["./*"] } } }"#)
        .write("web/util.js", "")
        .write("web/app.js", "import '~/util';\n")
        .commit();

    let document = project.graph();

    // A config that `references` names governs the files it covers (spec CFG-2).
    assert_eq!(
        led(&document, "src/main.ts", "@/lib/util"),
        files("paths", &["src/lib/util.ts"])
    );
    // A file that is not TypeScript is covered as a `.ts` file of its name would be.
    assert_eq!(
        led(&document, "src/legacy.js", "@/lib/util"),
        files("paths", &["src/lib/util.ts"])
    );
    // A path from `baseUrl` that leads to no file goes on down the order (spec RES-2, step 4).
    assert_eq!(
        led(&document, "scripts/build.ts", "src/lib/util"),
        files("base-url", &["src/lib/util.ts"])
    );
    assert_eq!(
        led(&document, "scripts/build.ts", "src/lib/missing"),
        package("outside", "src", false)
    );
    // A file no config covers has no `paths`.
    assert_eq!(
        led(&document, "other/x.ts", "@/lib/util"),
        package("outside", "@/lib", false)
    );
    // A `jsconfig.json` governs as a `tsconfig.json` does.
    assert_eq!(
        led(&document, "web/app.js", "~/util"),
        files("paths", &["web/util.js"])
    );
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "tsconfig-reference-missing"]
    );
}

#[test]
fn sends_an_import_a_paths_pattern_cannot_settle_on_down_the_order() {
    let project = Project::new("tsconfig-paths-miss");

    project
        .write(".gitignore", "src/generated/\n")
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"], "dependencies": { "@scope/installed": "^1.0.0" } }"#,
        )
        .write("packages/ui/package.json", r#"{ "name": "@scope/ui" }"#)
        .write("packages/ui/index.ts", "")
        .write(
            "tsconfig.json",
            r#"{
                "compilerOptions": {
                    "paths": {
                        "@app/*": ["./src/app/*"],
                        "@gen/*": ["./src/generated/*"],
                        "@scope/*": ["./src/scoped/*"],
                        "*": ["./types/*"]
                    }
                }
            }"#,
        )
        .write("src/app/found.ts", "")
        .write("types/lodash.d.ts", "")
        .write(
            "src/main.ts",
            "import '@app/found';\nimport '@app/missing';\nimport '@gen/api';\nimport '@scope/missing';\n\
             import '@scope/ui';\nimport '@scope/installed';\nimport 'lodash';\nimport 'react';\n",
        )
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(main("@app/found"), files("paths", &["src/app/found.ts"]));
    // A pattern that matches and leads to no file is unresolved, or ignored (spec RES-4).
    assert_eq!(
        main("@app/missing"),
        json!({ "outcome": "unresolved", "rule": "paths" })
    );
    assert_eq!(
        main("@gen/api"),
        json!({ "outcome": "ignored", "rule": "paths", "ignored": "src/generated/api" })
    );
    assert_eq!(
        main("@scope/missing"),
        json!({ "outcome": "unresolved", "rule": "paths" })
    );
    // A workspace package's name, and a dependency's, go on down the order.
    assert_eq!(
        main("@scope/ui"),
        files("package", &["packages/ui/index.ts"])
    );
    assert_eq!(
        main("@scope/installed"),
        package("outside", "@scope/installed", true)
    );
    // So does whatever the bare `*` cannot settle.
    assert_eq!(main("lodash"), files("paths", &["types/lodash.d.ts"]));
    assert_eq!(main("react"), package("outside", "react", false));
}

#[test]
fn maps_one_alias_to_source_by_paths_and_build_output_back_by_out_dir() {
    let project = Project::new("tsconfig-alias");

    project
        .write(".gitignore", "dist/\nout/\n")
        .write(
            "package.json",
            r##"{ "name": "app", "workspaces": ["packages/*"], "imports": { "#lib/*": "./dist/lib/*.js" } }"##,
        )
        .write(
            "tsconfig.json",
            r##"{ "include": ["src"], "compilerOptions": { "outDir": "dist", "paths": { "#lib/*": ["./src/lib/*"] } } }"##,
        )
        .write("src/lib/a.ts", "")
        .write("src/lib/b.ts", "")
        .write(
            "src/main.ts",
            "import '#lib/a';\nimport '../dist/lib/b.js';\nimport 'legacy';\n",
        )
        .write("scripts/run.mjs", "import '#lib/a';\n")
        .write(
            "packages/legacy/package.json",
            r#"{ "name": "legacy", "main": "./out/index.js" }"#,
        )
        .write(
            "packages/legacy/tsconfig.json",
            r#"{ "extends": "./tsconfig.base.json" }"#,
        )
        .write(
            "packages/legacy/tsconfig.base.json",
            r#"{ "compilerOptions": { "outDir": "out", "rootDir": "source" } }"#,
        )
        .write("packages/legacy/source/index.ts", "")
        .commit();

    let document = project.graph();

    // One alias: source by the tsconfig's `paths`, which comes first (spec FIX-2, pattern 9).
    assert_eq!(
        led(&document, "src/main.ts", "#lib/a"),
        files("paths", &["src/lib/a.ts"])
    );
    // Build output by `imports`, for a file the tsconfig does not cover, mapped back by its `outDir`.
    assert_eq!(
        led(&document, "scripts/run.mjs", "#lib/a"),
        files("imports", &["src/lib/a.ts"])
    );
    // A relative import into the build folder (spec MAP-1, step 1).
    assert_eq!(
        led(&document, "src/main.ts", "../dist/lib/b.js"),
        files("relative", &["src/lib/b.ts"])
    );
    // An `outDir` and a `rootDir` taken through `extends` (spec MAP-3).
    assert_eq!(
        led(&document, "src/main.ts", "legacy"),
        files("package", &["packages/legacy/source/index.ts"])
    );
}

#[test]
fn reads_a_config_above_a_sub_folder_root_that_one_under_it_extends() {
    let project = Project::new("tsconfig-above");

    project
        .write(
            "tsconfig.base.json",
            r#"{ "compilerOptions": { "paths": { "@shared/*": ["./shared/*"] } } }"#,
        )
        .write(
            "apps/web/tsconfig.json",
            r#"{ "extends": "../../tsconfig.base.json", "include": ["src"] }"#,
        )
        .write(
            "apps/web/src/main.ts",
            "import '@shared/util';\nimport '../../../shared/util';\n",
        )
        .write("shared/util.ts", "")
        .commit();

    // From the repository's root, both lead to the shared file (spec FIX-2, pattern 15).
    let document = project.graph();

    assert_eq!(
        led(&document, "apps/web/src/main.ts", "@shared/util"),
        files("paths", &["shared/util.ts"])
    );
    assert_eq!(
        led(&document, "apps/web/src/main.ts", "../../../shared/util"),
        files("relative", &["shared/util.ts"])
    );

    // From the sub-folder, the config above it is read, and both lead above the root (spec LIST-8).
    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let above = |rule: &str| json!({ "outcome": "outside", "rule": rule, "outside": "above-root" });

    // Nothing above the root is listed, the config read there included.
    assert_eq!(paths(&document), ["src/main.ts", "tsconfig.json"]);

    assert_eq!(
        led(&document, "src/main.ts", "@shared/util"),
        above("paths")
    );
    assert_eq!(
        led(&document, "src/main.ts", "../../../shared/util"),
        above("relative")
    );
    assert_eq!(notices(&document), Vec::<&str>::new());
}

#[test]
fn uses_a_config_without_the_config_of_a_dependency_it_extends() {
    let project = Project::new("tsconfig-dependency");

    project
        .write(".gitignore", "node_modules/\nbuild/\n")
        .write(
            "tsconfig.json",
            r#"{ "extends": "@tsconfig/node22/tsconfig.json", "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#,
        )
        .write("src/a.ts", "")
        .write("src/main.ts", "import '@/a';\nimport './build/out.js';\n")
        .commit();

    let run = |label: &str| {
        let document = project.graph();

        assert_eq!(
            led(&document, "src/main.ts", "@/a"),
            files("paths", &["src/a.ts"]),
            "{label}"
        );
        assert_eq!(
            led(&document, "src/main.ts", "./build/out.js"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/build/out.js" }),
            "{label}"
        );
        assert_eq!(
            document["notices"][1],
            json!({
                "file": "tsconfig.json",
                "kind": "tsconfig-extends-left-out",
                "sentence": "The tsconfig extends `@tsconfig/node22/tsconfig.json`, which is not in the project, so it is \
                             used without it.",
            }),
            "{label}"
        );
    };

    // The same with and without the dependency's config and the build output on disk (spec FIX-3).
    run("without node_modules");
    project
        .write(
            "node_modules/@tsconfig/node22/tsconfig.json",
            r#"{ "compilerOptions": { "paths": { "@/*": ["./elsewhere/*"] } } }"#,
        )
        .write("src/build/out.js", "");
    run("with node_modules");
}

#[test]
fn reads_no_tsconfig_declared_as_data() {
    let project = Project::new("tsconfig-data");

    project
        .write("fixtures/broken/tsconfig.json", "{ not json")
        .write("fixtures/broken/case.ts", "import '@/x';\n")
        .write(
            "fixtures/working/tsconfig.json",
            r#"{ "compilerOptions": { "paths": { "@/*": ["./*"] } } }"#,
        )
        .write("fixtures/working/case.ts", "")
        .write("src/main.ts", "import '../fixtures/working/case';\n")
        .commit();

    let declared = json!([{
        "name": "typescript",
        "interfaceVersion": 1,
        "pack": {},
        "project": { "data": ["fixtures/**"] },
    }]);
    let file = project.hand_off(&project.root, &["."], declared);
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // Data is never read, so a broken config among it stops nothing (spec FIX-2, pattern 5, KEY-4).
    assert_eq!(
        led(&document, "src/main.ts", "../fixtures/working/case"),
        files("relative", &["fixtures/working/case.ts"])
    );
    assert_eq!(notices(&document), Vec::<&str>::new());
}

#[test]
fn stops_on_a_config_a_file_depends_on_that_cannot_be_read_and_skips_any_other() {
    let project = Project::new("tsconfig-unreadable");

    project
        .write("tsconfig.json", "{ \"include\": [\"src\"] }")
        .write("tsconfig.lint.json", "{ broken")
        .write("src/a.ts", "")
        .commit();

    // A config no file depends on is left out, with a notice (spec CFG-4).
    let document = project.graph();

    assert_eq!(
        document["notices"][1],
        json!({
            "file": "tsconfig.lint.json",
            "kind": "tsconfig-skipped",
            "sentence": "The tsconfig cannot be read (line 1, column 3), and no file the analyzer reads depends on it, \
                         so it is left out.",
        })
    );

    // The config in a folder above a file the analyzer reads.
    project.write("tsconfig.json", "{ \"include\": [\"src\"], }}");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig tsconfig.json cannot be read (line 1, column 24); fix it, or declare it as data if \
         it is broken on purpose"
    );

    // A config it extends.
    project
        .write("tsconfig.json", r#"{ "extends": "./tsconfig.lint.json" }"#)
        .write("tsconfig.lint.json", "{ broken");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig tsconfig.lint.json cannot be read (line 1, column 3); fix it, or declare it as \
         data if it is broken on purpose"
    );

    // A file it extends that is not there.
    project.write("tsconfig.json", r#"{ "extends": "./base" }"#);

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig tsconfig.json extends `./base`, which should be the file base and is not there; \
         fix its `extends`, or add the file"
    );
}

#[test]
fn stops_on_a_config_field_of_a_form_it_does_not_have_after_any_config_it_cannot_read() {
    let project = Project::new("tsconfig-fields");

    project
        .write("package.json", r#"{ "main": 1 }"#)
        .write("a/tsconfig.json", r#"{ "include": "src" }"#)
        .write("a/src/x.ts", "")
        .commit();

    // Of two fields of a form they do not have, the first by path: `a/tsconfig.json` before
    // `package.json` (spec RUN-4).
    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `include` in a/tsconfig.json holds a value of a form it does not have; it must be a \
         list of paths; fix it"
    );

    // A config that cannot be read is reported before any field (spec RUN-4, step 10).
    project.write("b/tsconfig.json", "{").write("b/y.ts", "");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig b/tsconfig.json cannot be read (line 1, column 1); fix it, or declare it as data \
         if it is broken on purpose"
    );

    let cases = [
        (
            r#"{ "extends": 1 }"#,
            "extends",
            "a path, or a list of paths",
        ),
        (r#"{ "files": [1] }"#, "files", "a list of paths"),
        (r#"{ "exclude": {} }"#, "exclude", "a list of paths"),
        (
            r#"{ "references": ["./a"] }"#,
            "references",
            "a list of objects, each with a `path`",
        ),
        (
            r#"{ "compilerOptions": [] }"#,
            "compilerOptions",
            "an object",
        ),
        (
            r#"{ "compilerOptions": { "paths": { "@/*": "./src/*" } } }"#,
            "compilerOptions.paths",
            "an object that maps each pattern to a list of paths",
        ),
        (
            r#"{ "compilerOptions": { "baseUrl": true } }"#,
            "compilerOptions.baseUrl",
            "a path",
        ),
        (
            r#"{ "compilerOptions": { "outDir": 1 } }"#,
            "compilerOptions.outDir",
            "a path",
        ),
        (
            r#"{ "compilerOptions": { "rootDir": null } }"#,
            "compilerOptions.rootDir",
            "a path",
        ),
    ];

    for (config, field, form) in cases {
        let project = Project::new("tsconfig-field-form");

        project.write("tsconfig.json", config).commit();

        assert_eq!(
            stopped(&project.run(|_| {})),
            format!(
                "yak-analyzer: the field `{field}` in tsconfig.json holds a value of a form it does not have; it must \
                 be {form}; fix it"
            ),
            "{config}"
        );
    }
}
