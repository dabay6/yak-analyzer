# Reading a project, part 1: the two trials, the commands and the hand-off file: implementation plan

**Goal:** The two things the spec leaves to a trial are tried and their results are on the record, and
the binary has its two new commands: `interface`, and a `graph` that reads the hand-off file, refuses
one it cannot use with one line, and checks every declaration in it.

**Architecture:** The binary stays one crate. Its command line, the one-line way of failing, the
hand-off reader and the declaration keys each get a module of their own under `src/`, and `graph` prints
one JSON document whose lists of packages and files are still empty. The tests start the built binary,
as the skeleton's tests do, and build the hand-off files they hand it in code, because a hand-off file's
`root` is an absolute path and so differs by machine.

**Spec:** docs/specs/2026-11-11-reading-a-project-design.md

**Implements:** RUN-1 to RUN-5, RUN-6a, RUN-7, RUN-8, HO-1 to HO-5, KEY-1, KEY-2, KEY-5, RES-9, OUT-1, FIX-5, DEP-1, DEP-2

This is the first of six plans for this item. Each is written after the one before it is carried out,
since the two trials in this one decide how the later ones are built:

1. this plan;
2. the file list and the packages (LIST, PACK, RUN-6, the rest of RUN-6a, KEY-3, KEY-4, FIX-6);
3. parsing scripts and what an import is (PARSE-1, PARSE-2, PARSE-6, IMP);
4. resolving imports, tsconfig files, and build output back to source (RES, CFG, MAP);
5. Vue components, MDX documents and HTML pages (PARSE-3 to PARSE-5, RES-7);
6. the check on the named repositories, and the repository's own text (LIVE, DOC-1, DEP-3).

## Global constraints

- Every crate is pinned to one exact version, written with `=`, and listed here with its reason (DEP-1).
- A run that cannot go on exits 1, prints nothing on standard output, and writes exactly one line on
  standard error; a command the binary does not have exits 2; a fault in the analyzer exits 3 (RUN-2,
  RUN-3).
- The stops are checked in RUN-4's order, so two faults always give the same line. This plan builds the
  first five: the file cannot be read or is not JSON; its format; its fields; a stack's interface
  version; a declaration.
- The interface version is 1, and the analyzer reads hand-off format 1 (RUN-7, HO-1).
- The output holds no absolute path and is the same bytes on every platform (RUN-8, OUT-1).
- The tests use made-up content only, and nothing committed names a machine or a private repository
  other than yakshaver.
- The trials are run in folders outside every repository, and nothing they write is committed.

## The decisions this plan makes

1. **Two crates are added.** `serde_json`, at `=1.0.154`, reads the hand-off file and writes what
   `graph` and `interface` print; hand-written JSON reading would be a second parser to keep right.
   `jsonschema`, at `=0.60.2` with its default features off, is a dependency of the tests alone: it
   checks the tests' hand-off files against the schema's copy (HO-1). It is the newest version that was
   at least two weeks old on 2026-11-12, since the newest of all had been published that day. Its
   default features would add a network client, which the tests do not need.
2. **No crate reads the command line.** There are two commands and one option, and the skeleton's
   `match` on the arguments still fits.
3. **`graph` prints its document indented, with its fields in name order.** A person reads it, and the
   fixtures of later plans compare it.
4. **A notice's kind is a short name in lower case with hyphens.** This plan adds two:
   `no-hand-off-file` and `detection-cut-short`.
5. **The hand-off reader keeps only what a part of the analyzer uses.** It checks every field of the
   format now. It keeps the stacks and whether the detection list was cut short; the root, the
   folders, `siblings` and `base` are kept by the plans that first use them, since an unused field
   fails the lint.
6. **The tests build their hand-off files in code,** from one usable document that each test changes,
   and write them under the folder cargo gives tests for their files. A committed fixture file could
   not hold a `root` that is absolute on both Windows and Linux.
7. **The schema's copy lies in `tests/hand-off/`,** with a note beside it that says where it came from,
   and a Node test checks the copy against the digest in the note.
8. **A path or a glob whose second character is `:` counts as absolute,** which is how a Windows drive
   is written. A relative path that names a file so is refused; none of the design spec's examples do.
9. **The trials come first.** Their results change nothing in this plan's code and decide the next
   plans', so they are carried out before the code tasks and recorded with a ruling each.

## Review focus

- **A fault in the analyzer itself** (exit 3) has no test, since no test can cause one. A reviewer reads
  `main` in `src/main.rs`: the hook that writes the one line, and the catch that sets the exit code.
- **The branch that says "update the stack's declarations"** in `src/hand_off.rs` cannot be reached
  while the interface version is 1, because the format refuses a version below 1 first. It is there
  for the day the version rises, and is read, not tested.
- **A hand-off file that the schema accepts and the analyzer refuses.** The analyzer is stricter on
  purpose in four places: `root` and `siblings` must be absolute, `detection.folders` must begin with
  `.` and stay inside the root, and `interfaceVersion` written as `1.0` is refused though JSON Schema
  counts it a whole number. yak writes none of these. `tests/hand-off-schema.test.rs` pins the files
  both accept and the files both refuse.
- **An absolute path means the running system's.** A hand-off file written on Windows and read on Linux
  is refused for its `root`. yak writes the file on the machine it runs the analyzer on.
- **A file's name that is not valid UTF-8** is shown in a line with the bytes that are not replaced, so
  the line names it only roughly.
- **Declaration keys are checked in name order within a stack,** not in the order they are written,
  because the JSON reader keeps an object's keys sorted. The order across stacks, and the pack's before
  the project's, is the file's. `tests/declarations.test.rs` pins both.
- **The trials are judged by reading their reports.** Nothing in CI repeats them; the fixtures of the
  later plans hold what they found.

### Task 1: the trial of the resolver library

**Implements:** RES-9

**Check:** the report `trial-resolver.md` in the work item's report folder lists each rule of the
library's half of RES-9 with what the trial saw, and a ruling on this plan says that the split stands,
or which rules move to the analyzer's own code, or that the design spec's AN-2 must be amended before
the next plan is written.

**Files:** none in this repository. One throwaway Rust program in a new folder outside every
repository, removed when the report is written.

**Interfaces:**
- Consumes: the spec's RES-3, RES-9, RES-10, CFG-1 to CFG-3, LIST-4.
- Produces: the report, and the ruling the fourth plan is written from.

- [ ] **Step 1: Write the throwaway program**

In a new folder outside every repository, a Rust program that depends on `oxc_resolver`, pinned to the
newest version that is at least two weeks old, read from crates.io on the day. It gives the library a
file system of its own: the library's `FileSystem` trait, implemented over a fixed list of paths and
contents held in memory. That file system counts every call it gets and never asks the disk.

- [ ] **Step 2: Try each rule against the in-memory list**

Each of these is one case, with the answer the spec asks for written beside it before the case is run:

- RES-3: `./a.js` where only `a.ts` is listed; `./a.ts` written out; `./a` with no extension; `./dir`
  with `dir/index.ts`; `./types` where only `types.d.ts` is listed; `./pair.js` and `./pair` where
  `pair.js` and `pair.d.ts` are both listed.
- RES-10: `./Name` where the list holds `name.ts`.
- LIST-4: a path that is on the real disk and not in the list, and a path that is in the list and not
  on the real disk.
- CFG-1 to CFG-3: an alias from `paths`; a specifier resolved from `baseUrl`; a config that extends a
  relative config and takes its `paths`; a `tsconfig.json` with no files of its own whose `references`
  name the config that holds the `paths`; a config that extends a package that is not in the list; a
  tsconfig with comments and trailing commas.

For each case the report says: given as the spec asks, given differently and how, or not given. It also
says how many times the library asked the file system, and that it asked nothing else, which is checked
by running the program from an empty folder.

- [ ] **Step 3: Write the report and record the ruling**

The report goes into the work item's report folder as `trial-resolver.md`: the version tried, each case
and its result, and what follows for RES-9's split.

```bash
yak work ruling --plan plan:2026-11-12-reading-1-trials-commands-hand-off --decision "<what the trial showed, and what follows for RES-9's split>" --why "<the cases that decide it>" --cost "<what it costs if the reading of the trial is wrong>"
```

Expected: yak records the ruling. When the trial leaves the library nothing but finding a file for a
path, or shows it cannot be used at all, the work stops here and the owner is told, since the design
spec is amended before any other code is written (RES-9).

### Task 2: the trial of git

**Implements:** RUN-6a, DEP-2

**Check:** the report `trial-git.md` in the work item's report folder names the git commands and
settings the analyzer will use, shows for each rule of RUN-6a what the trial saw on a file system that
ignores letter case, and names the oldest git version those commands work with; and a ruling on this
plan says whether letter case can count in ignore patterns without changing which tracked files git
lists. If it cannot, the owner is asked before the rule is given up.

**Files:** none in this repository. Throwaway git repositories in a new folder outside every
repository, removed when the report is written.

**Interfaces:**
- Consumes: the spec's LIST-2, LIST-3, LIST-5, RUN-6, RUN-6a, DEP-2.
- Produces: the report, and the ruling the second plan is written from.

- [ ] **Step 1: Make the throwaway repositories**

In a new folder outside every repository, on the owner's Windows machine, whose file system ignores
letter case: a repository with tracked files, untracked files, files ignored through `.gitignore` and
through `.git/info/exclude`, a nested repository, a submodule, a second worktree inside the first, a
tracked symbolic link, an ignore pattern whose letter case differs from the folder it names, and a
tracked file whose name on disk differs in case from the name git holds.

- [ ] **Step 2: Try the commands**

The candidates are git's own listing of tracked files and of untracked files it does not ignore, with
names ended by a zero byte, and its check of whether paths are ignored, fed many paths at once. Each is
run with optional locks off, with no pager, with the setting that makes letter case count, and with the
settings that would make git start another program turned off. The report records, for each:

- the exact command and settings;
- what it lists for each kind of file in the repository of step 1, and how a nested repository, a
  submodule, a worktree and a symbolic link show;
- whether the list of tracked files is the same with and without the setting for letter case;
- that no file under `.git` changed, by a digest of every file there before and after;
- that a repository setting naming a file-system monitor program does not start it;
- that the variables `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE`, set to another repository, change
  nothing once the analyzer's way of starting git clears them, and which other variables must be
  cleared;
- whether the check of ignored paths answers for a path that is not on disk, and what it does for a
  path inside a submodule;
- the git version that first had each option used, from git's release notes, and so the oldest version;
- the time each command takes on the largest checkout on the machine.

- [ ] **Step 3: Write the report and record the ruling**

The report goes into the work item's report folder as `trial-git.md`.

```bash
yak work ruling --plan plan:2026-11-12-reading-1-trials-commands-hand-off --decision "<the commands and settings chosen, the oldest git version, and whether letter case can count>" --why "<what the trial saw>" --cost "<what it costs if the reading of the trial is wrong>"
```

Expected: yak records the ruling. When letter case cannot count without changing which tracked files
git lists, the owner is asked before the second plan is written (RUN-6a).

### Task 3: the two commands

**Implements:** RUN-1, RUN-2, RUN-3, RUN-5, RUN-7, RUN-8, OUT-1, DEP-1

**Test:** `cargo test --release --locked --test cli`

**Files:**
- Create: `src/cli.rs`, `src/graph.rs`, `src/stop.rs`
- Modify: `Cargo.toml`, `Cargo.lock`, `src/main.rs`
- Test: `tests/cli.test.rs`

**Interfaces:**
- Consumes: the skeleton's binary, its version option and its tests.
- Produces: `cli::parse` and `cli::Command`; `stop::line`, `stop::REFUSED` and `stop::FAULT`;
  `graph::read`, which returns the document `graph` prints; the constants `INTERFACE_VERSION` and
  `HAND_OFF_FORMATS` in `src/main.rs`.

- [ ] **Step 1: Write the failing tests**

The skeleton's tests of how the binary refuses change with RUN-2, and tests of the two commands join
them.

```rust path=tests/cli.test.rs
//! Starts the built binary and checks its commands and how it refuses (spec RUN-1, RUN-2, RUN-3, RUN-7).

use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs the built binary with the arguments given.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .output()
        .expect("the binary starts")
}

#[test]
fn prints_its_version_on_standard_output_and_exits_zero() {
    let output = run(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("yak-analyzer {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn prints_its_interface_version_and_the_hand_off_formats_it_reads_as_one_line_of_json() {
    let output = run(&["interface"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"handOffFormats\":[1],\"interfaceVersion\":1}\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn graph_without_a_hand_off_file_prints_one_document_and_says_so_in_a_notice() {
    let output = run(&["graph"]);
    let document: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(document["handOff"], json!(false));
    assert_eq!(document["files"], json!([]));
    assert_eq!(document["packages"], json!([]));
    assert_eq!(document["notices"].as_array().map(Vec::len), Some(1));
    assert_eq!(document["notices"][0]["kind"], json!("no-hand-off-file"));
    assert_eq!(document["notices"][0]["file"], Value::Null);
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    let cases: [&[&str]; 6] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
        &["interface", "extra"],
        &["graph", "extra"],
        &["graph", "--base", "main"],
    ];

    for args in cases {
        let output = run(args);
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(error.lines().count(), 1, "{args:?}");
        assert!(error.starts_with("yak-analyzer: cannot do `"), "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(
            error.contains("--version, interface, and graph"),
            "{args:?}"
        );
    }
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version, interface, and graph"));
}

#[test]
fn keeps_a_refusal_on_one_line_when_an_argument_holds_a_line_break() {
    let output = run(&["first\nsecond"]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("first\\nsecond"));
}
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test cli
```

Expected: the build fails, because the tests use `serde_json`, which is not a dependency yet.

- [ ] **Step 3: Add the dependency**

```toml path=Cargo.toml
[package]
name = "yak-analyzer"
version = "0.1.0"
edition = "2024"
description = "A static analyzer for TypeScript and JavaScript, built for yakshaver"
license = "MIT"
repository = "https://github.com/dabay6/yak-analyzer"
homepage = "https://github.com/dabay6/yak-analyzer#readme"
keywords = ["static-analysis", "dead-code", "typescript", "javascript", "sarif"]
categories = ["command-line-utilities", "development-tools"]
publish = false
autotests = false

[[bin]]
name = "yak-analyzer"
path = "src/main.rs"

[[test]]
name = "cli"
path = "tests/cli.test.rs"

[dependencies]
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"
```

```toml path=Cargo.lock
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "itoa"
version = "1.0.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3bd1574e617a9a6c1f8f8181bd7e841e1f20e4277607ef28945765188d6284d4"

[[package]]
name = "memchr"
version = "2.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"

[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
dependencies = [
 "serde_core",
]

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "serde_json"
version = "1.0.154"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "syn"
version = "3.0.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ae9ccbea3a7b0b09443036490c8f34a8b1b760fd345e5c0dae0aac718faea79"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "unicode-ident"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563"

[[package]]
name = "yak-analyzer"
version = "0.1.0"
dependencies = [
 "serde_json",
]

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"
```

- [ ] **Step 4: Write the commands**

```rust path=src/stop.rs
//! How the binary speaks on standard error: one line, and an exit code (spec RUN-2, RUN-3).

use std::fmt::Write as _;

/// The exit code for a command or an option the binary does not have (spec RUN-2).
pub const REFUSED: u8 = 2;

/// The exit code of a fault in the analyzer itself (spec RUN-3).
pub const FAULT: u8 = 3;

/// The name every line on standard error begins with.
const NAME: &str = "yak-analyzer";

/// One line for standard error: the program's name, then the text with every line break and control
/// character escaped, so that one line stays one line whatever a file is named.
pub fn line(text: &str) -> String {
    let mut escaped = String::with_capacity(NAME.len() + 2 + text.len());

    escaped.push_str(NAME);
    escaped.push_str(": ");

    for character in text.chars() {
        if character.is_control() {
            // Writing into a String cannot fail.
            let _ = write!(escaped, "{}", character.escape_default());
        } else {
            escaped.push(character);
        }
    }

    escaped
}
```

```rust path=src/cli.rs
//! The command line: what the binary was asked to do (spec RUN-1, RUN-2).

use std::ffi::OsString;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The command that prints the interface version and the hand-off formats the analyzer reads.
const INTERFACE_COMMAND: &str = "interface";

/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, and graph";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the version.
    Version,
    /// Print the interface version and the hand-off formats.
    Interface,
    /// Read the project and print what was read.
    Graph,
}

/// Reads the arguments after the program's name. The error is the text of the refusal.
pub fn parse(args: &[OsString]) -> Result<Command, String> {
    let words: Vec<&str> = args.iter().map(|arg| arg.to_str().unwrap_or("")).collect();

    match words.as_slice() {
        [VERSION_OPTION] => Ok(Command::Version),
        [INTERFACE_COMMAND] => Ok(Command::Interface),
        [GRAPH_COMMAND] => Ok(Command::Graph),
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => {
            let given = args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            Err(format!("cannot do `{given}`; it answers {ACCEPTED}"))
        }
    }
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads nothing of the project: the lists of packages and files are empty until the parts that
//! fill them are built.

use serde_json::{Value, json};

/// Something a person should know about a run that went on (spec RUN-5).
struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
}

/// Reads the project and returns the document to print.
pub fn read() -> String {
    let notices = [Notice {
        kind: "no-hand-off-file",
        file: None,
        sentence: "No hand-off file was given, so the project is read from its own standard files, with no \
                   declarations."
            .to_owned(),
    }];
    let document = json!({
        "files": [],
        "handOff": false,
        "notices": notices.iter().map(notice).collect::<Vec<Value>>(),
        "packages": [],
    });

    format!("{document:#}")
}

/// One notice as it is printed.
fn notice(notice: &Notice) -> Value {
    json!({ "file": notice.file, "kind": notice.kind, "sentence": notice.sentence })
}
```

```rust path=src/main.rs
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project. It reports no findings yet.

mod cli;
mod graph;
mod stop;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;

/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
        eprintln!(
            "{}",
            stop::line(&format!(
                "a fault in the analyzer itself, not in the project: {fault}"
            ))
        );
    }));

    panic::catch_unwind(run).unwrap_or_else(|_| ExitCode::from(stop::FAULT))
}

/// Does what the command line asks.
fn run() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match cli::parse(&args) {
        Ok(Command::Version) => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        Ok(Command::Interface) => {
            println!(
                "{}",
                serde_json::json!({ "handOffFormats": HAND_OFF_FORMATS, "interfaceVersion": INTERFACE_VERSION })
            );

            ExitCode::SUCCESS
        }
        Ok(Command::Graph) => {
            println!("{}", graph::read());

            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

            ExitCode::from(stop::REFUSED)
        }
    }
}
```

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked --test cli
```

Expected: each exits 0, and the last reports 6 tests passed.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src tests/cli.test.rs
git commit -m "feat(cli): add the interface and graph commands"
```

Expected: one commit on the item's branch.

### Task 4: the hand-off file and its declarations

**Implements:** HO-2, HO-3, HO-4, HO-5, KEY-1, KEY-2, KEY-5, RUN-3, RUN-4, RUN-5, FIX-5

**Test:** `cargo test --release --locked --test hand-off --test declarations`

**Files:**
- Create: `src/hand_off.rs`, `src/declarations.rs`, `tests/support/mod.rs`
- Modify: `Cargo.toml`, `src/cli.rs`, `src/graph.rs`, `src/main.rs`, `src/stop.rs`
- Test: `tests/hand-off.test.rs`, `tests/declarations.test.rs`

**Interfaces:**
- Consumes: Task 3's `cli`, `graph` and `stop` modules, and its two constants.
- Produces: `graph --hand-off <file>`; `hand_off::read`, `hand_off::HandOff`, `hand_off::Stack` and
  `hand_off::leaves_its_folder`; `declarations::check`; `stop::Stop` and `stop::STOPPED`;
  `tests/support/mod.rs`, with `run`, `folder`, `document` and `written` for every later test that
  hands the binary a hand-off file.

- [ ] **Step 1: Write the failing tests**

```rust path=tests/hand-off.test.rs
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
    assert_eq!(
        printed,
        json!({ "files": [], "handOff": true, "notices": [], "packages": [] })
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
    assert_eq!(printed["notices"].as_array().map(Vec::len), Some(1));
    assert_eq!(printed["notices"][0]["kind"], json!("detection-cut-short"));
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
```

```rust path=tests/declarations.test.rs
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4).

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
        "configEntries": [{ "file": "runner.config.*", "property": ["test", "include"] }],
        "scriptCommands": { "any": "JSON", "is": ["accepted", 1, null] }
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
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test hand-off --test declarations
```

Expected: cargo refuses, because the manifest names no test called `hand-off` yet.

- [ ] **Step 3: Name the tests in the manifest, and write what they share**

```toml path=Cargo.toml
[package]
name = "yak-analyzer"
version = "0.1.0"
edition = "2024"
description = "A static analyzer for TypeScript and JavaScript, built for yakshaver"
license = "MIT"
repository = "https://github.com/dabay6/yak-analyzer"
homepage = "https://github.com/dabay6/yak-analyzer#readme"
keywords = ["static-analysis", "dead-code", "typescript", "javascript", "sarif"]
categories = ["command-line-utilities", "development-tools"]
publish = false
autotests = false

[[bin]]
name = "yak-analyzer"
path = "src/main.rs"

[[test]]
name = "cli"
path = "tests/cli.test.rs"

[[test]]
name = "hand-off"
path = "tests/hand-off.test.rs"

[[test]]
name = "declarations"
path = "tests/declarations.test.rs"

[dependencies]
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"
```

```rust path=tests/support/mod.rs
//! What the tests that hand the binary a hand-off file share.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs the built binary with the arguments given.
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .output()
        .expect("the binary starts")
}

/// A new, empty folder of one test's own, under cargo's folder for the files tests make.
pub fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);

    // The folder is left over from an earlier run, or is not there: either way it is made new.
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the test's folder can be made");

    folder
}

/// A hand-off file of format 1 that the analyzer can use, for a checkout in the folder given.
pub fn document(folder: &Path) -> Value {
    let place = folder.to_str().expect("the test's folder is valid UTF-8");

    json!({
        "formatVersion": 1,
        "root": place,
        "detection": { "folders": [".", "packages/app"], "truncated": false },
        "stacks": [
            {
                "name": "typescript",
                "interfaceVersion": 1,
                "pack": {
                    "data": ["**/fixtures/**"],
                    "cyclomaticLimit": 20,
                    "configEntries": [{ "file": "runner.config.*", "property": "include" }]
                },
                "project": { "generated": ["src/generated/**"], "internal": ["packages/core"] }
            },
            { "name": "bare", "pack": {}, "project": {} }
        ],
        "siblings": place
    })
}

/// Writes a document as the hand-off file in the folder, and returns the file's path as text.
pub fn written(folder: &Path, document: &Value) -> String {
    let file = folder.join("hand-off.json");

    fs::write(&file, document.to_string()).expect("the hand-off file can be written");

    file.to_str()
        .expect("the test's folder is valid UTF-8")
        .to_owned()
}
```

- [ ] **Step 4: Write the reader, the keys, and the option**

```rust path=src/stop.rs
//! How a run ends when it cannot go on: one line on standard error and an exit code (spec RUN-2, RUN-3).

use std::fmt::Write as _;

/// The exit code of a run that could not go on (spec RUN-3).
pub const STOPPED: u8 = 1;

/// The exit code for a command or an option the binary does not have (spec RUN-2).
pub const REFUSED: u8 = 2;

/// The exit code of a fault in the analyzer itself (spec RUN-3).
pub const FAULT: u8 = 3;

/// The name every line on standard error begins with.
const NAME: &str = "yak-analyzer";

/// A run that cannot go on, as the one line that says what failed and what fixes it.
#[derive(Debug)]
pub struct Stop(String);

impl Stop {
    /// A stop whose line is the text given, without the program's name.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The line for standard error.
    pub fn line(&self) -> String {
        line(&self.0)
    }
}

/// One line for standard error: the program's name, then the text with every line break and control
/// character escaped, so that one line stays one line whatever a file is named.
pub fn line(text: &str) -> String {
    let mut escaped = String::with_capacity(NAME.len() + 2 + text.len());

    escaped.push_str(NAME);
    escaped.push_str(": ");

    for character in text.chars() {
        if character.is_control() {
            // Writing into a String cannot fail.
            let _ = write!(escaped, "{}", character.escape_default());
        } else {
            escaped.push(character);
        }
    }

    escaped
}
```

```rust path=src/hand_off.rs
//! Reads the hand-off file yak writes for one checkout (spec HO-1 to HO-4).
//!
//! The file is checked in a fixed order, so that two faults in one file always give the same line: that
//! it can be read, its format version, the shape of each field, and then each stack's interface version
//! (spec RUN-4).

use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

use crate::stop::Stop;
use crate::{HAND_OFF_FORMATS, INTERFACE_VERSION};

/// The largest whole number the format allows, which is the largest JSON carries exactly.
const LARGEST_NUMBER: u64 = 9_007_199_254_740_991;

/// The lengths, in hexadecimal digits, a commit's full name can have.
const COMMIT_NAME_LENGTHS: [usize; 2] = [40, 64];

/// What the analyzer takes from a hand-off file of format 1. Every field of the format is checked when
/// the file is read; a field is kept here once a part of the analyzer uses it.
#[derive(Debug)]
pub struct HandOff {
    /// Whether yak's detection stopped at its limit, so the folders it lists are not the whole workspace.
    pub truncated: bool,
    /// The stacks that carry declarations, in the order yak applies them.
    pub stacks: Vec<Stack>,
}

/// One stack and the declarations it carries, the pack's kept apart from the project's.
#[derive(Debug)]
pub struct Stack {
    /// The stack's name.
    pub name: String,
    /// The interface version the stack's declarations were written for.
    pub interface_version: Option<u64>,
    /// The pack's declarations: each key with its value.
    pub pack: Map<String, Value>,
    /// The project's own declarations under this stack.
    pub project: Map<String, Value>,
}

/// Reads and checks the hand-off file at the path given.
pub fn read(path: &Path) -> Result<HandOff, Stop> {
    let shown = path.display();
    let bytes = fs::read(path).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} cannot be read ({error}); pass the path of the file yak wrote for this \
             checkout"
        ))
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let document: Value = serde_json::from_slice(text).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} is not JSON (line {}, column {}); yak writes this file, so write it again \
             with yak",
            error.line(),
            error.column()
        ))
    })?;
    let shape = |problem: String| {
        Stop::new(format!(
            "the hand-off file {shown} is not format 1: {problem}; yak writes this file, so write it again with \
             yak"
        ))
    };
    let fields = document
        .as_object()
        .ok_or_else(|| shape("the file must hold one object".to_owned()))?;

    format_version(fields, &shown.to_string()).map_err(|stopped| match stopped {
        Fault::Stop(stop) => stop,
        Fault::Shape(problem) => shape(problem),
    })?;

    // The fields are checked in the order the format's table lists them (spec HO-2).
    absolute_path(fields, "root").map_err(shape)?;
    folders(fields).map_err(shape)?;

    let truncated = truncated(fields).map_err(shape)?;
    let stacks = stacks(fields).map_err(shape)?;

    absolute_path(fields, "siblings").map_err(shape)?;
    base(fields).map_err(shape)?;

    let handed = HandOff { truncated, stacks };

    for stack in &handed.stacks {
        interface_version(stack, &shown.to_string())?;
    }

    Ok(handed)
}

/// What can be wrong with the format version: a format the analyzer does not read, or a shape.
enum Fault {
    /// The file states a format the analyzer does not read.
    Stop(Stop),
    /// The field is missing or is not a whole number.
    Shape(String),
}

/// Checks the format version, before any other field is looked at (spec HO-3).
fn format_version(fields: &Map<String, Value>, shown: &str) -> Result<(), Fault> {
    let stated = fields
        .get("formatVersion")
        .ok_or_else(|| Fault::Shape("`formatVersion` is missing".to_owned()))?
        .as_f64()
        .filter(|number| number.fract() == 0.0)
        .ok_or_else(|| Fault::Shape("`formatVersion` must be a whole number".to_owned()))?;
    let reads = HAND_OFF_FORMATS.map(|format| format.to_string()).join(", ");
    let oldest = f64::from(HAND_OFF_FORMATS[0]);
    let newest = f64::from(HAND_OFF_FORMATS[HAND_OFF_FORMATS.len() - 1]);

    if stated < oldest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, older than any this analyzer reads ({reads}); update \
             yak"
        ))));
    }

    if stated > newest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, newer than any this analyzer reads ({reads}); update \
             the analyzer's version pinned in the pack"
        ))));
    }

    Ok(())
}

/// Checks a field that must be text holding an absolute path.
fn absolute_path(fields: &Map<String, Value>, name: &str) -> Result<(), String> {
    let text = fields
        .get(name)
        .ok_or_else(|| format!("`{name}` is missing"))?
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("`{name}` must be text that is not empty"))?;

    if Path::new(text).is_absolute() {
        Ok(())
    } else {
        Err(format!("`{name}` must be an absolute path"))
    }
}

/// The object under `detection`, which allows two fields and no other.
fn detection(fields: &Map<String, Value>) -> Result<&Map<String, Value>, String> {
    let detection = fields
        .get("detection")
        .ok_or_else(|| "`detection` is missing".to_owned())?
        .as_object()
        .ok_or_else(|| "`detection` must be an object".to_owned())?;

    match detection
        .keys()
        .find(|key| !matches!(key.as_str(), "folders" | "truncated"))
    {
        Some(other) => Err(format!("`detection.{other}` is not a field of the format")),
        None => Ok(detection),
    }
}

/// Checks the folders yak's detection looked in: at least one, the first `.`, each inside the root.
fn folders(fields: &Map<String, Value>) -> Result<(), String> {
    let listed = detection(fields)?
        .get("folders")
        .ok_or_else(|| "`detection.folders` is missing".to_owned())?
        .as_array()
        .filter(|listed| !listed.is_empty())
        .ok_or_else(|| "`detection.folders` must be a list of at least one path".to_owned())?;

    for (index, folder) in listed.iter().enumerate() {
        let folder = folder
            .as_str()
            .filter(|folder| !folder.is_empty())
            .ok_or_else(|| {
                format!("`detection.folders[{index}]` must be text that is not empty")
            })?;

        if leaves_its_folder(folder) || folder.contains('\\') {
            return Err(format!(
                "`detection.folders[{index}]` must be a path inside the root, written with forward slashes"
            ));
        }
    }

    if listed[0] == "." {
        Ok(())
    } else {
        Err("`detection.folders` must begin with `.`, the root itself".to_owned())
    }
}

/// Whether yak's detection stopped at its limit.
fn truncated(fields: &Map<String, Value>) -> Result<bool, String> {
    detection(fields)?
        .get("truncated")
        .ok_or_else(|| "`detection.truncated` is missing".to_owned())?
        .as_bool()
        .ok_or_else(|| "`detection.truncated` must be true or false".to_owned())
}

/// The stacks, each of which allows four fields and no other.
fn stacks(fields: &Map<String, Value>) -> Result<Vec<Stack>, String> {
    let listed = fields
        .get("stacks")
        .ok_or_else(|| "`stacks` is missing".to_owned())?
        .as_array()
        .ok_or_else(|| "`stacks` must be a list".to_owned())?;
    let mut stacks = Vec::with_capacity(listed.len());

    for (index, stack) in listed.iter().enumerate() {
        let stack = stack
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}]` must be an object"))?;
        let name = stack
            .get("name")
            .ok_or_else(|| format!("`stacks[{index}].name` is missing"))?
            .as_str()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| format!("`stacks[{index}].name` must be text that is not empty"))?;
        let interface_version = match stack.get("interfaceVersion") {
            Some(version) => Some(
                version
                    .as_u64()
                    .filter(|version| (1..=LARGEST_NUMBER).contains(version))
                    .ok_or_else(|| {
                        format!(
                            "`stacks[{index}].interfaceVersion` must be a positive whole number"
                        )
                    })?,
            ),
            None => None,
        };
        let pack = stack
            .get("pack")
            .ok_or_else(|| format!("`stacks[{index}].pack` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].pack` must be an object"))?;
        let project = stack
            .get("project")
            .ok_or_else(|| format!("`stacks[{index}].project` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].project` must be an object"))?;

        if let Some((key, _)) = project.iter().find(|(_, value)| !is_project_value(value)) {
            return Err(format!(
                "`stacks[{index}].project.{key}` must be a list of text or a positive whole number"
            ));
        }

        if let Some(other) = stack.keys().find(|key| {
            !matches!(
                key.as_str(),
                "name" | "interfaceVersion" | "pack" | "project"
            )
        }) {
            return Err(format!(
                "`stacks[{index}].{other}` is not a field of the format"
            ));
        }

        stacks.push(Stack {
            name: name.to_owned(),
            interface_version,
            pack: pack.clone(),
            project: project.clone(),
        });
    }

    Ok(stacks)
}

/// Whether a value is one the format lets a project state: a list of text, or a positive whole number.
fn is_project_value(value: &Value) -> bool {
    match value {
        Value::Array(entries) => entries.iter().all(Value::is_string),
        other => other
            .as_u64()
            .is_some_and(|number| (1..=LARGEST_NUMBER).contains(&number)),
    }
}

/// Checks the base commit, where the file states one: a commit's full name in lower-case hexadecimal
/// digits.
fn base(fields: &Map<String, Value>) -> Result<(), String> {
    let Some(base) = fields.get("base") else {
        return Ok(());
    };

    base.as_str()
        .filter(|name| {
            COMMIT_NAME_LENGTHS.contains(&name.len())
                && name
                    .bytes()
                    .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
        })
        .map(|_| ())
        .ok_or_else(|| {
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits".to_owned()
        })
}

/// Checks that a stack's declarations were written for the interface version this analyzer speaks, and
/// that a stack which states no version holds no declarations (spec HO-4).
fn interface_version(stack: &Stack, shown: &str) -> Result<(), Stop> {
    let name = &stack.name;

    match stack.interface_version {
        Some(INTERFACE_VERSION) => Ok(()),
        Some(stated) => {
            let update = if stated > INTERFACE_VERSION {
                "the analyzer's version pinned in the pack"
            } else {
                "the stack's declarations"
            };

            Err(Stop::new(format!(
                "the stack `{name}` in {shown} states declarations for interface version {stated}, and this \
                 analyzer speaks version {INTERFACE_VERSION}; update {update}"
            )))
        }
        None if stack.pack.is_empty() && stack.project.is_empty() => Ok(()),
        None => Err(Stop::new(format!(
            "the stack `{name}` in {shown} holds declarations and states no interface version; state the \
             version they were written for"
        ))),
    }
}

/// Whether a path written with forward slashes is absolute or steps out of its folder with `..`.
pub fn leaves_its_folder(path: &str) -> bool {
    let has_drive = path.as_bytes().get(1) == Some(&b':');

    path.starts_with('/') || has_drive || path.split('/').any(|segment| segment == "..")
}
```

```rust path=src/declarations.rs
//! The declaration keys the analyzer knows, and the check that it can use every declaration a hand-off
//! file holds (spec KEY-1, KEY-2, KEY-5).
//!
//! The keys are drafts until the design spec's stage 2 ends. This part checks them; the parts that read a
//! project use them.

use serde_json::Value;

use crate::hand_off::{Stack, leaves_its_folder};
use crate::stop::Stop;

/// What kind of value a key takes.
#[derive(Clone, Copy)]
enum Kind {
    /// A list of globs.
    Globs,
    /// A list of paths.
    Paths,
    /// A positive whole number.
    Number,
    /// Any JSON, which only a pack may state.
    Opaque,
}

/// Every key the analyzer knows, with the kind of its value (spec KEY-1).
const KEYS: [(&str, Kind); 9] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("internal", Kind::Paths),
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("configEntries", Kind::Opaque),
    ("scriptCommands", Kind::Opaque),
];

/// Who states a declaration: the hand-off file keeps the two apart.
#[derive(Clone, Copy)]
enum Source {
    /// The pack that carries the stack.
    Pack,
    /// The project itself.
    Project,
}

impl Source {
    /// The word for the line.
    const fn word(self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::Project => "project",
        }
    }

    /// Where a person changes the declaration. The line names no `yak` command: those names are yak's.
    const fn fix(self) -> &'static str {
        match self {
            Self::Pack => "change it in the pack",
            Self::Project => "change it in the project's own declarations",
        }
    }
}

/// Checks every declaration of every stack, the stacks in the file's order and the pack's before the
/// project's. The first that cannot be used stops the run (spec KEY-2).
pub fn check(stacks: &[Stack]) -> Result<(), Stop> {
    for stack in stacks {
        for (source, declarations) in [
            (Source::Pack, &stack.pack),
            (Source::Project, &stack.project),
        ] {
            for (key, value) in declarations {
                if let Some(why) = problem(key, value, source) {
                    return Err(Stop::new(format!(
                        "cannot use the declaration `{key}` that the {} states for the stack `{}`: {why}; {}",
                        source.word(),
                        stack.name,
                        source.fix()
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Why a declaration cannot be used, or nothing when it can.
fn problem(key: &str, value: &Value, source: Source) -> Option<String> {
    let Some((_, kind)) = KEYS.iter().find(|(known, _)| *known == key) else {
        let known = KEYS.map(|(known, _)| known).join(", ");

        return Some(format!("the analyzer knows no such key, only {known}"));
    };

    match kind {
        Kind::Globs => match texts(value) {
            Some(globs) => globs.iter().find_map(|glob| glob_problem(glob)),
            None => Some("its value must be a list of globs".to_owned()),
        },
        Kind::Paths => match texts(value) {
            Some(paths) => paths.iter().find_map(|path| path_problem(path)),
            None => Some("its value must be a list of paths".to_owned()),
        },
        Kind::Number => match value.as_u64() {
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
        Kind::Opaque => match source {
            Source::Pack => None,
            Source::Project => Some("only a pack may state this key".to_owned()),
        },
    }
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
}

/// Why a path cannot be used, or nothing when it can.
fn path_problem(path: &str) -> Option<String> {
    if path.is_empty() {
        Some("a path in it is empty".to_owned())
    } else if leaves_its_folder(path) {
        Some(format!(
            "the path `{path}` is absolute or steps out of its folder with `..`"
        ))
    } else {
        None
    }
}

/// Why a glob cannot be used, or nothing when it can (spec KEY-5).
fn glob_problem(glob: &str) -> Option<String> {
    if glob.is_empty() {
        return Some("a glob in it is empty".to_owned());
    }

    if leaves_its_folder(glob) {
        return Some(format!(
            "the glob `{glob}` is absolute or steps out of its folder with `..`"
        ));
    }

    let mut braces = 0_usize;
    let mut in_set = false;
    let mut escaped = false;

    for character in glob.chars() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if in_set {
            in_set = character != ']';
        } else if character == '[' {
            in_set = true;
        } else if character == '{' {
            braces += 1;
        } else if character == '}' {
            braces = braces.saturating_sub(1);
        }
    }

    if in_set {
        Some(format!("the glob `{glob}` has a bracket left open"))
    } else if braces > 0 {
        Some(format!("the glob `{glob}` has a brace left open"))
    } else if escaped {
        Some(format!(
            "the glob `{glob}` ends in a backslash that escapes nothing"
        ))
    } else {
        None
    }
}
```

```rust path=src/cli.rs
//! The command line: what the binary was asked to do (spec RUN-1, RUN-2).

use std::ffi::OsString;
use std::path::PathBuf;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The command that prints the interface version and the hand-off formats the analyzer reads.
const INTERFACE_COMMAND: &str = "interface";

/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// The option of `graph` that names the hand-off file.
const HAND_OFF_OPTION: &str = "--hand-off";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, and graph [--hand-off <file>]";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the version.
    Version,
    /// Print the interface version and the hand-off formats.
    Interface,
    /// Read the project and print what was read, with a hand-off file or without one.
    Graph { hand_off: Option<PathBuf> },
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
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => {
            let given = args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            Err(format!("cannot do `{given}`; it answers {ACCEPTED}"))
        }
    }
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file and nothing of the project: the lists of packages and files are
//! empty until the parts that fill them are built.

use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::hand_off;
use crate::stop::Stop;

/// Something a person should know about a run that went on (spec RUN-5).
struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
}

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
    let mut notices = Vec::new();

    match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice {
                    kind: "detection-cut-short",
                    file: None,
                    sentence: "yak's detection stopped at its limit of folders, so the folders the hand-off \
                               file lists are not the whole workspace."
                        .to_owned(),
                });
            }
        }
        None => notices.push(Notice {
            kind: "no-hand-off-file",
            file: None,
            sentence:
                "No hand-off file was given, so the project is read from its own standard files, \
                       with no declarations."
                    .to_owned(),
        }),
    }

    // Notices about no file come first, then by file, by kind and by sentence (spec RUN-8).
    notices.sort_by(|a, b| (&a.file, a.kind, &a.sentence).cmp(&(&b.file, b.kind, &b.sentence)));

    let document = json!({
        "files": [],
        "handOff": hand_off_path.is_some(),
        "notices": notices.iter().map(notice).collect::<Vec<Value>>(),
        "packages": [],
    });

    Ok(format!("{document:#}"))
}

/// One notice as it is printed.
fn notice(notice: &Notice) -> Value {
    json!({ "file": notice.file, "kind": notice.kind, "sentence": notice.sentence })
}
```

```rust path=src/main.rs
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project. It reports no findings yet.

mod cli;
mod declarations;
mod graph;
mod hand_off;
mod stop;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;

/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
        eprintln!(
            "{}",
            stop::line(&format!(
                "a fault in the analyzer itself, not in the project: {fault}"
            ))
        );
    }));

    panic::catch_unwind(run).unwrap_or_else(|_| ExitCode::from(stop::FAULT))
}

/// Does what the command line asks.
fn run() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match cli::parse(&args) {
        Ok(Command::Version) => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        Ok(Command::Interface) => {
            println!(
                "{}",
                serde_json::json!({ "handOffFormats": HAND_OFF_FORMATS, "interfaceVersion": INTERFACE_VERSION })
            );

            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => match graph::read(hand_off.as_deref()) {
            Ok(document) => {
                println!("{document}");

                ExitCode::SUCCESS
            }
            Err(stopped) => {
                eprintln!("{}", stopped.line());

                ExitCode::from(stop::STOPPED)
            }
        },
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

            ExitCode::from(stop::REFUSED)
        }
    }
}
```

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0. The tests report 6 passed for `cli`, 5 for `declarations` and 10 for
`hand-off`.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src tests
git commit -m "feat(hand-off): read the hand-off file and check its declarations"
```

Expected: one commit on the item's branch.

### Task 5: the schema's copy

**Implements:** HO-1, DEP-1

**Test:** `cargo test --release --locked --test hand-off-schema && node --test scripts/hand-off-schema.test.mjs`

**Files:**
- Create: `tests/hand-off/hand-off-1.schema.json`, `tests/hand-off/README.md`
- Modify: `Cargo.toml`, `Cargo.lock`
- Test: `tests/hand-off-schema.test.rs`, `scripts/hand-off-schema.test.mjs`

**Interfaces:**
- Consumes: Task 4's `graph --hand-off` and `tests/support/mod.rs`; `ROOT` from `scripts/lib/run.mjs`;
  the schema in the yakshaver repository, at the commit the note names.
- Produces: the copy every later plan's hand-off files are checked against.

- [ ] **Step 1: Write the failing tests**

```rust path=tests/hand-off-schema.test.rs
//! Checks the hand-off files the tests use against the copy of yakshaver's schema, and that the analyzer
//! and the schema agree on files made wrong on purpose (spec HO-1).

mod support;

use serde_json::{Value, json};

use support::{document, folder, run, written};

/// The copy of the schema of hand-off format 1. `tests/hand-off/README.md` says where it came from.
const SCHEMA: &str = include_str!("hand-off/hand-off-1.schema.json");

/// Whether the schema accepts a document.
fn accepted(document: &Value) -> bool {
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema's copy is JSON");

    jsonschema::validator_for(&schema)
        .expect("the schema's copy is a JSON Schema")
        .is_valid(document)
}

#[test]
fn the_schema_accepts_every_hand_off_file_the_tests_use_and_so_does_the_analyzer() {
    let place = folder("schema-valid");
    let plain = document(&place);
    let mut with_base = plain.clone();
    let mut cut_short = plain.clone();
    let mut with_more = plain.clone();

    with_base["base"] = json!("0123456789abcdef0123456789abcdef01234567");
    cut_short["detection"]["truncated"] = json!(true);
    with_more["cache"] = json!({ "folder": "somewhere" });

    for (name, handed) in [
        ("plain", plain),
        ("with a base", with_base),
        ("cut short", cut_short),
        ("with a field the format does not know", with_more),
    ] {
        let output = run(&["graph", "--hand-off", &written(&place, &handed)]);

        assert!(accepted(&handed), "{name}");
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn the_schema_and_the_analyzer_both_refuse_a_file_made_wrong_on_purpose() {
    let place = folder("schema-invalid");
    let plain = document(&place);
    let mut no_root = plain.clone();
    let mut other_format = plain.clone();
    let mut more_in_detection = plain.clone();
    let mut no_folders = plain.clone();
    let mut more_in_a_stack = plain.clone();
    let mut odd_project_value = plain.clone();
    let mut short_base = plain.clone();

    no_root.as_object_mut().expect("an object").remove("root");
    other_format["formatVersion"] = json!(2);
    more_in_detection["detection"]["limit"] = json!(64);
    no_folders["detection"]["folders"] = json!([]);
    more_in_a_stack["stacks"][0]["tool"] = json!("analyzer");
    odd_project_value["stacks"][0]["project"]["data"] = json!({ "globs": [] });
    short_base["base"] = json!("abc123");

    for (name, handed) in [
        ("no root", no_root),
        ("another format", other_format),
        (
            "a field the format does not allow under detection",
            more_in_detection,
        ),
        ("no folders", no_folders),
        (
            "a field the format does not allow in a stack",
            more_in_a_stack,
        ),
        (
            "a project value that is neither a list nor a number",
            odd_project_value,
        ),
        ("a base that is not a commit's full name", short_base),
    ] {
        let output = run(&["graph", "--hand-off", &written(&place, &handed)]);

        assert!(!accepted(&handed), "{name}");
        assert_eq!(output.status.code(), Some(1), "{name}");
    }
}
```

```js path=scripts/hand-off-schema.test.mjs
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The folder that holds the schema's copy and the note on where it came from. */
const FOLDER = join(ROOT, 'tests', 'hand-off');

/** A text file's content with line feeds, whatever the checkout wrote. */
const text = (name) => readFileSync(join(FOLDER, name), 'utf8').replaceAll('\r\n', '\n');

test('the copy of the hand-off schema is the file the note beside it describes (spec HO-1)', () => {
  const note = text('README.md');
  const stated = /SHA-256 of the file, with line feeds:\*\* `(?<digest>[0-9a-f]{64})`/u.exec(note)?.groups?.digest;
  const digest = createHash('sha256').update(text('hand-off-1.schema.json')).digest('hex');

  assert.equal(stated, digest);
});

test('the note says where the copy came from: the path, the pull request and the commit', () => {
  const note = text('README.md');

  assert.ok(note.includes('`packages/core/src/stacks/hand-off/hand-off-1.schema.json`'));
  assert.ok(note.includes('dabay6/yakshaver#213'));
  assert.match(note, /\*\*Commit:\*\* `[0-9a-f]{40}`/u);
});

test('the copy is hand-off format 1, as a JSON Schema', () => {
  const schema = JSON.parse(text('hand-off-1.schema.json'));

  assert.equal(schema.properties.formatVersion.const, 1);
  assert.deepEqual(schema.required, ['detection', 'formatVersion', 'root', 'siblings', 'stacks']);
});
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test hand-off-schema && node --test scripts/hand-off-schema.test.mjs
```

Expected: cargo refuses, because the manifest names no test called `hand-off-schema` yet.

- [ ] **Step 3: Copy the schema, and write where it came from**

The copy is the file as git holds it at the commit the note names, byte for byte, with line feeds.

```json path=tests/hand-off/hand-off-1.schema.json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "base": {
      "description": "The commit the gate would compare the checkout's branch against, as a full object name. Absent when yak can work none out.",
      "type": "string",
      "pattern": "^(?:[0-9a-f]{40}|[0-9a-f]{64})$"
    },
    "detection": {
      "type": "object",
      "properties": {
        "folders": {
          "minItems": 1,
          "type": "array",
          "items": {
            "type": "string",
            "minLength": 1
          },
          "description": "The folders, each relative to `root` with forward slashes. The first is `.`, the root itself; the rest are the workspace roots."
        },
        "truncated": {
          "type": "boolean",
          "description": "True when detection stopped at its limit of folders, so `folders` is not the whole workspace."
        }
      },
      "required": [
        "folders",
        "truncated"
      ],
      "additionalProperties": false,
      "description": "The folders yak looked in for stacks: the checkout root and the workspace roots."
    },
    "formatVersion": {
      "type": "number",
      "const": 1,
      "description": "The hand-off format. It rises only when the format breaks."
    },
    "root": {
      "type": "string",
      "minLength": 1,
      "description": "The checkout's root: absolute, with symbolic links resolved, as the operating system spells it. It is the root to analyze, whatever directory the tool was started in."
    },
    "siblings": {
      "type": "string",
      "minLength": 1,
      "description": "The folder this file lies in, absolute. Every file in it whose name ends in `.json` is the hand-off file of one checkout of the same repository, for the same tool server. A reader finds a checkout's file by its `root`, never by its name: a file's name means nothing, and a file still being written has another ending."
    },
    "stacks": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "interfaceVersion": {
            "description": "The interface version the stack's declarations file says it was written for. Absent only when the stack has no declarations file for this tool server; `pack` and `project` are then empty.",
            "type": "integer",
            "exclusiveMinimum": 0,
            "maximum": 9007199254740991
          },
          "name": {
            "type": "string",
            "minLength": 1,
            "description": "The stack's name."
          },
          "pack": {
            "type": "object",
            "propertyNames": {
              "type": "string"
            },
            "additionalProperties": {
              "$ref": "#/$defs/__schema0"
            },
            "description": "The pack's declarations for this tool server: each key with its value, any JSON. yak passes them through and reads no key."
          },
          "project": {
            "type": "object",
            "propertyNames": {
              "type": "string"
            },
            "additionalProperties": {
              "anyOf": [
                {
                  "type": "array",
                  "items": {
                    "type": "string"
                  }
                },
                {
                  "type": "integer",
                  "exclusiveMinimum": 0,
                  "maximum": 9007199254740991
                }
              ]
            },
            "description": "The project's own declarations under this stack, kept apart from the pack's: each key with a list of globs or paths, or a positive whole number."
          }
        },
        "required": [
          "name",
          "pack",
          "project"
        ],
        "additionalProperties": false,
        "description": "One stack and the declarations it carries for this tool server."
      },
      "description": "The stacks that carry declarations for this tool server, in the stacks' order. The stack whose audit is running is always among them."
    }
  },
  "required": [
    "detection",
    "formatVersion",
    "root",
    "siblings",
    "stacks"
  ],
  "additionalProperties": {
    "$ref": "#/$defs/__schema0"
  },
  "description": "What yak hands a tool server for one checkout: where the checkout is, where yak looked for stacks, and the declarations in force, with the pack's and the project's kept apart. yak replaces the file whole each time it runs the tool. A reader ignores a top-level field it does not know.",
  "title": "yak hand-off file, format 1",
  "$defs": {
    "__schema0": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "type": "number"
        },
        {
          "type": "boolean"
        },
        {
          "type": "null"
        },
        {
          "type": "array",
          "items": {
            "$ref": "#/$defs/__schema0"
          }
        },
        {
          "type": "object",
          "propertyNames": {
            "type": "string"
          },
          "additionalProperties": {
            "$ref": "#/$defs/__schema0"
          }
        }
      ]
    }
  }
}
```

```markdown path=tests/hand-off/README.md
# The hand-off format's schema, copied

`hand-off-1.schema.json` is a copy of the JSON Schema of hand-off format 1, the file yak writes for
one checkout and hands to the analyzer. The original is in the yakshaver repository. The tests check
every hand-off file they use against this copy (spec HO-1).

- **Path there:** `packages/core/src/stacks/hand-off/hand-off-1.schema.json`
- **Pull request:** dabay6/yakshaver#213, not merged when the copy was made
- **Commit:** `0a47b4e134711667010446b046e65aee0bc31540`
- **SHA-256 of the file, with line feeds:** `1f0a37fe9c6b7e4c78fbf0de599d13a6f73b20405eb1f4bcd58c1cbae319b948`

When the schema changes in yakshaver, copy it again, write the new commit and digest here, and change
the tests that depend on it in the same commit. `scripts/hand-off-schema.test.mjs` fails when the copy
and the digest written here disagree.
```

- [ ] **Step 4: Add the validator for the tests, and name the test**

```toml path=Cargo.toml
[package]
name = "yak-analyzer"
version = "0.1.0"
edition = "2024"
description = "A static analyzer for TypeScript and JavaScript, built for yakshaver"
license = "MIT"
repository = "https://github.com/dabay6/yak-analyzer"
homepage = "https://github.com/dabay6/yak-analyzer#readme"
keywords = ["static-analysis", "dead-code", "typescript", "javascript", "sarif"]
categories = ["command-line-utilities", "development-tools"]
publish = false
autotests = false

[[bin]]
name = "yak-analyzer"
path = "src/main.rs"

[[test]]
name = "cli"
path = "tests/cli.test.rs"

[[test]]
name = "hand-off"
path = "tests/hand-off.test.rs"

[[test]]
name = "declarations"
path = "tests/declarations.test.rs"

[[test]]
name = "hand-off-schema"
path = "tests/hand-off-schema.test.rs"

[dependencies]
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"

[dev-dependencies]
# Checks the tests' hand-off files against the copy of yakshaver's schema. Never part of the binary.
jsonschema = { version = "=0.60.2", default-features = false }
```

```toml path=Cargo.lock
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "ahash"
version = "0.8.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
dependencies = [
 "cfg-if",
 "getrandom",
 "once_cell",
 "serde",
 "version_check",
 "zerocopy",
]

[[package]]
name = "aho-corasick"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c982642fa9e8606056828ee9a8505737230110bb1099153c79efe865c59d12ba"
dependencies = [
 "memchr",
]

[[package]]
name = "allocator-api2"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923"

[[package]]
name = "autocfg"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"

[[package]]
name = "bit-set"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3"
dependencies = [
 "bit-vec",
]

[[package]]
name = "bit-vec"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7"

[[package]]
name = "bitflags"
version = "2.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"

[[package]]
name = "borrow-or-share"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc0b364ead1874514c8c2855ab558056ebfeb775653e7ae45ff72f28f8f3166c"

[[package]]
name = "bumpalo"
version = "3.20.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649"

[[package]]
name = "bytecount"
version = "0.6.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175812e0be2bccb6abe50bb8d566126198344f707e304f45c648fd8f2cc0365e"

[[package]]
name = "cfg-if"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"

[[package]]
name = "data-encoding"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "email_address"
version = "0.2.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e079f19b08ca6239f47f8ba8509c11cf3ea30095831f7fed61441475edd8c449"
dependencies = [
 "serde",
]

[[package]]
name = "equivalent"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00d174d5400e5e8fd687ad1049e2f578285fa914201b1af7e8b112a4546bd826"

[[package]]
name = "fancy-regex"
version = "0.19.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d301f5bf187b3c295fce6468d3875037a0bccc5f6b151c63cac2f85babf21912"
dependencies = [
 "bit-set",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "fluent-uri"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bc74ac4d8359ae70623506d512209619e5cf8f347124910440dbc221714b328e"
dependencies = [
 "borrow-or-share",
 "ref-cast",
 "serde",
]

[[package]]
name = "foldhash"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77ce24cb58228fbb8aa041425bb1050850ac19177686ea6e0f41a70416f56fdb"

[[package]]
name = "fraction"
version = "0.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e246562084dde8ebbcc943b261c406ce4f68e5032ec28029a251a47d6a295500"
dependencies = [
 "num",
 "num-bigint",
]

[[package]]
name = "getrandom"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "r-efi",
 "wasip2",
 "wasm-bindgen",
]

[[package]]
name = "hashbrown"
version = "0.17.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a"
dependencies = [
 "allocator-api2",
 "equivalent",
 "foldhash",
]

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "itoa"
version = "1.0.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3bd1574e617a9a6c1f8f8181bd7e841e1f20e4277607ef28945765188d6284d4"

[[package]]
name = "js-sys"
version = "0.3.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8dc09d2154098bbf4ef54eab6b117a3c24cbbffe94046b16ee6d6d4510584d3a"
dependencies = [
 "cfg-if",
 "wasm-bindgen",
]

[[package]]
name = "jsonschema"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a42a11b47492e6a2cc76e9c7bcc8077776cb31dac444e0795d7f61bcc6441c08"
dependencies = [
 "ahash",
 "bytecount",
 "data-encoding",
 "email_address",
 "fancy-regex",
 "fraction",
 "getrandom",
 "itoa",
 "jsonschema-regex",
 "jsonschema-value",
 "num-cmp",
 "num-traits",
 "percent-encoding",
 "referencing",
 "regex",
 "serde",
 "serde_json",
 "strum",
 "unicode-general-category",
 "uuid-simd",
]

[[package]]
name = "jsonschema-regex"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "107aeb6ff14b8deb5610096a8ce5c554ec89806b521147eaeb3cae05b07edb7a"
dependencies = [
 "regex-syntax",
]

[[package]]
name = "jsonschema-value"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e281d6b177bb4834f2ffd6f92aa0443cffe29c996b251c8d94eb6f456186feba"
dependencies = [
 "ahash",
 "bytecount",
 "fraction",
 "getrandom",
 "memchr",
 "num-cmp",
 "num-traits",
 "serde_json",
 "zmij",
]

[[package]]
name = "libc"
version = "0.2.191"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "65cd87c9a6d60536580b316ad985726104730a5151c99845f2490428ef91240a"

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "memchr"
version = "2.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"

[[package]]
name = "micromap"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a86d3146ed3995b5913c414f6664344b9617457320782e64f0bb44afd49d74"

[[package]]
name = "num"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "35bd024e8b2ff75562e5f34e7f4905839deb4b22955ef5e73d2fea1b9813cb23"
dependencies = [
 "num-bigint",
 "num-complex",
 "num-integer",
 "num-iter",
 "num-rational",
 "num-traits",
]

[[package]]
name = "num-bigint"
version = "0.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-cmp"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63335b2e2c34fae2fb0aa2cecfd9f0832a1e24b3b32ecec612c3426d46dc8aaa"

[[package]]
name = "num-complex"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-integer"
version = "0.1.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7ce2d95d4b3734dc35aa2f45e1aa22cd416814592a4f9d9205e11affd5b8e10b"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-iter"
version = "0.1.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c92800bd69a1eac91786bcfe9da64a897eb72911b8dc3095decbd07429e8048b"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-rational"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824"
dependencies = [
 "num-bigint",
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-traits"
version = "0.2.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
dependencies = [
 "autocfg",
]

[[package]]
name = "once_cell"
version = "1.21.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"

[[package]]
name = "outref"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a80800c0488c3a21695ea981a54918fbb37abf04f4d0720c453632255e2ff0e"

[[package]]
name = "parking_lot"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
dependencies = [
 "lock_api",
 "parking_lot_core",
]

[[package]]
name = "parking_lot_core"
version = "0.9.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
dependencies = [
 "cfg-if",
 "libc",
 "redox_syscall",
 "smallvec",
 "windows-link",
]

[[package]]
name = "percent-encoding"
version = "2.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"

[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "r-efi"
version = "5.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"

[[package]]
name = "redox_syscall"
version = "0.5.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
dependencies = [
 "bitflags",
]

[[package]]
name = "ref-cast"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e440fb4e4b4147295338efb76001ab9e4efc0e5839df2c47fc5ac2381d365c3"
dependencies = [
 "ref-cast-impl",
]

[[package]]
name = "ref-cast-impl"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92ecd8964f8453721699a1ed72037b0db49ce2f5a5138486ee89bed6f67cdf3a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "referencing"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f42818036d3b37cda6bd4c494543158a36e431cb717015ef7c93acc6ba214d3b"
dependencies = [
 "ahash",
 "fluent-uri",
 "getrandom",
 "hashbrown",
 "itoa",
 "micromap",
 "parking_lot",
 "percent-encoding",
 "serde_json",
]

[[package]]
name = "regex"
version = "1.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "regex-automata"
version = "0.4.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad8553b9b26413251cbf30e620595c7a41b3887f03da04579c0e6b0d6a06b4b2"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-syntax",
]

[[package]]
name = "regex-syntax"
version = "0.8.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4"

[[package]]
name = "rustversion"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"

[[package]]
name = "scopeguard"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"

[[package]]
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
dependencies = [
 "serde_core",
 "serde_derive",
]

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "serde_json"
version = "1.0.154"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "smallvec"
version = "1.16.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "149309498a5f7334b12ddae0cad8aba81ce785fe91755068867e017cf98fee46"

[[package]]
name = "strum"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9628de9b8791db39ceda2b119bbe13134770b56c138ec1d3af810d045c04f9bd"
dependencies = [
 "strum_macros",
]

[[package]]
name = "strum_macros"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab85eea0270ee17587ed4156089e10b9e6880ee688791d45a905f5b1ca36f664"
dependencies = [
 "heck",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "syn"
version = "2.0.119"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "3.0.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ae9ccbea3a7b0b09443036490c8f34a8b1b760fd345e5c0dae0aac718faea79"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "unicode-general-category"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b993bddc193ae5bd0d623b49ec06ac3e9312875fdae725a975c51db1cc1677f"

[[package]]
name = "unicode-ident"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563"

[[package]]
name = "uuid-simd"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b082222b4f6619906941c17eb2297fff4c2fb96cb60164170522942a200bd8"
dependencies = [
 "outref",
 "vsimd",
]

[[package]]
name = "version_check"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"

[[package]]
name = "vsimd"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c3082ca00d5a5ef149bb8b555a72ae84c9c59f7250f013ac822ac2e49b19c64"

[[package]]
name = "wasip2"
version = "1.0.4+wasi-0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b67efb37e106e55ce722a510d6b5f9c17f083e5fc79afc2badeb12cc313d9487"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasm-bindgen"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0181c14ddd6c7b8b0b0132a5f1ea2f32be0d893cd3ea97b6e3b28886b8a3d1c"
dependencies = [
 "cfg-if",
 "rustversion",
 "wasm-bindgen-macro",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e931a02a39abdbf66966fc35902b5ed58344750a70885bed4ebb3d8eaf28ed9"
dependencies = [
 "quote",
 "wasm-bindgen-macro-support",
]

[[package]]
name = "wasm-bindgen-macro-support"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ca4b54280e455bd831e33a6b8e734baf432fe63fbc35c42ac14ce67a043a53a"
dependencies = [
 "bumpalo",
 "proc-macro2",
 "quote",
 "syn 3.0.10",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-shared"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b1f9c3798725016c6ee5f2f57869ec0831e8bbc0ab0310a86e787e97f41389e"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "wit-bindgen"
version = "0.57.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e"

[[package]]
name = "yak-analyzer"
version = "0.1.0"
dependencies = [
 "jsonschema",
 "serde_json",
]

[[package]]
name = "zerocopy"
version = "0.8.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5fe1f8f1b06191a00962174c61aa5005e0bb391a6d80d07e24d115c01a92ed8"
dependencies = [
 "zerocopy-derive",
]

[[package]]
name = "zerocopy-derive"
version = "0.8.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "863ad3ac83293fb4d740aedbfdc9240dd8d1a50c1099acd76ce80ce7c7230c7f"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"
```

- [ ] **Step 5: Run every gate**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo build --release --locked && node scripts/check-floor.mjs
```

Expected: each exits 0. The Rust tests report 6, 5, 10 and 2 passed, the Node tests 45 passed, and the
last line says what the binary asks of the system, which is no more than before.

- [ ] **Step 6: Commit, push, and open the pull request as a draft**

```bash
git add Cargo.toml Cargo.lock tests scripts/hand-off-schema.test.mjs
git commit -m "test(hand-off): check the tests' hand-off files against the schema's copy"
git push -u origin feat/reading-a-project
gh pr create --draft --base main --head feat/reading-a-project --title "feat: the interface and graph commands, and the hand-off file" --body "A draft, opened so that CI runs on all five targets. Not for merging until the owner has validated the plan."
gh pr checks --watch
```

Expected: the pull request's seven checks pass. Its description is written out in full, with the two
trials' results, when the plan is handed over for validation.
