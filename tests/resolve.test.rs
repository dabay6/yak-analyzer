//! Runs the built binary on made-up projects and checks where imports lead: relative ones, packages of
//! the project by name, `#` names, and every kind of outside (spec RES-1 to RES-6, RES-8, RES-10, MAP-1
//! to MAP-4, LIST-5, LIST-8, PACK-3, PACK-5, PACK-6, FIX-2, FIX-3, FIX-4).

#[path = "support/project.rs"]
mod project;

use std::fs;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The import of a file, by its specifier, as `graph` prints it.
fn import<'a>(document: &'a Value, path: &str, specifier: &str) -> &'a Value {
    document["files"]
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
        .unwrap_or_else(|| panic!("{path} imports {specifier}"))
}

/// What an import leads to: its outcome, and the files, the place or the ignored path that go with it.
fn led(document: &Value, path: &str, specifier: &str) -> Value {
    let import = import(document, path, specifier);
    let mut led = json!({ "outcome": import["outcome"], "rule": import["rule"] });

    for key in ["files", "outside", "package", "listed", "ignored", "from"] {
        if let Some(value) = import.get(key) {
            led[key] = value.clone();
        }
    }

    led
}

/// An import that leads to the files given, by a relative specifier.
fn files(found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": "relative", "files": found })
}

#[test]
fn finds_a_file_for_each_way_a_relative_path_is_written() {
    let project = Project::new("resolve-relative");

    project
        .write(
            "src/main.ts",
            "import './a.js';\nimport './b.ts';\nimport './types';\nimport './dir';\nimport './c.mjs';\n\
             import '../lib/d';\nimport './styles.css';\nimport './notes.md?raw';\nimport '.';\n",
        )
        .write("src/a.ts", "")
        .write("src/b.ts", "")
        .write("src/types.d.ts", "")
        .write("src/dir/index.tsx", "")
        .write("src/c.mts", "")
        .write("lib/d.js", "")
        .write("src/styles.css", "")
        .write("src/notes.md", "")
        .write("src/index.ts", "")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(main("./a.js"), files(&["src/a.ts"]));
    assert_eq!(main("./b.ts"), files(&["src/b.ts"]));
    assert_eq!(main("./types"), files(&["src/types.d.ts"]));
    assert_eq!(main("./dir"), files(&["src/dir/index.tsx"]));
    assert_eq!(main("./c.mjs"), files(&["src/c.mts"]));
    assert_eq!(main("../lib/d"), files(&["lib/d.js"]));
    assert_eq!(main("./styles.css"), files(&["src/styles.css"]));
    assert_eq!(main("./notes.md?raw"), files(&["src/notes.md"]));
    assert_eq!(main("."), files(&["src/index.ts"]));
}

#[test]
fn leads_to_both_a_script_and_the_declaration_file_beside_it() {
    let project = Project::new("resolve-pair");

    project
        .write(
            "src/main.ts",
            "import './plain.js';\nimport './plain';\nimport './typed.d.ts';\n",
        )
        .write("src/plain.js", "")
        .write("src/plain.d.ts", "")
        .write("src/typed.mjs", "")
        .write("src/typed.d.mts", "")
        .write("src/typed.d.ts", "")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(
        main("./plain.js"),
        files(&["src/plain.d.ts", "src/plain.js"])
    );
    assert_eq!(main("./plain"), files(&["src/plain.d.ts", "src/plain.js"]));
    assert_eq!(main("./typed.d.ts"), files(&["src/typed.d.ts"]));
}

#[test]
fn tells_an_ignored_path_from_a_missing_one() {
    let project = Project::new("resolve-ignored");

    project
        .write(".gitignore", "src/generated/\n*.js\n")
        .write("src/types/.gitignore", "*.d.ts\n")
        .write(
            "src/main.ts",
            "import './generated/api';\nimport './fooo';\nimport './fooo.js';\nimport './types/built';\n\
             import './missing';\nimport './Main';\n",
        )
        .commit();

    let run = |label: &str| {
        let document = project.graph();
        let main = |specifier: &str| led(&document, "src/main.ts", specifier);

        assert_eq!(
            main("./generated/api"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/generated/api" }),
            "{label}"
        );
        // A mistyped import with no extension stays unresolved though every `.js` file is ignored.
        assert_eq!(
            main("./fooo"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
        // Written with `.js`, the path itself is ignored, so the import is (spec RES-4).
        assert_eq!(
            main("./fooo.js"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/fooo.js" }),
            "{label}"
        );
        // An import with no extension of a file git ignores by its ending.
        assert_eq!(
            main("./types/built"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/types/built.d.ts" }),
            "{label}"
        );
        assert_eq!(
            main("./missing"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
        // Letter case counts on every platform (spec RES-10).
        assert_eq!(
            main("./Main"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
    };

    // The same with and without the ignored files on disk (spec AN-5, FIX-3).
    run("without the ignored files");
    project
        .write("src/generated/api.ts", "")
        .write("src/fooo.js", "")
        .write("src/types/built.d.ts", "");
    run("with the ignored files");
}

#[test]
fn names_each_place_outside_the_project() {
    let other = Project::new("resolve-outside-other");

    other.write("other.ts", "").commit();

    let project = Project::new("resolve-outside");

    project
        .write("package.json", r#"{ "dependencies": { "react": "^19.0.0" } }"#)
        .write(
            "src/main.ts",
            "import 'node:fs';\nimport 'fs/promises';\nimport 'react';\nimport 'react-dom/client';\n\
             import '@scope/pkg/deep';\nimport '/abs/file.js';\nimport 'https://cdn.example/x.js';\n\
             import 'C:/drive/file.js';\nimport '../nested/inner';\nimport '../sub/other';\nimport './link-to-main';\n\
             import './linked-folder/inner';\n",
        )
        .commit()
        .write("nested/inner.ts", "");
    project.git(&["-C", "nested", "init", "--quiet"]);
    project.git(&[
        "submodule",
        "--quiet",
        "add",
        other.root.to_str().expect("valid UTF-8"),
        "sub",
    ]);
    project.git(&["commit", "--quiet", "--message", "a submodule"]);

    let source = project.home.join("link-content");

    fs::write(&source, "src/main.ts").expect("the content can be written");

    let blob = project.git(&["hash-object", "-w", source.to_str().expect("valid UTF-8")]);

    fs::write(&source, "../lib").expect("the content can be written");

    let folder = project.git(&["hash-object", "-w", source.to_str().expect("valid UTF-8")]);

    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("120000,{},src/link-to-main", blob.trim()),
    ]);
    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("120000,{},src/linked-folder", folder.trim()),
    ]);
    project.git(&["commit", "--quiet", "--message", "two links"]);
    project.git(&["checkout", "--", "src/link-to-main", "src/linked-folder"]);

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);
    let place =
        |rule: &str, place: &str| json!({ "outcome": "outside", "rule": rule, "outside": place });
    let package = |name: &str, listed: bool| json!({ "outcome": "outside", "rule": "outside", "outside": "package", "package": name, "listed": listed });

    assert_eq!(main("node:fs"), place("outside", "builtin"));
    assert_eq!(main("fs/promises"), place("outside", "builtin"));
    assert_eq!(main("react"), package("react", true));
    assert_eq!(main("react-dom/client"), package("react-dom", false));
    assert_eq!(main("@scope/pkg/deep"), package("@scope/pkg", false));
    assert_eq!(main("/abs/file.js"), place("outside", "elsewhere"));
    assert_eq!(
        main("https://cdn.example/x.js"),
        place("outside", "elsewhere")
    );
    assert_eq!(main("C:/drive/file.js"), place("outside", "elsewhere"));
    assert_eq!(
        main("../nested/inner"),
        place("relative", "other-repository")
    );
    assert_eq!(main("../sub/other"), place("relative", "other-repository"));
    assert_eq!(main("./link-to-main"), place("relative", "link"));
    // A path through a linked folder is through a link (spec LIST-3).
    assert_eq!(main("./linked-folder/inner"), place("relative", "link"));
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
}

#[test]
fn leads_a_path_above_a_sub_folder_root_outside() {
    let project = Project::new("resolve-above");

    project
        .write("shared/util.ts", "")
        .write(
            "apps/web/src/main.ts",
            "import '../../../shared/util';\nimport '../lib';\n",
        )
        .write("apps/web/lib/index.ts", "")
        .commit();

    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(paths(&document), ["lib/index.ts", "src/main.ts"]);
    assert_eq!(
        led(&document, "src/main.ts", "../../../shared/util"),
        json!({ "outcome": "outside", "rule": "relative", "outside": "above-root" })
    );
    assert_eq!(
        led(&document, "src/main.ts", "../lib"),
        files(&["lib/index.ts"])
    );
}

#[test]
fn stops_on_a_dependency_list_of_a_form_it_does_not_have() {
    let project = Project::new("resolve-dependencies-form");

    project
        .write("package.json", r#"{ "devDependencies": ["typescript"] }"#)
        .commit();

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `devDependencies` in package.json holds a value of a form it does not have; \
         it must be an object that maps each package's name to the version it asks for; fix it"
    );
}

/// An import that leads to the files given, by the name of a package of the project.
fn by_name(found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": "package", "files": found })
}

/// A workspace whose one package offers its build output through `exports`, which the worktree does
/// not have, and an app that imports it by name (spec FIX-2, pattern 10).
fn workspace_with_build_output(name: &str) -> Project {
    let project = Project::new(name);

    project
        .write(".gitignore", "node_modules/\ndist/\n")
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*", "apps/*"] }"#,
        )
        .write(
            "packages/lib/package.json",
            r#"{
                "name": "@demo/lib",
                "exports": {
                    ".": { "types": "./dist/index.d.ts", "import": "./dist/index.js" },
                    "./util": "./dist/util.js",
                    "./icons/*": "./dist/icons/*/*.js",
                    "./list": ["./dist/list.js", "./dist/fallback.js"],
                    "./platform": { "browser": "./dist/browser.js", "node": "./dist/node.js" },
                    "./typed": { "types": "./dist/typed.d.ts", "default": "./dist/main.js" },
                    "./generated": "./dist/generated.js",
                    "./hidden": null
                }
            }"#,
        )
        .write("packages/lib/src/index.ts", "")
        .write("packages/lib/src/util.ts", "")
        .write("packages/lib/src/icons/star/star.ts", "")
        .write("packages/lib/src/list.ts", "")
        .write("packages/lib/src/browser.ts", "")
        .write("packages/lib/src/node.ts", "")
        .write("packages/lib/src/main.ts", "")
        .write(
            "apps/web/package.json",
            r#"{ "name": "web", "dependencies": { "@demo/lib": "workspace:*" } }"#,
        )
        .write(
            "apps/web/src/main.ts",
            "import '@demo/lib';\nimport '@demo/lib/util';\nimport '@demo/lib/icons/star';\n\
             import '@demo/lib/list';\nimport '@demo/lib/platform';\nimport '@demo/lib/typed';\n\
             import '@demo/lib/generated';\nimport '@demo/lib/hidden';\nimport '@demo/lib/missing';\n",
        )
        .commit();

    project
}

#[test]
fn leads_an_import_by_a_workspace_packages_name_through_its_exports_to_source() {
    let project = workspace_with_build_output("resolve-exports");

    let run = |label: &str| {
        let document = project.graph();
        let web = |specifier: &str| led(&document, "apps/web/src/main.ts", specifier);

        // Two conditions that name build output of one source lead to that one file (spec RES-5).
        assert_eq!(
            web("@demo/lib"),
            by_name(&["packages/lib/src/index.ts"]),
            "{label}"
        );
        assert_eq!(
            web("@demo/lib/util"),
            by_name(&["packages/lib/src/util.ts"]),
            "{label}"
        );
        // A `*` written twice in a target is filled in at both places (spec RES-6).
        assert_eq!(
            web("@demo/lib/icons/star"),
            by_name(&["packages/lib/src/icons/star/star.ts"]),
            "{label}"
        );
        // A list is followed entry by entry; the entry with no source adds nothing (spec PACK-6).
        assert_eq!(
            web("@demo/lib/list"),
            by_name(&["packages/lib/src/list.ts"]),
            "{label}"
        );
        // Conditions that name two different sources lead to both.
        assert_eq!(
            web("@demo/lib/platform"),
            by_name(&["packages/lib/src/browser.ts", "packages/lib/src/node.ts"]),
            "{label}"
        );
        // Conditions whose first target has no source lead to what the others find.
        assert_eq!(
            web("@demo/lib/typed"),
            by_name(&["packages/lib/src/main.ts"]),
            "{label}"
        );
        // A target in an ignored build folder that maps to no source is ignored, with the target it
        // came from (spec MAP-4).
        assert_eq!(
            web("@demo/lib/generated"),
            json!({
                "outcome": "ignored",
                "rule": "package",
                "ignored": "packages/lib/dist/generated.js",
                "from": "packages/lib/dist/generated.js",
            }),
            "{label}"
        );
        // A subpath `exports` holds back, or does not offer, leads nowhere (spec RES-6).
        assert_eq!(
            web("@demo/lib/hidden"),
            json!({ "outcome": "unresolved", "rule": "package" }),
            "{label}"
        );
        assert_eq!(
            web("@demo/lib/missing"),
            json!({ "outcome": "unresolved", "rule": "package" }),
            "{label}"
        );
    };

    // The same with and without `node_modules` and the build output on disk (spec AN-5, FIX-3).
    run("without node_modules and build output");
    project
        .write(
            "node_modules/@demo/lib/package.json",
            r#"{ "name": "@demo/lib", "main": "./elsewhere.js" }"#,
        )
        .write("node_modules/@demo/lib/elsewhere.js", "")
        .write("packages/lib/dist/index.js", "")
        .write("packages/lib/dist/index.d.ts", "")
        .write("packages/lib/dist/generated.js", "");
    run("with node_modules and build output");
}

#[test]
fn looks_up_a_hash_name_in_the_imports_of_the_files_package() {
    let project = Project::new("resolve-hash");

    project
        .write(".gitignore", "dist/\nsrc/generated/\n")
        .write(
            "package.json",
            r##"{
                "name": "app",
                "dependencies": { "react": "^19.0.0" },
                "imports": {
                    "#lib/*": { "types": "./dist/lib/*.d.ts", "default": "./dist/lib/*.js" },
                    "#lib/special": "./dist/special.js",
                    "#config": "./config.js",
                    "#react": "react",
                    "#generated": "./src/generated/api.ts"
                }
            }"##,
        )
        .write("config.js", "")
        .write("src/lib/a.ts", "")
        .write("src/special.ts", "")
        .write(
            "src/main.ts",
            "import '#lib/a';\nimport '#lib/special';\nimport '#config';\nimport '#react';\n\
             import '#generated';\nimport '#missing';\n",
        )
        .write("tools/package.json", r#"{ "name": "tools" }"#)
        .write("tools/run.ts", "import '#config';\n")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);
    let imports = |found: &[&str]| json!({ "outcome": "files", "rule": "imports", "files": found });

    assert_eq!(main("#lib/a"), imports(&["src/lib/a.ts"]));
    // A key with no `*` wins over a pattern that also matches (spec RES-5).
    assert_eq!(main("#lib/special"), imports(&["src/special.ts"]));
    // A committed JavaScript file is its own source (spec MAP-2).
    assert_eq!(main("#config"), imports(&["config.js"]));
    // A target that is a package's name leads where an import of that name does.
    assert_eq!(
        main("#react"),
        json!({ "outcome": "outside", "rule": "imports", "outside": "package", "package": "react", "listed": true })
    );
    assert_eq!(
        main("#generated"),
        json!({
            "outcome": "ignored",
            "rule": "imports",
            "ignored": "src/generated/api.ts",
            "from": "src/generated/api.ts",
        })
    );
    assert_eq!(
        main("#missing"),
        json!({ "outcome": "unresolved", "rule": "imports" })
    );
    // A `#` name is looked up in the importing file's own package only.
    assert_eq!(
        led(&document, "tools/run.ts", "#config"),
        json!({ "outcome": "unresolved", "rule": "imports" })
    );
}

#[test]
fn leads_a_package_with_no_exports_through_its_entries_its_index_or_a_path_inside_it() {
    let project = Project::new("resolve-entries");

    project
        .write(".gitignore", "dist/\n")
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )
        .write(
            "packages/typed/package.json",
            r#"{ "name": "typed", "typings": "./dist/index.d.ts", "main": "dist/index.cjs" }"#,
        )
        .write("packages/typed/src/index.ts", "")
        .write("packages/typed/src/helper.ts", "")
        .write("packages/plain/package.json", r#"{ "name": "plain" }"#)
        .write("packages/plain/index.js", "")
        .write(
            "packages/both/package.json",
            r#"{ "name": "both", "module": "./esm/index.js", "main": "./cjs/index.js" }"#,
        )
        .write("packages/both/esm/index.js", "")
        .write("packages/both/cjs/index.js", "")
        .write(
            "src/main.ts",
            "import 'typed';\nimport 'typed/src/helper';\nimport 'typed/dist/helper.js';\nimport 'plain';\n\
             import 'both';\nimport '../packages/typed';\n",
        )
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    // `typings` is the older name of `types`; both entries map to one source (spec PACK-6, MAP-1).
    assert_eq!(main("typed"), by_name(&["packages/typed/src/index.ts"]));
    assert_eq!(
        main("typed/src/helper"),
        by_name(&["packages/typed/src/helper.ts"])
    );
    // A subpath into build output maps to its source (spec MAP-1).
    assert_eq!(
        main("typed/dist/helper.js"),
        by_name(&["packages/typed/src/helper.ts"])
    );
    // With no entries, the bare name leads to the index file of the package's folder.
    assert_eq!(main("plain"), by_name(&["packages/plain/index.js"]));
    // Every file the entries lead to.
    assert_eq!(
        main("both"),
        by_name(&["packages/both/cjs/index.js", "packages/both/esm/index.js"])
    );
    // A folder that holds a `package.json` is tried through its entries (spec RES-3).
    assert_eq!(
        main("../packages/typed"),
        files(&["packages/typed/src/index.ts"])
    );
}

#[test]
fn leads_a_name_to_a_package_outside_the_workspace_only_by_itself_or_a_local_protocol() {
    let project = Project::new("resolve-stray");

    project
        .write(
            "package.json",
            r#"{
                "name": "root",
                "workspaces": ["packages/*"],
                "dependencies": { "react": "^19.0.0", "local-tool": "file:./tools/local" },
                "devDependencies": { "shared": "workspace:^" }
            }"#,
        )
        .write("examples/react/package.json", r#"{ "name": "react" }"#)
        .write("examples/react/index.ts", "import 'react';\n")
        .write(
            "tools/local/package.json",
            r#"{ "name": "local-tool", "main": "run.js" }"#,
        )
        .write("tools/local/run.js", "")
        .write("vendor/shared/package.json", r#"{ "name": "shared" }"#)
        .write("vendor/shared/index.ts", "")
        .write(
            "src/main.ts",
            "import 'react';\nimport 'local-tool';\nimport 'shared';\n",
        )
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    // A stray folder with an installed dependency's name does not take its imports (spec PACK-3).
    assert_eq!(
        main("react"),
        json!({ "outcome": "outside", "rule": "outside", "outside": "package", "package": "react", "listed": true })
    );
    // A package may import itself by its own name (spec RES-2, step 5).
    assert_eq!(
        led(&document, "examples/react/index.ts", "react"),
        by_name(&["examples/react/index.ts"])
    );
    assert_eq!(main("local-tool"), by_name(&["tools/local/run.js"]));
    assert_eq!(main("shared"), by_name(&["vendor/shared/index.ts"]));
}

#[test]
fn leads_a_name_two_workspace_packages_share_to_the_first_by_path() {
    let project = Project::new("resolve-repeated");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "same" }"#)
        .write("packages/a/index.ts", "")
        .write("packages/b/package.json", r#"{ "name": "same" }"#)
        .write("packages/b/index.ts", "")
        .write("src/main.ts", "import 'same';\n")
        .commit();

    let document = project.graph();

    assert_eq!(
        led(&document, "src/main.ts", "same"),
        by_name(&["packages/a/index.ts"])
    );
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "package-name-repeated"]
    );
}

#[test]
fn stops_on_a_package_field_of_a_form_it_does_not_have() {
    let cases = [
        (
            "exports",
            r#"{ "exports": { ".": "./a.js", "import": "./b.js" } }"#,
            "a path, a list, or an object whose keys are all subpaths beginning with `.` or all conditions",
        ),
        (
            "exports",
            r#"{ "exports": 1 }"#,
            "a path, a list, or an object whose keys are all subpaths beginning with `.` or all conditions",
        ),
        (
            "imports",
            r#"{ "imports": { "lib": "./lib.js" } }"#,
            "an object whose keys begin with `#` and whose values are paths, lists or conditions",
        ),
        ("types", r#"{ "types": ["./a.d.ts"] }"#, "a path"),
        ("typings", r#"{ "typings": true }"#, "a path"),
        ("module", r#"{ "module": {} }"#, "a path"),
        ("main", r#"{ "main": null }"#, "a path"),
    ];

    for (field, manifest, form) in cases {
        let project = Project::new("resolve-fields-form");

        project.write("package.json", manifest).commit();

        assert_eq!(
            stopped(&project.run(|_| {})),
            format!(
                "yak-analyzer: the field `{field}` in package.json holds a value of a form it does not have; \
                 it must be {form}; fix it"
            ),
            "{manifest}"
        );
    }
}
