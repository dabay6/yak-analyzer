//! Hands the built binary a hand-off file and checks what it reads and what it refuses (spec HO-2 to
//! HO-5, RUN-3, RUN-4).

mod support;

use std::fs;

use serde_json::{Value, json};

use support::{document, folder, run, written};

/// The one line a stopped run wrote, after checking that it stopped as a stop does.
fn stopped(args: &[&str]) -> String {
    let output = run(args);
    let error = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(output.stdout.is_empty(), "{error}");
    assert_eq!(error.lines().count(), 1, "{error}");
    assert!(error.starts_with("yak-analyzer: "), "{error}");

    error.trim_end().to_owned()
}

#[test]
fn reads_a_hand_off_file_it_can_use() {
    let place = folder("hand-off-usable");
    let file = written(&place, &document(&place));
    let output = run(&["graph", "--hand-off", &file]);
    let printed: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    // The checkout is empty, so the two globs the hand-off file declares match nothing (spec KEY-5).
    let nothing = |key: &str, glob: &str, source: &str| {
        json!({
            "file": null,
            "kind": "glob-matches-nothing",
            "sentence": format!(
                "The `{key}` glob `{glob}` that the {source} states for the stack `typescript` matches no file."
            ),
        })
    };

    assert_eq!(
        printed,
        json!({
            "files": [],
            "handOff": true,
            "notices": [
                nothing("data", "**/fixtures/**", "pack"),
                nothing("generated", "src/generated/**", "project"),
            ],
            "packages": [],
        })
    );
}

#[test]
fn passes_over_a_byte_order_mark_and_a_top_level_field_it_does_not_know() {
    let place = folder("hand-off-mark");
    let mut handed = document(&place);

    handed["cache"] = json!({ "folder": "somewhere" });

    let file = written(&place, &handed);
    let mut bytes = b"\xEF\xBB\xBF".to_vec();

    bytes.extend(fs::read(&file).expect("the file was written"));
    fs::write(&file, bytes).expect("the file can be written again");

    assert!(run(&["graph", "--hand-off", &file]).status.success());
}

#[test]
fn says_in_a_notice_that_the_detection_list_was_cut_short() {
    let place = folder("hand-off-cut-short");
    let mut handed = document(&place);

    handed["detection"]["truncated"] = json!(true);

    let output = run(&["graph", "--hand-off", &written(&place, &handed)]);
    let printed: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(
        printed["notices"]
            .as_array()
            .expect("a list of notices")
            .iter()
            .any(|notice| notice["kind"] == json!("detection-cut-short"))
    );
}

#[test]
fn refuses_the_option_with_no_file_after_it() {
    let output = run(&["graph", "--hand-off"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stderr).lines().count(), 1);
}

#[test]
fn stops_on_a_file_that_is_not_there_or_is_not_json() {
    let place = folder("hand-off-unreadable");
    let missing = place.join("none.json");
    let missing = missing.to_str().expect("the test's folder is valid UTF-8");
    let broken = place.join("broken.json");

    fs::write(&broken, "{ \"formatVersion\": 1,\n").expect("the file can be written");

    let broken = broken.to_str().expect("the test's folder is valid UTF-8");
    let unread = stopped(&["graph", "--hand-off", missing]);

    assert!(
        unread.starts_with(&format!(
            "yak-analyzer: the hand-off file {missing} cannot be read ("
        )),
        "{unread}"
    );
    assert!(
        unread.ends_with("); pass the path of the file yak wrote for this checkout"),
        "{unread}"
    );
    assert_eq!(
        stopped(&["graph", "--hand-off", broken]),
        format!(
            "yak-analyzer: the hand-off file {broken} is not JSON (line 2, column 0); yak writes this file, so \
             write it again with yak"
        )
    );
}

#[test]
fn stops_on_a_format_it_does_not_read_and_says_which_side_to_update() {
    let place = folder("hand-off-format");
    let mut newer = document(&place);
    let mut older = document(&place);

    // The format is asked before any other field, so the missing root is not what is reported.
    newer["formatVersion"] = json!(2);
    newer.as_object_mut().expect("an object").remove("root");
    older["formatVersion"] = json!(0);

    let file = written(&place, &newer);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the hand-off file {file} is format 2, newer than any this analyzer reads (1); update \
             the analyzer's version pinned in the pack"
        )
    );

    let file = written(&place, &older);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the hand-off file {file} is format 0, older than any this analyzer reads (1); update \
             yak"
        )
    );
}

#[test]
fn stops_on_a_field_that_is_missing_misshapen_or_not_allowed_and_names_it() {
    let place = folder("hand-off-shape");
    let relative = json!("checkout");
    let cases: [(&str, Value, &str); 22] = [
        (
            "/formatVersion",
            Value::Null,
            "`formatVersion` must be a whole number",
        ),
        (
            "/formatVersion",
            json!(1.5),
            "`formatVersion` must be a whole number",
        ),
        (
            "/root",
            Value::Null,
            "`root` must be text that is not empty",
        ),
        ("/root", relative.clone(), "`root` must be an absolute path"),
        ("/detection", json!([]), "`detection` must be an object"),
        (
            "/detection/extra",
            json!(1),
            "`detection.extra` is not a field of the format",
        ),
        (
            "/detection/folders",
            json!([]),
            "`detection.folders` must be a list of at least one path",
        ),
        (
            "/detection/folders",
            json!(["src"]),
            "`detection.folders` must begin with `.`, the root itself",
        ),
        (
            "/detection/folders/1",
            json!("../other"),
            "`detection.folders[1]` must be a path inside the root, written with forward slashes",
        ),
        (
            "/detection/folders/1",
            json!("packages\\app"),
            "`detection.folders[1]` must be a path inside the root, written with forward slashes",
        ),
        (
            "/detection/truncated",
            json!("no"),
            "`detection.truncated` must be true or false",
        ),
        ("/stacks", json!({}), "`stacks` must be a list"),
        (
            "/stacks/0",
            json!("typescript"),
            "`stacks[0]` must be an object",
        ),
        (
            "/stacks/0/name",
            json!(""),
            "`stacks[0].name` must be text that is not empty",
        ),
        (
            "/stacks/0/interfaceVersion",
            json!(0),
            "`stacks[0].interfaceVersion` must be a positive whole number",
        ),
        (
            "/stacks/0/pack",
            json!([]),
            "`stacks[0].pack` must be an object",
        ),
        (
            "/stacks/1/project",
            json!(null),
            "`stacks[1].project` must be an object",
        ),
        (
            "/stacks/0/project/data",
            json!({ "globs": [] }),
            "`stacks[0].project.data` must be a list of text or a positive whole number",
        ),
        (
            "/stacks/1/tool",
            json!("analyzer"),
            "`stacks[1].tool` is not a field of the format",
        ),
        ("/siblings", relative, "`siblings` must be an absolute path"),
        (
            "/base",
            json!("abc123"),
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits",
        ),
        (
            "/base",
            json!("ABCDEF0123456789ABCDEF0123456789ABCDEF01"),
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits",
        ),
    ];

    for (pointer, value, problem) in cases {
        let mut handed = document(&place);

        set(&mut handed, pointer, value);

        let file = written(&place, &handed);

        assert_eq!(
            stopped(&["graph", "--hand-off", &file]),
            format!(
                "yak-analyzer: the hand-off file {file} is not format 1: {problem}; yak writes this file, so write \
                 it again with yak"
            ),
            "{pointer}"
        );
    }

    for field in ["formatVersion", "root", "detection", "stacks", "siblings"] {
        let mut handed = document(&place);

        handed.as_object_mut().expect("an object").remove(field);

        let line = stopped(&["graph", "--hand-off", &written(&place, &handed)]);

        assert!(
            line.contains(&format!("is not format 1: `{field}` is missing; ")),
            "{line}"
        );
    }
}

#[test]
fn takes_a_base_and_a_stack_with_no_declarations_and_no_interface_version() {
    let place = folder("hand-off-base");
    let mut handed = document(&place);

    handed["base"] = json!("0123456789abcdef0123456789abcdef01234567");

    assert!(
        run(&["graph", "--hand-off", &written(&place, &handed)])
            .status
            .success()
    );
}

#[test]
fn stops_on_declarations_written_for_another_interface_version_or_for_none() {
    let place = folder("hand-off-interface");
    let mut newer = document(&place);
    let mut unstated = document(&place);

    // The interface version is asked before the declarations, so the unknown key is not what is reported.
    newer["stacks"][0]["interfaceVersion"] = json!(2);
    newer["stacks"][0]["pack"]["unknownKey"] = json!([]);
    unstated["stacks"][1]["project"] = json!({ "data": ["fixtures/**"] });

    let file = written(&place, &newer);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the stack `typescript` in {file} states declarations for interface version 2, and this \
             analyzer speaks version 1; update the analyzer's version pinned in the pack"
        )
    );

    let file = written(&place, &unstated);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the stack `bare` in {file} holds declarations and states no interface version; state \
             the version they were written for"
        )
    );
}

#[test]
fn reports_the_shape_before_the_interface_version_when_both_are_wrong() {
    let place = folder("hand-off-order");
    let mut handed = document(&place);

    handed["stacks"][0]["interfaceVersion"] = json!(2);
    handed["siblings"] = json!("relative");

    let line = stopped(&["graph", "--hand-off", &written(&place, &handed)]);

    assert!(
        line.contains("is not format 1: `siblings` must be an absolute path; "),
        "{line}"
    );
}

/// Puts a value at a JSON pointer, making the last step when it is not there yet.
fn set(document: &mut Value, pointer: &str, value: Value) {
    let (parent, last) = pointer.rsplit_once('/').expect("a pointer with a step");
    let parent = if parent.is_empty() {
        document
    } else {
        document
            .pointer_mut(parent)
            .expect("the pointer's parent is there")
    };

    match parent {
        Value::Object(fields) => {
            fields.insert(last.to_owned(), value);
        }
        Value::Array(entries) => entries[last.parse::<usize>().expect("an index")] = value,
        _ => panic!("the pointer's parent holds nothing"),
    }
}
