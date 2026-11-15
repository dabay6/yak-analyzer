# Dead-code findings, part 5: reading a base commit, new against inherited, and the life of a run: implementation plan

**Goal:** `audit --base <commit>` reads the base commit from git's stored files as well as the working
tree, and marks each finding `new`, or `unchanged` where the base has it too, through git's renames and
by the edges of each cycle; and the binary ends with the launcher that started it, leaves no git or lock
behind when it is killed, and runs no project out of stack.

**Architecture:** Every reader that read a listed file from disk now reads it through a `Tree`
(`src/tree.rs`): the working tree, or a commit whose file list comes from `git ls-tree` and whose
contents come from one `git cat-file --batch`. `reading::read` takes a tree, its file list and the
hand-off file, so the base is read exactly as the working tree is, on a thread of its own beside it.
`src/inherit.rs` compares the two sides' findings; `src/launcher.rs` watches the npm launcher, and every
reading runs on a thread with a large stack.

**Spec:** docs/specs/2026-11-13-dead-code-findings-design.md

**Implements:** CMD-1, CMD-2, CMD-3, COMMIT-1 to COMMIT-5, INH-1 to INH-4, REPORT-1, REPORT-4, LIFE-1 to LIFE-3, FIX-1, FIX-2, FIX-4

The fifth of the item's six plans, listed in plan 1. Plan 4 has merged, so this plan's branch,
`feat/dead-code-5-base`, is made from `main`, and its pull request goes against `main`. Plan 4 left
`--base` to this plan: CMD-2, the base's half of CMD-1 and CMD-3 (the stops 7b and 7c), REPORT-1's
`baselineState` and REPORT-4's notifications of the base. Plan 6 runs the analyzer on the named
repositories (LIVE-1 to LIVE-8) and changes the README and `CONTRIBUTING.md` (DOC-1).

## Global constraints

- Nothing is checked out, no worktree is made, and nothing is written, inside the root or `.git`; no
  network is used (COMMIT-1, reading RUN-6).
- A commit's contents are read in one batch from git, never with one start of git per file, as git
  stores them, with no filter applied (COMMIT-2).
- Lazy fetching is turned off for every start of git; a partial clone with a git too old to honour that
  stops an audit with a base before the base is read (COMMIT-2, CMD-3 stop 7c).
- A base that begins with `-` or holds a control character is refused before git sees it, exit 2; only
  git's full name for the base is used afterwards (CMD-2).
- At the base, the project's own files stop nothing; git failing does (COMMIT-4).
- The output is the same bytes whether the two sides are read in parallel or not (INH-4, reading RUN-8).
- The binary ends within five seconds of the launcher that started it, writing nothing (LIFE-1).
- A killed audit leaves no lock, and within thirty seconds no git it started (LIFE-2).
- An import chain, a cycle and a re-export chain of 10,000 files each, and a file nested 1,000 levels
  deep, run on every target (LIFE-3).
- The interface version stays 1: `--base` joins the drafts (CMD-5).

## The decisions this plan makes

1. **Three tasks.** Task 1 reads the base and marks findings new or inherited by fingerprint, with
   COMMIT-4's skipping; task 2 adds renames (INH-2) and judges cycles by their edges (INH-3), which
   until then are compared by fingerprint like the other kinds; task 3 is the life of a run.
2. **One reading, two trees (COMMIT-1).** `tree::Tree` is the working tree on disk, or a commit. The
   `package.json` and workspace-list reader, the tsconfig reader (a config above the root included), the
   script reader and the reader of `configEntries`' configuration files all read a listed file through
   `Tree::read`; the library that finds a governing tsconfig already read only the analyzer's own file
   system. `reading::start` reads the hand-off file and finds the root once; `reading::read` reads one
   side from its tree and its file list. `graph` reads the working tree as it did.
3. **A commit's files (COMMIT-2).** `git ls-tree -r -z <commit>`, started in the root, lists only what
   lies under it, with paths from it. A `blob` entry is a file, tracked; mode `120000` is a link and a
   `commit` entry a submodule, both left out as on the working tree, with the same notice for links.
   Contents come from one `git cat-file --batch` per base reading, asked by each file's object id, and a
   tsconfig above the root by `<commit>:<path from the top>`; `cat-file --batch` applies no filter. The
   batch's input is closed when the reading ends, which ends it (LIFE-2).
4. **Lazy fetching.** Every start of git gets `GIT_NO_LAZY_FETCH=1`, which git honours from 2.44. A
   repository is a partial clone when its settings hold `extensions.partialClone` or a `remote.*.promisor`
   that is true; then a git older than 2.44 stops an audit with a base, naming both versions (stop 7c).
   The DEP-2 floor stays at 2.15: every command this plan adds, `rev-parse --verify --quiet`,
   `ls-tree -r -z`, `cat-file --batch`, `config --list -z` and `diff --name-status -z --relative -M -l`,
   is older than it.
5. **The base's name (CMD-2).** `audit` takes `--hand-off` and `--base` in either order, each once. A base
   that is empty, begins with `-` or holds a control character is refused with a line of its own, exit 2.
   Otherwise git is asked for `<name>^{commit}` with `rev-parse --verify --quiet`, so a name of a file,
   such as `HEAD:package.json`, is not a commit either; a name that is not a commit stops the run (stop
   7b) with the name and `git fetch` as the fix. The base is asked after the working tree's file list
   (stops 6, 7 and 7a) and before anything else is read.
6. **Skipping at the base (COMMIT-4).** Each check of RUN-4 from step 8 on skips its file at the base
   with a notice of the kind `file-skipped`, whose sentence is the stop's own without its fix: a
   `package.json` makes no package, a workspace list adds no folder, a tsconfig governs nothing and
   leaves every other config's `extends` and `references`, so the search for a governing one goes on
   upward, and a source file has no imports and no exports. git failing while the base is read stops the
   run with git's first line, once the reading ends.
7. **All new after a skip.** When the base skipped any file, every finding is `new`, and one notification,
   `base-files-skipped`, names the files.
8. **Counting (INH-1).** The base's findings are counted by fingerprint; the working tree's, in the
   report's order, take the base's count first by line and column, and the rest are new. A finding only
   the base has is not printed.
9. **Notifications (REPORT-4).** One notification, `base`, names the base's full name; it and the other
   notifications of the comparison are the run's, not the base's. Each notice of the base's reading is a
   notification with the property `base: true`, after the working tree's. The notices `audit` adds from
   the findings, ENTRY-9's and ENTRY-10's (plan 4's decision 7), are the working tree's alone: on the base
   they say nothing a person can act on.
10. **In parallel (INH-4).** The base is read on a scoped thread beside the working tree. Each side has
    its own notices, and the report sorts them, so the bytes do not depend on which ends first.
11. **Renames (INH-2).** `git diff --no-ext-diff --name-status -z --relative -M -l1000 <commit> --`,
    started in the root, compares the commit with the working tree for every path the index holds, so
    staged additions take part and untracked files do not. The limit is 1000, git's own default; a warning
    that names renames becomes the notification `renames-cut-short`. A finding keeps the parts its
    fingerprint is made from, so a base finding's fingerprint is made again with each renamed path moved
    to its new one, and a cycle's paths sorted again.
12. **Cycles (INH-3).** A working cycle is `unchanged` when the base's strongly connected sets, with
    renamed paths moved, put all its files in one set, and every edge between two of its files is an edge
    of the base. Otherwise it is new; when the branch added an edge, its first location is the first such
    import by path and line, its related locations the set's other files, and its message gains one
    sentence naming the import's specifier, its line, its file and the file it leads to.
13. **The launcher (LIFE-1).** The launcher starts the binary with `YAK_ANALYZER_LAUNCHER` set to its own
    process id. On macOS and Linux the binary looks every 200 milliseconds whether its parent is still that
    process, since a child whose parent ends is given another, and ends with exit 1 and nothing written
    when it is not. On Windows Node starts every child in a job that the system ends when Node ends, so
    the binary ends with the launcher already; the test shows it on every target. No crate is added, and
    nothing is `unsafe`.
14. **The stack (LIFE-3).** Every reading runs on a thread with 256 MiB of stack reserved: the main work
    on one, the base on another. The system gives memory to a stack only as it is used. On the 1 MiB main
    thread of Windows, the deeply nested file overflows it.
15. **The long project (LIFE-3).** One project of 10,000 files, each passing on the next one's exports
    with `export *` and the last importing the first, is at once the import chain, the cycle and the
    re-export chain; a file beside it nests 1,000 levels in parentheses and in blocks. The tests make it
    once a run and commit it, so it is also the base the LIFE-2 test kills an audit in.
16. **Where the tests go.** The base's tests join `tests/audit.test.rs`, since a target that includes
    `tests/support/project.rs` must use every helper it holds or the lint fails on the unused ones, and
    `audit`'s target uses them all. The life tests are a target of their own, `tests/life.test.rs`, which
    includes nothing; its `[[test]]` entry is in `Cargo.toml`, applied with the code, so task 3's test
    command runs the launcher's tests first, which fail before the code for the right reason.
17. **The README and `CONTRIBUTING.md` wait for plan 6 (DOC-1),** as plan 4 left them, with `audit`'s
    options and the note that a partial clone needs git 2.44.

## Review focus

- **Speed.** On three local checkouts of 2,600 to 3,800 files, an audit with a base 20 commits back took
  1.6 to 2.2 seconds, against 1.1 to 1.4 without. LIVE-5 measures it in plan 6.
- **The long project on Windows.** Writing and committing 10,000 new files, and reading them the first
  time, took about two and a half minutes here, most of it the system scanning new files; the life tests
  take about three minutes on this machine. The warm audit of a 10,000-file chain takes about 7 seconds,
  and grew faster than its length from 5,000 files (1.9 seconds); that is for stage 4 if LIVE-5 shows it
  on a real repository.
- **Stop 7c** cannot be caused with a git of 2.44 or newer. The partial-clone test expects the stop with
  an older git and a report with a newer one, so which half runs depends on each CI image's git.
- **No fixture** holds a stored symbolic link or submodule at the base, a base whose root folder did not
  exist yet, more renames than git's limit, or a cycle whose base set a skipped file cut.
- **Windows and LIFE-1** rely on Node's job for its children. A launcher started detached from its own
  job would leave the binary running on Windows; yak starts it as an ordinary child.
- **A finding moved by git's rename detection** onto an unrelated file of similar content is inherited
  wrongly; the branch audits of LIVE-3 show how often.

### Task 1: reading the base, and new against inherited

**Implements:** CMD-1, CMD-2, CMD-3, COMMIT-1 to COMMIT-5, INH-1, INH-4, REPORT-1, REPORT-4, FIX-2, FIX-4

**Test:** `cargo test --release --locked --test cli && cargo test --release --locked --test audit`

**Files:**
- Create: `src/tree.rs`, `src/inherit.rs`
- Modify: `src/git.rs`, `src/files.rs`, `src/reading.rs`, `src/packages.rs`, `src/tsconfig.rs`,
  `src/scripts.rs`, `src/entries.rs`, `src/graph.rs`, `src/notice.rs`, `src/stop.rs`, `src/findings.rs`,
  `src/audit.rs`, `src/cli.rs`, `src/main.rs`
- Test: `tests/cli.test.rs`, `tests/audit.test.rs`

**Interfaces:**
- Consumes: plan 4's `reading::read`, `audit::printed`, `findings::find` and `Finding`, `Notice`,
  `cli::Command::Audit`; the readers `packages::load`, `tsconfig::load`, `scripts::read` and
  `entries::find`.
- Produces: `tree::Tree` (`working`, `base`, `root`, `is_base`, `read`, `above`, `failure`);
  `files::stored`; `git::commit`, `git::partial`, `git::older_than`, `git::prefix`,
  `git::leaves_work_tree`, `git::stored`, `git::Batch`, `git::NO_LAZY_FETCH`; `reading::start` and
  `Start`, `reading::working`, `reading::read(handed, tree, listing, notices)`, `Reading::handed` as a
  flag; `Notice::skipped`, `Notice::of_base`, `Notice::is_base`, `notice::SKIPPED`; `Stop::reason`,
  `Fault::parts`; `findings::Baseline`, `Finding::baseline`, `findings::sort`; `inherit::Base` and
  `inherit::mark`; `audit::run(hand_off, base)`; `Command::Audit { hand_off, base }`. Task 2 adds the
  renames to `Base` and the cycles to `mark`; task 3 runs the readings on threads with a large stack.

- [ ] **Step 1: Write the failing tests**

The command line refuses a base git would read as an option, and each of `audit`'s options twice:

```rust edit path=tests/cli.test.rs
<<<<<<< find

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    // `audit` takes no base yet: reading a commit comes with the plan after this one.
    let cases: [&[&str]; 8] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
=======

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    // `graph` takes no base, and `audit` takes each option once, with its value.
    let cases: [&[&str]; 10] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
>>>>>>> replace
<<<<<<< find
        &["graph", "extra"],
        &["graph", "--base", "main"],
        &["audit", "extra"],
        &["audit", "--base", "main"],
    ];

    for args in cases {
=======
        &["graph", "extra"],
        &["graph", "--base", "main"],
        &["audit", "extra"],
        &["audit", "--base"],
        &["audit", "--base", "main", "--base", "main"],
        &["audit", "--hand-off", "a.json", "--hand-off", "b.json"],
    ];

    for args in cases {
>>>>>>> replace
<<<<<<< find
    }
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
=======
    }
}

#[test]
fn refuses_a_base_git_would_read_as_an_option_before_git_sees_it() {
    // A base that begins with `-`, such as one of git's options that writes a file, or that holds a
    // control character, or that is empty, is refused as a wrong option is, before any git starts
    // (dead-code spec CMD-2).
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-base-refused");
    let _ = fs::remove_dir_all(&folder);

    fs::create_dir_all(&folder).expect("the test's folder can be made");

    for base in ["--output=written.txt", "-", "main\nother", ""] {
        let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
            .args(["audit", "--base", base])
            .current_dir(&folder)
            .output()
            .expect("the binary starts");
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{base:?}");
        assert!(output.stdout.is_empty(), "{base:?}");
        assert_eq!(error.lines().count(), 1, "{base:?}");
        assert!(error.starts_with("yak-analyzer: the base `"), "{error}");
        assert!(error.contains("name the commit another way"), "{error}");
    }

    assert_eq!(
        fs::read_dir(&folder)
            .expect("the folder can be read")
            .count(),
        0
    );
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
>>>>>>> replace
```

Seven tests join `tests/audit.test.rs`, each with its base as a commit on `main` and its branch as later
commits and edits: new against inherited, reading the commit and writing nothing, the notifications, a
base that is not a commit, a base whose files cannot be read, the declarations and ignore rules at the
base, and a partial clone:

```rust edit path=tests/audit.test.rs
<<<<<<< find
//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};
=======
//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4).

#[path = "support/project.rs"]
mod project;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};
>>>>>>> replace
<<<<<<< find
const SCHEMA: &str = include_str!("sarif/sarif-schema-2.1.0.json");

/// The one run of the report `audit` prints for the checkout, with the hand-off file given or with
/// none. The run must succeed and write no line, and the schema must accept the report (spec FIX-3).
fn audit(project: &Project, hand_off: Option<&str>) -> Value {
    let report = printed(&project.start("audit", |binary| {
        if let Some(file) = hand_off {
            binary.args(["--hand-off", file]);
        }
    }));
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema's copy is JSON");
    let faults: Vec<String> = jsonschema::validator_for(&schema)
=======
const SCHEMA: &str = include_str!("sarif/sarif-schema-2.1.0.json");

/// The one run of the report `audit` prints for the checkout, with the hand-off file given or with
/// none.
fn audit(project: &Project, hand_off: Option<&str>) -> Value {
    match hand_off {
        Some(file) => audited(project, &["--hand-off", file]),
        None => audited(project, &[]),
    }
}

/// The one run of the report `audit` prints for the checkout with the options given. The run must
/// succeed and write no line, and the schema must accept the report (spec FIX-3).
fn audited(project: &Project, options: &[&str]) -> Value {
    let report = printed(&project.start("audit", |binary| {
        binary.args(options);
    }));
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema's copy is JSON");
    let faults: Vec<String> = jsonschema::validator_for(&schema)
>>>>>>> replace
<<<<<<< find
        Some(11)
    );
}
=======
        Some(11)
    );
}

/// The findings of a run, in its order: each with its rule, its file as the report writes it, its line
/// where it has one, and whether it is new or inherited where the run says.
fn states(run: &Value) -> Vec<(&str, &str, Option<u64>, Option<&str>)> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .map(|result| {
            let place = &result["locations"][0]["physicalLocation"];

            (
                result["ruleId"].as_str().expect("a rule"),
                place["artifactLocation"]["uri"].as_str().expect("a file"),
                place["region"]["startLine"].as_u64(),
                result["baselineState"].as_str(),
            )
        })
        .collect()
}

/// The notifications of a run, in its order: each with its kind, the file it is about as the report
/// writes it, and whether the base's reading gave it.
fn marked(run: &Value) -> Vec<(&str, Option<&str>, bool)> {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .map(|notification| {
            (
                notification["descriptor"]["id"].as_str().expect("a kind"),
                notification["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str(),
                notification["properties"]["base"] == json!(true),
            )
        })
        .collect()
}

/// The sentence of the notification of the kind given.
fn sentence<'r>(run: &'r Value, kind: &str) -> &'r str {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .find(|notification| notification["descriptor"]["id"] == json!(kind))
        .and_then(|notification| notification["message"]["text"].as_str())
        .unwrap_or_else(|| panic!("a notification of the kind {kind}"))
}

/// Every path under a folder, `.git` included, with the size of each file and when it was last
/// changed. A folder's own time is left out: the system may record a change made in it a little
/// later, and a change shows in its files anyway.
fn snapshot(folder: &Path) -> BTreeMap<String, Option<(u64, Option<SystemTime>)>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![folder.to_path_buf()];

    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).expect("the folder can be read") {
            let entry = entry.expect("an entry can be read");
            let facts = entry.metadata().expect("an entry can be looked at");
            let path = entry.path();

            if facts.is_dir() {
                found.insert(path.display().to_string(), None);
                waiting.push(path);
            } else {
                found.insert(
                    path.display().to_string(),
                    Some((facts.len(), facts.modified().ok())),
                );
            }
        }
    }

    found
}

/// A project of one package whose base is the commit on `main`, with the branch `branch` checked out.
fn branched(name: &str, files: &[(&str, &str)]) -> Project {
    let project = Project::new(name);

    for (path, content) in files {
        project.write(path, content);
    }

    project.commit();
    project.git(&["switch", "--quiet", "-c", "branch"]);

    project
}

#[test]
fn a_finding_the_base_has_too_is_unchanged_and_one_the_branch_made_is_new() {
    let project = branched(
        "base-new-and-inherited",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            (
                "src/index.ts",
                "import { a } from './a';\nimport './missing';\na();\n",
            ),
            (
                "src/a.ts",
                "export const a = (): void => {};\nexport const old = 1;\nexport const dropped = 2;\n",
            ),
        ],
    );

    // The branch drops one unused export and adds another in a commit, and adds unresolved imports and
    // an untracked file without committing them (spec FIX-4).
    project
        .write(
            "src/a.ts",
            "export const a = (): void => {};\nexport const old = 1;\nexport const fresh = 3;\n",
        )
        .commit()
        .write(
            "src/index.ts",
            "import { a } from './a';\nimport './missing';\nimport './missing';\nimport './elsewhere';\na();\n",
        )
        .write("src/scratch.ts", "export const scratch = 1;\n");

    let run = audited(&project, &["--base", "main"]);

    // A finding the base has too is unchanged; one the branch made is new; one only the base has is not
    // printed. Of two findings with one fingerprint, the base's one is the first by line (spec INH-1,
    // REPORT-1, FIX-1 pattern 22).
    assert_eq!(
        states(&run),
        [
            ("unused-file", "src/scratch.ts", None, Some("new")),
            ("unused-export", "src/a.ts", Some(2), Some("unchanged")),
            ("unused-export", "src/a.ts", Some(3), Some("new")),
            (
                "unresolved-import",
                "src/index.ts",
                Some(2),
                Some("unchanged")
            ),
            ("unresolved-import", "src/index.ts", Some(3), Some("new")),
            ("unresolved-import", "src/index.ts", Some(4), Some("new")),
        ]
    );
    // The same project gives the same bytes every run (spec INH-4, RUN-8).
    assert_eq!(run, audited(&project, &["--base", "main"]));
    // Without a base, no finding is new or inherited (spec CMD-1).
    assert!(
        states(&audited(&project, &[]))
            .iter()
            .all(|(_, _, _, state)| state.is_none())
    );
}

#[test]
fn the_base_is_read_from_the_commit_and_nothing_is_written() {
    let project = branched(
        "base-from-the-commit",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            ("src/index.ts", "import { used } from './util';\nused();\n"),
            (
                "src/util.ts",
                "export const used = (): void => {};\nexport const spare = 1;\n",
            ),
            ("src/old.ts", "export const old = 1;\n"),
        ],
    );

    // The working tree changes a file and deletes another, and commits neither change: the base still
    // reads both as the commit stores them.
    project.write(
        "src/util.ts",
        "export const used = (): void => {};\nexport const extra = 1;\n",
    );
    fs::remove_file(project.root.join("src/old.ts")).expect("the file can be removed");

    // git's own `status` may write its index, so it is asked before the snapshot is taken.
    let status = project.git(&["status", "--porcelain"]);
    let before = snapshot(&project.root);
    let run = audited(&project, &["--base", "branch"]);

    // The base's `spare` and its unused file are the commit's, so neither is printed, and `extra` is
    // the working tree's alone (spec COMMIT-1, COMMIT-2).
    assert_eq!(
        states(&run),
        [("unused-export", "src/util.ts", Some(2), Some("new"))]
    );
    // Nothing is checked out and nothing is written, `.git` included (spec COMMIT-1, RUN-6).
    assert_eq!(snapshot(&project.root), before);
    assert_eq!(project.git(&["status", "--porcelain"]), status);
}

#[test]
fn names_the_base_commit_and_marks_the_notices_of_the_bases_reading() {
    let project = branched(
        "base-notifications",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            ("src/index.ts", "export {};\n"),
            // A tsconfig no script depends on, which extends a file that is not there: left out, with a
            // notice.
            ("other/tsconfig.json", r#"{ "extends": "./missing.json" }"#),
        ],
    );

    project.git(&["rm", "--quiet", "other/tsconfig.json"]);
    project.commit();

    let full = project.git(&["rev-parse", "main"]).trim().to_owned();
    let run = audited(&project, &["--base", "main"]);

    // One notification names the base by its full name, whatever name it was given by; each notice of
    // the base's reading is marked as the base's (spec REPORT-4, CMD-2).
    assert_eq!(
        marked(&run),
        [
            ("base", None, false),
            ("no-hand-off-file", None, false),
            (
                "tsconfig-extends-left-out",
                Some("other/tsconfig.json"),
                true
            ),
        ]
    );
    assert!(sentence(&run, "base").contains(&full), "{full}");
}

#[test]
fn a_base_that_is_not_a_commit_stops_the_run_after_the_file_list() {
    let project = Project::new("base-not-a-commit");

    project
        .write("package.json", r#"{ "name": "app" }"#)
        .write("src/index.ts", "export {};\n")
        .commit();

    // A name git does not know, and a name of a file rather than a commit, stop the run with the name
    // and the fix (spec CMD-2, CMD-3 stop 7b).
    for name in ["nosuch", "HEAD:package.json"] {
        let line = stopped(&project.start("audit", |binary| {
            binary.args(["--base", name]);
        }));

        assert_eq!(
            line,
            format!(
                "yak-analyzer: the base `{name}` is not a commit in this repository; fetch it with `git \
                 fetch`, or name another, and run again"
            )
        );
    }

    // It is asked before any `package.json` is read: a broken one is reported after it (spec RUN-4).
    project.write("package.json", "{ broken");

    let line = stopped(&project.start("audit", |binary| {
        binary.args(["--base", "nosuch"]);
    }));

    assert!(line.contains("`nosuch` is not a commit"), "{line}");
}

#[test]
fn a_base_whose_files_cannot_be_read_skips_them_and_marks_every_finding_new() {
    let project = branched(
        "base-skipped-files",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            // A tsconfig the scripts depend on, and a script, that do not parse at the base: on the
            // working tree each would stop the run.
            ("tsconfig.json", "{ broken"),
            ("src/index.ts", "import { a } from './a';\na();\n"),
            (
                "src/a.ts",
                "export const a = (): void => {};\nexport const old = 1;\n",
            ),
            ("src/broken.ts", "const = ;\n"),
        ],
    );

    project
        .write("tsconfig.json", "{}")
        .write("src/broken.ts", "export {};\n")
        .commit();

    let run = audited(&project, &["--base", "main"]);

    // At the base each is skipped with a notice marked as the base's, and since a skipped file can make
    // a finding look inherited, every finding is new, with one notification that says why (spec
    // COMMIT-4, FIX-2).
    assert_eq!(
        states(&run),
        [
            ("unused-file", "src/broken.ts", None, Some("new")),
            ("unused-export", "src/a.ts", Some(2), Some("new")),
        ]
    );
    assert_eq!(
        marked(&run),
        [
            ("base", None, false),
            ("base-files-skipped", None, false),
            ("no-hand-off-file", None, false),
            ("file-skipped", Some("src/broken.ts"), true),
            ("file-skipped", Some("tsconfig.json"), true),
        ]
    );
    assert_eq!(
        sentence(&run, "base-files-skipped"),
        "2 file(s) were skipped at the base, so no finding is marked unchanged: `tsconfig.json`, \
         `src/broken.ts`."
    );
}

#[test]
fn the_base_is_read_with_the_hand_off_files_declarations_and_the_working_trees_ignore_rules() {
    let project = branched(
        "base-declarations",
        &[
            ("package.json", r#"{ "name": "app" }"#),
            (".gitignore", "out/\n"),
            (
                "src/tool.ts",
                "import { used } from './helper';\nimport './out/built.js';\nused();\n",
            ),
            (
                "src/helper.ts",
                "export const used = (): void => {};\nexport const extra = 1;\n",
            ),
        ],
    );

    // The branch drops the ignore rule, so the import of the folder it ignored leads nowhere now.
    project.git(&["rm", "--quiet", ".gitignore"]);
    project.commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([{
            "name": "typescript",
            "interfaceVersion": 1,
            "pack": {},
            "project": { "entries": ["src/tool.ts"] }
        }]),
    );
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // The declaration makes `src/tool.ts` an entry point at the base too, so `extra` is unused on both
    // sides rather than in a file nothing reaches; and the base asks the working tree's ignore rules,
    // so the import was unresolved there too (spec COMMIT-3, COMMIT-5).
    assert_eq!(
        states(&run),
        [
            ("unused-export", "src/helper.ts", Some(2), Some("unchanged")),
            (
                "unresolved-import",
                "src/tool.ts",
                Some(2),
                Some("unchanged")
            ),
        ]
    );
}

#[test]
fn a_partial_clone_reads_its_base_with_a_git_that_never_fetches_and_stops_with_an_older_one() {
    let project = branched(
        "base-partial-clone",
        &[
            ("package.json", r#"{ "name": "app" }"#),
            ("src/index.ts", "export {};\n"),
        ],
    );

    // A promisor remote is what makes a repository a partial clone.
    project.git(&["config", "remote.origin.promisor", "true"]);

    let version = Command::new("git")
        .arg("--version")
        .output()
        .expect("git starts");
    let version = String::from_utf8_lossy(&version.stdout);
    let numbers: Vec<u32> = version
        .trim()
        .trim_start_matches("git version ")
        .split(['.', ' '])
        .take(2)
        .map(|number| number.parse().expect("a number"))
        .collect();
    let output = project.start("audit", |binary| {
        binary.args(["--base", "main"]);
    });

    // git 2.44 is the first to honour the switch that turns lazy fetching off (spec COMMIT-2, CMD-3
    // stop 7c).
    if numbers.as_slice() >= [2, 44].as_slice() {
        assert_eq!(printed(&output)["version"], json!("2.1.0"));
    } else {
        let line = stopped(&output);

        assert!(line.contains("partial clone"), "{line}");
        assert!(line.contains("2.44"), "{line}");
    }
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test cli && cargo test --release --locked --test audit
```

Expected: FAIL in `cli`: `refuses_a_base_git_would_read_as_an_option_before_git_sees_it` fails, since
the binary refuses `audit --base` as an option it does not have, with the line for that. The other seven
pass, and the `audit` target does not run.

- [ ] **Step 3: Write the code**

git lists a commit's files, hands out their contents in one batch, and tells a partial clone:

```rust path=src/git.rs
//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2; dead-code spec CMD-2, COMMIT-2, LIFE-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::thread::{self, JoinHandle};

use crate::stop::Stop;

/// The oldest git these commands work with: the release that brought `--no-optional-locks`.
const OLDEST: (u32, u32) = (2, 15);

/// The oldest git that never fetches a missing object when told not to, through the variable below
/// (dead-code spec COMMIT-2).
pub const NO_LAZY_FETCH: (u32, u32) = (2, 44);

/// What every start of git is given before its command (spec RUN-6a): no lock is taken, so nothing is
/// written inside `.git`; no pager and no file-system monitor is started; and letter case counts in an
/// ignore pattern on every platform.
const SETTINGS: [&str; 8] = [
    "--no-optional-locks",
    "--no-pager",
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.untrackedCache=false",
    "-c",
    "core.ignorecase=false",
];

/// The variables git itself calls local to a repository, which `git rev-parse --local-env-vars` prints.
/// Left in place, they would make git read another repository than the root's.
const REPOSITORY_VARIABLES: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// The variable that tells git never to fetch an object a partial clone lacks, set for every start
/// (dead-code spec COMMIT-2). A git older than `NO_LAZY_FETCH` passes it over.
const NO_LAZY_FETCH_VARIABLE: &str = "GIT_NO_LAZY_FETCH";

/// What git printed, and the first line of a warning it gave while succeeding.
pub struct Answer {
    /// What git wrote on standard output.
    pub output: Vec<u8>,
    /// The first line git wrote on standard error, where it wrote one.
    pub warning: Option<String>,
}

/// Runs one git command in the root and returns what it printed.
pub fn read(root: &Path, command: &[&str]) -> Result<Answer, Stop> {
    run(root, command, None)
}

/// Asks git which of the paths given, relative to the root, it ignores (spec LIST-5). It answers from
/// its patterns, whether or not a path is on disk. A path inside a submodule must not be given: git
/// stops on one.
pub fn ignored(root: &Path, paths: &[&str]) -> Result<Vec<String>, Stop> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let input: Vec<u8> = paths
        .iter()
        .flat_map(|path| path.bytes().chain([0]))
        .collect();
    let answer = run(
        root,
        &[
            "check-ignore",
            "-z",
            "--stdin",
            "--verbose",
            "--non-matching",
        ],
        Some(&input),
    )?;
    let fields: Vec<&[u8]> = answer.output.split(|byte| *byte == 0).collect();

    // Each path comes back as four fields: the file of the pattern that decided, its line, the pattern,
    // and the path. A path no pattern matched has the first three empty; a pattern that begins with `!`
    // says the path is not ignored.
    Ok(fields
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|answer| !answer[0].is_empty() && !answer[2].starts_with(b"!"))
        .map(|answer| String::from_utf8_lossy(answer[3]).into_owned())
        .collect())
}

/// What git says of a path above the root, given from the root with `..` (spec LIST-4, CFG-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Above {
    /// A file of the work tree that git does not ignore: tracked, or untracked and not ignored.
    File,
    /// A path git ignores.
    Ignored,
    /// A path of the work tree that is neither: nothing is there.
    Missing,
    /// A path above the top of the work tree.
    OutsideWorkTree,
}

/// The root's folder from the top of its work tree, ending in `/`, or empty at the top.
pub fn prefix(root: &Path) -> Result<String, Stop> {
    let answer = read(root, &["rev-parse", "--show-prefix"])?;

    Ok(String::from_utf8_lossy(&answer.output).trim().to_owned())
}

/// Asks git about one path above the root, as the file list asks about every path under it (spec
/// LIST-4).
pub fn above(root: &Path, path: &str) -> Result<Above, Stop> {
    if leaves_work_tree(&prefix(root)?, path) {
        return Ok(Above::OutsideWorkTree);
    }

    let pathspec = format!(":(literal){path}");
    let listed = read(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            &pathspec,
        ],
    )?;

    if !listed.output.is_empty() {
        Ok(Above::File)
    } else if ignored(root, &[path])?.is_empty() {
        Ok(Above::Missing)
    } else {
        Ok(Above::Ignored)
    }
}

/// Whether a path from the root climbs above the top of the work tree, the root lying at the prefix
/// given.
pub fn leaves_work_tree(prefix: &str, path: &str) -> bool {
    let depth = prefix
        .split('/')
        .filter(|segment| !segment.is_empty())
        .count();
    let up = path
        .split('/')
        .take_while(|segment| *segment == "..")
        .count();

    up > depth
}

/// The full name of the commit a name given on the command line names (dead-code spec CMD-2). The name
/// was checked before: it does not begin with `-` and holds no control character.
pub fn commit(root: &Path, name: &str) -> Result<String, Stop> {
    let asked = format!("{name}^{{commit}}");
    let ran = output(root, &["rev-parse", "--verify", "--quiet", &asked], None)?;
    let full = String::from_utf8_lossy(&ran.stdout).trim().to_owned();

    if ran.status.success() && !full.is_empty() {
        Ok(full)
    } else {
        Err(Stop::new(format!(
            "the base `{name}` is not a commit in this repository; fetch it with `git fetch`, or name another, \
             and run again"
        )))
    }
}

/// Whether the root's repository is a partial clone: its settings name a promisor remote (dead-code
/// spec COMMIT-2).
pub fn partial(root: &Path) -> Result<bool, Stop> {
    let answer = read(root, &["config", "--list", "-z"])?;

    // Each setting is its name, a line break, and its value; git writes the name in lower case.
    Ok(answer
        .output
        .split(|byte| *byte == 0)
        .filter_map(|setting| std::str::from_utf8(setting).ok())
        .any(|setting| {
            let (name, value) = setting.split_once('\n').unwrap_or((setting, ""));

            name == "extensions.partialclone"
                || (name.starts_with("remote.")
                    && name.ends_with(".promisor")
                    && matches!(value, "true" | "yes" | "on" | "1"))
        }))
}

/// The version of the git on the `PATH`, as git writes it, where it is older than the version given.
pub fn older_than(wanted: (u32, u32)) -> Option<String> {
    version()
        .filter(|(numbers, _)| *numbers < wanted)
        .map(|(_, written)| written)
}

/// The files git stores at a commit under the root, each with its mode, its type, its object and its
/// path from the root (dead-code spec COMMIT-2).
pub fn stored(root: &Path, commit: &str) -> Result<Answer, Stop> {
    read(root, &["ls-tree", "-r", "-z", commit])
}

/// A git that hands out the content of one stored object after another, started once for a whole
/// reading, so no file needs a start of git of its own (dead-code spec COMMIT-2). Its input is closed
/// when it is dropped, so it ends with the analyzer (dead-code spec LIFE-2).
pub struct Batch {
    /// The git that answers.
    child: Child,
    /// What it reads the names from; closed first when the batch is dropped.
    input: Option<ChildStdin>,
    /// What it writes each content on.
    output: BufReader<ChildStdout>,
    /// What it writes on standard error, read by a thread of its own so it never waits on a full pipe.
    errors: Option<JoinHandle<Vec<u8>>>,
}

impl Batch {
    /// Starts git's batch in the root.
    pub fn start(root: &Path) -> Result<Self, Stop> {
        let mut git = command(root);

        git.args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = git.spawn().map_err(not_started)?;
        let (Some(input), Some(output), Some(mut errors)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            return Err(Stop::new(
                "git's batch started without its pipes; this is a fault in the analyzer, so report it",
            ));
        };
        let errors = thread::spawn(move || {
            let mut said = Vec::new();
            let _ = errors.read_to_end(&mut said);

            said
        });

        Ok(Self {
            child,
            input: Some(input),
            output: BufReader::new(output),
            errors: Some(errors),
        })
    }

    /// The content of the stored file a name gives, an object's or a commit's and a path's, or none
    /// where it names no file.
    pub fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>, Stop> {
        if name.contains(['\n', '\r']) {
            return Ok(None);
        }

        let asked = self
            .input
            .as_mut()
            .map(|input| writeln!(input, "{name}").and_then(|()| input.flush()));
        let mut header = String::new();

        if !matches!(asked, Some(Ok(()))) || self.output.read_line(&mut header).unwrap_or(0) == 0 {
            return Err(self.failed());
        }

        // A file comes back as its object, its type and its size, then its content and a line break; a
        // name that names nothing, as the name and `missing`.
        let mut facts = header.trim_end().rsplitn(3, ' ');
        let (Some(size), Some(kind)) = (facts.next(), facts.next()) else {
            return Ok(None);
        };
        let Ok(size) = size.parse::<usize>() else {
            return Ok(None);
        };
        let mut content = vec![0; size + 1];

        if self.output.read_exact(&mut content).is_err() {
            return Err(self.failed());
        }

        content.pop();

        Ok((kind == "blob").then_some(content))
    }

    /// The stop for a batch that stopped answering, with git's own first line.
    fn failed(&mut self) -> Stop {
        self.input = None;

        let _ = self.child.wait();
        let said = self
            .errors
            .take()
            .and_then(|errors| errors.join().ok())
            .unwrap_or_default();
        let said = String::from_utf8_lossy(&said);
        let first = said
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("it gave no reason");

        Stop::new(format!("git cannot read the base: {first}"))
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        // Closing its input ends it.
        self.input = None;

        let _ = self.child.wait();

        if let Some(errors) = self.errors.take() {
            let _ = errors.join();
        }
    }
}

/// A git command in the root, with the settings and the variables every start gets (spec RUN-6a;
/// dead-code spec COMMIT-2).
fn command(root: &Path) -> Command {
    let mut git = Command::new("git");

    git.args(SETTINGS)
        .current_dir(root)
        .env(NO_LAZY_FETCH_VARIABLE, "1");

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    git
}

/// The stop for a git that cannot be started.
fn not_started(error: std::io::Error) -> Stop {
    let (oldest_major, oldest_minor) = OLDEST;

    Stop::new(format!(
        "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
         {oldest_minor} or newer, which must be on the PATH"
    ))
}

/// Runs one git command in the root, with what it reads on standard input where there is any, and
/// returns how it ended, whatever its exit code.
fn output(root: &Path, command_line: &[&str], input: Option<&[u8]>) -> Result<Output, Stop> {
    if !root.is_dir() {
        return Err(Stop::new(format!(
            "the root {} is not a folder; run the analyzer in a checkout, or hand it a hand-off file that \
             names one",
            root.display()
        )));
    }

    let mut git = command(root);

    git.args(command_line)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = git.spawn().map_err(not_started)?;

    // The input is written from a thread of its own, so that git never waits on a full output pipe
    // while the analyzer waits to finish writing.
    let writer = match (input, child.stdin.take()) {
        (Some(input), Some(mut stdin)) => {
            let input = input.to_vec();

            Some(thread::spawn(move || stdin.write_all(&input)))
        }
        _ => None,
    };
    let ran = child.wait_with_output().map_err(not_started)?;

    // git stops reading when it fails, which breaks the pipe; its exit code says what happened.
    if let Some(writer) = writer {
        let _ = writer.join();
    }

    Ok(ran)
}

/// Runs one git command in the root, with what it reads on standard input where there is any. With
/// input, exit code 1 is an answer too: `check-ignore` gives it when no path is ignored.
fn run(root: &Path, command_line: &[&str], input: Option<&[u8]>) -> Result<Answer, Stop> {
    let ran = output(root, command_line, input)?;
    let answered = ran.status.success() || (input.is_some() && ran.status.code() == Some(1));
    let said = String::from_utf8_lossy(&ran.stderr);
    let first = said.lines().find(|line| !line.trim().is_empty());

    if answered {
        return Ok(Answer {
            output: ran.stdout,
            warning: first.map(str::to_owned),
        });
    }

    // git failed. An old git fails on the settings themselves, so the version is asked only now.
    let (oldest_major, oldest_minor) = OLDEST;

    if let Some(version) = older_than(OLDEST) {
        return Err(Stop::new(format!(
            "git {version} is older than {oldest_major}.{oldest_minor}, the oldest the analyzer works with; \
             update git"
        )));
    }

    Err(Stop::new(format!(
        "git cannot read the root {}: {}; run the analyzer inside a git work tree",
        root.display(),
        first.unwrap_or("it gave no reason")
    )))
}

/// The version of the git on the `PATH`: its first two numbers, and the version as git writes it.
fn version() -> Option<((u32, u32), String)> {
    let answer = Command::new("git")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&answer.stdout);
    // git prints `git version 2.56.0`, and some builds add more after the numbers.
    let written = text
        .trim()
        .strip_prefix("git version ")?
        .split(' ')
        .next()?;
    let mut numbers = written.split('.').map(str::parse::<u32>);
    let major = numbers.next()?.ok()?;
    let minor = numbers.next()?.ok()?;

    Some(((major, minor), written.to_owned()))
}
```

```rust edit path=src/files.rs
<<<<<<< find
        links: link_paths,
    })
}
=======
        links: link_paths,
    })
}

/// The file list of a commit: every file git stores at it under the root, each tracked, with the object
/// that holds each file's content, by its path (dead-code spec COMMIT-2). A stored symbolic link and a
/// submodule are left out, as the working tree's are (spec LIST-3).
pub fn stored(
    root: &Path,
    commit: &str,
    notices: &mut Vec<Notice>,
) -> Result<(Listing, BTreeMap<String, String>), Stop> {
    let answer = git::stored(root, commit)?;
    let mut listing = Listing::default();
    let mut objects = BTreeMap::new();
    let mut unreadable_names = 0_usize;

    // Each entry is its mode, its type and its object, then a tab and its path, ended by a zero byte.
    for record in answer
        .output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Ok(record) = std::str::from_utf8(record) else {
            unreadable_names += 1;

            continue;
        };
        let entry = record.split_once('\t').and_then(|(facts, name)| {
            let mut facts = facts.split(' ');

            Some((facts.next()?, facts.next()?, facts.next()?, name))
        });
        let Some((mode, kind, object, name)) = entry else {
            return Err(Stop::new(format!(
                "git listed an entry of the base the analyzer cannot read, `{record}`; this is a fault in the \
                 analyzer, so report it with the git version"
            )));
        };

        if mode == SUBMODULE_MODE || kind == "commit" {
            listing.elsewhere.insert(name.to_owned());
        } else if mode == LINK_MODE {
            listing.links.insert(name.to_owned());
        } else if kind == "blob" {
            objects.insert(name.to_owned(), object.to_owned());
            listing.files.push(File {
                kind: Kind::of(name),
                path: name.to_owned(),
                tracked: true,
            });
        }
    }

    if !listing.links.is_empty() {
        let links = listing.links.len();

        notices.push(Notice::new(
            "links-left-out",
            None,
            format!("{links} symbolic link(s) were left out of the file list; a link is never followed or read."),
        ));
    }

    if unreadable_names > 0 {
        notices.push(Notice::new(
            "names-left-out",
            None,
            format!("{unreadable_names} file(s) whose names are not valid UTF-8 were left out of the file list."),
        ));
    }

    listing
        .files
        .sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));

    Ok((listing, objects))
}
>>>>>>> replace
```

A tree is where a reading reads from:

```rust path=src/tree.rs
//! Where a reading reads its files from: the working tree on disk, or the files git stores at a commit
//! (spec LIST-9; dead-code spec COMMIT-1 to COMMIT-3).
//!
//! Every reader reads a listed file through a tree, so one reading works on either. Nothing is checked
//! out and nothing is written: a commit's files come from one batch of git's, started once.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::files::{self, Listing};
use crate::git::{self, Above, Batch};
use crate::notice::Notice;
use crate::stop::Stop;

/// A commit's stored files, and the git that reads them.
struct Stored {
    /// The commit's full name.
    commit: String,
    /// The root's folder from the top of the work tree, ending in `/`, or empty at the top.
    prefix: String,
    /// The object that holds each listed file's content, by the file's path.
    objects: BTreeMap<String, String>,
    /// The git that hands out the contents.
    batch: Mutex<Batch>,
    /// The first time git failed while the commit was read, which stops the run (dead-code spec
    /// COMMIT-4).
    failure: Mutex<Option<Stop>>,
}

/// Where a reading reads from.
pub struct Tree {
    /// The root.
    root: PathBuf,
    /// The commit's files, for the base; none for the working tree.
    stored: Option<Stored>,
}

impl Tree {
    /// The working tree under the root, as it is on disk.
    pub const fn working(root: PathBuf) -> Self {
        Self { root, stored: None }
    }

    /// The files git stores at a commit under the root, with their list (dead-code spec COMMIT-2).
    pub fn base(
        root: PathBuf,
        commit: &str,
        notices: &mut Vec<Notice>,
    ) -> Result<(Self, Listing), Stop> {
        let (listing, objects) = files::stored(&root, commit, notices)?;
        let stored = Stored {
            commit: commit.to_owned(),
            prefix: git::prefix(&root)?,
            objects,
            batch: Mutex::new(Batch::start(&root)?),
            failure: Mutex::new(None),
        };

        Ok((
            Self {
                root,
                stored: Some(stored),
            },
            listing,
        ))
    }

    /// The root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether this is the base, where a file that cannot be read or does not have its form is skipped
    /// rather than stopping the run (dead-code spec COMMIT-4).
    pub const fn is_base(&self) -> bool {
        self.stored.is_some()
    }

    /// The content of a file, by its path from the root; a path above the root begins with `..`.
    pub fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let Some(stored) = &self.stored else {
            return fs::read(self.root.join(path));
        };
        let listed = stored.objects.get(path);
        let name = match listed {
            Some(object) => object.clone(),
            None => format!("{}:{}", stored.commit, from_top(&stored.prefix, path)),
        };
        let content = stored
            .batch
            .lock()
            .map_err(|_| io::Error::other("git's batch was left broken"))
            .and_then(|mut batch| batch.read(&name).map_err(|stop| self.failed(stop)))?;

        match content {
            Some(content) => Ok(content),
            // A listed file whose content git does not have is in a partial clone that lacks it, and
            // the analyzer never fetches it (dead-code spec COMMIT-2).
            None if listed.is_some() => Err(self.failed(Stop::new(format!(
                "git does not have the content of {path} at the base, and the analyzer does not fetch it; fetch \
                 the base commit's files, and run again"
            )))),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "not stored at the base",
            )),
        }
    }

    /// What git says of a path above the root (spec LIST-4, CFG-3). At the base, a file is one the commit
    /// stores; whether git ignores a path is asked of the working tree's ignore rules (dead-code spec
    /// COMMIT-3).
    pub fn above(&self, path: &str) -> Result<Above, Stop> {
        let Some(stored) = &self.stored else {
            return git::above(&self.root, path);
        };

        if git::leaves_work_tree(&stored.prefix, path) {
            return Ok(Above::OutsideWorkTree);
        }

        let name = format!("{}:{}", stored.commit, from_top(&stored.prefix, path));
        let found = stored
            .batch
            .lock()
            .map_err(|_| Stop::new("git's batch was left broken; this is a fault in the analyzer"))?
            .read(&name)?;

        Ok(if found.is_some() {
            Above::File
        } else if git::ignored(&self.root, &[path])?.is_empty() {
            Above::Missing
        } else {
            Above::Ignored
        })
    }

    /// The stop of the first time git failed while the base was read, where it failed.
    pub fn failure(&self) -> Result<(), Stop> {
        match self
            .stored
            .as_ref()
            .and_then(|stored| stored.failure.lock().ok()?.take())
        {
            Some(stop) => Err(stop),
            None => Ok(()),
        }
    }

    /// Keeps the first failure of git's, and gives the reader an error that makes it skip the file;
    /// the run stops once the reading ends.
    fn failed(&self, stop: Stop) -> io::Error {
        if let Some(stored) = &self.stored
            && let Ok(mut failure) = stored.failure.lock()
        {
            failure.get_or_insert(stop);
        }

        io::Error::other("git failed while the base was read")
    }
}

/// A path from the root as a path from the top of the work tree, with `..` worked out.
fn from_top(prefix: &str, path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();

    for segment in prefix.split('/').chain(path.split('/')) {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    segments.join("/")
}
```

The readers read through it, and at the base skip what would stop the run:

```rust edit path=src/packages.rs
<<<<<<< find
//! package each file belongs to (spec PACK-1 to PACK-6).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};
use yaml_rust2::{Yaml, YamlLoader};
=======
//! package each file belongs to (spec PACK-1 to PACK-6).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};
use yaml_rust2::{Yaml, YamlLoader};
>>>>>>> replace
<<<<<<< find
use crate::glob::Glob;
use crate::notice::Notice;
use crate::stop::{Fault, Stop};

/// The workspace's list that pnpm reads, at the root.
const PNPM_LIST: &str = "pnpm-workspace.yaml";
=======
use crate::glob::Glob;
use crate::notice::Notice;
use crate::stop::{Fault, Stop};
use crate::tree::Tree;

/// The workspace's list that pnpm reads, at the root.
const PNPM_LIST: &str = "pnpm-workspace.yaml";
>>>>>>> replace
<<<<<<< find
    read: Vec<(String, Map<String, Value>)>,
    /// The workspace's list that pnpm reads, where the list holds one.
    pnpm: Option<Yaml>,
}

/// Reads every `package.json` that makes a package, then the workspace's list. `data` is the files
/// declared as data, whose `package.json` makes no package (spec KEY-4).
pub fn load(root: &Path, files: &[File], data: &BTreeSet<String>) -> Result<Loaded, Stop> {
    let manifests: Vec<&File> = files
        .iter()
        .filter(|file| file.kind == Kind::Manifest && !data.contains(&file.path))
=======
    read: Vec<(String, Map<String, Value>)>,
    /// The workspace's list that pnpm reads, where the list holds one.
    pnpm: Option<Yaml>,
    /// Whether a file whose field does not have its form is skipped rather than stopping the run, as at
    /// the base (dead-code spec COMMIT-4).
    skipping: bool,
}

/// Reads every `package.json` that makes a package, then the workspace's list. `data` is the files
/// declared as data, whose `package.json` makes no package (spec KEY-4). At the base, a file that
/// cannot be read is skipped with a notice: a `package.json` makes no package, and the workspace's list
/// adds no folder (dead-code spec COMMIT-4).
pub fn load(
    tree: &Tree,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
) -> Result<Loaded, Stop> {
    let manifests: Vec<&File> = files
        .iter()
        .filter(|file| file.kind == Kind::Manifest && !data.contains(&file.path))
>>>>>>> replace
<<<<<<< find
    let mut read: Vec<(String, Map<String, Value>)> = Vec::with_capacity(manifests.len());

    for manifest in &manifests {
        read.push((folder_of(&manifest.path), object(root, &manifest.path)?));
    }

    let pnpm = if files.iter().any(|file| file.path == PNPM_LIST) && !data.contains(PNPM_LIST) {
        Some(yaml(root)?)
    } else {
        None
    };

    Ok(Loaded { read, pnpm })
}

impl Loaded {
=======
    let mut read: Vec<(String, Map<String, Value>)> = Vec::with_capacity(manifests.len());

    for manifest in &manifests {
        if let Some(fields) = kept(tree, &manifest.path, object(tree, &manifest.path), notices)? {
            read.push((folder_of(&manifest.path), fields));
        }
    }

    let pnpm = if files.iter().any(|file| file.path == PNPM_LIST) && !data.contains(PNPM_LIST) {
        kept(tree, PNPM_LIST, yaml(tree), notices)?
    } else {
        None
    };

    Ok(Loaded {
        read,
        pnpm,
        skipping: tree.is_base(),
    })
}

/// What reading a file gave: its content, or at the base, where a file that cannot be read is skipped
/// with a notice, none (dead-code spec COMMIT-4).
fn kept<T>(
    tree: &Tree,
    file: &str,
    read: Result<T, Stop>,
    notices: &mut Vec<Notice>,
) -> Result<Option<T>, Stop> {
    match read {
        Ok(read) => Ok(Some(read)),
        Err(stop) if tree.is_base() => {
            notices.push(Notice::skipped(file, &stop));

            Ok(None)
        }
        Err(stop) => Err(stop),
    }
}

impl Loaded {
>>>>>>> replace
<<<<<<< find
    ) -> Result<Packages, Fault> {
        let mut faults = Vec::new();
        let mut names: BTreeMap<&str, Fields> = BTreeMap::new();

        for (folder, fields) in &self.read {
            let file = manifest_path(folder);
=======
    ) -> Result<Packages, Fault> {
        let mut faults = Vec::new();
        let mut names: BTreeMap<&str, Fields> = BTreeMap::new();
        // At the base, a fault skips its file with a notice rather than stopping the run (dead-code spec
        // COMMIT-4).
        let mut fault = |file: &str, stop: Stop| {
            if self.skipping {
                notices.push(Notice::skipped(file, &stop));
            } else {
                faults.push(Fault::new(file, stop));
            }
        };

        for (folder, fields) in &self.read {
            let file = manifest_path(folder);
>>>>>>> replace
<<<<<<< find
                Ok(read) => {
                    names.insert(folder, read);
                }
                Err(stop) => faults.push(Fault::new(file, stop)),
            }
        }

=======
                Ok(read) => {
                    names.insert(folder, read);
                }
                Err(stop) => fault(&file, stop),
            }
        }

>>>>>>> replace
<<<<<<< find
        if let Some((_, fields)) = self.read.iter().find(|(folder, _)| folder == ".") {
            match workspaces(fields) {
                Ok(patterns) => sources.push(("package.json", patterns)),
                Err(stop) => faults.push(Fault::new("package.json", stop)),
            }
        }

        if let Some(pnpm) = &self.pnpm {
            match pnpm_packages(pnpm) {
                Ok(patterns) => sources.push((PNPM_LIST, patterns)),
                Err(stop) => faults.push(Fault::new(PNPM_LIST, stop)),
            }
        }

=======
        if let Some((_, fields)) = self.read.iter().find(|(folder, _)| folder == ".") {
            match workspaces(fields) {
                Ok(patterns) => sources.push(("package.json", patterns)),
                Err(stop) => fault("package.json", stop),
            }
        }

        if let Some(pnpm) = &self.pnpm {
            match pnpm_packages(pnpm) {
                Ok(patterns) => sources.push((PNPM_LIST, patterns)),
                Err(stop) => fault(PNPM_LIST, stop),
            }
        }

>>>>>>> replace
<<<<<<< find
                    included.extend(include);
                    excluded.extend(exclude);
                }
                Err(stop) => faults.push(Fault::new(*file, stop)),
            }
        }

=======
                    included.extend(include);
                    excluded.extend(exclude);
                }
                Err(stop) => fault(file, stop),
            }
        }

>>>>>>> replace
<<<<<<< find
}

/// The content of a file, with a byte-order mark at its start passed over.
fn text(root: &Path, path: &str) -> Result<String, String> {
    let bytes = fs::read(root.join(path)).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);

    String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())
}

/// A `package.json` read as a JSON object (spec PACK-2).
fn object(root: &Path, path: &str) -> Result<Map<String, Value>, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the package.json {path} cannot be read as a JSON object ({why}); fix it, or declare it as data \
             if it is broken on purpose"
        ))
    };
    let content = text(root, path).map_err(refused)?;

    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(fields)) => Ok(fields),
=======
}

/// The content of a file, with a byte-order mark at its start passed over.
fn text(tree: &Tree, path: &str) -> Result<String, String> {
    let bytes = tree.read(path).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);

    String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())
}

/// A `package.json` read as a JSON object (spec PACK-2).
fn object(tree: &Tree, path: &str) -> Result<Map<String, Value>, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the package.json {path} cannot be read as a JSON object ({why}); fix it, or declare it as data \
             if it is broken on purpose"
        ))
    };
    let content = text(tree, path).map_err(refused)?;

    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(fields)) => Ok(fields),
>>>>>>> replace
<<<<<<< find
}

/// The workspace's list that pnpm reads, as YAML (spec PACK-3).
fn yaml(root: &Path) -> Result<Yaml, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the workspace's list {PNPM_LIST} cannot be read ({why}); fix it, or declare it as data"
        ))
    };
    let content = text(root, PNPM_LIST).map_err(refused)?;
    let documents =
        YamlLoader::load_from_str(&content).map_err(|error| refused(error.to_string()))?;

=======
}

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

>>>>>>> replace
```

```rust edit path=src/tsconfig.rs
<<<<<<< find
//! no `node_modules`, and meets no field the analyzer has not checked.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
=======
//! no `node_modules`, and meets no field the analyzer has not checked.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
>>>>>>> replace
<<<<<<< find
use crate::packages::{Packages, misshapen};
use crate::scripts;
use crate::stop::{Fault, Stop};

/// The names of the configs that govern the files of their folder (spec CFG-2).
const GOVERNING: [&str; 2] = ["tsconfig.json", "jsconfig.json"];
=======
use crate::packages::{Packages, misshapen};
use crate::scripts;
use crate::stop::{Fault, Stop};
use crate::tree::Tree;

/// The names of the configs that govern the files of their folder (spec CFG-2).
const GOVERNING: [&str; 2] = ["tsconfig.json", "jsconfig.json"];
>>>>>>> replace
<<<<<<< find
    configs: BTreeMap<String, Config>,
    /// The configs a file the analyzer reads depends on (spec CFG-4).
    named: BTreeSet<String>,
}

/// One build a config states, for MAP-1: the folder its output goes to, and the folders its sources are
=======
    configs: BTreeMap<String, Config>,
    /// The configs a file the analyzer reads depends on (spec CFG-4).
    named: BTreeSet<String>,
    /// Whether a config whose field does not have its form is skipped rather than stopping the run, as
    /// at the base (dead-code spec COMMIT-4).
    skipping: bool,
}

/// One build a config states, for MAP-1: the folder its output goes to, and the folders its sources are
>>>>>>> replace
<<<<<<< find
/// config that a file the analyzer reads depends on stops the run when it cannot be read, or when it
/// extends a file that should be read and is not there; any other is left out with a notice (spec
/// CFG-1, CFG-3, CFG-4). `packages` is there when the `package.json` files' fields hold their forms,
/// for an `extends` that names a workspace package.
pub fn load(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    packages: Option<&Packages>,
    notices: &mut Vec<Notice>,
) -> Result<Loaded, Stop> {
    let listed: BTreeSet<&str> = files
        .iter()
        .filter(|file| file.kind == Kind::Tsconfig && !data.contains(&file.path))
=======
/// config that a file the analyzer reads depends on stops the run when it cannot be read, or when it
/// extends a file that should be read and is not there; any other is left out with a notice (spec
/// CFG-1, CFG-3, CFG-4). `packages` is there when the `package.json` files' fields hold their forms,
/// for an `extends` that names a workspace package. At the base, a config that would stop the run is
/// skipped with a notice instead, and governs nothing (dead-code spec COMMIT-4).
pub fn load(
    tree: &Tree,
    files: &[File],
    data: &BTreeSet<String>,
    packages: Option<&Packages>,
    notices: &mut Vec<Notice>,
) -> Result<Loaded, Stop> {
    let root = tree.root();
    let listed: BTreeSet<&str> = files
        .iter()
        .filter(|file| file.kind == Kind::Tsconfig && !data.contains(&file.path))
>>>>>>> replace
<<<<<<< find
        .collect();
    let mut parsed: BTreeMap<String, Result<Map<String, Value>, String>> = listed
        .iter()
        .map(|path| ((*path).to_owned(), parse(root, path)))
        .collect();
    let finder = Finder {
        root,
        listed: &listed,
        data,
        packages,
=======
        .collect();
    let mut parsed: BTreeMap<String, Result<Map<String, Value>, String>> = listed
        .iter()
        .map(|path| ((*path).to_owned(), parse(tree, path)))
        .collect();
    let finder = Finder {
        tree,
        listed: &listed,
        data,
        packages,
>>>>>>> replace
<<<<<<< find
            if let Target::Read(above) = &target
                && !parsed.contains_key(above)
            {
                parsed.insert(above.clone(), parse(root, above));
                waiting.push(above.clone());
            }

=======
            if let Target::Read(above) = &target
                && !parsed.contains_key(above)
            {
                parsed.insert(above.clone(), parse(tree, above));
                waiting.push(above.clone());
            }

>>>>>>> replace
<<<<<<< find
        }
    }

    if let Some(fault) = Fault::first(faults) {
        return Err(fault.stop());
    }
=======
        }
    }

    // At the base, each such config is skipped with a notice, and governs nothing (dead-code spec
    // COMMIT-4).
    let mut skipped = BTreeSet::new();

    if tree.is_base() {
        for fault in faults.drain(..) {
            let (file, stop) = fault.parts();

            // A config with two faults is skipped once, for the first.
            if !skipped.contains(&file) {
                notices.push(Notice::skipped(&file, &stop));
                skipped.insert(file);
            }
        }
    }

    if let Some(fault) = Fault::first(faults) {
        return Err(fault.stop());
    }
>>>>>>> replace
<<<<<<< find
    let mut configs = BTreeMap::new();

    for (path, read) in parsed {
        let fields = match read {
            Ok(fields) => fields,
            Err(why) => {
=======
    let mut configs = BTreeMap::new();

    for (path, read) in parsed {
        if skipped.contains(&path) {
            continue;
        }

        let fields = match read {
            Ok(fields) => fields,
            Err(why) => {
>>>>>>> replace
<<<<<<< find
            path,
            Config {
                fields,
                extends: read_targets(extends),
                references: read_targets(references),
            },
        );
    }
=======
            path,
            Config {
                fields,
                extends: read_targets(extends, &skipped),
                references: read_targets(references, &skipped),
            },
        );
    }
>>>>>>> replace
<<<<<<< find
        root: clean(root),
        configs,
        named,
    })
}

impl Loaded {
    /// Checks the fields the analyzer reads in every config it read, and gives the library the configs a
    /// file depends on (spec CFG-1, RES-9, PACK-6).
    pub fn checked(self) -> Result<Configs, Fault> {
        let faults = self.configs.iter().filter_map(|(path, config)| {
            forms(&config.fields, path)
                .err()
                .map(|stop| Fault::new(path.clone(), stop))
        });

        if let Some(fault) = Fault::first(faults) {
            return Err(fault);
        }

        let mut served = BTreeMap::new();
=======
        root: clean(root),
        configs,
        named,
        skipping: tree.is_base(),
    })
}

impl Loaded {
    /// Checks the fields the analyzer reads in every config it read, and gives the library the configs a
    /// file depends on (spec CFG-1, RES-9, PACK-6). At the base, a config whose field does not have its
    /// form is skipped with a notice, and governs nothing (dead-code spec COMMIT-4).
    pub fn checked(mut self, notices: &mut Vec<Notice>) -> Result<Configs, Fault> {
        let faults: Vec<Fault> = self
            .configs
            .iter()
            .filter_map(|(path, config)| {
                forms(&config.fields, path)
                    .err()
                    .map(|stop| Fault::new(path.clone(), stop))
            })
            .collect();

        if !self.skipping {
            if let Some(fault) = Fault::first(faults) {
                return Err(fault);
            }
        } else if !faults.is_empty() {
            let mut skipped = BTreeSet::new();

            for fault in faults {
                let (file, stop) = fault.parts();

                notices.push(Notice::skipped(&file, &stop));
                skipped.insert(file);
            }

            self.configs.retain(|path, _| !skipped.contains(path));

            for config in self.configs.values_mut() {
                config.extends.retain(|target| !skipped.contains(target));
                config.references.retain(|target| !skipped.contains(target));
            }
        }

        let mut served = BTreeMap::new();
>>>>>>> replace
<<<<<<< find

/// Finds the targets of `extends` and `references`.
struct Finder<'a> {
    root: &'a Path,
    listed: &'a BTreeSet<&'a str>,
    data: &'a BTreeSet<String>,
    packages: Option<&'a Packages>,
=======

/// Finds the targets of `extends` and `references`.
struct Finder<'a> {
    tree: &'a Tree,
    listed: &'a BTreeSet<&'a str>,
    data: &'a BTreeSet<String>,
    packages: Option<&'a Packages>,
>>>>>>> replace
<<<<<<< find

        if is_above(&tried[0]) {
            for candidate in &tried {
                match git::above(self.root, candidate)? {
                    Above::File => return Ok(Target::Read(candidate.clone())),
                    Above::Ignored => return Ok(Target::Without("a file git ignores")),
                    Above::OutsideWorkTree => {
=======

        if is_above(&tried[0]) {
            for candidate in &tried {
                match self.tree.above(candidate)? {
                    Above::File => return Ok(Target::Read(candidate.clone())),
                    Above::Ignored => return Ok(Target::Without("a file git ignores")),
                    Above::OutsideWorkTree => {
>>>>>>> replace
<<<<<<< find

        let asked: Vec<&str> = tried.iter().map(String::as_str).collect();

        Ok(if git::ignored(self.root, &asked)?.is_empty() {
            Target::Missing(tried[0].clone())
        } else {
            Target::Without("a file git ignores")
=======

        let asked: Vec<&str> = tried.iter().map(String::as_str).collect();

        Ok(if git::ignored(self.tree.root(), &asked)?.is_empty() {
            Target::Missing(tried[0].clone())
        } else {
            Target::Without("a file git ignores")
>>>>>>> replace
<<<<<<< find
    named
}

/// The targets that are read, by path.
fn read_targets(targets: Vec<(String, Target)>) -> Vec<String> {
    targets
        .into_iter()
        .filter_map(|(_, target)| match target {
            Target::Read(path) => Some(path),
            _ => None,
        })
        .collect()
=======
    named
}

/// The targets that are read, by path, but those skipped at the base.
fn read_targets(targets: Vec<(String, Target)>, skipped: &BTreeSet<String>) -> Vec<String> {
    targets
        .into_iter()
        .filter_map(|(_, target)| match target {
            Target::Read(path) if !skipped.contains(&path) => Some(path),
            _ => None,
        })
        .collect()
>>>>>>> replace
<<<<<<< find

/// A config's content as JSON that may hold comments and trailing commas, with a byte-order mark at its
/// start passed over (spec CFG-1, PARSE-1); or why it cannot be read.
fn parse(root: &Path, path: &str) -> Result<Map<String, Value>, String> {
    let bytes = fs::read(root.join(path)).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let mut text =
        String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())?;
=======

/// A config's content as JSON that may hold comments and trailing commas, with a byte-order mark at its
/// start passed over (spec CFG-1, PARSE-1); or why it cannot be read.
fn parse(tree: &Tree, path: &str) -> Result<Map<String, Value>, String> {
    let bytes = tree.read(path).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let mut text =
        String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())?;
>>>>>>> replace
```

```rust edit path=src/scripts.rs
<<<<<<< find
//! Reading every file of the list that holds imports: its content from disk, parsed, and its imports
//! found. Scripts and type declaration files are parsed whole; Vue components, MDX documents and HTML
//! pages by their own readers (spec PARSE-1 to PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::files::{File, Kind};
use crate::imports::{self, Broken, Read};
use crate::notice::Notice;
use crate::pages::{self, Skipped};
use crate::stop::Stop;

/// What reading the scripts found.
#[derive(Debug, Default)]
=======
//! Reading every file of the list that holds imports: its content from its tree, parsed, and its
//! imports found. Scripts and type declaration files are parsed whole; Vue components, MDX documents
//! and HTML pages by their own readers (spec PARSE-1 to PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;

use crate::files::{File, Kind};
use crate::imports::{self, Broken, Read};
use crate::notice::Notice;
use crate::pages::{self, Skipped};
use crate::stop::Stop;
use crate::tree::Tree;

/// What reading the scripts found.
#[derive(Debug, Default)]
>>>>>>> replace
<<<<<<< find

/// Reads every script, type declaration file, Vue component, MDX document and HTML page in the list
/// that is not data, in the order of the list. A tracked file that cannot be read stops the run; an
/// untracked one is skipped with a notice (spec PARSE-2).
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
=======

/// Reads every script, type declaration file, Vue component, MDX document and HTML page in the list
/// that is not data, in the order of the list. A tracked file that cannot be read stops the run; an
/// untracked one is skipped with a notice (spec PARSE-2), and so is any at the base, where it has no
/// imports and no exports (dead-code spec COMMIT-4).
pub fn read(
    tree: &Tree,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
>>>>>>> replace
<<<<<<< find
            continue;
        }

        match one(root, file) {
            Ok((read, skipped)) => {
                notices.extend(
                    skipped
=======
            continue;
        }

        match one(tree, file) {
            Ok((read, skipped)) => {
                notices.extend(
                    skipped
>>>>>>> replace
<<<<<<< find
                scripts.gone.insert(file.path.clone());
            }
            Err(Unread::Broken(why)) if file.tracked => {
                return Err(Stop::new(format!(
                    "the file {} {why}; fix it, or declare it as data if it is broken on purpose",
                    file.path
                )));
            }
            Err(Unread::Broken(why)) => notices.push(Notice::new(
                "untracked-file-skipped",
=======
                scripts.gone.insert(file.path.clone());
            }
            Err(Unread::Broken(why)) if file.tracked => {
                let stop = Stop::new(format!(
                    "the file {} {why}; fix it, or declare it as data if it is broken on purpose",
                    file.path
                ));

                if !tree.is_base() {
                    return Err(stop);
                }

                notices.push(Notice::skipped(&file.path, &stop));
            }
            Err(Unread::Broken(why)) => notices.push(Notice::new(
                "untracked-file-skipped",
>>>>>>> replace
<<<<<<< find
}

/// Reads one file and finds its imports, with what its reader passed over.
fn one(root: &Path, file: &File) -> Result<(Read, Vec<Skipped>), Unread> {
    let path = file.path.as_str();
    let bytes = fs::read(root.join(path)).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Unread::Gone,
        _ => Unread::Broken(format!("cannot be read ({error})")),
    })?;
=======
}

/// Reads one file and finds its imports, with what its reader passed over.
fn one(tree: &Tree, file: &File) -> Result<(Read, Vec<Skipped>), Unread> {
    let path = file.path.as_str();
    let bytes = tree.read(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Unread::Gone,
        _ => Unread::Broken(format!("cannot be read ({error})")),
    })?;
>>>>>>> replace
```

```rust edit path=src/entries.rs
<<<<<<< find
//! `internal` declarations, and the configuration files `configEntries` names (spec ENTRY-6, ENTRY-7).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

=======
//! `internal` declarations, and the configuration files `configEntries` names (spec ENTRY-6, ENTRY-7).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

>>>>>>> replace
<<<<<<< find
use crate::resolve::{self, Outcome, Resolution, Resolver};
use crate::stop::Stop;
use crate::tools;
use crate::tsconfig::Configs;

/// The endings of build output an `entries` glob may end in, a type declaration's first, since `.d.ts`
=======
use crate::resolve::{self, Outcome, Resolution, Resolver};
use crate::stop::Stop;
use crate::tools;
use crate::tree::Tree;
use crate::tsconfig::Configs;

/// The endings of build output an `entries` glob may end in, a type declaration's first, since `.d.ts`
>>>>>>> replace
<<<<<<< find

/// What the entry points are read from.
pub struct Project<'p> {
    /// The root, from which a configuration file's content is read.
    pub root: &'p Path,
    /// The file list.
    pub files: &'p [File],
    /// The packages.
=======

/// What the entry points are read from.
pub struct Project<'p> {
    /// The tree a configuration file's content is read from.
    pub tree: &'p Tree,
    /// The file list.
    pub files: &'p [File],
    /// The packages.
>>>>>>> replace
<<<<<<< find
                };

                for glob in declaration
                    .globs(project.root, &config.path, notices)
                    .iter()
                    .filter_map(|written| Glob::new(written).ok())
                {
=======
                };

                for glob in declaration
                    .globs(project.tree, &config.path, notices)
                    .iter()
                    .filter_map(|written| Glob::new(written).ok())
                {
>>>>>>> replace
<<<<<<< find
    /// The globs a configuration file's value gives, from the folder the value is relative to: the
    /// value where it is a literal and leaves a glob the analyzer reads, and the declaration's
    /// `default` where it does not.
    fn globs(&self, root: &Path, path: &str, notices: &mut Vec<Notice>) -> Vec<String> {
        let stated = fs::read(root.join(path))
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| {
=======
    /// The globs a configuration file's value gives, from the folder the value is relative to: the
    /// value where it is a literal and leaves a glob the analyzer reads, and the declaration's
    /// `default` where it does not.
    fn globs(&self, tree: &Tree, path: &str, notices: &mut Vec<Notice>) -> Vec<String> {
        let stated = tree
            .read(path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| {
>>>>>>> replace
```

```rust edit path=src/stop.rs
<<<<<<< find
    pub fn line(&self) -> String {
        line(&self.0)
    }
}

/// One line for standard error: the program's name, then the text with every line break and control
=======
    pub fn line(&self) -> String {
        line(&self.0)
    }

    /// What failed, without the fix after the last `;`: at the base, where a file is skipped rather than
    /// fixed (dead-code spec COMMIT-4).
    pub fn reason(&self) -> &str {
        self.0
            .rsplit_once("; ")
            .map_or(&self.0, |(reason, _)| reason)
    }
}

/// One line for standard error: the program's name, then the text with every line break and control
>>>>>>> replace
<<<<<<< find
    pub fn stop(self) -> Stop {
        self.stop
    }
}
=======
    pub fn stop(self) -> Stop {
        self.stop
    }

    /// The file and the stop.
    pub fn parts(self) -> (String, Stop) {
        (self.file, self.stop)
    }
}
>>>>>>> replace
```

```rust path=src/notice.rs
//! Notices: what a person should know about a run that went on (spec RUN-5).

use serde_json::{Value, json};

use crate::stop::Stop;

/// The kind of the notice for a file the base's reading skipped (dead-code spec COMMIT-4).
pub const SKIPPED: &str = "file-skipped";

/// Something a person should know about a run that went on.
#[derive(Debug)]
pub struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
    /// Whether the base's reading gave it (dead-code spec REPORT-4).
    base: bool,
}

impl Notice {
    /// A notice of the kind given, about a file or about none.
    pub const fn new(kind: &'static str, file: Option<String>, sentence: String) -> Self {
        Self {
            kind,
            file,
            sentence,
            base: false,
        }
    }

    /// The notice for a file the base's reading skips, with what would have stopped the run on the
    /// working tree (dead-code spec COMMIT-4).
    pub fn skipped(file: &str, stop: &Stop) -> Self {
        Self::new(
            SKIPPED,
            Some(file.to_owned()),
            format!(
                "The file is skipped at the base, where nobody can fix it: {}.",
                stop.reason()
            ),
        )
    }

    /// The notice, marked as the base's.
    pub const fn of_base(mut self) -> Self {
        self.base = true;
        self
    }

    /// The kind of notice.
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// The file it is about, where there is one.
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// What happened, as a sentence.
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// Whether the base's reading gave it.
    pub const fn is_base(&self) -> bool {
        self.base
    }

    /// The notice as `graph` prints it.
    fn printed(&self) -> Value {
        json!({ "file": self.file, "kind": self.kind, "sentence": self.sentence })
    }
}

/// The notices in the order they are printed in: the working tree's before the base's, and of each,
/// those about no file first, then by file, by kind and by sentence (spec RUN-8).
pub fn sorted(mut notices: Vec<Notice>) -> Vec<Notice> {
    notices.sort_by(|a, b| {
        (a.base, &a.file, a.kind, &a.sentence).cmp(&(b.base, &b.file, b.kind, &b.sentence))
    });

    notices
}

/// The notices as `graph` prints them, in their order.
pub fn printed(notices: Vec<Notice>) -> Vec<Value> {
    sorted(notices).iter().map(Notice::printed).collect()
}
```

One reading for either side:

```rust path=src/reading.rs
//! Reading a project: its hand-off file, its files, its packages, its scripts' imports and where they
//! lead, its entry points, and which export of which file is used. `graph` prints a reading, and
//! `audit` reports what one shows (spec OUT-1, CMD-1, CMD-4).
//!
//! The working tree and a base commit are read the same way, each from its own tree and file list
//! (dead-code spec COMMIT-1).

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File, Listing};
use crate::hand_off::{self, HandOff};
use crate::marks::{self, Marks};
use crate::notice::Notice;
use crate::packages::{self, Packages};
use crate::resolve::{Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tree::Tree;
use crate::tsconfig;
use crate::uses::{self, Uses};

/// What reading a project found.
pub struct Reading {
    /// Whether a hand-off file was given.
    pub handed: bool,
    /// The files of the list, in its order.
    pub files: Vec<File>,
    /// Which files the declarations mark as data or as generated.
    pub marks: Marks,
    /// The packages, and which package each file belongs to.
    pub packages: Packages,
    /// What each file that was read holds.
    pub scripts: Scripts,
    /// Where each import of each file leads, by the file's path, in the order of its imports.
    pub resolved: BTreeMap<String, Vec<Resolution>>,
    /// The entry points, each with every source that made it one.
    pub entries: Entries,
    /// Every use of an export, and the files the entry points reach.
    pub uses: Uses,
    /// What a person should know about the run, in the order it was found.
    pub notices: Vec<Notice>,
}

/// What every reading of a run starts from: the hand-off file, the root, and the notices so far.
pub struct Start {
    /// The hand-off file, where one was given.
    pub handed: Option<HandOff>,
    /// The root.
    pub root: PathBuf,
    /// What a person should know about the run so far.
    pub notices: Vec<Notice>,
}

/// Reads the hand-off file at the path given, or notes that there is none, and finds the root (spec
/// RUN-4, steps 1 to 5; LIST-1).
pub fn start(hand_off_path: Option<&Path>) -> Result<Start, Stop> {
    let mut notices = Vec::new();
    let handed = match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice::new(
                    "detection-cut-short",
                    None,
                    "yak's detection stopped at its limit of folders, so the folders the hand-off file lists \
                     are not the whole workspace."
                        .to_owned(),
                ));
            }

            Some(handed)
        }
        None => {
            notices.push(Notice::new(
                "no-hand-off-file",
                None,
                "No hand-off file was given, so the project is read from its own standard files, with no \
                 declarations."
                    .to_owned(),
            ));

            None
        }
    };

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match &handed {
        Some(handed) => handed.root.clone(),
        None => env::current_dir().map_err(|error| {
            Stop::new(format!(
                "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
            ))
        })?,
    };

    Ok(Start {
        handed,
        root,
        notices,
    })
}

/// Reads the working tree, with the hand-off file at the path given or with none.
pub fn working(hand_off_path: Option<&Path>) -> Result<Reading, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = start(hand_off_path)?;
    let listing = files::list(&root, &mut notices)?;

    read(handed.as_ref(), &Tree::working(root), listing, notices)
}

/// Reads a project from a tree and its file list, with the hand-off file's declarations where there is
/// one (dead-code spec COMMIT-5).
pub fn read(
    handed: Option<&HandOff>,
    tree: &Tree,
    listing: Listing,
    mut notices: Vec<Notice>,
) -> Result<Reading, Stop> {
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed, &mut notices)?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = packages::load(tree, files, &marks.data, &mut notices)?.checked(
        files,
        handed.map(|handed| handed.folders.as_slice()),
        &mut notices,
    );
    let configs = tsconfig::load(
        tree,
        files,
        &marks.data,
        packages.as_ref().ok(),
        &mut notices,
    )?
    .checked(&mut notices);
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(tree, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(tree.root(), &scripts)?;
    let entries = entries::find(
        &entries::Project {
            tree,
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed,
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);

    // git failing while the base was read stops the run (dead-code spec COMMIT-4).
    tree.failure()?;

    Ok(Reading {
        handed: handed.is_some(),
        files: listing.files,
        marks,
        packages,
        scripts,
        resolved,
        entries,
        uses,
        notices,
    })
}
```

```rust edit path=src/graph.rs
<<<<<<< find
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
        "reachable": reachable(&uses),
=======
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed,
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
        "reachable": reachable(&uses),
>>>>>>> replace
```

New against inherited, and the report that prints it:

```rust edit path=src/findings.rs
<<<<<<< find
    pub column: Option<usize>,
}

/// One finding.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
=======
    pub column: Option<usize>,
}

/// Whether a finding is new on the branch or the base has it too (spec REPORT-1, INH-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Baseline {
    /// The branch made it.
    New,
    /// The base has it too: it is inherited.
    Unchanged,
}

impl Baseline {
    /// Its name as SARIF's `baselineState` writes it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Unchanged => "unchanged",
        }
    }
}

/// One finding.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
>>>>>>> replace
<<<<<<< find
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
}

impl Finding {
=======
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
}

impl Finding {
>>>>>>> replace
<<<<<<< find
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
        }
    }
}
=======
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
            baseline: None,
        }
    }
}
>>>>>>> replace
<<<<<<< find
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

    found.sort_by(|a, b| {
        (a.rule, &a.location, &a.fingerprint).cmp(&(b.rule, &b.location, &b.fingerprint))
    });

    (found, notices)
}

/// Whether the three kinds that depend on entry points are reported for a file: its package's own
=======
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

    sort(&mut found);

    (found, notices)
}

/// Puts findings in the report's order: by rule, then path, line and column, then fingerprint (spec
/// REPORT-1).
pub fn sort(found: &mut [Finding]) {
    found.sort_by(|a, b| {
        (a.rule, &a.location, &a.fingerprint).cmp(&(b.rule, &b.location, &b.fingerprint))
    });
}

/// Whether the three kinds that depend on entry points are reported for a file: its package's own
>>>>>>> replace
```

```rust path=src/inherit.rs
//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too (dead-code spec INH-1, COMMIT-4, REPORT-4).

use std::collections::BTreeMap;

use crate::findings::{self, Baseline, Finding};
use crate::notice::{Notice, SKIPPED};
use crate::reading::Reading;

/// What the base gave: its commit and its reading.
pub struct Base {
    /// The commit's full name.
    pub commit: String,
    /// The commit's reading.
    pub reading: Reading,
}

/// Marks each finding of the working tree new or inherited, and gives the notices the comparison makes
/// (dead-code spec INH-1, COMMIT-4, REPORT-4).
pub fn mark(found: &mut [Finding], base: &Base) -> Vec<Notice> {
    let skipped: Vec<&str> = base
        .reading
        .notices
        .iter()
        .filter(|notice| notice.kind() == SKIPPED)
        .filter_map(Notice::file)
        .collect();
    let mut notices = vec![Notice::new(
        "base",
        None,
        format!(
            "The findings are compared with the base commit {}: one the base has too is marked unchanged, \
             and any other new.",
            base.commit
        ),
    )];

    // A file skipped at the base can leave it with findings it never had, which would make the same
    // findings on the branch look inherited: so then every finding is new (dead-code spec COMMIT-4).
    if !skipped.is_empty() {
        notices.push(Notice::new(
            "base-files-skipped",
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding is marked unchanged: `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
        ));

        for finding in found.iter_mut() {
            finding.baseline = Some(Baseline::New);
        }

        return notices;
    }

    let (base_found, _) = findings::find(&base.reading);
    // How many findings of each fingerprint the base has.
    let mut held: BTreeMap<&str, usize> = BTreeMap::new();

    for finding in &base_found {
        *held.entry(&finding.fingerprint).or_default() += 1;
    }

    // The findings are in the report's order, so of those that share a fingerprint, the base's count
    // are taken first by line and column (dead-code spec INH-1).
    for finding in found.iter_mut() {
        let inherited = held
            .get_mut(finding.fingerprint.as_str())
            .filter(|count| **count > 0)
            .map(|count| *count -= 1)
            .is_some();

        finding.baseline = Some(if inherited {
            Baseline::Unchanged
        } else {
            Baseline::New
        });
    }

    notices
}
```

```rust edit path=src/audit.rs
<<<<<<< find
//! The `audit` command: prints what a reading shows as one SARIF 2.1.0 document with one run (spec
//! CMD-1, REPORT-1).
//!
//! The run names the tool and lists the five rules, the findings and the notices.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::findings::{self, Finding, Location, Rule};
use crate::notice::{self, Notice};
use crate::reading::Reading;

/// The key of `partialFingerprints` that holds a finding's fingerprint, with the scheme's version in
/// it, so that a later scheme can sit beside it (spec REPORT-3).
=======
//! The `audit` command: reads the project, and with a base the base commit too, and prints what the
//! reading shows as one SARIF 2.1.0 document with one run (spec CMD-1, REPORT-1, COMMIT-1, INH-4).
//!
//! The run names the tool and lists the five rules, the findings and the notices.

use std::fmt::Write as _;
use std::path::Path;
use std::thread;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::files;
use crate::findings::{self, Finding, Location, Rule};
use crate::git;
use crate::inherit::{self, Base};
use crate::notice::{self, Notice};
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::tree::Tree;

/// The key of `partialFingerprints` that holds a finding's fingerprint, with the scheme's version in
/// it, so that a later scheme can sit beside it (spec REPORT-3).
>>>>>>> replace
<<<<<<< find
/// ones, the sub-delimiters, `@`, and the `/` between segments (RFC 3986).
const PLAIN: &str = "-._~!$&'()*+,;=@/";

/// The report `audit` prints for a reading.
pub fn printed(mut reading: Reading) -> String {
    let (found, notices) = findings::find(&reading);

    reading.notices.extend(notices);

    let mut document = json!({
        "$schema": SCHEMA,
        "version": SARIF_VERSION,
=======
/// ones, the sub-delimiters, `@`, and the `/` between segments (RFC 3986).
const PLAIN: &str = "-._~!$&'()*+,;=@/";

/// Reads the project, with the hand-off file given or with none, and with the base given the base
/// commit too, and returns the report (spec CMD-1, CMD-3). The stops come in RUN-4's order: the base is
/// asked about after the working tree's file list, as its steps 7b and 7c.
pub fn run(hand_off: Option<&Path>, base: Option<&str>) -> Result<String, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = reading::start(hand_off)?;
    let listing = files::list(&root, &mut notices)?;
    let Some(base) = base else {
        return reading::read(handed.as_ref(), &Tree::working(root), listing, notices)
            .map(|reading| printed(reading, None));
    };
    let commit = git::commit(&root, base)?;

    if git::partial(&root)?
        && let Some(version) = git::older_than(git::NO_LAZY_FETCH)
    {
        let (major, minor) = git::NO_LAZY_FETCH;

        return Err(Stop::new(format!(
            "the root is in a partial clone, and git {version} cannot read the base without fetching over the \
             network; a partial clone needs git {major}.{minor} or newer, so update git"
        )));
    }

    let mut base_notices = Vec::new();
    let (base_tree, base_listing) = Tree::base(root.clone(), &commit, &mut base_notices)?;
    let working_tree = Tree::working(root);

    // The two sides are read at once (dead-code spec INH-4).
    let (working, base) = thread::scope(|scope| {
        let base = scope.spawn(|| {
            let reading = reading::read(handed.as_ref(), &base_tree, base_listing, base_notices)?;

            Ok::<_, Stop>(Base {
                commit: commit.clone(),
                reading,
            })
        });
        let working = reading::read(handed.as_ref(), &working_tree, listing, notices);
        let base = base
            .join()
            .unwrap_or_else(|fault| std::panic::resume_unwind(fault));

        Ok::<_, Stop>((working?, base?))
    })?;

    Ok(printed(working, Some(base)))
}

/// The report `audit` prints for a reading, compared with the base's where there is one.
pub fn printed(mut reading: Reading, base: Option<Base>) -> String {
    let (mut found, notices) = findings::find(&reading);

    reading.notices.extend(notices);

    if let Some(base) = base {
        let compared = inherit::mark(&mut found, &base);

        reading.notices.extend(compared);
        reading
            .notices
            .extend(base.reading.notices.into_iter().map(Notice::of_base));
    }

    let mut document = json!({
        "$schema": SCHEMA,
        "version": SARIF_VERSION,
>>>>>>> replace
<<<<<<< find
        "ruleIndex": finding.rule.index(),
    });

    if !finding.related.is_empty() {
        result["relatedLocations"] = finding
            .related
=======
        "ruleIndex": finding.rule.index(),
    });

    if let Some(baseline) = finding.baseline {
        result["baselineState"] = json!(baseline.name());
    }

    if !finding.related.is_empty() {
        result["relatedLocations"] = finding
            .related
>>>>>>> replace
<<<<<<< find
            json!([{ "physicalLocation": { "artifactLocation": artifact(file) } }]);
    }

    notification
}

=======
            json!([{ "physicalLocation": { "artifactLocation": artifact(file) } }]);
    }

    // A notice of the base's reading says so in its property bag (dead-code spec REPORT-4).
    if notice.is_base() {
        notification["properties"] = json!({ "base": true });
    }

    notification
}

>>>>>>> replace
```

The command line:

```rust path=src/cli.rs
//! The command line: what the binary was asked to do (spec RUN-1, RUN-2; dead-code spec CMD-1, CMD-2).

use std::ffi::OsString;
use std::path::PathBuf;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The command that prints the interface version and the hand-off formats the analyzer reads.
const INTERFACE_COMMAND: &str = "interface";

/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// The command that reads a project and prints a report of what it found (spec CMD-1).
const AUDIT_COMMAND: &str = "audit";

/// The option of `graph` and of `audit` that names the hand-off file.
const HAND_OFF_OPTION: &str = "--hand-off";

/// The option of `audit` that names the commit to compare with (dead-code spec CMD-1).
const BASE_OPTION: &str = "--base";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, graph [--hand-off <file>], and audit [--hand-off <file>] \
                        [--base <commit>]";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the version.
    Version,
    /// Print the interface version and the hand-off formats.
    Interface,
    /// Read the project and print what was read, with a hand-off file or without one.
    Graph { hand_off: Option<PathBuf> },
    /// Read the project and print a report of what was found, with a hand-off file or without one, and
    /// compared with a base commit or with none.
    Audit {
        hand_off: Option<PathBuf>,
        base: Option<String>,
    },
}

/// Reads the arguments after the program's name. The error is the text of the refusal.
pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let words: Vec<&str> = args.iter().map(|arg| arg.to_str().unwrap_or("")).collect();

    match words.as_slice() {
        [VERSION_OPTION] => Ok(Command::Version),
        [INTERFACE_COMMAND] => Ok(Command::Interface),
        [GRAPH_COMMAND] => Ok(Command::Graph { hand_off: None }),
        [GRAPH_COMMAND, HAND_OFF_OPTION, _] => Ok(Command::Graph {
            hand_off: Some(PathBuf::from(&args[2])),
        }),
        [AUDIT_COMMAND, ..] => audit(args),
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => Err(refused(args)),
    }
}

/// `audit` and its options, each at most once, in either order. A base that begins with `-`, or holds
/// a control character, is refused before git sees it (dead-code spec CMD-2).
fn audit(args: &[OsString]) -> Result<Command, String> {
    let mut hand_off = None;
    let mut base = None;

    for pair in args[1..].chunks(2) {
        let [option, value] = pair else {
            return Err(refused(args));
        };

        match option.to_str() {
            Some(HAND_OFF_OPTION) if hand_off.is_none() => hand_off = Some(PathBuf::from(value)),
            Some(BASE_OPTION) if base.is_none() => {
                let named = value.to_string_lossy();

                if named.is_empty() || named.starts_with('-') || named.chars().any(char::is_control)
                {
                    return Err(format!(
                        "the base `{named}` cannot be a commit's name: it is empty, begins with `-` or holds a \
                         control character, which git would not read as a name; name the commit another way"
                    ));
                }

                base = Some(named.into_owned());
            }
            _ => return Err(refused(args)),
        }
    }

    Ok(Command::Audit { hand_off, base })
}

/// The refusal of a command line the binary does not answer.
fn refused(args: &[OsString]) -> String {
    let given = args
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    format!("cannot do `{given}`; it answers {ACCEPTED}")
}
```

```rust edit path=src/main.rs
<<<<<<< find
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, and `audit` prints a report of the findings it shows.

mod audit;
mod cli;
=======
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, and `audit` prints a report of the findings it shows,
//! compared with a base commit where it is given one.

mod audit;
mod cli;
>>>>>>> replace
<<<<<<< find
mod graph;
mod hand_off;
mod imports;
mod marks;
mod notice;
mod packages;
=======
mod graph;
mod hand_off;
mod imports;
mod inherit;
mod marks;
mod notice;
mod packages;
>>>>>>> replace
<<<<<<< find
mod scripts;
mod stop;
mod tools;
mod tsconfig;
mod uses;

=======
mod scripts;
mod stop;
mod tools;
mod tree;
mod tsconfig;
mod uses;

>>>>>>> replace
<<<<<<< find
            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => {
            ended(reading::read(hand_off.as_deref()).map(graph::printed))
        }
        Ok(Command::Audit { hand_off }) => {
            ended(reading::read(hand_off.as_deref()).map(audit::printed))
        }
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));
=======
            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => {
            ended(reading::working(hand_off.as_deref()).map(graph::printed))
        }
        Ok(Command::Audit { hand_off, base }) => {
            ended(audit::run(hand_off.as_deref(), base.as_deref()))
        }
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test cli && cargo test --release --locked --test audit
```

Expected: PASS: 8 tests in `cli`, 20 in `audit`.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two, 52 of them in the node
tests.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add src/tree.rs src/inherit.rs src/git.rs src/files.rs src/reading.rs src/packages.rs src/tsconfig.rs src/scripts.rs src/entries.rs src/graph.rs src/notice.rs src/stop.rs src/findings.rs src/audit.rs src/cli.rs src/main.rs tests/cli.test.rs tests/audit.test.rs
git commit -m "feat(audit): compare with a base commit and mark findings new or inherited"
```

Expected: one commit on `feat/dead-code-5-base`.

### Task 2: renames and cycles

**Implements:** INH-2, INH-3, FIX-1

**Test:** `cargo test --release --locked --test audit`

**Files:**
- Modify: `src/git.rs`, `src/findings.rs`, `src/inherit.rs`, `src/audit.rs`
- Test: `tests/audit.test.rs`

**Interfaces:**
- Consumes: task 1's `Base`, `mark`, `Finding` and `Baseline`; plan 4's `edges` and
  `strongly_connected`.
- Produces: `git::renamed` and `git::RENAME_LIMIT`; `Finding::moved` and `Finding::files`;
  `findings::Graph` (`of`, `pairs`, `sets`, `first`); `inherit::renames`; `Base::renames` and
  `Base::renames_cut_short`; `inherit::mark(working, found, base)`.

- [ ] **Step 1: Write the failing tests**

Two tests join `tests/audit.test.rs`: files renamed in a commit, renamed and staged, and removed and
written again untracked; and cycles the base has, a file joining one, and an import added inside one:

```rust edit path=tests/audit.test.rs
<<<<<<< find
        assert!(line.contains("2.44"), "{line}");
    }
}
=======
        assert!(line.contains("2.44"), "{line}");
    }
}

#[test]
fn a_renamed_files_findings_are_inherited_under_its_new_path() {
    let project = branched(
        "base-renames",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            ("src/index.ts", "import { kept } from './kept';\nkept();\n"),
            (
                "src/kept.ts",
                "export const kept = (): void => {};\nexport const spare = 1;\n",
            ),
            ("src/moved.ts", "export const moved = 1;\n"),
            ("src/staged.ts", "export const staged = 1;\n"),
            ("src/copied.ts", "export const copied = 1;\n"),
        ],
    );

    // Renamed in a commit, and the import that names it changed with it; and moved to a new folder.
    fs::create_dir_all(project.root.join("src/lib")).expect("the folder can be made");
    project.git(&["mv", "src/kept.ts", "src/held.ts"]);
    project.git(&["mv", "src/moved.ts", "src/lib/moved.ts"]);
    project
        .write("src/index.ts", "import { kept } from './held';\nkept();\n")
        .commit();
    // Renamed and staged, not committed.
    project.git(&["mv", "src/staged.ts", "src/staged-now.ts"]);
    // Removed, and the same content written again as an untracked file, which git cannot pair with it.
    project.git(&["rm", "--quiet", "src/copied.ts"]);
    project.write("src/copy.ts", "export const copied = 1;\n");

    let run = audited(&project, &["--base", "main"]);

    // Before fingerprints are compared, each base path git found renamed takes its new path (spec
    // INH-2, FIX-1 pattern 22).
    assert_eq!(
        states(&run),
        [
            ("unused-file", "src/copy.ts", None, Some("new")),
            ("unused-file", "src/lib/moved.ts", None, Some("unchanged")),
            ("unused-file", "src/staged-now.ts", None, Some("unchanged")),
            ("unused-export", "src/held.ts", Some(2), Some("unchanged")),
        ]
    );
}

#[test]
fn a_cycle_the_base_has_is_unchanged_and_one_the_branch_joined_or_grew_is_new() {
    let project = branched(
        "base-cycles",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            (
                "src/index.ts",
                "import './a';\nimport './c';\nimport './p';\n",
            ),
            ("src/a.ts", "import './b';\nexport {};\n"),
            ("src/b.ts", "import './a';\nexport {};\n"),
            ("src/c.ts", "import './d';\nexport {};\n"),
            ("src/d.ts", "import './c';\nexport {};\n"),
            ("src/p.ts", "import './q';\nexport {};\n"),
            ("src/q.ts", "import './r';\nexport {};\n"),
            ("src/r.ts", "import './p';\nexport {};\n"),
        ],
    );

    // The branch rewrites a specifier without changing where it leads, has a new file join a cycle,
    // and adds an import inside another (spec FIX-1 pattern 18).
    project
        .write("src/b.ts", "import './a.js';\nexport {};\n")
        .write("src/d.ts", "import './c';\nimport './x';\nexport {};\n")
        .write("src/x.ts", "import './c';\nexport {};\n")
        .write("src/q.ts", "import './r';\nimport './p';\nexport {};\n")
        .commit();

    let run = audited(&project, &["--base", "main"]);
    let cycles: Vec<_> = states(&run)
        .into_iter()
        .filter(|(rule, ..)| *rule == "circular-dependency")
        .collect();

    // A cycle is inherited when one set of the base holds all its files and the branch added no edge
    // between them; otherwise it is new, at the first import that adds one (spec INH-3).
    assert_eq!(
        cycles,
        [
            (
                "circular-dependency",
                "src/a.ts",
                Some(1),
                Some("unchanged")
            ),
            ("circular-dependency", "src/d.ts", Some(2), Some("new")),
            ("circular-dependency", "src/q.ts", Some(2), Some("new")),
        ]
    );
    assert_eq!(
        message(&run, "circular-dependency", "src/d.ts"),
        "These files import each other in a loop: `src/c.ts` → `src/d.ts` → `src/c.ts`. The loop is part \
         of a set of 3 files that each reach all the others. The branch added the import of `./x` on line \
         2 of `src/d.ts`, which leads to `src/x.ts`."
    );
    assert_eq!(
        message(&run, "circular-dependency", "src/q.ts"),
        "These files import each other in a loop: `src/p.ts` → `src/q.ts` → `src/p.ts`. The loop is part \
         of a set of 3 files that each reach all the others. The branch added the import of `./p` on line \
         2 of `src/q.ts`, which leads to `src/p.ts`."
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test audit
```

Expected: FAIL: the two new tests fail, since a renamed file's findings are new and a cycle is still
compared by its fingerprint. The other twenty pass.

- [ ] **Step 3: Write the code**

git finds the renames:

```rust edit path=src/git.rs
<<<<<<< find
//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2; dead-code spec CMD-2, COMMIT-2, LIFE-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
=======
//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2; dead-code spec CMD-2, COMMIT-2, INH-2,
//! LIFE-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
>>>>>>> replace
<<<<<<< find
/// (dead-code spec COMMIT-2). A git older than `NO_LAZY_FETCH` passes it over.
const NO_LAZY_FETCH_VARIABLE: &str = "GIT_NO_LAZY_FETCH";

/// What git printed, and the first line of a warning it gave while succeeding.
pub struct Answer {
    /// What git wrote on standard output.
=======
/// (dead-code spec COMMIT-2). A git older than `NO_LAZY_FETCH` passes it over.
const NO_LAZY_FETCH_VARIABLE: &str = "GIT_NO_LAZY_FETCH";

/// How many files git compares when it looks for renames, whatever the user's or the repository's
/// settings say: git's own default (dead-code spec INH-2).
pub const RENAME_LIMIT: u32 = 1000;

/// What git printed, and the first line of a warning it gave while succeeding.
pub struct Answer {
    /// What git wrote on standard output.
>>>>>>> replace
<<<<<<< find
    read(root, &["ls-tree", "-r", "-z", commit])
}

/// A git that hands out the content of one stored object after another, started once for a whole
/// reading, so no file needs a start of git of its own (dead-code spec COMMIT-2). Its input is closed
/// when it is dropped, so it ends with the analyzer (dead-code spec LIFE-2).
=======
    read(root, &["ls-tree", "-r", "-z", commit])
}

/// The files renamed between a commit and the working tree, under the root, as git finds them with its
/// own rename detection and its default similarity (dead-code spec INH-2).
pub fn renamed(root: &Path, commit: &str) -> Result<Answer, Stop> {
    let limit = format!("-l{RENAME_LIMIT}");

    read(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--name-status",
            "-z",
            "--relative",
            "-M",
            &limit,
            commit,
            "--",
        ],
    )
}

/// A git that hands out the content of one stored object after another, started once for a whole
/// reading, so no file needs a start of git of its own (dead-code spec COMMIT-2). Its input is closed
/// when it is dropped, so it ends with the analyzer (dead-code spec LIFE-2).
>>>>>>> replace
```

A finding keeps its fingerprint's parts, and the graph of loads is shared:

```rust edit path=src/findings.rs
<<<<<<< find
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
}
=======
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
    /// What the fingerprint is made from, after the rule: paths first, as REPORT-3 lists them.
    parts: Vec<String>,
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
}
>>>>>>> replace
<<<<<<< find
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
            baseline: None,
        }
    }
}

/// The fingerprint of a finding: the first digits of the SHA-256 digest of its rule and its parts, each
=======
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
            parts: parts.iter().map(|part| (*part).to_owned()).collect(),
            baseline: None,
        }
    }

    /// The fingerprint the finding would have with each path it names moved as `moved` says, as a
    /// rename moves a base's file to its path on the branch (spec INH-2). A cycle's paths are sorted
    /// again after they move.
    pub fn moved(&self, moved: impl Fn(&str) -> String) -> String {
        let parts: Vec<String> = if self.rule == Rule::CircularDependency {
            let mut paths: Vec<String> = self.parts.iter().map(|path| moved(path)).collect();

            paths.sort();
            paths
        } else {
            // The first part is the path; a name or a specifier after it does not move.
            self.parts
                .iter()
                .enumerate()
                .map(|(place, part)| {
                    if place == 0 {
                        moved(part)
                    } else {
                        part.clone()
                    }
                })
                .collect()
        };
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

        fingerprint(self.rule, &parts)
    }

    /// The files of a circular dependency: its first location's, then its related locations'.
    pub fn files(&self) -> Vec<&str> {
        std::iter::once(&self.location)
            .chain(&self.related)
            .map(|location| location.path.as_str())
            .collect()
    }
}

/// The fingerprint of a finding: the first digits of the SHA-256 digest of its rule and its parts, each
>>>>>>> replace
<<<<<<< find
    (nodes, edges)
}

/// The strongly connected sets of a graph, each in the order of its nodes, found by Tarjan's walk
/// written as a loop over a stack of its own, so that no chain of files is too long for the stack (spec
/// LIFE-3).
=======
    (nodes, edges)
}

/// The files cycles are found over, and which file loads which, by their paths (spec KIND-4, INH-3).
pub struct Graph<'r> {
    /// The files, in path order.
    nodes: Vec<&'r str>,
    /// What each file loads.
    edges: Edges,
}

impl<'r> Graph<'r> {
    /// The graph of a reading.
    pub fn of(reading: &'r Reading) -> Self {
        let (nodes, edges) = edges(reading);

        Self { nodes, edges }
    }

    /// Each pair of files where the first loads the second.
    pub fn pairs(&self) -> Vec<(&'r str, &'r str)> {
        self.edges
            .iter()
            .enumerate()
            .flat_map(|(from, targets)| {
                targets
                    .keys()
                    .map(move |to| (self.nodes[from], self.nodes[*to]))
            })
            .collect()
    }

    /// The strongly connected set each file is in, as the set's number, by the file's path.
    pub fn sets(&self) -> BTreeMap<&'r str, usize> {
        strongly_connected(&self.edges)
            .iter()
            .enumerate()
            .flat_map(|(number, set)| set.iter().map(move |node| (self.nodes[*node], number)))
            .collect()
    }

    /// The first import, by line, by which one file loads another: its line, and its place among the
    /// file's imports.
    pub fn first(&self, from: &str, to: &str) -> Option<(usize, usize)> {
        let from = self.nodes.binary_search(&from).ok()?;
        let to = self.nodes.binary_search(&to).ok()?;

        self.edges[from].get(&to).copied()
    }
}

/// The strongly connected sets of a graph, each in the order of its nodes, found by Tarjan's walk
/// written as a loop over a stack of its own, so that no chain of files is too long for the stack (spec
/// LIFE-3).
>>>>>>> replace
```

The comparison moves the base's paths and judges cycles by their edges:

```rust path=src/inherit.rs
//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too, through the files git finds renamed between them (dead-code spec INH-1 to INH-3,
//! COMMIT-4, REPORT-4).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::findings::{self, Baseline, Finding, Graph, Location, Rule};
use crate::git;
use crate::notice::{Notice, SKIPPED};
use crate::reading::Reading;

/// What the base gave: its commit, its reading, and the files renamed since.
pub struct Base {
    /// The commit's full name.
    pub commit: String,
    /// The commit's reading.
    pub reading: Reading,
    /// Each base path git found renamed, with its path on the branch.
    pub renames: BTreeMap<String, String>,
    /// Whether git gave up looking for renames for its limit.
    pub renames_cut_short: bool,
}

/// The files git found renamed, from what `git diff --name-status -z` printed: each base path with its
/// path on the branch (dead-code spec INH-2).
pub fn renames(output: &[u8]) -> BTreeMap<String, String> {
    let mut fields = output
        .split(|byte| *byte == 0)
        .map(|field| String::from_utf8_lossy(field).into_owned());
    let mut renames = BTreeMap::new();

    // A rename or a copy is its letter and score, then two paths; any other change, its letter and one.
    while let Some(status) = fields.next() {
        match status.chars().next() {
            Some('R') => {
                if let (Some(old), Some(new)) = (fields.next(), fields.next()) {
                    renames.insert(old, new);
                }
            }
            Some('C') => {
                fields.next();
                fields.next();
            }
            Some(_) => {
                fields.next();
            }
            None => {}
        }
    }

    renames
}

/// Marks each finding of the working tree new or inherited, and gives the notices the comparison makes
/// (dead-code spec INH-1, INH-3, COMMIT-4, REPORT-4).
pub fn mark(working: &Reading, found: &mut [Finding], base: &Base) -> Vec<Notice> {
    let skipped: Vec<&str> = base
        .reading
        .notices
        .iter()
        .filter(|notice| notice.kind() == SKIPPED)
        .filter_map(Notice::file)
        .collect();
    let mut notices = vec![Notice::new(
        "base",
        None,
        format!(
            "The findings are compared with the base commit {}: one the base has too is marked unchanged, \
             and any other new.",
            base.commit
        ),
    )];

    if base.renames_cut_short {
        notices.push(Notice::new(
            "renames-cut-short",
            None,
            format!(
                "git stopped looking for renamed files at its limit of {}, so a finding in a file renamed \
                 since the base may be marked new.",
                git::RENAME_LIMIT
            ),
        ));
    }

    // A file skipped at the base can leave it with findings it never had, which would make the same
    // findings on the branch look inherited: so then every finding is new (dead-code spec COMMIT-4).
    if !skipped.is_empty() {
        notices.push(Notice::new(
            "base-files-skipped",
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding is marked unchanged: `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
        ));

        for finding in found.iter_mut() {
            finding.baseline = Some(Baseline::New);
        }

        return notices;
    }

    let moved = |path: &str| {
        base.renames
            .get(path)
            .cloned()
            .unwrap_or_else(|| path.to_owned())
    };
    let (base_found, _) = findings::find(&base.reading);
    // How many findings of each fingerprint the base has, with its paths moved to the branch's.
    let mut held: BTreeMap<String, usize> = BTreeMap::new();

    for finding in base_found
        .iter()
        .filter(|finding| finding.rule != Rule::CircularDependency)
    {
        *held.entry(finding.moved(moved)).or_default() += 1;
    }

    let working_graph = Graph::of(working);
    let base_graph = Graph::of(&base.reading);
    let base_pairs: BTreeSet<(String, String)> = base_graph
        .pairs()
        .into_iter()
        .map(|(from, to)| (moved(from), moved(to)))
        .collect();
    let base_sets: BTreeMap<String, usize> = base_graph
        .sets()
        .into_iter()
        .map(|(path, set)| (moved(path), set))
        .collect();

    // The findings are in the report's order, so of those that share a fingerprint, the base's count
    // are taken first by line and column (dead-code spec INH-1).
    for finding in found.iter_mut() {
        if finding.rule == Rule::CircularDependency {
            cycle(working, &working_graph, finding, &base_pairs, &base_sets);

            continue;
        }

        let inherited = held
            .get_mut(&finding.fingerprint)
            .filter(|count| **count > 0)
            .map(|count| *count -= 1)
            .is_some();

        finding.baseline = Some(if inherited {
            Baseline::Unchanged
        } else {
            Baseline::New
        });
    }

    findings::sort(found);

    notices
}

/// Marks a circular dependency: inherited when one strongly connected set of the base holds all its
/// files and the branch added no edge between two of them; otherwise new, placed at the first import, by
/// path and line, that makes an edge the branch added (dead-code spec INH-3).
fn cycle(
    working: &Reading,
    graph: &Graph<'_>,
    finding: &mut Finding,
    base_pairs: &BTreeSet<(String, String)>,
    base_sets: &BTreeMap<String, usize>,
) {
    let files: Vec<String> = finding.files().into_iter().map(str::to_owned).collect();
    let one_set = files
        .iter()
        .map(|file| base_sets.get(file))
        .collect::<Option<BTreeSet<_>>>()
        .is_some_and(|sets| sets.len() == 1);
    // Each edge between two of its files that the base lacks: the importing file, the line and place
    // of its first import that makes it, and the file it leads to.
    let mut added: Vec<(&str, usize, usize, &str)> = Vec::new();

    for from in &files {
        for to in &files {
            if let Some((line, place)) = graph.first(from, to)
                && !base_pairs.contains(&(from.clone(), to.clone()))
            {
                added.push((from, line, place, to));
            }
        }
    }

    added.sort_unstable();

    let Some(&(from, line, place, to)) = added.first() else {
        finding.baseline = Some(if one_set {
            Baseline::Unchanged
        } else {
            Baseline::New
        });

        return;
    };
    let specifier = working
        .scripts
        .read
        .get(from)
        .and_then(|read| read.imports.get(place))
        .map_or("", |import| import.specifier.as_str());
    let mut locations: Vec<Location> = std::iter::once(finding.location.clone())
        .chain(finding.related.drain(..))
        .filter(|location| location.path != from)
        .collect();

    locations.sort();
    finding.location = Location {
        path: from.to_owned(),
        line: Some(line),
        column: None,
    };
    finding.related = locations;
    let _ = write!(
        finding.message,
        " The branch added the import of `{specifier}` on line {line} of `{from}`, which leads to `{to}`."
    );
    finding.baseline = Some(Baseline::New);
}
```

```rust edit path=src/audit.rs
<<<<<<< find
    let (working, base) = thread::scope(|scope| {
        let base = scope.spawn(|| {
            let reading = reading::read(handed.as_ref(), &base_tree, base_listing, base_notices)?;

            Ok::<_, Stop>(Base {
                commit: commit.clone(),
                reading,
            })
        });
        let working = reading::read(handed.as_ref(), &working_tree, listing, notices);
=======
    let (working, base) = thread::scope(|scope| {
        let base = scope.spawn(|| {
            let reading = reading::read(handed.as_ref(), &base_tree, base_listing, base_notices)?;
            let renamed = git::renamed(base_tree.root(), &commit)?;

            Ok::<_, Stop>(Base {
                commit: commit.clone(),
                reading,
                renames: inherit::renames(&renamed.output),
                renames_cut_short: renamed
                    .warning
                    .is_some_and(|warning| warning.contains("rename")),
            })
        });
        let working = reading::read(handed.as_ref(), &working_tree, listing, notices);
>>>>>>> replace
<<<<<<< find
    reading.notices.extend(notices);

    if let Some(base) = base {
        let compared = inherit::mark(&mut found, &base);

        reading.notices.extend(compared);
        reading
=======
    reading.notices.extend(notices);

    if let Some(base) = base {
        let compared = inherit::mark(&reading, &mut found, &base);

        reading.notices.extend(compared);
        reading
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test audit
```

Expected: PASS: 22 tests.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add src/git.rs src/findings.rs src/inherit.rs src/audit.rs tests/audit.test.rs
git commit -m "feat(audit): follow renames and compare cycles by their edges"
```

Expected: one commit on `feat/dead-code-5-base`.

### Task 3: the life of a run

**Implements:** LIFE-1, LIFE-2, LIFE-3, FIX-2

**Test:** `node --test npm/yak-analyzer/test/launcher.test.mjs && cargo test --release --locked --test life`

**Files:**
- Create: `src/launcher.rs`
- Modify: `src/main.rs`, `src/audit.rs`, `Cargo.toml`, `npm/yak-analyzer/lib/launch.js`,
  `npm/yak-analyzer/bin/yak-analyzer.js`
- Test: `npm/yak-analyzer/test/launcher.test.mjs`, `tests/life.test.rs`

**Interfaces:**
- Consumes: task 1's `audit::run` and `git::Batch`; the launcher's `launch`.
- Produces: `launcher::watch`; `STACK`; the launcher's `LAUNCHER_VARIABLE` and `binaryEnvironment`.

- [ ] **Step 1: Write the failing tests**

The launcher's tests check what it adds to the binary's environment:

```js edit path=npm/yak-analyzer/test/launcher.test.mjs
<<<<<<< find
import { join } from 'node:path';
import { test } from 'node:test';

import { launch, linuxLibc, TARGETS } from '#yak-analyzer/launch.js';

/** A folder that stands for an installed platform package. */
const INSTALLED = join('made-up', 'node_modules', '@dabay6', 'yak-analyzer-linux-x64');
=======
import { join } from 'node:path';
import { test } from 'node:test';

import { binaryEnvironment, LAUNCHER_VARIABLE, launch, linuxLibc, TARGETS } from '#yak-analyzer/launch.js';

/** A folder that stands for an installed platform package. */
const INSTALLED = join('made-up', 'node_modules', '@dabay6', 'yak-analyzer-linux-x64');
>>>>>>> replace
<<<<<<< find
  }
});

test('reads the C library only on Linux', () => {
  if (process.platform === 'linux') {
    assert.ok(['glibc', 'musl'].includes(linuxLibc()));
=======
  }
});

test('hands the binary its environment with its own process id added (dead-code spec LIFE-1)', () => {
  const given = { PATH: '/usr/bin', [LAUNCHER_VARIABLE]: 'stale' };

  assert.equal(LAUNCHER_VARIABLE, 'YAK_ANALYZER_LAUNCHER');
  assert.deepEqual(binaryEnvironment(given, 4242), { PATH: '/usr/bin', YAK_ANALYZER_LAUNCHER: '4242' });
  // The launcher's own environment is left as it was.
  assert.deepEqual(given, { PATH: '/usr/bin', YAK_ANALYZER_LAUNCHER: 'stale' });
});

test('reads the C library only on Linux', () => {
  if (process.platform === 'linux') {
    assert.ok(['glibc', 'musl'].includes(linuxLibc()));
>>>>>>> replace
```

A new test target starts the binary through the launcher as npm installs it and stops the launcher,
starts it directly with its standard input open, kills an audit while it reads a base, and runs the long
project:

```rust path=tests/life.test.rs
//! The life of a run: the binary ends soon after the npm launcher that started it, a killed audit
//! leaves nothing behind, and no project's size or depth runs it out of stack (dead-code spec LIFE-1
//! to LIFE-3).

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{BufRead as _, BufReader, Read as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use serde_json::Value;

/// The built binary.
const BINARY: &str = env!("CARGO_BIN_EXE_yak-analyzer");

/// How many files the long chain, the long cycle and the long re-export chain each hold (dead-code spec
/// LIFE-3).
const LENGTH: usize = 10_000;

/// How deep the deeply nested file nests its expressions and its blocks (dead-code spec LIFE-3).
const DEPTH: usize = 1_000;

/// A new, empty folder of one test's own, under cargo's folder for the files tests make.
fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);

    // The folder is left over from an earlier run, or is not there: either way it is made new.
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the test's folder can be made");

    folder
}

/// Runs git in a folder, shut off from the machine's and the user's own settings. It must succeed.
fn git(root: &Path, args: &[&str]) {
    let none = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-git-settings");

    if !none.exists() {
        fs::write(&none, "").expect("an empty settings file can be written");
    }

    let ran = Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(["-c", "maintenance.auto=false", "-c", "gc.auto=0"])
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_GLOBAL", &none)
        .env("GIT_CONFIG_SYSTEM", &none)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("git starts");

    assert!(
        ran.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&ran.stderr)
    );
}

/// The project too long and too deep for a small stack, made once for the tests that share it and
/// committed, so it is its own base: a chain of files, each of which passes on everything the next one
/// exports with `export *`, whose last file imports the first. It is at once an import chain, a cycle
/// and a re-export chain of `LENGTH` files. A file beside it nests `DEPTH` levels deep in parentheses
/// and in blocks (dead-code spec LIFE-3). It is made by the tests, and never committed here.
fn long_project() -> &'static Path {
    static PROJECT: OnceLock<PathBuf> = OnceLock::new();

    PROJECT.get_or_init(|| {
        let root = folder("life-long-project");
        let chain = root.join("src").join("chain");

        fs::create_dir_all(&chain).expect("the folder can be made");
        git(&root, &["init", "--quiet", "--initial-branch=main"]);
        fs::write(
            root.join("package.json"),
            r#"{ "name": "long", "main": "./src/index.ts" }"#,
        )
        .expect("the file can be written");
        fs::write(
            root.join("src").join("index.ts"),
            "import { value } from './chain/f0';\nimport { deep } from './deep';\n\
             export const total = value + deep;\n",
        )
        .expect("the file can be written");

        for place in 0..LENGTH {
            let content = if place + 1 < LENGTH {
                format!("export * from './f{}';\n", place + 1)
            } else {
                "import './f0';\nexport const value = 1;\n".to_owned()
            };

            fs::write(chain.join(format!("f{place}.ts")), content)
                .expect("the file can be written");
        }

        let mut deep = String::from("export const deep = ");

        deep.push_str(&"(".repeat(DEPTH));
        deep.push('1');
        deep.push_str(&")".repeat(DEPTH));
        deep.push_str(";\n");
        deep.push_str(&"if (deep) {\n".repeat(DEPTH));
        deep.push_str(&"}\n".repeat(DEPTH));
        fs::write(root.join("src").join("deep.ts"), deep).expect("the file can be written");

        git(&root, &["add", "--all"]);
        git(
            &root,
            &["commit", "--quiet", "--message", "the long project"],
        );

        root
    })
}

/// Runs a command of the binary in a folder and returns how it ended.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BINARY)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .expect("the binary starts")
}

/// The document a run printed, which must have succeeded and written no line.
fn printed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).expect("the binary prints JSON")
}

#[test]
fn long_chains_a_long_cycle_and_deep_nesting_do_not_run_the_analyzer_out_of_stack() {
    let root = long_project();

    // `graph` and `audit` read it whole, and `audit` with a base reads it twice, each on a thread of
    // its own (dead-code spec LIFE-3, INH-4).
    printed(&run(root, &["graph"]));

    for options in [&[][..], &["--base", "main"][..]] {
        let args: Vec<&str> = std::iter::once("audit")
            .chain(options.iter().copied())
            .collect();
        let report = printed(&run(root, &args));
        let results = report["runs"][0]["results"]
            .as_array()
            .expect("a list of results");

        // The one finding is the cycle, all its files in one set; the chain and the re-exports bring
        // `value` to the entry point, so nothing else is unused.
        assert_eq!(results.len(), 1, "{options:?}");
        assert_eq!(results[0]["ruleId"], "circular-dependency");
        assert_eq!(
            results[0]["relatedLocations"].as_array().map(Vec::len),
            Some(LENGTH - 1)
        );
        assert!(
            results[0]["message"]["text"]
                .as_str()
                .expect("a message")
                .ends_with(&format!("→ {} more → `src/chain/f0.ts`.", LENGTH - 10)),
            "{}",
            results[0]["message"]["text"]
        );
    }
}

/// The name of the platform package's target on this machine, as Node names it.
fn target() -> String {
    let system = match env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        other => other,
    };
    let processor = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };

    format!("{system}-{processor}")
}

/// The launcher and the binary laid out in a folder as npm installs them: the main package, and the
/// platform package beside it. Returns the launcher's script.
fn installed(home: &Path) -> PathBuf {
    let scope = home.join("node_modules").join("@dabay6");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("npm")
        .join("yak-analyzer");
    let main = scope.join("yak-analyzer");

    for part in ["bin/yak-analyzer.js", "lib/launch.js", "package.json"] {
        let to = main.join(part);

        fs::create_dir_all(to.parent().expect("a file has a folder"))
            .expect("the folder can be made");
        fs::copy(source.join(part), &to).expect("the launcher's file can be copied");
    }

    let platform = scope.join(format!("yak-analyzer-{}", target()));
    let binary = if cfg!(windows) {
        "yak-analyzer.exe"
    } else {
        "yak-analyzer"
    };

    fs::create_dir_all(&platform).expect("the folder can be made");
    fs::write(
        platform.join("package.json"),
        format!(
            r#"{{ "name": "@dabay6/yak-analyzer-{}", "version": "0.0.0" }}"#,
            target()
        ),
    )
    .expect("the file can be written");
    fs::copy(BINARY, platform.join(binary)).expect("the binary can be copied");

    main.join("bin").join("yak-analyzer.js")
}

/// A hand-off file that nobody ever writes, so that a run waits on it for as long as it runs: a named
/// pipe nobody writes to. On Windows a Node process holds the pipe open; it is returned so the test can
/// end it.
fn never_written(home: &Path) -> (String, Option<Child>) {
    if cfg!(windows) {
        let name = format!(r"\\.\pipe\yak-analyzer-life-{}", std::process::id());
        let script = format!(
            "require('node:net').createServer(() => {{}}).listen({}, () => console.log('listening'));",
            serde_json::to_string(&name).expect("a name is JSON")
        );
        let mut server = Command::new("node")
            .args(["-e", &script])
            .stdout(Stdio::piped())
            .spawn()
            .expect("node starts");
        let mut line = String::new();

        BufReader::new(server.stdout.take().expect("the server's output"))
            .read_line(&mut line)
            .expect("the server says it listens");
        assert_eq!(line.trim(), "listening");

        (name, Some(server))
    } else {
        let fifo = home.join("hand-off.json");
        let made = Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("mkfifo starts");

        assert!(made.success());

        (fifo.display().to_string(), None)
    }
}

#[test]
fn the_binary_ends_soon_after_the_launcher_that_started_it_and_writes_nothing() {
    let home = folder("life-launcher");
    let launcher = installed(&home);
    let (hand_off, server) = never_written(&home);
    let mut node = Command::new("node")
        .arg(&launcher)
        .args(["audit", "--hand-off", &hand_off])
        .current_dir(&home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the launcher starts");

    // The launcher starts the binary, whose audit waits on the hand-off file.
    thread::sleep(Duration::from_secs(2));
    assert!(
        node.try_wait()
            .expect("the launcher can be asked")
            .is_none(),
        "the audit is still running"
    );

    // The binary holds the launcher's output and its errors open until it ends, so both close when it
    // does.
    let mut output = node.stdout.take().expect("the launcher's output");
    let mut errors = node.stderr.take().expect("the launcher's errors");
    let (sender, closed) = mpsc::channel();

    thread::spawn(move || {
        let mut written = Vec::new();
        let _ = output.read_to_end(&mut written);
        let _ = errors.read_to_end(&mut written);
        let _ = sender.send(written);
    });

    node.kill().expect("the launcher can be stopped");
    node.wait().expect("the launcher ends");

    let stopped = Instant::now();
    let written = closed
        .recv_timeout(Duration::from_secs(10))
        .expect("the binary ends after the launcher");

    // It ends within five seconds, and writes nothing (dead-code spec LIFE-1).
    assert!(
        stopped.elapsed() < Duration::from_secs(5),
        "{:?}",
        stopped.elapsed()
    );
    assert_eq!(String::from_utf8_lossy(&written), "");

    if let Some(mut server) = server {
        let _ = server.kill();
        let _ = server.wait();
    }
}

#[test]
fn a_binary_started_directly_does_not_wait_on_its_standard_input() {
    let root = folder("life-direct");

    git(&root, &["init", "--quiet"]);

    // Its standard input open and never written to, the binary still runs to its end (dead-code spec
    // LIFE-1).
    let started = Command::new(BINARY)
        .arg("audit")
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");
    let output = started.wait_with_output().expect("the binary ends");

    assert_eq!(printed(&output)["version"], "2.1.0");
    assert_eq!(output.stdout, run(&root, &["audit"]).stdout);
}

/// Every path under a folder, `.git` included, with the size of each file and when it was last
/// changed. A folder's own time is left out: the system may record a change made in it a little
/// later, and a change shows in its files anyway.
fn snapshot(folder: &Path) -> BTreeMap<PathBuf, Option<(u64, Option<SystemTime>)>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![folder.to_path_buf()];

    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).expect("the folder can be read") {
            let entry = entry.expect("an entry can be read");
            let facts = entry.metadata().expect("an entry can be looked at");
            let path = entry.path();

            if facts.is_dir() {
                found.insert(path.clone(), None);
                waiting.push(path);
            } else {
                found.insert(path, Some((facts.len(), facts.modified().ok())));
            }
        }
    }

    found
}

/// A command that prints a line once the binary with the process id given has started git's batch,
/// which reads the base's files, and prints nothing before.
fn watching(binary: u32) -> Command {
    if cfg!(windows) {
        let mut watcher = Command::new("powershell");

        watcher.args([
            "-NoProfile",
            "-Command",
            &format!(
                "while (-not (Get-CimInstance Win32_Process -Filter 'ParentProcessId={binary}' | \
                 Where-Object {{ $_.CommandLine -like '*cat-file*' }})) {{ }}; 'reading'"
            ),
        ]);

        watcher
    } else {
        // The binary leads a process group of its own, which every git it starts joins.
        let mut watcher = Command::new("sh");

        watcher.args([
            "-c",
            &format!(
                "while ! ps -A -o pgid=,args= | grep -q '^ *{binary} .*cat-file'; do :; done; echo reading"
            ),
        ]);

        watcher
    }
}

/// How many git processes the binary with the process id given started, and their own, are still
/// running.
fn gits_running(binary: u32) -> usize {
    let output = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "$all = Get-CimInstance Win32_Process; $ids = @({binary}); $found = 0; \
                     do {{ $more = @($all | Where-Object {{ $ids -contains $_.ParentProcessId -and \
                     $ids -notcontains $_.ProcessId }}); $ids += $more.ProcessId; \
                     $found += @($more | Where-Object {{ $_.Name -like 'git*' }}).Count }} \
                     while ($more.Count -gt 0); $found"
                ),
            ])
            .output()
    } else {
        Command::new("sh")
            .args([
                "-c",
                &format!("ps -A -o pgid=,comm= | grep -c '^ *{binary} .*git' || true"),
            ])
            .output()
    }
    .expect("the process list can be read");

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("a count")
}

/// Starts an audit of the long project with its own commit as the base, leading a process group of its
/// own where there are process groups.
fn audit_with_base(root: &Path) -> Child {
    let mut binary = Command::new(BINARY);

    binary
        .args(["audit", "--base", "main"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut binary, 0);

    binary.spawn().expect("the binary starts")
}

#[test]
fn an_audit_killed_while_it_reads_a_base_leaves_nothing_changed_and_no_git_running() {
    let root = long_project();
    let before = snapshot(root);
    let mut binary = audit_with_base(root);
    let id = binary.id();
    let mut watcher = watching(id)
        .stdout(Stdio::piped())
        .spawn()
        .expect("the watcher starts");
    let said = watcher.stdout.take().expect("the watcher's output");
    let (sender, reading) = mpsc::channel();

    thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(said).read_line(&mut line);
        let _ = sender.send(line);
    });

    // It is killed as soon as git's batch is reading the base's files.
    loop {
        if reading.try_recv().is_ok() {
            break;
        }

        if binary
            .try_wait()
            .expect("the binary can be asked")
            .is_some()
        {
            let _ = watcher.kill();

            panic!("the audit ended before the test saw it read the base");
        }

        thread::sleep(Duration::from_millis(5));
    }

    let _ = watcher.wait();
    binary.kill().expect("the binary can be killed");

    let ended = binary.wait_with_output().expect("the binary ends");

    assert!(!ended.status.success());
    assert!(ended.stdout.is_empty());

    // Within thirty seconds no git it started is still running, and nothing under the root, `.git`
    // included, has changed: no lock is left (dead-code spec LIFE-2).
    let killed = Instant::now();

    while gits_running(id) > 0 {
        assert!(
            killed.elapsed() < Duration::from_secs(30),
            "a git the audit started is still running"
        );
        thread::sleep(Duration::from_millis(200));
    }

    assert_eq!(snapshot(root), before);
}
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
node --test npm/yak-analyzer/test/launcher.test.mjs && cargo test --release --locked --test life
```

Expected: FAIL: the launcher's tests do not load, since `launch.js` exports no `binaryEnvironment`, and
the `life` target does not run (decision 16).

- [ ] **Step 3: Write the code**

The launcher hands the binary its process id:

```js edit path=npm/yak-analyzer/lib/launch.js
<<<<<<< find
/** The exit code when the launcher itself cannot run the binary. */
const FAILED = 1;

/**
 * The targets as a person reads them: the Linux ones are for glibc.
 *
=======
/** The exit code when the launcher itself cannot run the binary. */
const FAILED = 1;

/** The variable the binary finds the launcher's process id in, so it ends soon after the launcher does. */
export const LAUNCHER_VARIABLE = 'YAK_ANALYZER_LAUNCHER';

/**
 * The binary's environment: the launcher's own, with the launcher's process id added (dead-code spec LIFE-1). The
 * binary watches that process and ends when it is gone, as when yak stops the launcher.
 *
 * @param {Readonly<Record<string, string | undefined>>} environment - The launcher's environment.
 * @param {number} id - The launcher's process id.
 * @returns {Record<string, string | undefined>} The environment to start the binary with.
 */
export function binaryEnvironment(environment, id) {
  return { ...environment, [LAUNCHER_VARIABLE]: String(id) };
}

/**
 * The targets as a person reads them: the Linux ones are for glibc.
 *
>>>>>>> replace
```

```js path=npm/yak-analyzer/bin/yak-analyzer.js
#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

import { binaryEnvironment, launch, linuxLibc } from '#yak-analyzer/launch.js';

/** Resolves a package from where this package is installed, which is where its platform package is too. */
const require = createRequire(import.meta.url);

process.exitCode = launch({
  arch: process.arch,
  args: process.argv.slice(2),
  libc: linuxLibc(),
  platform: process.platform,
  resolve: (id) => {
    return require.resolve(id);
  },
  run: (file, args) => {
    return spawnSync(file, args, {
      env: binaryEnvironment(process.env, process.pid),
      stdio: 'inherit',
      windowsHide: true,
    });
  },
  write: (line) => {
    process.stderr.write(line);
  },
});
```

The binary watches it, and every reading runs on a thread with a large stack:

```rust path=src/launcher.rs
//! The npm launcher that started the binary: when it ends, as when yak stops it, the binary ends soon
//! after, writing nothing (dead-code spec LIFE-1).
//!
//! The launcher names its own process in a variable of the binary's environment. On macOS and Linux
//! the binary watches it: the launcher is the binary's parent until it ends, when the system gives the
//! binary another. On Windows nothing needs watching: Node starts every child in a job that the system
//! ends with Node. A binary started without the variable is stopped directly, and nothing here changes
//! how it runs.

/// The variable the launcher names its process id in.
#[cfg(unix)]
const VARIABLE: &str = "YAK_ANALYZER_LAUNCHER";

/// How often, in milliseconds, the binary looks whether the launcher is still its parent.
#[cfg(unix)]
const EVERY: u64 = 200;

/// Watches the launcher that the environment names, where it names one, from a thread of its own, and
/// ends the process when the launcher is gone.
#[cfg(unix)]
pub fn watch() {
    use std::os::unix::process::parent_id;
    use std::time::Duration;
    use std::{env, process, thread};

    use crate::stop;

    let Some(launcher) = env::var(VARIABLE)
        .ok()
        .and_then(|id| id.parse::<u32>().ok())
    else {
        return;
    };

    // The thread lives as long as the process; it is never joined.
    let _ = thread::Builder::new()
        .name("launcher".to_owned())
        .spawn(move || {
            while parent_id() == launcher {
                thread::sleep(Duration::from_millis(EVERY));
            }

            process::exit(i32::from(stop::STOPPED));
        });
}

/// Nothing to watch: Node starts the binary in a job that the system ends with Node.
#[cfg(not(unix))]
pub const fn watch() {}
```

```rust edit path=src/main.rs
<<<<<<< find
mod hand_off;
mod imports;
mod inherit;
mod marks;
mod notice;
mod packages;
=======
mod hand_off;
mod imports;
mod inherit;
mod launcher;
mod marks;
mod notice;
mod packages;
>>>>>>> replace
<<<<<<< find
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;
use stop::Stop;
=======
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;
use std::thread;

use cli::Command;
use stop::Stop;
>>>>>>> replace
<<<<<<< find
/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
=======
/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

/// The stack of each thread that reads a project: far more than the deepest nesting of a file or the
/// longest chain of files needs, so none runs out of it, on Windows' small main thread least of all
/// (dead-code spec LIFE-3). It is reserved, and the system gives it memory only as it is used.
pub const STACK: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
>>>>>>> replace
<<<<<<< find
        );
    }));

    panic::catch_unwind(run).unwrap_or_else(|_| ExitCode::from(stop::FAULT))
}

/// Does what the command line asks.
=======
        );
    }));

    // Started by the npm launcher, the binary ends soon after the launcher does (dead-code spec LIFE-1).
    launcher::watch();

    // The work runs on a thread with a stack of its own size; a fault there ends that thread, which the
    // join reports.
    thread::Builder::new()
        .stack_size(STACK)
        .spawn(run)
        .map_or(ExitCode::from(stop::FAULT), |work| {
            work.join().unwrap_or_else(|_| ExitCode::from(stop::FAULT))
        })
}

/// Does what the command line asks.
>>>>>>> replace
```

```rust edit path=src/audit.rs
<<<<<<< find

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::files;
use crate::findings::{self, Finding, Location, Rule};
use crate::git;
=======

use serde_json::{Value, json};

use crate::files;
use crate::findings::{self, Finding, Location, Rule};
use crate::git;
>>>>>>> replace
<<<<<<< find
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::tree::Tree;

/// The key of `partialFingerprints` that holds a finding's fingerprint, with the scheme's version in
/// it, so that a later scheme can sit beside it (spec REPORT-3).
=======
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::tree::Tree;
use crate::{INTERFACE_VERSION, STACK};

/// The key of `partialFingerprints` that holds a finding's fingerprint, with the scheme's version in
/// it, so that a later scheme can sit beside it (spec REPORT-3).
>>>>>>> replace
<<<<<<< find
    let (base_tree, base_listing) = Tree::base(root.clone(), &commit, &mut base_notices)?;
    let working_tree = Tree::working(root);

    // The two sides are read at once (dead-code spec INH-4).
    let (working, base) = thread::scope(|scope| {
        let base = scope.spawn(|| {
            let reading = reading::read(handed.as_ref(), &base_tree, base_listing, base_notices)?;
            let renamed = git::renamed(base_tree.root(), &commit)?;

            Ok::<_, Stop>(Base {
                commit: commit.clone(),
                reading,
                renames: inherit::renames(&renamed.output),
                renames_cut_short: renamed
                    .warning
                    .is_some_and(|warning| warning.contains("rename")),
            })
        });
        let working = reading::read(handed.as_ref(), &working_tree, listing, notices);
        let base = base
            .join()
=======
    let (base_tree, base_listing) = Tree::base(root.clone(), &commit, &mut base_notices)?;
    let working_tree = Tree::working(root);

    // The two sides are read at once, each on a thread with the stack every reading has (dead-code spec
    // INH-4, LIFE-3).
    let (working, base) = thread::scope(|scope| {
        let base = thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(scope, || {
                let reading =
                    reading::read(handed.as_ref(), &base_tree, base_listing, base_notices)?;
                let renamed = git::renamed(base_tree.root(), &commit)?;

                Ok::<_, Stop>(Base {
                    commit: commit.clone(),
                    reading,
                    renames: inherit::renames(&renamed.output),
                    renames_cut_short: renamed
                        .warning
                        .is_some_and(|warning| warning.contains("rename")),
                })
            })
            .map_err(|error| {
                Stop::new(format!(
                    "the base cannot be read on a thread of its own ({error})"
                ))
            })?;
        let working = reading::read(handed.as_ref(), &working_tree, listing, notices);
        let base = base
            .join()
>>>>>>> replace
```

The new test target:

```toml edit path=Cargo.toml
<<<<<<< find
name = "audit"
path = "tests/audit.test.rs"

[dependencies]
# Parse scripts into syntax trees, walk them, and tie each reference to the binding it names
# (`oxc_semantic`, spec USE-1); released together, so pinned together (spec AN-2, DEP-3).
=======
name = "audit"
path = "tests/audit.test.rs"

[[test]]
name = "life"
path = "tests/life.test.rs"

[dependencies]
# Parse scripts into syntax trees, walk them, and tie each reference to the binding it names
# (`oxc_semantic`, spec USE-1); released together, so pinned together (spec AN-2, DEP-3).
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
node --test npm/yak-analyzer/test/launcher.test.mjs && cargo test --release --locked --test life
```

Expected: PASS: 12 launcher tests, then 4 tests in `life`, which take a few minutes the first time the
long project is written.

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
yak stack format
git add src/launcher.rs src/main.rs src/audit.rs Cargo.toml npm/yak-analyzer/lib/launch.js npm/yak-analyzer/bin/yak-analyzer.js npm/yak-analyzer/test/launcher.test.mjs tests/life.test.rs
git commit -m "feat(audit): end with the launcher and keep deep projects off a small stack"
```

Expected: one commit on `feat/dead-code-5-base`.
