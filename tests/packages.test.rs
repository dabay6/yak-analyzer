//! Runs the built binary on made-up projects and checks the `data` and `generated` declarations and the
//! packages it prints, and how it stops on a standard file it cannot read (spec KEY-3, KEY-4, KEY-5,
//! PACK-1 to PACK-6, RUN-4).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// One stack named `typescript` that holds the declarations given, the pack's and the project's.
fn stack(pack: Value, project: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": project }])
}

/// The files a document marks as the key given, `data` or `generated`.
fn marked<'a>(document: &'a Value, key: &str) -> Vec<&'a str> {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .filter(|file| file[key] == json!(true))
        .map(|file| file["path"].as_str().expect("a path"))
        .collect()
}

/// The package each file belongs to, by the file's path.
fn package_of<'a>(document: &'a Value, path: &str) -> &'a Value {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .map_or(&Value::Null, |file| &file["package"])
}

#[test]
fn marks_data_and_generated_files_a_packs_globs_from_each_folder_and_a_projects_from_the_root() {
    let project = Project::new("marks");

    project
        .write("fixtures/a.json", "")
        .write("fixtures/broken/package.json", "{ not json")
        .write("fixtures/broken/tsconfig.json", "{ not json")
        .write("packages/app/fixtures/b.ts", "")
        .write("packages/app/src/x.ts", "")
        .write("other/fixtures/c.ts", "")
        .write("src/gen/types.ts", "")
        .write("src/main.ts", "")
        .commit();

    let declared = stack(
        json!({ "data": ["fixtures/**"], "generated": ["**/*.never"] }),
        json!({ "generated": ["src/gen/**"] }),
    );
    let file = project.hand_off(&project.root, &[".", "packages/app"], declared);
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // The pack's glob applies under each folder detection looked in, and not under `other/`.
    assert_eq!(
        marked(&document, "data"),
        [
            "fixtures/a.json",
            "fixtures/broken/package.json",
            "fixtures/broken/tsconfig.json",
            "packages/app/fixtures/b.ts"
        ]
    );
    assert_eq!(marked(&document, "generated"), ["src/gen/types.ts"]);

    // A broken `package.json` among data makes no package and stops nothing (spec KEY-4).
    assert_eq!(document["packages"], json!([]));
    assert_eq!(notices(&document), ["glob-matches-nothing"]);
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The `generated` glob `**/*.never` that the pack states for the stack `typescript` matches no file."
        )
    );
}

#[test]
fn makes_a_package_of_every_package_json_and_names_the_workspace_ones() {
    let project = Project::new("packages-npm");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*", "./tools/", "!packages/excluded"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write("packages/a/src/x.ts", "")
        .write(
            "packages/excluded/package.json",
            r#"{ "name": "excluded" }"#,
        )
        .write("tools/package.json", "\u{feff}{}")
        .write("examples/demo/package.json", r#"{ "name": "a" }"#)
        .write("examples/demo/index.ts", "")
        .write("README.md", "")
        .commit();

    let document = project.graph();
    let package = |folder: &str, name: Value, workspace: bool, detected: bool| {
        json!({
            "detected": detected,
            "folder": folder,
            "name": name,
            "workspace": workspace,
        })
    };

    // Without a hand-off file the detected folders are the root and the workspace's (spec PACK-4).
    assert_eq!(
        document["packages"],
        json!([
            package(".", json!("root"), true, true),
            package("examples/demo", json!("a"), false, false),
            package("packages/a", json!("a"), true, true),
            package("packages/excluded", json!("excluded"), false, false),
            package("tools", Value::Null, true, true),
        ])
    );

    // A stray package of a workspace package's name is no workspace package, so it is no repeat.
    assert_eq!(notices(&document), ["no-hand-off-file"]);
    assert_eq!(
        package_of(&document, "packages/a/src/x.ts"),
        &json!("packages/a")
    );
    assert_eq!(
        package_of(&document, "examples/demo/index.ts"),
        &json!("examples/demo")
    );
    assert_eq!(package_of(&document, "README.md"), &json!("."));
}

#[test]
fn reads_the_workspace_from_an_object_and_from_pnpms_list() {
    let object = Project::new("packages-object");

    object
        .write(
            "package.json",
            r#"{ "workspaces": { "packages": ["libs/*"], "nohoist": ["**"] } }"#,
        )
        .write("libs/one/package.json", r#"{ "name": "one" }"#)
        .commit();

    let pnpm = Project::new("packages-pnpm");

    pnpm.write("package.json", "{}")
        .write(
            "pnpm-workspace.yaml",
            "# the workspace\npackages:\n  - 'apps/*'\n  - \"!apps/legacy\"\n",
        )
        .write("apps/web/package.json", r#"{ "name": "web" }"#)
        .write("apps/legacy/package.json", r#"{ "name": "legacy" }"#)
        .commit();

    let workspace = |document: &Value| -> Vec<String> {
        document["packages"]
            .as_array()
            .expect("a list of packages")
            .iter()
            .filter(|package| package["workspace"] == json!(true))
            .map(|package| package["folder"].as_str().expect("a folder").to_owned())
            .collect()
    };

    assert_eq!(workspace(&object.graph()), [".", "libs/one"]);
    assert_eq!(workspace(&pnpm.graph()), [".", "apps/web"]);
}

#[test]
fn takes_the_folders_a_hand_off_file_lists_as_detected_and_as_workspace_packages() {
    let project = Project::new("packages-detection");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write("apps/web/package.json", r#"{ "name": "web" }"#)
        .commit();

    let file = project.hand_off(&project.root, &[".", "apps/web"], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let facts: Vec<(&str, bool, bool)> = document["packages"]
        .as_array()
        .expect("a list of packages")
        .iter()
        .map(|package| {
            (
                package["folder"].as_str().expect("a folder"),
                package["workspace"] == json!(true),
                package["detected"] == json!(true),
            )
        })
        .collect();

    assert_eq!(
        facts,
        [
            (".", true, true),
            ("apps/web", true, true),
            ("packages/a", true, false)
        ]
    );
}

#[test]
fn says_when_two_workspace_packages_have_one_name() {
    let project = Project::new("packages-repeated");

    project
        .write("package.json", r#"{ "workspaces": ["packages/*"] }"#)
        .write("packages/first/package.json", r#"{ "name": "same" }"#)
        .write("packages/second/package.json", r#"{ "name": "same" }"#)
        .commit();

    let document = project.graph();

    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "package-name-repeated"]
    );
    assert_eq!(
        document["notices"][1],
        json!({
            "file": "packages/second/package.json",
            "kind": "package-name-repeated",
            "sentence": "The workspace packages in `packages/first` and `packages/second` are both named `same`; \
                         an import of `same` leads to the first, `packages/first`."
        })
    );
}

#[test]
fn leaves_a_file_with_no_package_above_it_in_no_package() {
    let project = Project::new("packages-none");

    project
        .write("web/package.json", "{}")
        .write("web/main.ts", "")
        .write("server/main.ts", "")
        .commit();

    let document = project.graph();

    assert_eq!(
        paths(&document),
        ["server/main.ts", "web/main.ts", "web/package.json"]
    );
    assert_eq!(package_of(&document, "server/main.ts"), &Value::Null);
    assert_eq!(package_of(&document, "web/main.ts"), &json!("web"));
}

#[test]
fn stops_on_a_package_json_or_a_workspace_list_it_cannot_read() {
    let cases: [(&str, &str, &str, &str); 4] = [
        (
            "packages-stop-json",
            "package.json",
            "{ \"name\": ",
            "yak-analyzer: the package.json package.json cannot be read as a JSON object (line 1, column 10); fix \
             it, or declare it as data if it is broken on purpose",
        ),
        (
            "packages-stop-array",
            "lib/package.json",
            "[]",
            "yak-analyzer: the package.json lib/package.json cannot be read as a JSON object (it holds something \
             other than an object); fix it, or declare it as data if it is broken on purpose",
        ),
        (
            "packages-stop-yaml",
            "pnpm-workspace.yaml",
            "packages: [\n",
            "yak-analyzer: the workspace's list pnpm-workspace.yaml cannot be read (",
        ),
        (
            "packages-stop-utf8",
            "package.json",
            "\u{0}",
            "yak-analyzer: the package.json package.json cannot be read as a JSON object (line 1, column 1)",
        ),
    ];

    for (name, path, content, expected) in cases {
        let project = Project::new(name);

        project.write(path, content).commit();

        let line = stopped(&project.run(|_| {}));

        assert!(line.starts_with(expected), "{name}: {line}");
    }
}

#[test]
fn stops_on_a_field_of_a_form_it_does_not_have() {
    let cases: [(&str, &str, &str, &str); 5] = [
        (
            "packages-field-name",
            "lib/package.json",
            r#"{ "name": 4 }"#,
            "yak-analyzer: the field `name` in lib/package.json holds a value of a form it does not have; it \
             must be text; fix it",
        ),
        (
            "packages-field-workspaces",
            "package.json",
            r#"{ "workspaces": 4 }"#,
            "yak-analyzer: the field `workspaces` in package.json holds a value of a form it does not have; it \
             must be a list of globs, or an object whose `packages` is one; fix it",
        ),
        (
            "packages-field-workspace-entry",
            "package.json",
            r#"{ "workspaces": ["packages/*", 4] }"#,
            "yak-analyzer: the field `workspaces` in package.json holds a value of a form it does not have; it \
             must be a list of globs, or an object whose `packages` is one; fix it",
        ),
        (
            "packages-field-pnpm",
            "pnpm-workspace.yaml",
            "packages: apps/*\n",
            "yak-analyzer: the field `packages` in pnpm-workspace.yaml holds a value of a form it does not \
             have; it must be a list of globs; fix it",
        ),
        (
            "packages-field-glob",
            "package.json",
            r#"{ "workspaces": ["packages/[a"] }"#,
            "yak-analyzer: a glob of the workspace's list cannot be read: the glob `packages/[a` has a bracket \
             left open; fix it in package.json or pnpm-workspace.yaml",
        ),
    ];

    for (name, path, content, expected) in cases {
        let project = Project::new(name);

        project.write(path, content).commit();
        assert_eq!(stopped(&project.run(|_| {})), expected, "{name}");
    }
}

#[test]
fn reports_an_unreadable_package_json_before_an_unreadable_workspace_list_and_either_before_a_field()
 {
    let project = Project::new("packages-order");

    project
        .write("package.json", r#"{ "name": 4 }"#)
        .write("pnpm-workspace.yaml", "packages: [\n")
        .write("z/package.json", "{")
        .commit();

    let line = stopped(&project.run(|_| {}));

    assert!(
        line.starts_with("yak-analyzer: the package.json z/package.json cannot be read"),
        "{line}"
    );
}
