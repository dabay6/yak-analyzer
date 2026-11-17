# Stage 2, part 4: the owner's go, YAML read within its bounds, and the keys fixed: implementation plan

**Goal:** every YAML file the analyzer reads, a CI system's command file or the root's
`pnpm-workspace.yaml`, is read one parser event at a time within bounds on its values, its text and
its nesting, with anchored values shared rather than copied, so no file of a branch can end the run
with nothing written (CI-6, CI-7); the comparison is rerun with that fix and judged (CURE-5); and,
on the owner's go of 2026-11-17, the declaration keys, rule names, fingerprints, `audit`'s options
and `interface`'s answer stop being drafts under interface version 1 (GO-1 to GO-3, FREEZE-1,
FREEZE-2).

**Architecture:** a new part, `src/yaml.rs`, takes yaml-rust2's parser events one at a time through
`Parser::next_token`, never through the recursive loader, and builds the values with `Rc`, so an
alias shares the node it names. It counts values, text and levels as the events come and stops at
the first past a bound. `src/command_files.rs` keeps only the walk of property paths over those
values; `src/entries.rs` and `src/packages.rs` both read through `yaml::read`, the second keeping
YAML's types for a plain scalar as the old loader gave them. Task 2 is CURE-5's rerun, judged in the
report folder. Task 3 writes in `CONTRIBUTING.md` what interface version 1 covers.

**Spec:** docs/specs/2026-11-15-dead-code-comparison-design.md

**Implements:** GO-1, GO-2, GO-3, FREEZE-1, FREEZE-2, CI-6, CI-7

The last of the item's four plans, on its own branch, `feat/comparison-4-decision`, made from
`main` after plan 3 merged as #32. Its fix is the spec's amendment of 2026-11-17 on bounds,
`amendment:2` on the record, approved by the owner. GO-1's final run, GO-2's judged list and the
owner's go (GO-3) were done while this plan was written; the decision is on the record in the
owner's words, with the counts and the bar that held, and the pack's declarations beside it.

## Global constraints

- Nothing committed names a machine or a private repository other than yakshaver: the judged list,
  the hand-off files and every run stay in the report folder,
  `.git/yak-reports/2026-11-16-dead-code-comparison/cmp/`, committed nowhere (JDG-6, GO-2).
- A YAML file over 1 MiB, or holding more than 100,000 values or 4 MiB of text once its aliases are
  followed, all its documents together, or nesting deeper than 1,000 levels, adds nothing; the
  count is kept while the file is read, and reading stops at the first event past a bound (CI-6).
- An alias shares the node it names; nothing the reader keeps is copied (CI-6).
- A command file past a bound, or not valid YAML, or holding an alias inside the value it names,
  gives one `command-file-not-read` notice and never stops the run (CI-3, CI-6).
- `pnpm-workspace.yaml` past a bound, or with an alias inside its own value, stops the run as one
  that cannot be read does, with the line naming why; a plain scalar in it keeps its YAML type
  (CI-7, reading PACK-3, PACK-6).
- After FREEZE-1, a change to a declaration key or its shape, a rule name, the fingerprint scheme,
  `audit`'s options or `interface`'s answer raises the interface version (FREEZE-1, design IF-6).
- Fixtures are made up and copy no code from the named repositories (CURE-3, CONTRIBUTING.md).

## The decisions this plan makes

1. **GO-1 was run while this plan was written,** after #32 merged, on each repository's
   default-branch head: only yakshaver had moved (`2a5406ff` to `17467a8f`), and its final run gave
   the same 48 shared findings; the other three are at their pinned commits. The judged list's
   "final run" section holds it; two yakshaver findings in changed files were judged afresh, true.
2. **GO-3 is recorded:** go, with the owner's caveat that it could still be a failure once yakshaver's
   `2026-11-11-stack-detection-in-sub-folders` lets the sub-folder packages be compared from the
   repository root. FREEZE-2's list went to the owner with the go; they took the session's
   recommendations: the bounds amendment, and no other change.
3. **A part of its own, `src/yaml.rs`,** reads both kinds of YAML file, since CI-7 asks the same
   reader for both; `src/command_files.rs` keeps the walk of property paths, which only command files
   need.
4. **A plain scalar and a quoted one are kept apart** (`Node::Plain`, `Node::Quoted`): a command file
   reads both as text (CI-6), and `pnpm-workspace.yaml` reads a plain one as the old loader typed it:
   `""`, `~` and `null` as a null, `true`, `True`, `TRUE`, `false`, `False`, `FALSE` as booleans, and
   decimal, `0x`, `0o` and `+` integers and floats (with `.inf` and `.nan`) as numbers (CI-7).
5. **The three refusals** are `Unread::Invalid` (with the parser's reason, which `pnpm-workspace.yaml`'s
   line keeps), `Unread::TooLarge` and `Unread::SelfAlias`; an alias that names no anchor of its
   document and none open around it is invalid. A command file's notices say: "<file> is not valid
   YAML, so its commands add no entry point."; "<file> is over 1 MiB, or holds more than 100,000
   values, 4 MiB of text or 1,000 levels of nesting once its aliases are followed, so its commands add
   no entry point."; "<file> holds an alias inside the value it names, which the analyzer does not
   follow, so its commands add no entry point." `pnpm-workspace.yaml`'s line puts the same words for
   the last two after "cannot be read (".
6. **The sharing fixture's red run is a measurement,** recorded as a ruling on Task 1: the chain run
   on the build before this plan (`a72b10d`), with its time and memory, since a reader that copies
   would exhaust memory rather than fail cleanly (CI-6).
7. **CURE-5's rerun** is on GO-1's final commits, which for three repositories are the pinned ones,
   and on the twelve branch-audit roots; its counts and outcome go on the record as a ruling on the
   plan, as plan 3's did, since the engine takes none on an unapplied live-check task.
8. **FREEZE-1's paragraph** goes in `CONTRIBUTING.md` under "The commands", after the commands and
   before what `graph` prints; it names each thing interface version 1 covers by its value. The
   README's status text is untouched (FREEZE-1). `src/declarations.rs`'s module comment, which says
   the keys are drafts, says they are fixed.

## Review focus

- **A document with no value** (an empty file, or `---` alone) gives no document from the parser;
  `pnpm-workspace.yaml` reads it as no workspace list, as before. No fixture holds an empty one.
- **A mapping with a key that is not a text** passes over the key and its value, in both kinds of
  file. No fixture holds it.
- **Duplicate keys** in `pnpm-workspace.yaml`: the last `packages` wins, as the old loader's mapping
  kept it. No fixture holds it.
- **An alias across documents** is invalid. No fixture holds it.

### Task 1: YAML read one event at a time, within its bounds

**Implements:** CI-6, CI-7, CURE-3

**Test:** `cargo test --release --locked --test entries --test packages`

**Files:**
- Create: `src/yaml.rs`
- Modify: `src/main.rs`, `src/command_files.rs`, `src/entries.rs`, `src/packages.rs`
- Test: `tests/entries.test.rs`, `tests/packages.test.rs`

**Interfaces:**
- Consumes: yaml-rust2 0.13's `parser::{Parser, Event}` and `scanner::TScalarStyle`;
  `Parser::next_token`, which takes one event; `packages::misshapen` (PACK-6).
- Produces: `yaml::{read, Node, Unread, MAX_BYTES}`; `command_files::{reached, folder}` over
  `yaml::Node`; the same entry points, notices and packages as before for every file within the
  bounds; Task 2 runs `audit` with the build.

- [ ] **Step 1: Write the failing tests**

The `entries` test: a compact block sequence 520,000 levels deep, a file past the value bound whose
syntax breaks after that point, an alias inside its own value, and a chain of 500 anchored mappings
over a list of 98,000 items, which is read with no notice.

```rust edit path=tests/entries.test.rs
<<<<<<< find
/// The uses a document holds that count from an entry point: each with the file whose exports are
=======
#[test]
fn a_command_file_nested_deep_aliasing_itself_or_past_a_bound_before_an_error_adds_nothing() {
    let project = Project::new("entries-command-bounds");
    // A chain of 500 anchored mappings over a list of 98,000 items: within every bound, but a reader
    // that copied each anchored value would copy about 49 million values.
    let mut chain = String::new();

    for level in 0..500 {
        chain.push_str(&format!("{}k{level}: &a{level}\n", " ".repeat(level)));
    }

    chain.push_str(&format!(
        "{}k500: [{}]\n",
        " ".repeat(500),
        ["x"; 98_000].join(", ")
    ));

    project
        .write("package.json", r#"{ "name": "root" }"#)
        .write(
            ".github/workflows/deep.yml",
            &format!("{}x\n", "- ".repeat(520_000)),
        )
        .write(
            ".github/workflows/late.yml",
            "a: &a [x, x, x, x, x, x, x, x, x, x]\n\
             b: &b [*a, *a, *a, *a, *a, *a, *a, *a, *a, *a]\n\
             c: &c [*b, *b, *b, *b, *b, *b, *b, *b, *b, *b]\n\
             d: &d [*c, *c, *c, *c, *c, *c, *c, *c, *c, *c]\n\
             e: &e [*d, *d, *d, *d, *d, *d, *d, *d, *d, *d]\n\
             z: [unclosed\n",
        )
        .write(".github/workflows/self.yml", "a: &a [x, *a]\n")
        .write(".github/workflows/chain.yml", &chain)
        .commit();

    let declarations = json!({
        "commandFiles": [{
            "file": ".github/workflows/*.yml",
            "commands": [["jobs", "*", "steps", "*", "run"]]
        }]
    });
    let file = project.hand_off(&project.root, &["."], stack(declarations, json!({})));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let said: Vec<(&str, &str)> = document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .map(|notice| {
            (
                notice["file"].as_str().expect("a file"),
                notice["sentence"].as_str().expect("a sentence"),
            )
        })
        .collect();
    let past = |file: &str| {
        format!(
            "{file} is over 1 MiB, or holds more than 100,000 values, 4 MiB of text or 1,000 levels of \
             nesting once its aliases are followed, so its commands add no entry point."
        )
    };

    // Nesting past 1,000 levels, and a value bound passed before a later error, give the bounds notice;
    // an alias inside its own value gives its own; the chain within the bounds gives none, and the run
    // ends with its document written (comparison spec CI-6).
    assert_eq!(
        said,
        [
            (
                ".github/workflows/deep.yml",
                past(".github/workflows/deep.yml").as_str()
            ),
            (
                ".github/workflows/late.yml",
                past(".github/workflows/late.yml").as_str()
            ),
            (
                ".github/workflows/self.yml",
                ".github/workflows/self.yml holds an alias inside the value it names, which the analyzer \
                 does not follow, so its commands add no entry point."
            ),
        ]
    );
}

/// The uses a document holds that count from an entry point: each with the file whose exports are
>>>>>>> replace
```

The `packages` test: a `pnpm-workspace.yaml` nested past the bound stops the run with the bound in
its line, and one whose `packages` is `~` names no workspace package.

```rust edit path=tests/packages.test.rs
<<<<<<< find
#[test]
fn stops_on_a_field_of_a_form_it_does_not_have() {
=======
#[test]
fn reads_pnpms_list_within_its_bounds_and_its_values_as_typed() {
    let deep = Project::new("packages-pnpm-deep");

    deep.write("package.json", "{}")
        .write(
            "pnpm-workspace.yaml",
            &format!("packages:\n{}x\n", "- ".repeat(520_000)),
        )
        .commit();

    // Nested past 1,000 levels, the list stops the run with the bound named, and the run ends with its
    // line written (comparison spec CI-7).
    assert_eq!(
        stopped(&deep.run(|_| {})),
        "yak-analyzer: the workspace's list pnpm-workspace.yaml cannot be read (it is over 1 MiB, or holds \
         more than 100,000 values, 4 MiB of text or 1,000 levels of nesting once its aliases are \
         followed); fix it, or declare it as data"
    );

    let null = Project::new("packages-pnpm-null");

    null.write("package.json", "{}")
        .write("pnpm-workspace.yaml", "packages: ~\n")
        .write("apps/web/package.json", r#"{ "name": "web" }"#)
        .commit();

    let document = null.graph();
    let workspace: Vec<&str> = document["packages"]
        .as_array()
        .expect("a list of packages")
        .iter()
        .filter(|package| package["workspace"] == json!(true))
        .map(|package| package["folder"].as_str().expect("a folder"))
        .collect();

    // A plain `~` is a null, as the YAML loader read it, so the list names no package (comparison spec
    // CI-7, spec PACK-3).
    assert_eq!(workspace, ["."]);
}

#[test]
fn stops_on_a_field_of_a_form_it_does_not_have() {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test entries --test packages
```

Expected: FAIL: the new `entries` test fails, since the 520,000-level file overflows the loader's
stack and ends the run, and the self-alias and late-error files give no notice or the wrong one; the
new `packages` test fails, since the deep list overflows the stack. The `~` case and the other tests
pass.

- [ ] **Step 3: Write the code**

The reader, shared by both kinds of YAML file.

```rust path=src/yaml.rs
//! The YAML files the analyzer reads, the root's `pnpm-workspace.yaml` and the files a CI system reads
//! commands from: each read one parser event at a time, within bounds kept while it is read, with an
//! anchored value shared by its aliases rather than copied (comparison spec CI-6, CI-7).

use std::collections::BTreeMap;
use std::rc::Rc;

use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

/// The largest YAML file the analyzer reads, in bytes (comparison spec CI-3, CI-6).
pub const MAX_BYTES: usize = 1024 * 1024;

/// The most values a file may hold, all its documents together, once its aliases are followed
/// (comparison spec CI-6).
const MAX_VALUES: usize = 100_000;

/// The most text, in bytes, a file may hold, all its documents together, once its aliases are followed
/// (comparison spec CI-6).
const MAX_TEXT: usize = 4 * MAX_BYTES;

/// The deepest a file may nest once its aliases are followed, the top value at level 1 (comparison
/// spec CI-6).
const MAX_LEVELS: usize = 1_000;

/// One value of a YAML file.
#[derive(Debug)]
pub enum Node {
    /// A scalar written plain, which YAML may read as a null, a boolean or a number.
    Plain(String),
    /// A scalar written in quotes or as a block: always a text.
    Quoted(String),
    /// A sequence.
    List(Vec<Rc<Self>>),
    /// A mapping, by its keys that are scalars; a key of another kind is passed over with its value.
    Map(Vec<(String, Rc<Self>)>),
}

impl Node {
    /// The scalar's text, however it was written; none for a sequence or a mapping.
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Plain(text) | Self::Quoted(text) => Some(text),
            Self::List(_) | Self::Map(_) => None,
        }
    }
}

/// Why a YAML file was not read (comparison spec CI-3, CI-6).
#[derive(Debug)]
pub enum Unread {
    /// It is not valid YAML, for the reason given.
    Invalid(String),
    /// It is over 1 MiB, or holds more than 100,000 values, 4 MiB of text or 1,000 levels once its
    /// aliases are followed.
    TooLarge,
    /// It holds an alias inside the value it names, which has no end.
    SelfAlias,
}

/// Every document of a YAML file's text, taken one parser event at a time and never through the
/// recursive loader. Values, their text and their levels are counted as the events come, an alias
/// counting those of the node it names where it stands, and reading stops at the first event past a
/// bound, so a later error never hides it (comparison spec CI-6).
pub fn read(text: &str) -> Result<Vec<Rc<Node>>, Unread> {
    if text.len() > MAX_BYTES {
        return Err(Unread::TooLarge);
    }

    let mut parser = Parser::new_from_str(text);
    let mut builder = Builder::default();

    loop {
        let (event, _) = parser
            .next_token()
            .map_err(|error| Unread::Invalid(error.to_string()))?;

        if event == Event::StreamEnd {
            return Ok(builder.documents);
        }

        builder.take(event)?;
    }
}

/// What a value weighs against the bounds: the values it holds, itself included, and the bytes of the
/// texts among them, keys included.
#[derive(Clone, Copy, Default)]
struct Weight {
    values: usize,
    bytes: usize,
}

impl Weight {
    /// A sequence or a mapping alone, before what it holds.
    const CONTAINER: Self = Self {
        values: 1,
        bytes: 0,
    };

    /// The two weights together.
    const fn plus(self, other: Self) -> Self {
        Self {
            values: self.values.saturating_add(other.values),
            bytes: self.bytes.saturating_add(other.bytes),
        }
    }
}

/// A sequence or a mapping being built.
enum Open {
    /// A sequence, with its items so far.
    List(Vec<Rc<Node>>),
    /// A mapping, with its entries so far and the key waiting for its value: a text, or none for a key
    /// of another kind, whose value is passed over.
    Map {
        entries: Vec<(String, Rc<Node>)>,
        key: Option<Option<String>>,
    },
}

/// A sequence or a mapping being built, with its anchor, the weight it holds so far, itself included,
/// and the most levels any value it holds has.
struct Frame {
    open: Open,
    anchor: usize,
    size: Weight,
    levels: usize,
}

/// Builds the documents from the parser's events.
#[derive(Default)]
struct Builder {
    documents: Vec<Rc<Node>>,
    frames: Vec<Frame>,
    anchors: BTreeMap<usize, (Rc<Node>, Weight, usize)>,
    counted: Weight,
}

impl Builder {
    /// Takes one event: a value begun, ended or named by an alias.
    fn take(&mut self, event: Event) -> Result<(), Unread> {
        match event {
            Event::DocumentStart => self.anchors.clear(),
            Event::Scalar(text, style, anchor, _) => {
                let weight = Weight {
                    values: 1,
                    bytes: text.len(),
                };

                self.count(weight)?;

                let node = if style == TScalarStyle::Plain {
                    Node::Plain(text)
                } else {
                    Node::Quoted(text)
                };

                self.finish(Rc::new(node), weight, 0, anchor);
            }
            Event::SequenceStart(anchor, _) => self.open(Open::List(Vec::new()), anchor)?,
            Event::MappingStart(anchor, _) => self.open(
                Open::Map {
                    entries: Vec::new(),
                    key: None,
                },
                anchor,
            )?,
            Event::SequenceEnd | Event::MappingEnd => {
                if let Some(frame) = self.frames.pop() {
                    let node = match frame.open {
                        Open::List(items) => Node::List(items),
                        Open::Map { entries, .. } => Node::Map(entries),
                    };

                    self.finish(Rc::new(node), frame.size, frame.levels + 1, frame.anchor);
                }
            }
            Event::Alias(anchor) => {
                let Some((node, size, levels)) = self
                    .anchors
                    .get(&anchor)
                    .map(|(node, size, levels)| (Rc::clone(node), *size, *levels))
                else {
                    return Err(if self.frames.iter().any(|frame| frame.anchor == anchor) {
                        Unread::SelfAlias
                    } else {
                        Unread::Invalid("an alias names no anchor of its document".to_owned())
                    });
                };

                if self.frames.len() + levels > MAX_LEVELS {
                    return Err(Unread::TooLarge);
                }

                self.count(size)?;
                self.finish(node, size, levels, 0);
            }
            _ => {}
        }

        Ok(())
    }

    /// Begins a sequence or a mapping, a level deeper than the one it lies in.
    fn open(&mut self, open: Open, anchor: usize) -> Result<(), Unread> {
        if self.frames.len() + 1 > MAX_LEVELS {
            return Err(Unread::TooLarge);
        }

        self.count(Weight::CONTAINER)?;
        self.frames.push(Frame {
            open,
            anchor,
            size: Weight::CONTAINER,
            levels: 0,
        });

        Ok(())
    }

    /// Counts a weight against the bounds, across the whole file.
    fn count(&mut self, weight: Weight) -> Result<(), Unread> {
        self.counted = self.counted.plus(weight);

        if self.counted.values > MAX_VALUES || self.counted.bytes > MAX_TEXT {
            Err(Unread::TooLarge)
        } else {
            Ok(())
        }
    }

    /// Puts a finished value where it belongs: under its anchor, shared, then in the sequence or
    /// mapping that holds it, or as a document.
    fn finish(&mut self, node: Rc<Node>, size: Weight, levels: usize, anchor: usize) {
        if anchor > 0 {
            self.anchors
                .insert(anchor, (Rc::clone(&node), size, levels));
        }

        let Some(frame) = self.frames.last_mut() else {
            self.documents.push(node);
            return;
        };

        frame.size = frame.size.plus(size);
        frame.levels = frame.levels.max(levels);

        match &mut frame.open {
            Open::List(items) => items.push(node),
            Open::Map { entries, key } => match key.take() {
                None => *key = Some(node.text().map(str::to_owned)),
                Some(Some(name)) => entries.push((name, node)),
                Some(None) => {}
            },
        }
    }
}
```

The walk of a command file's property paths, over the reader's values.

```rust path=src/command_files.rs
//! The files a CI system reads commands from: the command texts and folders a declaration's property
//! paths reach in one, over the values `yaml` reads (comparison spec CI-1, CI-6).

use crate::yaml::Node;

/// Each command text a property path reaches in a document, with the mappings it lies under, the one
/// that holds it last. `*` stands for every key of a mapping and every item of a sequence, and a
/// sequence of texts at the path's end is a command text each; every scalar is a text (comparison spec
/// CI-1, CI-6).
pub fn reached<'n>(document: &'n Node, path: &[String]) -> Vec<(&'n str, Vec<&'n Node>)> {
    let mut found = Vec::new();

    walk(document, path, &mut Vec::new(), &mut found);
    found
}

/// The folder text a command runs in: each path tried from the mapping that holds the command, then
/// from each mapping above it, nearest first, every path at one mapping before the next one up; the
/// first text found. A value of another kind is passed over (comparison spec CI-1).
pub fn folder<'n>(above: &[&'n Node], paths: &[Vec<String>]) -> Option<&'n str> {
    above
        .iter()
        .rev()
        .find_map(|mapping| paths.iter().find_map(|path| text_at(mapping, path)))
}

/// Walks a property path down from a node, keeping the mappings passed through.
fn walk<'n>(
    node: &'n Node,
    path: &[String],
    above: &mut Vec<&'n Node>,
    found: &mut Vec<(&'n str, Vec<&'n Node>)>,
) {
    let Some((key, rest)) = path.split_first() else {
        if let Some(text) = node.text() {
            found.push((text, above.clone()));
        } else if let Node::List(items) = node {
            for item in items {
                if let Some(text) = item.text() {
                    found.push((text, above.clone()));
                }
            }
        }

        return;
    };

    match node {
        Node::Map(entries) => {
            above.push(node);

            for (name, value) in entries {
                if key == "*" || name == key {
                    walk(value, rest, above, found);
                }
            }

            above.pop();
        }
        Node::List(items) if key == "*" => {
            for item in items {
                walk(item, rest, above, found);
            }
        }
        _ => {}
    }
}

/// The text a property path reaches from a node, where it reaches one.
fn text_at<'n>(node: &'n Node, path: &[String]) -> Option<&'n str> {
    let Some((key, rest)) = path.split_first() else {
        return node.text();
    };

    match node {
        Node::Map(entries) => entries
            .iter()
            .filter(|(name, _)| name == key)
            .find_map(|(_, value)| text_at(value, rest)),
        _ => None,
    }
}
```

```rust edit path=src/main.rs
<<<<<<< find
mod tsconfig;
mod uses;
=======
mod tsconfig;
mod uses;
mod yaml;
>>>>>>> replace
```

```rust edit path=src/entries.rs
<<<<<<< find
use crate::command_files::{self, Node, Unread};
=======
use crate::command_files;
>>>>>>> replace
<<<<<<< find
use crate::tsconfig::Configs;
=======
use crate::tsconfig::Configs;
use crate::yaml::{self, Node, Unread};
>>>>>>> replace
<<<<<<< find
    let read = match project.tree.read(path) {
        Ok(bytes) if bytes.len() > command_files::MAX_BYTES => Err(Unread::TooLarge),
        Ok(bytes) => String::from_utf8(bytes)
            .map_err(|_| Unread::Invalid)
            .and_then(|text| command_files::read(text.strip_prefix('\u{feff}').unwrap_or(&text))),
        Err(_) => Err(Unread::Invalid),
    };
    let documents = match read {
        Ok(documents) => documents,
        Err(why) => {
            let sentence = match why {
                Unread::Invalid => {
                    format!("{path} is not valid YAML, so its commands add no entry point.")
                }
                Unread::TooLarge => format!(
                    "{path} is over 1 MiB, or holds more than 100,000 values or 4 MiB of text once its \
                     aliases are followed, so its commands add no entry point."
                ),
            };
=======
    let read = match project.tree.read(path) {
        Ok(bytes) if bytes.len() > yaml::MAX_BYTES => Err(Unread::TooLarge),
        Ok(bytes) => String::from_utf8(bytes)
            .map_err(|_| Unread::Invalid("it is not valid UTF-8".to_owned()))
            .and_then(|text| yaml::read(text.strip_prefix('\u{feff}').unwrap_or(&text))),
        Err(error) => Err(Unread::Invalid(error.to_string())),
    };
    let documents = match read {
        Ok(documents) => documents,
        Err(why) => {
            let sentence = match why {
                Unread::Invalid(_) => {
                    format!("{path} is not valid YAML, so its commands add no entry point.")
                }
                Unread::TooLarge => format!(
                    "{path} is over 1 MiB, or holds more than 100,000 values, 4 MiB of text or 1,000 \
                     levels of nesting once its aliases are followed, so its commands add no entry \
                     point."
                ),
                Unread::SelfAlias => format!(
                    "{path} holds an alias inside the value it names, which the analyzer does not \
                     follow, so its commands add no entry point."
                ),
            };
>>>>>>> replace
```

`pnpm-workspace.yaml` through the same reader, a plain scalar typed as the old loader typed it.

```rust edit path=src/packages.rs
<<<<<<< find
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};
use yaml_rust2::{Yaml, YamlLoader};

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::notice::Notice;
use crate::stop::{Fault, Stop};
use crate::tree::Tree;
=======
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use serde_json::{Map, Value};

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::notice::Notice;
use crate::stop::{Fault, Stop};
use crate::tree::Tree;
use crate::yaml::{self, Node, Unread};
>>>>>>> replace
<<<<<<< find
    /// The workspace's list that pnpm reads, where the list holds one.
    pnpm: Option<Yaml>,
=======
    /// The workspace's list that pnpm reads, where the list holds one.
    pnpm: Option<Rc<Node>>,
>>>>>>> replace
<<<<<<< find
/// The workspace's list that pnpm reads, as YAML (spec PACK-3).
fn yaml(tree: &Tree) -> Result<Yaml, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the workspace's list {PNPM_LIST} cannot be read ({why}); fix it, or declare it as data"
        ))
    };
    let content = text(tree, PNPM_LIST).map_err(refused)?;
    let documents =
        YamlLoader::load_from_str(&content).map_err(|error| refused(error.to_string()))?;

    Ok(documents.into_iter().next().unwrap_or(Yaml::Null))
}
=======
/// The workspace's list that pnpm reads, its first document, read within the YAML bounds; an empty
/// file is a null (spec PACK-3; comparison spec CI-7).
fn yaml(tree: &Tree) -> Result<Rc<Node>, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the workspace's list {PNPM_LIST} cannot be read ({why}); fix it, or declare it as data"
        ))
    };
    let content = text(tree, PNPM_LIST).map_err(refused)?;
    let documents = yaml::read(&content).map_err(|unread| {
        refused(match unread {
            Unread::Invalid(why) => why,
            Unread::TooLarge => {
                "it is over 1 MiB, or holds more than 100,000 values, 4 MiB of text or \
                                 1,000 levels of nesting once its aliases are followed"
                    .to_owned()
            }
            Unread::SelfAlias => "it holds an alias inside the value it names".to_owned(),
        })
    })?;

    Ok(documents
        .into_iter()
        .next()
        .unwrap_or_else(|| Rc::new(Node::Plain(String::new()))))
}
>>>>>>> replace
<<<<<<< find
/// The globs of pnpm's `packages`.
fn pnpm_packages(document: &Yaml) -> Result<Vec<String>, Stop> {
    let form = "a list of globs";
    let packages = match document {
        Yaml::Null => return Ok(Vec::new()),
        Yaml::Hash(_) => &document["packages"],
        _ => {
            return Err(misshapen(
                "packages",
                PNPM_LIST,
                "a mapping that holds `packages`",
            ));
        }
    };

    match packages {
        Yaml::BadValue | Yaml::Null => Ok(Vec::new()),
        Yaml::Array(list) => list
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| misshapen("packages", PNPM_LIST, form))
            })
            .collect(),
        _ => Err(misshapen("packages", PNPM_LIST, form)),
    }
}
=======
/// The globs of pnpm's `packages`: none where the document or `packages` is a null or `packages` is
/// absent, the last `packages` where the mapping holds two (spec PACK-3; comparison spec CI-7).
fn pnpm_packages(document: &Node) -> Result<Vec<String>, Stop> {
    let form = "a list of globs";
    let packages = match document {
        Node::Map(entries) => entries
            .iter()
            .rev()
            .find(|(key, _)| key == "packages")
            .map(|(_, value)| &**value),
        node if is_null(node) => return Ok(Vec::new()),
        _ => {
            return Err(misshapen(
                "packages",
                PNPM_LIST,
                "a mapping that holds `packages`",
            ));
        }
    };

    match packages {
        None => Ok(Vec::new()),
        Some(node) if is_null(node) => Ok(Vec::new()),
        Some(Node::List(list)) => list
            .iter()
            .map(|entry| {
                glob_text(entry)
                    .map(str::to_owned)
                    .ok_or_else(|| misshapen("packages", PNPM_LIST, form))
            })
            .collect(),
        Some(_) => Err(misshapen("packages", PNPM_LIST, form)),
    }
}

/// Whether a value is a null as YAML reads one: a plain scalar that is empty, `~` or `null`, as the
/// YAML loader read it (comparison spec CI-7).
fn is_null(node: &Node) -> bool {
    matches!(node, Node::Plain(text) if matches!(text.as_str(), "" | "~" | "null"))
}

/// The text of a value that YAML reads as a text: a quoted scalar, or a plain one that is not a null, a
/// boolean or a number as the YAML loader read them (comparison spec CI-7).
fn glob_text(node: &Node) -> Option<&str> {
    match node {
        Node::Quoted(text) => Some(text),
        Node::Plain(text) if !typed(text) => Some(text),
        _ => None,
    }
}

/// Whether the YAML loader read a plain scalar as a null, a boolean or a number rather than a text.
fn typed(plain: &str) -> bool {
    let radix = |digits: Option<&str>, radix: u32| {
        digits.is_some_and(|digits| i64::from_str_radix(digits, radix).is_ok())
    };
    let special = [
        ".inf", ".Inf", ".INF", "+.inf", "+.Inf", "+.INF", "-.inf", "-.Inf", "-.INF", ".nan",
        ".NaN", ".NAN",
    ];

    radix(plain.strip_prefix("0x"), 16)
        || radix(plain.strip_prefix("0o"), 8)
        || plain
            .strip_prefix('+')
            .is_some_and(|number| number.parse::<i64>().is_ok())
        || matches!(
            plain,
            "" | "~" | "null" | "true" | "True" | "TRUE" | "false" | "False" | "FALSE"
        )
        || plain.parse::<i64>().is_ok()
        || special.contains(&plain)
        || (plain.bytes().any(|byte| byte.is_ascii_digit()) && plain.parse::<f64>().is_ok())
}
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test entries --test packages
```

Expected: PASS: 17 `entries` tests and 10 `packages` tests.

- [ ] **Step 5: Measure the sharing fixture's red run**

Write the `entries` test's chain file to a made-up repository with the same declaration, and run
`graph --hand-off` on it with the build of `a72b10d`, before this plan, and with this branch's,
recording each one's time and peak memory.

Expected: the earlier build copies each anchored value and takes far longer and far more memory, or
runs out of memory; this branch's reads it in about the time of any small file. Record both as a
ruling on Task 1 (decision 6).

- [ ] **Step 6: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two, 53 of them in the node
tests.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add src/yaml.rs src/main.rs src/command_files.rs src/entries.rs src/packages.rs tests/entries.test.rs tests/packages.test.rs
git commit -m "fix(yaml): read YAML one event at a time within its bounds"
```

Expected: one commit on `feat/comparison-4-decision`. `yak stack format` covers no Rust file here,
so `cargo fmt` formats, as plans 1 to 3 ruled.

### Task 2: the comparison run again with the fix (CURE-5)

**Implements:** GO-1, GO-2, FREEZE-2

**Check:** the owner reads the report folder's `cmp/judged.md`: its top's bar, the final run, the
CURE-5 rerun of Task 1's fix with every finding it added or removed judged, and the fix with its
fixtures among the fixes. A finding the rerun changes is told to the owner before FREEZE-1.

**Files:**
- None in this repository: the report folder is committed nowhere (JDG-6).

**Interfaces:**
- Consumes: the binary built from this branch after Task 1; the worktrees, fallow's runs and the
  hand-off files of plans 3 and 4.
- Produces: the judged list updated by the rerun, and its outcome on the record.

- [ ] **Step 1: Rerun the analyzer**

Copy the branch's `target/release/yak-analyzer.exe` into `yak-cmp-3/bin/`, and run
`audit --hand-off <file>` on yakshaver's final worktree and the other five roots, and
`audit --hand-off <file> --base <base>` on the twelve branch-audit roots.

Expected: each run exits 0, and each gives the findings plan 3's rerun and GO-1's final run gave.

- [ ] **Step 2: Judge and update the judged list**

Match each run against fallow's as before, judge every finding the fix added or removed, and add a
section to `cmp/judged.md` for the rerun and the fix.

Expected: no finding changes; the floor and the ceiling still hold, with case A set apart.

- [ ] **Step 3: Record the outcome**

Record a ruling on the plan that holds the rerun's counts and outcome, and that FREEZE-1 may follow.

Expected: the ruling is on the record.

### Task 3: the keys fixed under interface version 1

**Implements:** FREEZE-1, GO-3

**Test:** `node --test scripts/contributing.test.mjs`

**Files:**
- Modify: `CONTRIBUTING.md`, `src/declarations.rs`
- Test: `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: the declaration keys of `src/declarations.rs`, the rule names of `src/findings.rs`, the
  fingerprint key of `src/audit.rs`, `audit`'s options and `interface`'s answer, as they stand.
- Produces: the guide's paragraph saying what interface version 1 covers and that a change to any of
  it raises the number; yak relies on it from stage 3.

- [ ] **Step 1: Write the failing test**

```js edit path=scripts/contributing.test.mjs
<<<<<<< find
test("the gates it names run the scripts' tests as well as the launcher's (spec GATE-2)", () => {
=======
test('the guide says what interface version 1 covers, and that a change to it raises the number (comparison spec FREEZE-1)', () => {
  for (const said of [
    'Interface version 1 covers',
    '`entries`, `data`, `generated`, `internal`',
    '`configEntries`, `scriptCommands` and `commandFiles`',
    '`unused-file`, `unused-export`, `unused-type`,\n`unresolved-import` and `circular-dependency`',
    '`yakAnalyzer/v1`',
    'A change to any of them raises\nthe interface version',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
});

test("the gates it names run the scripts' tests as well as the launcher's (spec GATE-2)", () => {
>>>>>>> replace
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test scripts/contributing.test.mjs
```

Expected: FAIL: the new test fails, since the guide does not say this yet; the other three pass.

- [ ] **Step 3: Write the paragraph and the comment**

```markdown edit path=CONTRIBUTING.md
<<<<<<< find
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
=======
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

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
>>>>>>> replace
```

```rust edit path=src/declarations.rs
<<<<<<< find
//! The keys are drafts until the design spec's stage 2 ends. This part checks them, the shapes of the two
//! that describe what a tool does included (spec ENTRY-6a); the parts that read a project use them.
=======
//! The keys and their shapes are fixed under interface version 1 since stage 2 ended (comparison spec
//! FREEZE-1): a change to any raises the version. This part checks them, the shapes of the three that
//! describe what a tool does included (spec ENTRY-6a; comparison spec CI-1); the parts that read a
//! project use them.
>>>>>>> replace
```

- [ ] **Step 4: Run every gate**

```bash
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0, and the node tests report 54 passed.

- [ ] **Step 5: Commit**

```bash
yak stack format
git add CONTRIBUTING.md src/declarations.rs scripts/contributing.test.mjs
git commit -m "docs: say what interface version 1 covers"
```

Expected: one commit on `feat/comparison-4-decision`.
