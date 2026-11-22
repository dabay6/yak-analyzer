# Read the working copy's files in parallel, to narrow audit's cold gap to fallow

**Status:** draft.

**Work item:** `2026-11-19-match-fallow-speed`, split from `2026-11-17-cold-warm-performance`.

**Amends:** `docs/specs/2026-11-19-cold-warm-measurements-design.md`, the meaning of the timing
keys `scriptsRead` and `scriptsParse` in its TIME-2 and TIME-2b, and TIME-2's sentence that
`scripts::read` reads one file after another (CLOCK-1 here).

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. This item serves its SPEED-1 and SPEED-2 (the audit's time after an edit and
at the gate) on the first run after a restart, and gives SPEED-3 and SPEED-4's comparison with
fallow its first side-by-side numbers. It builds none of them whole, and builds no cache
(design CACHE-1).

How to read the marks: **owner** marks what the owner decided in the brainstorm. Everything else is
the **session's call**: the session that wrote this spec decided it, and the owner has not ruled on
it. The owner was asked about the build choices together and answered "your calls feel right".

## Intent

yak-analyzer is meant to replace fallow, the dead-code tool yak runs today, so its speed is judged
against fallow's, measured the same way, on the yakshaver repository (owner). A throwaway check on
2026-11-22 timed both side by side: yak-analyzer 0.10.0's `audit --hand-off <file>` against fallow
3.39.0's `fallow dead-code --format json`, both as bare executables with their output thrown away,
timed by the wall clock from outside. The check's scripts and results are in the work item's report
folder.

| Run | yak-analyzer `audit` | fallow `dead-code` |
|---|---|---|
| warm, median of ten alternating runs, four rounds | 1.10 to 1.49s | 1.40 to 1.79s (2.18s without its cache) |
| first run after a restart, that tool going first | 26.5s | 5.6s (5.7s without its cache) |

Warm, yak-analyzer is already as fast or faster. Cold it is about 21 seconds slower. By the owner's
bands (below), a gap of a second or more is investigated now (owner). fallow's cache saves nothing
on a cold run, so the gap is in how the files are read. On Windows, Microsoft Defender scans each
file the first time it is opened after a restart; the earlier item found that reading the script
files (`scriptsRead`) is 97% of a cold `graph` run, and yak-analyzer reads them one after another.

A second check read yakshaver's script files cold after one restart, in four equal shares, each with
a different number of threads: 1 thread 8.4ms a file, 4 threads 2.3ms, 8 threads 1.3ms, 16 threads
0.9ms. Defender's scans overlap when files are opened in parallel. At 16 threads all of yakshaver's
2,744 script files would read cold in about 2.4 seconds, against about 23 one at a time.

So this item reads the working copy's script files on several threads at once, leaving everything
else about the reading as it is, and then measures the gap to fallow again.

Terms:

- **yakshaver** is the developer tool this analyzer is built for; **yak** is its command. **fallow**
  is the dead-code tool yak runs today. **`audit`** is the analyzer's dead-code command;
  **`graph`** prints what a reading found (CONTRIBUTING.md, "The commands").
- **The hand-off file** is the file yak writes to tell the analyzer which folder to read and what
  the project declares.
- **The owner** decides this work; **the brainstorm** is the conversation this spec was written
  from; **the builder** plans and builds from it. **The work item's record** is the log of decisions
  yakshaver keeps for each work item, outside this repository. **The report folder** is the work
  item's `yak-reports/2026-11-19-match-fallow-speed` folder in git's common directory, on the
  owner's machine and committed nowhere.
- **A cold run** is a run with the project's files not in the operating system's file cache, here
  always the first run on the repository after a restart of the machine, with Defender's real-time
  scanning on. **A warm run** is a run straight after another, with the files cached.
- **The working copy** is the project's files on disk (`Tree::working` in `src/tree.rs`). **The base**
  is a commit `audit --base` also reads, through one `git cat-file` process behind a lock
  (`Tree::base`).
- **The script files** are the listed files `scripts::read` reads: the kinds `scripts::reads`
  accepts (scripts, type declaration files, Vue components, MDX documents and HTML pages), less those
  declared as data.
- **The pre-read** is the new step of PAR-1: reading the script files' bytes on several threads,
  before the existing loop.
- **The owner's bands** say what a gap to fallow leads to, judged on cold and warm runs separately
  (owner; applying them to cold runs too is the session's call, which the owner did not change).
  yak-analyzer faster, or slower by under 100ms: the goal, met. 100 to 300ms slower: acceptable, no
  investigation now. Over 300ms and under 1000ms: wait for yakshaver's Rust linting support, measure
  again, and investigate if the gap remains. 1000ms or more: investigate now. The owner noted that
  no fixed number captures this perfectly: a gap matters more when fallow itself is fast.
- IDs given with "reading" before them (RUN-4, PARSE-2, LIST-3) are
  `docs/specs/2026-11-11-reading-a-project-design.md`'s; with "dead-code" before them (LIFE-3,
  COMMIT-4, INH-4) `docs/specs/2026-11-13-dead-code-findings-design.md`'s; TIME-, MEAS- and TAKE-
  IDs are `docs/specs/2026-11-19-cold-warm-measurements-design.md`'s.

## Goals, non-goals and anti-goals

**Goals:**

- `audit`'s and `graph`'s cold run on Windows close to fallow's, by reading the working copy's
  script files in parallel (owner: parallel reading is the direction).
- The gap measured again the same way as the throwaway check, and judged by the owner's bands
  (owner).

**Non-goals:**

- A cache between runs (design CACHE-1). fallow's cache saves nothing cold.
- Reading the base in parallel. Its reads go through one git process behind a lock, so threads
  would queue on it, and git's first failure would stop the run on whichever file failed first in
  time, not in list order (dead-code COMMIT-4).
- Parsing on several threads. The reading stays a cold-disk problem; parsing is not where the cold
  time goes.
- Reading `package.json` and tsconfig files, or git's listing, in parallel. Session's call: they are
  a small share of a cold run, and their order decides which of two faults stops a run (reading
  RUN-4). If the measurement shows they matter, that is a later item.
- Linux and macOS. They have no Defender scan; the change runs there too, and must keep every
  result, but nothing is measured there.
- Timing `audit` with the timing switch. It stays as dead-code TIME-6a says: no timing line.
- Changing the warm run. It already meets the goal.

**Anti-goals,** failures even if it technically works:

- Any change to what the analyzer prints or how it exits, on any project, whatever the number of
  threads: findings, their order, the notices and their order, the set of gone files, and which
  broken file's line stops a run when two are broken.
- A new option, setting or environment variable for the number of threads.
- The analyzer writing anything, or starting any process it doesn't start today.

## Requirements

### The pre-read

- **PAR-1** On the working copy, before its loop, `scripts::read` reads the bytes of every script
  file on several threads at once, and keeps each file's result (its bytes, or the `io::Error` the
  read gave) at that file's position in the file list. The loop then runs as today, in list order,
  and takes each file's bytes from the pre-read instead of calling `Tree::read`. Everything the loop
  does with the bytes is unchanged: the same parsing, the same notices, the same `gone` set, and the
  same first tracked broken file stopping the run (reading PARSE-2, LIST-3, RUN-4). So the output is
  the same as today's by construction. Session's call.
- **PAR-2** The pre-read runs only on the working copy (`!tree.is_base()`). On the base the loop
  reads each file through `Tree::read` as today. Session's call.
- **PAR-3** A file the pre-read holds no result for is read in the loop through `Tree::read`, as
  today. This covers the base, and a thread that could not be started: when starting a thread fails,
  the pre-read goes on with the threads it has, or with none, and the run's result is unchanged.
  Session's call: a run that would succeed today never stops because a thread could not start.
- **PAR-4** The number of threads is `std::thread::available_parallelism()`, or 1 when that call
  fails, and never more than the number of script files. With one thread or fewer than two script
  files, no thread is started and the loop reads as today. There is no setting for it (owner
  approved). On the owner's 16-thread machine this is 16, which the second check measured.
- **PAR-5** The threads are `std::thread::scope` threads from the standard library, with no new
  crate. They only read bytes; no thread but the reading's own parses, so they need no larger stack
  (dead-code LIFE-3). Session's call.
- **PAR-6** Only the bytes of files the loop would read are read: a data file, or a file of a kind
  `scripts::reads` refuses, is never opened, as today.
- **PAR-7** `audit --base` keeps reading the working copy and the base at once, on two threads
  (dead-code INH-4). The working copy's reading starts its own pre-read; the base's doesn't, by
  PAR-2. So one run never has two pre-reads.

### The timing switch

- **CLOCK-1** `scriptsRead` becomes the wall-clock time of the pre-read, plus each `Tree::read` the
  loop still makes (PAR-3), summed. `scriptsParse` stays each file's time in the loop after its bytes
  are in hand, summed over the files. This replaces TIME-2's and TIME-2b's meaning of the two keys,
  and TIME-2's sentence that `scripts::read` reads one file after another. The two keys still add up
  to nearly all of `scripts::read`'s wall-clock time, so `other` (MEAS-3) keeps its meaning. The
  line's shape and keys don't change (TIME-5). Session's call.
- **CLOCK-2** Rounds of `scripts/measure.mjs` taken before and after this change are never pooled:
  `--combine` already refuses files whose executables' hashes differ (MEAS-3a). Nothing in the
  script changes.

### Tests

- **PTEST-1** A unit test of the pre-read runs it with 1, 2, 3 and 8 threads over a made-up list of
  more files than threads, with a data file, a file of a kind not read, a missing file and a file
  that cannot be read among them, and checks that every run gives the same results at the same
  positions as reading the files one after another. So the pre-read takes its thread count as an
  argument, and `scripts::read` passes PAR-4's.
- **PTEST-2** The existing test that two broken tracked scripts stop the run on the first by path
  (`stops_on_a_tracked_script_that_does_not_parse_the_first_by_path` in `tests/imports.test.rs`)
  passes unchanged. A new fixture test does the same with a tracked script that is not valid UTF-8
  listed after one that does not parse, and checks the stop line names the first.
- **PTEST-3** Every existing test passes unchanged, its written-down output included. The fixtures
  have fewer files than the threads, so a test of a fixture of at least 64 script files, importing
  one another in a chain, checks that `graph`'s output lists every file and import, in order, as
  written in the test.

### The comparison with fallow

- **CMP-1** Once built, the gap is measured as the throwaway check measured it (owner): on the
  owner's Windows machine, on yakshaver, the release build of this branch's analyzer run as a bare
  executable, `audit --hand-off <file>` with the hand-off file the check used, against fallow 3.39.0's
  bare executable, `fallow dead-code --format json`, with yakshaver's own fallow configuration; both
  with their output thrown away and timed by the wall clock from outside the process.
- **CMP-2** Warm: ten alternating runs of each, fallow with its cache, in two rounds; the medians are
  compared.
- **CMP-3** Cold: two restarts, one with each tool going first, each tool's cold time being its time
  when it goes first, since the first to run warms the files for the other. After a restart the
  check is the first thing run, before an editor, a terminal whose prompt runs `git status`, or
  anything else opens yakshaver, and yakshaver doesn't change between the restarts. fallow's earlier
  cold time (5.6s) may stand in for a fresh one only if fallow, yakshaver's commit and the machine's
  Defender settings haven't changed; otherwise both are taken again. Session's call.
- **CMP-4** When a cold gap lies within 300ms of a band's edge, one more pair of restarts is taken
  and the gap is the median of the cold times. Session's call: one cold run spread by about 260ms
  between restarts in the throwaway check, more than the 100ms band.
- **CMP-5** The spike's scripts in the report folder are reused, and stay there: they run fallow,
  which this repository doesn't depend on, and their numbers are this item's. Session's call.
- **CMP-6** The numbers and the band each gap falls in go into a report in the report folder, and
  the headline (cold and warm gaps, and their bands) on the work item's record. The report says that
  fallow reads more files than `audit` does, so the two cold times aren't strictly like for like.

### Done

- **DONE-1** The item is done when PAR-1 to PTEST-3 have passed the gates and CMP-1 to CMP-6 are
  done. The band decides what follows, never more work in this item: a cold or warm gap of 300 to
  1000ms leaves a queued work item that waits for yakshaver's Rust linting support; a gap of 1000ms
  or more opens a work item to investigate it now, with priority `now`. Session's call: this item
  builds one change and measures it; investigating a remaining gap is new work with its own spec.

## Alternatives considered

- **Parse on the threads too, each thread reading and parsing its share, the results merged in list
  order.** Faster warm, and less memory held at once. Rejected: the warm run already meets the goal;
  parsing threads need dead-code LIFE-3's 256 MiB stacks; and the loop's notices, stop and `gone`
  set would be rebuilt from merged results instead of kept by construction. Session's call.
- **A bounded pipeline: threads read ahead a fixed window of files, the loop parses each as it
  arrives.** Holds only the window in memory. Rejected for now: more code (a channel and its order)
  for a saving that matters only on far larger projects than yakshaver (Risks). Session's call.
- **A thread-pool crate, such as rayon.** Rejected: the standard library's scoped threads do this
  in a few lines, and every dependency is pinned and watched (CONTRIBUTING.md). Session's call.
- **A fixed number of threads, such as 16, whatever the machine.** Defender's scans are waits, not
  computing, so more threads than cores might still help. Rejected: nothing has measured more
  threads than cores, and the owner's machine gives 16 anyway. Session's call.
- **A cache between runs (design CACHE-1).** Rejected: fallow's cache saved nothing cold (5.6s with,
  5.7s without), so the gap is not about remembering results.
- **Asking users to exclude their checkouts from Defender.** Rejected: fallow is fast cold without
  that, so the analyzer should be too, and an exclusion weakens the machine's protection.
- **Adding the comparison with fallow to `scripts/measure.mjs`.** Rejected: it would make this
  repository's scripts run fallow, for a comparison taken once. Session's call.

## Risks

- **Memory.** The pre-read holds every script file's bytes at once, where today one file's bytes
  are dropped after it is parsed. On yakshaver that is about 9MB. A project of 50,000 script files
  averaging 5KB would hold about 250MB for the reading's length. Accepted for now; the bounded
  pipeline above is the way out if a measured project needs it.
- **More work on a run that stops.** Today a broken tracked file stops the reading of later files;
  now they are all read first. The output is the same; only a stopping run takes longer, by at most
  a warm read of the rest.
- **Order depends on the threads.** PAR-1 keeps each result at its file's list position, and the
  loop alone decides what happens, so the order of reading never reaches the output. PTEST-1 runs
  several thread counts against the one-at-a-time result.
- **A file changes between the pre-read and the loop.** The pre-read reads it slightly earlier than
  today's loop would; a file deleted or edited during the run gives one of the two answers it could
  give today. Nothing new.
- **Fewer cores, smaller gain.** A machine with 4 threads would read cold at about the second
  check's 4-thread speed (2.3ms a file, about 6s on yakshaver), still far better than today. Only
  the owner's machine is measured (design SPEED-3).
- **Parallel reading isn't enough.** The pre-read covers what the earlier item found to be 97% of a
  cold `graph` run; git's listing and the `package.json` and tsconfig reads remain one at a time.
  DONE-1 sends a remaining gap to its own item.
- **Too many threads on a very large machine.** `available_parallelism` may be 64 or more; each
  thread needs only a small default stack and the files to read are shared out, so extra threads
  just finish early.

## Left to the builder

- How the files are shared out among the threads: a shared counter each thread takes the next file
  from, or fixed slices, as reads best; and how each result reaches its position.
- Where the pre-read's code lives: in `src/scripts.rs`, or a small module of its own.
- The names of the new function and its test.
- The report's layout (CMP-6).
