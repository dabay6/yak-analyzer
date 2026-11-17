//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4, ENTRY-6a; comparison spec CONF-1).

mod support;

use serde_json::{Value, json};

use support::{document, folder, run, written};

/// The line the run stopped with when the stack `typescript` is handed the declarations given.
fn refused(place: &str, pack: Value, project: Value) -> String {
    let place = folder(place);
    let mut handed = document(&place);

    handed["stacks"][0]["pack"] = pack;
    handed["stacks"][0]["project"] = project;

    let output = run(&["graph", "--hand-off", &written(&place, &handed)]);
    let error = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(output.stdout.is_empty(), "{error}");
    assert_eq!(error.lines().count(), 1, "{error}");

    error.trim_end().to_owned()
}

#[test]
fn uses_every_key_it_knows_in_every_form_a_glob_may_take() {
    let place = folder("declarations-usable");
    let mut handed = document(&place);

    handed["stacks"][0]["pack"] = json!({
        "entries": ["src/main.ts", "**/*.{test,spec}.{ts,tsx}", "scripts/[!_]*.mjs", "docs/{a,{b,c}}/*.mdx"],
        "data": ["**/fixtures/**", "odd\\[name\\]/?.ts", "letters/[a-z].ts"],
        "generated": [],
        "internal": ["packages/core", "packages/pack-sdk"],
        "cyclomaticLimit": 20,
        "cognitiveLimit": 15,
        "duplicationMinimum": 50,
        "configEntries": [
            {
                "file": "runner.config.*",
                "property": ["test", "include"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["**/*.test.ts"],
                "options": { "program": "runner", "names": ["--config", "-c"], "relativeTo": "script" }
            },
            {
                "file": "**/runner.config.{js,ts}",
                "property": ["roots"],
                "value": "folder",
                "pattern": "**/*.spec.{js,ts}",
                "relativeTo": "detection",
                "default": []
            }
        ],
        "scriptCommands": ["cross-env", "tsx"]
    });
    handed["stacks"][0]["project"] = json!({
        "entries": ["tools/run.ts"],
        "data": ["samples/**"],
        "generated": ["src/generated/**"],
        "internal": ["packages/app"],
        "cyclomaticLimit": 25
    });

    let output = run(&["graph", "--hand-off", &written(&place, &handed)]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn names_the_stack_the_key_and_who_states_it() {
    assert_eq!(
        refused(
            "declarations-pack",
            json!({ "entrypoints": ["src/main.ts"] }),
            json!({})
        ),
        "yak-analyzer: cannot use the declaration `entrypoints` that the pack states for the stack `typescript`: \
         the analyzer knows no such key, only entries, data, generated, internal, cyclomaticLimit, \
         cognitiveLimit, duplicationMinimum, configEntries, scriptCommands; change it in the pack"
    );
    assert_eq!(
        refused(
            "declarations-project",
            json!({}),
            json!({ "scriptCommands": ["bundle"] })
        ),
        "yak-analyzer: cannot use the declaration `scriptCommands` that the project states for the stack \
         `typescript`: only a pack may state this key; change it in the project's own declarations"
    );
}

#[test]
fn refuses_a_value_that_is_not_of_its_keys_kind() {
    let cases: [(Value, &str); 7] = [
        (
            json!({ "data": "fixtures/**" }),
            "`data` that the pack states",
        ),
        (
            json!({ "data": ["fixtures/**", 3] }),
            ": its value must be a list of globs; ",
        ),
        (
            json!({ "internal": { "packages": [] } }),
            ": its value must be a list of paths; ",
        ),
        (
            json!({ "cyclomaticLimit": 0 }),
            ": its value must be a positive whole number; ",
        ),
        (
            json!({ "cognitiveLimit": 12.5 }),
            ": its value must be a positive whole number; ",
        ),
        (
            json!({ "duplicationMinimum": "50" }),
            ": its value must be a positive whole number; ",
        ),
        (
            json!({ "duplicationMinimum": ["50"] }),
            ": its value must be a positive whole number; ",
        ),
    ];

    for (index, (pack, expected)) in cases.into_iter().enumerate() {
        let line = refused(&format!("declarations-kind-{index}"), pack, json!({}));

        assert!(line.contains(expected), "{line}");
    }

    let line = refused(
        "declarations-kind-project",
        json!({}),
        json!({ "entries": 4 }),
    );

    assert!(
        line.contains("`entries` that the project states")
            && line.contains(": its value must be a list of globs; "),
        "{line}"
    );
}

#[test]
fn refuses_a_configuration_or_script_declaration_of_another_shape() {
    let usable = || {
        json!({
            "file": "runner.config.js",
            "property": ["include"],
            "value": "globs",
            "relativeTo": "config",
            "default": ["**/*.test.js"]
        })
    };
    let changed = |change: &dyn Fn(&mut Value)| {
        let mut entry = usable();

        change(&mut entry);

        json!({ "configEntries": [entry] })
    };
    let cases: [(Value, &str); 20] = [
        (
            json!({ "configEntries": {} }),
            ": its value must be a list of objects; ",
        ),
        (
            json!({ "configEntries": [3] }),
            ": its entry 1 must be an object; ",
        ),
        (
            changed(&|entry| entry["extra"] = json!(1)),
            ": its entry 1's field `extra` is not one it may hold; ",
        ),
        (
            changed(&|entry| {
                entry.as_object_mut().expect("an object").remove("default");
            }),
            ": its entry 1's field `default` is missing; ",
        ),
        (
            changed(&|entry| entry["value"] = json!("folder")),
            ": its entry 1's field `pattern` is missing, and a `folder` value needs it; ",
        ),
        (
            changed(&|entry| entry["pattern"] = json!("**/*.js")),
            ": its entry 1's field `pattern` is only for a `folder` value; ",
        ),
        (
            changed(&|entry| entry["property"] = json!([])),
            ": its entry 1's field `property` must be a list of keys, one or more; ",
        ),
        (
            changed(&|entry| entry["value"] = json!("list")),
            ": its entry 1's field `value` must be `globs` or `folder`; ",
        ),
        (
            changed(&|entry| entry["relativeTo"] = json!("package")),
            ": its entry 1's field `relativeTo` must be `config` or `detection`; ",
        ),
        (
            changed(&|entry| entry["file"] = json!("src/[a")),
            ": its entry 1's field `file` cannot be used: the glob `src/[a` has a bracket left open; ",
        ),
        (
            changed(&|entry| entry["file"] = json!(4)),
            ": its entry 1's field `file` must be a glob; ",
        ),
        (
            changed(&|entry| entry["default"] = json!(["../x/*.js"])),
            ": its entry 1's field `default` cannot be used: the glob `../x/*.js` is absolute or steps out",
        ),
        (
            changed(&|entry| entry["options"] = json!(3)),
            ": its entry 1's field `options` must be an object; ",
        ),
        (
            changed(&|entry| {
                entry["options"] = json!({ "program": "runner", "names": ["-c"], "extra": 1 });
            }),
            ": its entry 1's field `options` holds `extra`, which it may not; ",
        ),
        (
            changed(&|entry| entry["options"] = json!({ "names": ["-c"] })),
            ": its entry 1's field `options` must name its `program`, one word with no white space; ",
        ),
        (
            changed(&|entry| entry["options"] = json!({ "program": "runner", "names": [] })),
            ": its entry 1's field `options` must list its option `names`, one or more; ",
        ),
        (
            changed(&|entry| {
                entry["options"] = json!({ "program": "runner", "names": ["--config x"] });
            }),
            ": its entry 1's field `options` names the option `--config x`, which is empty or holds white \
             space or `=`; ",
        ),
        (
            changed(&|entry| {
                entry["options"] =
                    json!({ "program": "runner", "names": ["-c"], "relativeTo": "package" });
            }),
            ": its entry 1's field `options` has a `relativeTo` that is not `config`, `detection` or \
             `script`; ",
        ),
        (
            json!({ "scriptCommands": "node" }),
            ": its value must be a list of program names; ",
        ),
        (
            json!({ "scriptCommands": ["cross env"] }),
            ": the program name `cross env` is empty or holds white space; ",
        ),
    ];

    for (index, (pack, expected)) in cases.into_iter().enumerate() {
        let line = refused(&format!("declarations-shape-{index}"), pack, json!({}));

        assert!(line.contains(expected), "{line}");
    }
}

#[test]
fn refuses_a_glob_or_a_path_it_cannot_read_or_that_leaves_its_folder() {
    let cases: [(Value, &str); 10] = [
        (json!({ "data": [""] }), ": a glob in it is empty; "),
        (
            json!({ "data": ["/abs/**"] }),
            ": the glob `/abs/**` is absolute or steps out of its folder with `..`; ",
        ),
        (
            json!({ "data": ["C:/abs/**"] }),
            ": the glob `C:/abs/**` is absolute or steps out of",
        ),
        (
            json!({ "entries": ["../other/*.ts"] }),
            ": the glob `../other/*.ts` is absolute or steps out of",
        ),
        (
            json!({ "entries": ["src/../../x.ts"] }),
            ": the glob `src/../../x.ts` is absolute or steps out of",
        ),
        (
            json!({ "generated": ["src/[a-z.ts"] }),
            ": the glob `src/[a-z.ts` has a bracket left open; ",
        ),
        (
            json!({ "generated": ["src/{a,b.ts"] }),
            ": the glob `src/{a,b.ts` has a brace left open; ",
        ),
        (
            json!({ "generated": ["src/a\\"] }),
            ": the glob `src/a\\` ends in a backslash that escapes nothing; ",
        ),
        (json!({ "internal": [""] }), ": a path in it is empty; "),
        (
            json!({ "internal": ["../core"] }),
            ": the path `../core` is absolute or steps out of its folder with `..`; ",
        ),
    ];

    for (index, (pack, expected)) in cases.into_iter().enumerate() {
        let line = refused(&format!("declarations-glob-{index}"), pack, json!({}));

        assert!(line.contains(expected), "{line}");
    }
}

#[test]
fn reports_the_packs_before_the_projects_and_an_earlier_stack_before_a_later_one() {
    let line = refused(
        "declarations-order",
        json!({ "internal": ["/abs"] }),
        json!({ "unknownKey": ["x"] }),
    );

    assert!(line.contains("`internal` that the pack states"), "{line}");

    let place = folder("declarations-order-stacks");
    let mut handed = document(&place);

    handed["stacks"][0]["project"] = json!({ "unknownKey": ["x"] });
    handed["stacks"][1] = json!({
        "name": "second",
        "interfaceVersion": 1,
        "pack": { "alsoUnknown": 1 },
        "project": {}
    });

    let output = run(&["graph", "--hand-off", &written(&place, &handed)]);
    let line = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        line.contains("`unknownKey` that the project states for the stack `typescript`"),
        "{line}"
    );
}
