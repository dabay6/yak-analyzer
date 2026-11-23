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
