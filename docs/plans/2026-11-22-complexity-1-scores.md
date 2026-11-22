# Complexity, part 1: functions, their scores and the complexity command: implementation plan

**Goal:** the analyzer finds every function of a project's scripts and Vue components, names it by
what holds it, scores its cyclomatic and cognitive complexity as fallow 3.39.0 does, and a new
command, `yak-analyzer complexity`, prints the functions over a declared limit, or every function
with `--all`, each with the fingerprint its finding will carry.

**Architecture:** a new `src/functions.rs` walks the tree the import reading already parsed (oxc's,
with its binding analysis), so no file is parsed twice. It records each function, class and object
literal as a unit, with the unit around it and each function's scores. The units go on the file's
`Read`; a Vue component's blocks are joined into one list. `functions::named` turns a file's units
into functions with their name paths. A new `src/complexity.rs` reads the limits from the hand-off
file, measures a reading's functions against them with their fingerprints, and prints the
command's document. `audit` will use the same `measured` in plan 2 (SOLO-5).

**Spec:** docs/specs/2026-11-21-complexity-findings-design.md

**Implements:** FN-1 to FN-5, SCORE-1, SCORE-2, SCORE-3, LIMIT-1, LIMIT-2, CPX-4, CPX-5, SOLO-1 to SOLO-5, VER-1, VER-2, CASE-1, CASE-2

The first of the item's three plans, on the item's branch, `feat/complexity-findings`, which holds
the approved spec. The plans, in order:

1. **This plan:** functions, their names and scores, the limits, and the `complexity` command.
2. **The finding in `audit`:** the `complexity` result with its message and property bag (CPX-1 to
   CPX-3), its notice in `audit`, new against inherited (NEW-1 to NEW-5), FN-6's accepted shifts
   with their fixtures, and the rest of VER-1 and VER-2 (the property bag, and `audit` reporting six
   kinds). It is written once this plan has merged, from `main`.
3. **The comparison with fallow:** VS-1 to VS-7, on the pinned commits. Before it starts, the rule
   list this plan's decision 1 writes down goes back to the spec as SCORE-4's amendment, and is
   approved.

What waits for plan 2: `audit` reports no complexity finding yet. This plan adds `complexity` to the
run's list of rules, since the fingerprint is made under that rule's name (CPX-4), so the report
lists six rules and still holds only dead-code results. The LIMIT-2 notice is printed by the
command here, and by `audit` from plan 2.

## Global constraints

- A function is over a limit when its score is greater than the limit; a score equal to it is not
  over (the spec's terms).
- Each limit has one value for the whole run: the project's over the pack's, and of values from the
  same source the later stack's (LIMIT-1). No limit is built in: with none declared, nothing is over
  and a notice says so (LIMIT-2, the spec's anti-goals).
- The fingerprint keeps the scheme `yakAnalyzer/v1`: the first 32 hexadecimal digits of a SHA-256
  digest of length-prefixed parts, with no line number (CPX-4, dead-code spec REPORT-3).
- Nothing that `graph` prints changes, and no dead-code finding or fingerprint changes (FN-3, the
  spec's anti-goals).
- The interface version stays 1 (VER-1).
- No fixture copies code from fallow or from the named repositories, and fallow's code is read, not
  copied (CASE-2, SCORE-2).

## The decisions this plan makes

1. **The counting rules, read from fallow 3.39.0** (SCORE-2, SCORE-3). The plan's first step was done
   while this plan was written: fallow's licence is MIT, and its source at `v3.39.0` (commit
   `a6956af`) was read, not copied. Every count is in one visitor, `crates/extract/src/complexity.rs`,
   `ComplexityVisitor`. The rules the analyzer counts, each with its source there:
   - **Both measures.** A function with a body is scored on its own, from nesting 0, and adds
     nothing to the function around it (`visit_function`, `visit_arrow_function_expression`). A
     declaration with no body is not scored. Code outside every function counts nowhere, and code
     in a class's static block or field values counts toward the function around the class, if any
     (`visit_program`).
   - **Cyclomatic** is 1 (`push_function`), plus 1 for each `if` and `else if` (`visit_if_chain`),
     each loop (`visit_for_statement` and the others), each `case` with a test, empty fall-through
     cases included (`visit_switch_case`), each `catch` (`visit_catch_clause`), each `?:`
     (`visit_conditional_expression`), each `&&`, `||` and `??` (`visit_logical_expression`), each
     `&&=`, `||=` and `??=` (`visit_assignment_expression`), and each optional chain whose outermost
     link is optional (`visit_chain_expression`: `a?.b` and `a?.b?.c` add 1, `a?.b.c` and `a?.b()`
     add nothing, `a?.()` adds 1). `default`, `else`, `try`, `finally`, labelled jumps, default
     parameters and recursion add nothing. This replaces SCORE-1's "optional chaining adds
     nothing".
   - **Cognitive.** An `if`, a `?:`, a `switch` (once), a loop and a `catch` add 1 and the nesting,
     and each opens a level for its body; a test, a loop's head and a `switch`'s discriminant stay
     at the outer level (`visit_if_chain`, `visit_conditional_expression`, `visit_switch_statement`,
     `visit_catch_clause`). An `else if` and an `else` add 1 with no nesting, and their bodies are
     one level below the first `if`. `try` and `finally` add nothing and open no level. A labelled
     `break` or `continue` adds 1 (`visit_break_statement`, `visit_continue_statement`). Recursion
     adds nothing: fallow never detects it, which replaces SCORE-1's "1 for a direct recursive call".
   - **Runs of boolean operators** (`visit_logical_expression`, `handle_logical_operator`). Each
     function keeps the last operator it counted. An `&&`, `||` or `??` adds 1 when its operator is
     not that one, and becomes it; after its sides are walked, the last operator is forgotten if
     neither side is itself an `&&`, `||` or `??`, parentheses not seen through. So `a || b && c ||
     d` adds 2 where SonarSource adds 3, and `a && f(b && c)` adds 1 where SonarSource adds 2: fallow
     departs from SonarSource here, and its code wins (SCORE-3).
   - **React** (`record_hook`, `fold_component_prop_count`). A call of a bare `useX(…)`, `use` then a
     capital letter, adds 1 to a function whose name begins with a capital letter or is `useX…`. A
     function that renders JSX and whose first parameter takes more than 4 properties apart adds 1
     for each one past 4.
   - **Limits.** A finding when a score is greater than its limit (`engine/src/health/findings.rs`,
     `complexity_exceeded`).

   fallow's name for a function differs from FN-4's: for the React rule alone, a function's name is
   read as fallow reads it, from its own identifier, or what declares it, or what declares the call
   it is passed to (so `const Panel = memo(() => …)` is a component). Where fallow reads a name the
   analyzer can't, as a callback deeper inside an initializer, the comparison finds it and SCORE-3
   applies. The full reading, with line numbers, is `fallow-counting.md` in the item's report
   folder; it is not committed (it describes another project's code at length). SCORE-4's amendment
   carries this list back to the spec before the comparison.
2. **Where a function is placed** (CPX-2). At its own identifier, or the key of the method or
   property it is, with `get `/`set `/`static` passed over; otherwise at its first character: an
   arrow's first token, or `function`/`async`. fallow places a method at its `(` and a declaration
   at `function`; both are on the same line in every case the comparison is likely to meet, and
   VS-2 matches by line first.
3. **A unit list, named afterwards.** Whether a class or an object literal takes part in a name path
   is known only once the walk has left it (FN-5), so the walk records each unit with the unit around
   it, marks every unit around a function as holding one, and `named` numbers the parts in one pass
   over the list afterwards. A unit's name is read from its parent in oxc's node tree, past
   parentheses and type assertions, so `const wrapped = (() => {})` is `wrapped`.
4. **What a name is written as.** A key is its static name (`get value`, `#secret`, a string key's
   value, a numeric key's number); a computed key has none. A callee or left side is its chain of
   names as FN-4 writes it, and an expression with no such chain (`(a || b).map(…)`) gives no name,
   so the function is unnamed. A string argument is written as its value, in double quotes, with no
   escaping.
5. **A Vue component's places.** Its block's lines are added to the line, and on the block's first
   line its first columns to the column, so a function in `<script>export const f = () => 1;
   </script>` is placed at its column in the file.
6. **The command's document.** Its keys are sorted, as `graph`'s are; `limits` holds `cyclomatic`
   and `cognitive`; the notices are `graph`'s, and the LIMIT-2 notice is of the kind
   `complexity-not-checked`, about no file.
7. **Tests.** The made-up projects go in a new `tests/complexity.test.rs`, with its `[[test]]` entry
   in `Cargo.toml`, as every test file here is named and entered. That departs from the generic rule
   that a test file's name holds no dot, for consistency with the fifteen files already here
   (CONTRIBUTING.md names the pattern); a rename of them all is not this item's. The new modules sit
   flat in `src/` with the others, for the same reason.

## Review focus

- **A name read from the wrong child.** `name` checks that the unit is the holder's value, argument
  or right side by comparing spans. A function in a computed key, a decorator or a default value
  must not take the property's or the variable's name. Pinned by `[Symbol.iterator]: () => 0`
  (unnamed) in `names_each_function_by_what_holds_it_and_numbers_the_repeats`, Task 1.
- **The run of boolean operators across a nested function.** Each frame has its own last operator,
  so `a && xs.some(x => x && y)` counts 1 in each function. Pinned by `outer > inner` and `passed`
  in `scores_each_function_as_fallow_counts_it`, Task 2.
- **Nesting restored after a nested function or a thrown-away walk.** `nested` raises and lowers the
  innermost frame's level around the walk, and a function inside the walk pushes and pops its own
  frame. Pinned by `outer`, `builds` and `labelled` in Task 2's test.
- **A Vue block whose parse fails after an earlier block was appended.** The component returns its
  error, so the appended units are thrown away with the rest; no test needed beyond the existing
  `pages` tests, which still pass.
- **A function in a file the base can't read.** Not reached here: the command reads no base.

### Task 1: Functions, their names, the limits and the command

**Implements:** FN-1 to FN-5, LIMIT-1, LIMIT-2, CPX-4, CPX-5, SOLO-1 to SOLO-5, CASE-1, CASE-2

**Test:** `cargo test --locked --test complexity --test audit --test cli`

**Files:**
- Create: `src/functions.rs`, `src/complexity.rs`, `tests/complexity.test.rs`
- Modify: `src/imports.rs`, `src/pages.rs`, `src/findings.rs`, `src/audit.rs`, `src/cli.rs`,
  `src/main.rs`, `Cargo.toml`, `tests/audit.test.rs`, `tests/cli.test.rs`

**Interfaces:**
- Consumes: `imports::Lines` (made public), oxc's `Semantic` built in `imports::parse`,
  `reading::start`, `reading::read`, `files::list`, `hand_off::Stack`, `notice::printed`,
  `findings::fingerprint` (made public).
- Produces: `functions::{Unit, Scored, Part, Function, units, named, append}`; `Read::units`;
  `complexity::{Limits, Measured, measured, fingerprint, run}`; `Rule::Complexity`;
  `cli::Command::Complexity { hand_off, all }`. Every function scores cyclomatic 1 and cognitive 0
  here; Task 2 counts.

- [ ] **Step 1: Write the failing tests**

```rust path=tests/complexity.test.rs
//! Runs the built binary's `complexity` on made-up projects and checks the document it prints: the
//! limits it reads, the functions it finds and how it names them, and how it stops (complexity spec
//! FN-1 to FN-5, LIMIT-1, LIMIT-2, CPX-4, CPX-5, SOLO-1 to SOLO-4, CASE-1).

#[expect(
    dead_code,
    reason = "the shared project helpers include running `graph` and reading a document's paths, which these tests do not need"
)]
#[path = "support/project.rs"]
mod project;

use std::collections::BTreeMap;

use serde_json::{Value, json};

use project::{Project, notices, printed, stopped};

/// The document `complexity` prints for the checkout with the options given. The run must succeed
/// and write no line.
fn complexity(project: &Project, options: &[&str]) -> Value {
    printed(&project.start("complexity", |binary| {
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

/// The functions a document lists, in its order: each one's file and readable name path.
fn listed(document: &Value) -> Vec<(&str, &str)> {
    document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .map(|function| {
            (
                function["path"].as_str().expect("a path"),
                function["function"].as_str().expect("a name path"),
            )
        })
        .collect()
}

/// The readable name paths of the functions a document lists in the file given, in its order.
fn names<'d>(document: &'d Value, path: &str) -> Vec<&'d str> {
    listed(document)
        .into_iter()
        .filter(|(file, _)| *file == path)
        .map(|(_, name)| name)
        .collect()
}

/// The fingerprints of the functions a document lists in the file given, by their readable name
/// paths.
fn fingerprints<'d>(document: &'d Value, path: &str) -> BTreeMap<&'d str, &'d str> {
    document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .filter(|function| function["path"] == json!(path))
        .map(|function| {
            (
                function["function"].as_str().expect("a name path"),
                function["fingerprint"].as_str().expect("a fingerprint"),
            )
        })
        .collect()
}

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

#[test]
fn with_no_limit_declared_it_lists_no_function_and_says_so() {
    let project = Project::new("complexity-no-limit");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let document = complexity(&project, &[]);

    assert_eq!(document["interfaceVersion"], json!(1));
    assert_eq!(
        document["limits"],
        json!({ "cyclomatic": null, "cognitive": null })
    );
    assert_eq!(document["functions"], json!([]));
    assert_eq!(
        notices(&document),
        ["complexity-not-checked", "no-hand-off-file"]
    );
    assert!(
        sentence(&document, "complexity-not-checked")
            .contains("`cyclomaticLimit` or `cognitiveLimit`")
    );

    // With `--all`, every function is listed, over nothing (spec SOLO-3).
    let document = complexity(&project, &["--all"]);
    let function = &document["functions"][0];
    let fingerprint = function["fingerprint"].as_str().expect("a fingerprint");

    assert_eq!(fingerprint.len(), 32);
    assert!(fingerprint.chars().all(|digit| digit.is_ascii_hexdigit()));
    assert_eq!(
        document["functions"],
        json!([{
            "path": "src/index.ts",
            "line": 1,
            "column": 20,
            "function": "run",
            "fingerprint": fingerprint,
            "cyclomatic": 1,
            "cognitive": 0,
            "over": [],
        }])
    );
}

#[test]
fn the_project_s_limit_wins_over_the_pack_s_and_a_later_stack_s_over_an_earlier_one() {
    let project = Project::new("complexity-limits");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let stack = |name: &str, pack: Value, own: Value| json!({ "name": name, "interfaceVersion": 1, "pack": pack, "project": own });
    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([
            stack(
                "first",
                json!({ "cyclomaticLimit": 5 }),
                json!({ "cognitiveLimit": 9 })
            ),
            stack(
                "second",
                json!({ "cyclomaticLimit": 7, "cognitiveLimit": 3 }),
                json!({})
            ),
        ]),
    );
    let document = complexity(&project, &["--hand-off", &hand_off]);

    assert_eq!(
        document["limits"],
        json!({ "cyclomatic": 7, "cognitive": 9 })
    );
    assert!(!notices(&document).contains(&"complexity-not-checked"));
}

#[test]
fn with_one_limit_declared_only_its_measure_is_checked_with_a_notice() {
    let project = Project::new("complexity-one-limit");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let hand_off = declaring(&project, &json!({ "cognitiveLimit": 15 }), &json!({}));
    let document = complexity(&project, &["--hand-off", &hand_off]);

    assert_eq!(
        document["limits"],
        json!({ "cyclomatic": null, "cognitive": 15 })
    );
    assert!(
        sentence(&document, "complexity-not-checked")
            .starts_with("No `cyclomaticLimit` is declared")
    );
}

#[test]
fn a_score_equal_to_its_limit_is_not_over_it() {
    let project = Project::new("complexity-equal");

    project
        .write("src/index.ts", "export const run = () => 1;\n")
        .commit();

    let hand_off = declaring(
        &project,
        &json!({ "cyclomaticLimit": 1, "cognitiveLimit": 1 }),
        &json!({}),
    );

    assert_eq!(
        complexity(&project, &["--hand-off", &hand_off])["functions"],
        json!([])
    );
    assert_eq!(
        complexity(&project, &["--all", "--hand-off", &hand_off])["functions"][0]["over"],
        json!([])
    );
}

#[test]
fn names_each_function_by_what_holds_it_and_numbers_the_repeats() {
    let project = Project::new("complexity-names");

    project
        .write(
            "src/names.ts",
            r#"export function read() {}
export function load(a: string): void;
export function load(a: number): void;
export function load(a: unknown): void {}
const parse = () => {};
const object = {
  get value() { return 1; },
  set value(v: number) {},
  run() {},
  "": () => 0,
  'read #2': () => 0,
  [Symbol.iterator]: () => 0,
  data: { size: 1 },
};
const first = { go() {} };
const second = { go() {} };
a.b.read = function () {};
c.b.read = function () {};
a.b.read = function () {};
{ const local = () => {}; }
{ const local = () => {}; }
const Parser = class { read() {} };
items.map((item) => item);
items.map((item) => item);
it("reads a file", () => {});
it(`reads a file`, () => {});
promise.then(() => 1).catch(() => 2);
describe.each(rows)("x", () => {});
new Promise((resolve) => resolve(1));
"#,
        )
        .write(
            "src/component.ts",
            "export default defineComponent({ setup() {} });\n",
        )
        .write("src/anonymous.ts", "export default function () {}\n")
        .write("src/settings.ts", "export default { run() {} };\n")
        .commit();

    let document = complexity(&project, &["--all"]);

    // Overload signatures take no place, and an object literal with no function in it is passed
    // over (spec FN-1, FN-5).
    assert_eq!(
        names(&document, "src/names.ts"),
        [
            "read",
            "load",
            "parse",
            "object > get value",
            "object > set value",
            "object > run",
            "object > ",
            "object > read #2",
            "object > <anonymous 1>",
            "first > go",
            "second > go",
            "a.b.read",
            "c.b.read",
            "a.b.read #2",
            "local",
            "local #2",
            "Parser > read",
            "items.map()",
            "items.map() #2",
            "it(\"reads a file\")",
            "it(\"reads a file\") #2",
            "promise.then()",
            "promise.then().catch()",
            "describe.each()(\"x\")",
            "new Promise()",
        ]
    );
    assert_eq!(
        names(&document, "src/component.ts"),
        ["defineComponent() > setup"]
    );
    assert_eq!(names(&document, "src/anonymous.ts"), ["default"]);
    assert_eq!(names(&document, "src/settings.ts"), ["default > run"]);
}

#[test]
fn places_a_function_at_its_own_name_or_else_at_its_first_character() {
    let project = Project::new("complexity-places");

    project
        .write(
            "src/places.ts",
            "export function own() {}\nconst arrow = () => {};\nclass Holder {\n  method() {}\n}\nconst wrapped = (() => {});\n",
        )
        .commit();

    let document = complexity(&project, &["--all"]);
    let places: Vec<(&str, u64, u64)> = document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .map(|function| {
            (
                function["function"].as_str().expect("a name path"),
                function["line"].as_u64().expect("a line"),
                function["column"].as_u64().expect("a column"),
            )
        })
        .collect();

    assert_eq!(
        places,
        [
            ("own", 1, 17),
            ("arrow", 2, 15),
            ("Holder > method", 4, 3),
            ("wrapped", 6, 18),
        ]
    );
}

#[test]
fn a_vue_component_s_script_blocks_are_one_file_and_an_mdx_document_is_not_scored() {
    let project = Project::new("complexity-vue");

    project
        .write(
            "src/App.vue",
            "<script lang=\"ts\">\nexport function shared() {}\n</script>\n\n<script setup lang=\"ts\">\nfunction shared() {}\nconst count = computed(() => 1);\n</script>\n<template><p>{{ count }}</p></template>\n",
        )
        .write(
            "src/Inline.vue",
            "<script>export const inline = () => 1;</script>\n",
        )
        .write("docs/page.mdx", "export function shown() { return 1; }\n\n# Title\n")
        .commit();

    let document = complexity(&project, &["--all"]);
    let places: Vec<(&str, &str, u64, u64)> = document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .map(|function| {
            (
                function["path"].as_str().expect("a path"),
                function["function"].as_str().expect("a name path"),
                function["line"].as_u64().expect("a line"),
                function["column"].as_u64().expect("a column"),
            )
        })
        .collect();

    assert_eq!(
        places,
        [
            ("src/App.vue", "shared", 2, 17),
            ("src/App.vue", "shared #2", 6, 10),
            ("src/App.vue", "computed()", 7, 24),
            ("src/Inline.vue", "inline", 1, 31),
        ]
    );
}

#[test]
fn leaves_out_data_generated_and_type_declaration_files() {
    let project = Project::new("complexity-left-out");

    project
        .write("src/index.ts", "export function kept() {}\n")
        .write("src/generated/made.ts", "export function made() {}\n")
        .write("src/fixtures/sample.ts", "export function sample() {}\n")
        .write("src/types.d.ts", "export declare function typed(): void;\n")
        .commit();

    let hand_off = declaring(
        &project,
        &json!({ "data": ["src/fixtures/**"], "generated": ["src/generated/**"] }),
        &json!({}),
    );
    let document = complexity(&project, &["--hand-off", &hand_off, "--all"]);

    assert_eq!(listed(&document), [("src/index.ts", "kept")]);
}

#[test]
fn a_fingerprint_holds_no_line_and_no_formatting_and_tells_names_apart() {
    let project = Project::new("complexity-fingerprints");

    project
        .write(
            "src/calls.ts",
            "items.map((item) => item);\nit('reads a file', () => {});\na.b.read = function () {};\n",
        )
        .write(
            "src/holder.js",
            "export const holder = { read() {}, read: () => 0, 'read #2': () => 0 };\n",
        )
        .commit();

    let before = complexity(&project, &["--all"]);

    // A formatter's rewrap, a comment, a quote change and optional chaining move no fingerprint (spec
    // FN-4).
    project.write(
        "src/calls.ts",
        "// A comment moves every line.\nitems\n  ?.map((item) => item);\nit(\"reads a file\", () => {});\na\n  .b /* a comment */ .read = function () {};\n",
    );

    let after = complexity(&project, &["--all"]);

    assert_eq!(
        fingerprints(&before, "src/calls.ts"),
        fingerprints(&after, "src/calls.ts")
    );
    assert_eq!(fingerprints(&after, "src/calls.ts").len(), 3);

    // The second `read` and the key `'read #2'` read alike and are told apart (spec CPX-4).
    let holder: Vec<&str> = after["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .filter(|function| function["path"] == json!("src/holder.js"))
        .map(|function| function["fingerprint"].as_str().expect("a fingerprint"))
        .collect();

    assert_eq!(
        names(&after, "src/holder.js"),
        ["holder > read", "holder > read #2", "holder > read #2"]
    );
    assert_ne!(holder[1], holder[2]);
}

#[test]
fn refuses_a_command_line_it_does_not_answer_and_stops_on_a_hand_off_file_it_cannot_read() {
    let project = Project::new("complexity-refusals");

    project.write("src/index.ts", "export {};\n").commit();

    for options in [
        &["--all", "--all"][..],
        &["--hand-off"],
        &["--base", "main"],
        &["--hand-off", "a.json", "--hand-off", "b.json"],
    ] {
        let output = project.start("complexity", |binary| {
            binary.args(options);
        });
        let line = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{options:?}");
        assert!(output.stdout.is_empty());
        assert!(
            line.contains("complexity [--hand-off <file>] [--all]"),
            "{line}"
        );
    }

    let missing = project.home.join("missing.json");
    let missing = missing.to_str().expect("the test's folder is valid UTF-8");
    let line = stopped(&project.start("complexity", |binary| {
        binary.args(["--hand-off", missing]);
    }));

    assert!(line.contains("cannot be read"), "{line}");
}
```

```rust edit path=tests/audit.test.rs
<<<<<<< find
fn prints_one_report_that_names_the_tool_the_interface_version_and_the_five_rules() {
=======
fn prints_one_report_that_names_the_tool_the_interface_version_and_the_six_rules() {
>>>>>>> replace
<<<<<<< find
            "unresolved-import",
            "circular-dependency",
        ]
=======
            "unresolved-import",
            "circular-dependency",
            "complexity",
        ]
>>>>>>> replace
```

```rust edit path=tests/cli.test.rs
<<<<<<< find
        run["tool"]["driver"]["rules"].as_array().map(Vec::len),
        Some(5)
    );
=======
        run["tool"]["driver"]["rules"].as_array().map(Vec::len),
        Some(6)
    );
>>>>>>> replace
<<<<<<< find
        assert!(
            error.contains("--version, interface, graph [--hand-off <file>], and audit"),
            "{args:?}"
        );
=======
        assert!(
            error.contains("and complexity [--hand-off <file>] [--all]"),
            "{args:?}"
        );
>>>>>>> replace
<<<<<<< find
    assert!(error.contains("--version, interface, graph [--hand-off <file>], and audit"));
=======
    assert!(error.contains("and complexity [--hand-off <file>] [--all]"));
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test complexity --test audit --test cli
```

Expected: the build fails, since `Cargo.toml` names no `complexity` test and the binary answers no
`complexity` command; once the entry is there, every test of `complexity` fails on the refused
command line, and the audit and cli tests on the five rules.

- [ ] **Step 3: Write the code**

```rust path=src/functions.rs
//! What a function is, its name, and its scores: every function of a script, found in the parse that
//! finds its imports, with the classes and object literals around it (complexity spec FN-1 to FN-5,
//! SCORE-1 to SCORE-3).
//!
//! The walk records units in source order: each function, class and object literal, with the unit
//! around it. A file's name paths are worked out once all its units are known, since a class or an
//! object literal takes part in them only where a function is inside it (spec FN-5).

use std::collections::BTreeMap;

use oxc_ast::AstKind;
use oxc_ast::ast::{
    self, Argument, ArrowFunctionExpression, AssignmentTarget, BindingIdentifier, BindingPattern,
    ChainElement, Class, Expression, FormalParameters, MethodDefinitionKind, ObjectExpression,
    Program, PropertyKey, PropertyKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{NodeId, ScopeFlags, Semantic};
use oxc_span::{GetSpan, Span};

use crate::imports::{DEFAULT, Lines};

/// One function, class or object literal of a file, as the walk found it.
#[derive(Debug, PartialEq, Eq)]
pub struct Unit {
    /// The unit around it, by its place in the file's list; none at the file's top.
    pub parent: Option<usize>,
    /// Its own name, where it has one (spec FN-4).
    pub name: Option<String>,
    /// Where a function is and its scores; none for a class or an object literal.
    pub scored: Option<Scored>,
    /// Whether it is a function, or a function is somewhere inside it (spec FN-5).
    pub holds: bool,
}

/// Where a function is, and its scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scored {
    /// The line it is placed on, counted from one (spec CPX-2).
    pub line: usize,
    /// The column it is placed at, counted from one in UTF-16 code units.
    pub column: usize,
    /// Its cyclomatic complexity (spec SCORE-1).
    pub cyclomatic: u64,
    /// Its cognitive complexity (spec SCORE-1).
    pub cognitive: u64,
}

/// One part of a name path (spec FN-5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    /// Its name; none for an unnamed part.
    pub name: Option<String>,
    /// Its number among the parts of the same name, or among the unnamed ones, in what is around it.
    pub number: usize,
}

/// One function, with its name path (spec FN-5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// The names of what is around it, from the outside in, then its own.
    pub path: Vec<Part>,
    /// Where it is, and its scores.
    pub scored: Scored,
}

impl Function {
    /// Its name path as a person reads it, as in `Parser > read #2 > <anonymous 1>` (spec FN-5).
    pub fn readable(&self) -> String {
        self.path
            .iter()
            .map(|part| match (&part.name, part.number) {
                (Some(name), 1) => name.clone(),
                (Some(name), number) => format!("{name} #{number}"),
                (None, number) => format!("<anonymous {number}>"),
            })
            .collect::<Vec<_>>()
            .join(" > ")
    }
}

/// The functions of a file's units, in source order, each with its name path. Within what is around
/// them, the parts that take part are counted in source order: a name's uses 1, 2 and on, and the
/// unnamed ones among themselves (spec FN-5).
pub fn named(units: &[Unit]) -> Vec<Function> {
    let mut paths: Vec<Option<Vec<Part>>> = Vec::with_capacity(units.len());
    let mut counts: BTreeMap<(Option<usize>, Option<&str>), usize> = BTreeMap::new();
    let mut functions = Vec::new();

    for unit in units {
        let around = match unit.parent {
            Some(parent) => paths.get(parent).cloned().flatten(),
            None => Some(Vec::new()),
        };
        // A class or an object literal with no function inside takes no place (spec FN-5).
        let path = around.filter(|_| unit.holds).map(|mut path| {
            let count = counts
                .entry((unit.parent, unit.name.as_deref()))
                .or_default();

            *count += 1;
            path.push(Part {
                name: unit.name.clone(),
                number: *count,
            });
            path
        });

        if let (Some(path), Some(scored)) = (&path, unit.scored) {
            functions.push(Function {
                path: path.clone(),
                scored,
            });
        }

        paths.push(path);
    }

    functions
}

/// Adds the units of a Vue component's script block to the component's, so that its blocks count as
/// one file, with their places counted in the file: `above` lines above the block's content, and
/// `indent` columns before it on its first line (spec FN-5).
pub fn append(units: &mut Vec<Unit>, block: Vec<Unit>, above: usize, indent: usize) {
    let offset = units.len();

    units.extend(block.into_iter().map(|unit| Unit {
        parent: unit.parent.map(|parent| parent + offset),
        scored: unit.scored.map(|scored| Scored {
            line: scored.line + above,
            column: if scored.line == 1 {
                scored.column + indent
            } else {
                scored.column
            },
            ..scored
        }),
        ..unit
    }));
}

/// The units of a parsed program, with what the binding analysis knows of its nodes (spec FN-3).
pub fn units<'a>(program: &Program<'a>, semantic: &Semantic<'a>, lines: &Lines<'_>) -> Vec<Unit> {
    let mut scorer = Scorer {
        semantic,
        lines,
        units: Vec::new(),
        open: Vec::new(),
        frames: Vec::new(),
    };

    scorer.visit_program(program);
    scorer.units
}

/// The scores so far of a function the walk is inside.
struct Frame {
    /// The function, by its place among the units.
    unit: usize,
    /// Its cyclomatic complexity so far.
    cyclomatic: u64,
    /// Its cognitive complexity so far.
    cognitive: u64,
}

/// Walks a program and records its units, with each function's scores.
struct Scorer<'s, 'a> {
    semantic: &'s Semantic<'a>,
    lines: &'s Lines<'s>,
    units: Vec<Unit>,
    /// The units the walk is inside, innermost last.
    open: Vec<usize>,
    /// The functions the walk is inside, innermost last.
    frames: Vec<Frame>,
}

impl<'a> Scorer<'_, 'a> {
    /// Records a unit the walk enters: a function, with its parameters, which takes a frame of scores
    /// of its own, or a class or an object literal. `own` is its own identifier, where it has one.
    fn enter(
        &mut self,
        node: NodeId,
        span: Span,
        own: Option<&BindingIdentifier<'a>>,
        params: Option<&FormalParameters<'a>>,
    ) {
        let (name, at) = match own {
            Some(identifier) => (Some(identifier.name.to_string()), identifier.span.start),
            None => self.name(node, span),
        };
        let place = self.units.len();
        let scored = params.map(|_| {
            let (line, column) = self.lines.place(at);

            Scored {
                line,
                column,
                cyclomatic: 1,
                cognitive: 0,
            }
        });

        if params.is_some() {
            // Each unit around a function holds one (spec FN-5).
            for open in self.open.iter().rev() {
                if self.units[*open].holds {
                    break;
                }

                self.units[*open].holds = true;
            }

            self.frames.push(Frame {
                unit: place,
                cyclomatic: 1,
                cognitive: 0,
            });
        }

        self.units.push(Unit {
            parent: self.open.last().copied(),
            name,
            scored,
            holds: params.is_some(),
        });
        self.open.push(place);
    }

    /// Records that the walk leaves the innermost unit, with a function's scores.
    fn leave(&mut self, function: bool) {
        if function
            && let Some(frame) = self.frames.pop()
            && let Some(scored) = &mut self.units[frame.unit].scored
        {
            scored.cyclomatic = frame.cyclomatic;
            scored.cognitive = frame.cognitive;
        }

        self.open.pop();
    }

    /// What holds a unit, past the parentheses and type assertions around it, with the span of the
    /// outermost of those, which is what the holder holds.
    fn holder(&self, node: NodeId, span: Span) -> (Span, AstKind<'a>) {
        let nodes = self.semantic.nodes();
        let mut node = node;
        let mut span = span;

        loop {
            let parent = nodes.parent_id(node);
            let kind = nodes.kind(parent);

            match kind {
                AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSTypeAssertion(_) => {
                    node = parent;
                    span = kind.span();
                }
                _ => return (span, kind),
            }
        }
    }

    /// A unit's name, from what holds it, and the offset it is placed at: the key, where the name is
    /// a method's or a property's, and otherwise the unit's own start (spec FN-4, CPX-2).
    fn name(&self, node: NodeId, span: Span) -> (Option<String>, u32) {
        let (outer, holder) = self.holder(node, span);

        if let Some((name, at)) = declared(holder, outer) {
            return (Some(name), at.unwrap_or(span.start));
        }

        let name = match holder {
            AstKind::AssignmentExpression(assignment) if assignment.right.span() == outer => {
                target(&assignment.left)
            }
            AstKind::CallExpression(call) if passed(&call.arguments, outer) => {
                called(&call.callee, &call.arguments)
            }
            AstKind::NewExpression(call) if passed(&call.arguments, outer) => {
                called(&call.callee, &call.arguments).map(|called| format!("new {called}"))
            }
            _ => None,
        };

        (name, span.start)
    }
}

/// The name a declaration gives what it holds, with the offset the name is placed at where it is a
/// key: the variable it is declared to, the key of an object's property or a class's member, with
/// `get ` or `set ` before an accessor's, or `default` for an anonymous default export. A computed key
/// gives no name (spec FN-4, CPX-2).
fn declared(holder: AstKind<'_>, outer: Span) -> Option<(String, Option<u32>)> {
    let keyed = |key: &PropertyKey<'_>, computed: bool, before: &str| {
        key_name(key, computed).map(|name| (format!("{before}{name}"), Some(key.span().start)))
    };

    match holder {
        AstKind::VariableDeclarator(declarator) if holds(declarator.init.as_ref(), outer) => {
            match &declarator.id {
                BindingPattern::BindingIdentifier(identifier) => {
                    Some((identifier.name.to_string(), None))
                }
                _ => None,
            }
        }
        AstKind::ObjectProperty(property) if property.value.span() == outer => {
            let before = match property.kind {
                PropertyKind::Get => "get ",
                PropertyKind::Set => "set ",
                PropertyKind::Init => "",
            };

            keyed(&property.key, property.computed, before)
        }
        AstKind::MethodDefinition(method) => {
            let before = match method.kind {
                MethodDefinitionKind::Get => "get ",
                MethodDefinitionKind::Set => "set ",
                MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => "",
            };

            keyed(&method.key, method.computed, before)
        }
        AstKind::PropertyDefinition(property) if holds(property.value.as_ref(), outer) => {
            keyed(&property.key, property.computed, "")
        }
        AstKind::AccessorProperty(property) if holds(property.value.as_ref(), outer) => {
            keyed(&property.key, property.computed, "")
        }
        AstKind::ExportDefaultDeclaration(_) => Some((DEFAULT.to_owned(), None)),
        _ => None,
    }
}

/// Whether a value a declaration may hold is the unit of the span given.
fn holds(value: Option<&Expression<'_>>, outer: Span) -> bool {
    value.is_some_and(|value| value.span() == outer)
}

/// Whether the unit of the span given is one of a call's arguments.
fn passed(arguments: &[Argument<'_>], outer: Span) -> bool {
    arguments.iter().any(|argument| argument.span() == outer)
}

/// The name of a key that is not computed: an identifier, a private name with its `#`, or a literal's
/// value.
fn key_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    match key {
        _ if computed => None,
        PropertyKey::PrivateIdentifier(private) => Some(format!("#{}", private.name)),
        _ => key.static_name().map(|name| name.display().to_string()),
    }
}

/// The whole left side of an assignment, as a chain of names (spec FN-4).
fn target(left: &AssignmentTarget<'_>) -> Option<String> {
    match left {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some(identifier.name.to_string())
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            after(&member.object, &format!(".{}", member.property.name))
        }
        AssignmentTarget::ComputedMemberExpression(member) => after(&member.object, "[]"),
        AssignmentTarget::PrivateFieldExpression(member) => {
            after(&member.object, &format!(".#{}", member.field.name))
        }
        AssignmentTarget::TSAsExpression(wrapped) => chain(&wrapped.expression),
        AssignmentTarget::TSSatisfiesExpression(wrapped) => chain(&wrapped.expression),
        AssignmentTarget::TSNonNullExpression(wrapped) => chain(&wrapped.expression),
        AssignmentTarget::TSTypeAssertion(wrapped) => chain(&wrapped.expression),
        _ => None,
    }
}

/// The name a function passed to a call takes from it: the callee with `()`, and inside them the
/// call's first argument where it is a string, written in double quotes (spec FN-4).
fn called(callee: &Expression<'_>, arguments: &[Argument<'_>]) -> Option<String> {
    let first = arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(literal)
        .map(|text| format!("\"{text}\""))
        .unwrap_or_default();

    chain(callee).map(|callee| format!("{callee}({first})"))
}

/// The value of a string literal, or of a template literal with no substitutions.
fn literal(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.display().to_string()),
        Expression::TemplateLiteral(template) => template
            .single_quasi()
            .map(|text| text.display().to_string()),
        _ => None,
    }
}

/// An expression as a chain of names joined by `.`, with `[]` for a computed member, `()` for a call
/// without its arguments, and `new` kept, whatever spacing, comments, parentheses or optional chaining
/// it is written with; none for any other expression (spec FN-4).
fn chain(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::ThisExpression(_) => Some("this".to_owned()),
        Expression::Super(_) => Some("super".to_owned()),
        Expression::StaticMemberExpression(member) => {
            after(&member.object, &format!(".{}", member.property.name))
        }
        Expression::ComputedMemberExpression(member) => after(&member.object, "[]"),
        Expression::PrivateFieldExpression(member) => {
            after(&member.object, &format!(".#{}", member.field.name))
        }
        Expression::CallExpression(call) => after(&call.callee, "()"),
        Expression::NewExpression(call) => {
            chain(&call.callee).map(|callee| format!("new {callee}()"))
        }
        Expression::ChainExpression(chained) => match &chained.expression {
            ChainElement::CallExpression(call) => after(&call.callee, "()"),
            ChainElement::TSNonNullExpression(wrapped) => chain(&wrapped.expression),
            ChainElement::StaticMemberExpression(member) => {
                after(&member.object, &format!(".{}", member.property.name))
            }
            ChainElement::ComputedMemberExpression(member) => after(&member.object, "[]"),
            ChainElement::PrivateFieldExpression(member) => {
                after(&member.object, &format!(".#{}", member.field.name))
            }
        },
        Expression::ParenthesizedExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSAsExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSSatisfiesExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSNonNullExpression(wrapped) => chain(&wrapped.expression),
        Expression::TSTypeAssertion(wrapped) => chain(&wrapped.expression),
        Expression::TSInstantiationExpression(wrapped) => chain(&wrapped.expression),
        _ => None,
    }
}

/// The chain of an expression with the text given after it.
fn after(object: &Expression<'_>, text: &str) -> Option<String> {
    chain(object).map(|object| format!("{object}{text}"))
}

impl<'a> Visit<'a> for Scorer<'_, 'a> {
    fn visit_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        // A declaration with no body is not a function: an overload, a `declare function` or an
        // abstract method (spec FN-1).
        let params = function.body.as_ref().map(|_| &*function.params);

        if params.is_some() {
            self.enter(
                function.node_id.get(),
                function.span,
                function.id.as_ref(),
                params,
            );
        }

        walk::walk_function(self, function, flags);

        if params.is_some() {
            self.leave(true);
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.enter(arrow.node_id.get(), arrow.span, None, Some(&*arrow.params));
        walk::walk_arrow_function_expression(self, arrow);
        self.leave(true);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.enter(class.node_id.get(), class.span, class.id.as_ref(), None);
        walk::walk_class(self, class);
        self.leave(false);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        self.enter(object.node_id.get(), object.span, None, None);
        walk::walk_object_expression(self, object);
        self.leave(false);
    }
}
```

```rust path=src/complexity.rs
//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint of its
//! finding, and the document the command prints (complexity spec LIMIT-1, LIMIT-2, CPX-4, SOLO-1 to
//! SOLO-5).

use std::path::Path;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::files::{self, Kind};
use crate::findings::{self, Rule};
use crate::functions::{self, Function};
use crate::hand_off::{HandOff, Stack};
use crate::notice::{self, Notice};
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::timings::Stages;
use crate::tree::Tree;

/// The kind of the notice for a measure no declaration gives a limit (spec LIMIT-2).
const NOT_CHECKED: &str = "complexity-not-checked";

/// The name of the cyclomatic measure in what is printed (spec CPX-3).
const CYCLOMATIC: &str = "cyclomatic";

/// The name of the cognitive measure in what is printed (spec CPX-3).
const COGNITIVE: &str = "cognitive";

/// The limit of each measure, where one is declared (spec LIMIT-1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    /// The cyclomatic limit, from `cyclomaticLimit`.
    pub cyclomatic: Option<u64>,
    /// The cognitive limit, from `cognitiveLimit`.
    pub cognitive: Option<u64>,
}

impl Limits {
    /// The limits the hand-off file declares, one value of each for the whole run: the project's over
    /// the pack's, and of values from the same source, the later stack's (spec LIMIT-1; design ENT-4).
    pub fn of(handed: Option<&HandOff>) -> Self {
        let stacks = handed.map_or(&[][..], |handed| handed.stacks.as_slice());

        Self {
            cyclomatic: declared(stacks, "cyclomaticLimit"),
            cognitive: declared(stacks, "cognitiveLimit"),
        }
    }

    /// The notice for the measures with no limit, which are not checked, where there are some (spec
    /// LIMIT-2).
    pub fn notice(self) -> Option<Notice> {
        let sentence = match (self.cyclomatic, self.cognitive) {
            (Some(_), Some(_)) => return None,
            (None, None) => {
                "No `cyclomaticLimit` or `cognitiveLimit` is declared, so no function's complexity is \
                 checked and no complexity finding is reported."
            }
            (None, Some(_)) => {
                "No `cyclomaticLimit` is declared, so functions' cyclomatic complexity is not checked."
            }
            (Some(_), None) => {
                "No `cognitiveLimit` is declared, so functions' cognitive complexity is not checked."
            }
        };

        Some(Notice::new(NOT_CHECKED, None, sentence.to_owned()))
    }

    /// The measures a function is over, cyclomatic first: a score greater than its limit, and not one
    /// equal to it (spec CPX-3; the spec's terms).
    pub fn over(self, function: &Function) -> Vec<&'static str> {
        [
            (CYCLOMATIC, self.cyclomatic, function.scored.cyclomatic),
            (COGNITIVE, self.cognitive, function.scored.cognitive),
        ]
        .into_iter()
        .filter(|(_, limit, score)| limit.is_some_and(|limit| *score > limit))
        .map(|(measure, _, _)| measure)
        .collect()
    }
}

/// The value of a limit's key that wins: the last stack's that the project states it for, or else the
/// last stack's that the pack states it for.
fn declared(stacks: &[Stack], key: &str) -> Option<u64> {
    let last = |project: bool| {
        stacks.iter().rev().find_map(|stack| {
            let declarations = if project { &stack.project } else { &stack.pack };

            declarations.get(key).and_then(Value::as_u64)
        })
    };

    last(true).or_else(|| last(false))
}

/// One function of a file, with the measures it is over and the fingerprint of its finding.
pub struct Measured {
    /// The file, by its path from the root.
    pub path: String,
    /// The function, with its name path and scores.
    pub function: Function,
    /// The measures it is over, cyclomatic first.
    pub over: Vec<&'static str>,
    /// The fingerprint `audit` gives, or would give, its finding (spec CPX-4).
    pub fingerprint: String,
}

/// Every function of the reading's own scripts and Vue components, sorted by path, line and column.
/// A file declared data or generated, and a type declaration file, gives none (spec FN-2, CPX-5,
/// SOLO-2).
pub fn measured(reading: &Reading, limits: Limits) -> Vec<Measured> {
    let mut measured = Vec::new();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || reading.marks.generated.contains(&file.path)
        {
            continue;
        }

        let Some(read) = reading.scripts.read.get(&file.path) else {
            continue;
        };

        for function in functions::named(&read.units) {
            measured.push(Measured {
                over: limits.over(&function),
                fingerprint: fingerprint(&file.path, &function),
                path: file.path.clone(),
                function,
            });
        }
    }

    measured.sort_by(|a, b| {
        (&a.path, a.function.scored.line, a.function.scored.column).cmp(&(
            &b.path,
            b.function.scored.line,
            b.function.scored.column,
        ))
    });

    measured
}

/// The fingerprint of a function's finding: its rule, its file's path, and for each part of its name
/// path `n` for a named part or `u` for an unnamed one, its name, and its number in decimal digits
/// (spec CPX-4; design IF-4).
pub fn fingerprint(path: &str, function: &Function) -> String {
    let mut parts = vec![path.to_owned()];

    for part in &function.path {
        parts.push(if part.name.is_some() { "n" } else { "u" }.to_owned());
        parts.push(part.name.clone().unwrap_or_default());
        parts.push(part.number.to_string());
    }

    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

    findings::fingerprint(Rule::Complexity, &parts)
}

/// Reads the working tree as `audit` does, with the hand-off file given or with none, and returns the
/// document `complexity` prints: the limits, the functions over a limit, or with `all` every function,
/// and the notices (spec SOLO-1 to SOLO-3).
pub fn run(hand_off: Option<&Path>, all: bool) -> Result<String, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = reading::start(hand_off)?;
    let listing = files::list(&root, &mut notices)?;
    let limits = Limits::of(handed.as_ref());
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
    let functions: Vec<Value> = measured(&reading, limits)
        .iter()
        .filter(|measured| all || !measured.over.is_empty())
        .map(entry)
        .collect();

    reading.notices.extend(limits.notice());

    let mut document = json!({
        "functions": functions,
        "interfaceVersion": INTERFACE_VERSION,
        "limits": { CYCLOMATIC: limits.cyclomatic, COGNITIVE: limits.cognitive },
        "notices": notice::printed(reading.notices),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// One function as the document lists it (spec SOLO-2).
fn entry(measured: &Measured) -> Value {
    json!({
        "column": measured.function.scored.column,
        "cognitive": measured.function.scored.cognitive,
        "cyclomatic": measured.function.scored.cyclomatic,
        "fingerprint": measured.fingerprint,
        "function": measured.function.readable(),
        "line": measured.function.scored.line,
        "over": measured.over,
        "path": measured.path,
    })
}
```

```rust edit path=src/imports.rs
<<<<<<< find
use oxc_span::{GetSpan, SourceType, Span};
=======
use oxc_span::{GetSpan, SourceType, Span};

use crate::functions::{self, Unit};
>>>>>>> replace
<<<<<<< find
    /// The uses a file makes of its own exports from outside its scripts, as a Vue component's
    /// template does (spec USE-4).
    pub named: Vec<Used>,
}
=======
    /// The uses a file makes of its own exports from outside its scripts, as a Vue component's
    /// template does (spec USE-4).
    pub named: Vec<Used>,
    /// Its functions, classes and object literals, in source order, from which its functions' names
    /// and scores are read (complexity spec FN-3).
    pub units: Vec<Unit>,
}
>>>>>>> replace
<<<<<<< find
    finder.followed();
    finder.used();

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
        exports,
        named: Vec::new(),
    })
=======
    finder.followed();
    finder.used();

    // The functions are found and scored in the same parse (complexity spec FN-3).
    let units = functions::units(&parsed.program, &semantic, &lines);

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
        exports,
        named: Vec::new(),
        units,
    })
>>>>>>> replace
<<<<<<< find
/// Where each line of a text begins, to turn an offset into a line and a column.
struct Lines<'t> {
=======
/// Where each line of a text begins, to turn an offset into a line and a column.
pub struct Lines<'t> {
>>>>>>> replace
<<<<<<< find
    /// The starts of the lines of a text.
    fn of(
=======
    /// The starts of the lines of a text.
    pub fn of(
>>>>>>> replace
<<<<<<< find
    /// code units, as SARIF counts one.
    fn place(
=======
    /// code units, as SARIF counts one.
    pub fn place(
>>>>>>> replace
```

```rust edit path=src/pages.rs
<<<<<<< find
use crate::imports::{self, Broken, DEFAULT, EVERY, Form, Import, Name, Read, Used};
=======
use crate::functions;
use crate::imports::{self, Broken, DEFAULT, EVERY, Form, Import, Name, Read, Used};
>>>>>>> replace
<<<<<<< find
        // The lines of the file above the block's content.
        let above = line_at(text, block.content.start) - 1;
=======
        // The lines of the file above the block's content.
        let above = line_at(text, block.content.start) - 1;
        // The columns of the block's first line before its content (complexity spec FN-5).
        let indent = text[..block.content.start]
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .encode_utf16()
            .count();
>>>>>>> replace
<<<<<<< find
                read.computed += found.computed;
=======
                read.computed += found.computed;
                functions::append(&mut read.units, found.units, above, indent);
>>>>>>> replace
```

```rust edit path=src/findings.rs
<<<<<<< find
    /// Files that import each other in a loop.
    CircularDependency,
}
=======
    /// Files that import each other in a loop.
    CircularDependency,
    /// A function whose complexity is over a declared limit (complexity spec CPX-1).
    Complexity,
}
>>>>>>> replace
<<<<<<< find
    pub const ALL: [Self; 5] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
    ];
=======
    pub const ALL: [Self; 6] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
        Self::Complexity,
    ];
>>>>>>> replace
<<<<<<< find
            Self::CircularDependency => "circular-dependency",
=======
            Self::CircularDependency => "circular-dependency",
            Self::Complexity => "complexity",
>>>>>>> replace
<<<<<<< find
            Self::CircularDependency => "Files that import each other in a loop.",
=======
            Self::CircularDependency => "Files that import each other in a loop.",
            Self::Complexity => {
                "A function whose cyclomatic or cognitive complexity is over its limit."
            }
>>>>>>> replace
<<<<<<< find
/// (spec REPORT-3).
fn fingerprint(rule: Rule, parts: &[&str]) -> String {
=======
/// (spec REPORT-3; complexity spec CPX-4).
pub fn fingerprint(rule: Rule, parts: &[&str]) -> String {
>>>>>>> replace
```

```rust edit path=src/audit.rs
<<<<<<< find
//! The run names the tool and lists the five rules, the findings and the notices.
=======
//! The run names the tool and lists its rules, the findings and the notices.
>>>>>>> replace
```

```rust edit path=src/cli.rs
<<<<<<< find
/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, graph [--hand-off <file>], and audit [--hand-off <file>] \
                        [--base <commit>]";
=======
/// The command that reads a project and prints each function's complexity (complexity spec SOLO-1).
const COMPLEXITY_COMMAND: &str = "complexity";

/// The option of `complexity` that lists every function, not only those over a limit (complexity spec
/// SOLO-3).
const ALL_OPTION: &str = "--all";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, graph [--hand-off <file>], audit [--hand-off <file>] \
                        [--base <commit>], and complexity [--hand-off <file>] [--all]";
>>>>>>> replace
<<<<<<< find
    Audit {
        hand_off: Option<PathBuf>,
        base: Option<String>,
    },
}
=======
    Audit {
        hand_off: Option<PathBuf>,
        base: Option<String>,
    },
    /// Read the project and print each function's complexity, with a hand-off file or without one: the
    /// functions over a limit, or with `all` every function (complexity spec SOLO-1).
    Complexity {
        hand_off: Option<PathBuf>,
        all: bool,
    },
}
>>>>>>> replace
<<<<<<< find
        [AUDIT_COMMAND, ..] => audit(args),
=======
        [AUDIT_COMMAND, ..] => audit(args),
        [COMPLEXITY_COMMAND, ..] => complexity(args),
>>>>>>> replace
<<<<<<< find
/// The refusal of a command line the binary does not answer.
=======
/// `complexity` and its options, each at most once, in either order (complexity spec SOLO-1).
fn complexity(args: &[OsString]) -> Result<Command, String> {
    let mut hand_off = None;
    let mut all = false;
    let mut words = args[1..].iter();

    while let Some(option) = words.next() {
        match option.to_str() {
            Some(HAND_OFF_OPTION) if hand_off.is_none() => {
                let Some(value) = words.next() else {
                    return Err(refused(args));
                };

                hand_off = Some(PathBuf::from(value));
            }
            Some(ALL_OPTION) if !all => all = true,
            _ => return Err(refused(args)),
        }
    }

    Ok(Command::Complexity { hand_off, all })
}

/// The refusal of a command line the binary does not answer.
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find
//! file is used. `graph` prints what was read, and `audit` prints a report of the findings it shows,
//! compared with a base commit where it is given one.
=======
//! file is used. `graph` prints what was read, `audit` prints a report of the findings it shows,
//! compared with a base commit where it is given one, and `complexity` prints the scores of each
//! function.
>>>>>>> replace
<<<<<<< find
mod command_files;
mod declarations;
=======
mod command_files;
mod complexity;
mod declarations;
>>>>>>> replace
<<<<<<< find
mod findings;
mod git;
=======
mod findings;
mod functions;
mod git;
>>>>>>> replace
<<<<<<< find
        Ok(Command::Audit { hand_off, base }) => {
            ended(audit::run(hand_off.as_deref(), base.as_deref()))
        }
=======
        Ok(Command::Audit { hand_off, base }) => {
            ended(audit::run(hand_off.as_deref(), base.as_deref()))
        }
        Ok(Command::Complexity { hand_off, all }) => {
            ended(complexity::run(hand_off.as_deref(), all))
        }
>>>>>>> replace
```

```toml edit path=Cargo.toml
<<<<<<< find
[[test]]
name = "timings"
path = "tests/timings.test.rs"
=======
[[test]]
name = "timings"
path = "tests/timings.test.rs"

[[test]]
name = "complexity"
path = "tests/complexity.test.rs"
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --locked --test complexity --test audit --test cli
cargo clippy --locked --all-targets
```

Expected: every test passes; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
git add -A && git commit -m "feat(complexity): name every function and list it with its limits"
```

Expected: one commit.

### Task 2: The scores

**Implements:** SCORE-1, SCORE-2, SCORE-3, CASE-1

**Test:** `cargo test --locked --test complexity`

**Files:**
- Modify: `src/functions.rs`, `tests/complexity.test.rs`

**Interfaces:**
- Consumes: Task 1's `Scorer`, `Frame`, `enter`, `leave`, `holder`, `declared` and `passed`.
- Produces: each function's cyclomatic and cognitive complexity as decision 1 lists the rules.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/complexity.test.rs
<<<<<<< find
    assert!(line.contains("cannot be read"), "{line}");
}
=======
    assert!(line.contains("cannot be read"), "{line}");
}

/// The functions a document lists in the file given, in its order: each one's readable name path,
/// cyclomatic complexity and cognitive complexity.
fn scores<'d>(document: &'d Value, path: &str) -> Vec<(&'d str, u64, u64)> {
    document["functions"]
        .as_array()
        .expect("a list of functions")
        .iter()
        .filter(|function| function["path"] == json!(path))
        .map(|function| {
            (
                function["function"].as_str().expect("a name path"),
                function["cyclomatic"].as_u64().expect("a score"),
                function["cognitive"].as_u64().expect("a score"),
            )
        })
        .collect()
}

#[test]
fn scores_each_function_as_fallow_counts_it() {
    let project = Project::new("complexity-scores");

    project
        .write(
            "src/scores.ts",
            r#"export function plain() {}
export function nested(a, b) { if (a) { if (b) {} } }
export function chained(a, b) { if (a) {} else if (b) {} else {} }
export function elsewise(a, b) { if (a) {} else { if (b) {} } }
export function looped(list) { for (const x of list) { if (x) {} } }
export function loops(a, list, value) { for (const k in value) {} for (let i = 0; i < 1; i++) {} while (a) {} do {} while (a); }
export function switched(c) { switch (c) { case 1: case 2: return 1; default: return 0; } }
export function caught(a) { try { if (a) {} } catch { if (a) {} } finally {} }
export function ternaries(x) { return x > 0 ? 1 : x < 0 ? -1 : 0; }
export function runs(a, b, c, d) { return a || b && c || d; }
export function grouped(a, b, c, d) { return (a || b) && c || d; }
export function passed(a, b, c) { return a && f(b && c); }
export function optional(a) { return [a?.b, a?.b?.c, a?.b.c, a?.b(), a?.()]; }
export function assigned(v) { v ||= 1; v &&= 2; v ??= 3; return v; }
export function labelled(list) { outer: for (const x of list) { for (const y of x) { if (y) continue outer; break outer; } } }
export function outer(a) { if (a) { const inner = (b) => { if (b) {} }; return inner; } }
export function builds(a) { return class { static { if (a) {} } }; }
export function defaults(a = b ?? 1) {}
class Top { static { if (x) {} } }
"#,
        )
        .commit();

    let document = complexity(&project, &["--all"]);

    // A nested function is scored on its own, and code outside every function, such as a static
    // block at the top, is never reported; one inside a function counts there (spec SCORE-1,
    // SCORE-2).
    assert_eq!(
        scores(&document, "src/scores.ts"),
        [
            ("plain", 1, 0),
            ("nested", 3, 3),
            ("chained", 3, 3),
            ("elsewise", 3, 4),
            ("looped", 3, 3),
            ("loops", 5, 4),
            ("switched", 3, 1),
            ("caught", 4, 4),
            ("ternaries", 3, 3),
            ("runs", 4, 2),
            ("grouped", 4, 3),
            ("passed", 3, 1),
            ("optional", 4, 0),
            ("assigned", 4, 0),
            ("labelled", 4, 8),
            ("outer", 2, 1),
            ("outer > inner", 2, 1),
            ("builds", 2, 1),
            ("defaults", 2, 1),
        ]
    );
}

#[test]
fn a_component_s_hooks_and_many_properties_add_to_its_cognitive_score() {
    let project = Project::new("complexity-react");

    project
        .write(
            "src/card.tsx",
            r#"export function Card({ a, b, c, d, e, f }) {
  const [x] = useState(0);
  useEffect(() => {});
  return <div>{a && b && x}</div>;
}
export const Panel = memo(() => { useMemo(() => 1); return <i />; });
export function helper() { useState(0); return <b />; }
"#,
        )
        .commit();

    let document = complexity(&project, &["--all"]);

    // A name read from what declares a function, or from the call it is passed to, makes it a
    // component; a lower-case name does not (spec SCORE-2).
    assert_eq!(
        scores(&document, "src/card.tsx"),
        [
            ("Card", 3, 5),
            ("Card > useEffect()", 1, 0),
            ("memo()", 1, 1),
            ("memo() > useMemo()", 1, 0),
            ("helper", 1, 0),
        ]
    );
}

#[test]
fn lists_the_functions_over_a_limit_with_the_measures_they_are_over() {
    let project = Project::new("complexity-over");

    project
        .write(
            "src/over.ts",
            "export function caught(a) { try { if (a) {} } catch { if (a) {} } }\nexport function nested(a, b) { if (a) { if (b) {} } }\nexport function assigned(v) { v ||= 1; v &&= 2; v ??= 3; return v; }\n",
        )
        .commit();

    let over = |document: &Value| -> Vec<(String, Value)> {
        document["functions"]
            .as_array()
            .expect("a list of functions")
            .iter()
            .map(|function| {
                (
                    function["function"]
                        .as_str()
                        .expect("a name path")
                        .to_owned(),
                    function["over"].clone(),
                )
            })
            .collect()
    };
    let both = declaring(
        &project,
        &json!({ "cyclomaticLimit": 3, "cognitiveLimit": 3 }),
        &json!({}),
    );

    assert_eq!(
        over(&complexity(&project, &["--hand-off", &both])),
        [
            ("caught".to_owned(), json!(["cyclomatic", "cognitive"])),
            ("assigned".to_owned(), json!(["cyclomatic"])),
        ]
    );

    let cognitive = declaring(&project, &json!({ "cognitiveLimit": 3 }), &json!({}));

    assert_eq!(
        over(&complexity(&project, &["--hand-off", &cognitive])),
        [("caught".to_owned(), json!(["cognitive"]))]
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test complexity
```

Expected: the three new tests fail, every function scoring 1 and 0.

- [ ] **Step 3: Write the code**

```rust edit path=src/functions.rs
<<<<<<< find
use oxc_ast::ast::{
    self, Argument, ArrowFunctionExpression, AssignmentTarget, BindingIdentifier, BindingPattern,
    ChainElement, Class, Expression, FormalParameters, MethodDefinitionKind, ObjectExpression,
    Program, PropertyKey, PropertyKind,
};
=======
use oxc_ast::ast::{
    self, Argument, ArrowFunctionExpression, AssignmentExpression, AssignmentTarget,
    BindingIdentifier, BindingPattern, BreakStatement, CallExpression, CatchClause, ChainElement,
    ChainExpression, Class, ConditionalExpression, ContinueStatement, DoWhileStatement, Expression,
    ForInStatement, ForOfStatement, ForStatement, FormalParameters, IfStatement, JSXElement,
    JSXFragment, LogicalExpression, LogicalOperator, MethodDefinitionKind, ObjectExpression,
    Program, PropertyKey, PropertyKind, Statement, SwitchCase, SwitchStatement, WhileStatement,
};
>>>>>>> replace
<<<<<<< find
use crate::imports::{DEFAULT, Lines};
=======
use crate::imports::{DEFAULT, Lines};

/// How many properties a component's first parameter may take apart before each more adds to its
/// cognitive complexity (fallow's `PROP_COUNT_FLOOR`; spec SCORE-2).
const PROPS_FLOOR: usize = 4;
>>>>>>> replace
<<<<<<< find
struct Frame {
    /// The function, by its place among the units.
    unit: usize,
    /// Its cyclomatic complexity so far.
    cyclomatic: u64,
    /// Its cognitive complexity so far.
    cognitive: u64,
}
=======
struct Frame {
    /// The function, by its place among the units.
    unit: usize,
    /// Its cyclomatic complexity so far.
    cyclomatic: u64,
    /// Its cognitive complexity so far.
    cognitive: u64,
    /// How many of the structures that nest are around the walk's place in it.
    nesting: u64,
    /// The operator of the last `&&`, `||` or `??` it counted, until a run of them ends (spec
    /// SCORE-2).
    last: Option<LogicalOperator>,
    /// Whether it renders JSX.
    jsx: bool,
    /// Whether its name is shaped like a React component's or hook's.
    react: bool,
    /// How many properties its first parameter takes apart, where it is an object pattern.
    props: usize,
}

impl Frame {
    /// What a component that takes many properties apart adds to its cognitive complexity: each
    /// property past the floor, where it renders JSX (fallow's `fold_component_prop_count`; spec
    /// SCORE-2).
    fn wide(&self) -> u64 {
        if self.jsx {
            u64::try_from(self.props.saturating_sub(PROPS_FLOOR)).unwrap_or(u64::MAX)
        } else {
            0
        }
    }
}
>>>>>>> replace
<<<<<<< find
            self.frames.push(Frame {
                unit: place,
                cyclomatic: 1,
                cognitive: 0,
            });
=======
            let react = self.react(node, span, own);
            let props = params
                .and_then(|params| params.items.first())
                .map_or(0, |first| match &first.pattern {
                    BindingPattern::ObjectPattern(object) => object.properties.len(),
                    _ => 0,
                });

            self.frames.push(Frame {
                unit: place,
                cyclomatic: 1,
                cognitive: 0,
                nesting: 0,
                last: None,
                jsx: false,
                react,
                props,
            });
>>>>>>> replace
<<<<<<< find
            scored.cognitive = frame.cognitive;
=======
            scored.cognitive = frame.cognitive + frame.wide();
>>>>>>> replace
<<<<<<< find
        (name, span.start)
    }
}
=======
        (name, span.start)
    }

    /// Whether a function's name is shaped like a React component's or hook's, read as fallow reads
    /// it: its own identifier, or else what declares it, or the call it is passed to, by name; never a
    /// name taken from a call (fallow's `frame_name_is_react_shaped`; spec SCORE-2).
    fn react(&self, node: NodeId, span: Span, own: Option<&BindingIdentifier<'a>>) -> bool {
        let mut node = node;
        let mut span = span;
        let name = own
            .map(|identifier| identifier.name.to_string())
            .or_else(|| {
                loop {
                    let (outer, holder) = self.holder(node, span);

                    if let Some((name, _)) = declared(holder, outer) {
                        break Some(name);
                    }

                    match holder {
                        AstKind::CallExpression(call) if passed(&call.arguments, outer) => {
                            node = call.node_id.get();
                            span = call.span;
                        }
                        _ => break None,
                    }
                }
            });

        name.is_some_and(|name| {
            name.starts_with(|first: char| first.is_ascii_uppercase()) || hook(&name)
        })
    }

    /// The innermost function's frame, where the walk is inside one.
    fn frame(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    /// Adds a path to the innermost function's cyclomatic complexity (spec SCORE-1, SCORE-2).
    fn branch(&mut self) {
        if let Some(frame) = self.frame() {
            frame.cyclomatic += 1;
        }
    }

    /// Adds one to the innermost function's cognitive complexity, whatever its nesting.
    fn flat(&mut self) {
        if let Some(frame) = self.frame() {
            frame.cognitive += 1;
        }
    }

    /// Adds one and the nesting to the innermost function's cognitive complexity.
    fn structural(&mut self) {
        if let Some(frame) = self.frame() {
            frame.cognitive += 1 + frame.nesting;
        }
    }

    /// Walks what the closure walks one level of nesting deeper in the innermost function.
    fn nested(&mut self, walk: impl FnOnce(&mut Self)) {
        if let Some(frame) = self.frame() {
            frame.nesting += 1;
        }

        walk(self);

        if let Some(frame) = self.frame() {
            frame.nesting -= 1;
        }
    }

    /// A loop: a path, and one and the nesting (spec SCORE-1).
    fn looped(&mut self) {
        self.branch();
        self.structural();
    }

    /// An `if` with the `else if`s and the `else` after it. Each `if` adds a path. The first adds one
    /// and its nesting to the cognitive score, and each `else if` and the `else` one, and each body is
    /// one level deeper than the first `if`; each test is at its level (fallow's `visit_if_chain`;
    /// spec SCORE-1, SCORE-2).
    fn if_chain(&mut self, statement: &IfStatement<'a>, alternate: bool) {
        self.branch();

        if alternate {
            self.flat();
        } else {
            self.structural();
        }

        self.visit_expression(&statement.test);
        self.nested(|scorer| scorer.visit_statement(&statement.consequent));

        match &statement.alternate {
            Some(Statement::IfStatement(next)) => self.if_chain(next, true),
            Some(otherwise) => {
                self.flat();
                self.nested(|scorer| scorer.visit_statement(otherwise));
            }
            None => {}
        }
    }
}

/// Whether a name is a React hook's: `use` and then a capital letter (fallow's `is_hook_callee`).
fn hook(name: &str) -> bool {
    name.strip_prefix("use")
        .is_some_and(|rest| rest.starts_with(|next: char| next.is_ascii_uppercase()))
}
>>>>>>> replace
<<<<<<< find
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        self.enter(object.node_id.get(), object.span, None, None);
        walk::walk_object_expression(self, object);
        self.leave(false);
    }
}
=======
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        self.enter(object.node_id.get(), object.span, None, None);
        walk::walk_object_expression(self, object);
        self.leave(false);
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
        self.if_chain(statement, false);
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.looped();

        if let Some(init) = &statement.init {
            self.visit_for_statement_init(init);
        }

        if let Some(test) = &statement.test {
            self.visit_expression(test);
        }

        if let Some(update) = &statement.update {
            self.visit_expression(update);
        }

        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        self.looped();
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.looped();
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_while_statement(&mut self, statement: &WhileStatement<'a>) {
        self.looped();
        self.visit_expression(&statement.test);
        self.nested(|scorer| scorer.visit_statement(&statement.body));
    }

    fn visit_do_while_statement(&mut self, statement: &DoWhileStatement<'a>) {
        self.looped();
        self.nested(|scorer| scorer.visit_statement(&statement.body));
        self.visit_expression(&statement.test);
    }

    // A `switch` adds one and its nesting once, and each `case` with a test a path, an empty one
    // included (fallow's `visit_switch_statement` and `visit_switch_case`).
    fn visit_switch_statement(&mut self, statement: &SwitchStatement<'a>) {
        self.structural();
        self.visit_expression(&statement.discriminant);
        self.nested(|scorer| scorer.visit_switch_cases(&statement.cases));
    }

    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        if case.test.is_some() {
            self.branch();
        }

        walk::walk_switch_case(self, case);
    }

    // A `try` and a `finally` add nothing and nest nothing; a `catch` adds a path, and one and its
    // nesting.
    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.branch();
        self.structural();
        self.nested(|scorer| walk::walk_catch_clause(scorer, clause));
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        self.branch();
        self.structural();
        self.visit_expression(&conditional.test);
        self.nested(|scorer| {
            scorer.visit_expression(&conditional.consequent);
            scorer.visit_expression(&conditional.alternate);
        });
    }

    // Each `&&`, `||` and `??` adds a path. One adds to the cognitive score when its operator is not
    // the last one counted, and the last one is forgotten after an operator neither of whose sides is
    // another, parentheses not seen through (fallow's `visit_logical_expression`; spec SCORE-2).
    fn visit_logical_expression(&mut self, logical: &LogicalExpression<'a>) {
        self.branch();

        if let Some(frame) = self.frame()
            && frame.last != Some(logical.operator)
        {
            frame.cognitive += 1;
            frame.last = Some(logical.operator);
        }

        walk::walk_logical_expression(self, logical);

        let is_logical = |side: &Expression<'_>| matches!(side, Expression::LogicalExpression(_));

        if !is_logical(&logical.left)
            && !is_logical(&logical.right)
            && let Some(frame) = self.frame()
        {
            frame.last = None;
        }
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if assignment.operator.is_logical() {
            self.branch();
        }

        walk::walk_assignment_expression(self, assignment);
    }

    // A chain adds a path where its outermost link is optional, and its other links add nothing
    // (fallow's `visit_chain_expression`; spec SCORE-2).
    fn visit_chain_expression(&mut self, chained: &ChainExpression<'a>) {
        let optional = match &chained.expression {
            ChainElement::CallExpression(call) => call.optional,
            ChainElement::TSNonNullExpression(_) => false,
            ChainElement::ComputedMemberExpression(member) => member.optional,
            ChainElement::StaticMemberExpression(member) => member.optional,
            ChainElement::PrivateFieldExpression(member) => member.optional,
        };

        if optional {
            self.branch();
        }

        walk::walk_chain_expression(self, chained);
    }

    fn visit_break_statement(&mut self, statement: &BreakStatement<'a>) {
        if statement.label.is_some() {
            self.flat();
        }

        walk::walk_break_statement(self, statement);
    }

    fn visit_continue_statement(&mut self, statement: &ContinueStatement<'a>) {
        if statement.label.is_some() {
            self.flat();
        }

        walk::walk_continue_statement(self, statement);
    }

    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if let Some(frame) = self.frame() {
            frame.jsx = true;
        }

        walk::walk_jsx_element(self, element);
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        if let Some(frame) = self.frame() {
            frame.jsx = true;
        }

        walk::walk_jsx_fragment(self, fragment);
    }

    // A hook's call adds one to a component's or a hook's cognitive score (fallow's `record_hook`;
    // spec SCORE-2).
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && hook(callee.name.as_str())
            && let Some(frame) = self.frame()
            && frame.react
        {
            frame.cognitive += 1;
        }

        walk::walk_call_expression(self, call);
    }
}
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --locked
cargo clippy --locked --all-targets
```

Expected: every test of every file passes; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
git add -A && git commit -m "feat(complexity): score functions as fallow counts them"
```

Expected: one commit.

### Task 3: The guide

**Implements:** VER-1, VER-2

**Test:** `node --test scripts/contributing.test.mjs`

**Files:**
- Modify: `CONTRIBUTING.md`, `src/declarations.rs`, `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: Task 1's command.
- Produces: the guide's description of `complexity`, and of what interface version 1 covers: the
  sixth rule, "a change that breaks", and the name-path rules as a draft until the swap. Plan 2
  adds the property bag and `audit`'s six kinds.

- [ ] **Step 1: Write the failing test**

```js edit path=scripts/contributing.test.mjs
<<<<<<< find
    '--hand-off <file> --base <commit>`',
=======
    '--hand-off <file> --base <commit>`',
    '`yak-analyzer complexity` reads the project as `audit` does',
    '`yak-analyzer complexity --hand-off <file> --all`',
>>>>>>> replace
<<<<<<< find
test('the guide says what interface version 1 covers, and that a change to it raises the number (comparison spec FREEZE-1)', () => {
=======
test('the guide says what interface version 1 covers, and that a change that breaks it raises the number (comparison spec FREEZE-1; complexity spec VER-1)', () => {
>>>>>>> replace
<<<<<<< find
    '`unused-file`, `unused-export`, `unused-type`,\n`unresolved-import` and `circular-dependency`',
    '`yakAnalyzer/v1`',
    'A change to any of them raises\nthe interface version',
=======
    '`unused-file`, `unused-export`, `unused-type`, `unresolved-import`,\n`circular-dependency` and `complexity`',
    '`yakAnalyzer/v1`',
    'A change that breaks any of them\nraises the interface version',
    'A change that only adds, such as a new rule name, does not raise it.',
    'a draft until the swap',
>>>>>>> replace
```

- [ ] **Step 2: Run the test to see it fail**

```text
node --test scripts/contributing.test.mjs
```

Expected: the commands test and the interface-version test fail on the sentences the guide does not
hold yet.

- [ ] **Step 3: Write the guide**

````markdown edit path=CONTRIBUTING.md
<<<<<<< find
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.

Interface version 1 covers what yak relies on, fixed when the analyzer's stage 2 ended on
2026-11-17: the hand-off format the analyzer reads; the declaration keys and their shapes,
`entries`, `data`, `generated`, `internal`, `cyclomaticLimit`, `cognitiveLimit`,
`duplicationMinimum`, `configEntries`, `scriptCommands` and `commandFiles`; the rule names `audit`
reports, `unused-file`, `unused-export`, `unused-type`,
`unresolved-import` and `circular-dependency`; the fingerprint scheme it marks each finding with,
`yakAnalyzer/v1`; `audit`'s options; and what `interface` prints. A change to any of them raises
the interface version, which `interface` prints and yak checks before it runs the analyzer.
=======
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.
- `yak-analyzer complexity` reads the project as `audit` does and prints one JSON document: the
  limits the declarations give `cyclomaticLimit` and `cognitiveLimit`, each function over a limit
  with its scores, the measures it is over and the fingerprint of its finding, and the notices.
  `yak-analyzer complexity --hand-off <file> --all` reads with the hand-off file's declarations
  and lists every function. A command line it refuses exits 2. yak never runs it: it is for a
  person or an agent, and its name, options and document are a draft until the analyzer's stage 5
  releases it as an MCP tool.

Interface version 1 covers what yak relies on, fixed when the analyzer's stage 2 ended on
2026-11-17: the hand-off format the analyzer reads; the declaration keys and their shapes,
`entries`, `data`, `generated`, `internal`, `cyclomaticLimit`, `cognitiveLimit`,
`duplicationMinimum`, `configEntries`, `scriptCommands` and `commandFiles`; the rule names `audit`
reports, `unused-file`, `unused-export`, `unused-type`, `unresolved-import`,
`circular-dependency` and `complexity`; the fingerprint scheme it marks each finding with,
`yakAnalyzer/v1`; `audit`'s options; and what `interface` prints. A change that breaks any of them
raises the interface version, which `interface` prints and yak checks before it runs the analyzer.
A change that only adds, such as a new rule name, does not raise it.

The rules that name a function in a `complexity` finding's fingerprint, its name path, are part of
the scheme and a draft until the swap, the pack release that replaces fallow with the analyzer in
yak: nothing yak runs reads them before then, so changing them raises nothing. From the swap they
are fixed, and a change that breaks them raises the interface version.
>>>>>>> replace
````

```rust edit path=src/declarations.rs
<<<<<<< find
//! The keys and their shapes are fixed under interface version 1 since stage 2 ended (comparison spec
//! FREEZE-1): a change to any raises the version. This part checks them, the shapes of the three that
=======
//! The keys and their shapes are fixed under interface version 1 since stage 2 ended (comparison spec
//! FREEZE-1): a change that breaks any raises the version, and one that only adds does not
//! (complexity spec VER-1). This part checks them, the shapes of the three that
>>>>>>> replace
```

- [ ] **Step 4: Run the test to see it pass**

```text
node --test scripts/contributing.test.mjs
```

Expected: every test passes.

- [ ] **Step 5: Commit**

```text
yak stack format
git add -A && git commit -m "docs: describe the complexity command and what only adds"
```

Expected: one commit.
