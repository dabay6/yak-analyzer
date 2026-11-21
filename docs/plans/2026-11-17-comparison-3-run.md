# Stage 2, part 3: the comparison on the pinned commits, and the commands a CI workflow runs: implementation plan

**Goal:** the analyzer reads the commands a CI system's files hold as it reads a `package.json`'s
scripts, so a file only a workflow starts is no longer reported unused (CI-1 to CI-5); and the
comparison on the pinned commits, run while this plan was written, is run again with that fix and
judged, so the owner holds one judged list (CMP-1 to CURE-5, JDG-5b).

**Architecture:** a new part, `src/command_files.rs`, reads a command file as YAML through
yaml-rust2's event parser, counting values as they arrive so its bounds hold before any alias is
copied, and finds the texts and folders a declaration's property paths reach. `src/tools.rs` cuts a
command text that spans lines. `src/entries.rs` gathers every command the project runs, a script's
or a command file's, each with the folder it runs in, and both the reading of scripts (ENTRY-7) and
of configuration files named by an option (CONF-2) take their commands from that one list.
`src/declarations.rs` knows the tenth key. Task 2 is the owner's check of the comparison's judged
list, which stays in the item's report folder.

**Spec:** docs/specs/2026-11-15-dead-code-comparison-design.md

**Implements:** CMP-1 to CMP-7, JDG-1 to JDG-6, JDG-5a, JDG-5b, BAR-1 to BAR-4, BRA-1 to BRA-3, CURE-1 to CURE-5, CI-1 to CI-5

The third of the item's four plans, on its own branch, `feat/comparison-3-run`, made from `main`
after plan 2 merged as #30. Its fix is the spec's amendment of 2026-11-17, `amendment:1` on the
record, approved by the owner, written from what the comparison found. Plan 4 is the owner's
decision (GO-1 to GO-3, FREEZE-1, FREEZE-2).

## Global constraints

- Nothing committed names a machine or a private repository other than yakshaver: the judged list,
  the hand-off files, fallow's configurations and every run stay in the report folder,
  `.git/yak-reports/2026-11-16-dead-code-comparison/cmp/`, committed nowhere (JDG-6, CMP-2).
- Every run is on the pinned commits: yakshaver `2a5406ff`, bricklog `5a7e747a`, hsp-platform
  `bacfb9db`, prj-hspterminal `677cb740` (CMP-1).
- The analyzer runs on a clean worktree; fallow 3.39.0 on a second worktree of the same commit,
  installed from its lockfile with install scripts off (CMP-3, CMP-4, CMP-6).
- Only a pack states `commandFiles`, as only a pack states `configEntries` (CI-4).
- A command file over 1 MiB, or a document that expands past 100,000 values once its aliases are
  followed, adds nothing and gives one notice; the count is kept while the file is read (CI-3).
- A command file never stops the run; a `commandFiles` declaration of another shape does, naming the
  stack, the key and the field (CI-3, stage-1 ENTRY-6a).
- A word holding `$` names no file and gives no notice, in a script as in a command file (CI-2).
- Fixtures are made up and copy no code from the named repositories (CURE-3, CONTRIBUTING.md).

## The decisions this plan makes

1. **The comparison was run while this plan was written,** on 2026-11-17, with `main` at `62850fe`,
   and every finding judged; the judged list is `cmp/judged.md` in the report folder. It found one
   analyzer-only false finding, the workflow case this plan's Task 1 cures, and the AN-4a case the
   owner set apart (JDG-5b). Task 2 runs it again with this branch's build (CURE-5).
2. **One list of commands.** `entries::commands` gathers every command, a script's from its
   `package.json`'s folder and a command file's from its folder, and `scripted` and `named_configs`
   both read that list, so a file is an entry point whether a script or a workflow starts it, as
   CI-2 asks, with one reading.
3. **The source `graph` prints** for a file a command file's command starts is
   `{ "by": "pack", "file": <the command file>, "from": "commandFiles", "stack": <its stack> }`, as a
   `configEntries` declaration's names its configuration file (CI-4).
4. **One notice kind, `command-file-not-read`,** about the command file, for each of CI-3's three
   cases: "<file> is not valid YAML, so its commands add no entry point."; "<file> is over 1 MiB, or
   over 100,000 values once its aliases are followed, so its commands add no entry point."; and
   "The folder `<text>` that a command of <file> runs in is not one the analyzer can know, so the
   commands there add no entry point." A notice two declarations would give twice is given once.
5. **A word naming no listed file in a command file's command,** after an option a `configEntries`
   declaration lists, gives `script-config-not-listed` about the command file, with the sentence
   "A command in <file> names `<word>` after `<option>` as a configuration file of `<program>`, and
   no listed file is there, so it adds no entry point."; a script's sentence is as plan 2 wrote it.
6. **`Resolver::script_target` takes the folder** a command runs in, from the root, in place of the
   package, since a command file's command runs in no package's folder.
7. **The bound counts every value,** keys included, and an alias counts the size of the node it names
   each time it appears; the count starts again at each document, as the spec's bound is per
   document. A file read past the bound gives the notice and nothing from any of its documents.
8. **The comparison's hand-off files gain one `commandFiles` declaration** in the pack's part of every
   root, in the `dead-code-tools` stack (the amendment's preamble): GitHub Actions' workflows, with
   `commands` `[["jobs", "*", "steps", "*", "run"]]` and `folder` `[["working-directory"],
   ["defaults", "run", "working-directory"]]`, written in the judged list under CURE-2.
9. **The counts and the outcome go on the record** as a ruling on Task 2, since a live-check task has
   no test command, as the reading item and the live check found.
10. **The worktrees stay until the owner's decision** (GO-3): plan 4's final run and FREEZE-2's rerun
    use the same tools, and CMP-4 removes them after GO-3.
11. **No README or CONTRIBUTING change.** Neither describes the declaration keys' shapes today;
    FREEZE-1 writes them down (plan 4).

## Review focus

- **A workflow in a sub-folder root** is never read, since the root holds no `.github` (CI-1). The
  comparison's sub-folder roots show it; no fixture holds it.
- **A `folder` path that reaches a text with a wildcard,** as `packages/*`, is joined as written and
  names a folder no file lies in, so its commands add nothing, quietly. No named repository has one.
- **A command file matched from two detection folders** is read once, from the deepest, as a
  configuration file is. The fixture's workflows lie under the root's detection folder only.
- **A key that is not a text** in a command file (`? [a, b] : x`) is passed over with its value. No
  fixture holds it.
- **`\r\n` line ends** in a command text: `\` before `\r\n` joins lines; an unquoted `\r` is white
  space. The fixture is written with `\n`.

### Task 1: the commands a CI workflow runs, read as a script's are

**Implements:** CI-1, CI-2, CI-3, CI-4, CURE-3

**Test:** `cargo test --release --locked --test entries --test declarations`

**Files:**
- Create: `src/command_files.rs`
- Modify: `src/main.rs`, `src/tools.rs`, `src/resolve.rs`, `src/entries.rs`, `src/declarations.rs`
- Test: `tests/entries.test.rs`, `tests/declarations.test.rs`

**Interfaces:**
- Consumes: yaml-rust2 0.13's `parser::{Parser, Event, EventReceiver}`; `tools::commands` and
  `tools::is_assignment` (stage-1 ENTRY-7); the `deepest`, `inside`, `resolved`, `folder_path`,
  `manifest_path` and `texts` helpers of `src/entries.rs`; `Options::values` (CONF-2).
- Produces: `command_files::{read, reached, folder, Node, Unread, MAX_BYTES}`;
  `tools::command_lines`; `Resolver::script_target(folder, word)`; `Entries::points` holding each
  file a command file's command starts, with the `commandFiles` source; the
  `command-file-not-read` notice; the tenth key, `commandFiles`. `graph` and `audit` read them
  unchanged; Task 2 runs `audit` with them.

- [ ] **Step 1: Write the failing tests**

The `entries` test: a workflow whose steps start files from a job's and the workflow's default
folder and their own, across a `run: |` block with a comment, a continued line, a quote spanning a
line end and a stray quote; a configuration file named by `--config` in a step; a folder the
analyzer cannot know; a file of two documents; one not valid YAML; and one whose aliases expand past
the bound, which a reader that copies first would never finish.

```rust edit path=tests/entries.test.rs
<<<<<<< find
//! exports, and the files the entry points reach (spec ENTRY-1 to ENTRY-8, USE-3, USE-6, CMD-4, FIX-1,
//! FIX-2; comparison spec CONF-1 to CONF-3).
=======
//! exports, and the files the entry points reach (spec ENTRY-1 to ENTRY-8, USE-3, USE-6, CMD-4, FIX-1,
//! FIX-2; comparison spec CONF-1 to CONF-3, CI-1 to CI-4).
>>>>>>> replace
<<<<<<< find
/// The uses a document holds that count from an entry point: each with the file whose exports are
=======
/// The source a command file's command gives each file it starts (comparison spec CI-4).
fn commanded(file: &str) -> Value {
    json!({ "by": "pack", "file": file, "from": "commandFiles", "stack": "typescript" })
}

#[test]
fn a_ci_workflows_commands_start_files_as_a_scripts_do() {
    let project = Project::new("entries-command-files");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "private": true, "workspaces": ["packages/*"] }"#,
        )
        .write("packages/web/package.json", r#"{ "name": "web" }"#)
        .write(
            ".github/workflows/ci.yml",
            r#"on: push
defaults:
  run:
    working-directory: tools
jobs:
  build:
    defaults:
      run:
        working-directory: ./packages/web/
    steps:
      - uses: actions/checkout@v4
      - run: node scripts/build.mjs
      - run: node scripts/mapped.mjs
        working-directory:
          - not a text
      - name: Several
        working-directory: .
        run: |
          node scripts/a.mjs # node scripts/commented.mjs
          node \
            scripts/b.mjs --flag
          echo "the next line is quoted
          node scripts/hidden.mjs"
          # don't run the next file
          node scripts/c.mjs
          echo it's here
          node scripts/after.mjs
          node scripts/missing.mjs $RUNNER_TEMP/x.mjs
      - run: vitest --config vitest.e2e.config.ts
        working-directory: packages/web
      - run: node scripts/never.mjs
        working-directory: ${{ matrix.dir }}
  lint:
    steps:
      - run:
          - node list-a.mjs
          - node list-b.mjs
      - run: node check.mjs
"#,
        )
        .write(
            ".github/workflows/two.yml",
            "jobs:\n  one:\n    steps:\n      - run: node first.mjs\n---\n\
             jobs:\n  two:\n    steps:\n      - run: node second.mjs\n",
        )
        .write(".github/workflows/broken.yml", "jobs: [unclosed\n")
        .write(
            ".github/workflows/bomb.yaml",
            "a: &a [x, x, x, x, x, x, x, x, x, x]\n\
             b: &b [*a, *a, *a, *a, *a, *a, *a, *a, *a, *a]\n\
             c: &c [*b, *b, *b, *b, *b, *b, *b, *b, *b, *b]\n\
             d: &d [*c, *c, *c, *c, *c, *c, *c, *c, *c, *c]\n\
             e: &e [*d, *d, *d, *d, *d, *d, *d, *d, *d, *d]\n\
             f: &f [*e, *e, *e, *e, *e, *e, *e, *e, *e, *e]\n\
             g: &g [*f, *f, *f, *f, *f, *f, *f, *f, *f, *f]\n\
             h: &h [*g, *g, *g, *g, *g, *g, *g, *g, *g, *g]\n\
             i: [*h, *h, *h, *h, *h, *h, *h, *h, *h, *h]\n",
        )
        .write(
            "packages/web/vitest.e2e.config.ts",
            "export default { test: { include: ['e2e/**/*.e2e.ts'] } };\n",
        )
        .write("packages/web/e2e/x.e2e.ts", "")
        .write("packages/web/scripts/build.mjs", "")
        .write("packages/web/scripts/mapped.mjs", "")
        .write("scripts/a.mjs", "")
        .write("scripts/b.mjs", "")
        .write("scripts/c.mjs", "")
        .write("scripts/after.mjs", "")
        .write("scripts/commented.mjs", "")
        .write("scripts/hidden.mjs", "")
        .write("scripts/never.mjs", "")
        .write("tools/check.mjs", "")
        .write("tools/list-a.mjs", "")
        .write("tools/list-b.mjs", "")
        .write("first.mjs", "")
        .write("second.mjs", "")
        .commit();

    let declarations = json!({
        "commandFiles": [{
            "file": ".github/workflows/*.{yml,yaml}",
            "commands": [["jobs", "*", "steps", "*", "run"]],
            "folder": [["working-directory"], ["defaults", "run", "working-directory"]]
        }],
        "configEntries": [{
            "file": "vitest.config.{ts,mts}",
            "property": ["test", "include"],
            "value": "globs",
            "relativeTo": "config",
            "default": [],
            "options": { "program": "vitest", "names": ["--config", "-c"], "relativeTo": "script" }
        }]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/web"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let ci = commanded(".github/workflows/ci.yml");
    let two = commanded(".github/workflows/two.yml");

    // Each file a step starts with `node` is an entry point, found from the step's own folder, else its
    // job's default, else the workflow's; a comment, a quoted text spanning a line end and a word holding
    // `$` start nothing, a continued line is one command, and a stray quote loses only its own line. A
    // configuration file named by `--config` in a step is read from the step's folder (comparison spec
    // CI-1, CI-2).
    assert_eq!(
        entries(&document),
        [
            ("first.mjs", vec![two.clone()]),
            (
                "packages/web/e2e/x.e2e.ts",
                vec![configured(Some("packages/web/vitest.e2e.config.ts"))]
            ),
            ("packages/web/scripts/build.mjs", vec![ci.clone()]),
            ("packages/web/scripts/mapped.mjs", vec![ci.clone()]),
            ("packages/web/vitest.e2e.config.ts", vec![configured(None)]),
            ("scripts/a.mjs", vec![ci.clone()]),
            ("scripts/after.mjs", vec![ci.clone()]),
            ("scripts/b.mjs", vec![ci.clone()]),
            ("scripts/c.mjs", vec![ci.clone()]),
            ("second.mjs", vec![two]),
            ("tools/check.mjs", vec![ci.clone()]),
            ("tools/list-a.mjs", vec![ci.clone()]),
            ("tools/list-b.mjs", vec![ci]),
        ]
    );

    // A file past the bound, one that is not valid YAML, and a folder the analyzer cannot know each give
    // one notice and add nothing; the run goes on (comparison spec CI-3).
    let said: Vec<(&str, &str)> = document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .map(|notice| {
            (
                notice["file"].as_str().expect("a file"),
                notice["kind"].as_str().expect("a kind"),
            )
        })
        .collect();

    assert_eq!(
        said,
        [
            (".github/workflows/bomb.yaml", "command-file-not-read"),
            (".github/workflows/broken.yml", "command-file-not-read"),
            (".github/workflows/ci.yml", "command-file-not-read"),
        ]
    );
    assert_eq!(
        document["notices"][2]["sentence"],
        json!(
            "The folder `${{ matrix.dir }}` that a command of .github/workflows/ci.yml runs in is not one the \
             analyzer can know, so the commands there add no entry point."
        )
    );
}

/// The uses a document holds that count from an entry point: each with the file whose exports are
>>>>>>> replace
```

The `declarations` test: `commandFiles` is usable in its shape, refused when a project states it, and
refused in every other shape.

```rust edit path=tests/declarations.test.rs
<<<<<<< find
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4, ENTRY-6a; comparison spec CONF-1).
=======
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4, ENTRY-6a; comparison spec CONF-1, CI-1, CI-3, CI-4).
>>>>>>> replace
<<<<<<< find
        "scriptCommands": ["cross-env", "tsx"]
    });
=======
        "scriptCommands": ["cross-env", "tsx"],
        "commandFiles": [
            {
                "file": ".github/workflows/*.{yml,yaml}",
                "commands": [["jobs", "*", "steps", "*", "run"]],
                "folder": [["working-directory"], ["defaults", "run", "working-directory"]]
            },
            { "file": ".gitlab-ci.yml", "commands": [["*", "script"], ["*", "before_script"]] }
        ]
    });
>>>>>>> replace
<<<<<<< find
         cognitiveLimit, duplicationMinimum, configEntries, scriptCommands; change it in the pack"
    );
=======
         cognitiveLimit, duplicationMinimum, configEntries, scriptCommands, commandFiles; change it in the \
         pack"
    );
>>>>>>> replace
<<<<<<< find
#[test]
fn refuses_a_glob_or_a_path_it_cannot_read_or_that_leaves_its_folder() {
=======
#[test]
fn refuses_a_command_file_declaration_a_project_states_or_of_another_shape() {
    assert_eq!(
        refused(
            "declarations-command-files-project",
            json!({}),
            json!({ "commandFiles": ["ci.yml"] })
        ),
        "yak-analyzer: cannot use the declaration `commandFiles` that the project states for the stack \
         `typescript`: only a pack may state this key; change it in the project's own declarations"
    );

    let usable = || json!({ "file": "ci.yml", "commands": [["jobs", "*", "run"]] });
    let changed = |change: &dyn Fn(&mut Value)| {
        let mut entry = usable();

        change(&mut entry);

        json!({ "commandFiles": [entry] })
    };
    let paths = "must be a list of property paths, one or more, each a list of keys, one or more";
    let cases: [(Value, String); 10] = [
        (
            json!({ "commandFiles": {} }),
            ": its value must be a list of objects; ".to_owned(),
        ),
        (
            json!({ "commandFiles": [3] }),
            ": its entry 1 must be an object; ".to_owned(),
        ),
        (
            changed(&|entry| entry["extra"] = json!(1)),
            ": its entry 1's field `extra` is not one it may hold; ".to_owned(),
        ),
        (
            changed(&|entry| {
                entry.as_object_mut().expect("an object").remove("commands");
            }),
            ": its entry 1's field `commands` is missing; ".to_owned(),
        ),
        (
            changed(&|entry| entry["file"] = json!(4)),
            ": its entry 1's field `file` must be a glob; ".to_owned(),
        ),
        (
            changed(&|entry| entry["file"] = json!("../ci/*.yml")),
            ": its entry 1's field `file` cannot be used: the glob `../ci/*.yml` is absolute or steps out"
                .to_owned(),
        ),
        (
            changed(&|entry| entry["commands"] = json!([])),
            format!(": its entry 1's field `commands` {paths}; "),
        ),
        (
            changed(&|entry| entry["commands"] = json!([[]])),
            format!(": its entry 1's field `commands` {paths}; "),
        ),
        (
            changed(&|entry| entry["commands"] = json!(["run"])),
            format!(": its entry 1's field `commands` {paths}; "),
        ),
        (
            changed(&|entry| entry["folder"] = json!([["dir", 1]])),
            format!(": its entry 1's field `folder` {paths}; "),
        ),
    ];

    for (index, (pack, expected)) in cases.into_iter().enumerate() {
        let line = refused(
            &format!("declarations-command-files-{index}"),
            pack,
            json!({}),
        );

        assert!(line.contains(&expected), "{line}");
    }
}

#[test]
fn refuses_a_glob_or_a_path_it_cannot_read_or_that_leaves_its_folder() {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test entries --test declarations
```

Expected: FAIL: the new `entries` test, the usable-keys test and the new refusal test fail, since the
analyzer knows no `commandFiles` key and stops the run on it; the names test fails, since its list of
keys lacks the tenth. The other tests pass.

- [ ] **Step 3: Write the code**

The new part, which reads a command file within its bounds and walks its property paths.

```rust path=src/command_files.rs
//! The files a CI system reads commands from: each read as YAML within fixed bounds, and the command
//! texts and folders a declaration's property paths reach in it (comparison spec CI-1 to CI-3).

use std::collections::BTreeMap;

use yaml_rust2::parser::{Event, EventReceiver, Parser};

/// The largest command file the analyzer reads, in bytes (comparison spec CI-3).
pub const MAX_BYTES: usize = 1024 * 1024;

/// The most values one document may hold once its aliases are followed (comparison spec CI-3).
const MAX_VALUES: usize = 100_000;

/// One value of a command file.
#[derive(Clone, Debug)]
pub enum Node {
    /// A text: every scalar, as written.
    Text(String),
    /// A sequence.
    List(Vec<Self>),
    /// A mapping, by its keys that are texts; a key of another kind is passed over with its value.
    Map(Vec<(String, Self)>),
}

/// Why a command file adds nothing (comparison spec CI-3).
#[derive(Debug)]
pub enum Unread {
    /// It is not valid YAML, or not UTF-8.
    Invalid,
    /// It is over 1 MiB, or a document of it holds more than 100,000 values once its aliases are
    /// followed.
    TooLarge,
}

/// Every document of a command file's text. Values are counted as the parser hands them over, and an
/// alias counts the values of the node it names each time it appears, so reading stops at the bound
/// before any copy is made (comparison spec CI-3).
pub fn read(text: &str) -> Result<Vec<Node>, Unread> {
    let mut builder = Builder::default();

    Parser::new_from_str(text)
        .load(&mut builder, true)
        .map_err(|_| Unread::Invalid)?;

    if builder.over {
        Err(Unread::TooLarge)
    } else {
        Ok(builder.documents)
    }
}

/// Each command text a property path reaches in a document, with the mappings it lies under, the one
/// that holds it last. `*` stands for every key of a mapping and every item of a sequence, and a
/// sequence of texts at the path's end is a command text each (comparison spec CI-1).
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
        match node {
            Node::Text(text) => found.push((text.as_str(), above.clone())),
            Node::List(items) => {
                for item in items {
                    if let Node::Text(text) = item {
                        found.push((text.as_str(), above.clone()));
                    }
                }
            }
            Node::Map(_) => {}
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
        return match node {
            Node::Text(text) => Some(text.as_str()),
            _ => None,
        };
    };

    match node {
        Node::Map(entries) => entries
            .iter()
            .filter(|(name, _)| name == key)
            .find_map(|(_, value)| text_at(value, rest)),
        _ => None,
    }
}

/// A sequence or a mapping being built.
enum Open {
    /// A sequence, with its items so far.
    List(Vec<Node>),
    /// A mapping, with its entries so far and the key waiting for its value: a text, or none for a key
    /// of another kind, whose value is passed over.
    Map {
        entries: Vec<(String, Node)>,
        key: Option<Option<String>>,
    },
}

/// A sequence or a mapping being built, with its anchor and the values it holds so far, itself
/// included.
struct Frame {
    open: Open,
    anchor: usize,
    size: usize,
}

/// Builds the documents from the parser's events, counting values as they come.
#[derive(Default)]
struct Builder {
    documents: Vec<Node>,
    frames: Vec<Frame>,
    anchors: BTreeMap<usize, (Node, usize)>,
    counted: usize,
    over: bool,
}

impl Builder {
    /// Counts values against the bound; false once it is passed, after which nothing more is built.
    fn count(&mut self, values: usize) -> bool {
        self.counted = self.counted.saturating_add(values);
        self.over = self.counted > MAX_VALUES;

        !self.over
    }

    /// Puts a finished value where it belongs: under its anchor, then in the sequence or mapping that
    /// holds it, or as a document.
    fn finish(&mut self, node: Node, size: usize, anchor: usize) {
        if anchor > 0 {
            self.anchors.insert(anchor, (node.clone(), size));
        }

        let Some(frame) = self.frames.last_mut() else {
            self.documents.push(node);
            return;
        };

        frame.size += size;

        match &mut frame.open {
            Open::List(items) => items.push(node),
            Open::Map { entries, key } => match key.take() {
                None => {
                    *key = Some(match node {
                        Node::Text(text) => Some(text),
                        _ => None,
                    });
                }
                Some(Some(name)) => entries.push((name, node)),
                Some(None) => {}
            },
        }
    }
}

impl EventReceiver for Builder {
    fn on_event(&mut self, event: Event) {
        if self.over {
            return;
        }

        match event {
            Event::DocumentStart => {
                self.counted = 0;
                self.anchors.clear();
            }
            Event::Scalar(text, _, anchor, _) => {
                if self.count(1) {
                    self.finish(Node::Text(text), 1, anchor);
                }
            }
            Event::SequenceStart(anchor, _) => {
                if self.count(1) {
                    self.frames.push(Frame {
                        open: Open::List(Vec::new()),
                        anchor,
                        size: 1,
                    });
                }
            }
            Event::MappingStart(anchor, _) => {
                if self.count(1) {
                    self.frames.push(Frame {
                        open: Open::Map {
                            entries: Vec::new(),
                            key: None,
                        },
                        anchor,
                        size: 1,
                    });
                }
            }
            Event::SequenceEnd | Event::MappingEnd => {
                if let Some(frame) = self.frames.pop() {
                    let node = match frame.open {
                        Open::List(items) => Node::List(items),
                        Open::Map { entries, .. } => Node::Map(entries),
                    };

                    self.finish(node, frame.size, frame.anchor);
                }
            }
            Event::Alias(anchor) => {
                let size = self.anchors.get(&anchor).map_or(0, |(_, size)| *size);

                if size > 0 && self.count(size) {
                    let node = self.anchors[&anchor].0.clone();

                    self.finish(node, size, 0);
                }
            }
            _ => {}
        }
    }
}
```

```rust edit path=src/main.rs
<<<<<<< find
mod cli;
mod declarations;
=======
mod cli;
mod command_files;
mod declarations;
>>>>>>> replace
```

Cutting a command text that spans lines; a script is cut as before.

```rust edit path=src/tools.rs
<<<<<<< find
/// The commands of a script, each as its words, quotes removed. Quotes are found first, from the left,
/// and nothing quoted splits; then the script is cut into commands at unquoted `&&`, `||`, `;`, `|` and
/// `&`, and each command into words at unquoted white space (spec ENTRY-7).
pub fn commands(script: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    let mut rest = script;

    while let Some(character) = rest.chars().next() {
        if let Some(open) = quote {
            if character == open {
                quote = None;
            } else {
                word.push(character);
            }

            rest = &rest[character.len_utf8()..];
            continue;
        }

        if let Some(separator) = SEPARATORS
=======
/// The commands of a script, each as its words, quotes removed. Quotes are found first, from the left,
/// and nothing quoted splits; then the script is cut into commands at unquoted `&&`, `||`, `;`, `|` and
/// `&`, and each command into words at unquoted white space (spec ENTRY-7).
pub fn commands(script: &str) -> Vec<Vec<String>> {
    cut(script, false)
}

/// The commands of a command file's command text, cut as a script's are, with what text that spans
/// lines adds: an unquoted `#` that begins a word starts a comment to the line's end; an unquoted `\`
/// just before a line end joins the two lines; any other unquoted line end cuts commands; and a quote is
/// closed by the next matching quote anywhere later, across line ends, while one with no match quotes
/// the rest of its own line only (comparison spec CI-2).
pub fn command_lines(text: &str) -> Vec<Vec<String>> {
    cut(text, true)
}

/// The commands of a text, each as its words, quotes removed: read as a script is, or, with `lines`, as
/// a command file's command text is.
fn cut(text: &str, lines: bool) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    // Whether the open quote has no match in the rest of the text, and so ends at its line's end.
    let mut to_line_end = false;
    let mut rest = text;

    while let Some(character) = rest.chars().next() {
        if let Some(open) = quote {
            if character == open {
                quote = None;
                rest = &rest[character.len_utf8()..];
                continue;
            }

            if !(to_line_end && character == '\n') {
                word.push(character);
                rest = &rest[character.len_utf8()..];
                continue;
            }

            // A quote with no match ends at its line's end, which is then read as an unquoted one.
            quote = None;
        }

        if lines {
            if character == '#' && !started {
                // A comment runs to the line's end; the line end itself is still read.
                rest = rest.find('\n').map_or("", |end| &rest[end..]);
                continue;
            }

            if let Some(after) = rest
                .strip_prefix("\\\r\n")
                .or_else(|| rest.strip_prefix("\\\n"))
            {
                rest = after;
                continue;
            }

            if character == '\n' {
                end_word(&mut words, &mut word, &mut started);
                commands.push(std::mem::take(&mut words));
                rest = &rest[1..];
                continue;
            }
        }

        if let Some(separator) = SEPARATORS
>>>>>>> replace
<<<<<<< find
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            started = true;
=======
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            to_line_end = lines && !rest[1..].contains(character);
            started = true;
>>>>>>> replace
```

```rust edit path=src/resolve.rs
<<<<<<< find
    /// The files a word of a package's script names, from the package's folder: the path itself, or the
    /// source of build output, mapped back by the package the path lies in, which may be another
    /// (spec ENTRY-7, ENTRY-7a).
    pub fn script_target(&self, package: &Package, word: &str) -> Option<Vec<String>> {
        let path = joined(folder_path(&package.folder), word)?;
=======
    /// The files a word of a command names, from the folder, from the root, the command runs in: the
    /// path itself, or the source of build output, mapped back by the package the path lies in (spec
    /// ENTRY-7, ENTRY-7a; comparison spec CI-2).
    pub fn script_target(&self, folder: &str, word: &str) -> Option<Vec<String>> {
        let path = joined(folder, word)?;
>>>>>>> replace
```

One list of commands, a script's or a command file's, read by both `scripted` and `named_configs`.

```rust edit path=src/entries.rs
<<<<<<< find
use crate::files::{File, Kind};
=======
use crate::command_files::{self, Node, Unread};
use crate::files::{File, Kind};
>>>>>>> replace
<<<<<<< find
    scripted(project, &mut entries);
    declared(project, &mut entries, notices)?;
    configured(project, &mut entries, notices);
=======
    let commands = commands(project, notices);

    scripted(project, &commands, &mut entries);
    declared(project, &mut entries, notices)?;
    configured(project, &commands, &mut entries, notices);
>>>>>>> replace
<<<<<<< find
/// The entry points the scripts of each `package.json` start: a file a word of a command names, from
/// the folder of that `package.json`, after `node` or a program a `scriptCommands` declaration names,
/// as RES-3 and MAP-1 find it. Words of the form `NAME=value` before the program are passed over, an
/// option's own name is never a file, and `--option=value` is tried by its value. A declared program
/// counts in every package's scripts (spec ENTRY-7).
fn scripted(project: &Project<'_>, entries: &mut Entries) {
    let mut programs: BTreeSet<&str> = BTreeSet::from(["node"]);

    if let Some(handed) = project.handed {
        for stack in &handed.stacks {
            programs.extend(texts(stack.pack.get("scriptCommands")));
        }
    }

    for package in &project.packages.list {
        let manifest = manifest_path(&package.folder);

        for script in &package.scripts {
            for command in tools::commands(script) {
                let mut words = command.iter().skip_while(|word| tools::is_assignment(word));

                if !words
                    .next()
                    .is_some_and(|program| programs.contains(program.as_str()))
                {
                    continue;
                }

                for word in words {
                    let named = match word.split_once('=') {
                        Some((option, value)) if option.starts_with("--") => value,
                        _ if word.starts_with('-') => continue,
                        _ => word.as_str(),
                    };

                    if named.is_empty() {
                        continue;
                    }

                    for path in project
                        .resolver
                        .script_target(package, named)
                        .into_iter()
                        .flatten()
                    {
                        entries.add(path, Source::named("scripts", &manifest));
                    }
                }
            }
        }
    }
}
=======
/// A command the project runs: a command of a `package.json`'s script, or one a CI system's command
/// file holds, with the folder it runs in, which its words are found from (spec ENTRY-7; comparison
/// spec CI-2).
struct Command<'p> {
    /// Its words, quotes removed.
    words: Vec<String>,
    /// The folder, from the root, it runs in: the empty path for the root.
    folder: String,
    /// The `package.json` whose script it is, or the command file that holds it.
    file: String,
    /// The script's text, for a script's command; none for a command file's.
    script: Option<&'p str>,
    /// For a command file's command, the stack whose pack declares the command file.
    stack: Option<&'p str>,
}

impl Command<'_> {
    /// What makes a file the command names an entry point, as `graph` prints it (comparison spec
    /// CI-4).
    fn source(&self) -> Source {
        match self.stack {
            None => Source::named("scripts", &self.file),
            Some(stack) => Source {
                from: "commandFiles",
                file: Some(self.file.clone()),
                stack: Some(stack.to_owned()),
                by: Some("pack"),
            },
        }
    }
}

/// Every command the project runs: each command of each `package.json`'s scripts, run in that
/// `package.json`'s folder, then each command of each file a pack's `commandFiles` declaration matches,
/// read once, from the deepest folder yak's detection looked in that holds it (spec ENTRY-7;
/// comparison spec CI-1, CI-2).
fn commands<'p>(project: &Project<'p>, notices: &mut Vec<Notice>) -> Vec<Command<'p>> {
    let mut found = Vec::new();

    for package in &project.packages.list {
        for script in &package.scripts {
            for words in tools::commands(script) {
                found.push(Command {
                    words,
                    folder: folder_path(&package.folder).to_owned(),
                    file: manifest_path(&package.folder),
                    script: Some(script.as_str()),
                    stack: None,
                });
            }
        }
    }

    let Some(handed) = project.handed else {
        return found;
    };

    for stack in &handed.stacks {
        let declarations = stack.pack.get("commandFiles").and_then(Value::as_array);

        for declaration in declarations
            .into_iter()
            .flatten()
            .filter_map(CommandFiles::of)
        {
            for file in project.files {
                let Some(base) = deepest(&handed.folders, &file.path).filter(|base| {
                    inside(base, &file.path).is_some_and(|rest| declaration.file.matches(rest))
                }) else {
                    continue;
                };

                read_commands(
                    project,
                    &declaration,
                    (&file.path, base),
                    &stack.name,
                    &mut found,
                    notices,
                );
            }
        }
    }

    found
}

/// The commands a command file holds, each at a path the declaration's `commands` gives, run in the
/// folder its `folder` paths find, else in the detection folder the file was matched from. A file past
/// the bounds or not valid YAML adds nothing, and nor does a command whose folder the analyzer cannot
/// know; each gives one notice (comparison spec CI-1 to CI-3).
fn read_commands<'p>(
    project: &Project<'p>,
    declaration: &CommandFiles,
    (path, base): (&str, &str),
    stack: &'p str,
    found: &mut Vec<Command<'p>>,
    notices: &mut Vec<Notice>,
) {
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
                    "{path} is over 1 MiB, or over 100,000 values once its aliases are followed, so its \
                     commands add no entry point."
                ),
            };

            notify(
                notices,
                Notice::new("command-file-not-read", Some(path.to_owned()), sentence),
            );
            return;
        }
    };

    for document in &documents {
        for keys in &declaration.commands {
            for (text, above) in command_files::reached(document, keys) {
                let Some(folder) = command_folder(declaration, &above, path, base, notices) else {
                    continue;
                };

                for words in tools::command_lines(text) {
                    found.push(Command {
                        words,
                        folder: folder.clone(),
                        file: path.to_owned(),
                        script: None,
                        stack: Some(stack),
                    });
                }
            }
        }
    }
}

/// The folder, from the root, a command file's command runs in: the text the declaration's `folder`
/// paths find, joined to the detection folder, else the detection folder itself. Nothing, with a
/// notice, where the text names a folder the analyzer cannot know (comparison spec CI-1).
fn command_folder(
    declaration: &CommandFiles,
    above: &[&Node],
    path: &str,
    base: &str,
    notices: &mut Vec<Notice>,
) -> Option<String> {
    let Some(written) = declaration
        .folder
        .as_deref()
        .and_then(|paths| command_files::folder(above, paths))
    else {
        return Some(base.to_owned());
    };
    let folder = run_folder(base, written);

    if folder.is_none() {
        notify(
            notices,
            Notice::new(
                "command-file-not-read",
                Some(path.to_owned()),
                format!(
                    "The folder `{written}` that a command of {path} runs in is not one the analyzer can \
                     know, so the commands there add no entry point."
                ),
            ),
        );
    }

    folder
}

/// The folder, from the root, a command file's folder text names, joined to the detection folder: a
/// leading `./`, a trailing `/` and each `.` say nothing, and each `..` takes off the segment before
/// it. Nothing where the text is empty, holds `$` or a backslash, is absolute, or would leave the root
/// (comparison spec CI-1).
fn run_folder(base: &str, written: &str) -> Option<String> {
    if written.is_empty() || written.contains(['$', '\\']) {
        return None;
    }

    resolved(base, written)
}

/// Adds a notice unless the run already has the same one, as a file two declarations match would give.
fn notify(notices: &mut Vec<Notice>, notice: Notice) {
    if !notices.contains(&notice) {
        notices.push(notice);
    }
}

/// The entry points the project's commands start: a file a word of a command names, from the folder
/// the command runs in, after `node` or a program a `scriptCommands` declaration names, as RES-3 and
/// MAP-1 find it. Words of the form `NAME=value` before the program are passed over, an option's own
/// name is never a file, `--option=value` is tried by its value, and a word holding `$` names no file.
/// A declared program counts in every command (spec ENTRY-7; comparison spec CI-2).
fn scripted(project: &Project<'_>, commands: &[Command<'_>], entries: &mut Entries) {
    let mut programs: BTreeSet<&str> = BTreeSet::from(["node"]);

    if let Some(handed) = project.handed {
        for stack in &handed.stacks {
            programs.extend(texts(stack.pack.get("scriptCommands")));
        }
    }

    for command in commands {
        let mut words = command
            .words
            .iter()
            .skip_while(|word| tools::is_assignment(word));

        if !words
            .next()
            .is_some_and(|program| programs.contains(program.as_str()))
        {
            continue;
        }

        for word in words {
            let named = match word.split_once('=') {
                Some((option, value)) if option.starts_with("--") => value,
                _ if word.starts_with('-') => continue,
                _ => word.as_str(),
            };

            if named.is_empty() || named.contains('$') {
                continue;
            }

            for path in project
                .resolver
                .script_target(&command.folder, named)
                .into_iter()
                .flatten()
            {
                entries.add(path, command.source());
            }
        }
    }
}
>>>>>>> replace
<<<<<<< find
/// file's that the analyzer does not read is passed over with a notice, and so is a word of a script
/// that names no listed file (spec ENTRY-6, CONF-1 to CONF-3).
fn configured(project: &Project<'_>, entries: &mut Entries, notices: &mut Vec<Notice>) {
=======
/// file's that the analyzer does not read is passed over with a notice, and so is a word of a command
/// that names no listed file (spec ENTRY-6; comparison spec CONF-1 to CONF-3, CI-2).
fn configured(
    project: &Project<'_>,
    commands: &[Command<'_>],
    entries: &mut Entries,
    notices: &mut Vec<Notice>,
) {
>>>>>>> replace
<<<<<<< find
                for (script, path) in named_configs(project, options, &listed, &mut missing) {
=======
                for (script, path) in named_configs(commands, options, &listed, &mut missing) {
>>>>>>> replace
<<<<<<< find
/// The configuration files the scripts of every `package.json` name after one of a declaration's
/// options, each with the folder, from the root, of the `package.json` whose script names it, which the
/// word is found from. A word that names no listed file is kept in `missing`, by its `package.json`,
/// with the sentence of its notice (spec CONF-2).
fn named_configs<'p>(
    project: &Project<'p>,
    options: &Options<'_>,
    listed: &BTreeSet<&str>,
    missing: &mut BTreeSet<(String, String)>,
) -> Vec<(&'p str, String)> {
    let mut named = Vec::new();

    for package in &project.packages.list {
        let folder = folder_path(&package.folder);
        let manifest = manifest_path(&package.folder);

        for script in &package.scripts {
            for command in tools::commands(script) {
                for (option, value) in options.values(&command) {
                    match resolved(folder, value).filter(|path| listed.contains(path.as_str())) {
                        Some(path) => named.push((folder, path)),
                        None => {
                            missing.insert((
                                manifest.clone(),
                                format!(
                                    "The script `{script}` in {manifest} names `{value}` after `{option}` as \
                                     a configuration file of `{}`, and no listed file is there, so it adds no \
                                     entry point.",
                                    options.program
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    named
}
=======
/// The configuration files the project's commands name after one of a declaration's options, each with
/// the folder, from the root, the command runs in, which the word is found from. A word that names no
/// listed file is kept in `missing`, by the `package.json` or command file it comes from, with the
/// sentence of its notice; a word holding `$` names no file and gives none (comparison spec CONF-2,
/// CI-2).
fn named_configs<'c>(
    commands: &'c [Command<'_>],
    options: &Options<'_>,
    listed: &BTreeSet<&str>,
    missing: &mut BTreeSet<(String, String)>,
) -> Vec<(&'c str, String)> {
    let mut named = Vec::new();

    for command in commands {
        for (option, value) in options.values(&command.words) {
            if value.contains('$') {
                continue;
            }

            match resolved(&command.folder, value).filter(|path| listed.contains(path.as_str())) {
                Some(path) => named.push((command.folder.as_str(), path)),
                None => {
                    let naming = match command.script {
                        Some(script) => format!("The script `{script}` in {}", command.file),
                        None => format!("A command in {}", command.file),
                    };

                    missing.insert((
                        command.file.clone(),
                        format!(
                            "{naming} names `{value}` after `{option}` as a configuration file of `{}`, and \
                             no listed file is there, so it adds no entry point.",
                            options.program
                        ),
                    ));
                }
            }
        }
    }

    named
}
>>>>>>> replace
<<<<<<< find
/// Whether the analyzer reads a glob a tool's configuration file states: one KEY-5 reads, that does not
=======
/// One `commandFiles` declaration, read. Its shape was checked when the hand-off file was read
/// (comparison spec CI-3), so a declaration of another shape is passed over.
struct CommandFiles {
    /// The command files, from each folder yak's detection looked in.
    file: Glob,
    /// The property paths to the commands in such a file.
    commands: Vec<Vec<String>>,
    /// The property paths to the folder a command runs in, where the declaration states them.
    folder: Option<Vec<Vec<String>>>,
}

impl CommandFiles {
    /// The declaration, where it has its shape.
    fn of(declaration: &Value) -> Option<Self> {
        Some(Self {
            file: Glob::new(declaration.get("file")?.as_str()?).ok()?,
            commands: property_paths(declaration.get("commands")),
            folder: declaration
                .get("folder")
                .map(|value| property_paths(Some(value))),
        })
    }
}

/// The property paths a declaration's field holds, each a list of keys.
fn property_paths(value: Option<&Value>) -> Vec<Vec<String>> {
    value
        .and_then(Value::as_array)
        .map(|paths| {
            paths
                .iter()
                .map(|path| texts(Some(path)).into_iter().map(str::to_owned).collect())
                .collect()
        })
        .unwrap_or_default()
}

/// Whether the analyzer reads a glob a tool's configuration file states: one KEY-5 reads, that does not
>>>>>>> replace
```

The tenth key, stated by a pack only, and its shape.

```rust edit path=src/declarations.rs
<<<<<<< find
    /// The programs whose file arguments in scripts are entry points, which only a pack may state (spec
    /// ENTRY-7).
    ScriptCommands,
}
=======
    /// The programs whose file arguments in scripts are entry points, which only a pack may state (spec
    /// ENTRY-7).
    ScriptCommands,
    /// The files a CI system reads commands from, and where in them, which only a pack may state
    /// (comparison spec CI-1).
    CommandFiles,
}
>>>>>>> replace
<<<<<<< find
const OPTIONS_FIELDS: [&str; 3] = ["program", "names", "relativeTo"];
=======
const OPTIONS_FIELDS: [&str; 3] = ["program", "names", "relativeTo"];

/// The fields an object of `commandFiles` may hold, each required but `folder` (comparison spec CI-1).
const COMMAND_FIELDS: [&str; 3] = ["file", "commands", "folder"];
>>>>>>> replace
<<<<<<< find
const KEYS: [(&str, Kind); 9] = [
=======
const KEYS: [(&str, Kind); 10] = [
>>>>>>> replace
<<<<<<< find
    ("scriptCommands", Kind::ScriptCommands),
];
=======
    ("scriptCommands", Kind::ScriptCommands),
    ("commandFiles", Kind::CommandFiles),
];
>>>>>>> replace
<<<<<<< find
        Kind::ConfigEntries | Kind::ScriptCommands if matches!(source, Source::Project) => {
            Some("only a pack may state this key".to_owned())
        }
        Kind::ConfigEntries => config_entries_problem(value),
=======
        Kind::ConfigEntries | Kind::ScriptCommands | Kind::CommandFiles
            if matches!(source, Source::Project) =>
        {
            Some("only a pack may state this key".to_owned())
        }
        Kind::ConfigEntries => config_entries_problem(value),
        Kind::CommandFiles => command_files_problem(value),
>>>>>>> replace
<<<<<<< find
/// Why the `options` of a `configEntries` object cannot be used, or nothing when they can: an object
=======
/// Why a `commandFiles` value cannot be used, or nothing when it can: a list of objects, each with a
/// `file` glob and its `commands` paths, optionally its `folder` paths, and no other field (comparison
/// spec CI-1, CI-3).
fn command_files_problem(value: &Value) -> Option<String> {
    let Some(list) = value.as_array() else {
        return Some("its value must be a list of objects".to_owned());
    };

    list.iter().enumerate().find_map(|(index, entry)| {
        let number = index + 1;
        let Some(fields) = entry.as_object() else {
            return Some(format!("its entry {number} must be an object"));
        };
        let field =
            |name: &str, why: &str| Some(format!("its entry {number}'s field `{name}` {why}"));

        if let Some(unknown) = fields
            .keys()
            .find(|key| !COMMAND_FIELDS.contains(&key.as_str()))
        {
            return field(unknown, "is not one it may hold");
        }

        if let Some(missing) = ["file", "commands"]
            .iter()
            .find(|name| !fields.contains_key(**name))
        {
            return field(missing, "is missing");
        }

        let Some(glob) = fields["file"].as_str() else {
            return field("file", "must be a glob");
        };

        if let Some(why) = glob_problem(glob) {
            return field("file", &format!("cannot be used: {why}"));
        }

        ["commands", "folder"]
            .iter()
            .find(|name| fields.get(**name).is_some_and(|paths| !is_paths(paths)))
            .and_then(|name| {
                field(
                    name,
                    "must be a list of property paths, one or more, each a list of keys, one or more",
                )
            })
    })
}

/// Whether a value is a list of one or more property paths, each a list of one or more keys.
fn is_paths(value: &Value) -> bool {
    value.as_array().is_some_and(|paths| {
        !paths.is_empty()
            && paths
                .iter()
                .all(|path| texts(path).is_some_and(|keys| !keys.is_empty()))
    })
}

/// Why the `options` of a `configEntries` object cannot be used, or nothing when they can: an object
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test entries --test declarations
```

Expected: PASS: 16 `entries` tests and 7 `declarations` tests.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two, 53 of them in the node
tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add src/command_files.rs src/main.rs src/tools.rs src/resolve.rs src/entries.rs src/declarations.rs tests/entries.test.rs tests/declarations.test.rs
git commit -m "feat(entries): read the commands a CI workflow runs as a script's"
```

Expected: one commit on `feat/comparison-3-run`. `yak stack format` covers no Rust file here, so
`cargo fmt` formats, as plans 1 and 2 ruled.

### Task 2: the comparison on the pinned commits, run again with the fix

**Implements:** CMP-1, CMP-2, CMP-3, CMP-4, CMP-4a, CMP-5, CMP-6, CMP-7, JDG-1, JDG-2, JDG-3, JDG-4, JDG-5, JDG-5a, JDG-5b, JDG-6, BAR-1, BAR-2, BAR-3, BAR-4, BRA-1, BRA-2, BRA-3, CURE-1, CURE-2, CURE-4, CURE-5, CI-5

**Check:** the owner reads the report folder's `cmp/judged.md`: which of BAR-1 to BAR-3 holds at its
top, and each case left with why; the pinned commits, fallow's version and both tools'
configurations with each change and its reason, the `commandFiles` declaration among them; per root,
the counts of each kind for each tool, shared, analyzer-only and fallow-only, and the judgment of
each finding only one tool reports and of each shared one, with JDG-5's section apart, the AN-4a
packages with their detection folders among it (JDG-5b); the eight branch audits; the times; and the
fix with its fixture and the case it cured. A declaration the owner rejects is removed and the runs
judged again.

**Files:**
- None in this repository: the report folder, `.git/yak-reports/2026-11-16-dead-code-comparison/cmp/`,
  is committed nowhere (JDG-6).

**Interfaces:**
- Consumes: the binary built from this branch; the worktrees, fallow's runs and the hand-off files of
  decision 1.
- Produces: the judged list updated by CURE-5's rerun, and the counts and outcome on the record.

- [ ] **Step 1: Add the declaration and rerun the analyzer**

Copy the branch's `target/release/yak-analyzer.exe` beside the earlier binary in
`yak-cmp-3/bin/`, add decision 8's `commandFiles` declaration to the `dead-code-tools` stack of every
root's hand-off file and every branch audit's, write the change in `cmp/CURE-2.md`, and run
`audit --hand-off <file>` on the six roots and `audit --hand-off <file> --base <base>` on the twelve
branch-audit roots, saving each beside the earlier run.

Expected: each run exits 0. hsp-platform's `scripts/check-ci-gates.mjs` is no longer reported; no
other finding changes on any root or branch, or each that does is judged in Step 2.

- [ ] **Step 2: Match, judge and update the judged list**

Match each new run against fallow's as before (`match.py`, `branchmatch.py`), judge every finding the
fix added or removed (CURE-5), and update `cmp/judged.md`: the top's bar, case B as cured with the
fixture, the counts, the AN-4a packages with each root's detection folders checked against its
`workspaces` (JDG-5b), and the times measured again with nothing else running.

Expected: no analyzer-only false finding, no miss within the five kinds, and no shared false finding
on the pinned commits: the floor and the ceiling hold there (BAR-1, BAR-3), with case A set apart.

- [ ] **Step 3: Record the counts and the outcome**

Record a ruling on Task 2 that holds, for each root, the analyzer's and fallow's counts by kind, the
shared, analyzer-only and fallow-only counts, the branch audits' agreement, the times, and which of
BAR-1 to BAR-3 holds.

Expected: the ruling is on the record. The worktrees stay for plan 4 (decision 10).
