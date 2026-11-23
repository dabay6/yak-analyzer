# Duplication, part 1: copies, their groups and the duplication command: implementation plan

**Goal:** the analyzer reads the statement lists of every script and Vue component in the parse that
finds their imports, finds the runs of whole statements that repeat, groups them as the declarations
say, and a new command, `yak-analyzer duplication`, prints the groups with their sizes, copies and
fingerprints, leaving out the groups a person accepted unless `--all` is given.

**Architecture:** a new `src/forms.rs` walks the tree the import reading already parsed and writes
each statement's form, the node names and values that tell two statements apart, with what the
declarations loosen as placeholders; it keeps a digest of each form, the statement's size counted on
the parser's tokens, and its place, as one `List` per statement list on the file's `Read`. A new
`src/duplication.rs` reads the settings from the hand-off file, lays every list end to end as one
sequence of symbols, finds the runs that repeat with a suffix array, and makes the groups. `audit`
will use the same `groups` in plan 2 (SHOW-5).

**Spec:** docs/specs/2026-11-23-duplication-findings-design.md

**Implements:** COPY-1 to COPY-5, SAME-1 to SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7, DUP-4, DUP-5, QUIET-1 to QUIET-4, SHOW-1 to SHOW-5, VERS-1, FIXT-1, FIXT-2

The first of the item's three plans, on the item's branch, `feat/duplication-findings`, which holds
the approved spec. The plans, in order:

1. **This plan:** the declarations, the statements and their forms, the groups, accepted groups, and
   the `duplication` command.
2. **The finding in `audit`:** the `duplication` result with its message, locations and property bag
   (DUP-1 to DUP-3), new against inherited (FRESH-1 to FRESH-4) with each statement's exact form
   added to the reading, its fixtures, and the rest of VERS-1 (the property bag, and `audit`
   reporting seven kinds). It is written once this plan has merged, from `main`.
3. **The comparison with fallow:** YARD-1 to YARD-7, and OWED-1's list filed in yakshaver.

What waits for plan 2: `audit` reports no duplication finding yet. This plan adds `duplication` to
the run's list of rules, since a group's fingerprint is made under that rule's name (DUP-4), so the
report lists seven rules and holds no duplication result.

## Global constraints

- A copy is a run of whole statements of one statement list; it never starts or ends inside a
  statement (COPY-1).
- Imports and `export … from` never join a run (COPY-2); a statement holding a test callee's call
  never joins one, and a test callee's callbacks are not looked at (COPY-3).
- Runs are read in the one parse that finds imports; `graph` prints none of it (COPY-5).
- No minimum is built in: with no `duplicationMinimum` nothing is looked for, and a notice says so
  (TUNE-1). `duplicationCopies` is 2 when undeclared (TUNE-3).
- A finding carries no code (the spec's anti-goals).
- The fingerprint keeps the scheme `yakAnalyzer/v1`: the first 32 hexadecimal digits of a SHA-256
  digest of length-prefixed parts, with no path and no line (DUP-4).
- Fingerprints are the same on every machine for the same code: built from the tree's values, line
  endings written as line feeds (DUP-5).
- Nothing that `graph` prints changes, and no other kind's finding or fingerprint changes.
- The interface version stays 1 (VERS-1).
- No fixture copies code from fallow or from the named repositories (FIXT-2).

## The decisions this plan makes

1. **A statement's form** (SAME-1, SAME-2, DUP-4, DUP-5). The walk writes, for each node it enters,
   the name of its kind as oxc 0.158 calls it (`format!("{:?}", kind.ty())`, as `BinaryExpression`),
   then the values that make two nodes of one kind differ, then a close when it leaves the node. The
   values are every field oxc's own `ContentEq` compares that is not a child node: names, strings,
   numbers, operators and flags (the full list is `values` in Task 2), plus which optional child is
   present for a `for` statement, a type parameter and a mapped type, whose optional children are
   of one type. Names, strings and numbers are each written as a placeholder byte where
   `duplicationIgnore` loosens them. A `ParenthesizedExpression` and a `TSParenthesizedType` write
   nothing, so `(a + b)` alone is `a + b`; a quoted property key is written as the name it spells; a
   JSX text is written as it renders, and one that holds only white space writes nothing; a
   template piece is its cooked value, or its raw text where it has none, with its line endings line
   feeds. Each piece is a tag byte, then for one with content its length as four bytes and its
   bytes, so no two forms run together. A statement keeps the first 16 bytes of the SHA-256 digest
   of its form. A group's fingerprint is `findings::fingerprint(Rule::Duplication, parts)`, each
   part a statement of the run's digest in hexadecimal. The kind names come from oxc: an oxc release
   that renames a node changes the fingerprints of runs holding it, so the release that moves oxc's
   pin says so until the swap fixes the form.
2. **A statement's size** (SIZE-1). Its tokens are the parser's own (oxc's `TokensParserConfig`,
   through `RuntimeParserConfig::new(true)` when a minimum is declared) whose start lies in the
   statement, less those the fixed form leaves out: every `;`; a `,` just before `)`, `]` or `}`;
   the parentheses of a `ParenthesizedExpression` or a `TSParenthesizedType`, needed or not; the
   parentheses around an arrow function's lone, plain parameter; the empty `()` after `new`; and a
   JSX text that holds only white space. So the fixed form writes no semicolons and no parentheses
   around expressions. A token is on the line it starts on; a statement's lines are the lines its
   tokens start on, and a run's lines are its statements' together, a line two of them share
   counted once.
3. **Statement lists** (COPY-1). Each list the walk meets through `visit_statements`, a file's top
   level, a block's, a function's, a `switch` case's, a static block's and a namespace's body, and
   each class body through `visit_class_elements`. A function's directives are not statements. Lists
   are read only when `duplicationMinimum` is declared, and only for scripts and Vue blocks: a type
   declaration file, an MDX document and an HTML page are parsed without them, so a project that
   declares no minimum pays nothing.
4. **Finding the runs** (SAME-3, SAME-4). Every list of every file is laid end to end, each statement
   that joins runs as one number per form, each that joins none and each list's end as a number of
   its own, so no run crosses a list or one of them. A suffix array of that sequence, by prefix
   doubling, with Kasai's longest common prefixes and the bottom-up walk of its intervals, gives each
   run that repeats with the places it starts at. A run whose copies are all preceded by the same
   statement is passed over, since the longer run holds it. Its places are taken in order of file,
   list and statement, and one that overlaps the copy before it in its list is dropped. A group is
   then kept where its size and copies meet the minimums, and dropped where every copy lies in a
   different copy of another kept group with at least as many copies; of two that each lie in the
   other, the first found stays. Groups are sorted by their first copy's file and lines, then by
   their number of copies; copies by file and first line.
5. **The settings.** `src/duplication.rs` reads them, with the complexity limits' `declared` (made
   public) for each number. `Reading` gains `duplication: Settings`, and `scripts::read` takes the
   `Lens` the parse needs, so the base is read with the same settings (TUNE-7; design BASE-3).
6. **Test callees** (COPY-3, TUNE-5). A call's callee chain is written by the function complexity
   names callbacks with, `functions::chain`, given a switch that also writes a tagged template as
   two backticks. Complexity calls it with the switch off, so no function's name changes.
7. **Accepted groups** (QUIET-1, QUIET-2). Acceptance is applied after SAME-4's dropping, so an
   accepted group still hides the groups inside it. An entry is a group's fingerprint, `/` and its
   copies; the notice for an entry no group matches names the entry and whether the pack or the
   project states it, once for each.
8. **The test files** are named `tests/duplication.test.rs` with a `[[test]]` entry, as every test
   file of this repository is (CONTRIBUTING.md, "Fixtures"), though the pack's `rust-test-file-names`
   rule asks for no dot: the repository's own convention wins here, and renaming every test file is
   not this item's.

## Review focus

- **A value the form leaves out.** Two statements that differ only in a value `values` does not write
  would be one form, and a false group. The list was taken from oxc 0.158's `derive_content_eq.rs`;
  Task 2's tests pin an operator (`3 + 4` against `3 - 4`). Check `values` against that file.
- **The parser's tokens.** The count relies on oxc's collected tokens being the re-lexed ones for
  regular expressions, templates and JSX. Task 2's tests count a run with JSX, a template, an arrow,
  `new`, quoted keys and trailing commas; a group's size is its smallest copy's, so a count too high
  in one copy only can hide. Check `Counted::of` against `oxc_parser`'s `Kind` names.
- **Size of a large repository.** The suffix array is O(n log² n) in the statements; `widest`
  compares each kept group with each other one. Plan 3 measures `audit --base` on the largest named
  root (YARD-6).
- **Optional children of one type.** Only a `for` statement, a type parameter and a mapped type write
  which optional child is present; another node with two optional children of one type would read
  as one form either way. None was found in oxc 0.158's AST beyond those three.

### Task 1: The declaration keys

**Implements:** TUNE-1, TUNE-2, TUNE-3, TUNE-4, TUNE-5, TUNE-6

**Test:** `cargo test --locked --test declarations`

**Files:**
- Modify: `src/declarations.rs`, `src/findings.rs`, `tests/declarations.test.rs`

**Interfaces:**
- Consumes: `findings::FINGERPRINT_DIGITS`, made public.
- Produces: the five new keys in `KEYS`, each checked by its kind: `duplicationMinimumLines` a
  positive whole number, `duplicationCopies` a whole number of 2 or more, `duplicationIgnore` a list
  of the three words, `duplicationTestCallees` a list of callee chains, `duplicationAccepted` a list of
  `<fingerprint>/<copies>`.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/declarations.test.rs
<<<<<<< find
        "duplicationMinimum": 50,
        "configEntries": [
=======
        "duplicationMinimum": 50,
        "duplicationMinimumLines": 5,
        "duplicationCopies": 2,
        "duplicationIgnore": ["strings"],
        "duplicationTestCallees": ["it", "it.each()", "test.each``", "describe.only", "suite[]"],
        "duplicationAccepted": ["0123456789abcdef0123456789abcdef/2"],
        "configEntries": [
>>>>>>> replace
<<<<<<< find
        "cyclomaticLimit": 25
    });
=======
        "cyclomaticLimit": 25,
        "duplicationIgnore": [],
        "duplicationTestCallees": ["spec"],
        "duplicationAccepted": ["fedcba9876543210fedcba9876543210/12"]
    });
>>>>>>> replace
<<<<<<< find
         the analyzer knows no such key, only entries, data, generated, internal, cyclomaticLimit, \
         cognitiveLimit, duplicationMinimum, configEntries, scriptCommands, commandFiles; change it in the \
         pack"
=======
         the analyzer knows no such key, only entries, data, generated, internal, cyclomaticLimit, \
         cognitiveLimit, duplicationMinimum, duplicationMinimumLines, duplicationCopies, duplicationIgnore, \
         duplicationTestCallees, duplicationAccepted, configEntries, scriptCommands, commandFiles; change it \
         in the pack"
>>>>>>> replace
<<<<<<< find
#[test]
fn refuses_a_configuration_or_script_declaration_of_another_shape() {
=======
#[test]
fn refuses_a_duplication_declaration_of_another_shape() {
    let cases: [(Value, &str); 10] = [
        (
            json!({ "duplicationMinimumLines": 0 }),
            ": its value must be a positive whole number; ",
        ),
        (
            json!({ "duplicationCopies": 1 }),
            ": its value must be a whole number of 2 or more",
        ),
        (
            json!({ "duplicationIgnore": "strings" }),
            ": its value must be a list of the words `names`, `strings` and `numbers`; ",
        ),
        (
            json!({ "duplicationIgnore": ["strings", "comments"] }),
            ": `comments` is not one of the words it takes, `names`, `strings` and `numbers`; ",
        ),
        (
            json!({ "duplicationTestCallees": ["it", "it each"] }),
            ": `it each` is not a callee chain, such as `it`, `it.only` or `it.each()`; ",
        ),
        (
            json!({ "duplicationTestCallees": ["it."] }),
            ": `it.` is not a callee chain",
        ),
        (
            json!({ "duplicationTestCallees": ["1it"] }),
            ": `1it` is not a callee chain",
        ),
        (
            json!({ "duplicationAccepted": ["0123456789abcdef0123456789abcdef"] }),
            ": `0123456789abcdef0123456789abcdef` is not a group's fingerprint, its 32 lower-case \
             hexadecimal digits, then `/` and its number of copies, 2 or more; ",
        ),
        (
            json!({ "duplicationAccepted": ["0123456789ABCDEF0123456789abcdef/2"] }),
            "is not a group's fingerprint",
        ),
        (
            json!({ "duplicationAccepted": ["0123456789abcdef0123456789abcdef/1"] }),
            "is not a group's fingerprint",
        ),
    ];

    for (index, (pack, expected)) in cases.into_iter().enumerate() {
        let line = refused(
            &format!("declarations-duplication-{index}"),
            pack,
            json!({}),
        );

        assert!(line.contains(expected), "{line}");
    }

    // A project may state each of them, and is refused the same way (duplication spec TUNE-4).
    let line = refused(
        "declarations-duplication-project",
        json!({}),
        json!({ "duplicationIgnore": ["names", "numbers", "names", "text"] }),
    );

    assert!(
        line.contains("`duplicationIgnore` that the project states")
            && line.contains("`text` is not one of the words it takes"),
        "{line}"
    );
}

#[test]
fn refuses_a_configuration_or_script_declaration_of_another_shape() {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test declarations
```

Expected: `uses_every_key_it_knows_in_every_form_a_glob_may_take`,
`names_the_stack_the_key_and_who_states_it` and `refuses_a_duplication_declaration_of_another_shape`
fail: the analyzer knows no such key.

- [ ] **Step 3: Write the code**

```rust edit path=src/findings.rs
<<<<<<< find
/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
const FINGERPRINT_DIGITS: usize = 32;
=======
/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
pub const FINGERPRINT_DIGITS: usize = 32;
>>>>>>> replace
```

```rust edit path=src/declarations.rs
<<<<<<< find
//! The keys and their shapes are fixed under interface version 1 since stage 2 ended (comparison spec
//! FREEZE-1): a change that breaks any raises the version, and one that only adds does not
//! (complexity spec VER-1). This part checks them, the shapes of the three that
//! describe what a tool does included (spec ENTRY-6a; comparison spec CI-1); the parts that read a
//! project use them.
=======
//! The keys and their shapes are fixed under interface version 1 since stage 2 ended (comparison spec
//! FREEZE-1): a change that breaks any raises the version, and one that only adds does not
//! (complexity spec VER-1), as the keys that tune duplication only add (duplication spec VERS-1). This
//! part checks them, the shapes of the three that describe what a tool does included (spec ENTRY-6a;
//! comparison spec CI-1); the parts that read a project use them.
>>>>>>> replace
<<<<<<< find
use crate::glob::Glob;
=======
use crate::findings::FINGERPRINT_DIGITS;
use crate::glob::Glob;
>>>>>>> replace
<<<<<<< find
    /// A positive whole number.
    Number,
=======
    /// A positive whole number.
    Number,
    /// A whole number of 2 or more: the copies a group needs (duplication spec TUNE-3).
    Copies,
    /// A list of the words `duplicationIgnore` takes (duplication spec TUNE-4).
    Words,
    /// A list of callee chains (duplication spec TUNE-5).
    Chains,
    /// A list of accepted groups, each a fingerprint, `/` and a number of copies (duplication spec
    /// TUNE-6).
    Accepted,
>>>>>>> replace
<<<<<<< find
/// Every key the analyzer knows, with the kind of its value (spec KEY-1).
const KEYS: [(&str, Kind); 10] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("internal", Kind::Paths),
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
=======
/// The words `duplicationIgnore` takes, in the order the `duplication` command prints them (duplication
/// spec TUNE-4, SHOW-2).
pub const WORDS: [&str; 3] = ["names", "strings", "numbers"];

/// Every key the analyzer knows, with the kind of its value (spec KEY-1; duplication spec TUNE-1 to
/// TUNE-6).
const KEYS: [(&str, Kind); 15] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("internal", Kind::Paths),
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("duplicationMinimumLines", Kind::Number),
    ("duplicationCopies", Kind::Copies),
    ("duplicationIgnore", Kind::Words),
    ("duplicationTestCallees", Kind::Chains),
    ("duplicationAccepted", Kind::Accepted),
>>>>>>> replace
<<<<<<< find
        Kind::Number => match value.as_u64() {
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
=======
        Kind::Number => match value.as_u64() {
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
        Kind::Copies => match value.as_u64() {
            Some(number) if number >= 2 => None,
            _ => Some(
                "its value must be a whole number of 2 or more, since a group is two copies at the least"
                    .to_owned(),
            ),
        },
        Kind::Words => match texts(value) {
            Some(words) => words.iter().find(|word| !WORDS.contains(*word)).map(|word| {
                format!(
                    "`{word}` is not one of the words it takes, `names`, `strings` and `numbers`"
                )
            }),
            None => Some(
                "its value must be a list of the words `names`, `strings` and `numbers`".to_owned(),
            ),
        },
        Kind::Chains => match texts(value) {
            Some(chains) => chains.iter().find(|chain| !is_chain(chain)).map(|chain| {
                format!("`{chain}` is not a callee chain, such as `it`, `it.only` or `it.each()`")
            }),
            None => Some("its value must be a list of callee chains".to_owned()),
        },
        Kind::Accepted => match texts(value) {
            Some(entries) => entries
                .iter()
                .find(|entry| !is_accepted(entry))
                .map(|entry| {
                    format!(
                        "`{entry}` is not a group's fingerprint, its {FINGERPRINT_DIGITS} lower-case \
                         hexadecimal digits, then `/` and its number of copies, 2 or more"
                    )
                }),
            None => Some("its value must be a list of accepted groups".to_owned()),
        },
>>>>>>> replace
<<<<<<< find
/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
}
=======
/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
}

/// Whether a text is a callee chain as the duplication spec's terms write one: a name, then any number
/// of `.` and a name (`#` and a name for a private one), `()` for a call, `[]` for a computed member,
/// and two backticks for a tagged template (duplication spec TUNE-5).
fn is_chain(text: &str) -> bool {
    let Some(mut rest) = after_name(text) else {
        return false;
    };

    while !rest.is_empty() {
        rest = if let Some(after) = ["()", "[]", "``"]
            .iter()
            .find_map(|piece| rest.strip_prefix(piece))
        {
            after
        } else if let Some(after) = rest.strip_prefix('.') {
            match after_name(after.strip_prefix('#').unwrap_or(after)) {
                Some(after) => after,
                None => return false,
            }
        } else {
            return false;
        };
    }

    true
}

/// What follows the name a text starts with: one or more letters, digits, `_` or `$`, not starting with
/// a digit; nothing where the text does not start with a name.
fn after_name(text: &str) -> Option<&str> {
    let end = text
        .find(|letter: char| !(letter.is_alphanumeric() || letter == '_' || letter == '$'))
        .unwrap_or(text.len());

    (end > 0 && !text.starts_with(|letter: char| letter.is_ascii_digit())).then(|| &text[end..])
}

/// Whether a text is an accepted group: its fingerprint in lower-case hexadecimal digits, `/`, and its
/// number of copies, 2 or more, written with no sign and no leading zero (duplication spec TUNE-6).
fn is_accepted(text: &str) -> bool {
    text.split_once('/').is_some_and(|(fingerprint, copies)| {
        fingerprint.len() == FINGERPRINT_DIGITS
            && fingerprint
                .bytes()
                .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
            && !copies.is_empty()
            && !copies.starts_with('0')
            && copies.bytes().all(|digit| digit.is_ascii_digit())
            && copies.parse::<u64>().is_ok_and(|copies| copies >= 2)
    })
}
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --locked --test declarations
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
```

Expected: every test passes; rustfmt changes nothing; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
yak stack check
git add src/declarations.rs src/findings.rs tests/declarations.test.rs
git commit -m "feat(declarations): add the keys that tune duplication"
```

Expected: one commit.

### Task 2: The statements, their groups, and the duplication command

**Implements:** COPY-1, COPY-2, COPY-4, COPY-5, SAME-1, SAME-2, SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1, TUNE-2, TUNE-3, TUNE-4, TUNE-7, DUP-4, DUP-5, SHOW-1, SHOW-2, SHOW-3, SHOW-4, SHOW-5, FIXT-1, FIXT-2

**Test:** `cargo test --locked --test duplication --test cli --test audit`

**Files:**
- Create: `src/forms.rs`, `src/duplication.rs`, `tests/duplication.test.rs`
- Modify: `src/imports.rs`, `src/pages.rs`, `src/scripts.rs`, `src/reading.rs`, `src/graph.rs`,
  `src/complexity.rs`, `src/findings.rs`, `src/cli.rs`, `src/main.rs`, `Cargo.toml`,
  `tests/cli.test.rs`, `tests/audit.test.rs`

**Interfaces:**
- Consumes: Task 1's keys; `complexity::declared`, made public; `imports::Lines::at`, made public;
  `findings::fingerprint`.
- Produces: `forms::{List, Item, Held, lists, append}`; `duplication::{Lens, Settings, Group, Place,
  groups, run}`; `Read::lists`; `Reading::duplication`; `Rule::Duplication`; `Command::Duplication`.

- [ ] **Step 1: Write the failing tests**

```toml edit path=Cargo.toml
<<<<<<< find
[[test]]
name = "complexity"
path = "tests/complexity.test.rs"
=======
[[test]]
name = "complexity"
path = "tests/complexity.test.rs"

[[test]]
name = "duplication"
path = "tests/duplication.test.rs"
>>>>>>> replace
```

```rust edit path=tests/audit.test.rs
<<<<<<< find
            "circular-dependency",
            "complexity",
        ]
    );
=======
            "circular-dependency",
            "complexity",
            "duplication",
        ]
    );
>>>>>>> replace
```

```rust edit path=tests/cli.test.rs
<<<<<<< find
        assert!(
            error.contains("and complexity [--hand-off <file>] [--all]"),
            "{args:?}"
        );
=======
        assert!(
            error.contains(
                "complexity [--hand-off <file>] [--all], and duplication [--hand-off <file>] [--all]"
            ),
            "{args:?}"
        );
>>>>>>> replace
<<<<<<< find
    assert!(error.contains("and complexity [--hand-off <file>] [--all]"));
=======
    assert!(error.contains(
        "complexity [--hand-off <file>] [--all], and duplication [--hand-off <file>] [--all]"
    ));
>>>>>>> replace
```

```rust path=tests/duplication.test.rs
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
        (json!({ "duplicationMinimum": 5, "duplicationCopies": 2 }), 1),
        (json!({ "duplicationMinimum": 5, "duplicationCopies": 3 }), 0),
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

    assert_eq!(groups(&lf), [vec![("src/a.tsx", 2, 6), ("src/a.tsx", 9, 13)]]);
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
    let body = "  let sum = 0;\n  for (const item of items) {\n    sum += item;\n  }\n  return sum;\n";

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
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test duplication --test cli --test audit
```

Expected: every `duplication` test fails (the binary refuses `duplication`), the two refusal tests of
`cli` fail (the accepted list names no `duplication`), and `audit`'s rule list lacks `duplication`.

- [ ] **Step 3: Write the code**

```rust path=src/forms.rs
//! The statements duplication compares, read in the parse that finds a script's imports: each statement
//! list, and for each of its statements the digest of its form, its size, its place, and whether it can
//! join a run (duplication spec COPY-1, COPY-2, COPY-5, SAME-1, SAME-2, SIZE-1, DUP-5).
//!
//! A statement's form is written as the walk meets its syntax tree: the name of each node, and each
//! value that makes two nodes of one kind differ. Formatting, comments and quotes are not in the tree,
//! and parentheses that change nothing are passed over, so they leave the form as it is. What the lens
//! loosens is written as a placeholder.

use std::collections::BTreeSet;
use std::fmt;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, ClassElement, Program, PropertyKey, Statement,
    TemplateElement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::{Kind, Token};
use oxc_span::{GetSpan, Span};
use sha2::{Digest, Sha256};

use crate::duplication::Lens;
use crate::imports::Lines;

/// How many bytes of the SHA-256 digest of a statement's form it keeps.
pub const DIGEST: usize = 16;

/// What ends a node in a form.
const CLOSE: u8 = b')';

/// One statement list: a file's top level, a block's body, a `switch` case's statements or a class's
/// body (spec COPY-1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct List {
    /// Its statements, in source order.
    pub items: Vec<Item>,
}

/// The lines of a statement that a token its size counts starts on (spec SIZE-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    /// The first such line, counted from one.
    pub first: usize,
    /// The last such line.
    pub last: usize,
    /// How many such lines there are.
    pub count: u64,
}

/// One statement of a list, or one member of a class's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// The first bytes of the SHA-256 digest of its form, with what the lens loosens as placeholders
    /// (spec SAME-1, SAME-2).
    pub digest: [u8; DIGEST],
    /// Whether it can join a run: an import cannot, nor a statement that holds a test callee's call
    /// (spec COPY-2, COPY-3).
    pub joins: bool,
    /// How many tokens it holds, as its tree written out in the fixed form counts them (spec SIZE-1).
    pub tokens: u64,
    /// The lines those tokens start on; none where it holds none.
    pub lines: Option<Held>,
    /// Its first byte in the file.
    pub start: u32,
    /// The byte after its last.
    pub end: u32,
    /// The line it starts on, counted from one.
    pub line: usize,
    /// The line it ends on.
    pub end_line: usize,
}

/// The statement lists of a parsed program, in the order the walk meets them, each statement read
/// through the lens, with its size counted on the program's tokens (spec COPY-5).
pub fn lists(
    program: &Program<'_>,
    tokens: &[Token],
    text: &str,
    lines: &Lines<'_>,
    lens: &Lens,
) -> Vec<List> {
    let mut former = Former {
        lens,
        stream: Vec::new(),
        lists: Vec::new(),
        open: Vec::new(),
        dropped: Dropped::default(),
    };

    former.visit_program(program);

    let counted = Counted::of(tokens, text, lines, &former.dropped);

    former
        .lists
        .iter()
        .map(|raws| List {
            items: raws
                .iter()
                .map(|raw| counted.item(raw, &former.stream, lines))
                .collect(),
        })
        .collect()
}

/// A Vue component's block's lists added to the component's, with their lines and bytes counted in the
/// file: the lines above the block and the bytes before it are added.
pub fn append(lists: &mut Vec<List>, block: Vec<List>, above: usize, offset: u32) {
    lists.extend(block.into_iter().map(|list| List {
        items: list
            .items
            .into_iter()
            .map(|item| Item {
                lines: item.lines.map(|held| Held {
                    first: held.first + above,
                    last: held.last + above,
                    ..held
                }),
                start: item.start + offset,
                end: item.end + offset,
                line: item.line + above,
                end_line: item.end_line + above,
                ..item
            })
            .collect(),
    }));
}

/// A JSX text as it renders, which is what is compared: each line trimmed of the white space at its
/// ends, the lines that hold nothing else left out, and the rest joined by one space (spec SAME-1,
/// DUP-5).
fn rendered(text: &str) -> String {
    text.split(['\n', '\r'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A template piece's value: its cooked value, or its raw text where an invalid escape in a tagged
/// template leaves it none, with every line ending a line feed (spec SAME-1, DUP-5).
fn template(element: &TemplateElement<'_>) -> String {
    element
        .value
        .cooked
        .map_or_else(
            || element.value.raw.as_str().to_owned(),
            |cooked| cooked.display().to_string(),
        )
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

/// The tokens the fixed form leaves out that only the tree tells apart (spec SIZE-1).
#[derive(Default)]
struct Dropped {
    /// Where an opening parenthesis around an expression or a type starts.
    opens: BTreeSet<u32>,
    /// Where the closing one ends.
    closes: BTreeSet<u32>,
    /// Where a `new` with no arguments ends, at its empty `()` where it has one.
    empty: BTreeSet<u32>,
    /// Where an arrow function's lone, plain parameter starts, which parentheses may wrap.
    lone: BTreeSet<u32>,
}

/// One statement as the walk found it.
struct Raw {
    /// Where its form starts in the stream.
    from: usize,
    /// Where it ends.
    to: usize,
    /// Where the statement is in the text.
    span: Span,
    /// Whether it can join a run.
    joins: bool,
}

/// Walks a program, writing the form of everything in it to one stream and noting where each
/// statement's form is.
struct Former<'l> {
    /// What to loosen.
    lens: &'l Lens,
    /// The forms, end to end.
    stream: Vec<u8>,
    /// Each list's statements, in the order the walk met the lists.
    lists: Vec<Vec<Raw>>,
    /// The statements the walk is inside, by their list and their place in it.
    open: Vec<(usize, usize)>,
    /// The tokens the size leaves out.
    dropped: Dropped,
}

impl Former<'_> {
    /// Walks one statement of a list, noting where its form starts and ends in the stream.
    fn item(&mut self, list: usize, span: Span, joins: bool, walk: impl FnOnce(&mut Self)) {
        let index = self.lists[list].len();

        self.lists[list].push(Raw {
            from: self.stream.len(),
            to: self.stream.len(),
            span,
            joins,
        });
        self.open.push((list, index));
        walk(self);
        self.open.pop();
        self.lists[list][index].to = self.stream.len();
    }

    /// Writes a piece of a form: its tag, then its bytes after their length.
    fn put(&mut self, tag: u8, bytes: &[u8]) {
        self.stream.push(tag);
        self.stream
            .extend_from_slice(&u32::try_from(bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
        self.stream.extend_from_slice(bytes);
    }

    /// Writes a name, or its placeholder where names are loosened (spec SAME-2).
    fn name(&mut self, name: &str) {
        if self.lens.names {
            self.stream.push(b'n');
        } else {
            self.put(b'N', name.as_bytes());
        }
    }

    /// Writes a string's value, or its placeholder where strings are loosened (spec SAME-2).
    fn string(&mut self, value: &str) {
        if self.lens.strings {
            self.stream.push(b's');
        } else {
            self.put(b'S', value.as_bytes());
        }
    }

    /// Writes a number's value, or its placeholder where numbers are loosened (spec SAME-2).
    fn number(&mut self, value: &[u8]) {
        if self.lens.numbers {
            self.stream.push(b'd');
        } else {
            self.put(b'D', value);
        }
    }

    /// Writes the values of a node that are neither names, strings nor numbers.
    fn flags(&mut self, values: fmt::Arguments<'_>) {
        self.put(b'f', values.to_string().as_bytes());
    }

    /// Notes the parentheses of a parenthesized expression or type, which the size leaves out.
    fn unwrapped(&mut self, span: Span) {
        self.dropped.opens.insert(span.start);
        self.dropped.closes.insert(span.end);
    }

    /// Notes an arrow function's lone, plain parameter, whose parentheses the size leaves out: one with
    /// no type, default, decorator or `?`, in a function with no type parameters and no return type.
    fn bare(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        let params = &arrow.params;

        if let [only] = &params.items[..]
            && params.rest.is_none()
            && arrow.type_parameters.is_none()
            && arrow.return_type.is_none()
            && matches!(only.pattern, BindingPattern::BindingIdentifier(_))
            && only.type_annotation.is_none()
            && only.initializer.is_none()
            && only.decorators.is_empty()
            && !only.optional
        {
            self.dropped.lone.insert(only.span.start);
        }
    }

    /// Writes the values that make two nodes of one kind differ: every field oxc's `ContentEq` compares
    /// that is not a child node, and which optional child is present where two are of one type (spec
    /// SAME-1).
    fn values(&mut self, kind: AstKind<'_>) {
        match kind {
            AstKind::IdentifierName(node) => self.name(node.name.as_str()),
            AstKind::IdentifierReference(node) => self.name(node.name.as_str()),
            AstKind::BindingIdentifier(node) => self.name(node.name.as_str()),
            AstKind::LabelIdentifier(node) => self.name(node.name.as_str()),
            AstKind::PrivateIdentifier(node) => self.name(node.name.as_str()),
            AstKind::JSXIdentifier(node) => self.name(node.name.as_str()),
            AstKind::TSIndexSignatureName(node) => self.name(node.name.as_str()),
            AstKind::StringLiteral(node) => self.string(&node.value.display().to_string()),
            AstKind::TemplateElement(node) => {
                self.string(&template(node));
                self.flags(format_args!("{}", node.tail));
            }
            AstKind::JSXText(node) => self.string(&rendered(node.value.as_str())),
            AstKind::RegExpLiteral(node) => self.string(&format!(
                "/{}/{:?}",
                node.regex.pattern.text.as_str(),
                node.regex.flags
            )),
            AstKind::Directive(node) => self.string(node.directive.as_str()),
            AstKind::NumericLiteral(node) => self.number(&node.value.to_bits().to_le_bytes()),
            AstKind::BigIntLiteral(node) => self.number(node.value.as_str().as_bytes()),
            AstKind::BooleanLiteral(node) => self.flags(format_args!("{}", node.value)),
            AstKind::AccessorProperty(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.computed,
                    &node.definite,
                    &node.r#override,
                    &node.r#static,
                    &node.r#type
                )
            )),
            AstKind::ArrowFunctionExpression(node) => {
                self.flags(format_args!("{}", node.r#async));
                self.bare(node);
            }
            AstKind::AssignmentExpression(node) => {
                self.flags(format_args!("{:?}", node.operator));
            }
            AstKind::AssignmentTargetPropertyProperty(node) => {
                self.flags(format_args!("{}", node.computed));
            }
            AstKind::BinaryExpression(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::BindingProperty(node) => {
                self.flags(format_args!("{:?}", (&node.computed, &node.shorthand)));
            }
            AstKind::CallExpression(node) => self.flags(format_args!("{}", node.optional)),
            AstKind::Class(node) => self.flags(format_args!(
                "{:?}",
                (&node.declare, &node.r#abstract, &node.r#type)
            )),
            AstKind::ComputedMemberExpression(node) => {
                self.flags(format_args!("{}", node.optional));
            }
            AstKind::ExportAllDeclaration(node) => {
                self.flags(format_args!("{:?}", node.export_kind));
            }
            AstKind::ExportNamedDeclaration(node) => {
                self.flags(format_args!("{:?}", node.export_kind));
            }
            AstKind::ExportSpecifier(node) => self.flags(format_args!("{:?}", node.export_kind)),
            AstKind::ForOfStatement(node) => self.flags(format_args!("{}", node.r#await)),
            AstKind::ForStatement(node) => self.flags(format_args!(
                "{:?}",
                (
                    node.init.is_some(),
                    node.test.is_some(),
                    node.update.is_some()
                )
            )),
            AstKind::FormalParameter(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.optional,
                    &node.r#override,
                    &node.readonly
                )
            )),
            AstKind::FormalParameters(node) => self.flags(format_args!("{:?}", node.kind)),
            AstKind::Function(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.declare,
                    &node.generator,
                    &node.r#async,
                    &node.r#type
                )
            )),
            AstKind::ImportDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.import_kind, &node.phase)));
            }
            AstKind::ImportExpression(node) => self.flags(format_args!("{:?}", node.phase)),
            AstKind::ImportSpecifier(node) => self.flags(format_args!("{:?}", node.import_kind)),
            AstKind::JSDocNullableType(node) => self.flags(format_args!("{}", node.postfix)),
            AstKind::JSDocNonNullableType(node) => self.flags(format_args!("{}", node.postfix)),
            AstKind::LogicalExpression(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::MethodDefinition(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.computed,
                    &node.kind,
                    &node.optional,
                    &node.r#override,
                    &node.r#static,
                    &node.r#type
                )
            )),
            AstKind::NewExpression(node) if node.arguments.is_empty() => {
                self.dropped.empty.insert(node.span.end);
            }
            AstKind::ObjectProperty(node) => self.flags(format_args!(
                "{:?}",
                (&node.computed, &node.kind, &node.method, &node.shorthand)
            )),
            AstKind::PrivateFieldExpression(node) => {
                self.flags(format_args!("{}", node.optional));
            }
            AstKind::PropertyDefinition(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.accessibility,
                    &node.computed,
                    &node.declare,
                    &node.definite,
                    &node.optional,
                    &node.r#override,
                    &node.r#static,
                    &node.r#type,
                    &node.readonly
                )
            )),
            AstKind::StaticMemberExpression(node) => {
                self.flags(format_args!("{}", node.optional));
            }
            AstKind::TSConstructorType(node) => self.flags(format_args!("{}", node.r#abstract)),
            AstKind::TSEnumDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.declare, &node.r#const)));
            }
            AstKind::TSExternalModuleDeclaration(node) => {
                self.flags(format_args!("{}", node.declare));
            }
            AstKind::TSGlobalDeclaration(node) => self.flags(format_args!("{}", node.declare)),
            AstKind::TSImportEqualsDeclaration(node) => {
                self.flags(format_args!("{:?}", node.import_kind));
            }
            AstKind::TSIndexSignature(node) => {
                self.flags(format_args!("{:?}", (&node.r#static, &node.readonly)));
            }
            AstKind::TSInterfaceDeclaration(node) => self.flags(format_args!("{}", node.declare)),
            AstKind::TSMappedType(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.optional,
                    &node.readonly,
                    node.name_type.is_some(),
                    node.type_annotation.is_some()
                )
            )),
            AstKind::TSMethodSignature(node) => self.flags(format_args!(
                "{:?}",
                (&node.computed, &node.kind, &node.optional)
            )),
            AstKind::TSNamedTupleMember(node) => self.flags(format_args!("{}", node.optional)),
            AstKind::TSNamespaceDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.declare, &node.kind)));
            }
            AstKind::TSPropertySignature(node) => self.flags(format_args!(
                "{:?}",
                (&node.computed, &node.optional, &node.readonly)
            )),
            AstKind::TSTypeAliasDeclaration(node) => self.flags(format_args!("{}", node.declare)),
            AstKind::TSTypeOperator(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::TSTypeParameter(node) => self.flags(format_args!(
                "{:?}",
                (
                    &node.r#in,
                    &node.out,
                    &node.r#const,
                    node.constraint.is_some(),
                    node.default.is_some()
                )
            )),
            AstKind::TSTypePredicate(node) => self.flags(format_args!("{}", node.asserts)),
            AstKind::UnaryExpression(node) => self.flags(format_args!("{:?}", node.operator)),
            AstKind::UpdateExpression(node) => {
                self.flags(format_args!("{:?}", (&node.operator, &node.prefix)));
            }
            AstKind::VariableDeclaration(node) => {
                self.flags(format_args!("{:?}", (&node.declare, &node.kind)));
            }
            AstKind::VariableDeclarator(node) => self.flags(format_args!("{}", node.definite)),
            AstKind::WithClause(node) => self.flags(format_args!("{:?}", node.keyword)),
            AstKind::YieldExpression(node) => self.flags(format_args!("{}", node.delegate)),
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Former<'_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            // Parentheses that change nothing are not part of the shape (spec SAME-1).
            AstKind::ParenthesizedExpression(node) => self.unwrapped(node.span),
            AstKind::TSParenthesizedType(node) => self.unwrapped(node.span),
            // A JSX text that holds only white space is not part of the shape (spec SAME-1).
            AstKind::JSXText(node) if rendered(node.value.as_str()).is_empty() => {}
            _ => {
                self.put(b'w', format!("{:?}", kind.ty()).as_bytes());
                self.values(kind);
            }
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::ParenthesizedExpression(_) | AstKind::TSParenthesizedType(_) => {}
            AstKind::JSXText(node) if rendered(node.value.as_str()).is_empty() => {}
            _ => self.stream.push(CLOSE),
        }
    }

    fn visit_statements(&mut self, statements: &ArenaVec<'a, Statement<'a>>) {
        let list = self.lists.len();

        self.lists.push(Vec::new());

        for statement in statements {
            // An import never joins a run (spec COPY-2).
            let joins = !matches!(
                statement,
                Statement::ImportDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
                    | Statement::ExportFromDeclaration(_)
                    | Statement::TSImportEqualsDeclaration(_)
            );

            self.item(list, statement.span(), joins, |former| {
                former.visit_statement(statement);
            });
        }
    }

    fn visit_class_elements(&mut self, elements: &ArenaVec<'a, ClassElement<'a>>) {
        let list = self.lists.len();

        self.lists.push(Vec::new());

        for element in elements {
            self.item(list, element.span(), true, |former| {
                former.visit_class_element(element);
            });
        }
    }

    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
        // A key written in quotes is the name it spells (spec SAME-1).
        if let PropertyKey::StringLiteral(literal) = key {
            self.put(b'w', b"IdentifierName");
            self.name(&literal.value.display().to_string());
            self.stream.push(CLOSE);
        } else {
            walk::walk_property_key(self, key);
        }
    }
}

/// The tokens a size counts, with the lines they start on (spec SIZE-1).
struct Counted {
    /// Where each starts, in order.
    starts: Vec<u32>,
    /// The line each starts on.
    on: Vec<usize>,
    /// How many of the tokens before each, and before the end, start on a line the token before them
    /// does not.
    breaks: Vec<u64>,
}

impl Counted {
    /// The tokens of a program the fixed form counts: all but every `;`, a `,` just before a closing
    /// bracket, the parentheses the walk dropped, an empty `()` after `new`, the parentheses around an
    /// arrow function's lone parameter, and a JSX text of white space only.
    fn of(tokens: &[Token], text: &str, lines: &Lines<'_>, dropped: &Dropped) -> Self {
        let tokens: Vec<&Token> = tokens
            .iter()
            .filter(|token| !matches!(token.kind(), Kind::Skip | Kind::Eof))
            .collect();
        let mut kept = vec![true; tokens.len()];

        for (index, token) in tokens.iter().enumerate() {
            let before = index
                .checked_sub(1)
                .and_then(|before| tokens.get(before))
                .map(|before| before.kind());
            let next = tokens.get(index + 1).map(|next| next.kind());

            match token.kind() {
                Kind::Semicolon => kept[index] = false,
                Kind::Comma if matches!(next, Some(Kind::RParen | Kind::RBrack | Kind::RCurly)) => {
                    kept[index] = false;
                }
                Kind::LParen if dropped.opens.contains(&token.start()) => kept[index] = false,
                Kind::RParen if dropped.closes.contains(&token.end()) => kept[index] = false,
                Kind::RParen
                    if dropped.empty.contains(&token.end())
                        && matches!(before, Some(Kind::LParen)) =>
                {
                    kept[index] = false;
                    kept[index - 1] = false;
                }
                Kind::JSXText => {
                    let offset = |at: u32| usize::try_from(at).unwrap_or(usize::MAX);
                    let piece = text.get(offset(token.start())..offset(token.end()));

                    if piece.is_none_or(|piece| piece.trim().is_empty()) {
                        kept[index] = false;
                    }
                }
                _ => {}
            }

            if dropped.lone.contains(&token.start())
                && matches!(before, Some(Kind::LParen))
                && matches!(next, Some(Kind::RParen))
            {
                kept[index - 1] = false;
                kept[index + 1] = false;
            }
        }

        let mut starts = Vec::new();
        let mut on = Vec::new();

        for (token, kept) in tokens.iter().zip(kept) {
            if kept {
                starts.push(token.start());
                on.push(lines.at(token.start()));
            }
        }

        let mut breaks = Vec::with_capacity(on.len() + 1);

        breaks.push(0);

        for (index, line) in on.iter().enumerate() {
            let new = index > 0 && on[index - 1] != *line;

            breaks.push(breaks[index] + u64::from(new));
        }

        Self { starts, on, breaks }
    }

    /// One statement, with its digest, its size and its place.
    fn item(&self, raw: &Raw, stream: &[u8], lines: &Lines<'_>) -> Item {
        let first = self.starts.partition_point(|start| *start < raw.span.start);
        let after = self.starts.partition_point(|start| *start < raw.span.end);
        let digest = Sha256::digest(&stream[raw.from..raw.to]);
        let mut kept = [0; DIGEST];

        kept.copy_from_slice(&digest[..DIGEST]);

        Item {
            digest: kept,
            joins: raw.joins,
            tokens: u64::try_from(after - first).unwrap_or(u64::MAX),
            lines: (after > first).then(|| Held {
                first: self.on[first],
                last: self.on[after - 1],
                count: 1 + self.breaks[after] - self.breaks[first + 1],
            }),
            start: raw.span.start,
            end: raw.span.end,
            line: lines.at(raw.span.start),
            end_line: lines.at(raw.span.end.saturating_sub(1)),
        }
    }
}
```

```rust path=src/duplication.rs
//! The `duplication` command, and the groups of copies a reading holds, which `audit` will report too:
//! the settings the declarations give, the runs of statements that repeat, the groups they make with
//! their sizes and fingerprints, and the document the command prints (duplication spec SAME-3, SAME-4,
//! SIZE-1, SIZE-2, TUNE-1 to TUNE-7, DUP-4, SHOW-1 to SHOW-5).

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::INTERFACE_VERSION;
use crate::complexity;
use crate::declarations::WORDS;
use crate::files::{self, Kind};
use crate::findings::{self, Rule};
use crate::forms::{DIGEST, Item, List};
use crate::hand_off::{HandOff, Stack};
use crate::notice::{self, Notice};
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::timings::Stages;
use crate::tree::Tree;

/// The kind of the notice for a run with no `duplicationMinimum` (spec TUNE-1).
const NOT_CHECKED: &str = "duplication-not-checked";

/// The fewest copies a group needs where no declaration says (spec TUNE-3).
const COPIES: u64 = 2;

/// What the parse needs of the declarations to read a script's statements (spec SAME-2, COPY-3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lens {
    /// Whether any name matches any other.
    pub names: bool,
    /// Whether any string matches any other.
    pub strings: bool,
    /// Whether any number matches any other.
    pub numbers: bool,
    /// The callee chains of the test callees.
    pub test_callees: BTreeSet<String>,
}

/// The settings of duplication, one of each for the whole run (spec TUNE-1 to TUNE-7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The fewest tokens a group's run holds, from `duplicationMinimum`; none, and nothing is looked
    /// for, where it is not declared.
    pub minimum_tokens: Option<u64>,
    /// The fewest lines, from `duplicationMinimumLines`; none where it is not declared.
    pub minimum_lines: Option<u64>,
    /// The fewest copies, from `duplicationCopies`, 2 where it is not declared.
    pub copies: u64,
    /// What the parse loosens and which calls are test callees.
    pub lens: Lens,
    /// The entries of `duplicationAccepted`, the pack's before the project's.
    pub accepted: Vec<String>,
}

impl Settings {
    /// The settings the hand-off file declares: a number is the project's, else the later stack's; the
    /// project's word list replaces the pack's, and of one source the later stack's wins; the test
    /// callees of one source join across its stacks, and the project's replace the pack's; accepted
    /// groups join (spec TUNE-1 to TUNE-7).
    pub fn of(handed: Option<&HandOff>) -> Self {
        let stacks = handed.map_or(&[][..], |handed| handed.stacks.as_slice());
        let ignore = replaced(stacks, "duplicationIgnore").unwrap_or_default();
        let loosened = |word: &str| ignore.iter().any(|listed| listed == word);
        let test_callees = joined(stacks, "duplicationTestCallees", true)
            .or_else(|| joined(stacks, "duplicationTestCallees", false))
            .unwrap_or_default();

        Self {
            minimum_tokens: complexity::declared(stacks, "duplicationMinimum"),
            minimum_lines: complexity::declared(stacks, "duplicationMinimumLines"),
            copies: complexity::declared(stacks, "duplicationCopies").unwrap_or(COPIES),
            lens: Lens {
                names: loosened("names"),
                strings: loosened("strings"),
                numbers: loosened("numbers"),
                test_callees: test_callees.into_iter().collect(),
            },
            accepted: [false, true]
                .into_iter()
                .flat_map(|project| {
                    joined(stacks, "duplicationAccepted", project).unwrap_or_default()
                })
                .collect(),
        }
    }

    /// What the parse reads statements through, where duplication is looked for at all (spec TUNE-1).
    pub fn lens(&self) -> Option<&Lens> {
        self.minimum_tokens.map(|_| &self.lens)
    }

    /// The notice for a run with no `duplicationMinimum`, where it has none (spec TUNE-1).
    pub fn notice(&self) -> Option<Notice> {
        self.minimum_tokens.is_none().then(|| {
            Notice::new(
                NOT_CHECKED,
                None,
                "No `duplicationMinimum` is declared, so no duplication is looked for and no duplication \
                 finding is reported."
                    .to_owned(),
            )
        })
    }

    /// The settings as the command prints them (spec SHOW-2).
    fn printed(&self) -> Value {
        json!({
            "accepted": self.accepted,
            "ignore": self.ignored(),
            "minimumCopies": self.copies,
            "minimumLines": self.minimum_lines,
            "minimumTokens": self.minimum_tokens,
            "testCallees": self.lens.test_callees,
        })
    }

    /// The words loosened, in the order `WORDS` gives them.
    fn ignored(&self) -> Vec<&'static str> {
        WORDS
            .into_iter()
            .zip([self.lens.names, self.lens.strings, self.lens.numbers])
            .filter_map(|(word, loosened)| loosened.then_some(word))
            .collect()
    }
}

/// The list the project states for a key in its last stack that states it, else the pack's in its last
/// stack that states it (spec TUNE-4).
fn replaced(stacks: &[Stack], key: &str) -> Option<Vec<String>> {
    let last = |project: bool| {
        stacks.iter().rev().find_map(|stack| {
            declarations(stack, project)
                .get(key)
                .and_then(texts)
        })
    };

    last(true).or_else(|| last(false))
}

/// The lists one source states for a key, joined across its stacks in their order; none where none of
/// its stacks states the key (spec TUNE-5, TUNE-6).
fn joined(stacks: &[Stack], key: &str, project: bool) -> Option<Vec<String>> {
    let lists: Vec<Vec<String>> = stacks
        .iter()
        .filter_map(|stack| {
            declarations(stack, project)
                .get(key)
                .and_then(texts)
        })
        .collect();

    (!lists.is_empty()).then(|| lists.concat())
}

/// The pack's or the project's declarations of a stack.
const fn declarations(stack: &Stack, project: bool) -> &Map<String, Value> {
    if project { &stack.project } else { &stack.pack }
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|entry| entry.as_str().map(str::to_owned))
        .collect()
}

/// One copy of a group: a run of statements of one list of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// The file, by its path from the root.
    pub path: String,
    /// The line its first statement starts on, counted from one.
    pub start_line: usize,
    /// The line its last statement ends on.
    pub end_line: usize,
    /// Its first byte in the file.
    pub start: u32,
    /// The byte after its last.
    pub end: u32,
}

/// One group of copies (spec SAME-1 to SAME-4, SIZE-1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// The fingerprint `audit` gives, or would give, its finding (spec DUP-4).
    pub fingerprint: String,
    /// Its tokens: the fewest of any of its copies.
    pub tokens: u64,
    /// Its lines: the fewest of any of its copies.
    pub lines: u64,
    /// Its copies, by path and first line.
    pub copies: Vec<Place>,
    /// Whether a person accepted it (spec QUIET-1).
    pub accepted: bool,
}

/// Every group of copies in the reading's own scripts and Vue components, each run as large as it goes,
/// sorted by its first copy; none where no `duplicationMinimum` is declared. A file declared data or
/// generated, and a type declaration file, holds no copy (spec COPY-4, SAME-3, SAME-4, SIZE-2).
pub fn groups(reading: &Reading) -> Vec<Group> {
    let settings = &reading.duplication;
    let Some(minimum) = settings.minimum_tokens else {
        return Vec::new();
    };
    let files: Vec<(&str, &[List])> = reading
        .files
        .iter()
        .filter(|file| {
            matches!(file.kind, Kind::Script | Kind::Vue)
                && !reading.marks.generated.contains(&file.path)
        })
        .filter_map(|file| {
            reading
                .scripts
                .read
                .get(&file.path)
                .map(|read| (file.path.as_str(), read.lists.as_slice()))
        })
        .collect();
    let text = Text::of(files);
    let found: Vec<Group> = text
        .repeats()
        .into_iter()
        .filter_map(|(length, at)| text.group(length, at, settings, minimum))
        .collect();
    let mut groups = widest(found);

    for group in &mut groups {
        group
            .copies
            .sort_by(|a, b| (&a.path, a.start_line).cmp(&(&b.path, b.start_line)));
    }

    groups.sort_by(|a, b| order(a).cmp(&order(b)));
    groups
}

/// What groups are sorted by: their first copy's path and lines, then their number of copies.
fn order(group: &Group) -> (Option<(&str, usize, usize)>, usize) {
    (
        group
            .copies
            .first()
            .map(|copy| (copy.path.as_str(), copy.start_line, copy.end_line)),
        group.copies.len(),
    )
}

/// Every statement list of the files, end to end, as one sequence of symbols, in which the runs that
/// repeat are found.
struct Text<'r> {
    /// The files, by their paths, with their lists.
    files: Vec<(&'r str, &'r [List])>,
    /// Each place's symbol: one number for each form a statement that joins runs has, a number of its
    /// own for each statement that joins none, and one after each list, so no run crosses a list.
    symbols: Vec<u64>,
    /// Where each place is: its file, its list and its statement; none after a list.
    places: Vec<Option<(usize, usize, usize)>>,
}

impl<'r> Text<'r> {
    /// The sequence of the files' lists.
    fn of(files: Vec<(&'r str, &'r [List])>) -> Self {
        let mut forms: HashMap<[u8; DIGEST], u64> = HashMap::new();
        // The numbers of what joins no run start above any form's.
        let mut alone = u64::from(u32::MAX);
        let mut symbols = Vec::new();
        let mut places = Vec::new();

        for (file, (_, lists)) in files.iter().enumerate() {
            for (list, statements) in lists.iter().enumerate() {
                for (index, item) in statements.items.iter().enumerate() {
                    let symbol = if item.joins {
                        let next = u64::try_from(forms.len()).unwrap_or(u64::MAX);

                        *forms.entry(item.digest).or_insert(next)
                    } else {
                        alone += 1;
                        alone
                    };

                    symbols.push(symbol);
                    places.push(Some((file, list, index)));
                }

                alone += 1;
                symbols.push(alone);
                places.push(None);
            }
        }

        Self {
            files,
            symbols,
            places,
        }
    }

    /// Each run that repeats, as its length in statements and the places it starts at, where its
    /// copies are not all preceded by the same statement, so it cannot be made longer at its start
    /// (spec SAME-4).
    fn repeats(&self) -> Vec<(usize, Vec<usize>)> {
        let order = suffixes(&self.symbols);
        let common = common(&self.symbols, &order);

        intervals(&common)
            .into_iter()
            .filter_map(|(length, first, last)| {
                let at = order[first..=last].to_vec();
                let before: BTreeSet<Option<u64>> = at
                    .iter()
                    .map(|place| place.checked_sub(1).map(|before| self.symbols[before]))
                    .collect();

                (before.len() > 1 || before.contains(&None)).then_some((length, at))
            })
            .collect()
    }

    /// The group a run makes from the places given, where it has enough copies that do not overlap and
    /// is large enough (spec SAME-3, SIZE-1, SIZE-2, DUP-4).
    fn group(
        &self,
        length: usize,
        mut at: Vec<usize>,
        settings: &Settings,
        minimum: u64,
    ) -> Option<Group> {
        at.sort_unstable_by_key(|place| self.places[*place]);

        // Two copies never overlap: in one list, the copies are taken from its start (spec SAME-3).
        let mut kept: Vec<(usize, usize, usize)> = Vec::new();

        for (file, list, index) in at.into_iter().filter_map(|place| self.places[place]) {
            let overlaps = kept
                .last()
                .is_some_and(|&(last_file, last_list, last_index)| {
                    last_file == file && last_list == list && index < last_index + length
                });

            if !overlaps {
                kept.push((file, list, index));
            }
        }

        if u64::try_from(kept.len()).unwrap_or(u64::MAX) < settings.copies {
            return None;
        }

        let runs: Vec<(&str, &[Item])> = kept
            .iter()
            .map(|&(file, list, index)| {
                let (path, lists) = self.files[file];

                (path, &lists[list].items[index..index + length])
            })
            .collect();
        let tokens = runs.iter().map(|(_, run)| tokens(run)).min()?;
        let lines = runs.iter().map(|(_, run)| lines(run)).min()?;

        if tokens < minimum || settings.minimum_lines.is_some_and(|least| lines < least) {
            return None;
        }

        let parts: Vec<String> = runs
            .first()?
            .1
            .iter()
            .map(|item| hex(&item.digest))
            .collect();
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

        Some(Group {
            fingerprint: findings::fingerprint(Rule::Duplication, &parts),
            tokens,
            lines,
            copies: runs.iter().map(|(path, run)| place(path, run)).collect(),
            accepted: false,
        })
    }
}

/// The suffix array of a sequence: its places, sorted by what follows each, by prefix doubling.
fn suffixes(symbols: &[u64]) -> Vec<usize> {
    let count = symbols.len();
    let mut order: Vec<usize> = (0..count).collect();

    if count < 2 {
        return order;
    }

    let mut sorted = symbols.to_vec();

    sorted.sort_unstable();
    sorted.dedup();

    let mut rank: Vec<usize> = symbols
        .iter()
        .map(|symbol| sorted.partition_point(|other| other < symbol))
        .collect();
    let mut next = vec![0; count];
    let mut step = 1;

    loop {
        let key = |place: usize| {
            (
                rank[place],
                rank.get(place + step).map_or(0, |after| after + 1),
            )
        };

        order.sort_unstable_by_key(|place| key(*place));
        next[order[0]] = 0;

        for pair in order.windows(2) {
            next[pair[1]] = next[pair[0]] + usize::from(key(pair[0]) != key(pair[1]));
        }

        std::mem::swap(&mut rank, &mut next);

        if rank[order[count - 1]] == count - 1 {
            break;
        }

        step *= 2;
    }

    order
}

/// For each place of the suffix array, the length of the prefix it shares with the place before it; 0
/// for the first (Kasai's method).
fn common(symbols: &[u64], order: &[usize]) -> Vec<usize> {
    let count = symbols.len();
    let mut rank = vec![0; count];

    for (at, place) in order.iter().enumerate() {
        rank[*place] = at;
    }

    let mut common = vec![0; count];
    let mut shared = 0;

    for (place, &at) in rank.iter().enumerate() {
        if at == 0 {
            shared = 0;

            continue;
        }

        let other = order[at - 1];

        while place + shared < count
            && other + shared < count
            && symbols[place + shared] == symbols[other + shared]
        {
            shared += 1;
        }

        common[at] = shared;
        shared = shared.saturating_sub(1);
    }

    common
}

/// The intervals of the suffix array whose places share a prefix longer than nothing: its length, and
/// the interval's first and last places in the array (the bottom-up walk of Abouelhoda, Kurtz and
/// Ohlebusch).
fn intervals(common: &[usize]) -> Vec<(usize, usize, usize)> {
    let mut found = Vec::new();
    let mut open: Vec<(usize, usize)> = vec![(0, 0)];

    for at in 1..=common.len() {
        let shared = common.get(at).copied().unwrap_or(0);
        let mut left = at - 1;

        while open.last().is_some_and(|&(length, _)| shared < length) {
            if let Some((length, first)) = open.pop() {
                found.push((length, first, at - 1));
                left = first;
            }
        }

        if open.last().is_none_or(|&(length, _)| shared > length) {
            open.push((shared, left));
        }
    }

    found
}

/// A run's tokens: its statements' together (spec SIZE-1).
fn tokens(run: &[Item]) -> u64 {
    run.iter().map(|item| item.tokens).sum()
}

/// A run's lines: its statements' together, a line two of them share counted once (spec SIZE-1).
fn lines(run: &[Item]) -> u64 {
    let mut count = 0;
    let mut last = None;

    for held in run.iter().filter_map(|item| item.lines) {
        count += held.count;

        if last == Some(held.first) {
            count -= 1;
        }

        last = Some(held.last);
    }

    count
}

/// Where a run is.
fn place(path: &str, run: &[Item]) -> Place {
    let first = run.first();
    let last = run.last();

    Place {
        path: path.to_owned(),
        start_line: first.map_or(0, |item| item.line),
        end_line: last.map_or(0, |item| item.end_line),
        start: first.map_or(0, |item| item.start),
        end: last.map_or(0, |item| item.end),
    }
}

/// A digest in lower-case hexadecimal digits.
fn hex(digest: &[u8]) -> String {
    let mut digits = String::with_capacity(digest.len() * 2);

    for byte in digest {
        // Writing into a String cannot fail.
        let _ = write!(digits, "{byte:02x}");
    }

    digits
}

/// The groups left once each that lies inside another is dropped: one whose every copy lies inside a
/// different copy of another group with at least as many copies. Of two that each lie inside the other,
/// the first stays (spec SAME-4).
fn widest(found: Vec<Group>) -> Vec<Group> {
    let inside = |inner: &Group, outer: &Group| {
        let mut used = BTreeSet::new();

        outer.copies.len() >= inner.copies.len()
            && inner.copies.iter().all(|copy| {
                outer
                    .copies
                    .iter()
                    .position(|around| {
                        around.path == copy.path
                            && around.start <= copy.start
                            && copy.end <= around.end
                    })
                    .is_some_and(|at| used.insert(at))
            })
    };
    let kept: Vec<bool> = found
        .iter()
        .enumerate()
        .map(|(at, group)| {
            !found.iter().enumerate().any(|(other_at, other)| {
                other_at != at && inside(group, other) && !(other_at > at && inside(other, group))
            })
        })
        .collect();

    found
        .into_iter()
        .zip(kept)
        .filter_map(|(group, kept)| kept.then_some(group))
        .collect()
}

/// Reads the working tree as `audit` does, with the hand-off file given or with none, and returns the
/// document `duplication` prints: the settings, the groups a person has not accepted, or with `all`
/// every group, and the notices (spec SHOW-1 to SHOW-3).
pub fn run(hand_off: Option<&Path>, all: bool) -> Result<String, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = reading::start(hand_off)?;
    let listing = files::list(&root, &mut notices)?;
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
    let listed: Vec<Value> = groups(&reading)
        .iter()
        .filter(|group| all || !group.accepted)
        .map(entry)
        .collect();

    reading.notices.extend(reading.duplication.notice());

    let mut document = json!({
        "groups": listed,
        "interfaceVersion": INTERFACE_VERSION,
        "notices": notice::printed(reading.notices),
        "settings": reading.duplication.printed(),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// One group as the document lists it (spec SHOW-2).
fn entry(group: &Group) -> Value {
    json!({
        "accepted": group.accepted,
        "copies": group
            .copies
            .iter()
            .map(|copy| json!({ "endLine": copy.end_line, "path": copy.path, "startLine": copy.start_line }))
            .collect::<Vec<Value>>(),
        "fingerprint": group.fingerprint,
        "lines": group.lines,
        "tokens": group.tokens,
    })
}
```

```rust edit path=src/complexity.rs
<<<<<<< find
/// The value of a limit's key that wins: the last stack's that the project states it for, or else the
/// last stack's that the pack states it for.
fn declared(stacks: &[Stack], key: &str) -> Option<u64> {
=======
/// The value of a number key that wins: the last stack's that the project states it for, or else the
/// last stack's that the pack states it for (spec LIMIT-1; duplication spec TUNE-7).
pub fn declared(stacks: &[Stack], key: &str) -> Option<u64> {
>>>>>>> replace
```

```rust edit path=src/findings.rs
<<<<<<< find
    /// A function whose complexity is over a declared limit (complexity spec CPX-1).
    Complexity,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 6] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
        Self::Complexity,
    ];
=======
    /// A function whose complexity is over a declared limit (complexity spec CPX-1).
    Complexity,
    /// A group of runs of statements that are the same (duplication spec DUP-1).
    Duplication,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 7] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
        Self::Complexity,
        Self::Duplication,
    ];
>>>>>>> replace
<<<<<<< find
            Self::Complexity => "complexity",
        }
    }
=======
            Self::Complexity => "complexity",
            Self::Duplication => "duplication",
        }
    }
>>>>>>> replace
<<<<<<< find
            Self::Complexity => {
                "A function whose cyclomatic or cognitive complexity is over its limit."
            }
        }
    }
=======
            Self::Complexity => {
                "A function whose cyclomatic or cognitive complexity is over its limit."
            }
            Self::Duplication => "Runs of statements that are the same in two or more places.",
        }
    }
>>>>>>> replace
```

```rust edit path=src/imports.rs
<<<<<<< find
use oxc_parser::Parser;
=======
use oxc_parser::Parser;
use oxc_parser::config::RuntimeParserConfig;
>>>>>>> replace
<<<<<<< find
use crate::functions::{self, Unit};
=======
use crate::duplication::Lens;
use crate::forms::{self, List};
use crate::functions::{self, Unit};
>>>>>>> replace
<<<<<<< find
    /// Its functions, classes and object literals, in source order, from which its functions' names
    /// and scores are read (complexity spec FN-3).
    pub units: Vec<Unit>,
}
=======
    /// Its functions, classes and object literals, in source order, from which its functions' names
    /// and scores are read (complexity spec FN-3).
    pub units: Vec<Unit>,
    /// Its statement lists, where duplication is looked for; none otherwise (duplication spec COPY-5).
    pub lists: Vec<List>,
}
>>>>>>> replace
<<<<<<< find
/// Parses a script, as a module first and then as a script, and finds its imports and its names (spec
/// PARSE-1).
pub fn read(path: &str, text: &str) -> Result<Read, Broken> {
    read_as(text, source_type(path), false)
}
=======
/// Parses a script, as a module first and then as a script, and finds its imports and its names (spec
/// PARSE-1), and its statement lists through the lens where one is given (duplication spec COPY-5).
pub fn read(path: &str, text: &str, lens: Option<&Lens>) -> Result<Read, Broken> {
    read_as(text, source_type(path), false, lens)
}
>>>>>>> replace
<<<<<<< find
pub fn read_block(text: &str, lang: Option<&str>) -> Option<Result<Read, Broken>> {
=======
pub fn read_block(
    text: &str,
    lang: Option<&str>,
    lens: Option<&Lens>,
) -> Option<Result<Read, Broken>> {
>>>>>>> replace
<<<<<<< find
    Some(read_as(text, source_type, true))
}
=======
    Some(read_as(text, source_type, true, lens))
}
>>>>>>> replace
<<<<<<< find
    parse(text, SourceType::jsx().with_module(true), false).map(|read| {
=======
    parse(text, SourceType::jsx().with_module(true), false, None).map(|read| {
>>>>>>> replace
<<<<<<< find
/// Parses as a module first and then as a script.
fn read_as(text: &str, source_type: SourceType, component: bool) -> Result<Read, Broken> {
    let module = source_type.with_module(true);

    match parse(text, module, component) {
        Ok(read) => Ok(read),
        Err(broken) => parse(text, module.with_script(true), component).map_err(|_| broken),
    }
}
=======
/// Parses as a module first and then as a script.
fn read_as(
    text: &str,
    source_type: SourceType,
    component: bool,
    lens: Option<&Lens>,
) -> Result<Read, Broken> {
    let module = source_type.with_module(true);

    match parse(text, module, component, lens) {
        Ok(read) => Ok(read),
        Err(broken) => parse(text, module.with_script(true), component, lens).map_err(|_| broken),
    }
}
>>>>>>> replace
<<<<<<< find
/// Parses once with the source type given. In a Vue component's block, the default export is the
/// component itself, so what it names is used and not passed on (spec USE-4).
fn parse(text: &str, source_type: SourceType, component: bool) -> Result<Read, Broken> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
=======
/// Parses once with the source type given. In a Vue component's block, the default export is the
/// component itself, so what it names is used and not passed on (spec USE-4). With a lens, the parser
/// keeps its tokens, and the statement lists are read too (duplication spec COPY-5).
fn parse(
    text: &str,
    source_type: SourceType,
    component: bool,
    lens: Option<&Lens>,
) -> Result<Read, Broken> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type)
        .with_config(RuntimeParserConfig::new(lens.is_some()))
        .parse();
>>>>>>> replace
<<<<<<< find
    // The functions are found and scored in the same parse (complexity spec FN-3).
    let units = functions::units(&parsed.program, &semantic, &lines);

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
        exports,
        named: Vec::new(),
        units,
    })
}
=======
    // The functions are found and scored in the same parse (complexity spec FN-3), and the statements
    // read (duplication spec COPY-5).
    let units = functions::units(&parsed.program, &semantic, &lines);
    let lists = lens
        .map(|lens| forms::lists(&parsed.program, &parsed.tokens, text, &lines, lens))
        .unwrap_or_default();

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
        exports,
        named: Vec::new(),
        units,
        lists,
    })
}
>>>>>>> replace
<<<<<<< find
    /// The line an offset lies on, counted from one.
    fn at(&self, offset: u32) -> usize {
=======
    /// The line an offset lies on, counted from one.
    pub fn at(&self, offset: u32) -> usize {
>>>>>>> replace
```

```rust edit path=src/pages.rs
<<<<<<< find
use crate::functions;
use crate::imports::{self, Broken, DEFAULT, EVERY, Form, Import, Name, Read, Used};
=======
use crate::duplication::Lens;
use crate::forms;
use crate::functions;
use crate::imports::{self, Broken, DEFAULT, EVERY, Form, Import, Name, Read, Used};
>>>>>>> replace
<<<<<<< find
pub fn vue(text: &str) -> Result<(Read, Vec<Skipped>), Broken> {
=======
pub fn vue(text: &str, lens: Option<&Lens>) -> Result<(Read, Vec<Skipped>), Broken> {
>>>>>>> replace
<<<<<<< find
        match imports::read_block(&text[block.content.clone()], lang) {
=======
        match imports::read_block(&text[block.content.clone()], lang, lens) {
>>>>>>> replace
<<<<<<< find
                functions::append(&mut read.units, found.units, above, indent);
=======
                functions::append(&mut read.units, found.units, above, indent);
                // A block's statements are counted in the file (duplication spec COPY-4).
                forms::append(
                    &mut read.lists,
                    found.lists,
                    above,
                    u32::try_from(block.content.start).unwrap_or(u32::MAX),
                );
>>>>>>> replace
```

```rust edit path=src/scripts.rs
<<<<<<< find
use crate::files::{File, Kind};
=======
use crate::duplication::Lens;
use crate::files::{File, Kind};
>>>>>>> replace
<<<<<<< find
/// the files (spec TIME-2, TIME-2b; parallel reading spec CLOCK-1).
pub fn read(
    tree: &Tree,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
    stages: &mut Stages,
) -> Result<Scripts, Stop> {
=======
/// the files (spec TIME-2, TIME-2b; parallel reading spec CLOCK-1).
///
/// With a lens, each script's and Vue block's statement lists are read through it (duplication spec
/// COPY-4, COPY-5).
pub fn read(
    tree: &Tree,
    files: &[File],
    data: &BTreeSet<String>,
    lens: Option<&Lens>,
    notices: &mut Vec<Notice>,
    stages: &mut Stages,
) -> Result<Scripts, Stop> {
>>>>>>> replace
<<<<<<< find
        match stages.time("scriptsParse", || one(file, bytes)) {
=======
        match stages.time("scriptsParse", || one(file, bytes, lens)) {
>>>>>>> replace
<<<<<<< find
/// Finds the imports of one file from what reading it from its tree gave, with what its reader passed
/// over.
fn one(file: &File, bytes: io::Result<Vec<u8>>) -> Result<(Read, Vec<Skipped>), Unread> {
=======
/// Finds the imports of one file from what reading it from its tree gave, with what its reader passed
/// over. A script's and a Vue component's statement lists are read through the lens, and no other
/// file's (duplication spec COPY-4).
fn one(
    file: &File,
    bytes: io::Result<Vec<u8>>,
    lens: Option<&Lens>,
) -> Result<(Read, Vec<Skipped>), Unread> {
>>>>>>> replace
<<<<<<< find
        Kind::Vue => pages::vue(text).map_err(broken),
        Kind::Mdx => Ok(pages::mdx(text)),
        Kind::Page => Ok((pages::html(text), Vec::new())),
        _ => imports::read(path, text)
            .map(|read| (read, Vec::new()))
            .map_err(broken),
=======
        Kind::Vue => pages::vue(text, lens).map_err(broken),
        Kind::Mdx => Ok(pages::mdx(text)),
        Kind::Page => Ok((pages::html(text), Vec::new())),
        _ => imports::read(path, text, lens.filter(|_| matches!(file.kind, Kind::Script)))
            .map(|read| (read, Vec::new()))
            .map_err(broken),
>>>>>>> replace
```

```rust edit path=src/reading.rs
<<<<<<< find
use crate::complexity::Limits;
use crate::declarations;
=======
use crate::complexity::Limits;
use crate::declarations;
use crate::duplication::Settings;
>>>>>>> replace
<<<<<<< find
    /// The limit of each measure of complexity the declarations give, one for the whole run
    /// (complexity spec LIMIT-1).
    pub limits: Limits,
}
=======
    /// The limit of each measure of complexity the declarations give, one for the whole run
    /// (complexity spec LIMIT-1).
    pub limits: Limits,
    /// The settings of duplication the declarations give, one of each for the whole run (duplication
    /// spec TUNE-7).
    pub duplication: Settings,
}
>>>>>>> replace
<<<<<<< find
    let scripts = scripts::read(tree, files, &marks.data, &mut notices, stages)?;
=======
    // The base is read with the same settings as the working tree (duplication spec TUNE-7).
    let duplication = Settings::of(handed);
    let scripts = scripts::read(
        tree,
        files,
        &marks.data,
        duplication.lens(),
        &mut notices,
        stages,
    )?;
>>>>>>> replace
<<<<<<< find
        limits: Limits::of(handed),
    })
}
=======
        limits: Limits::of(handed),
        duplication,
    })
}
>>>>>>> replace
```

```rust edit path=src/graph.rs
<<<<<<< find
        limits: _,
    } = reading;
=======
        limits: _,
        duplication: _,
    } = reading;
>>>>>>> replace
```

```rust edit path=src/cli.rs
<<<<<<< find
/// The option of `complexity` that lists every function, not only those over a limit (complexity spec
/// SOLO-3).
const ALL_OPTION: &str = "--all";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, graph [--hand-off <file>], audit [--hand-off <file>] \
                        [--base <commit>], and complexity [--hand-off <file>] [--all]";
=======
/// The command that reads a project and prints its groups of copies (duplication spec SHOW-1).
const DUPLICATION_COMMAND: &str = "duplication";

/// The option of `complexity` that lists every function, not only those over a limit, and of
/// `duplication` that lists every group, the accepted ones too (complexity spec SOLO-3; duplication
/// spec SHOW-3).
const ALL_OPTION: &str = "--all";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, graph [--hand-off <file>], audit [--hand-off <file>] \
                        [--base <commit>], complexity [--hand-off <file>] [--all], and duplication \
                        [--hand-off <file>] [--all]";
>>>>>>> replace
<<<<<<< find
    Complexity {
        hand_off: Option<PathBuf>,
        all: bool,
    },
}
=======
    Complexity {
        hand_off: Option<PathBuf>,
        all: bool,
    },
    /// Read the project and print its groups of copies, with a hand-off file or without one: those a
    /// person has not accepted, or with `all` every group (duplication spec SHOW-1).
    Duplication {
        hand_off: Option<PathBuf>,
        all: bool,
    },
}
>>>>>>> replace
<<<<<<< find
        [COMPLEXITY_COMMAND, ..] => complexity(args),
=======
        [COMPLEXITY_COMMAND, ..] => {
            listing(args).map(|(hand_off, all)| Command::Complexity { hand_off, all })
        }
        [DUPLICATION_COMMAND, ..] => {
            listing(args).map(|(hand_off, all)| Command::Duplication { hand_off, all })
        }
>>>>>>> replace
<<<<<<< find
/// `complexity` and its options, each at most once, in either order (complexity spec SOLO-1).
fn complexity(args: &[OsString]) -> Result<Command, String> {
=======
/// The options of `complexity` and of `duplication`, each at most once, in either order: the hand-off
/// file, and whether to list everything (complexity spec SOLO-1; duplication spec SHOW-1).
fn listing(args: &[OsString]) -> Result<(Option<PathBuf>, bool), String> {
>>>>>>> replace
<<<<<<< find
    Ok(Command::Complexity { hand_off, all })
}
=======
    Ok((hand_off, all))
}
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find
//! compared with a base commit where it is given one, and `complexity` prints the scores of each
//! function.
=======
//! compared with a base commit where it is given one, `complexity` prints the scores of each function,
//! and `duplication` prints the groups of copies.
>>>>>>> replace
<<<<<<< find
mod declarations;
mod entries;
mod files;
mod findings;
=======
mod declarations;
mod duplication;
mod entries;
mod files;
mod findings;
mod forms;
>>>>>>> replace
<<<<<<< find
        Ok(Command::Complexity { hand_off, all }) => {
            ended(complexity::run(hand_off.as_deref(), all))
        }
=======
        Ok(Command::Complexity { hand_off, all }) => {
            ended(complexity::run(hand_off.as_deref(), all))
        }
        Ok(Command::Duplication { hand_off, all }) => {
            ended(duplication::run(hand_off.as_deref(), all))
        }
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --release --locked
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
```

Expected: every test of every file passes, `graph`'s and `complexity`'s fixtures unchanged; rustfmt
changes nothing; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
yak stack check
git add Cargo.toml src tests
git commit -m "feat(duplication): find groups of repeated statements"
```

Expected: one commit.

### Task 3: Test callees

**Implements:** COPY-3, TUNE-5, FIXT-1

**Test:** `cargo test --locked --test duplication --test complexity`

**Files:**
- Modify: `src/forms.rs`, `src/functions.rs`, `tests/duplication.test.rs`

**Interfaces:**
- Consumes: Task 2's `Former`, `Lens::test_callees`.
- Produces: `functions::callee`, the callee chain of a call with a tagged template written as two
  backticks; a statement holding a test callee's call marked as joining no run, and the callbacks of
  the call and of the calls in its callee chain not walked.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/duplication.test.rs
<<<<<<< find
#[test]
fn finds_copies_in_scripts_and_vue_blocks_only() {
=======
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
            .map(|group| group.into_iter().map(|(_, first, last)| (first, last)).collect())
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
        [vec![("src/table.test.ts", 5, 6), ("src/table.test.ts", 9, 10)]]
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
fn finds_copies_in_scripts_and_vue_blocks_only() {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test duplication --test complexity
```

Expected: the two new tests fail: every test's body counts whatever the declarations say. Every
`complexity` test passes.

- [ ] **Step 3: Write the code**

```rust edit path=src/functions.rs
<<<<<<< find
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
=======
fn chain(expression: &Expression<'_>) -> Option<String> {
    written(expression, false)
}

/// A call's callee as a callee chain, as the duplication spec's terms write one: as `chain` writes it,
/// with a tagged template written as two backticks (duplication spec TUNE-5).
pub fn callee(expression: &Expression<'_>) -> Option<String> {
    written(expression, true)
}

/// An expression as `chain` writes it; with `tagged`, a tagged template is written as its tag and two
/// backticks, and without it, it is no chain, so complexity's names do not change.
fn written(expression: &Expression<'_>, tagged: bool) -> Option<String> {
    // The chain of an object with the text given after it, written the same way.
    let followed = |object: &Expression<'_>, text: &str| {
        written(object, tagged).map(|object| format!("{object}{text}"))
    };

    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::ThisExpression(_) => Some("this".to_owned()),
        Expression::Super(_) => Some("super".to_owned()),
        Expression::StaticMemberExpression(member) => {
            followed(&member.object, &format!(".{}", member.property.name))
        }
        Expression::ComputedMemberExpression(member) => followed(&member.object, "[]"),
        Expression::PrivateFieldExpression(member) => {
            followed(&member.object, &format!(".#{}", member.field.name))
        }
        Expression::CallExpression(call) => followed(&call.callee, "()"),
        Expression::TaggedTemplateExpression(template) if tagged => {
            followed(&template.tag, "``")
        }
        Expression::NewExpression(call) => {
            written(&call.callee, tagged).map(|callee| format!("new {callee}()"))
        }
        Expression::ChainExpression(chained) => match &chained.expression {
            ChainElement::CallExpression(call) => followed(&call.callee, "()"),
            ChainElement::TSNonNullExpression(wrapped) => written(&wrapped.expression, tagged),
            ChainElement::StaticMemberExpression(member) => {
                followed(&member.object, &format!(".{}", member.property.name))
            }
            ChainElement::ComputedMemberExpression(member) => followed(&member.object, "[]"),
            ChainElement::PrivateFieldExpression(member) => {
                followed(&member.object, &format!(".#{}", member.field.name))
            }
        },
        Expression::ParenthesizedExpression(wrapped) => written(&wrapped.expression, tagged),
        Expression::TSAsExpression(wrapped) => written(&wrapped.expression, tagged),
        Expression::TSSatisfiesExpression(wrapped) => written(&wrapped.expression, tagged),
        Expression::TSNonNullExpression(wrapped) => written(&wrapped.expression, tagged),
        Expression::TSTypeAssertion(wrapped) => written(&wrapped.expression, tagged),
        Expression::TSInstantiationExpression(wrapped) => written(&wrapped.expression, tagged),
        _ => None,
    }
}

/// The chain of an expression with the text given after it.
fn after(object: &Expression<'_>, text: &str) -> Option<String> {
    chain(object).map(|object| format!("{object}{text}"))
}
>>>>>>> replace
```

```rust edit path=src/forms.rs
<<<<<<< find
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, ClassElement, Program, PropertyKey, Statement,
    TemplateElement,
};
=======
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression, ClassElement, Program,
    PropertyKey, Statement, TemplateElement,
};
>>>>>>> replace
<<<<<<< find
use crate::duplication::Lens;
use crate::imports::Lines;
=======
use crate::duplication::Lens;
use crate::functions;
use crate::imports::Lines;
>>>>>>> replace
<<<<<<< find
        lists: Vec::new(),
        open: Vec::new(),
        dropped: Dropped::default(),
    };
=======
        lists: Vec::new(),
        open: Vec::new(),
        chained: false,
        dropped: Dropped::default(),
    };
>>>>>>> replace
<<<<<<< find
    /// The statements the walk is inside, by their list and their place in it.
    open: Vec<(usize, usize)>,
    /// The tokens the size leaves out.
    dropped: Dropped,
}
=======
    /// The statements the walk is inside, by their list and their place in it.
    open: Vec<(usize, usize)>,
    /// Whether the walk is in the callee chain of a test callee's call, where each call is part of the
    /// test's (spec COPY-3).
    chained: bool,
    /// The tokens the size leaves out.
    dropped: Dropped,
}
>>>>>>> replace
<<<<<<< find
    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
=======
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let test = self.chained
            || functions::callee(&call.callee)
                .is_some_and(|chain| self.lens.test_callees.contains(&chain));

        if !test {
            walk::walk_call_expression(self, call);

            return;
        }

        // A statement that holds a test callee's call joins no run (spec COPY-3).
        if !self.chained {
            for &(list, index) in &self.open {
                self.lists[list][index].joins = false;
            }
        }

        let kind = AstKind::CallExpression(self.alloc(call));
        let chained = self.chained;

        self.enter_node(kind);
        self.chained = true;
        self.visit_expression(&call.callee);
        self.chained = false;

        if let Some(arguments) = &call.type_arguments {
            self.visit_ts_type_parameter_instantiation(arguments);
        }

        // Nothing inside a test callee's callback is looked at (spec COPY-3).
        for argument in call.arguments.iter().filter(|argument| {
            !matches!(
                argument,
                Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_)
            )
        }) {
            self.visit_argument(argument);
        }

        self.chained = chained;
        self.leave_node(kind);
    }

    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --release --locked
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
```

Expected: every test of every file passes, `complexity`'s names unchanged; rustfmt changes nothing;
clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
yak stack check
git add src/forms.rs src/functions.rs tests/duplication.test.rs
git commit -m "feat(duplication): leave the bodies of tests out of the copies"
```

Expected: one commit.

### Task 4: Accepted groups

**Implements:** TUNE-6, QUIET-1, QUIET-2, QUIET-3, QUIET-4, SHOW-3, FIXT-1

**Test:** `cargo test --locked --test duplication`

**Files:**
- Modify: `src/duplication.rs`, `tests/duplication.test.rs`

**Interfaces:**
- Consumes: Task 2's `groups`, `Settings`.
- Produces: `duplication::Accepted`; `groups` returning the notices with the groups, each group
  marked accepted where an entry names its fingerprint and copies.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/duplication.test.rs
<<<<<<< find
use project::{Project, notices, printed};
=======
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
>>>>>>> replace
<<<<<<< find
#[test]
fn finds_copies_in_scripts_and_vue_blocks_only() {
=======
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
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test duplication
```

Expected: the two new tests fail: the accepted group is still listed, and no notice names the stale
entry.

- [ ] **Step 3: Write the code**

```rust edit path=src/duplication.rs
<<<<<<< find
//! the settings the declarations give, the runs of statements that repeat, the groups they make with
//! their sizes and fingerprints, and the document the command prints (duplication spec SAME-3, SAME-4,
//! SIZE-1, SIZE-2, TUNE-1 to TUNE-7, DUP-4, SHOW-1 to SHOW-5).
=======
//! the settings the declarations give, the runs of statements that repeat, the groups they make with
//! their sizes and fingerprints, the groups a person accepted, and the document the command prints
//! (duplication spec SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7, DUP-4, QUIET-1 to QUIET-4,
//! SHOW-1 to SHOW-5).
>>>>>>> replace
<<<<<<< find
/// The kind of the notice for a run with no `duplicationMinimum` (spec TUNE-1).
const NOT_CHECKED: &str = "duplication-not-checked";
=======
/// The kind of the notice for a run with no `duplicationMinimum` (spec TUNE-1).
const NOT_CHECKED: &str = "duplication-not-checked";

/// The kind of the notice for an accepted group no group matches (spec QUIET-2).
const NOT_FOUND: &str = "accepted-group-not-found";
>>>>>>> replace
<<<<<<< find
/// The settings of duplication, one of each for the whole run (spec TUNE-1 to TUNE-7).
=======
/// An entry of `duplicationAccepted`, with who states it (spec TUNE-6, QUIET-2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accepted {
    /// The entry: a group's fingerprint, `/`, and its number of copies.
    pub entry: String,
    /// Who states it: `pack` or `project`.
    pub source: &'static str,
}

/// The settings of duplication, one of each for the whole run (spec TUNE-1 to TUNE-7).
>>>>>>> replace
<<<<<<< find
    /// The entries of `duplicationAccepted`, the pack's before the project's.
    pub accepted: Vec<String>,
}
=======
    /// The entries of `duplicationAccepted`, the pack's before the project's.
    pub accepted: Vec<Accepted>,
}
>>>>>>> replace
<<<<<<< find
            accepted: [false, true]
                .into_iter()
                .flat_map(|project| {
                    joined(stacks, "duplicationAccepted", project).unwrap_or_default()
                })
                .collect(),
=======
            accepted: [(false, "pack"), (true, "project")]
                .into_iter()
                .flat_map(|(project, source)| {
                    joined(stacks, "duplicationAccepted", project)
                        .unwrap_or_default()
                        .into_iter()
                        .map(move |entry| Accepted { entry, source })
                })
                .collect(),
>>>>>>> replace
<<<<<<< find
        json!({
            "accepted": self.accepted,
            "ignore": self.ignored(),
=======
        json!({
            "accepted": self
                .accepted
                .iter()
                .map(|accepted| accepted.entry.as_str())
                .collect::<Vec<&str>>(),
            "ignore": self.ignored(),
>>>>>>> replace
<<<<<<< find
/// Every group of copies in the reading's own scripts and Vue components, each run as large as it goes,
/// sorted by its first copy; none where no `duplicationMinimum` is declared. A file declared data or
/// generated, and a type declaration file, holds no copy (spec COPY-4, SAME-3, SAME-4, SIZE-2).
pub fn groups(reading: &Reading) -> Vec<Group> {
    let settings = &reading.duplication;
    let Some(minimum) = settings.minimum_tokens else {
        return Vec::new();
    };
=======
/// Every group of copies in the reading's own scripts and Vue components, each run as large as it goes,
/// sorted by its first copy and marked accepted where an entry names its fingerprint and copies, with a
/// notice for each entry no group matches; none where no `duplicationMinimum` is declared. A file
/// declared data or generated, and a type declaration file, holds no copy (spec COPY-4, SAME-3, SAME-4,
/// SIZE-2, QUIET-1, QUIET-2).
pub fn groups(reading: &Reading) -> (Vec<Group>, Vec<Notice>) {
    let settings = &reading.duplication;
    let Some(minimum) = settings.minimum_tokens else {
        return (Vec::new(), Vec::new());
    };
>>>>>>> replace
<<<<<<< find
    let mut groups = widest(found);

    for group in &mut groups {
        group
            .copies
            .sort_by(|a, b| (&a.path, a.start_line).cmp(&(&b.path, b.start_line)));
    }

    groups.sort_by(|a, b| order(a).cmp(&order(b)));
    groups
}
=======
    let mut groups = widest(found);
    // Acceptance comes after the groups inside others are dropped, so an accepted group still hides
    // them (spec QUIET-1).
    let entries: BTreeSet<String> = groups.iter().map(accepting).collect();
    let accepted: BTreeSet<&str> = settings
        .accepted
        .iter()
        .map(|accepted| accepted.entry.as_str())
        .collect();
    let mut named = BTreeSet::new();
    let notices = settings
        .accepted
        .iter()
        .filter(|accepted| !entries.contains(&accepted.entry))
        .filter(|accepted| named.insert((accepted.entry.as_str(), accepted.source)))
        .map(|accepted| {
            Notice::new(
                NOT_FOUND,
                None,
                format!(
                    "The accepted group `{}` that the {} states matches no group found, so the entry can be \
                     removed.",
                    accepted.entry, accepted.source
                ),
            )
        })
        .collect();

    for group in &mut groups {
        group
            .copies
            .sort_by(|a, b| (&a.path, a.start_line).cmp(&(&b.path, b.start_line)));
        group.accepted = accepted.contains(accepting(group).as_str());
    }

    groups.sort_by(|a, b| order(a).cmp(&order(b)));

    (groups, notices)
}

/// The entry of `duplicationAccepted` that accepts a group: its fingerprint, `/`, and its number of
/// copies (spec TUNE-6).
fn accepting(group: &Group) -> String {
    format!("{}/{}", group.fingerprint, group.copies.len())
}
>>>>>>> replace
<<<<<<< find
    let listed: Vec<Value> = groups(&reading)
        .iter()
        .filter(|group| all || !group.accepted)
        .map(entry)
        .collect();

    reading.notices.extend(reading.duplication.notice());
=======
    let (groups, found) = groups(&reading);
    let listed: Vec<Value> = groups
        .iter()
        .filter(|group| all || !group.accepted)
        .map(entry)
        .collect();

    reading.notices.extend(found);
    reading.notices.extend(reading.duplication.notice());
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --release --locked
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
```

Expected: every test of every file passes; rustfmt changes nothing; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
yak stack check
git add src/duplication.rs tests/duplication.test.rs
git commit -m "feat(duplication): leave out the groups a person accepted"
```

Expected: one commit.

### Task 5: The guide

**Implements:** VERS-1

**Test:** `node --test scripts/contributing.test.mjs`

**Files:**
- Modify: `CONTRIBUTING.md`, `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: Tasks 1 to 4's command and keys.
- Produces: the guide naming the `duplication` command, and what duplication adds to interface
  version 1.

- [ ] **Step 1: Write the failing tests**

```js edit path=scripts/contributing.test.mjs
<<<<<<< find
    '`yak-analyzer complexity --hand-off <file> --all`',
=======
    '`yak-analyzer complexity --hand-off <file> --all`',
    '`yak-analyzer duplication` reads the project as `audit` does',
    '`yak-analyzer duplication --hand-off <file> --all`',
>>>>>>> replace
<<<<<<< find
    'its property bag, `complexity`',
    '`cyclomaticLimit`, `cognitiveLimit` and `over`',
  ]) {
=======
    'its property bag, `complexity`',
    '`cyclomaticLimit`, `cognitiveLimit` and `over`',
    // What duplication adds (duplication spec VERS-1).
    'Duplication adds to interface version 1 and breaks nothing in it',
    'the rule name `duplication`',
    '`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and',
    'The form of a run that a group\'s fingerprint is made from is part of the fingerprint',
  ]) {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
node --test scripts/contributing.test.mjs
```

Expected: the commands test and the interface version test fail, naming the missing text.

- [ ] **Step 3: Write the text**

```markdown edit path=CONTRIBUTING.md
<<<<<<< find
  person or an agent, and its name, options and document are a draft until the analyzer's stage 5
  releases it as an MCP tool.
=======
  person or an agent, and its name, options and document are a draft until the analyzer's stage 5
  releases it as an MCP tool.
- `yak-analyzer duplication` reads the project as `audit` does and prints one JSON document: the
  settings the declarations give duplication, each group of copies with its size, its copies and the
  fingerprint of its finding, and the notices.
  `yak-analyzer duplication --hand-off <file> --all` reads with the hand-off file's declarations and
  lists the groups a person accepted too. A command line it refuses exits 2. yak never runs it: it is
  for a person or an agent, and its name, options and document are a draft until the analyzer's
  stage 5.
>>>>>>> replace
<<<<<<< find
are fixed, and a change that breaks them raises the interface version.
=======
are fixed, and a change that breaks them raises the interface version.

Duplication adds to interface version 1 and breaks nothing in it: the rule name `duplication`; the
unit of `duplicationMinimum`, tokens, which no pack declared before; and the keys
`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and
`duplicationAccepted`.
The form of a run that a group's fingerprint is made from is part of the fingerprint scheme, and a
draft until the swap: from then, a change that breaks it raises the interface version, since it stops
every accepted group and recorded reason from matching.
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: every test passes.

- [ ] **Step 5: Commit**

```text
yak stack format
yak stack check
git add CONTRIBUTING.md scripts/contributing.test.mjs
git commit -m "docs(guide): name the duplication command and its keys"
```

Expected: one commit.
