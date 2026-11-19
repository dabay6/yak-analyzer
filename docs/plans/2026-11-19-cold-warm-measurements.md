# Measure cold against warm runs: implementation plan

**Goal:** a switch that makes `graph` report the time of each stage, a script that measures cold,
warm and first-start runs with it and combines rounds, a CI job that measures on Linux, and the
measurements themselves.

**Architecture:** A new `src/timings.rs` holds `Stages`, which times a piece of work only when the
switch is on. `graph::read` wraps each stage in it, `scripts::read` splits each file's disk read from
its parsing, and `src/main.rs` times printing and the whole run and writes the one line. The script
is plain Node: its arithmetic and its table live in `scripts/lib/measure.mjs`, tested on their own,
and `scripts/measure.mjs` runs the analyzer. A new workflow file, started by hand, runs the script on
a public repository pinned to one commit.

**Spec:** docs/specs/2026-11-19-cold-warm-measurements-design.md

**Implements:** TIME-1, TIME-2, TIME-2a, TIME-3, TIME-4, TIME-4a, TIME-5, TIME-6, TEST-1, TEST-1a, TEST-2, MEAS-1, MEAS-2, MEAS-3, MEAS-3a, MEAS-4, MEAS-5, MEAS-6, GEN-1, GEN-2, JOB-1, JOB-2, JOB-3, JOB-4, TAKE-1, TAKE-1a, TAKE-2, TAKE-3, TAKE-4

## Global constraints

- The switch is `YAK_ANALYZER_TIMINGS` set to exactly `1`; anything else, or unset, changes nothing
  (TIME-1).
- Times are whole microseconds (TIME-1). The line is
  `{"timings":{"reading":{...},"print":n,"total":n}}` (TIME-5), written only on a run that
  succeeded, with `writeln!`, its failure ignored (TIME-3, TIME-4a).
- The clock is read only when the switch is on (the spec's first risk).
- The analyzer only reads: nothing here writes inside a checkout (CONTRIBUTING.md). The script's git
  commands take `--no-optional-locks`, so git writes no index refresh in the measured folder.
- Every crate stays pinned; this plan adds none. The scripts use Node alone.
- Nothing committed names a machine or a private repository other than yakshaver (CONTRIBUTING.md).

## The decisions this plan makes

1. **The stand-in is `mui/base-ui` at commit `35fb3007e948750601045d5b88404f5f49646b61`** (GEN-2):
   a pnpm TypeScript workspace of 2,848 tracked files with a `tsconfig.json`, MIT-licensed. `graph`
   read it at that commit without a hand-off file and without stopping (checked on 2026-11-19: 1.3
   to 1.6s warm on Windows). Four others were tried and stopped on fixtures that are broken on
   purpose, which only a hand-off file's `data` declarations would pass over: `vitejs/vite`,
   `vitest-dev/vitest`, `trpc/trpc` and `ionic-team/stencil`.
2. **`Stages::time(stage, work)`** runs a closure and adds its time to the stage; off, it only runs
   the closure. A stage timed twice adds up, which is how `scriptsRead` and `scriptsParse` sum over
   files and how `resolve` covers both building the resolver and resolving.
3. **The line always holds every reading key** of TIME-2, 0 for one that never ran (`READING` in
   `src/timings.rs`), so a reader never meets a missing key.
4. **`scripts::read` reads each file from disk inside `scriptsRead`, then hands the bytes to `one`
   inside `scriptsParse`.** `one` no longer reads the disk itself.
5. **The script is split** into `scripts/lib/measure.mjs` (finding the timing line, the columns,
   medians, combining, the table) and `scripts/measure.mjs` (options, running, temporary folders), so
   MEAS-6's tests need no executable.
6. **The table is Markdown**, so the CI job appends it to the job's summary unchanged. Each cell is
   the median in milliseconds with the lowest and highest in brackets. Its rows are the stages, then
   `print`, `other`, `total`, `start` (wall-clock time minus `total`: process start and exit) and
   `wall`.
7. **The timed runs discard standard output** (`stdio` `ignore`), so the script doesn't hold a
   document of several megabytes per run and Node's reading of the pipe doesn't land in the
   wall-clock time. So `print` measures a write to the null device, not to yak's pipe.
8. **The warm-up project** is four files left untracked in a fresh `git init` folder: a
   `package.json`, a `tsconfig.json`, and two TypeScript files that import each other.
9. **CONTRIBUTING.md is not changed.** The spec doesn't ask for it, and the script's own header says
   how to run it.

## Review focus

- **A developer's environment with the switch set** (TEST-1a): every place a test starts the binary
  removes the variable, but no test can prove it, since setting a variable in the test process is
  `unsafe` in edition 2024 and would leak into the other tests. The reviewer checks Task 1's three
  `env_remove` calls cover every `CARGO_BIN_EXE_yak-analyzer` in `tests/`.
- **TIME-4's files read and git commands** are checked by reading `graph::read` and `scripts::read`:
  each closure wraps the call that was there, with the same arguments.
- **A script stopped with Ctrl+C** removes its temporary folder through the `exit` handler; no test
  sends it a signal.
- **The executable's copy on Linux** must stay executable: `copyFileSync` keeps the file's mode
  through libuv. The CI job's first-start run is the check.
- **`--drop-caches` without passwordless sudo** stops with its own line; only a machine without it
  shows that, which the CI runner isn't.

### Task 1: the timing switch

**Implements:** TIME-1, TIME-2, TIME-2a, TIME-3, TIME-4, TIME-4a, TIME-5, TIME-6, TEST-1, TEST-1a, TEST-2

**Test:** `cargo test --release --locked --test timings`

**Files:**
- Create: `src/timings.rs`, `tests/timings.test.rs`
- Modify: `src/main.rs`, `src/graph.rs`, `src/scripts.rs`, `Cargo.toml`,
  `tests/support/project.rs`, `tests/support/mod.rs`, `tests/cli.test.rs`

**Interfaces:**
- Consumes: `graph::read`, `scripts::read`, `files::list`, `marks::apply`, `packages::load`,
  `tsconfig::load`, `Resolver`, `entries::find`, as they are.
- Produces: `timings::asked() -> bool`, `timings::Stages` with `new(bool)` and
  `time(&'static str, FnOnce() -> T) -> T`, `timings::line(&Stages, Duration) -> String`,
  `timings::write(&str)`; `graph::read(Option<&Path>, &mut Stages)`;
  `scripts::read(.., &mut Stages)`; the line on standard error, which Task 2 reads.

- [ ] **Step 1: Write the failing test**

```toml edit path=Cargo.toml
<<<<<<< find
[[test]]
name = "entries"
path = "tests/entries.test.rs"
=======
[[test]]
name = "entries"
path = "tests/entries.test.rs"

[[test]]
name = "timings"
path = "tests/timings.test.rs"
>>>>>>> replace
```

```rust path=tests/timings.test.rs
//! Runs `graph` with the timing switch set and unset, and checks that it changes nothing but the one
//! line it adds on a run that succeeds (spec TIME-1 to TIME-5, TEST-1, TEST-2).

#[path = "support/project.rs"]
mod project;

use serde_json::Value;

use project::{Project, printed, stopped};

/// The variable that turns the timings on.
const VARIABLE: &str = "YAK_ANALYZER_TIMINGS";

/// The keys of a reading's stages (spec TIME-2).
const READING: [&str; 10] = [
    "handOff",
    "files",
    "marks",
    "packages",
    "tsconfig",
    "scriptsRead",
    "scriptsParse",
    "resolve",
    "entries",
    "build",
];

/// A small project: a package, a tsconfig, and two scripts that import each other.
fn project(name: &str) -> Project {
    let project = Project::new(name);

    project
        .write("package.json", r#"{ "name": "app", "main": "src/a.ts" }"#)
        .write("tsconfig.json", r#"{ "compilerOptions": {} }"#)
        .write("src/a.ts", "import { b } from './b';\n\nexport const a = b;\n")
        .write("src/b.ts", "import type { a } from './a';\n\nexport const b = 1;\n")
        .commit();

    project
}

#[test]
fn set_to_1_it_adds_one_line_with_every_stage_and_changes_nothing_else() {
    let project = project("timings-set");
    let unset = project.run(|_| {});
    let set = project.run(|binary| {
        binary.env(VARIABLE, "1");
    });

    printed(&unset);
    assert_eq!(set.status.code(), unset.status.code());
    assert_eq!(set.stdout, unset.stdout);

    let error = String::from_utf8_lossy(&set.stderr);

    assert_eq!(error.lines().count(), 1, "{error}");

    let line: Value = serde_json::from_str(error.trim_end()).expect("the line is JSON");
    let timings = &line["timings"];

    for stage in READING {
        assert!(timings["reading"][stage].is_u64(), "{stage}: {line}");
    }

    assert_eq!(
        timings["reading"].as_object().map(serde_json::Map::len),
        Some(READING.len()),
        "{line}"
    );
    assert!(timings["print"].is_u64(), "{line}");
    assert!(timings["total"].is_u64(), "{line}");
    assert_eq!(line.as_object().map(serde_json::Map::len), Some(1), "{line}");
}

#[test]
fn any_other_value_leaves_standard_error_empty() {
    let project = project("timings-other-value");

    for value in ["0", "true", "yes", ""] {
        printed(&project.run(|binary| {
            binary.env(VARIABLE, value);
        }));
    }
}

#[test]
fn a_run_that_stops_writes_its_one_stop_line_and_no_timings() {
    let project = project("timings-stopped");
    let missing = project.home.join("no-such-hand-off.json");
    let output = project.run(|binary| {
        binary.arg("--hand-off").arg(&missing).env(VARIABLE, "1");
    });
    let line = stopped(&output);

    assert!(line.starts_with("yak-analyzer: "), "{line}");
    assert!(!line.contains("timings"), "{line}");
}
```

```rust edit path=tests/support/project.rs
<<<<<<< find
/// Shuts the machine's and the user's own git settings out of a command, global excludes included, so
/// that a test's result does not depend on the machine.
fn alone(command: &mut Command) -> &mut Command {
=======
/// Shuts the machine's and the user's own git settings out of a command, global excludes included, and
/// the analyzer's timing switch, so that a test's result does not depend on the machine (spec TEST-1a).
fn alone(command: &mut Command) -> &mut Command {
>>>>>>> replace
<<<<<<< find
        .env("GIT_CONFIG_NOSYSTEM", "1")
}
=======
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("YAK_ANALYZER_TIMINGS")
}
>>>>>>> replace
```

```rust edit path=tests/support/mod.rs
<<<<<<< find
/// Runs the built binary with the arguments given.
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
=======
/// Runs the built binary with the arguments given, without the timing switch a developer's environment
/// may set (spec TEST-1a).
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .env_remove("YAK_ANALYZER_TIMINGS")
>>>>>>> replace
```

```rust edit path=tests/cli.test.rs
<<<<<<< find
/// Runs the built binary with the arguments given.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
=======
/// Runs the built binary with the arguments given, without the timing switch a developer's environment
/// may set (spec TEST-1a).
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .env_remove("YAK_ANALYZER_TIMINGS")
>>>>>>> replace
<<<<<<< find
    let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .arg("graph")
        .current_dir(&root)
=======
    let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .arg("graph")
        .env_remove("YAK_ANALYZER_TIMINGS")
        .current_dir(&root)
>>>>>>> replace
```

- [ ] **Step 2: Run the test to see it fail**

```bash
cargo test --release --locked --test timings
```

Expected: FAIL. `set_to_1_it_adds_one_line_with_every_stage_and_changes_nothing_else` finds no line
on standard error; the other two pass.

- [ ] **Step 3: Write the code**

```rust path=src/timings.rs
//! The time each stage of a run takes, written on standard error as one line when the variable
//! `YAK_ANALYZER_TIMINGS` is `1` (spec TIME-1 to TIME-6).

use std::env;
use std::io::{self, Write as _};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

/// The variable that turns the timings on.
const VARIABLE: &str = "YAK_ANALYZER_TIMINGS";

/// The stages of a reading, in the order `graph::read` takes them (spec TIME-2). The line holds every
/// one, 0 for a stage that never ran.
const READING: [&str; 10] = [
    "handOff",
    "files",
    "marks",
    "packages",
    "tsconfig",
    "scriptsRead",
    "scriptsParse",
    "resolve",
    "entries",
    "build",
];

/// Whether the timings were asked for: the variable is set to `1`, and to nothing else (spec TIME-1).
pub fn asked() -> bool {
    env::var_os(VARIABLE).is_some_and(|value| value == "1")
}

/// The time each stage took. It reads the clock only when the timings were asked for.
#[derive(Debug)]
pub struct Stages {
    /// Whether the timings were asked for.
    on: bool,
    /// Each stage that ran, with the time it took, summed over every time it ran.
    taken: Vec<(&'static str, Duration)>,
}

impl Stages {
    /// Stages that keep their times when `on` is true, and do nothing but run the work otherwise.
    pub const fn new(on: bool) -> Self {
        Self {
            on,
            taken: Vec::new(),
        }
    }

    /// Runs the work given and adds the time it took to the stage named.
    pub fn time<T>(&mut self, stage: &'static str, work: impl FnOnce() -> T) -> T {
        if !self.on {
            return work();
        }

        let started = Instant::now();
        let done = work();

        self.add(stage, started.elapsed());

        done
    }

    /// Adds a time to the stage named, to what it already holds.
    fn add(&mut self, stage: &'static str, taken: Duration) {
        match self.taken.iter_mut().find(|(name, _)| *name == stage) {
            Some((_, sum)) => *sum += taken,
            None => self.taken.push((stage, taken)),
        }
    }

    /// The time the stage named took, or nothing when it never ran.
    fn of(&self, stage: &str) -> Duration {
        self.taken
            .iter()
            .find(|(name, _)| *name == stage)
            .map_or(Duration::ZERO, |(_, taken)| *taken)
    }
}

/// The line: the reading's stages, printing, and the whole run, in whole microseconds (spec TIME-5).
/// A later command adds its own reading under a key of its own beside `reading` (spec TIME-6).
pub fn line(stages: &Stages, total: Duration) -> String {
    let reading: Map<String, Value> = READING
        .iter()
        .map(|stage| ((*stage).to_owned(), micros(stages.of(stage))))
        .collect();

    json!({
        "timings": {
            "reading": reading,
            "print": micros(stages.of("print")),
            "total": micros(total),
        }
    })
    .to_string()
}

/// Writes the line on standard error. A standard error that cannot be written to is passed over, so a
/// run that succeeded never ends as a fault (spec TIME-4a).
pub fn write(line: &str) {
    let _ = writeln!(io::stderr(), "{line}");
}

/// A time as whole microseconds.
fn micros(taken: Duration) -> Value {
    Value::from(u64::try_from(taken.as_micros()).unwrap_or(u64::MAX))
}
```

```rust edit path=src/main.rs
<<<<<<< find
mod stop;
mod tsconfig;
=======
mod stop;
mod timings;
mod tsconfig;
>>>>>>> replace
<<<<<<< find
use std::process::ExitCode;

use cli::Command;
=======
use std::process::ExitCode;
use std::time::Instant;

use cli::Command;
use timings::Stages;
>>>>>>> replace
<<<<<<< find
fn run() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
=======
fn run() -> ExitCode {
    // When the timings are asked for, the whole run is timed from here, before the arguments are read
    // (spec TIME-2a).
    let started = timings::asked().then(Instant::now);
    let args: Vec<OsString> = env::args_os().skip(1).collect();
>>>>>>> replace
<<<<<<< find
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
=======
        Ok(Command::Graph { hand_off }) => {
            let mut stages = Stages::new(started.is_some());

            match graph::read(hand_off.as_deref(), &mut stages) {
                Ok(document) => {
                    stages.time("print", || println!("{document}"));

                    // Only a run that succeeded writes the line (spec TIME-3).
                    if let Some(started) = started {
                        timings::write(&timings::line(&stages, started.elapsed()));
                    }

                    ExitCode::SUCCESS
                }
                Err(stopped) => {
                    eprintln!("{}", stopped.line());

                    ExitCode::from(stop::STOPPED)
                }
            }
        }
>>>>>>> replace
```

```rust edit path=src/graph.rs
<<<<<<< find
use crate::hand_off;
=======
use crate::hand_off::{self, HandOff};
>>>>>>> replace
<<<<<<< find
use crate::stop::Stop;
use crate::tsconfig;
=======
use crate::stop::Stop;
use crate::timings::Stages;
use crate::tsconfig;
>>>>>>> replace
<<<<<<< find
/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
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
=======
/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print. The time of each stage goes into the stages given (spec TIME-2).
pub fn read(hand_off_path: Option<&Path>, stages: &mut Stages) -> Result<String, Stop> {
    let mut notices = Vec::new();
    let handed = stages.time("handOff", || -> Result<Option<HandOff>, Stop> {
        match hand_off_path {
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

                Ok(Some(handed))
            }
            None => {
                notices.push(Notice::new(
                    "no-hand-off-file",
                    None,
                    "No hand-off file was given, so the project is read from its own standard files, with no \
                     declarations."
                        .to_owned(),
                ));

                Ok(None)
            }
        }
    })?;
>>>>>>> replace
<<<<<<< find
    let listing = files::list(&root, &mut notices)?;
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed.as_ref(), &mut notices)?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = packages::load(&root, files, &marks.data)?.checked(
        files,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    );
    let configs = tsconfig::load(
        &root,
        files,
        &marks.data,
        packages.as_ref().ok(),
        &mut notices,
    )?
    .checked();
=======
    let listing = stages.time("files", || files::list(&root, &mut notices))?;
    let files = listing.files.as_slice();
    let marks = stages.time("marks", || {
        marks::apply(files, handed.as_ref(), &mut notices)
    })?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = stages.time("packages", || {
        Ok::<_, Stop>(packages::load(&root, files, &marks.data)?.checked(
            files,
            handed.as_ref().map(|handed| handed.folders.as_slice()),
            &mut notices,
        ))
    })?;
    let configs = stages.time("tsconfig", || {
        Ok::<_, Stop>(
            tsconfig::load(
                &root,
                files,
                &marks.data,
                packages.as_ref().ok(),
                &mut notices,
            )?
            .checked(),
        )
    })?;
>>>>>>> replace
<<<<<<< find
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed: handed.as_ref(),
        },
        &mut notices,
    )?;
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
            .faults
            .iter()
            .map(|(folder, fault)| json!({ "fault": fault.name(), "package": folder }))
            .collect::<Vec<Value>>(),
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}
=======
    let scripts = scripts::read(&root, files, &marks.data, &mut notices, stages)?;
    let resolver = stages.time("resolve", || {
        Resolver::new(&listing, &packages, &configs)
    });
    let resolved = stages.time("resolve", || resolver.resolve(&root, &scripts))?;
    let entries = stages.time("entries", || {
        entries::find(
            &entries::Project {
                files,
                packages: &packages,
                configs: &configs,
                resolver: &resolver,
                resolved: &resolved,
                handed: handed.as_ref(),
            },
            &mut notices,
        )
    })?;

    Ok(stages.time("build", || {
        let mut document = json!({
            "entries": entry_points(&entries),
            "entryFaults": entries
                .faults
                .iter()
                .map(|(folder, fault)| json!({ "fault": fault.name(), "package": folder }))
                .collect::<Vec<Value>>(),
            "files": files
                .iter()
                .filter(|each| !scripts.gone.contains(&each.path))
                .map(|each| file(each, &marks, &packages, &scripts, &resolved))
                .collect::<Vec<Value>>(),
            "handOff": handed.is_some(),
            "notices": notice::printed(notices),
            "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
        });

        // An object's keys in order, whatever order they were added in.
        document.sort_all_objects();

        format!("{document:#}")
    }))
}
>>>>>>> replace
```

```rust edit path=src/scripts.rs
<<<<<<< find
use std::io::ErrorKind;
=======
use std::io::{self, ErrorKind};
>>>>>>> replace
<<<<<<< find
use crate::stop::Stop;
=======
use crate::stop::Stop;
use crate::timings::Stages;
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
) -> Result<Scripts, Stop> {
    let mut scripts = Scripts::default();

    for file in files {
        if !reads(file.kind) || data.contains(&file.path) {
            continue;
        }

        match one(root, file) {
=======
/// Reads every script, type declaration file, Vue component, MDX document and HTML page in the list
/// that is not data, in the order of the list. A tracked file that cannot be read stops the run; an
/// untracked one is skipped with a notice (spec PARSE-2). The time spent reading from disk and the
/// time spent parsing go into the stages given apart, each summed over the files (spec TIME-2).
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
    stages: &mut Stages,
) -> Result<Scripts, Stop> {
    let mut scripts = Scripts::default();

    for file in files {
        if !reads(file.kind) || data.contains(&file.path) {
            continue;
        }

        let bytes = stages.time("scriptsRead", || fs::read(root.join(&file.path)));

        match stages.time("scriptsParse", || one(file, bytes)) {
>>>>>>> replace
<<<<<<< find
/// Reads one file and finds its imports, with what its reader passed over.
fn one(root: &Path, file: &File) -> Result<(Read, Vec<Skipped>), Unread> {
    let path = file.path.as_str();
    let bytes = fs::read(root.join(path)).map_err(|error| match error.kind() {
=======
/// Finds the imports of one file from what reading it from disk gave, with what its reader passed over.
fn one(file: &File, bytes: io::Result<Vec<u8>>) -> Result<(Read, Vec<Skipped>), Unread> {
    let path = file.path.as_str();
    let bytes = bytes.map_err(|error| match error.kind() {
>>>>>>> replace
```

- [ ] **Step 4: Run the test to see it pass**

```bash
cargo test --release --locked --test timings
```

Expected: PASS, three tests.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check && cargo clippy --release --all-targets --locked -- -D warnings && cargo test --release --locked
```

Expected: no output from the first two; every test passes.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src tests && git commit -m "feat(timings): report the time of each stage when asked"
```

Expected: one commit.

### Task 2: the measuring script

**Implements:** MEAS-1, MEAS-2, MEAS-3, MEAS-3a, MEAS-4, MEAS-5, MEAS-6

**Test:** `node --test scripts/measure.test.mjs`

**Files:**
- Create: `scripts/lib/measure.mjs`, `scripts/measure.mjs`, `scripts/measure.test.mjs`

**Interfaces:**
- Consumes: Task 1's line on standard error; `BINARY`, `BUILT`, `TARGET` and `fail` from
  `scripts/lib/run.mjs`.
- Produces: `node scripts/measure.mjs <folder> [--exe <path>] [--hand-off <file>] [--cold |
  --drop-caches] [--label <text>] [--json]`, and `node scripts/measure.mjs --combine <file>...
  [--json]`, which Task 3's job and Task 4's measurements run.

- [ ] **Step 1: Write the failing test**

```js path=scripts/measure.test.mjs
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { coldKind, columns, combine, median, STAGES, table, timingLine } from './lib/measure.mjs';

/**
 * A run, as the script records one.
 *
 * @param {string} kind - Its kind.
 * @param {number} wall - Its wall-clock time, in microseconds.
 * @param {number} each - The time of every stage, in microseconds.
 * @returns {import('./lib/measure.mjs').Run} The run.
 */
const run = (kind, wall, each) => {
  const reading = Object.fromEntries(
    STAGES.map((stage) => {
      return [stage, each];
    })
  );

  return { kind, timings: { print: 100, reading, total: each * STAGES.length + 300 }, wall };
};

/**
 * A measured document, as `--json` writes one.
 *
 * @param {import('./lib/measure.mjs').Run[]} runs - Its runs.
 * @param {Partial<import('./lib/measure.mjs').Measured>} [changed] - The fields that differ from the default.
 * @returns {import('./lib/measure.mjs').Measured} The document.
 */
const measured = (runs, changed = {}) => {
  return {
    commit: 'abc',
    dirty: false,
    exeHash: 'hash',
    folder: '/stand-in',
    label: null,
    platform: 'linux-x64',
    runs,
    version: 'yak-analyzer 0.3.0',
    ...changed,
  };
};

test('finds the timing line among the lines on standard error (spec TIME-5)', () => {
  const line = JSON.stringify({ timings: { print: 2, reading: { files: 1 }, total: 9 } });

  assert.deepEqual(timingLine(`something else\r\n${line}\n`), { print: 2, reading: { files: 1 }, total: 9 });
});

test('gives nothing when no timing line came, so the run is not averaged in (spec MEAS-5)', () => {
  assert.equal(timingLine(''), undefined);
  assert.equal(timingLine('yak-analyzer: the file a.ts does not parse\n'), undefined);
});

test('takes the middle value, or the mean of the two middle values', () => {
  assert.equal(median([5, 1, 3]), 3);
  assert.equal(median([4, 1, 3, 2]), 2.5);
});

test("shows the time no stage covers as other, and process start as wall minus total (spec TIME-2, MEAS-3)", () => {
  const shown = columns(run('warm', 5000, 10));

  assert.equal(shown.files, 10);
  assert.equal(shown.print, 100);
  assert.equal(shown.total, 10 * STAGES.length + 300);
  assert.equal(shown.other, 200);
  assert.equal(shown.start, 5000 - (10 * STAGES.length + 300));
  assert.equal(shown.wall, 5000);
});

test('labels the first run cold only on the caller\'s word or with the cache emptied (spec MEAS-4)', () => {
  assert.equal(coldKind({ cold: true, dropCaches: false }), 'cold');
  assert.equal(coldKind({ cold: false, dropCaches: true }), 'cold');
  assert.equal(coldKind({ cold: false, dropCaches: false }), 'unknown');
});

test('pools the runs of documents that agree (spec MEAS-3a)', () => {
  const combined = combine([measured([run('cold', 900, 50)]), measured([run('cold', 800, 40), run('warm', 100, 5)])]);

  assert.equal(combined.runs.length, 3);
  assert.equal(combined.platform, 'linux-x64');
});

test('refuses documents that differ, naming the difference (spec MEAS-3a)', () => {
  for (const field of ['platform', 'exeHash', 'commit', 'dirty', 'label']) {
    const changed = { [field]: field === 'dirty' ? true : 'other' };

    assert.throws(
      () => {
        return combine([measured([]), measured([], changed)]);
      },
      { message: new RegExp(`differ in ${field}`, 'u') },
      field
    );
  }

  assert.throws(
    () => {
      return combine([]);
    },
    { message: /nothing to combine/u }
  );
});

test('fills the cold minus warm column from cold runs only (spec MEAS-3a)', () => {
  const shown = table(measured([run('cold', 9000, 500), run('warm', 1000, 100), run('warm', 1200, 100)]));
  const files = shown.split('\n').find((line) => {
    return line.startsWith('| files |');
  });

  assert.ok(files?.endsWith('| 0.4 |'), files);
});

test('reads none in the cold columns when there are no cold runs, and never pools unknown runs with cold ones', () => {
  const shown = table(measured([run('unknown', 9000, 500), run('warm', 1000, 100)]));
  const files = shown.split('\n').find((line) => {
    return line.startsWith('| files |');
  });

  assert.ok(files?.startsWith('| files | none | 0.5 (0.5 to 0.5) | 0.1 (0.1 to 0.1) |'), files);
  assert.ok(files?.endsWith('| none |'), files);
});
```

- [ ] **Step 2: Run the test to see it fail**

```bash
node --test scripts/measure.test.mjs
```

Expected: FAIL: `scripts/lib/measure.mjs` does not exist.

- [ ] **Step 3: Write the code**

```js path=scripts/lib/measure.mjs
/**
 * What the measuring script reads, works out and prints, kept apart from running anything so that it can be
 * tested on its own (spec MEAS-3, MEAS-3a, MEAS-5, MEAS-6).
 */

/** @typedef {{ print: number, reading: Record<string, number>, total: number }} Timings */

/** @typedef {{ kind: string, timings: Timings, wall: number }} Run */

/**
 * @typedef {{
 *   commit: string,
 *   dirty: boolean,
 *   exeHash: string,
 *   folder: string,
 *   label: string | null,
 *   platform: string,
 *   runs: Run[],
 *   version: string,
 * }} Measured
 */

/** The stages of a reading, in the order the analyzer takes them (spec TIME-2). */
export const STAGES = [
  'handOff',
  'files',
  'marks',
  'packages',
  'tsconfig',
  'scriptsRead',
  'scriptsParse',
  'resolve',
  'entries',
  'build',
];

/** The table's rows, in order: the stages, then what lies around them (spec MEAS-3). */
export const ROWS = [...STAGES, 'print', 'other', 'total', 'start', 'wall'];

/** The kinds of run, in the order the table shows them (spec MEAS-2, MEAS-4). */
const KINDS = ['cold', 'unknown', 'warm', 'first-start'];

/** What the table calls each kind of run. */
const KIND_NAMES = {
  cold: 'cold',
  'first-start': 'first start',
  unknown: 'first run, cache unknown',
  warm: 'warm',
};

/** The fields the documents of one series agree on before their runs are pooled (spec MEAS-3a). */
const AGREED = ['platform', 'exeHash', 'commit', 'dirty', 'label'];

/**
 * Finds the timing line among the lines a run wrote on standard error (spec TIME-5).
 *
 * @param {string} stderr - What the run wrote on standard error.
 * @returns {Timings | undefined} The timings, or nothing when no timing line came.
 */
export const timingLine = (stderr) => {
  for (const line of stderr.split(/\r?\n/u)) {
    if (line.startsWith('{"timings":')) {
      return JSON.parse(line).timings;
    }
  }

  return undefined;
};

/**
 * The kind of the first timed run: cold on the caller's word, or with the file cache emptied first (spec MEAS-4).
 *
 * @param {{ cold: boolean, dropCaches: boolean }} options - The options the script was given.
 * @returns {'cold' | 'unknown'} The kind.
 */
export const coldKind = ({ cold, dropCaches }) => {
  return cold || dropCaches ? 'cold' : 'unknown';
};

/**
 * The middle value, or the mean of the two middle values.
 *
 * @param {readonly number[]} values - The values; at least one.
 * @returns {number} The median.
 */
export const median = (values) => {
  const sorted = [...values].sort((left, right) => {
    return left - right;
  });
  const middle = Math.floor(sorted.length / 2);

  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

/**
 * One run's value for each row of the table, in microseconds. `other` is the time no stage covers, and `start`
 * the time outside the analyzer's own clock: starting the process and ending it (spec TIME-2, MEAS-3).
 *
 * @param {Run} run - The run.
 * @returns {Record<string, number>} Its values, by row.
 */
export const columns = (run) => {
  const { print, reading, total } = run.timings;
  const staged = STAGES.reduce((sum, stage) => {
    return sum + (reading[stage] ?? 0);
  }, 0);

  return {
    ...Object.fromEntries(
      STAGES.map((stage) => {
        return [stage, reading[stage] ?? 0];
      })
    ),
    other: total - staged - print,
    print,
    start: run.wall - total,
    total,
    wall: run.wall,
  };
};

/**
 * The median, lowest and highest of each row over the runs of one kind.
 *
 * @param {readonly Run[]} runs - Every run.
 * @param {string} kind - The kind to sum up.
 * @returns {Record<string, { high: number, low: number, median: number }> | undefined} By row, or nothing when no
 *   run is of that kind.
 */
const summary = (runs, kind) => {
  const chosen = runs
    .filter((run) => {
      return run.kind === kind;
    })
    .map(columns);

  if (chosen.length === 0) {
    return undefined;
  }

  return Object.fromEntries(
    ROWS.map((row) => {
      const values = chosen.map((each) => {
        return each[row];
      });

      return [row, { high: Math.max(...values), low: Math.min(...values), median: median(values) }];
    })
  );
};

/**
 * Pools the runs of documents the script wrote with `--json`. They agree on the platform, the executable, the
 * measured commit, whether it had uncommitted changes, and the label, or nothing is pooled (spec MEAS-3a).
 *
 * @param {readonly Measured[]} documents - The documents.
 * @returns {Measured} One document holding every run.
 * @throws {Error} When there is no document, or two differ.
 */
export const combine = (documents) => {
  const [first] = documents;

  if (first === undefined) {
    throw new Error('nothing to combine: name the files the script wrote with --json');
  }

  for (const document of documents.slice(1)) {
    for (const field of AGREED) {
      if (document[field] !== first[field]) {
        throw new Error(
          `the files differ in ${field} (${String(first[field])} and ${String(document[field])}), so their runs are not pooled`
        );
      }
    }
  }

  return {
    ...first,
    runs: documents.flatMap((document) => {
      return document.runs;
    }),
  };
};

/**
 * A time in milliseconds, to a tenth.
 *
 * @param {number} micros - The time, in microseconds.
 * @returns {string} The time as text.
 */
const ms = (micros) => {
  return (micros / 1000).toFixed(1);
};

/**
 * The table, in Markdown: a row for each stage and for what lies around them, a column for each kind of run that
 * happened and always one for cold runs, and a last column for what a cold run adds to a warm one (spec MEAS-3,
 * MEAS-3a).
 *
 * @param {Measured} document - A measured or combined document.
 * @returns {string} The table, with a line above it saying what was measured.
 */
export const table = (document) => {
  const shown = KINDS.map((kind) => {
    return { kind, summed: summary(document.runs, kind) };
  }).filter(({ kind, summed }) => {
    return summed !== undefined || kind === 'cold';
  });
  const cold = summary(document.runs, 'cold');
  const warm = summary(document.runs, 'warm');
  const head = [
    'stage (ms: median, lowest to highest)',
    ...shown.map(({ kind }) => {
      return KIND_NAMES[kind];
    }),
    'cold minus warm',
  ];
  const changes = document.dirty ? ', with uncommitted changes' : '';
  const label = document.label === null ? '' : `, ${document.label}`;
  const rows = ROWS.map((row) => {
    const cells = shown.map(({ summed }) => {
      if (summed === undefined) {
        return 'none';
      }

      const { high, low, median: middle } = summed[row];

      return `${ms(middle)} (${ms(low)} to ${ms(high)})`;
    });
    const added = cold === undefined || warm === undefined ? 'none' : ms(cold[row].median - warm[row].median);

    return `| ${[row, ...cells, added].join(' | ')} |`;
  });

  return [
    `${document.version} on ${document.platform}, ${document.folder} at ${document.commit}${changes}${label}`,
    '',
    `| ${head.join(' | ')} |`,
    `|${head
      .map(() => {
        return ' --- |';
      })
      .join('')}`,
    ...rows,
  ].join('\n');
};
```

```js path=scripts/measure.mjs
/**
 * Measures how long the analyzer's `graph` takes on a folder, stage by stage: one cold run, ten warm runs and a
 * first start of a fresh copy of the executable (spec MEAS-1 to MEAS-5). Or pools what earlier runs of it wrote
 * with `--json` (spec MEAS-3a).
 *
 *   node scripts/measure.mjs <folder> [--exe <path>] [--hand-off <file>] [--cold | --drop-caches]
 *     [--label <text>] [--json]
 *   node scripts/measure.mjs --combine <file>... [--json]
 *
 * `--cold` says the folder's files are not in the file cache, as on the first run after a restart. On Linux,
 * `--drop-caches` empties the cache first, with passwordless sudo.
 */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { parseArgs } from 'node:util';

import { coldKind, combine, table, timingLine } from './lib/measure.mjs';
import { BINARY, BUILT, fail, TARGET } from './lib/run.mjs';

/** How many warm runs follow the cold one (spec MEAS-2). */
const WARM_RUNS = 10;

/** The variable that turns the analyzer's timings on (spec TIME-1). */
const SWITCH = 'YAK_ANALYZER_TIMINGS';

/** The tiny project the warm-up reads, left untracked, so no commit and no git identity are needed (spec MEAS-2). */
const WARM_UP = {
  'package.json': '{ "name": "warm-up", "main": "src/a.ts" }\n',
  'src/a.ts': "import { b } from './b';\n\nexport const a = b + 1;\n",
  'src/b.ts': "import type { a } from './a';\n\nexport const b = 1;\n",
  'tsconfig.json': '{ "compilerOptions": { "strict": true } }\n',
};

/**
 * Reads the command line, or stops with what was wrong with it.
 *
 * @returns {{ options: Record<string, string | boolean | undefined>, positionals: string[] }} What it holds.
 */
const commandLine = () => {
  try {
    return parseArgs({
      allowPositionals: true,
      options: {
        cold: { default: false, type: 'boolean' },
        combine: { default: false, type: 'boolean' },
        'drop-caches': { default: false, type: 'boolean' },
        exe: { type: 'string' },
        'hand-off': { type: 'string' },
        json: { default: false, type: 'boolean' },
        label: { type: 'string' },
      },
    });
  } catch (error) {
    return fail(`${error.message}; see the head of scripts/measure.mjs for what the script takes`);
  }
};

const { options, positionals } = commandLine();

/**
 * Runs a program to its end.
 *
 * @param {string} program - The program.
 * @param {readonly string[]} args - Its arguments.
 * @param {import('node:child_process').SpawnSyncOptions} [settings] - Where to run it, and what it gets.
 * @returns {import('node:child_process').SpawnSyncReturns<string>} How it ended.
 */
const ran = (program, args, settings = {}) => {
  return spawnSync(program, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, ...settings });
};

/**
 * The first line of what a program that failed wrote, or why it could not start.
 *
 * @param {import('node:child_process').SpawnSyncReturns<string>} ended - How it ended.
 * @returns {string} The line.
 */
const firstLine = (ended) => {
  return (ended.error?.message ?? ended.stderr ?? '').split(/\r?\n/u)[0] || `it ended with exit code ${ended.status}`;
};

/**
 * Prints a measured or combined document: as a table, or with `--json` as one JSON document (spec MEAS-3).
 *
 * @param {import('./lib/measure.mjs').Measured} document - The document.
 */
const show = (document) => {
  console.log(options.json ? JSON.stringify(document) : table(document));
};

/** Pools the files `--combine` names, and prints them (spec MEAS-3a). */
const pooled = () => {
  if (positionals.length === 0) {
    fail('--combine needs the files the script wrote with --json');
  }

  try {
    show(
      combine(
        positionals.map((file) => {
          return JSON.parse(readFileSync(file, 'utf8'));
        })
      )
    );
  } catch (error) {
    fail(error.message);
  }
};

/** Empties the file cache, on Linux, through passwordless sudo, with the script itself an ordinary user (spec MEAS-4). */
const dropCaches = () => {
  if (process.platform !== 'linux') {
    fail('--drop-caches works on Linux only; elsewhere run the script first after a restart, and say so with --cold');
  }

  const dropped = ran('sudo', ['-n', 'sh', '-c', 'sync; echo 3 > /proc/sys/vm/drop_caches']);

  if (dropped.status !== 0) {
    fail(`the file cache could not be emptied, which needs passwordless sudo: ${firstLine(dropped)}`);
  }
};

/**
 * Runs git in a folder, without taking a lock, and gives what it printed.
 *
 * @param {string} folder - The folder.
 * @param {readonly string[]} args - Git's arguments.
 * @returns {string} What it printed, trimmed.
 */
const git = (folder, args) => {
  const ended = ran('git', ['--no-optional-locks', ...args], { cwd: folder });

  if (ended.status !== 0) {
    fail(`git ${args.join(' ')} failed in ${folder}: ${firstLine(ended)}`);
  }

  return ended.stdout.trim();
};

/**
 * Runs `graph` on the warm-up project, untimed, which starts the executable for the first time and loads nearly
 * all the analyzer's and git's code from disk before the cold run (spec MEAS-2).
 *
 * @param {string} exe - The executable.
 * @param {string} temporary - The script's temporary folder.
 * @param {string} fix - What to do when it does not run.
 */
const warmUp = (exe, temporary, fix) => {
  const project = join(temporary, 'warm-up');

  for (const [path, text] of Object.entries(WARM_UP)) {
    mkdirSync(dirname(join(project, path)), { recursive: true });
    writeFileSync(join(project, path), text);
  }

  git(project, ['init', '--quiet']);

  const ended = ran(exe, ['graph'], { cwd: project, env: { ...process.env, [SWITCH]: '' }, stdio: ['ignore', 'ignore', 'pipe'] });

  if (ended.status !== 0) {
    fail(`${exe} does not run: ${firstLine(ended)}${fix}`);
  }
};

/**
 * Runs `graph` once, timed, and records it. A run that fails, or writes no timing line, stops the script, so a
 * broken run is never averaged in (spec MEAS-5).
 *
 * @param {string} exe - The executable.
 * @param {string} folder - The folder measured.
 * @param {readonly string[]} args - The arguments of `graph`.
 * @param {string} kind - The kind of run.
 * @returns {import('./lib/measure.mjs').Run} The run.
 */
const timed = (exe, folder, args, kind) => {
  const started = process.hrtime.bigint();
  const ended = ran(exe, args, { cwd: folder, env: { ...process.env, [SWITCH]: '1' }, stdio: ['ignore', 'ignore', 'pipe'] });
  const wall = Number((process.hrtime.bigint() - started) / 1000n);

  if (ended.status !== 0) {
    fail(firstLine(ended));
  }

  const timings = timingLine(ended.stderr);

  if (timings === undefined) {
    fail(`${exe} wrote no timing line; it may be older than the timing switch, so build it again`);
  }

  return { kind, timings, wall };
};

/** Measures the folder the command line names, and prints what it measured (spec MEAS-1 to MEAS-4). */
const measure = () => {
  if (positionals.length !== 1) {
    fail('name the one folder to measure: node scripts/measure.mjs <folder>');
  }

  const folder = resolve(positionals[0]);
  const exe = resolve(options.exe ?? BUILT);
  const fix = options.exe === undefined ? '; build it with `cargo build --release`' : '';
  const args = ['graph', ...(options['hand-off'] === undefined ? [] : ['--hand-off', resolve(options['hand-off'])])];

  if (!existsSync(exe)) {
    fail(`there is no executable at ${exe}${fix}`);
  }

  if (options['drop-caches']) {
    dropCaches();
  }

  const temporary = mkdtempSync(join(tmpdir(), 'yak-measure-'));

  // The temporary folder goes when the script ends, also when it stops on a failure or is interrupted.
  process.on('exit', () => {
    rmSync(temporary, { force: true, recursive: true });
  });
  process.on('SIGINT', () => {
    process.exit(130);
  });

  warmUp(exe, temporary, fix);

  const runs = [timed(exe, folder, args, coldKind({ cold: options.cold, dropCaches: options['drop-caches'] }))];

  for (let i = 0; i < WARM_RUNS; i += 1) {
    runs.push(timed(exe, folder, args, 'warm'));
  }

  const copy = join(temporary, 'copy', BINARY);

  mkdirSync(dirname(copy));
  copyFileSync(exe, copy);
  runs.push(timed(copy, folder, args, 'first-start'));

  // Read only now, after the cold run, so that git does not warm the folder's index first (spec MEAS-2).
  show({
    commit: git(folder, ['rev-parse', 'HEAD']),
    dirty: git(folder, ['status', '--porcelain']) !== '',
    exeHash: createHash('sha256').update(readFileSync(exe)).digest('hex'),
    folder,
    label: options.label ?? null,
    platform: TARGET,
    runs,
    version: ran(exe, ['--version']).stdout.trim(),
  });
};

if (options.combine) {
  pooled();
} else {
  measure();
}
```

- [ ] **Step 4: Run the test to see it pass**

```bash
node --test scripts/measure.test.mjs
```

Expected: PASS, nine tests.

- [ ] **Step 5: Run the script once on this repository**

```bash
cargo build --release && node scripts/measure.mjs .
```

Expected: a line naming the version, platform and commit, then a Markdown table with a `cold` column
reading `none`, a `first run, cache unknown` column, `warm` and `first start` columns, and a `cold
minus warm` column reading `none`.

- [ ] **Step 6: Run the gates and commit**

```bash
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs" && git add scripts && git commit -m "feat(scripts): measure cold, warm and first-start runs by stage"
```

Expected: every test passes; one commit.

### Task 3: the CI job

**Implements:** GEN-1, GEN-2, JOB-1, JOB-2, JOB-3, JOB-4

**Test:** `node --test scripts/measure-workflow.test.mjs`

**Files:**
- Create: `.github/workflows/measure.yml`, `scripts/measure-workflow.test.mjs`

**Interfaces:**
- Consumes: `node scripts/measure.mjs` from Task 2.
- Produces: the workflow `Measure`, started by hand from the Actions tab, whose summary holds the
  combined table and whose artifact `measured` holds the three rounds' JSON files.

- [ ] **Step 1: Write the failing test**

```js path=scripts/measure-workflow.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The measuring workflow's text, read as text as `workflow.test.mjs` reads `ci.yml`. */
const workflow = readFileSync(join(ROOT, '.github', 'workflows', 'measure.yml'), 'utf8').replaceAll('\r\n', '\n');

test('it runs only when started by hand (spec JOB-1)', () => {
  assert.match(workflow, /\non:\n {2}workflow_dispatch:\n/u);
  assert.doesNotMatch(workflow, /pull_request|\n {2}push:/u);
});

test('it runs straight on the Linux runner ci.yml uses, with no container (spec JOB-1)', () => {
  assert.match(workflow, /\n {4}runs-on: ubuntu-24\.04\n/u);
  assert.doesNotMatch(workflow, /container:/u);
});

test('it measures the stand-in at one commit pinned by its full hash (spec GEN-1, GEN-2)', () => {
  assert.match(workflow, /STAND_IN: mui\/base-ui\n/u);
  assert.match(workflow, /STAND_IN_COMMIT: [0-9a-f]{40}\n/u);
  assert.match(workflow, /fetch --quiet --depth 1 "https:\/\/github\.com\/\$STAND_IN" "\$STAND_IN_COMMIT"/u);
});

test('it builds, measures three rounds with the cache emptied, and combines them (spec JOB-2)', () => {
  assert.ok(workflow.includes('cargo build --release --locked'));
  assert.ok(workflow.includes('for round in 1 2 3; do'));
  assert.ok(workflow.includes('--drop-caches --json > "measured/round-$round.json"'));
  assert.ok(
    workflow.includes(
      'node scripts/measure.mjs --combine measured/round-1.json measured/round-2.json measured/round-3.json >> "$GITHUB_STEP_SUMMARY"'
    )
  );
});

test('it keeps the rounds as an artifact, with the actions pinned as ci.yml pins them (spec JOB-3)', () => {
  const ci = readFileSync(join(ROOT, '.github', 'workflows', 'ci.yml'), 'utf8');

  for (const action of [/actions\/checkout@[0-9a-f]{40}/u, /actions\/setup-node@[0-9a-f]{40}/u, /actions\/upload-artifact@[0-9a-f]{40}/u]) {
    const pinned = workflow.match(action)?.[0];

    assert.ok(pinned !== undefined && ci.includes(pinned), String(action));
  }

  assert.match(workflow, /name: measured\n {10}path: measured\/\*\.json\n/u);
});
```

- [ ] **Step 2: Run the test to see it fail**

```bash
node --test scripts/measure-workflow.test.mjs
```

Expected: FAIL: `.github/workflows/measure.yml` does not exist.

- [ ] **Step 3: Write the workflow**

```yaml path=.github/workflows/measure.yml
name: Measure

# Started by hand from the Actions tab, never on a push or a pull request (spec JOB-1).
on:
  workflow_dispatch:

permissions:
  contents: read

jobs:
  measure:
    name: Measure cold against warm runs
    # Straight on the runner, with no container: emptying the file cache, sudo and an ordinary user all need it
    # (spec JOB-1, MEAS-4).
    runs-on: ubuntu-24.04
    defaults:
      run:
        shell: bash
    env:
      # The stand-in for yakshaver while yakshaver is private: a public TypeScript workspace near its size, pinned
      # to one commit, which graph reads without a hand-off file (spec GEN-1, GEN-2). Once yakshaver is public, it
      # is one more checkout step here (spec JOB-4).
      STAND_IN: mui/base-ui
      STAND_IN_COMMIT: 35fb3007e948750601045d5b88404f5f49646b61
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
      # The runner's own rustup picks the version rust-toolchain.toml names (spec JOB-2).
      - name: Build the release executable
        run: cargo build --release --locked
      - name: Check out the stand-in
        run: |
          git init --quiet "$RUNNER_TEMP/stand-in"
          git -C "$RUNNER_TEMP/stand-in" fetch --quiet --depth 1 "https://github.com/$STAND_IN" "$STAND_IN_COMMIT"
          git -C "$RUNNER_TEMP/stand-in" checkout --quiet FETCH_HEAD
      - name: Measure three rounds, each with its own cold run
        run: |
          mkdir -p measured
          for round in 1 2 3; do
            node scripts/measure.mjs "$RUNNER_TEMP/stand-in" --drop-caches --json > "measured/round-$round.json"
          done
      - name: Combine the rounds into the job's summary
        run: node scripts/measure.mjs --combine measured/round-1.json measured/round-2.json measured/round-3.json >> "$GITHUB_STEP_SUMMARY"
      - uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          name: measured
          path: measured/*.json
          if-no-files-found: error
```

- [ ] **Step 4: Run the test to see it pass**

```bash
node --test scripts/measure-workflow.test.mjs
```

Expected: PASS, five tests.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/measure.yml scripts/measure-workflow.test.mjs && git commit -m "ci(measure): measure cold against warm runs on Linux by hand"
```

Expected: one commit.

### Task 4: the measurements

**Implements:** TAKE-1, TAKE-1a, TAKE-2, TAKE-3, TAKE-4

**Check:** the owner reads the report in the item's report folder: Windows numbers for yakshaver and
one other local repository of similar size, each from at least two cold runs after a restart and one
Defender-excluded round, each combined with `--combine`; the CI job's combined table for the
stand-in; the conclusion, naming the stage that takes most of a cold run on each platform and how
much of Windows' cold cost is Defender's; and the headline numbers on the work item's record.

- [ ] **Step 1: Run the CI job.** Push the branch, start `Measure` from the Actions tab on it
  (`gh workflow run measure.yml --ref feat/cold-warm-performance`), and save its summary and its
  `measured` artifact into the report folder.

```bash
gh workflow run measure.yml --ref feat/cold-warm-performance
```

Expected: a run that ends green; its summary holds one table whose `cold` column isn't `none`.

- [ ] **Step 2: Rehearse on Windows.** With hand-off files for yakshaver and the other repository in
  the report folder, run the script once on each without `--cold`, to check `graph` reads it without
  stopping (TAKE-1).

```bash
node scripts/measure.mjs <repository> --hand-off <report folder>/<name>.hand-off.json
```

Expected: a table, and no stop line.

- [ ] **Step 3: The cold rounds.** Two restarts. After each, before opening an editor or a terminal
  whose prompt runs `git status`, the owner runs the script on each repository with `--cold --json`
  and the same `--hand-off`, saving each round's output in the report folder. The analyzer isn't
  rebuilt and neither repository changes between restarts.

```bash
node scripts/measure.mjs <repository> --hand-off <file> --cold --json > <report folder>/<name>-cold-<n>.json
```

Expected: one JSON document per repository and restart.

- [ ] **Step 4: The Defender-excluded round (TAKE-1a).** With admin rights, add each repository's
  folder to Defender's exclusions (`Add-MpPreference -ExclusionPath <folder>`), restart, run as in
  Step 3 with `--label defender-excluded`, then remove the exclusions
  (`Remove-MpPreference -ExclusionPath <folder>`).

```bash
node scripts/measure.mjs <repository> --hand-off <file> --cold --json --label defender-excluded > <report folder>/<name>-excluded.json
```

Expected: one JSON document per repository; `Get-MpPreference` lists neither folder afterwards.

- [ ] **Step 5: Combine and report (TAKE-3).** Combine each repository's cold rounds, and each one's
  excluded round on its own, with `--combine`. Write the report into the report folder, naming only
  yakshaver among the repositories. Say whether CI's cold median is within a tenth of its warm median.
  Record the headline numbers on the work item's record.

```bash
node scripts/measure.mjs --combine <report folder>/<name>-cold-1.json <report folder>/<name>-cold-2.json
```

Expected: one table per repository and kind of round.
