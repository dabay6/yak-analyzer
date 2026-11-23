# Read the working copy's script files in parallel: implementation plan

**Goal:** `scripts::read` reads the working copy's script files on several threads before its loop,
so a cold run's file openings, which Windows Defender scans, overlap; then the gap to fallow is
measured again and judged by the owner's bands.

**Architecture:** Three private functions join `src/scripts.rs`: `wanted` (the files the loop
reads), `threads` (how many threads read, from the machine's parallelism and the number of files)
and `ahead` (the pre-read: scoped threads that take files from one shared counter and put each
result in that file's slot). `scripts::read` runs `ahead` on the working copy inside the
`scriptsRead` stage, then runs today's loop over `wanted`, taking each file's bytes from the
pre-read, or reading it through `Tree::read` when there is none, as on the base. The unit tests sit
in `src/scripts/tests.rs`.

**Spec:** docs/specs/2026-11-21-parallel-cold-reading-design.md

**Implements:** PAR-1 to PAR-7, CLOCK-1, CLOCK-2, PTEST-1, PTEST-1a, PTEST-1b, PTEST-2, PTEST-3, CMP-1 to CMP-6, DONE-1

## Global constraints

- Output, exit code, notices and their order, the gone set, and which broken file stops a run are
  unchanged on every project, whatever the thread count (the spec's anti-goals).
- No new option, setting or environment variable; the thread count is
  `std::thread::available_parallelism()`, or 1 when it fails, capped at the number of script files
  (PAR-4).
- No new crate: standard-library scoped threads, each started with
  `std::thread::Builder::spawn_scoped` (PAR-5).
- Only the working copy is read ahead; the base reads one file at a time through its git (PAR-2).
- The analyzer only reads, and starts no process it doesn't start today (CONTRIBUTING.md).

## The decisions this plan makes

1. **The code stays in `src/scripts.rs`**, the module whose loop it feeds, rather than in `Tree` or a
   module of its own: `ahead` reads through `Tree::read`, so it serves any tree, and `scripts::read`
   alone decides it runs on the working copy.
2. **Each file's result goes into a `Mutex<Option<_>>` slot of its own, at its index among the
   wanted files** (the spec's "how each result reaches its file's place"). A slot's lock is taken
   once, by the one thread that read the file, so there is no contention.
3. **The seam of PTEST-1a is an argument, `may_start: impl Fn(usize) -> bool`**, asked before each
   thread is started, numbered from 1; `scripts::read` passes `|_| true`. A refusal takes the same
   path as `spawn_scoped` returning an error.
4. **`threads` returns 0 with no files**, so "no pre-read" (PTEST-1b) is a count of 0; `ahead` over
   an empty list starts nothing and returns nothing.
5. **CLOCK-2 needs no code:** `scripts/measure.mjs --combine` already refuses files whose
   executables' hashes differ.
6. **PTEST-3's fixture goes in `tests/imports.test.rs`**, beside the other tests of what `graph`
   reads from scripts, using its `file`, `own` and `plain` helpers.
7. **The unit tests write their files under the system's temporary folder**, in a folder named for
   the test and the process, removed when the test ends; the crate has no temporary-folder crate and
   this plan adds none.
8. **The Test line runs the whole suite**, so the fixture tests PTEST-2 and PTEST-3 name run with
   the unit tests; before the code is written, the unit tests don't compile, so it fails.

## Review focus

- **A pre-read thread that panics** (PAR-3): no test makes `fs::read` panic. The reviewer checks
  that every started thread is joined explicitly and its panic passed on with
  `panic::resume_unwind`, so the hook prints one line, as `src/audit.rs` does.
- **`spawn_scoped` itself failing** (PAR-5): PTEST-1a refuses starts through the seam; the real
  error path is the `.ok()` that drops a failed start, which the reviewer reads.
- **`audit --base`** (PAR-7): no new test runs it; `tests/audit.test.rs`'s base tests pass
  unchanged, and the reviewer checks the `tree.is_base()` branch skips the pre-read.
- **Memory on a large project** (the spec's Risks): every script file's bytes are held at once;
  nothing tests a project larger than 64 files.

### Task 1: read the working copy's script files ahead, on several threads

**Implements:** PAR-1, PAR-2, PAR-3, PAR-4, PAR-5, PAR-6, PAR-7, CLOCK-1, CLOCK-2, PTEST-1, PTEST-1a, PTEST-1b, PTEST-2, PTEST-3

**Test:** `cargo test --release --locked`

**Files:**
- Modify: `src/scripts.rs`
- Create: `src/scripts/tests.rs`
- Modify: `tests/imports.test.rs`

**Interfaces:**
- Consumes: `Tree::read`, `Tree::is_base` and `Tree::working` (`src/tree.rs`); `File` and `Kind`
  (`src/files.rs`); `Stages::time` (`src/timings.rs`); `Project`, `paths` (`tests/support/project.rs`).
- Produces: `scripts::read`, with the same signature; private `wanted`, `threads` and `ahead` in
  `src/scripts.rs`.

- [ ] **Step 1: Write the failing tests**

```rust path=src/scripts/tests.rs
//! The pre-read: which files it reads, that its results don't depend on the number of threads or on
//! threads that could not start, and how many threads it takes (parallel reading spec PTEST-1,
//! PTEST-1a, PTEST-1b).

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, ErrorKind};
use std::num::NonZeroUsize;
use std::path::PathBuf;

use super::{ahead, threads, wanted};
use crate::files::{File, Kind};
use crate::tree::Tree;

/// What reading one file gave, in a form two readings can be compared by: its bytes, or its error's
/// kind, since `io::Error` can't be compared.
type Seen = Option<Result<Vec<u8>, ErrorKind>>;

/// Which threads are refused, in words, and the refusal: whether the thread of each number may start.
type Refusal = (&'static str, fn(usize) -> bool);

/// A folder of the test's own under the system's temporary folder, removed when the test ends.
struct Folder(PathBuf);

impl Folder {
    /// A new, empty folder named for the test and this process.
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("yak-analyzer-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        fs::create_dir_all(&path).expect("the folder can be made");

        Self(path)
    }

    /// Writes a file at the path given, from the folder, making its folders.
    fn write(&self, path: &str, content: &str) {
        let full = self.0.join(path);

        fs::create_dir_all(full.parent().expect("a file has a folder"))
            .expect("the folders can be made");
        fs::write(full, content).expect("the file can be written");
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A tracked file of the list.
fn listed(path: &str, kind: Kind) -> File {
    File {
        path: path.to_owned(),
        kind,
        tracked: true,
    }
}

/// A project of twelve scripts, a script declared as data, a `package.json`, a listed script that is
/// gone and a folder named like a script, with its file list in path order.
fn project(folder: &Folder) -> Vec<File> {
    let mut files: Vec<File> = (0..12)
        .map(|number| {
            let path = format!("src/f{number:02}.ts");

            folder.write(&path, &format!("export const n{number} = {number};\n"));

            listed(&path, Kind::Script)
        })
        .collect();

    folder.write("src/data.ts", "export const data = 1;\n");
    folder.write("package.json", "{}\n");
    fs::create_dir_all(folder.0.join("src/folder.ts")).expect("the folder can be made");
    files.push(listed("src/data.ts", Kind::Script));
    files.push(listed("package.json", Kind::Manifest));
    files.push(listed("src/gone.ts", Kind::Script));
    files.push(listed("src/folder.ts", Kind::Script));
    files.sort_by(|one, other| one.path.cmp(&other.path));

    files
}

/// The files declared as data in `project`.
fn data() -> BTreeSet<String> {
    BTreeSet::from(["src/data.ts".to_owned()])
}

/// What the pre-read gave, as it can be compared.
fn seen(results: Vec<Option<io::Result<Vec<u8>>>>) -> Vec<Seen> {
    results
        .into_iter()
        .map(|result| result.map(|bytes| bytes.map_err(|error| error.kind())))
        .collect()
}

/// Reading each file one after another, as the loop did before the pre-read.
fn one_by_one(tree: &Tree, files: &[&File]) -> Vec<Seen> {
    files
        .iter()
        .map(|file| Some(tree.read(&file.path).map_err(|error| error.kind())))
        .collect()
}

#[test]
fn reads_ahead_only_the_files_the_loop_reads() {
    let folder = Folder::new("wanted");
    let files = project(&folder);
    let data = data();
    let paths: Vec<&str> = wanted(&files, &data)
        .iter()
        .map(|file| file.path.as_str())
        .collect();

    assert_eq!(paths.len(), 14, "{paths:?}");
    assert!(
        !paths.contains(&"src/data.ts"),
        "a data file is never read: {paths:?}"
    );
    assert!(
        !paths.contains(&"package.json"),
        "a `package.json` is not a script: {paths:?}"
    );
    assert!(
        paths.contains(&"src/gone.ts") && paths.contains(&"src/folder.ts"),
        "{paths:?}"
    );
}

#[test]
fn gives_the_same_results_in_the_same_places_whatever_the_number_of_threads() {
    let folder = Folder::new("threads");
    let files = project(&folder);
    let data = data();
    let wanted = wanted(&files, &data);
    let tree = Tree::working(folder.0.clone());
    let expected = one_by_one(&tree, &wanted);
    let place = |path: &str| {
        wanted
            .iter()
            .position(|file| file.path == path)
            .expect("the file is wanted")
    };

    assert_eq!(
        expected[place("src/gone.ts")],
        Some(Err(ErrorKind::NotFound))
    );
    assert!(
        matches!(expected[place("src/folder.ts")], Some(Err(_))),
        "a folder cannot be read as a file"
    );

    for count in [1, 2, 3, 8] {
        assert_eq!(
            seen(ahead(&tree, &wanted, count, |_| true)),
            expected,
            "{count} threads"
        );
    }
}

#[test]
fn reads_every_file_when_threads_cannot_start() {
    let folder = Folder::new("refused");
    let files = project(&folder);
    let data = data();
    let wanted = wanted(&files, &data);
    let tree = Tree::working(folder.0.clone());
    let expected = one_by_one(&tree, &wanted);
    let refusals: [Refusal; 2] = [
        ("every thread", |_| false),
        ("the second thread", |number| number != 2),
    ];

    for (refused, may_start) in refusals {
        assert_eq!(
            seen(ahead(&tree, &wanted, 4, may_start)),
            expected,
            "{refused} refused"
        );
    }
}

#[test]
fn counts_the_threads_from_the_machine_and_the_files() {
    let sixteen = || Ok(NonZeroUsize::new(16).expect("16 is not 0"));
    let unknown = || Err(io::Error::other("the parallelism is unknown"));

    assert_eq!(threads(unknown(), 2_744), 1);
    assert_eq!(threads(sixteen(), 3), 3);
    assert_eq!(threads(sixteen(), 2_744), 16);
    assert_eq!(threads(sixteen(), 0), 0);
    assert_eq!(threads(unknown(), 0), 0);
}
```

```rust edit path=tests/imports.test.rs
<<<<<<< find
#[test]
fn finds_every_form_of_import_with_its_line() {
=======
#[test]
fn reads_a_chain_of_more_scripts_than_threads_in_the_list_order() {
    // 64 scripts: more than the threads of any machine the tests run on, so the pre-read hands some
    // thread several files (parallel reading spec PTEST-3).
    let project = Project::new("imports-many");
    let names: Vec<String> = (0..64)
        .map(|number| format!("src/f{number:02}.ts"))
        .collect();

    for (number, name) in names.iter().enumerate() {
        let next = number + 1;
        let content = if next < names.len() {
            format!(
                "import {{ n{next} }} from './f{next:02}';\nexport const n{number} = n{next};\n"
            )
        } else {
            format!("export const n{number} = {number};\n")
        };

        project.write(name, &content);
    }

    project.commit();

    let document = project.graph();

    assert_eq!(
        paths(&document),
        names.iter().map(String::as_str).collect::<Vec<_>>()
    );

    for (number, name) in names.iter().enumerate() {
        let next = number + 1;
        let expected = if next < names.len() {
            json!([plain(&format!("./f{next:02}"), "import", 1)])
        } else {
            json!([])
        };

        assert_eq!(own(&file(&document, name)["imports"]), expected, "{name}");
    }
}

#[test]
fn finds_every_form_of_import_with_its_line() {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```bash
cargo test --release --locked
```

Expected: the build fails: `src/scripts/tests.rs` is not yet declared, and `wanted`, `threads` and
`ahead` don't exist.

- [ ] **Step 3: Write the pre-read**

```rust edit path=src/scripts.rs
<<<<<<< find
//! and HTML pages by their own readers (spec PARSE-1 to PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
=======
//! and HTML pages by their own readers (spec PARSE-1 to PARSE-6, LIST-3).
//!
//! The working copy's files are read ahead on several threads before they are parsed one after
//! another, so the first opening of each, which Windows Defender scans, overlaps with the others
//! (parallel reading spec PAR-1 to PAR-7).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::num::NonZeroUsize;
use std::panic;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
>>>>>>> replace
<<<<<<< find
/// imports and no exports (dead-code spec COMMIT-4). The time spent reading from the tree and the time
/// spent parsing go into the stages given apart, each summed over the files (spec TIME-2, TIME-2b).
=======
/// imports and no exports (dead-code spec COMMIT-4).
///
/// On the working copy, every file is read ahead on several threads before the loop; the base's files
/// are read one at a time in the loop, through its one git (parallel reading spec PAR-1, PAR-2). The
/// time spent reading goes into `scriptsRead`: on the working copy the wall-clock time of the pre-read,
/// on the base each file's read, summed. The time spent parsing goes into `scriptsParse`, summed over
/// the files (spec TIME-2, TIME-2b; parallel reading spec CLOCK-1).
>>>>>>> replace
<<<<<<< find
    let mut scripts = Scripts::default();

    for file in files {
        if !reads(file.kind) || data.contains(&file.path) {
            continue;
        }

        let bytes = stages.time("scriptsRead", || tree.read(&file.path));
=======
    let mut scripts = Scripts::default();
    let wanted = wanted(files, data);
    let read_ahead = if tree.is_base() {
        Vec::new()
    } else {
        stages.time("scriptsRead", || {
            let count = threads(thread::available_parallelism(), wanted.len());

            ahead(tree, &wanted, count, |_| true)
        })
    };
    let mut read_ahead = read_ahead.into_iter();

    for file in wanted {
        // A file the pre-read holds no result for, every file at the base, is read here (parallel
        // reading spec PAR-3).
        let bytes = match read_ahead.next().flatten() {
            Some(bytes) => bytes,
            None => stages.time("scriptsRead", || tree.read(&file.path)),
        };
>>>>>>> replace
<<<<<<< find
/// The notice for what a reader passed over (spec PARSE-3, PARSE-4).
=======
/// The files of the list whose imports are read, in its order: the kinds the analyzer reads, less
/// those declared as data (spec PARSE-1; parallel reading spec PAR-6).
fn wanted<'a>(files: &'a [File], data: &BTreeSet<String>) -> Vec<&'a File> {
    files
        .iter()
        .filter(|file| reads(file.kind) && !data.contains(&file.path))
        .collect()
}

/// How many threads read ahead, the reading's own included: the machine's parallelism, or 1 when it
/// cannot be told, and never more than the files to read, so 0 when there are none (parallel reading
/// spec PAR-4).
fn threads(parallelism: io::Result<NonZeroUsize>, files: usize) -> usize {
    parallelism.map_or(1, NonZeroUsize::get).min(files)
}

/// Reads every file given from the tree, on the number of threads given, the calling thread included,
/// and returns each file's result in its place. Each thread takes the next file nobody has taken until
/// none is left, so the files of a thread that could not start, or that `may_start` refuses, are read
/// by the others. A thread that panics passes its panic on, once every thread is joined (parallel
/// reading spec PAR-1, PAR-3, PAR-5).
fn ahead(
    tree: &Tree,
    files: &[&File],
    threads: usize,
    may_start: impl Fn(usize) -> bool,
) -> Vec<Option<io::Result<Vec<u8>>>> {
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<io::Result<Vec<u8>>>>> =
        files.iter().map(|_| Mutex::new(None)).collect();
    let work = || {
        let mut index = next.fetch_add(1, Ordering::Relaxed);

        while let Some(file) = files.get(index) {
            let bytes = tree.read(&file.path);

            if let Ok(mut slot) = slots[index].lock() {
                *slot = Some(bytes);
            }

            index = next.fetch_add(1, Ordering::Relaxed);
        }
    };

    thread::scope(|scope| {
        let started: Vec<_> = (1..threads)
            .filter(|&number| may_start(number))
            .filter_map(|_| thread::Builder::new().spawn_scoped(scope, work).ok())
            .collect();

        work();

        for reader in started {
            if let Err(fault) = reader.join() {
                panic::resume_unwind(fault);
            }
        }
    });

    slots
        .into_iter()
        .map(|slot| slot.into_inner().ok().flatten())
        .collect()
}

/// The notice for what a reader passed over (spec PARSE-3, PARSE-4).
>>>>>>> replace
<<<<<<< find
        _ => imports::read(path, text)
            .map(|read| (read, Vec::new()))
            .map_err(broken),
    }
}
=======
        _ => imports::read(path, text)
            .map(|read| (read, Vec::new()))
            .map_err(broken),
    }
}

#[cfg(test)]
mod tests;
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```bash
cargo test --release --locked
```

Expected: every test passes, the four new unit tests in `scripts::tests`, the new
`reads_a_chain_of_more_scripts_than_threads_in_the_list_order`, and
`stops_on_a_tracked_script_that_does_not_parse_the_first_by_path` unchanged (PTEST-2), among them.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: each exits 0 with no warning.

- [ ] **Step 6: Commit**

```bash
git add src/scripts.rs src/scripts/tests.rs tests/imports.test.rs
git commit -m "perf(reading): read the working copy's scripts ahead on several threads"
```

Expected: one commit on `feat/match-fallow-speed`.

### Task 2: measure the gap to fallow again

**Implements:** CMP-1, CMP-2, CMP-3, CMP-4, CMP-5, CMP-6, DONE-1

**Check:** On the owner's Windows machine, with this branch's release build (`cargo build --release`)
copied over `spike/yak-analyzer.exe` in the report folder (the 0.10.0 copy renamed
`yak-analyzer-0.10.0.exe` first), `node spike.mjs warm` is run twice before any restart (CMP-2,
CMP-5); then two restarts, after each `spike.mjs` is the first thing run, `cold yak` after one and
`cold fallow` after the other (CMP-3), with yakshaver's commit recorded and unchanged between them,
and one more pair of restarts if the cold gap lies within 300ms of a band's edge (CMP-4). The report
in the report folder gives the warm gap (median of twenty runs each, minus), the cold gap, the band
of each, the build measured and yakshaver's commit, and says the cold times aren't strictly like for
like (CMP-6); the headline goes on the work item's record; and a gap over 300ms opens the work item
DONE-1 names, with `yak work open`.
