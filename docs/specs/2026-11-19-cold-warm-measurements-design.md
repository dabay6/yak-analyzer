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

- **A cold run** is a run with the project's files not in the operating system's file cache. **A
  warm run** is the same run again straight after, with them cached.
- **A first start** is the first run of an executable file that has never run, which Windows
  Defender may scan. It is measured separately from a cold run (owner).
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
  through git, `files::list`), `marks`, `packages` (the `package.json` files), `tsconfig`,
  `scriptsRead` (the time spent in `fs::read` of each script, summed over the files) and
  `scriptsParse` (the rest of `scripts::read`: parsing and finding imports, summed the same way),
  `resolve`, `entries`. Reading and parsing are kept apart because which of the two takes a cold
  run's time is the question this item asks. Each is measured with `std::time::Instant`.
  `scripts::read` reads one file after another, so the two sums add up to its wall-clock time.
- **TIME-2a** Two more keys are measured outside the reading: `print` (building and formatting the
  output document, and writing it to standard output) and `total` (from the start of the command in
  `src/main.rs` to the end of printing). So the clock for `print` and `total` lives in `src/main.rs`,
  and `graph::read` hands back its stage times beside the document it returns. A cold run's extra
  time per stage is read as the cold median minus the warm median for that stage (MEAS-3).
- **TIME-3** The line is only written on a run that succeeded. A run that stops still writes
  exactly its one line, as RUN-3 says, and nothing else. This is the one change to RUN-3. It is safe
  because yak reads standard error only when a run fails (design IF-3), and a failed run never has
  the line, even if a user's environment sets the variable and yak passes it on.
- **TIME-4** With the variable set or not, standard output, the exit code, the files read and the
  git commands run are identical.
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
  existing `Project::run` hook in `tests/support/project.rs`, and that file's helper `alone`, which
  shuts out the machine's own settings, also removes `YAK_ANALYZER_TIMINGS`, so a developer's
  environment never changes another test's result.
- **TEST-2** A test runs a stopping `graph` (for example, a missing hand-off file) with the variable
  set, and checks that standard error is the one stop line and nothing else.

### The measuring script

- **MEAS-1** `scripts/measure.mjs` takes the path of the analyzer's executable and a folder, runs
  `graph` there with `YAK_ANALYZER_TIMINGS=1`, and reads the timing line of each run. It uses Node
  alone, as the other scripts do, and the helpers they share in `scripts/lib/run.mjs`: `BUILT` as
  the default executable, and `fail` for stopping.
- **MEAS-2** It takes, in this order: one cold run (MEAS-4), then 10 warm runs, then a first start
  (a copy of the executable in a new temporary folder, outside the folder measured, run once).
  Session's call on the counts.
- **MEAS-3** It prints, for each kind of run and each stage, the median and the lowest and highest
  values, as a table for a person and, with `--json`, as one JSON document. It also prints the
  wall-clock time it measured around each run, so that process start, which no stage covers,
  shows as wall-clock time minus `total`.
- **MEAS-4** The cold run: on Linux, with `--drop-caches`, the script empties the file cache first
  (`sync; echo 3 > /proc/sys/vm/drop_caches`, which needs root). Elsewhere, and without the option,
  it can't empty the cache. With `--cold`, it labels the first run as cold on the caller's word;
  without either option, it labels it "first run, cache unknown". Session's call: the script never
  needs admin rights on Windows.
- **MEAS-5** A run that exits other than 0, or writes no timing line, stops the script with the
  analyzer's own stop line, so a broken run is never averaged in.
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
  push or a pull request, on `ubuntu-latest`. Session's call: Linux is the platform least like the
  owner's Windows machine, and the only one of the CI runners where the file cache can be emptied
  on purpose.
- **JOB-2** It builds the release executable, checks out the stand-in (GEN-1), and runs the script
  with `--drop-caches`, three times over, each with its own cold run. Session's call: three cold
  runs, because a single one says nothing about noise.
- **JOB-3** It writes the script's table to the job's summary and keeps the JSON as an artifact.
- **JOB-4** Once yakshaver is public, measuring it here is one more checkout step. That isn't built
  here.

### The measurements

- **TAKE-1** On the owner's Windows machine: yakshaver and at least one other local repository of
  similar size, each with at least two cold runs, each taken as the first run on that repository
  after a restart of the machine, passed with `--cold`. Session's call: a restart is the only way to
  empty the cache without admin tools.
- **TAKE-2** On Linux: the CI job of JOB-1, run once.
- **TAKE-3** The numbers and the conclusion go into a report in the report folder. It names only
  yakshaver among the repositories; any other is described by its size alone. The headline numbers
  (cold, warm and first start on each platform, and the stage that takes most of a cold run) go on
  the work item's record, where `2026-11-19-match-fallow-speed` will read them.
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
- **macOS as the second platform.** Rejected: emptying its file cache (`purge`) needs root as well,
  and Linux differs more from Windows in how it reads files. Session's call.

## Risks

- **The switch alters what it measures.** Reading the clock costs tens of nanoseconds. It's read a
  dozen times per run, plus twice per script file for `scriptsRead` and `scriptsParse`: about
  6,000 reads on 3,000 files, well under a millisecond. TIME-4 and TEST-1 hold standard output
  identical. The clock is only read when the variable is set.
- **A restart on Windows is not a clean cold run.** Windows may load files at start-up (its
  SuperFetch service), and Defender's state changes. TAKE-1 takes two cold runs for each repository,
  and the report says how far apart they were.
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
- The workflow file's name, and whether it is a new file or a job in `ci.yml` with its own trigger.
  The tests in `scripts/workflow.test.mjs` read `ci.yml`, so a new file avoids touching them.
- The script's option names other than those this spec names, and its table's layout.
