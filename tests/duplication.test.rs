//! Runs the built binary's `duplication` on made-up projects and checks the document it prints: the
//! settings it reads, the groups of copies it finds, their sizes and fingerprints, and how it refuses a
//! command line (duplication spec COPY-1 to COPY-5, SAME-1 to SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7,
//! DUP-4, DUP-5, SHOW-1 to SHOW-4, FIXT-1).

#[expect(
    dead_code,
    reason = "the shared project helpers include running `graph` and reading a document's paths, which these tests do not need"
)]
#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, printed};

/// The sentence of the first notice of the kind given that a document holds.
fn sentence<'d>(document: &'d Value, kind: &str) -> &'d str {
    document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .find(|notice| notice["kind"] == json!(kind))
        .and_then(|notice| notice["sentence"].as_str())
        .unwrap_or_default()
}

/// Two functions whose bodies are the same: lines 2 to 6, and lines 9 to 15 with a blank line and a
/// comment among them.
const BODIES: &str = r#"export function first(items: number[]) {
  const total = items.length;
  if (total > 10) {
    return total * 2;
  }
  return total;
}
export function second(items: number[]) {
  const total = items.length;

  // more than ten doubles
  if (total > 10) {
    return total * 2;
  }
  return total;
}
"#;

/// The document `duplication` prints for the checkout with the options given. The run must succeed and
/// write no line.
fn duplication(project: &Project, options: &[&str]) -> Value {
    printed(&project.start("duplication", |binary| {
        binary.args(options);
    }))
}

/// A hand-off file for the checkout, with one stack whose pack and project declare what is given.
fn declaring(project: &Project, pack: &Value, own: &Value) -> String {
    project.hand_off(
        &project.root,
        &["."],
        json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": own }]),
    )
}

/// The document `duplication` prints with a hand-off file whose pack declares what is given.
fn looking(project: &Project, pack: &Value) -> Value {
    let hand_off = declaring(project, pack, &json!({}));

    duplication(project, &["--hand-off", &hand_off])
}

/// The groups a document lists, in its order: each one's copies, as their file and their first and
/// last lines.
fn groups(document: &Value) -> Vec<Vec<(&str, u64, u64)>> {
    document["groups"]
        .as_array()
        .expect("a list of groups")
        .iter()
        .map(|group| {
            group["copies"]
                .as_array()
                .expect("a list of copies")
                .iter()
                .map(|copy| {
                    (
                        copy["path"].as_str().expect("a path"),
                        copy["startLine"].as_u64().expect("a line"),
                        copy["endLine"].as_u64().expect("a line"),
                    )
                })
                .collect()
        })
        .collect()
}

#[test]
fn with_no_minimum_declared_it_looks_for_nothing_and_says_so() {
    let project = Project::new("duplication-no-minimum");

    project.write("src/a.ts", BODIES).commit();

    let document = duplication(&project, &[]);

    assert_eq!(document["interfaceVersion"], json!(1));
    assert_eq!(
        document["settings"],
        json!({
            "minimumTokens": null,
            "minimumLines": null,
            "minimumCopies": 2,
            "ignore": [],
            "testCallees": [],
            "accepted": [],
        })
    );
    assert_eq!(document["groups"], json!([]));
    assert_eq!(
        notices(&document),
        ["duplication-not-checked", "no-hand-off-file"]
    );
}

#[test]
fn reads_each_setting_as_its_key_says_which_value_wins() {
    let project = Project::new("duplication-settings");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let first = format!("{}/2", "a".repeat(32));
    let second = format!("{}/3", "b".repeat(32));
    let stack = |name: &str, pack: Value, own: Value| json!({ "name": name, "interfaceVersion": 1, "pack": pack, "project": own });
    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([
            stack(
                "first",
                json!({
                    "duplicationMinimum": 30,
                    "duplicationIgnore": ["strings"],
                    "duplicationTestCallees": ["it"],
                    "duplicationAccepted": [first],
                }),
                json!({ "duplicationMinimumLines": 4, "duplicationTestCallees": ["check"] })
            ),
            stack(
                "second",
                json!({
                    "duplicationMinimum": 40,
                    "duplicationCopies": 3,
                    "duplicationIgnore": ["names", "numbers"],
                    "duplicationTestCallees": ["test"],
                }),
                json!({
                    "duplicationIgnore": [],
                    "duplicationTestCallees": ["spec.only"],
                    "duplicationAccepted": [second],
                })
            ),
        ]),
    );
    let document = duplication(&project, &["--hand-off", &hand_off]);

    // A number is the project's, else the later stack's; the project's word list, empty too, replaces
    // the pack's; the project's test callees, joined across its stacks, replace the pack's; accepted
    // groups join (spec TUNE-4 to TUNE-7).
    assert_eq!(
        document["settings"],
        json!({
            "minimumTokens": 40,
            "minimumLines": 4,
            "minimumCopies": 3,
            "ignore": [],
            "testCallees": ["check", "spec.only"],
            "accepted": [first, second],
        })
    );
    assert!(!notices(&document).contains(&"duplication-not-checked"));

    // Where the project states no list, the later stack's words win, and the stacks' test callees
    // join.
    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([
            stack(
                "first",
                json!({
                    "duplicationMinimum": 30,
                    "duplicationIgnore": ["strings"],
                    "duplicationTestCallees": ["it"],
                }),
                json!({})
            ),
            stack(
                "second",
                json!({
                    "duplicationIgnore": ["numbers", "names"],
                    "duplicationTestCallees": ["test", "it"],
                }),
                json!({})
            ),
        ]),
    );

    assert_eq!(
        duplication(&project, &["--hand-off", &hand_off])["settings"],
        json!({
            "minimumTokens": 30,
            "minimumLines": null,
            "minimumCopies": 2,
            "ignore": ["names", "numbers"],
            "testCallees": ["it", "test"],
            "accepted": [],
        })
    );
}

#[test]
fn refuses_a_command_line_it_does_not_take() {
    let project = Project::new("duplication-refused");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    for options in [
        &["--all", "--all"][..],
        &["--every"],
        &["--hand-off"],
        &["--hand-off", "a.json", "--hand-off", "b.json"],
    ] {
        let output = project.start("duplication", |binary| {
            binary.args(options);
        });
        let line = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{options:?}");
        assert!(output.stdout.is_empty());
        assert!(
            line.contains("duplication [--hand-off <file>] [--all]"),
            "{line}"
        );
    }
}

#[test]
fn finds_the_bodies_two_functions_share_and_counts_their_size() {
    let project = Project::new("duplication-bodies");

    project.write("src/a.ts", BODIES).commit();

    let document = looking(&project, &json!({ "duplicationMinimum": 5 }));
    let group = &document["groups"][0];
    let fingerprint = group["fingerprint"].as_str().expect("a fingerprint");

    assert_eq!(
        groups(&document),
        [vec![("src/a.ts", 2, 6), ("src/a.ts", 9, 15)]]
    );
    // `const total = items.length` is 6 tokens, the `if` 12, `return total` 2; the blank line and the
    // comment hold none (spec SIZE-1).
    assert_eq!(group["tokens"], json!(20));
    assert_eq!(group["lines"], json!(5));
    assert_eq!(group["accepted"], json!(false));
    assert_eq!(fingerprint.len(), 32);
    assert!(fingerprint.chars().all(|digit| digit.is_ascii_hexdigit()));
    assert!(!notices(&document).contains(&"duplication-not-checked"));

    // With names loosened the two functions are one run, and their bodies' group lies inside it (spec
    // SAME-2, SAME-4).
    assert_eq!(
        groups(&looking(
            &project,
            &json!({ "duplicationMinimum": 5, "duplicationIgnore": ["names"] })
        )),
        [vec![("src/a.ts", 1, 7), ("src/a.ts", 8, 16)]]
    );
}

#[test]
fn reports_a_group_only_at_or_over_each_minimum() {
    let project = Project::new("duplication-minimums");

    project.write("src/a.ts", BODIES).commit();

    for (pack, found) in [
        (json!({ "duplicationMinimum": 20 }), 1),
        (json!({ "duplicationMinimum": 21 }), 0),
        (
            json!({ "duplicationMinimum": 5, "duplicationMinimumLines": 5 }),
            1,
        ),
        (
            json!({ "duplicationMinimum": 5, "duplicationMinimumLines": 6 }),
            0,
        ),
        (
            json!({ "duplicationMinimum": 5, "duplicationCopies": 2 }),
            1,
        ),
        (
            json!({ "duplicationMinimum": 5, "duplicationCopies": 3 }),
            0,
        ),
    ] {
        assert_eq!(groups(&looking(&project, &pack)).len(), found, "{pack}");
    }
}

#[test]
fn formatting_comments_quotes_and_parentheses_change_no_run() {
    let project = Project::new("duplication-formatting");

    project
        .write(
            "src/a.ts",
            r#"export function one() {
  const settings = { "mode": 'fast', limit: (3 + 4) };
  const make = (x) => new Map();
  return [settings, make];
}
"#,
        )
        .write(
            "src/b.ts",
            r#"export function two() {
  // the same, formatted another way
  const settings = {
    mode: "fast",
    limit: 3 + 4,
  }
  const make = x => new Map
  return [
    settings,
    make,
  ]
}
"#,
        )
        .write(
            "src/c.ts",
            r#"export function three() {
  const settings = { mode: "fast", limit: 3 - 4 };
  const make = (x) => new Map();
  return [settings, make];
}
"#,
        )
        .commit();

    let document = looking(&project, &json!({ "duplicationMinimum": 5 }));

    // The operator tells `c.ts`'s first statement apart, so its last two join the others' only (spec
    // SAME-1).
    assert_eq!(
        groups(&document),
        [
            vec![("src/a.ts", 2, 4), ("src/b.ts", 3, 11)],
            vec![("src/a.ts", 3, 4), ("src/b.ts", 7, 11), ("src/c.ts", 3, 4)],
        ]
    );
    // No semicolon, trailing comma, parenthesis around an expression or a lone parameter, or empty
    // `()` after `new` is counted (spec SIZE-1).
    assert_eq!(document["groups"][0]["tokens"], json!(27));
    assert_eq!(document["groups"][0]["lines"], json!(3));
    assert_eq!(document["groups"][1]["tokens"], json!(13));
}

#[test]
fn gives_the_same_groups_whatever_a_checkout_s_line_endings() {
    let text = "export function First() {\n  const note = `one\ntwo`;\n  return <p>\n    {note} and more\n  </p>;\n}\nexport function Second() {\n  const note = `one\ntwo`;\n  return <p>\n    {note} and more\n  </p>;\n}\n";
    let unix = Project::new("duplication-lf");
    let windows = Project::new("duplication-crlf");

    unix.write("src/a.tsx", text).commit();
    windows
        .write("src/a.tsx", &text.replace('\n', "\r\n"))
        .commit();

    let pack = json!({ "duplicationMinimum": 5 });
    let lf = looking(&unix, &pack);
    let crlf = looking(&windows, &pack);

    assert_eq!(
        groups(&lf),
        [vec![("src/a.tsx", 2, 6), ("src/a.tsx", 9, 13)]]
    );
    // The fingerprint, the size and the lines are the same (spec DUP-5).
    assert_eq!(lf["groups"], crlf["groups"]);
}

#[test]
fn gives_the_same_groups_for_an_attribute_written_over_lines_whatever_the_line_endings() {
    let text = "export function First() {\n  const size = 3;\n  return <div className=\"panel\n    wide\">{size}</div>;\n}\nexport function Second() {\n  const size = 3;\n  return <div className=\"panel\n    wide\">{size}</div>;\n}\n";
    let unix = Project::new("duplication-attribute-lf");
    let windows = Project::new("duplication-attribute-crlf");

    unix.write("src/a.tsx", text).commit();
    windows
        .write("src/a.tsx", &text.replace('\n', "\r\n"))
        .commit();

    let pack = json!({ "duplicationMinimum": 5 });
    let lf = looking(&unix, &pack);
    let crlf = looking(&windows, &pack);

    assert_eq!(
        groups(&lf),
        [vec![("src/a.tsx", 2, 4), ("src/a.tsx", 7, 9)]]
    );
    // The attribute's line ending is a line feed in both, so the fingerprint is the same (spec DUP-5).
    assert_eq!(lf["groups"], crlf["groups"]);
}

#[test]
fn loosens_names_strings_and_numbers_as_declared() {
    let project = Project::new("duplication-loosened");

    project
        .write(
            "src/e.ts",
            r#"export function alpha() {
  const label = "first";
  const size = 10;
  return { label, size };
}
export function beta() {
  const title = "second";
  const width = 20;
  return { title, width };
}
"#,
        )
        .commit();

    let found = |ignore: Value| {
        let document = looking(
            &project,
            &json!({ "duplicationMinimum": 5, "duplicationIgnore": ignore }),
        );

        groups(&document)
            .into_iter()
            .map(|group| {
                group
                    .into_iter()
                    .map(|(path, first, last)| (path.to_owned(), first, last))
                    .collect()
            })
            .collect::<Vec<Vec<(String, u64, u64)>>>()
    };
    let at = |first: u64, last: u64| ("src/e.ts".to_owned(), first, last);

    assert_eq!(found(json!([])), Vec::<Vec<(String, u64, u64)>>::new());
    // The numbers still differ, so only the returns join; the declarations are under the minimum.
    assert_eq!(
        found(json!(["names", "strings"])),
        [vec![at(4, 4), at(9, 9)]]
    );
    assert_eq!(
        found(json!(["names", "strings", "numbers"])),
        [vec![at(1, 5), at(6, 10)]]
    );
}

#[test]
fn takes_copies_that_repeat_back_to_back_without_overlap() {
    let project = Project::new("duplication-back-to-back");

    project
        .write(
            "src/grow.ts",
            r#"export function grow(first: number, second: number, third: number) {
  let total = 0;
  total += compute(first, second, third);
  total += compute(first, second, third);
  total += compute(first, second, third);
  return total;
}
"#,
        )
        .commit();

    // The run of two statements would overlap itself, so only the one-statement run is a group (spec
    // SAME-3).
    assert_eq!(
        groups(&looking(&project, &json!({ "duplicationMinimum": 5 }))),
        [vec![
            ("src/grow.ts", 3, 3),
            ("src/grow.ts", 4, 4),
            ("src/grow.ts", 5, 5)
        ]]
    );
}

#[test]
fn reports_a_group_with_a_copy_the_larger_group_lacks() {
    let project = Project::new("duplication-overlapping");

    project
        .write(
            "src/d.ts",
            r#"export function a() {
  step(one, two, three);
  finish(one, two, three);
}
export function b() {
  step(one, two, three);
  finish(one, two, three);
}
export function c() {
  step(one, two, three);
  other(one, two, three);
}
"#,
        )
        .commit();

    // `step` alone has a copy in `c`, so it is reported beside the larger run; `finish` alone is not
    // (spec SAME-4).
    assert_eq!(
        groups(&looking(&project, &json!({ "duplicationMinimum": 5 }))),
        [
            vec![("src/d.ts", 2, 2), ("src/d.ts", 6, 6), ("src/d.ts", 10, 10)],
            vec![("src/d.ts", 2, 3), ("src/d.ts", 6, 7)],
        ]
    );
}

#[test]
fn an_import_never_joins_a_run() {
    let project = Project::new("duplication-imports");

    project
        .write(
            "src/b.ts",
            "import { readFileSync } from \"node:fs\";\nimport { join } from \"node:path\";\nexport const b = join(\"a\", \"b\", readFileSync);\n",
        )
        .write(
            "src/c.ts",
            "import { readFileSync } from \"node:fs\";\nimport { join } from \"node:path\";\nexport const c = join(\"a\", \"c\", readFileSync);\n",
        )
        .commit();

    assert_eq!(
        looking(&project, &json!({ "duplicationMinimum": 5 }))["groups"],
        json!([])
    );
}

#[test]
fn counts_hooks_and_helpers_in_tests_but_no_test_s_body() {
    let project = Project::new("duplication-tests");

    project
        .write(
            "src/panel.test.ts",
            r#"describe("panel", () => {
  beforeEach(() => {
    const store = createStore({ size: 3 });
    store.reset(true);
  });
  it("opens", () => {
    const store = createStore({ size: 3 });
    store.reset(true);
  });
});
describe("drawer", () => {
  beforeEach(() => {
    const store = createStore({ size: 3 });
    store.reset(true);
  });
  it.each([1, 2])("closes %i", (count) => {
    const store = createStore({ size: 3 });
    store.reset(true);
  });
});
"#,
        )
        .commit();

    let found = |callees: Value| {
        let document = looking(
            &project,
            &json!({ "duplicationMinimum": 5, "duplicationTestCallees": callees }),
        );

        groups(&document)
            .into_iter()
            .map(|group| {
                group
                    .into_iter()
                    .map(|(_, first, last)| (first, last))
                    .collect()
            })
            .collect::<Vec<Vec<(u64, u64)>>>()
    };

    // Each form of a test callee is named: the hooks count, the tests' bodies do not (spec COPY-3).
    assert_eq!(found(json!(["it", "it.each()"])), [vec![(2, 5), (12, 15)]]);
    // `it` alone does not name `it.each(…)(…)`, so that test's body counts (spec TUNE-5).
    assert_eq!(
        found(json!(["it"])),
        [vec![(2, 5), (12, 15)], vec![(3, 4), (13, 14), (17, 18)]]
    );
    // With no test callee, every body counts.
    assert_eq!(
        found(json!([])),
        [
            vec![(2, 5), (12, 15)],
            vec![(3, 4), (7, 8), (13, 14), (17, 18)]
        ]
    );
}

#[test]
fn names_a_tagged_template_test_by_its_backticks() {
    let project = Project::new("duplication-tagged");

    project
        .write(
            "src/table.test.ts",
            "describe(\"table\", () => {\n  it.each`\n    a | b\n  `(\"adds $a\", ({ a, b }) => {\n    const store = createStore({ size: 3 });\n    store.reset(true);\n  });\n  helper(() => {\n    const store = createStore({ size: 3 });\n    store.reset(true);\n  });\n});\n",
        )
        .commit();

    assert_eq!(
        groups(&looking(&project, &json!({ "duplicationMinimum": 5 }))),
        [vec![
            ("src/table.test.ts", 5, 6),
            ("src/table.test.ts", 9, 10)
        ]]
    );
    assert_eq!(
        looking(
            &project,
            &json!({ "duplicationMinimum": 5, "duplicationTestCallees": ["it.each``"] })
        )["groups"],
        json!([])
    );
}

#[test]
fn an_accepted_group_is_left_out_until_its_run_or_its_copies_change() {
    let project = Project::new("duplication-accepted");

    project.write("src/a.ts", BODIES).commit();

    let found = looking(&project, &json!({ "duplicationMinimum": 5 }));
    let fingerprint = found["groups"][0]["fingerprint"]
        .as_str()
        .expect("a fingerprint")
        .to_owned();
    let accepted = format!("{fingerprint}/2");
    let stale = format!("{}/2", "0".repeat(32));
    let hand_off = declaring(
        &project,
        &json!({ "duplicationMinimum": 5, "duplicationAccepted": [stale] }),
        &json!({ "duplicationAccepted": [accepted] }),
    );
    let document = duplication(&project, &["--hand-off", &hand_off]);

    // The accepted group is left out, and the entry that matches no group is named, with who states it
    // (spec QUIET-1, QUIET-2).
    assert_eq!(document["groups"], json!([]));
    assert_eq!(
        notices(&document)
            .iter()
            .filter(|kind| **kind == "accepted-group-not-found")
            .count(),
        1
    );
    assert!(
        sentence(&document, "accepted-group-not-found").contains(&format!("`{stale}`"))
            && sentence(&document, "accepted-group-not-found").contains("the pack"),
        "{document}"
    );

    // With `--all` it is listed, marked accepted (spec SHOW-3).
    let all = duplication(&project, &["--hand-off", &hand_off, "--all"]);

    assert_eq!(
        groups(&all),
        [vec![("src/a.ts", 2, 6), ("src/a.ts", 9, 15)]]
    );
    assert_eq!(all["groups"][0]["accepted"], json!(true));
    assert_eq!(all["settings"]["accepted"], json!([stale, accepted]));

    // A third copy makes it a group of three, which the entry does not name, so it is reported again.
    project
        .write(
            "src/b.ts",
            "export function third(items: number[]) {\n  const total = items.length;\n  if (total > 10) {\n    return total * 2;\n  }\n  return total;\n}\n",
        )
        .commit();

    let document = duplication(&project, &["--hand-off", &hand_off]);

    assert_eq!(
        groups(&document),
        [vec![
            ("src/a.ts", 2, 6),
            ("src/a.ts", 9, 15),
            ("src/b.ts", 2, 6)
        ]]
    );
    assert_eq!(document["groups"][0]["fingerprint"], json!(fingerprint));
    assert_eq!(
        notices(&document)
            .iter()
            .filter(|kind| **kind == "accepted-group-not-found")
            .count(),
        2
    );
}

#[test]
fn an_accepted_group_still_hides_the_groups_inside_it() {
    let project = Project::new("duplication-accepted-inside");

    project.write("src/a.ts", BODIES).commit();

    let pack = json!({ "duplicationMinimum": 5, "duplicationIgnore": ["names"] });
    let fingerprint = looking(&project, &pack)["groups"][0]["fingerprint"]
        .as_str()
        .expect("a fingerprint")
        .to_owned();
    let hand_off = declaring(
        &project,
        &pack,
        &json!({ "duplicationAccepted": [format!("{fingerprint}/2")] }),
    );

    // The bodies' group lies inside the accepted functions' group, so it is not reported either (spec
    // QUIET-1).
    assert_eq!(
        duplication(&project, &["--hand-off", &hand_off])["groups"],
        json!([])
    );
    assert_eq!(
        groups(&duplication(&project, &["--hand-off", &hand_off, "--all"])),
        [vec![("src/a.ts", 1, 7), ("src/a.ts", 8, 16)]]
    );
}

#[test]
fn finds_copies_in_scripts_and_vue_blocks_only() {
    let project = Project::new("duplication-files");
    let body =
        "  let sum = 0;\n  for (const item of items) {\n    sum += item;\n  }\n  return sum;\n";

    project
        .write(
            "src/util.ts",
            &format!("export function total(items: number[]) {{\n{body}}}\n"),
        )
        .write(
            "src/Panel.vue",
            &format!(
                "<template>\n  <p>{{{{ count }}}}</p>\n</template>\n\n<script setup lang=\"ts\">\nfunction total(items: number[]) {{\n{body}}}\nconst count = total([1, 2]);\n</script>\n"
            ),
        )
        .write(
            "src/gen.ts",
            &format!("export function made(items: number[]) {{\n{body}}}\n"),
        )
        .write(
            "fixtures/copy.ts",
            &format!("export function kept(items: number[]) {{\n{body}}}\n"),
        )
        .write(
            "src/shape.ts",
            "export interface Shape { width: number; height: number; depth: number; label: string }\n",
        )
        .write(
            "src/types.d.ts",
            "export interface Shape { width: number; height: number; depth: number; label: string }\n",
        )
        .write(
            "docs/guide.mdx",
            &format!("export function told(items) {{\n{body}}}\n\n# Guide\n"),
        )
        .commit();

    // Generated, data and type declaration files and MDX documents hold no copy; a Vue block's lines
    // are counted in the file (spec COPY-4).
    assert_eq!(
        groups(&looking(
            &project,
            &json!({
                "duplicationMinimum": 5,
                "data": ["fixtures/**"],
                "generated": ["src/gen.ts"],
            })
        )),
        [vec![("src/Panel.vue", 7, 11), ("src/util.ts", 2, 6)]]
    );
}

#[test]
fn finds_a_run_of_members_two_classes_share() {
    let project = Project::new("duplication-class-body");

    project
        .write(
            "src/shapes.ts",
            r#"export class Square {
  width = 10;
  height = 20;
  area() { return this.width * this.height; }
  name() { return "square"; }
}
export class Plate {
  width = 10;
  height = 20;
  area() { return this.width * this.height; }
  name() { return "plate"; }
}
"#,
        )
        .commit();

    // A class's body is a list, so its members join a run (spec COPY-1).
    assert_eq!(
        groups(&looking(&project, &json!({ "duplicationMinimum": 5 }))),
        [vec![("src/shapes.ts", 2, 4), ("src/shapes.ts", 8, 10)]]
    );
}

#[test]
fn reads_types_as_code() {
    let project = Project::new("duplication-types");

    project
        .write(
            "src/count.ts",
            r#"export function first(items: number[], offset: number) {
  const total: number = items.length + offset;
}
export function second(items: number[], offset: number) {
  const total: Count = items.length + offset;
}
export function third(items: number[], offset: number) {
  const total: number = items.length + offset;
}
"#,
        )
        .write(
            "src/point.ts",
            "export interface Point { x: number; y: number; label: string }\n",
        )
        .write(
            "src/place.ts",
            "export interface Point { x: number; y: number; label: string }\n",
        )
        .commit();

    // `second`'s statement differs only in its annotation, so it is no copy of the others; the
    // interfaces are copies (spec COPY-1, SAME-1).
    assert_eq!(
        groups(&looking(&project, &json!({ "duplicationMinimum": 5 }))),
        [
            vec![("src/count.ts", 2, 2), ("src/count.ts", 8, 8)],
            vec![("src/place.ts", 1, 1), ("src/point.ts", 1, 1)],
        ]
    );
}

#[test]
fn compares_a_tagged_template_with_an_invalid_escape_by_its_raw_text() {
    let project = Project::new("duplication-raw-template");

    project
        .write(
            "src/tex.ts",
            r"export function one(value: string) {
  const shown = latex`\unit{${value}} and \xyz`;
  return shown;
}
export function two(value: string) {
  const shown = latex`\unit{${value}} and \xyz`;
  return shown;
}
export function three(value: string) {
  const shown = latex`\unit{${value}} and \xzw`;
  return shown;
}
",
        )
        .commit();

    // The pieces have no cooked value, so their raw text is compared: `three`'s differs (spec SAME-1).
    assert_eq!(
        groups(&looking(&project, &json!({ "duplicationMinimum": 5 }))),
        [vec![("src/tex.ts", 2, 3), ("src/tex.ts", 6, 7)]]
    );
}

#[test]
fn loosens_strings_alone_and_numbers_alone() {
    let project = Project::new("duplication-loosened-alone");

    project
        .write(
            "src/s.ts",
            r#"export function alpha(size: number) {
  const label = format("first", size);
  return describe(label, "first");
}
export function beta(size: number) {
  const label = format("second", size);
  return describe(label, "second");
}
"#,
        )
        .write(
            "src/n.ts",
            r#"export function gamma(label: string) {
  const size = measure(label, 10);
  return describe(label, size * 10);
}
export function delta(label: string) {
  const size = measure(label, 20);
  return describe(label, size * 20);
}
"#,
        )
        .commit();

    let found = |ignore: Value| {
        groups(&looking(
            &project,
            &json!({ "duplicationMinimum": 5, "duplicationIgnore": ignore }),
        ))
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .map(|(path, first, last)| (path.to_owned(), first, last))
                .collect()
        })
        .collect::<Vec<Vec<(String, u64, u64)>>>()
    };
    let at = |path: &str, first: u64, last: u64| (path.to_owned(), first, last);

    // Each word loosens only its own kind of value (spec SAME-2).
    assert_eq!(found(json!([])), Vec::<Vec<(String, u64, u64)>>::new());
    assert_eq!(
        found(json!(["strings"])),
        [vec![at("src/s.ts", 2, 3), at("src/s.ts", 6, 7)]]
    );
    assert_eq!(
        found(json!(["numbers"])),
        [vec![at("src/n.ts", 2, 3), at("src/n.ts", 6, 7)]]
    );
}

#[test]
fn jsx_reindented_and_rewrapped_is_the_same_copy() {
    let compact = "export function One({ items }) {\n  const visible = items.filter((item) => item.shown);\n  return <ul className=\"list\">{visible.map((item) => <li key={item.id}>Item {item.name}</li>)}</ul>;\n}\n";
    let rewrapped = r#"export function Two({ items }) {
    const visible = items.filter(
        (item) => item.shown,
    );
    return (
        <ul className="list">
            {visible.map((item) => (
                <li key={item.id}>
                    Item {item.name}
                </li>
            ))}
        </ul>
    );
}
"#;
    let alike = Project::new("duplication-jsx-alike");
    let wrapped = Project::new("duplication-jsx-rewrapped");

    alike
        .write("src/a.tsx", compact)
        .write("src/b.tsx", &compact.replace("One", "Two"))
        .commit();
    wrapped
        .write("src/a.tsx", compact)
        .write("src/b.tsx", rewrapped)
        .commit();

    let pack = json!({ "duplicationMinimum": 5 });
    let first = looking(&alike, &pack);
    let second = looking(&wrapped, &pack);

    assert_eq!(
        groups(&second),
        [vec![("src/a.tsx", 2, 3), ("src/b.tsx", 2, 13)]]
    );
    // Indentation and wrapping change neither the fingerprint nor the counts (spec SAME-1, SIZE-1).
    for key in ["fingerprint", "tokens", "lines"] {
        assert_eq!(first["groups"][0][key], second["groups"][0][key], "{key}");
    }
}

#[test]
fn a_hole_in_an_array_s_destructuring_is_part_of_its_form() {
    let project = Project::new("duplication-array-holes");

    project
        .write(
            "src/h.ts",
            r"export function alpha(t) {
  const [a, , b] = t();
  return a + b;
}
export function beta(t) {
  const [a, b] = t();
  return a + b;
}
export function gamma(t) {
  let a, b;
  [a, , b] = t();
  return a + b;
}
export function delta(t) {
  let a, b;
  [a, b] = t();
  return a + b;
}
export function epsilon(u) {
  const [c, , d] = u();
  return c + d;
}
",
        )
        .commit();

    let document = looking(
        &project,
        &json!({ "duplicationMinimum": 5, "duplicationIgnore": ["names"] }),
    );

    // A pattern with a hole binds other elements than one without, so only the functions whose holes
    // sit alike are copies (spec SAME-1).
    assert_eq!(
        groups(&document),
        [vec![("src/h.ts", 1, 4), ("src/h.ts", 19, 22)]]
    );
}
