//! Runs the built binary on made-up projects and checks where relative imports lead, and every kind of
//! outside (spec RES-1 to RES-4, RES-8, RES-10, LIST-5, LIST-8, PACK-6, FIX-2, FIX-3, FIX-4).

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

    for key in ["files", "outside", "package", "listed", "ignored"] {
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
