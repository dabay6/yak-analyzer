# Ignoring files: the unreported key: implementation plan

**Goal:** a project lists, under the key `unreported`, globs of files that are read like any other but
in which no check reports a finding, and `audit`, `complexity` and `duplication` say how many findings
each glob left out.

**Architecture:** `src/marks.rs` gains the `unreported` set and its globs beside `data` and `generated`;
`src/declarations.rs` knows the key and refuses it from a pack. An unreported file becomes an entry
point after the faults are found, and its exports are not all used, so it reaches what it imports. Each
check leaves unreported files out where it leaves generated ones out, and hands back what it left out,
by the files that tie each finding to the globs; `Marks::counts` turns that into the notices. The dead-
code count works the uses out a second time over the same reading, without the unreported entry points.

**Spec:** docs/specs/2026-11-27-ignoring-files-design.md

**Implements:** KEY-6, KEY-7, KEY-8, READ-1, READ-2, READ-3, DEAD-1, CPLX-1, DUPE-1, CYCLE-1, COUNT-1, COUNT-2, COUNT-3, COUNT-4, COUNT-5, REFUSE-1, REFUSE-2, REFUSE-3, VERS-1, TEST-1, OWED-1

## Global constraints

- `unreported` is a list of globs, relative to the root, matching any file of the checkout; the globs
  of every stack join (KEY-6). Only a project may state it; a pack is refused with the reason `only a
  project may state this key` and the fix `change it in the pack` (KEY-7).
- A file both `data` and unreported is not read and is no entry point (KEY-8).
- An unreported file is an entry point with the reason `unreported`; an entry point with only that
  reason does not have its exports all used, and does not count for ENTRY-8's fault (READ-2).
- The notice kind is `findings-unreported`; a glob's sentence names the glob, the stacks that state it
  and the count, and the extra notice the findings outside the unreported files (COUNT-1).
- An unknown key's line gains `, or, if the key is newer than this analyzer, update the analyzer's
  version pinned in the pack` (REFUSE-2); `interface` prints the declaration keys (REFUSE-3).
- The interface version stays 1 (VERS-1). Fixtures are made-up projects (CONTRIBUTING.md).

## The decisions this plan makes

1. **The tests go in `tests/unreported.rs`,** with a `[[test]]` entry, since `Cargo.toml` sets
   `autotests = false`. The name has no dot, as the `rust-test-file-names` rule says, though the older
   test files carry `.test.rs`.
2. **`Marks` holds the set and the globs.** `unreported` is every path a glob matches; `globs` is one
   `Unreported` per glob as written, with the stacks that state it and its files. A data file a glob
   matches stays in the glob's files, so the glob's notice says 0, but `Entries::unreported` skips it
   (KEY-8).
3. **The unreported entry points are added in `src/reading.rs`, after `entries::find`,** by
   `Entries::unreported`, so the faults are already found (READ-2). `uses::begun` skips the reason
   `unreported` when it starts the uses of an entry point's exports; a file with another reason keeps
   that reason's use.
4. **What was left out is a list of findings, each by its files** (`Vec<Vec<String>>`): a dead-code
   finding by its file, a cycle by every file of its set, a complexity finding by its file, and a
   duplication group by the files of the copies it lost. `Marks::counts` makes the notices from it.
5. **The dead-code count** works the uses out again with `Entries::declared` (the entry points without
   the reason `unreported`) and no file left out, and keeps each such finding whose fingerprint the
   reported findings don't hold, as many times as it occurs more often (COUNT-2). It runs only when a
   glob matches a file.
6. **Duplication** keeps one pass: the groups are found over unreported files too, each loses its
   copies there, a group left with too few is dropped, SAME-4 runs again over the rest, and acceptance
   matches an entry by the group's copies before or after (DUPE-1). Each `Place` gains its own `tokens`
   and `lines`, so a reduced group's size is worked out from its remaining copies. `groups` returns a
   `Grouped` with the groups, the notices and what was left out.
7. **Complexity** splits `measured` into `measured` (every file but the unreported ones) and `left` (the
   functions over a limit in unreported files), over one private walk.
8. **A cycle with `--base`** moves to the first added edge whose importing file is not unreported; when
   there is none, it keeps CYCLE-1's place, and its message still names the first added edge. INH-3's
   case of a new cycle with no added edge cannot be reached through `audit`: it needs a file the base
   skipped, and then `inherit::mark` marks every dead-code finding new before any cycle is compared.
   TEST-1's fixture for it is therefore left out, and the case keeps CYCLE-1's place by the code path
   above. The owner sees this at the plan's approval.
9. **`interface`** prints `declarationKeys`, the keys in the order of `src/declarations.rs`'s list, with
   `unreported` after `generated`.

## Review focus

- **The glob's `*`.** Decision 2 and the fixtures rely on `src/*.ts` matching `src/a.ts` and not
  `src/sub/a.ts`, as `src/glob.rs` reads a glob (spec KEY-5). The counts test in Task 2 pins it.
- **A finding counted twice or under the wrong notice.** Two globs matching one file each count it; a
  finding outside every glob goes only to the extra notice. Task 2's counts test pins both.
- **The second uses pass.** Its cost falls only on runs whose globs match a file; nothing parses again
  (spec Risks). No test measures it.
- **Acceptance across the removal.** An entry for the old or the new number of copies accepts a group;
  one for a group dropped altogether gives no notice. Task 4's duplication test pins all three.

### Task 1: The key, its refusals, `interface` and `graph`'s flag

**Implements:** KEY-6, KEY-7, REFUSE-2, REFUSE-3

**Test:** `cargo test --release --locked --test unreported --test declarations --test cli --test file-list`

**Files:**
- Create: `tests/unreported.rs`
- Modify: `Cargo.toml`, `src/declarations.rs`, `src/marks.rs`, `src/main.rs`, `src/cli.rs`,
  `src/graph.rs`, `tests/declarations.test.rs`, `tests/cli.test.rs`, `tests/file-list.test.rs`

**Interfaces:**
- Consumes: `Glob`, `Notice`, `HandOff` as they are.
- Produces: `marks::UNREPORTED`; `Marks::unreported: BTreeSet<String>`, `Marks::globs: Vec<Unreported>`
  (`written`, `stacks`, `files`); `Marks::counts(&self, left: &[Vec<String>]) -> Vec<Notice>`;
  `declarations::names() -> Vec<&'static str>`; `graph`'s per-file `unreported` flag.

- [ ] **Step 1: Write the failing tests**

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
name = "unreported"
path = "tests/unreported.rs"
>>>>>>> replace
```

```rust path=tests/unreported.rs
//! Runs the built binary on made-up projects whose hand-off file states the `unreported` declaration,
//! and checks the files it names: that only a project may state it, that `graph` flags them, and that
//! no command reports a finding in them, each saying how many it left out (ignoring spec KEY-6 to
//! KEY-8, READ-1 to READ-3, DEAD-1, CPLX-1, DUPE-1, CYCLE-1, COUNT-1 to COUNT-5, TEST-1).

#[expect(
    dead_code,
    reason = "the shared project helpers include running `graph` with no hand-off file and reading a document's paths and notice kinds, which these tests do not need"
)]
#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, printed, stopped};

/// One stack named `typescript` whose project declares what is given.
fn stack(declared: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": {}, "project": declared }])
}

/// A hand-off file whose project declares what is given.
fn declaring(project: &Project, declared: Value) -> String {
    project.hand_off(&project.root, &["."], stack(declared))
}

/// The sentences of a document's notices of the kind given, in its order.
fn said<'d>(document: &'d Value, kind: &str) -> Vec<&'d str> {
    document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .filter(|notice| notice["kind"] == json!(kind))
        .map(|notice| notice["sentence"].as_str().expect("a sentence"))
        .collect()
}

#[test]
fn only_a_project_may_state_the_files_no_finding_is_reported_in() {
    let project = Project::new("unreported-pack");

    project.write("src/index.ts", "export {};\n").commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([{
            "name": "typescript",
            "interfaceVersion": 1,
            "pack": { "unreported": ["scripts/**"] },
            "project": {}
        }]),
    );

    // A pack's glob could not be taken back by a project, so a pack may not state the key (ignoring
    // spec KEY-7).
    assert_eq!(
        stopped(&project.run(|binary| {
            binary.args(["--hand-off", &hand_off]);
        })),
        "yak-analyzer: cannot use the declaration `unreported` that the pack states for the stack \
         `typescript`: only a project may state this key; change it in the pack"
    );
}

#[test]
fn graph_flags_each_file_an_unreported_glob_matches() {
    let project = Project::new("unreported-flags");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .write("scripts/tool.ts", "export {};\n")
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "unreported": ["scripts/**", "nothing/**"] }),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &hand_off]);
    }));
    let flagged: Vec<&str> = document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .filter(|file| file["unreported"] == json!(true))
        .map(|file| file["path"].as_str().expect("a path"))
        .collect();

    // Each file a glob matches is flagged, beside its `data` and `generated` flags (ignoring spec
    // KEY-6, COUNT-5).
    assert_eq!(flagged, ["scripts/tool.ts"]);
    // A glob that matches no file gives the notice a `data` or `generated` glob gives (ignoring spec
    // KEY-6).
    assert_eq!(
        said(&document, "glob-matches-nothing"),
        [
            "The `unreported` glob `nothing/**` that the project states for the stack `typescript` \
          matches no file."
        ]
    );
}
```

```rust edit path=tests/declarations.test.rs
<<<<<<< find
         the analyzer knows no such key, only entries, data, generated, internal, cyclomaticLimit, \
         cognitiveLimit, duplicationMinimum, duplicationMinimumLines, duplicationCopies, duplicationIgnore, \
         duplicationTestCallees, duplicationAccepted, configEntries, scriptCommands, commandFiles; change it \
         in the pack"
=======
         the analyzer knows no such key, only entries, data, generated, unreported, internal, cyclomaticLimit, \
         cognitiveLimit, duplicationMinimum, duplicationMinimumLines, duplicationCopies, duplicationIgnore, \
         duplicationTestCallees, duplicationAccepted, configEntries, scriptCommands, commandFiles; change it \
         in the pack, or, if the key is newer than this analyzer, update the analyzer's version pinned in the \
         pack"
>>>>>>> replace
```

```rust edit path=tests/cli.test.rs
<<<<<<< find
fn prints_its_interface_version_and_the_hand_off_formats_it_reads_as_one_line_of_json() {
    let output = run(&["interface"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"handOffFormats\":[1],\"interfaceVersion\":1}\n"
    );
=======
fn prints_its_interface_version_hand_off_formats_and_keys_as_one_line_of_json() {
    let output = run(&["interface"]);

    assert!(output.status.success());
    // The keys let yak tell a project's key that is newer than the analyzer its pack pins (ignoring
    // spec REFUSE-3).
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"declarationKeys\":[\"entries\",\"data\",\"generated\",\"unreported\",\"internal\",\
         \"cyclomaticLimit\",\"cognitiveLimit\",\"duplicationMinimum\",\"duplicationMinimumLines\",\
         \"duplicationCopies\",\"duplicationIgnore\",\"duplicationTestCallees\",\"duplicationAccepted\",\
         \"configEntries\",\"scriptCommands\",\"commandFiles\"],\"handOffFormats\":[1],\
         \"interfaceVersion\":1}\n"
    );
>>>>>>> replace
```

```rust edit path=tests/file-list.test.rs
<<<<<<< find
            "tracked": tracked,
        });
=======
            "tracked": tracked,
            "unreported": false,
        });
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```
cargo test --release --locked --test unreported --test declarations --test cli --test file-list
```

Expected: FAIL: the key is unknown, so both `unreported` tests stop on the old line; the unknown key's
line lacks its second fix; `interface` prints no keys; `graph` prints no `unreported` flag.

- [ ] **Step 3: Write the key and its refusals**

```rust edit path=src/declarations.rs
<<<<<<< find
    /// A list of globs.
    Globs,
=======
    /// A list of globs.
    Globs,
    /// A list of globs, which only a project may state (ignoring spec KEY-7).
    ProjectGlobs,
>>>>>>> replace
<<<<<<< find
/// Every key the analyzer knows, with the kind of its value (spec KEY-1; duplication spec TUNE-1 to
/// TUNE-6).
const KEYS: [(&str, Kind); 15] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
=======
/// Every key the analyzer knows, with the kind of its value (spec KEY-1; duplication spec TUNE-1 to
/// TUNE-6; ignoring spec KEY-6).
const KEYS: [(&str, Kind); 16] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("unreported", Kind::ProjectGlobs),
>>>>>>> replace
<<<<<<< find
/// Who states a declaration: the hand-off file keeps the two apart.
=======
/// The name of every key the analyzer knows, in the order of its list, as `interface` prints them
/// (ignoring spec REFUSE-3).
pub fn names() -> Vec<&'static str> {
    KEYS.iter().map(|(name, _)| *name).collect()
}

/// Who states a declaration: the hand-off file keeps the two apart.
>>>>>>> replace
<<<<<<< find
                        source.word(),
                        stack.name,
                        source.fix()
                    )));
=======
                        source.word(),
                        stack.name,
                        fix(key, source)
                    )));
>>>>>>> replace
<<<<<<< find
/// Why a declaration cannot be used, or nothing when it can.
=======
/// Where a person changes a declaration that cannot be used: for a key the analyzer doesn't know, also
/// the update that brings a key newer than this analyzer (ignoring spec REFUSE-2).
fn fix(key: &str, source: Source) -> String {
    if KEYS.iter().any(|(known, _)| *known == key) {
        source.fix().to_owned()
    } else {
        format!(
            "{}, or, if the key is newer than this analyzer, update the analyzer's version pinned in the \
             pack",
            source.fix()
        )
    }
}

/// Why a declaration cannot be used, or nothing when it can.
>>>>>>> replace
<<<<<<< find
        Kind::Globs => match texts(value) {
=======
        Kind::ProjectGlobs if matches!(source, Source::Pack) => {
            Some("only a project may state this key".to_owned())
        }
        Kind::Globs | Kind::ProjectGlobs => match texts(value) {
>>>>>>> replace
```

- [ ] **Step 4: Write the marks**

```rust path=src/marks.rs
//! The `data`, `generated` and `unreported` declarations, applied to the file list, and the notices that
//! say how many findings the `unreported` globs left out (spec KEY-3, KEY-4, KEY-5; ignoring spec
//! KEY-6, COUNT-1).

use std::collections::BTreeSet;

use serde_json::Value;

use crate::files::File;
use crate::glob::Glob;
use crate::hand_off::HandOff;
use crate::notice::Notice;
use crate::stop::Stop;

/// The key whose globs name the files no finding is reported in (ignoring spec KEY-6).
pub const UNREPORTED: &str = "unreported";

/// The kind of the notices that say how many findings the `unreported` globs left out (ignoring spec
/// COUNT-1).
const LEFT_OUT: &str = "findings-unreported";

/// The keys whose globs mark files, and what they mark them as.
const KEYS: [&str; 3] = ["data", "generated", UNREPORTED];

/// One glob of the `unreported` declaration, however many stacks state it, with the files it matches
/// (ignoring spec COUNT-1).
#[derive(Debug)]
pub struct Unreported {
    /// The glob, as it is written.
    pub written: String,
    /// The stacks that state it, in the order of the hand-off file.
    pub stacks: Vec<String>,
    /// The files it matches, by path.
    pub files: BTreeSet<String>,
}

/// Which files the declarations mark, by path.
#[derive(Debug, Default)]
pub struct Marks {
    /// Files that are not the project's code, which are never parsed or read (spec KEY-4).
    pub data: BTreeSet<String>,
    /// Committed files a generator writes, which are read like any other.
    pub generated: BTreeSet<String>,
    /// Files that are read like any other, in which no finding is reported (ignoring spec KEY-6).
    pub unreported: BTreeSet<String>,
    /// The globs of the `unreported` declaration, each once, in the order they are first stated.
    pub globs: Vec<Unreported>,
}

impl Marks {
    /// Notes a glob of the `unreported` declaration that a stack states, with the files it matches: a
    /// glob another stack states too, written the same, is one glob (ignoring spec COUNT-1).
    fn note_unreported(&mut self, written: &str, stack: &str, paths: BTreeSet<String>) {
        self.unreported.extend(paths.iter().cloned());

        let at = self
            .globs
            .iter()
            .position(|glob| glob.written == written)
            .unwrap_or_else(|| {
                self.globs.push(Unreported {
                    written: written.to_owned(),
                    stacks: Vec::new(),
                    files: BTreeSet::new(),
                });
                self.globs.len() - 1
            });
        let glob = &mut self.globs[at];

        if !glob.stacks.iter().any(|named| named == stack) {
            glob.stacks.push(stack.to_owned());
        }

        glob.files.extend(paths);
    }

    /// The notices that say how many findings the `unreported` globs left out, from each finding left
    /// out given by the files that tie it to the globs: one for each glob that matches a file, counting
    /// each finding one of whose files it matches, and one more for the findings none of whose files is
    /// unreported, where there are any (ignoring spec COUNT-1, COUNT-2).
    pub fn counts(&self, left: &[Vec<String>]) -> Vec<Notice> {
        let mut notices: Vec<Notice> = self
            .globs
            .iter()
            .filter(|glob| !glob.files.is_empty())
            .map(|glob| {
                let count = left
                    .iter()
                    .filter(|files| files.iter().any(|file| glob.files.contains(file)))
                    .count();
                let stacks = if glob.stacks.len() == 1 {
                    "stack"
                } else {
                    "stacks"
                };

                Notice::new(
                    LEFT_OUT,
                    None,
                    format!(
                        "The `{UNREPORTED}` glob `{}` that the project states for the {stacks} `{}` left \
                         out {count} finding(s).",
                        glob.written,
                        glob.stacks.join("`, `")
                    ),
                )
            })
            .collect();
        let outside = left
            .iter()
            .filter(|files| !files.iter().any(|file| self.unreported.contains(file)))
            .count();

        if outside > 0 {
            notices.push(Notice::new(
                LEFT_OUT,
                None,
                format!(
                    "{outside} more finding(s) outside the `{UNREPORTED}` files are not reported, since \
                     only those files use what they name."
                ),
            ));
        }

        notices
    }
}

/// Marks the files the declarations of the hand-off file name. Without a hand-off file there are no
/// declarations, and nothing is marked.
pub fn apply(
    files: &[File],
    handed: Option<&HandOff>,
    notices: &mut Vec<Notice>,
) -> Result<Marks, Stop> {
    let mut marks = Marks::default();
    let Some(handed) = handed else {
        return Ok(marks);
    };

    for stack in &handed.stacks {
        for key in KEYS {
            // A pack's globs are relative to each folder yak's detection looked in, and a project's to
            // the root (spec KEY-3). Only a project states `unreported` (ignoring spec KEY-7).
            let sources = [
                ("pack", &stack.pack, handed.folders.as_slice()),
                ("project", &stack.project, &[".".to_owned()][..]),
            ];

            for (source, declarations, folders) in sources {
                for written in globs(declarations.get(key)) {
                    let glob = Glob::new(written).map_err(Stop::new)?;
                    let matched: BTreeSet<String> = files
                        .iter()
                        .filter(|file| {
                            folders
                                .iter()
                                .any(|folder| under(folder, &file.path, &glob))
                        })
                        .map(|file| file.path.clone())
                        .collect();

                    if matched.is_empty() {
                        notices.push(Notice::new(
                            "glob-matches-nothing",
                            None,
                            format!(
                                "The `{key}` glob `{written}` that the {source} states for the stack `{}` \
                                 matches no file.",
                                stack.name
                            ),
                        ));
                    }

                    match key {
                        "data" => marks.data.extend(matched),
                        "generated" => marks.generated.extend(matched),
                        _ => marks.note_unreported(written, &stack.name, matched),
                    }
                }
            }
        }
    }

    Ok(marks)
}

/// The globs a declaration holds. The declaration was checked when the hand-off file was read, so
/// anything else holds none.
fn globs(declaration: Option<&Value>) -> Vec<&str> {
    declaration
        .and_then(Value::as_array)
        .map(|globs| globs.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// Whether a path lies under the folder and the glob matches it relative to that folder.
fn under(folder: &str, path: &str, glob: &Glob) -> bool {
    if folder == "." {
        return glob.matches(path);
    }

    path.strip_prefix(folder)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|rest| glob.matches(rest))
}
```

- [ ] **Step 5: Print the keys and the flag**

```rust edit path=src/main.rs
<<<<<<< find
                serde_json::json!({ "handOffFormats": HAND_OFF_FORMATS, "interfaceVersion": INTERFACE_VERSION })
=======
                serde_json::json!({
                    "declarationKeys": declarations::names(),
                    "handOffFormats": HAND_OFF_FORMATS,
                    "interfaceVersion": INTERFACE_VERSION,
                })
>>>>>>> replace
```

```rust edit path=src/cli.rs
<<<<<<< find
/// The command that prints the interface version and the hand-off formats the analyzer reads.
=======
/// The command that prints the interface version, the hand-off formats the analyzer reads and the
/// declaration keys it knows.
>>>>>>> replace
<<<<<<< find
    /// Print the interface version and the hand-off formats.
=======
    /// Print the interface version, the hand-off formats and the declaration keys.
>>>>>>> replace
```

```rust edit path=src/graph.rs
<<<<<<< find
//! So far that is the hand-off file, the file list, the `data` and `generated` declarations, the
=======
//! So far that is the hand-off file, the file list, the `data`, `generated` and `unreported` declarations, the
>>>>>>> replace
<<<<<<< find
        "generated": marks.generated.contains(&file.path),
=======
        "generated": marks.generated.contains(&file.path),
        "unreported": marks.unreported.contains(&file.path),
>>>>>>> replace
```

- [ ] **Step 6: Run the tests to see them pass**

```
cargo test --release --locked --test unreported --test declarations --test cli --test file-list
```

Expected: PASS.

- [ ] **Step 7: Commit**

```
yak stack format
yak stack check
git add Cargo.toml src/declarations.rs src/marks.rs src/main.rs src/cli.rs src/graph.rs tests/unreported.rs tests/declarations.test.rs tests/cli.test.rs tests/file-list.test.rs
git commit -m "feat(declarations): add the unreported key a project states"
```

Expected: the format and the check report nothing left to fix, and the commit is made.

### Task 2: Entry points, dead code, cycles and the counts

**Implements:** KEY-8, READ-1, READ-2, READ-3, DEAD-1, CYCLE-1, COUNT-1, COUNT-2, COUNT-3, COUNT-5, TEST-1

**Test:** `cargo test --release --locked --test unreported`

**Files:**
- Modify: `src/entries.rs`, `src/reading.rs`, `src/uses.rs`, `src/findings.rs`, `src/inherit.rs`,
  `tests/unreported.rs`

**Interfaces:**
- Consumes: Task 1's `Marks::unreported`, `Marks::globs`, `Marks::counts`, `marks::UNREPORTED`.
- Produces: `Source::unreported(&self) -> bool`; `Entries::unreported(&mut self, marks: &Marks)`;
  `Entries::declared(&self) -> Entries`; `findings::find`'s `findings-unreported` notices; the
  unreported filter in `findings::counted`, which the base's comparison uses too.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/unreported.rs
<<<<<<< find
            "The `unreported` glob `nothing/**` that the project states for the stack `typescript` \
          matches no file."
        ]
    );
}
=======
            "The `unreported` glob `nothing/**` that the project states for the stack `typescript` \
          matches no file."
        ]
    );
}

/// The one run of the report `audit` prints for the checkout with the options given. The run must
/// succeed and write no line.
fn audited(project: &Project, options: &[&str]) -> Value {
    printed(&project.start("audit", |binary| {
        binary.args(options);
    }))["runs"][0]
        .clone()
}

/// The one run of the report `audit` prints for the checkout with the hand-off file given.
fn audit(project: &Project, hand_off: &str) -> Value {
    audited(project, &["--hand-off", hand_off])
}

/// The document `graph` prints for the checkout with the hand-off file given.
fn graphed(project: &Project, hand_off: &str) -> Value {
    printed(&project.run(|binary| {
        binary.args(["--hand-off", hand_off]);
    }))
}

/// The results of a run, in its order.
fn results(run: &Value) -> &[Value] {
    run["results"].as_array().expect("a list of results")
}

/// Where a result is: its file as the report writes it, and its line where it has one.
fn place(result: &Value) -> (&str, Option<u64>) {
    let place = &result["locations"][0]["physicalLocation"];

    (
        place["artifactLocation"]["uri"].as_str().expect("a file"),
        place["region"]["startLine"].as_u64(),
    )
}

/// The findings of a run, in its order: each with its rule, its file and its line.
fn found(run: &Value) -> Vec<(&str, &str, Option<u64>)> {
    results(run)
        .iter()
        .map(|result| {
            let (file, line) = place(result);

            (result["ruleId"].as_str().expect("a rule"), file, line)
        })
        .collect()
}

/// The findings of the rule given, in the run's order: each with its file, its line, and whether it is
/// new or inherited where the run says.
fn states<'r>(run: &'r Value, rule: &str) -> Vec<(&'r str, Option<u64>, Option<&'r str>)> {
    results(run)
        .iter()
        .filter(|result| result["ruleId"] == json!(rule))
        .map(|result| {
            let (file, line) = place(result);

            (file, line, result["baselineState"].as_str())
        })
        .collect()
}

/// The finding of the rule given at the file given.
fn at<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r Value {
    results(run)
        .iter()
        .find(|result| result["ruleId"] == json!(rule) && place(result).0 == uri)
        .unwrap_or_else(|| panic!("a finding of {rule} at {uri}"))
}

/// The message of the finding of the rule given at the file given.
fn message<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r str {
    at(run, rule, uri)["message"]["text"]
        .as_str()
        .expect("a message")
}

/// The fingerprint of the finding of the rule given at the file given.
fn fingerprint<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r str {
    at(run, rule, uri)["partialFingerprints"]["yakAnalyzer/v1"]
        .as_str()
        .expect("a fingerprint")
}

/// The sentences of a run's notifications of the kind given, in its order.
fn sentences<'r>(run: &'r Value, kind: &str) -> Vec<&'r str> {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .filter(|notification| notification["descriptor"]["id"] == json!(kind))
        .map(|notification| {
            notification["message"]["text"]
                .as_str()
                .expect("a sentence")
        })
        .collect()
}

/// The reasons `graph` gives for the entry point at the path given; null where it is no entry point.
fn reasons(document: &Value, path: &str) -> Value {
    document["entries"]
        .as_array()
        .expect("a list of entry points")
        .iter()
        .find(|entry| entry["path"] == json!(path))
        .map_or(Value::Null, |entry| entry["sources"].clone())
}

/// The one reason the `unreported` declaration of the stack `typescript` gives an entry point.
fn unreported_reason() -> Value {
    json!([{ "by": "project", "file": null, "from": "unreported", "stack": "typescript" }])
}

#[test]
fn an_unreported_file_is_read_and_reaches_what_it_imports_but_nothing_is_reported_in_it() {
    let project = Project::new("unreported-dead-code");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import { used } from './lib';\n\nused();\n")
        .write(
            "src/lib.ts",
            "export const used = (): void => {};\nexport const helper = (): void => {};\n",
        )
        .write(
            "scripts/report.ts",
            "import { helper } from '../src/lib';\nimport { only } from './only';\nimport { gone } from \
             './missing';\n\nhelper();\nonly();\ngone();\n\nexport const spare = 1;\n",
        )
        .write(
            "scripts/only.ts",
            "export const only = (): void => {};\nexport const extra = 2;\n",
        )
        .commit();

    let hand_off = declaring(&project, json!({ "unreported": ["scripts/report.ts"] }));
    let run = audit(&project, &hand_off);

    // `helper` and `only` are used by the unreported script, an entry point; `extra`, which nothing
    // imports, is now reported, and nothing in the script is (ignoring spec READ-2, DEAD-1).
    assert_eq!(found(&run), [("unused-export", "scripts/only.ts", Some(2))]);
    // Without the declaration the run would report the script and `only.ts` as unused files, and the
    // script's import of `./missing`; `helper` is used even then, since an import in any file read is a
    // use. The two in the script count against its glob, and `only.ts` in the notice for the rest
    // (ignoring spec COUNT-1, COUNT-2).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "1 more finding(s) outside the `unreported` files are not reported, since only those files \
             use what they name.",
            "The `unreported` glob `scripts/report.ts` that the project states for the stack `typescript` \
             left out 2 finding(s).",
        ]
    );
    // `graph` lists the script as an entry point with the reason `unreported` (ignoring spec COUNT-5).
    assert_eq!(
        reasons(&graphed(&project, &hand_off), "scripts/report.ts"),
        unreported_reason()
    );
}

#[test]
fn an_unreported_files_re_exports_pass_no_use_on_unless_something_else_starts_it() {
    let project = Project::new("unreported-re-exports");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export { kept } from './kept';\n")
        .write("src/kept.ts", "export const kept = 1;\n")
        .write("scripts/barrel.ts", "export { lone } from '../src/lone';\n")
        .write("src/lone.ts", "export const lone = 1;\n")
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "unreported": ["src/index.ts", "scripts/**"] }),
    );

    // The package's `main` still uses every export of `src/index.ts`, so `kept` is used through it; the
    // unreported script alone starts nothing, so `lone`, which only it re-exports, is unused (ignoring
    // spec READ-2).
    assert_eq!(
        found(&audit(&project, &hand_off)),
        [("unused-export", "src/lone.ts", Some(1))]
    );
}

#[test]
fn a_cycle_is_left_out_only_when_every_file_in_it_is_unreported() {
    let project = Project::new("unreported-cycles");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import './c';\nimport '../scripts/x';\n")
        .write("src/c.ts", "import '../scripts/z';\nexport const c = 1;\n")
        .write("scripts/z.ts", "import '../src/c';\nexport const z = 1;\n")
        .write("scripts/x.ts", "import './y';\nexport const x = 1;\n")
        .write("scripts/y.ts", "import './x';\nexport const y = 1;\n")
        .write("gen/p.ts", "import './q';\n")
        .write("gen/q.ts", "import './p';\n")
        .commit();

    let plain = audit(
        &project,
        &declaring(&project, json!({ "generated": ["gen/**"] })),
    );
    let hand_off = declaring(
        &project,
        json!({ "generated": ["gen/**"], "unreported": ["scripts/**", "gen/**"] }),
    );
    let run = audit(&project, &hand_off);

    // The cycle of `src/c.ts` and `scripts/z.ts` is reported at its first file that is not unreported,
    // with the fingerprint it has without the declaration; the cycles of unreported files alone are
    // left out, generated ones included (ignoring spec CYCLE-1, KEY-8).
    assert_eq!(
        found(&run),
        [
            ("unused-export", "src/c.ts", Some(2)),
            ("circular-dependency", "src/c.ts", Some(1)),
        ]
    );
    assert_eq!(
        message(&run, "circular-dependency", "src/c.ts"),
        "These files import each other in a loop: `src/c.ts` → `scripts/z.ts` → `src/c.ts`."
    );
    assert_eq!(
        fingerprint(&run, "circular-dependency", "src/c.ts"),
        fingerprint(&plain, "circular-dependency", "scripts/z.ts")
    );
    // `scripts/**` left out the exports of `x`, `y` and `z` and the cycle of `x` and `y`; `gen/**` the
    // cycle of `p` and `q`, which `generated` alone does not leave out (ignoring spec COUNT-2).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "The `unreported` glob `gen/**` that the project states for the stack `typescript` left out \
             1 finding(s).",
            "The `unreported` glob `scripts/**` that the project states for the stack `typescript` left \
             out 4 finding(s).",
        ]
    );
    // A file both generated and unreported is an entry point (ignoring spec KEY-8).
    assert_eq!(
        reasons(&graphed(&project, &hand_off), "gen/p.ts"),
        unreported_reason()
    );
}

#[test]
fn each_glob_counts_the_findings_it_left_out_once_however_many_stacks_state_it() {
    let project = Project::new("unreported-counts");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import { a } from './a';\n\na();\n")
        .write(
            "src/a.ts",
            "export const a = (): void => {};\nexport const spare = 1;\n",
        )
        .write("docs/note.ts", "export const note = 1;\n")
        .write("fixtures/sample.ts", "export const sample = 1;\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([
            {
                "name": "typescript",
                "interfaceVersion": 1,
                "pack": { "data": ["fixtures/**"] },
                "project": {
                    "unreported": [
                        "src/a.ts",
                        "src/*.ts",
                        "src/index.ts",
                        "docs/**",
                        "fixtures/**",
                        "nothing/**"
                    ]
                }
            },
            {
                "name": "javascript",
                "interfaceVersion": 1,
                "pack": {},
                "project": { "unreported": ["src/a.ts"] }
            }
        ]),
    );
    let run = audit(&project, &hand_off);

    // `spare` and the unused `docs/note.ts` are left out, and nothing is reported (ignoring spec
    // DEAD-1).
    assert!(found(&run).is_empty());
    // Two globs that match one file each count its finding; a glob that two stacks state is one notice;
    // a glob with no effect, or matching only a data file, says 0; a glob that matches nothing gives
    // its own notice instead (ignoring spec COUNT-1, COUNT-2, KEY-8).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "The `unreported` glob `docs/**` that the project states for the stack `typescript` left out \
             1 finding(s).",
            "The `unreported` glob `fixtures/**` that the project states for the stack `typescript` left \
             out 0 finding(s).",
            "The `unreported` glob `src/*.ts` that the project states for the stack `typescript` left out \
             1 finding(s).",
            "The `unreported` glob `src/a.ts` that the project states for the stacks `typescript`, \
             `javascript` left out 1 finding(s).",
            "The `unreported` glob `src/index.ts` that the project states for the stack `typescript` left \
             out 0 finding(s).",
        ]
    );
    assert_eq!(
        sentences(&run, "glob-matches-nothing"),
        [
            "The `unreported` glob `nothing/**` that the project states for the stack `typescript` \
          matches no file."
        ]
    );
    // A data file is not read, so it is no entry point (ignoring spec KEY-8).
    assert_eq!(
        reasons(&graphed(&project, &hand_off), "fixtures/sample.ts"),
        Value::Null
    );
}

#[test]
fn an_unreported_entry_point_leaves_a_packages_fault_and_its_files_out_of_the_notices() {
    let project = Project::new("unreported-packages");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .write("tools/package.json", r#"{ "name": "tools" }"#)
        .write("tools/run.ts", "export {};\n")
        .write("tools/left.ts", "export {};\n")
        .write("extra/package.json", r#"{ "name": "extra" }"#)
        .write("extra/only.ts", "export {};\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &[".", "tools"],
        stack(json!({ "unreported": ["tools/run.ts", "extra/**"] })),
    );
    let run = audit(&project, &hand_off);

    // `tools/run.ts` is the package's only entry point, by the reason `unreported` alone, so the package
    // keeps its fault and the unused file its hint (ignoring spec READ-2).
    assert_eq!(found(&run), [("unused-file", "tools/left.ts", None)]);
    assert_eq!(
        message(&run, "unused-file", "tools/left.ts"),
        "No entry point reaches this file through any import, so it can be removed. Its package `tools` \
         has no entry point at all, so the likely fault is a missing `entries` declaration."
    );
    // The package `extra` holds only unreported files, so it gives no notice that its kinds are not
    // reported (ignoring spec READ-3).
    assert!(sentences(&run, "kinds-not-reported").is_empty());
}

#[test]
fn with_a_base_a_new_cycle_moves_only_to_an_added_import_in_a_file_that_is_not_unreported() {
    let project = Project::new("unreported-base");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import './c';\nimport './d';\n")
        .write("src/c.ts", "import '../scripts/z';\nexport const c = 1;\n")
        .write("scripts/z.ts", "import '../src/e';\nexport const z = 1;\n")
        .write("src/e.ts", "export const e = 1;\n")
        .write("src/d.ts", "import '../scripts/w';\nexport const d = 1;\n")
        .write("scripts/w.ts", "export const w = 1;\n")
        .write("docs/old.ts", "export const old = 1;\n")
        .commit();
    project.git(&["switch", "--quiet", "-c", "branch"]);
    project
        .write("src/e.ts", "import './c';\nexport const e = 1;\n")
        .write(
            "scripts/z.ts",
            "import '../src/e';\nimport '../src/c';\nexport const z = 1;\n",
        )
        .write("scripts/w.ts", "import '../src/d';\nexport const w = 1;\n");

    let hand_off = declaring(&project, json!({ "unreported": ["scripts/**", "docs/**"] }));
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // The branch added an import in `scripts/z.ts`, first by path, and one in `src/e.ts`: the new cycle
    // moves to the second. The other's only added import is in `scripts/w.ts`, so it keeps its place
    // at `src/d.ts` (ignoring spec CYCLE-1).
    assert_eq!(
        states(&run, "circular-dependency"),
        [
            ("src/d.ts", Some(1), Some("new")),
            ("src/e.ts", Some(1), Some("new")),
        ]
    );
    assert!(message(&run, "circular-dependency", "src/e.ts").ends_with(
        " The branch added the import of `./c` on line 1 of `src/e.ts`, which leads to `src/c.ts`."
    ));
    assert!(message(&run, "circular-dependency", "src/d.ts").ends_with(
        " The branch added the import of `../src/d` on line 1 of `scripts/w.ts`, which leads to \
             `src/d.ts`."
    ));
    // `docs/old.ts` is left out on both sides, and only the branch's finding is counted (ignoring spec
    // COUNT-3).
    assert!(sentences(&run, "findings-unreported").contains(
        &"The `unreported` glob `docs/**` that the project states for the stack `typescript` left out 1 \
          finding(s)."
    ));
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```
cargo test --release --locked --test unreported
```

Expected: FAIL: the six new tests find the unreported files reported, unreached and counted by no
notice.

- [ ] **Step 3: Make unreported files entry points**

```rust edit path=src/entries.rs
<<<<<<< find
use crate::hand_off::HandOff;
=======
use crate::hand_off::HandOff;
use crate::marks::{Marks, UNREPORTED};
>>>>>>> replace
<<<<<<< find
    /// `exports`), a tsconfig's `files`, a page's script (`page`), or the declaration `entries`.
=======
    /// `exports`), a tsconfig's `files`, a page's script (`page`), or the declaration `entries` or
    /// `unreported`.
>>>>>>> replace
<<<<<<< find
    /// The source as `graph` prints it.
    pub fn printed(&self) -> Value {
        json!({ "by": self.by, "file": self.file, "from": self.from, "stack": self.stack })
    }
}
=======
    /// The source as `graph` prints it.
    pub fn printed(&self) -> Value {
        json!({ "by": self.by, "file": self.file, "from": self.from, "stack": self.stack })
    }

    /// Whether the `unreported` declaration made the file an entry point (ignoring spec READ-2).
    pub fn unreported(&self) -> bool {
        self.from == UNREPORTED
    }
}
>>>>>>> replace
<<<<<<< find
impl Entries {
    /// Records that a source makes a file an entry point.
    fn add(&mut self, path: String, source: Source) {
        self.points.entry(path).or_default().insert(source);
    }
}
=======
impl Entries {
    /// Records that a source makes a file an entry point.
    fn add(&mut self, path: String, source: Source) {
        self.points.entry(path).or_default().insert(source);
    }

    /// Makes each file the `unreported` declaration names an entry point, with the reason `unreported`
    /// for each stack that names it, but a file declared data, which is not read. It is called once the
    /// faults are found, so an entry point with only this reason leaves a package's fault as it was
    /// (ignoring spec READ-2, KEY-8).
    pub fn unreported(&mut self, marks: &Marks) {
        for glob in &marks.globs {
            for path in glob.files.iter().filter(|path| !marks.data.contains(*path)) {
                for stack in &glob.stacks {
                    self.add(
                        path.clone(),
                        Source {
                            from: UNREPORTED,
                            file: None,
                            stack: Some(stack.clone()),
                            by: Some("project"),
                        },
                    );
                }
            }
        }
    }

    /// The entry points as they would be without the `unreported` declaration: each source but its, and
    /// none that only it made (ignoring spec COUNT-2).
    pub fn declared(&self) -> Self {
        Self {
            points: self
                .points
                .iter()
                .filter_map(|(path, sources)| {
                    let kept: BTreeSet<Source> = sources
                        .iter()
                        .filter(|source| !source.unreported())
                        .cloned()
                        .collect();

                    (!kept.is_empty()).then(|| (path.clone(), kept))
                })
                .collect(),
            faults: self.faults.clone(),
        }
    }
}
>>>>>>> replace
```

```rust edit path=src/reading.rs
<<<<<<< find
    /// Which files the declarations mark as data or as generated.
=======
    /// Which files the declarations mark as data, as generated or as unreported.
>>>>>>> replace
<<<<<<< find
    let entries = stages.time("entries", || {
=======
    let mut entries = stages.time("entries", || {
>>>>>>> replace
<<<<<<< find
    })?;
    let uses = stages.time("uses", || uses::find(&scripts, &resolved, &entries));
=======
    })?;

    // An unreported file is an entry point, made one once the faults are found (ignoring spec READ-2).
    entries.unreported(&marks);

    let uses = stages.time("uses", || uses::find(&scripts, &resolved, &entries));
>>>>>>> replace
```

```rust edit path=src/uses.rs
<<<<<<< find
/// The uses that begin in the files and at the entry points, each with where it counts from: what an
/// import uses of each file it leads to, what a file's own template names, and every export of an
/// entry point (spec USE-2, USE-3, USE-4, USE-6).
=======
/// The uses that begin in the files and at the entry points, each with where it counts from: what an
/// import uses of each file it leads to, what a file's own template names, and every export of an
/// entry point, for each reason but `unreported` (spec USE-2, USE-3, USE-4, USE-6; ignoring spec
/// READ-2).
>>>>>>> replace
<<<<<<< find
    for (path, sources) in &entries.points {
        for source in sources {
=======
    for (path, sources) in &entries.points {
        // Nothing starts an unreported file to use its exports: it only reaches what it imports.
        for source in sources.iter().filter(|source| !source.unreported()) {
>>>>>>> replace
```

- [ ] **Step 4: Leave unreported files out of dead code and cycles, and count what was left out**

```rust edit path=src/findings.rs
<<<<<<< find
use crate::scripts;
=======
use crate::scripts;
use crate::uses::{self, Uses};
>>>>>>> replace
<<<<<<< find
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = counted(reading);
    let mut notices = Vec::new();

    circular(reading, &mut found);
    complex(reading, &mut found);
    duplicated(reading, &mut found, &mut notices);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
    notices.extend(reading.limits.notice());
=======
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = counted(reading);
    let mut notices = Vec::new();
    // Each finding the `unreported` declaration left out, by the files that tie it to its globs
    // (ignoring spec COUNT-2).
    let mut left = left_dead(reading, &found);

    circular(reading, &mut found, &mut left);
    complex(reading, &mut found);
    duplicated(reading, &mut found, &mut notices);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
    notices.extend(reading.limits.notice());
    notices.extend(reading.marks.counts(&left));
>>>>>>> replace
<<<<<<< find
    let mut found = Vec::new();

    unused(reading, &mut found);
    unresolved(reading, &mut found);

    found
}
=======
    let mut found = Vec::new();

    unused(
        reading,
        &reading.uses,
        &reading.marks.unreported,
        &mut found,
    );
    unresolved(reading, &reading.marks.unreported, &mut found);

    found
}

/// The findings of the dead-code kinds the `unreported` declaration left out, each by its file: those
/// the run would report without it, worked out over the same reading from the entry points without the
/// reason `unreported` and with no file left out, that it does not report with it. None where the
/// declaration matches no file (ignoring spec COUNT-2).
fn left_dead(reading: &Reading, reported: &[Finding]) -> Vec<Vec<String>> {
    if reading.marks.unreported.is_empty() {
        return Vec::new();
    }

    let uses = uses::find(
        &reading.scripts,
        &reading.resolved,
        &reading.entries.declared(),
    );
    let none = BTreeSet::new();
    let mut plain = Vec::new();

    unused(reading, &uses, &none, &mut plain);
    unresolved(reading, &none, &mut plain);

    // How many findings of each fingerprint the run reports, which match as many it would report.
    let mut held: BTreeMap<&str, usize> = BTreeMap::new();

    for finding in reported {
        *held.entry(finding.fingerprint.as_str()).or_default() += 1;
    }

    plain
        .into_iter()
        .filter(|finding| {
            held.get_mut(finding.fingerprint.as_str())
                .filter(|count| **count > 0)
                .map(|count| *count -= 1)
                .is_none()
        })
        .map(|finding| vec![finding.location.path])
        .collect()
}
>>>>>>> replace
<<<<<<< find
/// Whether a file is the project's own code to report on: read, and neither data nor generated.
fn own(reading: &Reading, file: &File) -> bool {
    !reading.marks.data.contains(&file.path)
        && !reading.marks.generated.contains(&file.path)
        && !reading.scripts.gone.contains(&file.path)
}

/// The unused files, exports and types (spec KIND-1, KIND-2).
fn unused(reading: &Reading, found: &mut Vec<Finding>) {
    let used: BTreeSet<(&str, Option<&str>)> = reading
        .uses
        .list
=======
/// Whether a file is the project's own code to report on: read, neither data nor generated, and not one
/// of the files given, which are left out (ignoring spec READ-3).
fn own(reading: &Reading, file: &File, unreported: &BTreeSet<String>) -> bool {
    !reading.marks.data.contains(&file.path)
        && !reading.marks.generated.contains(&file.path)
        && !unreported.contains(&file.path)
        && !reading.scripts.gone.contains(&file.path)
}

/// The unused files, exports and types, by the uses given, in every file the analyzer reports on but
/// the files given (spec KIND-1, KIND-2; ignoring spec DEAD-1).
fn unused(reading: &Reading, uses: &Uses, unreported: &BTreeSet<String>, found: &mut Vec<Finding>) {
    let used: BTreeSet<(&str, Option<&str>)> = uses
        .list
>>>>>>> replace
<<<<<<< find
            || !own(reading, file)
            || !reported(reading, &file.path)
        {
            continue;
        }

        let path = file.path.as_str();

        if !reading.uses.reached.contains_key(path) {
=======
            || !own(reading, file, unreported)
            || !reported(reading, &file.path)
        {
            continue;
        }

        let path = file.path.as_str();

        if !uses.reached.contains_key(path) {
>>>>>>> replace
<<<<<<< find
        reading
            .files
            .iter()
            .filter(|file| scripts::reads(file.kind) && own(reading, file))
=======
        reading.files.iter().filter(|file| {
            scripts::reads(file.kind) && own(reading, file, &reading.marks.unreported)
        })
>>>>>>> replace
<<<<<<< find
/// The imports that lead to no file, in any file under the root that is read and is neither generated
/// nor a type declaration file (spec KIND-3).
fn unresolved(reading: &Reading, found: &mut Vec<Finding>) {
    let kinds = kinds(reading);

    for (path, read) in &reading.scripts.read {
        if kinds.get(path.as_str()) == Some(&Kind::Declaration)
            || reading.marks.generated.contains(path)
        {
=======
/// The imports that lead to no file, in any file under the root that is read and is neither generated
/// nor a type declaration file nor one of the files given (spec KIND-3; ignoring spec DEAD-1).
fn unresolved(reading: &Reading, unreported: &BTreeSet<String>, found: &mut Vec<Finding>) {
    let kinds = kinds(reading);

    for (path, read) in &reading.scripts.read {
        if kinds.get(path.as_str()) == Some(&Kind::Declaration)
            || reading.marks.generated.contains(path)
            || unreported.contains(path)
        {
>>>>>>> replace
<<<<<<< find
/// The circular dependencies: each strongly connected set of two or more files, or a file that loads
/// itself, as one finding (spec KIND-4).
fn circular(reading: &Reading, found: &mut Vec<Finding>) {
    let (nodes, edges) = edges(reading);

    for set in strongly_connected(&edges) {
        let first = set[0];

        if set.len() == 1 && !edges[first].contains_key(&first) {
            continue;
        }

        let members: BTreeSet<usize> = set.iter().copied().collect();
=======
/// The circular dependencies: each strongly connected set of two or more files, or a file that loads
/// itself, as one finding, at its first file by path that is not unreported; a set whose every file is
/// unreported is left out, by its files (spec KIND-4; ignoring spec CYCLE-1).
fn circular(reading: &Reading, found: &mut Vec<Finding>, left: &mut Vec<Vec<String>>) {
    let (nodes, edges) = edges(reading);

    for set in strongly_connected(&edges) {
        if set.len() == 1 && !edges[set[0]].contains_key(&set[0]) {
            continue;
        }

        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let Some(first) = set
            .iter()
            .copied()
            .find(|node| !reading.marks.unreported.contains(nodes[*node]))
        else {
            left.push(paths.iter().map(|path| (*path).to_owned()).collect());

            continue;
        };
        let members: BTreeSet<usize> = set.iter().copied().collect();
>>>>>>> replace
<<<<<<< find
        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let mut finding = Finding::new(
            Rule::CircularDependency,
            looped(&nodes, &edges, &members, first),
            placed(first),
            &paths,
        );

        finding.related = set[1..].iter().map(|node| placed(*node)).collect();
        found.push(finding);
=======
        let mut finding = Finding::new(
            Rule::CircularDependency,
            looped(&nodes, &edges, &members, first),
            placed(first),
            &paths,
        );

        finding.related = set
            .iter()
            .filter(|node| **node != first)
            .map(|node| placed(*node))
            .collect();
        found.push(finding);
>>>>>>> replace
```

```rust edit path=src/inherit.rs
<<<<<<< find
/// Marks a circular dependency: inherited when one strongly connected set of the base holds all its
/// files and the branch added no edge between two of them; otherwise new, placed at the first import, by
/// path and line, that makes an edge the branch added (dead-code spec INH-3).
=======
/// Marks a circular dependency: inherited when one strongly connected set of the base holds all its
/// files and the branch added no edge between two of them; otherwise new, placed at the first import, by
/// path and line, that makes an edge the branch added in a file that is not unreported, or where it was
/// when there is none (dead-code spec INH-3; ignoring spec CYCLE-1).
>>>>>>> replace
<<<<<<< find
    let Some(&(from, line, place, to)) = added.first() else {
=======
    let Some(&first) = added.first() else {
>>>>>>> replace
<<<<<<< find
        return;
    };
    let specifier = working
=======
        return;
    };
    let moving = added
        .iter()
        .copied()
        .find(|(from, ..)| !working.marks.unreported.contains(*from));
    let (from, line, place, to) = moving.unwrap_or(first);
    let specifier = working
>>>>>>> replace
<<<<<<< find
    let mut locations: Vec<Location> = std::iter::once(finding.location.clone())
        .chain(finding.related.drain(..))
        .filter(|location| location.path != from)
        .collect();

    locations.sort();
    finding.location = Location {
        path: from.to_owned(),
        line: Some(line),
        column: None,
        end_line: None,
    };
    finding.related = locations;
=======

    if moving.is_some() {
        let mut locations: Vec<Location> = std::iter::once(finding.location.clone())
            .chain(finding.related.drain(..))
            .filter(|location| location.path != from)
            .collect();

        locations.sort();
        finding.location = Location {
            path: from.to_owned(),
            line: Some(line),
            column: None,
            end_line: None,
        };
        finding.related = locations;
    }

>>>>>>> replace
```

- [ ] **Step 5: Run the tests to see them pass**

```
cargo test --release --locked --test unreported
```

Expected: PASS.

- [ ] **Step 6: Commit**

```
yak stack format
yak stack check
git add src/entries.rs src/reading.rs src/uses.rs src/findings.rs src/inherit.rs tests/unreported.rs
git commit -m "feat(audit): leave unreported files out of dead code and cycles"
```

Expected: the format and the check report nothing left to fix, and the commit is made.

### Task 3: Complexity

**Implements:** CPLX-1, COUNT-4

**Test:** `cargo test --release --locked --test unreported`

**Files:**
- Modify: `src/complexity.rs`, `src/findings.rs`, `tests/unreported.rs`

**Interfaces:**
- Consumes: Task 1's `Marks::counts`; Task 2's `left` list in `findings::find`.
- Produces: `complexity::left(reading: &Reading, limits: Limits) -> Vec<Vec<String>>`; `measured`
  without unreported files.

- [ ] **Step 1: Write the failing test**

```rust edit path=tests/unreported.rs
<<<<<<< find
    assert!(sentences(&run, "findings-unreported").contains(
        &"The `unreported` glob `docs/**` that the project states for the stack `typescript` left out 1 \
          finding(s)."
    ));
}
=======
    assert!(sentences(&run, "findings-unreported").contains(
        &"The `unreported` glob `docs/**` that the project states for the stack `typescript` left out 1 \
          finding(s)."
    ));
}

/// A body of as many `if`s as given, one point of cognitive complexity each.
fn ifs(count: usize) -> String {
    "if (a) {} ".repeat(count)
}

#[test]
fn no_complexity_is_reported_in_an_unreported_file_by_audit_or_by_complexity() {
    let project = Project::new("unreported-complexity");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { run } from './run';\n\nrun(true);\n",
        )
        .write(
            "src/run.ts",
            &format!(
                "export function run(a: boolean): void {{\n  {}\n}}\n",
                ifs(4)
            ),
        )
        .write(
            "scripts/tool.ts",
            &format!(
                "export function tool(a: boolean): void {{\n  {}\n}}\n",
                ifs(4)
            ),
        )
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "cyclomaticLimit": 10, "cognitiveLimit": 3, "unreported": ["scripts/**"] }),
    );
    let run = audit(&project, &hand_off);

    // `tool` is over the cognitive limit, as `run` is, but only `run` is reported; the script would be
    // an unused file too, and both count (ignoring spec CPLX-1, COUNT-2).
    assert_eq!(found(&run), [("complexity", "src/run.ts", Some(1))]);
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "The `unreported` glob `scripts/**` that the project states for the stack `typescript` left \
          out 2 finding(s)."
        ]
    );

    // The command lists no function of the script, even with `--all`, and counts only its own kind
    // (ignoring spec CPLX-1, COUNT-4).
    for all in [false, true] {
        let document = printed(&project.start("complexity", |binary| {
            binary.args(["--hand-off", &hand_off]);

            if all {
                binary.arg("--all");
            }
        }));
        let paths: Vec<&str> = document["functions"]
            .as_array()
            .expect("a list of functions")
            .iter()
            .map(|function| function["path"].as_str().expect("a path"))
            .collect();

        assert_eq!(paths, ["src/run.ts"]);
        assert_eq!(
            said(&document, "findings-unreported"),
            [
                "The `unreported` glob `scripts/**` that the project states for the stack `typescript` \
              left out 1 finding(s)."
            ]
        );
    }
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see the new one fail**

```
cargo test --release --locked --test unreported
```

Expected: FAIL: `scripts/tool.ts` is reported over the limit, and the counts leave its complexity out.

- [ ] **Step 3: Leave unreported files out of complexity**

```rust edit path=src/complexity.rs
<<<<<<< find
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
=======
/// Every function of the reading's own scripts and Vue components, sorted by path, line and column.
/// A file declared data, generated or unreported, and a type declaration file, gives none (spec FN-2,
/// CPX-5, SOLO-2; ignoring spec CPLX-1).
pub fn measured(reading: &Reading, limits: Limits) -> Vec<Measured> {
    every(reading, limits, false)
}

/// The functions over a limit in the unreported files, which `measured` leaves out, each by its file
/// (ignoring spec CPLX-1, COUNT-2).
pub fn left(reading: &Reading, limits: Limits) -> Vec<Vec<String>> {
    every(reading, limits, true)
        .into_iter()
        .filter(|measured| !measured.over.is_empty())
        .map(|measured| vec![measured.path])
        .collect()
}

/// Every function of the reading's own scripts and Vue components in the unreported files where
/// `unreported` is true, and in every other where it is not, sorted by path, line and column. A file
/// declared data or generated, and a type declaration file, gives none.
fn every(reading: &Reading, limits: Limits, unreported: bool) -> Vec<Measured> {
    let mut measured = Vec::new();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || reading.marks.generated.contains(&file.path)
            || reading.marks.unreported.contains(&file.path) != unreported
        {
            continue;
        }
>>>>>>> replace
<<<<<<< find
    reading.notices.extend(limits.notice());

    let mut document = json!({
        "functions": functions,
=======
    reading.notices.extend(limits.notice());

    // The command says how many of its own findings the `unreported` globs left out (ignoring spec
    // COUNT-4).
    let lost = left(&reading, limits);

    reading.notices.extend(reading.marks.counts(&lost));

    let mut document = json!({
        "functions": functions,
>>>>>>> replace
```

```rust edit path=src/findings.rs
<<<<<<< find
    complex(reading, &mut found);
=======
    complex(reading, &mut found, &mut left);
>>>>>>> replace
<<<<<<< find
/// One finding for each function over a limit, whichever measures are over (complexity spec CPX-1 to
/// CPX-5).
fn complex(reading: &Reading, found: &mut Vec<Finding>) {
    let limits = reading.limits;
=======
/// One finding for each function over a limit, whichever measures are over, and each one in an
/// unreported file left out, by its file (complexity spec CPX-1 to CPX-5; ignoring spec CPLX-1).
fn complex(reading: &Reading, found: &mut Vec<Finding>, left: &mut Vec<Vec<String>>) {
    let limits = reading.limits;

    left.extend(complexity::left(reading, limits));
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```
cargo test --release --locked --test unreported
```

Expected: PASS.

- [ ] **Step 5: Commit**

```
yak stack format
yak stack check
git add src/complexity.rs src/findings.rs tests/unreported.rs
git commit -m "feat(complexity): leave unreported files out of complexity"
```

Expected: the format and the check report nothing left to fix, and the commit is made.

### Task 4: Duplication

**Implements:** DUPE-1, COUNT-4

**Test:** `cargo test --release --locked --test unreported`

**Files:**
- Modify: `src/duplication.rs`, `src/findings.rs`, `src/inherit.rs`, `tests/unreported.rs`

**Interfaces:**
- Consumes: Task 1's `Marks::counts`; Task 2's `left` list in `findings::find`.
- Produces: `duplication::Grouped { groups, notices, left }`, which `duplication::groups` returns;
  `Place::tokens` and `Place::lines`.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/unreported.rs
<<<<<<< find
        assert_eq!(paths, ["src/run.ts"]);
        assert_eq!(
            said(&document, "findings-unreported"),
            [
                "The `unreported` glob `scripts/**` that the project states for the stack `typescript` \
              left out 1 finding(s)."
            ]
        );
    }
}
=======
        assert_eq!(paths, ["src/run.ts"]);
        assert_eq!(
            said(&document, "findings-unreported"),
            [
                "The `unreported` glob `scripts/**` that the project states for the stack `typescript` \
              left out 1 finding(s)."
            ]
        );
    }
}

/// A function whose statements are lines 2 to 6 of it, a run of 20 tokens.
fn body(name: &str) -> String {
    format!(
        "export function {name}(items: number[]) {{\n  const total = items.length;\n  if (total > 10) {{\n    return \
         total * 2;\n  }}\n  return total;\n}}\n"
    )
}

/// A function whose statements differ from another's only where its number does: lines 2 to 6 of it.
fn counting(name: &str, number: u32) -> String {
    format!(
        "export function {name}(items: number[]) {{\n  const total = items.length + {number};\n  if (total > \
         {number}) {{\n    return total * {number};\n  }}\n  return total;\n}}\n"
    )
}

/// The duplication findings of a run, in its order: each with its copies, its first location first,
/// each as its file with its first and last lines.
fn duplicates(run: &Value) -> Vec<Vec<(&str, u64, u64)>> {
    results(run)
        .iter()
        .filter(|result| result["ruleId"] == json!("duplication"))
        .map(|result| {
            result["locations"]
                .as_array()
                .into_iter()
                .chain(result["relatedLocations"].as_array())
                .flatten()
                .map(|location| {
                    let place = &location["physicalLocation"];

                    (
                        place["artifactLocation"]["uri"].as_str().expect("a file"),
                        place["region"]["startLine"].as_u64().expect("a first line"),
                        place["region"]["endLine"].as_u64().expect("a last line"),
                    )
                })
                .collect()
        })
        .collect()
}

#[test]
fn a_group_loses_its_copies_in_unreported_files_and_one_left_too_small_is_left_out() {
    let project = Project::new("unreported-duplication");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { a } from './a';\nimport { b } from './b';\nimport { d } from './d';\n\na([]);\nb([]);\n\
             d([]);\n",
        )
        .write("src/a.ts", &body("a"))
        .write("src/b.ts", &body("b"))
        .write("scripts/x.ts", &body("x"))
        .write("src/d.ts", &counting("d", 7))
        .write("scripts/y.ts", &counting("y", 7))
        .commit();

    let plain = audit(
        &project,
        &declaring(&project, json!({ "duplicationMinimum": 10 })),
    );
    let wide = fingerprint(&plain, "duplication", "scripts/x.ts").to_owned();
    let narrow = fingerprint(&plain, "duplication", "scripts/y.ts").to_owned();
    let with = |accepted: Vec<String>| {
        declaring(
            &project,
            json!({
                "duplicationMinimum": 10,
                "unreported": ["scripts/**"],
                "duplicationAccepted": accepted,
            }),
        )
    };
    let run = audit(&project, &with(Vec::new()));

    // The group of three loses its copy in the script and keeps its fingerprint, its size worked out
    // from the two copies left; the group of two is left with one copy, and left out (ignoring spec
    // DUPE-1).
    assert_eq!(
        duplicates(&run),
        [vec![("src/a.ts", 2, 6), ("src/b.ts", 2, 6)]]
    );
    assert_eq!(fingerprint(&run, "duplication", "src/a.ts"), wide);
    assert_eq!(
        message(&run, "duplication", "src/a.ts"),
        "2 copies of a 5-line run (20 tokens): `src/a.ts` lines 2–6, `src/b.ts` lines 2–6."
    );
    // The two scripts would be unused files, and the group of two would be reported (ignoring spec
    // COUNT-2).
    assert_eq!(
        sentences(&run, "findings-unreported"),
        [
            "The `unreported` glob `scripts/**` that the project states for the stack `typescript` left \
          out 3 finding(s)."
        ]
    );

    // An entry for the group's copies before the removal or after accepts it; an entry for the group
    // left out matches it, so it gives no notice, and the group is not counted (ignoring spec DUPE-1).
    for entry in [format!("{wide}/3"), format!("{wide}/2")] {
        let accepted = audit(&project, &with(vec![entry, format!("{narrow}/2")]));

        assert!(duplicates(&accepted).is_empty());
        assert!(sentences(&accepted, "accepted-group-not-found").is_empty());
        assert_eq!(
            sentences(&accepted, "findings-unreported"),
            [
                "The `unreported` glob `scripts/**` that the project states for the stack `typescript` \
              left out 2 finding(s)."
            ]
        );
    }

    // The command counts only the groups it left out (ignoring spec COUNT-4).
    let document = printed(&project.start("duplication", |binary| {
        binary.args(["--hand-off", &with(Vec::new())]);
    }));

    assert_eq!(
        said(&document, "findings-unreported"),
        [
            "The `unreported` glob `scripts/**` that the project states for the stack `typescript` left \
          out 1 finding(s)."
        ]
    );

    // The base's group loses its copy in the script too, so it pairs with the branch's (ignoring spec
    // DUPE-1).
    project.git(&["switch", "--quiet", "-c", "branch"]);

    let based = audited(
        &project,
        &["--hand-off", &with(Vec::new()), "--base", "main"],
    );

    assert_eq!(
        states(&based, "duplication"),
        [("src/a.ts", Some(2), Some("unchanged"))]
    );
}

#[test]
fn a_group_kept_only_by_its_unreported_copy_is_dropped_inside_a_wider_one() {
    let project = Project::new("unreported-inside");
    let both = "const tripled = [4, 5, 6].map((value) => value * 3);\nconst doubled = [1, 2, 3].map((value) => \
                value * 2);\n";

    project
        .write("src/p.ts", both)
        .write("src/q.ts", both)
        .write(
            "scripts/r.ts",
            "const other = 0;\nconst doubled = [1, 2, 3].map((value) => value * 2);\n",
        )
        .commit();

    let plain = audit(
        &project,
        &declaring(&project, json!({ "duplicationMinimum": 10 })),
    );
    let run = audit(
        &project,
        &declaring(
            &project,
            json!({ "duplicationMinimum": 10, "unreported": ["scripts/**"] }),
        ),
    );

    // Its copy in `scripts/r.ts` keeps the run of `doubled` a group of its own; without it, the group
    // lies inside the wider one and is dropped, not reported twice (spec SAME-4; ignoring spec DUPE-1).
    assert_eq!(
        duplicates(&plain),
        [
            vec![
                ("scripts/r.ts", 2, 2),
                ("src/p.ts", 2, 2),
                ("src/q.ts", 2, 2)
            ],
            vec![("src/p.ts", 1, 2), ("src/q.ts", 1, 2)],
        ]
    );
    assert_eq!(
        duplicates(&run),
        [vec![("src/p.ts", 1, 2), ("src/q.ts", 1, 2)]]
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see the new ones fail**

```
cargo test --release --locked --test unreported
```

Expected: FAIL: the groups keep their copies in the scripts, and no group is counted.

- [ ] **Step 3: Remove the copies in unreported files**

```rust edit path=src/duplication.rs
<<<<<<< find
//! document the command prints (duplication spec SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7,
//! DUP-2, DUP-4, QUIET-1 to QUIET-4, SHOW-1 to SHOW-5).
=======
//! document the command prints (duplication spec SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7,
//! DUP-2, DUP-4, QUIET-1 to QUIET-4, SHOW-1 to SHOW-5); and each group's copies in unreported files
//! removed (ignoring spec DUPE-1).
>>>>>>> replace
<<<<<<< find
    /// Its exact form: the fingerprint its run would have with nothing loosened (spec FRESH-1).
    pub exact: String,
}
=======
    /// Its exact form: the fingerprint its run would have with nothing loosened (spec FRESH-1).
    pub exact: String,
    /// Its tokens (spec SIZE-1).
    pub tokens: u64,
    /// Its lines (spec SIZE-1).
    pub lines: u64,
}
>>>>>>> replace
<<<<<<< find
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
=======
/// The groups of copies a reading holds, with what the run says of them (spec QUIET-1, QUIET-2;
/// ignoring spec DUPE-1, COUNT-2).
#[derive(Debug, Default)]
pub struct Grouped {
    /// Every group, each marked accepted or not, sorted by its first copy.
    pub groups: Vec<Group>,
    /// A notice for each entry of `duplicationAccepted` that matches no group.
    pub notices: Vec<Notice>,
    /// Each group the unreported files left with too few copies that nobody accepted, by the files of
    /// the copies it lost there.
    pub left: Vec<Vec<String>>,
}

/// Every group of copies in the reading's own scripts and Vue components, each run as large as it goes,
/// with its copies in unreported files removed and dropped where too few are left, sorted by its first
/// copy and marked accepted where an entry names its fingerprint and its copies before the removal or
/// after, with a notice for each entry no group matches; none where no `duplicationMinimum` is
/// declared. A file declared data or generated, and a type declaration file, holds no copy (spec
/// COPY-4, SAME-3, SAME-4, SIZE-2, QUIET-1, QUIET-2; ignoring spec DUPE-1).
pub fn groups(reading: &Reading) -> Grouped {
    let settings = &reading.duplication;
    let Some(minimum) = settings.minimum_tokens else {
        return Grouped::default();
    };
>>>>>>> replace
<<<<<<< find
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
=======
    let accepted: BTreeSet<&str> = settings
        .accepted
        .iter()
        .map(|accepted| accepted.entry.as_str())
        .collect();
    // Every entry a group can be accepted by: its own as it was found, and as it is once its copies in
    // unreported files are removed.
    let mut entries: BTreeSet<String> = BTreeSet::new();
    let mut left = Vec::new();
    let mut reduced = Vec::new();
    let mut found_as = Vec::new();

    for group in widest(found) {
        let before = accepting(&group);
        let (kept, lost): (Vec<Place>, Vec<Place>) = group
            .copies
            .iter()
            .cloned()
            .partition(|copy| !reading.marks.unreported.contains(&copy.path));

        if u64::try_from(kept.len()).unwrap_or(u64::MAX) < settings.copies {
            // A group left with too few copies is left out, and counted where nobody accepted it.
            if !accepted.contains(before.as_str()) {
                left.push(lost.into_iter().map(|copy| copy.path).collect());
            }
        } else {
            reduced.push(Group {
                tokens: kept
                    .iter()
                    .map(|copy| copy.tokens)
                    .min()
                    .unwrap_or(group.tokens),
                lines: kept
                    .iter()
                    .map(|copy| copy.lines)
                    .min()
                    .unwrap_or(group.lines),
                copies: kept,
                ..group
            });
            found_as.push(before.clone());
        }

        entries.insert(before);
    }

    // A group kept only by its copy in an unreported file can now lie inside a wider one, so the rule
    // runs again (spec SAME-4). Acceptance comes after, so an accepted group still hides the groups
    // inside it (spec QUIET-1).
    let outer = outermost(&reduced);
    let mut groups: Vec<Group> = Vec::new();

    for ((mut group, before), kept) in reduced.into_iter().zip(found_as).zip(outer) {
        if !kept {
            continue;
        }

        let after = accepting(&group);

        group.accepted = accepted.contains(before.as_str()) || accepted.contains(after.as_str());
        entries.insert(after);
        groups.push(group);
    }

    let mut named = BTreeSet::new();
>>>>>>> replace
<<<<<<< find
    for group in &mut groups {
        group
            .copies
            .sort_by(|a, b| (&a.path, a.start_line).cmp(&(&b.path, b.start_line)));
        group.accepted = accepted.contains(accepting(group).as_str());
    }

    groups.sort_by(|a, b| order(a).cmp(&order(b)));

    (groups, notices)
}
=======
    for group in &mut groups {
        group
            .copies
            .sort_by(|a, b| (&a.path, a.start_line).cmp(&(&b.path, b.start_line)));
    }

    groups.sort_by(|a, b| order(a).cmp(&order(b)));

    Grouped {
        groups,
        notices,
        left,
    }
}
>>>>>>> replace
<<<<<<< find
        exact: fingerprint(run, |item| &item.exact),
    }
}
=======
        exact: fingerprint(run, |item| &item.exact),
        tokens: tokens(run),
        lines: lines(run),
    }
}
>>>>>>> replace
<<<<<<< find
/// The groups left once each that lies inside another is dropped: one whose every copy lies inside a
/// different copy of another group with at least as many copies. Of two that each lie inside the other,
/// the first stays (spec SAME-4).
fn widest(found: Vec<Group>) -> Vec<Group> {
=======
/// The groups left once each that lies inside another is dropped (spec SAME-4).
fn widest(found: Vec<Group>) -> Vec<Group> {
    let kept = outermost(&found);

    found
        .into_iter()
        .zip(kept)
        .filter_map(|(group, kept)| kept.then_some(group))
        .collect()
}

/// For each group, whether it stays: not one whose every copy lies inside a different copy of another
/// group with at least as many copies. Of two that each lie inside the other, the first stays (spec
/// SAME-4).
fn outermost(found: &[Group]) -> Vec<bool> {
>>>>>>> replace
<<<<<<< find
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
=======
    found
        .iter()
        .enumerate()
        .map(|(at, group)| {
            !found.iter().enumerate().any(|(other_at, other)| {
                other_at != at && inside(group, other) && !(other_at > at && inside(other, group))
            })
        })
        .collect()
}
>>>>>>> replace
<<<<<<< find
    let (groups, found) = groups(&reading);
    let listed: Vec<Value> = groups
        .iter()
        .filter(|group| all || !group.accepted)
        .map(entry)
        .collect();

    reading.notices.extend(found);
    reading.notices.extend(reading.duplication.notice());
=======
    let grouped = groups(&reading);
    let listed: Vec<Value> = grouped
        .groups
        .iter()
        .filter(|group| all || !group.accepted)
        .map(entry)
        .collect();

    reading.notices.extend(grouped.notices);
    reading.notices.extend(reading.duplication.notice());
    // The command says how many groups the `unreported` globs left out (ignoring spec COUNT-4).
    reading.notices.extend(reading.marks.counts(&grouped.left));
>>>>>>> replace
```

```rust edit path=src/findings.rs
<<<<<<< find
    duplicated(reading, &mut found, &mut notices);
=======
    duplicated(reading, &mut found, &mut notices, &mut left);
>>>>>>> replace
<<<<<<< find
/// One finding for each group of copies a person has not accepted, with the notices for an entry of
/// `duplicationAccepted` that matches no group and for a run with no `duplicationMinimum` (duplication
/// spec DUP-1, TUNE-1, QUIET-1, QUIET-2).
fn duplicated(reading: &Reading, found: &mut Vec<Finding>, notices: &mut Vec<Notice>) {
    let (groups, unmatched) = duplication::groups(reading);

    found.extend(
        groups
            .into_iter()
            .filter(|group| !group.accepted)
            .filter_map(Finding::grouped),
    );
    notices.extend(unmatched);
    notices.extend(reading.duplication.notice());
}
=======
/// One finding for each group of copies a person has not accepted, with the notices for an entry of
/// `duplicationAccepted` that matches no group and for a run with no `duplicationMinimum`, and each
/// group the unreported files left out, by the files of the copies it lost there (duplication spec
/// DUP-1, TUNE-1, QUIET-1, QUIET-2; ignoring spec DUPE-1).
fn duplicated(
    reading: &Reading,
    found: &mut Vec<Finding>,
    notices: &mut Vec<Notice>,
    left: &mut Vec<Vec<String>>,
) {
    let grouped = duplication::groups(reading);

    found.extend(
        grouped
            .groups
            .into_iter()
            .filter(|group| !group.accepted)
            .filter_map(Finding::grouped),
    );
    notices.extend(grouped.notices);
    notices.extend(reading.duplication.notice());
    left.extend(grouped.left);
}
>>>>>>> replace
```

```rust edit path=src/inherit.rs
<<<<<<< find
    // The base's groups before acceptance, by fingerprint: each one's copies as their files on the
    // branch with their exact forms, sorted (duplication spec FRESH-1).
    let mut held: BTreeMap<String, Vec<Vec<(String, String)>>> = BTreeMap::new();

    for group in duplication::groups(&base.reading).0 {
=======
    // The base's groups before acceptance, with their copies in unreported files removed, by
    // fingerprint: each one's copies as their files on the branch with their exact forms, sorted
    // (duplication spec FRESH-1; ignoring spec DUPE-1).
    let mut held: BTreeMap<String, Vec<Vec<(String, String)>>> = BTreeMap::new();

    for group in duplication::groups(&base.reading).groups {
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass, then every gate**

```
cargo test --release --locked --test unreported
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: PASS for the new tests, and every gate passes: a run without the key reports what it reported
before, so no other test changes.

- [ ] **Step 5: Commit**

```
yak stack format
yak stack check
git add src/duplication.rs src/findings.rs src/inherit.rs tests/unreported.rs
git commit -m "feat(duplication): remove copies in unreported files from groups"
```

Expected: the format and the check report nothing left to fix, and the commit is made.

### Task 5: The interface paragraph

**Implements:** VERS-1, REFUSE-1

**Test:** `grep -q "findings-unreported" CONTRIBUTING.md && grep -q "declaration keys it knows" CONTRIBUTING.md`

**Files:**
- Modify: `CONTRIBUTING.md`

**Interfaces:**
- Consumes: Tasks 1 to 4's key, notices and `interface` output.
- Produces: CONTRIBUTING.md's account of what the key adds to interface version 1.

- [ ] **Step 1: Write the paragraph**

```markdown edit path=CONTRIBUTING.md
<<<<<<< find
- `yak-analyzer interface` prints one line of JSON: the interface version, and the hand-off
  formats the analyzer reads.
=======
- `yak-analyzer interface` prints one line of JSON: the interface version, the hand-off formats
  the analyzer reads, and the declaration keys it knows.
>>>>>>> replace
<<<<<<< find
the interface version, and a change to the form stops every accepted group and recorded reason from
matching.
=======
the interface version, and a change to the form stops every accepted group and recorded reason from
matching.

Ignoring files adds to interface version 1 and breaks nothing in it: the key `unreported`, a list of
globs that only a project states, whose files are read like any other but in which no finding is
reported; and the list of declaration keys `interface` prints, from which yak can tell that a
project states a key newer than the analyzer its pack pins. `audit`, `complexity` and `duplication`
say in notices of the kind `findings-unreported` how many findings each glob left out; a notice's
kind is not part of the interface. The analyzer release that knows a new key comes before any
project writes it, and the pack pins that release first.
>>>>>>> replace
```

- [ ] **Step 2: Run the check**

```
grep -q "findings-unreported" CONTRIBUTING.md && grep -q "declaration keys it knows" CONTRIBUTING.md
```

Expected: exit 0.

- [ ] **Step 3: Commit**

```
yak stack format
yak stack check
git add CONTRIBUTING.md
git commit -m "docs: describe what the unreported key adds to the interface"
```

Expected: the format and the check report nothing left to fix, and the commit is made.

### Task 6: What is owed to yakshaver

**Implements:** OWED-1

**Check:** the owner reads, in the yakshaver work item `2026-11-18-analyzer-design-spec-stage-2-amendments`
(or its successor where it has closed), each of OWED-1's five entries filed on its record; or, in the
judged list, each written as still owed there.

**Files:**
- None in this repository: each entry is filed on the yakshaver item's record.

**Interfaces:**
- Consumes: the spec's OWED-1 list and this branch's `interface` output.
- Produces: OWED-1's entries on the yakshaver item's record.

- [ ] **Step 1: File each entry**

Read the yakshaver item's state. Where it is open, file on its record, each as one entry: the
design-spec amendment adding `unreported` (KEY-6, KEY-7, COUNT-1); that DECL-2's `yak stack` commands
accept `unreported` for a project and refuse it for a pack; that the engine shows the
`findings-unreported` notices at the gate and that HAND-7's question covers a new `unreported` glob; a
check that yak passes a project's `unreported` list into the hand-off file as it passes `data` and
`generated`; the release order of REFUSE-1; and that the engine reads `interface`'s `declarationKeys`
and, for a project key the pinned analyzer does not know, stops with a line saying to update the pack.
Where it has closed, file them on its successor, or write each in the judged list as still owed.

```
yak work show --item 2026-11-18-analyzer-design-spec-stage-2-amendments
```

Expected: the item's state; then each entry on its record, or each written in the judged list as
still owed there.

- [ ] **Step 2: Record what was filed**

Record a ruling on Task 6 naming each entry and where it was filed.

Expected: the ruling is on the record, and the owner reads it at validation.
