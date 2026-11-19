# Measure cold against warm runs

**Status:** draft.

**Work item:** `2026-11-17-cold-warm-performance`. The comparison with fallow and any tuning were
split off into `2026-11-19-match-fallow-speed`, which waits for `audit` (owner).

**Amends:** `docs/specs/2026-11-11-reading-a-project-design.md`, its RUN-3: a successful run may
write one more line on standard error, and only when the timing variable is set (TIME-3).

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. This item prepares SPEED-3's measurement (the analyzer's own time, split from
what yak and the launcher add) and gives CACHE-1's later decision numbers to start from. It builds
neither requirement.

How to read the marks: **owner** marks what the owner decided in the brainstorm. Everything else
is the **session's call**: the session that wrote this spec decided it, and the owner has not ruled
on it.

## Intent

The first run of yak-analyzer on the yakshaver repository after a rebuild took 33 seconds; the next
runs took about 2. Nobody knew why. Two suspects were the operating system's file cache, which holds
recently read files in memory, being empty, and Windows Defender scanning the newly built
executable on its first start.

yakshaver will often start the analyzer cold, from a commit hook, a gate before a pull request, or
by hand, and all three count (owner). So this item finds out where the time of a cold run goes and
leaves a way of measuring that can be run again, so that later work, the fallow comparison first,
has numbers to start from (owner).

A throwaway test on 2026-11-19 (Windows 11, release build, bare executable, single runs; report in
the work item's report folder) already answered the first question in outline:

| What was timed | First run | Later runs |
|---|---|---|
| a never-run copy of the executable, `--version` | about 205ms | about 70ms |
| a never-run copy, `graph` on yakshaver, its files cached | 1.4 to 1.6s | about 1.2s |
| an executable that had run before, `graph` on a local repository of 2,903 files not read for a while | 11.0s | 0.9s |

A new executable costs 0.15 to 0.4 seconds. An empty file cache costs ten times the warm run. So the
cold cost is the project's files, not the rebuild, and the item stays: the owner's rule was that a
delay felt only after a rebuild would make it unnecessary (owner). What the test could not say is
which stage takes that time: listing the files through git, reading them, or parsing them.

Terms:

- **yakshaver** is the developer tool this analyzer is built for; **yak** is its command. yak
  starts the analyzer, through the **launcher**: the small Node program in
  `npm/yak-analyzer/lib/launch.js` that the npm package installs and that starts the platform's
  executable.
- **fallow** is the dead-code tool yak runs today, which the analyzer is to replace. **`audit`** is
  the analyzer's command for finding dead code, planned by
  `docs/specs/2026-11-13-dead-code-findings-design.md` and not built yet.
- **The hand-off file** is the file yak writes to tell the analyzer which folder to read and what a
  project declares (`graph --hand-off <file>`; see CONTRIBUTING.md, "The commands").
- **The owner** is the person who decides this work; **the brainstorm** is the conversation, between
  the owner and a session, that this spec was written from; **the builder** is whoever plans and
  builds from this spec. **The work item's record** is the log of decisions and notes yakshaver
  keeps for each work item, outside this repository.
- Design IDs, such as SPEED-3, CACHE-1 or IF-3, are requirements of the design spec. SPEED-3 says
  the analyzer's speed is measured on the owner's Windows machine, with its own time split from what
  yak and the launcher add. CACHE-1 says a cache between runs is built only if that measurement
  calls for one. IF-3 says yak reads the analyzer's standard error only when a run fails, and relays
  its first line. IDs given with "dead-code" before them (INH-4, LIVE-5) are
  `docs/specs/2026-11-13-dead-code-findings-design.md`'s; RUN-3 and RUN-6a, which have nothing before
  them, are `docs/specs/2026-11-11-reading-a-project-design.md`'s. RUN-6a fixes how git is started:
  no locks, no file-system monitor, no untracked cache.
- **A cold run** is a run with the project's files not in the operating system's file cache. **A
  warm run** is the same run again straight after, with them cached.
- **A first start** is the first run of an executable file that has never run, which Windows
  Defender may scan. It is measured separately from a cold run (owner). Linux has no such scan, so
  a first start there is expected to cost nothing extra; it is measured anyway, the same way, so the
  two platforms' tables match.
- **A stage** is one step of `graph`, in the order `src/graph.rs` takes them (TIME-2).
- **The stand-in** is a public open-source repository the CI job measures while yakshaver is private
  (GEN-1).
- **The report folder** is the work item's `yak-reports/<item id>` folder in git's common
  directory, on the owner's machine and committed nowhere.

## Goals, non-goals and anti-goals

**Goals:**

- The time of a cold run, of a warm run and of a first start, each split by stage, on Windows and
  on Linux (owner: a CI runner for the second platform).
- A way to take those numbers again that stays in the repository, for now (owner): a switch in the
  analyzer and a script.

**Non-goals:**

- Comparing with fallow, and making anything faster: both are `2026-11-19-match-fallow-speed`'s,
  once `audit` exists (owner).
- Timing `audit`. It doesn't exist yet. The switch is built so that `audit`'s stages can be added
  to it later (TIME-6), and the dead-code item's LIVE-5 can use it.
- Timing the npm launcher, or what yak adds around the analyzer. SPEED-3 splits those off, and the
  fallow item measures them.
- Measuring yakshaver on CI. It is private today. It will be public soon (owner), and then it is
  one more checkout step in the job (JOB-4).

**Anti-goals,** failures even if it technically works:

- The switch changes anything the analyzer prints, or how it exits, when it is not set.
- The switch changes what the analyzer reads or how it starts git (RUN-6a), when it is set.
- Numbers that mix a first start with a cold run, so that one could pass for the other.

## Requirements

### The timing switch

- **TIME-1** When the environment variable `YAK_ANALYZER_TIMINGS` is set to `1`, a `graph` run
  that succeeds writes one line on standard error after its output: a JSON object with the
  microseconds each stage took, as whole numbers, and the total. Microseconds, because a warm stage
  can take well under a millisecond. Set to anything else, or unset, the analyzer behaves as it does
  today. Session's call: an environment variable, because the command line accepts only exact forms
  (`src/cli.rs`) and yak's interface has no room for a new option.
- **TIME-2** The stages of the reading, in the order `graph::read` takes them, and their keys:
  `handOff` (reading and checking the hand-off file; 0 without one), `files` (listing the files
  through git, `files::list`; on a cold run this includes git reading its index and the repository's
  folders from disk, though not loading git's code, which MEAS-2's warm-up loads first), `marks` (applying
  the hand-off file's `data` and `generated` declarations to the file list, `marks::apply`; near 0
  without a hand-off file), `packages` (the `package.json` files), `tsconfig`,
  `scriptsRead` (the time spent in `fs::read` of each script, summed over the files) and
  `scriptsParse` (for each file, the time from `fs::read` returning to the end of that file's
  parsing and finding of imports, summed over the files),
  `resolve`, `entries`, and `build` (building the output document and formatting it as text, which
  `graph::read` does before it returns). Reading and parsing are kept apart because which of the two takes a cold
  run's time is the question this item asks. Each is measured with `std::time::Instant`.
  `scripts::read` reads one file after another, so the two sums add up to nearly all of its
  wall-clock time. The small steps between stages (the loop's own work, `env::current_dir`,
  combining two faults) fall into no stage; the script shows them as `other`, which is `total` minus
  the sum of every stage and `print` (MEAS-3).
- **TIME-2a** Two more keys are measured outside the reading: `print` (writing the formatted
  document to standard output) and `total` (from the first line of `run` in `src/main.rs`, before
  the arguments are parsed, to the end of printing). So the clock for `print` and `total` lives in `src/main.rs`, and `graph::read`
  hands back its stage times beside the document it returns.
- **TIME-3** The line is only written on a run that succeeded. A run that stops still writes
  exactly its one line, as RUN-3 says, and nothing else. This is the one change to RUN-3. It is safe
  because yak reads standard error only when a run fails (design IF-3), and a failed run never has
  the line, even if a user's environment sets the variable and yak passes it on.
- **TIME-4** With the variable set or not, standard output, the exit code, the files read and the
  git commands run are identical. TEST-1 checks standard output and the exit code. A reviewer checks
  the files read and the git commands in the code: the clocks wrap the existing calls and never
  change their arguments.
- **TIME-4a** The timing line is written with `writeln!` to standard error, and a failure to write
  it is ignored, so a closed standard error never turns a run that succeeded into a fault
  (`eprintln!` would panic, and the run would exit 3).
- **TIME-5** The line's shape is `{"timings":{"reading":{<the stages of TIME-2>},"print":n,"total":n}}`.
  The outer key tells it apart from a stop line, which begins with `yak-analyzer:`.
- **TIME-6** The stages of a reading are grouped under one key so that `audit` can add a second
  reading, of the base, under a key of its own, timed separately even if the two run in parallel
  (dead-code INH-4, LIVE-5). That is not built here.

### Tests of the switch

- **TEST-1** A test in `tests/` runs `graph` on a made-up fixture project with the variable set and
  unset. It checks that standard output and the exit code are the same both times, that standard
  error is empty when the variable is unset, and that when it's set, standard error is one line that
  parses as JSON with every key of TIME-2, TIME-2a and TIME-5. It sets the variable through the
  existing `Project::run` hook in `tests/support/project.rs`.
- **TEST-1a** Every place a test starts the binary removes `YAK_ANALYZER_TIMINGS` first, unless the
  test sets it: the helper `alone` in `tests/support/project.rs`, `run` in `tests/support/mod.rs`,
  and the commands `tests/cli.test.rs` builds itself. Otherwise a developer with the variable set
  would see the tests that require an empty standard error fail.
- **TEST-2** A test runs a stopping `graph` (for example, a missing hand-off file) with the variable
  set, and checks that standard error is the one stop line and nothing else.

### The measuring script

- **MEAS-1** `scripts/measure.mjs` takes a folder as its one argument, runs `graph` there with
  `YAK_ANALYZER_TIMINGS=1`, and reads the timing line of each run. `--exe <path>` names the
  executable; without it, the script uses `BUILT` from `scripts/lib/run.mjs`, the release build. It
  uses Node alone, as the other scripts do, and `fail` from the same file for stopping. It passes no
  hand-off file, so `handOff` is always 0: the stages this item asks about (listing, reading,
  parsing) read the same files with a hand-off file or without one. Session's call.
- **MEAS-2** It takes, in this order: on Linux with `--drop-caches`, the emptying of the file cache
  (MEAS-4); then a warm-up: one untimed `graph` run on a tiny project the script writes into a new
  temporary folder (a git repository holding a `package.json`, a `tsconfig.json` and two
  TypeScript files that import each other). That run goes through every stage, so it loads nearly
  all the analyzer's and git's code from disk, and it starts the executable for the first time. The
  cold run that follows carries the project's files and git's reading of the repository, and
  neither a first start nor the loading of program code. Then: one cold run;
  10 warm runs; then a first start
  (a copy of the executable in a new temporary folder, outside the folder measured, timed on its
  first `graph` run, with the files cached). Session's call on the counts.
- **MEAS-3** It prints, for each kind of run and each stage, the median and the lowest and highest
  values, as a table for a person and, with `--json`, as one JSON document. It also prints the
  wall-clock time it measured around each run, so that process start, which no stage covers,
  shows as wall-clock time minus `total`, and `other` as TIME-2 defines it. The JSON document holds
  every run (its kind, its wall-clock time and its timing line), the platform, the SHA-256 hash of
  the executable's file (two builds of one version differ there, which `--version` can't show), the
  folder measured, that folder's commit hash, and whether it had uncommitted changes (from
  `git status --porcelain`). Its exact layout is the builder's.
- **MEAS-3a** With `--combine <file> <file>...`, the script reads JSON documents that earlier runs
  of it wrote (`--json`) and prints the same table over all their runs together. Each kind of run is
  pooled across the files: the cold runs of three rounds give a median of three. A run labeled
  "first run, cache unknown" is never pooled with cold runs; it stays a kind of its own. The files
  must agree on the platform, the executable's hash, the measured folder's commit and whether it
  had uncommitted changes; when they
  don't, the script stops with a line naming the difference, so Linux and Windows numbers are never
  pooled. With no cold runs among the files, the cold columns read "none". It adds one more
  column per stage: the cold median minus the warm median, which is how much of a cold run's time
  that stage adds. The CI job's rounds and the Windows restarts are combined this way. Session's
  call: one way of combining for both platforms, since the Windows cold runs come from separate
  invocations, one after each restart.
- **MEAS-4** The cold run: on Linux, with `--drop-caches`, the script empties the file cache first
  by running `sudo -n sh -c 'sync; echo 3 > /proc/sys/vm/drop_caches'` as a child process. The
  script itself runs as the ordinary user, so git never sees a checkout owned by someone else (which
  it refuses as "dubious ownership"); `-n` makes sudo fail at once rather than ask for a password,
  and a failure stops the script with a line saying passwordless sudo is needed. GitHub's Linux
  runners allow passwordless sudo. `--drop-caches` on any other platform stops the script with a
  line saying it works on Linux only. With `--cold`, the script labels the cold run as cold on the
  caller's word; `--drop-caches` implies it. Without either option, it labels the run "first run,
  cache unknown". Session's call: the script never needs admin rights on Windows.
- **MEAS-5** A broken run is never averaged in. A run that exits other than 0 stops the script,
  through `fail`, with the analyzer's own stop line. A run that exits 0 but writes no timing line
  stops it with a line of the script's own saying that no timing line came, and that the
  executable may be older than the switch. The script needs a built executable: when the one it
  would use is missing, or doesn't run, it stops with a line that names the path, and, when that
  path is the default `BUILT`, gives `cargo build --release` as the fix.
- **MEAS-6** The script's own logic (reading the timing lines, the medians, the labels) has tests
  in `scripts/`, run by the gates' `node --test` command.

### The stand-in repository

- **GEN-1** On CI, the analyzer measures a public open-source repository, the stand-in, checked out
  at one commit that the workflow pins by its full hash, so runs on different days compare. Nothing
  from it is copied into this repository; it is only read, as the analyzer reads any project.
  Session's call, over writing a made-up project: a real repository is more realistic and needs no
  generator, the largest piece of new code the first draft had.
- **GEN-2** The builder picks the stand-in: a TypeScript workspace of several packages with a
  `tsconfig.json`, of 2,000 to 4,000 tracked files (yakshaver has 2,796 today), that `graph` reads
  without stopping. The report names it and its commit.

### The CI job

- **JOB-1** A workflow that runs only when it is started by hand (`workflow_dispatch`), never on a
  push or a pull request, directly on the `ubuntu-24.04` runner, with no container. `ci.yml`'s Linux
  `target` job builds inside a manylinux container, where emptying the cache, `sudo` and the
  ordinary user of MEAS-4 don't work, so this job doesn't copy that setup. Session's call: Linux is
  the platform least like the owner's Windows machine, and its file cache can be emptied without
  restarting.
- **JOB-2** It uses the runner's own rustup, which picks the version `rust-toolchain.toml` names,
  and `actions/setup-node`, pinned to the same commit as in `ci.yml`. It builds with
  `cargo build --release --locked`, checks out the stand-in (GEN-1), and runs the script
  with `--drop-caches --json`, three times over, each with its own cold run, then combines the
  three with `--combine` (MEAS-3a). Session's call: three cold runs, because a single one says
  nothing about noise.
- **JOB-3** It appends the combined table to the file `$GITHUB_STEP_SUMMARY` names, which GitHub
  shows as the job's summary, and keeps the JSON documents with `actions/upload-artifact`.
- **JOB-4** Once yakshaver is public, measuring it here is one more checkout step. That isn't built
  here.

### The measurements

- **TAKE-1** On the owner's Windows machine: yakshaver and at least one other local repository of
  similar size, each with at least two cold runs, each taken as the first run on that repository
  after a restart of the machine, passed with `--cold --json`. After the restart, the script is the
  first thing run, before an editor, a terminal whose prompt runs `git status`, or anything else
  opens either repository, and neither repository changes between restarts. One restart may serve
  both repositories, since running the script on one doesn't read the other's files. Each repository's runs are then
  combined with `--combine` (MEAS-3a). Session's call: a restart is the only way to empty the cache
  without admin tools.
- **TAKE-2** On Linux: the CI job of JOB-1, run once.
- **TAKE-3** The numbers and the conclusion go into a report in the report folder. It names only
  yakshaver among the repositories; any other is described by its size alone. The headline numbers
  (cold, warm and first start on each platform, and the stage that takes most of a cold run) go on
  the work item's record, where `2026-11-19-match-fallow-speed` will read them. If CI's cold median
  is within a tenth of its warm median, the report says emptying the cache had no visible effect
  there, and CI's stage split isn't used for conclusions about cold runs.
- **TAKE-4** The item is done when the switch, the script and the job have passed the gates and
  TAKE-1 to TAKE-3 are done. Whatever the numbers say, nothing is made faster here (owner).

## Alternatives considered

- **Wait for `audit`, and measure nothing now.** Rejected by the owner: numbers from today give
  later runs something to compare against, and the throwaway test showed the cold cost is real.
- **A one-time report, with stage times from a profiler.** No product code changes. Rejected by the
  owner: the fallow item would have to work out how to measure all over again, and its numbers
  might not compare.
- **A new command-line option, such as `--timings`.** Rejected: the command line accepts only exact
  forms, and yak's interface has no room for an option yak never passes. An environment variable
  changes neither. Session's call.
- **Writing the timings to a file.** Rejected: the analyzer never writes (CONTRIBUTING.md, "It only
  reads"), and a path outside the checkout would be the first exception. Session's call.
- **Measuring the whole run from outside only, with no switch.** Rejected: the throwaway test did
  that, and it couldn't say which stage takes the cold run's time, which is the question the fallow
  item needs answered.
- **A made-up project the script writes, about 3,000 files, the same every time.** Fits the rule
  that tests use made-up projects. Rejected: it is the largest piece of new code here, needed only
  until yakshaver is public, and less like a real repository than a real one pinned to a commit.
  This isn't a test, so the made-up-projects rule doesn't bind it. Session's call.
- **macOS as the second platform.** Its runners also allow passwordless sudo, and `purge` empties
  its file cache. Rejected: the owner asked for one platform beyond Windows, and Linux, with a
  different file system and a different way of caching files, is further from Windows than macOS
  is. Session's call.
- **A Windows CI runner.** Its runners run as administrator and could empty the cache. Rejected:
  Windows is already measured on the owner's own machine, which is what SPEED-3 names.

## Risks

- **The switch alters what it measures.** Reading the clock costs tens of nanoseconds. It's read a
  dozen times per run, plus twice per script file for `scriptsRead` and `scriptsParse`: about
  6,000 reads on 3,000 files, well under a millisecond. TIME-4 and TEST-1 hold standard output
  identical. The clock is only read when the variable is set.
- **A restart on Windows is not a clean cold run.** Windows may load files at start-up (its
  SuperFetch service), and Defender's state changes. Things that start at sign-in can read a
  repository before the script does: an editor reopening it, a shell prompt running `git status`,
  the Windows Search indexer. TAKE-1 has the script run first, takes two cold runs for each
  repository, and the report says how far apart they were.
- **On CI, emptying the cache may not make reads cold.** The runner is a virtual machine; the host
  or the storage beneath it may cache what the virtual machine reads, so `drop_caches` may make
  reads only partly cold. TAKE-3 checks the cold runs against the warm ones, and says so when there
  is no visible difference. The owner's Windows numbers are the ones SPEED-3 names, and they don't
  rely on CI.
- **The warm-up doesn't load every line of code.** The tiny project of MEAS-2 doesn't use every
  path the parser and resolver have, so a little code may still be loaded during the cold run. The
  code is about 4MB, which is small beside 3,000 project files.
- **The CI runner's numbers are not the owner's machine's.** They are reported as their own
  platform, never mixed with Windows. Their use is the split between stages, not the totals.
- **Noise.** One cold run per round, three rounds on CI, two on Windows: enough to see whether the
  cold cost is seconds or milliseconds, which is the question, not to settle a 100ms difference.
  The fallow item's 100 to 200ms band needs its own run counts, and they are that item's.
- **A first start that hides inside a cold run.** MEAS-2 takes the first start with a fresh copy
  after the warm runs, with the files cached, so it carries only the executable's own cost.
- **The stand-in isn't yakshaver.** TAKE-1 measures yakshaver itself on Windows; the stand-in
  only fills in on CI until yakshaver is public. A stand-in that changes upstream doesn't matter,
  because it's pinned to one commit (GEN-1).

## Left to the builder

- How the stage clocks are threaded through `graph::read`: a small struct of `Instant`s, or a list,
  as reads best.
- Which public repository is the stand-in, within GEN-2.
- The workflow file's name. It is a new file in `.github/workflows/`, because a trigger belongs to a
  whole workflow, and `ci.yml` runs on every push; a new file also leaves the tests in
  `scripts/workflow.test.mjs`, which read `ci.yml`, untouched.
- The script's option names other than those this spec names, and its table's layout.
