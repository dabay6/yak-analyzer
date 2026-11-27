# Ignoring files: a project's list of files no finding is reported in

**Status:** draft.

**Work item:** `2026-11-26-configuration`. It answers no issue.

**Amends:** `docs/specs/2026-11-11-reading-a-project-design.md` (**the reading spec** below), its
KEY-1 and KEY-2: one key is added, which only a project may state, and the line that refuses an
unknown key gains a second fix (KEY-6 below, REFUSE-2).

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository (**the design
spec** below). It follows DECL-2 (a project states its own declarations in its `yak.json`), DECL-5
(the analyzer keeps no configuration file of its own), FIND-10 (a generated file is read but nothing
is reported in it) and BASE-3 (both sides of a comparison use the declarations in force now). It adds
one key to the design spec's declarations, which is owed there (OWED-1).

The requirement IDs are this spec's own. KEY-6 to KEY-8 continue the reading spec's KEY numbering,
since they add to its keys; the other prefixes may repeat IDs that other specs use, and a reference to
another spec's ID always names that spec.

How to read the marks: **owner** marks what the owner decided in this item's brainstorm, which is on
the work item's record: the log yakshaver keeps of each question, answer and decision of the
brainstorm (`yak work brainstorm-show --item 2026-11-26-configuration`). Everything else not taken from the design spec or an earlier spec of this
repository is the **session's call**: the session that wrote this spec decided it, and the owner has
not ruled on it.

## Intent

A person sometimes wants a set of files the analyzer reports nothing in: scripts kept beside the
code that nobody maintains as product code (yakshaver's performance scripts under `docs/research`
were the example given), or test files, if a project decides their duplication and complexity are not
worth acting on (the owner's first example, given as an example of the control wanted, not a setting
the owner means to turn on). Today a project has no way to say so.

The two keys that come close each say the wrong thing:

- `data` (the reading spec's KEY-4) says a file is not the project's code: it is never parsed, so
  what it imports doesn't count as used, and ignoring a script with it would turn the helpers only
  that script uses into unused exports.
- `generated` (design FIND-10) says a generator wrote the file. Its behaviour is close, but writing
  `docs/research/**` under `generated` is untrue, and the next person to read the declaration is
  misled. It also still reports circular dependencies through a generated file, and a project-only
  list it is not: a pack declares it for the tools it knows.

So this item adds one list of globs, stated by a project in its own declarations, for files that are
still read but in which no audit check reports anything (owner: one global list for all checks, not
one per check; a list of globs, like `data` and `generated`).

How a person gets a setting to the analyzer does not change (owner). The project states it in its
`yak.json`, yak passes it in the hand-off file, and the analyzer keeps no configuration file of its own
(design DECL-2, DECL-5). The brainstorm weighed making the analyzer configurable on its own, without
yak, and dropped it: the separate repository exists for its language, its shipping and the versioned
hand-off between the two, not for use without yak, and standing alone would need the knowledge of
what each tool starts built into the analyzer, which a pack gives today (owner).

Terms:

- **yak**, **the engine**, **the pack**, **the gate**, **the base**, **the hand-off file** and **the swap** are as
  the complexity spec (`docs/specs/2026-11-21-complexity-findings-design.md`) defines them.
- **A source** of a declaration is the pack or the project: the hand-off file keeps the two apart for
  each stack. What made a file an entry point is called its **reason** here, to keep the two apart
  (the dead-code spec's ENTRY-1 calls it a source).
- **The dead-code spec** is `docs/specs/2026-11-13-dead-code-findings-design.md`, and **the
  duplication spec** `docs/specs/2026-11-23-duplication-findings-design.md`.
- **The idea review** was the step of this item's brainstorm where three fresh readers read the idea
  before the spec was written; their findings are on the work item's record.
- **The judged list** is as the complexity spec defines it: the written list of differences the
  owner has judged, and of what is still owed elsewhere.
- **An unreported file** is a file the project's `unreported` globs match (KEY-6).
- **An audit check** is one kind of finding `audit` reports: the five dead-code kinds
  (`unused-file`, `unused-export`, `unused-type`, `unresolved-import`, `circular-dependency`),
  `complexity` and `duplication`.

## Goals, non-goals and anti-goals

**Goals:**

- A project lists, in its own declarations, globs of files no audit check reports anything in.
- An unreported file is still read, so what it imports still counts as used: ignoring a file makes
  nothing it uses look unused. (Ignoring can still bring out a true finding: a file that was unused
  only because nothing but the ignored file reached it is now used, and its other exports, which
  nobody uses, are reported. READ-2.)
- The audit report says, for each glob, how many findings it left out, so ignoring is never silent
  (owner).

**Non-goals:**

- A list per audit check. One list covers every check (owner).
- A pack's default list. The list is the project's alone (owner).
- Taking back a pack's `data` or `generated` glob from a project. Lists still only join.
- Running the analyzer without yak, or a configuration file of the analyzer's own (owner).
- The yak commands that write a project's declarations (design DECL-2's `yak stack` commands). They
  are yakshaver's and not built yet; until they are, a person edits `yak.json` by hand. That runs
  against the README's "no setting a person edits by hand", which holds once those commands exist;
  the gap is yakshaver's to close, and this item leaves the README's line as it is. Session's call.
- Ignoring one finding or one line. `duplicationAccepted` already accepts a duplication group, and a
  finding of another kind is answered at the gate.

**Anti-goals,** failures even if it technically works:

- An ignore glob that hides findings with nothing in the report to show it. One line in a project's
  declarations, such as `src/**`, would otherwise make the audit pass, on the branch and at the base
  alike, since both read the same declarations (design BASE-3).
- Ignoring a file that makes what it uses look unused: an unused file or export that only the
  unreported file used.
- A key a pack can set for every project that uses it, which no project could take back.

## Requirements

### The key

- **KEY-6** `unreported` is a list of globs, a key the analyzer knows (the reading spec's KEY-1).
  Each glob is relative to the checkout's root and follows the reading spec's KEY-5, as a project's
  `data` and `generated` globs do (the reading spec's KEY-3). The globs of every stack that states it
  join, and each matches any file of the checkout, not only the files of its stack, as `data` and
  `generated` do today. A glob that matches no file gives the notice `glob-matches-nothing`, as the reading spec's
  KEY-5 gives for `data` and `generated`. Session's call for the name: `ignore` was the owner's word,
  but `duplicationIgnore` and the `duplication` command's `ignore` field already use it for what
  matches loosely, and `unreported` says what happens to the file, not that it is skipped.
- **KEY-7** Only a project may state `unreported` (owner). When a pack states it, the run stops with
  the reading spec's KEY-2 line. That line, as `src/declarations.rs` writes it today, is "cannot use
  the declaration `<key>` that the <source> states for the stack `<stack>`: <why>; <fix>"; here the
  why is `only a project may state this key` and the fix `change it in the pack`, as `configEntries`
  is refused the other way round.
- **KEY-8** A file that is both `data` and unreported is data: it is not read (the reading spec's
  KEY-4), it is not an entry point, and nothing in it is counted (COUNT-2). A file that is both
  `generated` and unreported is both: it is an entry point (READ-2), and a cycle is left out when
  every file of the set is unreported (CYCLE-1); it holds no duplication copy, being generated (the
  duplication spec's COPY-4). Session's call.

### What an unreported file is

- **READ-1** An unreported file is listed, read and resolved like any other file of its kind (owner,
  as the session's call the owner confirmed: "the file is still read").
- **READ-2** An unreported file is an entry point, with the reason `unreported` (no file names it, as
  for the declaration `entries`, and the stack and the declaration's source are kept). So every file
  and export it imports is used, and a file only it reaches is not an unused file; that file's other
  exports, which nothing imports, are still reported unused (owner, for exports; session's call, for files: the idea review found that counting
  its imports as uses but not reaching through it would still report the files only it imports as
  unused). Making it an entry point only adds to what is reached: an unreported file that was already
  reached stays reached. Unlike every other entry point, an entry point whose only reason is
  `unreported` does not have its own exports all used by the dead-code spec's USE-3: nothing outside
  the project starts it and consumes them. An unreported file that is also an entry point for another
  reason (a `main`, an `exports` target, an `entries` glob) keeps USE-3, as without the key.
  So its re-exports pass no use on to the files they re-export from, and an export of maintained code
  that only an unreported file re-exports is still reported unused; and the unused exports in the
  unreported file itself are worked out, then left out by DEAD-1, and counted when COUNT-2 counts
  them (when the run without the key would report them too). Session's
  call: otherwise a re-export in an ignored file would hide findings in maintained code, uncounted,
  which is the first anti-goal. An entry point with only the reason `unreported` does not count when the
  dead-code spec's ENTRY-8 judges whether a package has no entry point at all, so a package whose
  only entry points are unreported files keeps the hint in its unused-file messages that the likely
  fault is a missing `entries` declaration. Session's call: an ignored script does not make a package
  correctly wired.
- **READ-3** Wherever the analyzer today leaves a generated file out of what it reports on, it leaves
  an unreported file out the same way, except where DUPE-1 and CYCLE-1 say otherwise. That includes
  the `kinds-not-reported` notice (the dead-code spec's ENTRY-9 as `src/findings.rs` gives it), which
  counts only files the analyzer reports on: a package holding only unreported files gives none.
  Session's call: one rule, the generated file's, so an unreported file adds no new kind of exception
  for the builder to find.

### What each check leaves out

- **DEAD-1** No `unused-file`, `unused-export`, `unused-type` or `unresolved-import` finding is
  reported in an unreported file.
- **CPLX-1** No `complexity` finding is reported for a function in an unreported file, by `audit` or
  by the `complexity` command, with or without `--all` (as the complexity spec's CPX-5 does for a
  generated file).
- **DUPE-1** Duplication is found over every file it is found over today, unreported files included.
  Then each group loses its copies in unreported files, and a group left with fewer copies than
  `duplicationCopies` (2 without it) is not reported. The rule that drops a group lying inside another
  (the duplication spec's SAME-4) then runs again on the groups left, since a group kept only by its
  copy in an unreported file can now lie inside a wider group, and would otherwise be reported twice.
  An entry of `duplicationAccepted` (the duplication spec's QUIET-1) accepts a group when its number
  of copies is the group's either before or after the removal, so adding an ignore glob does not
  bring back a group a person accepted, and an entry written from the reduced group, as `audit` now
  shows it, works too. The notice for an entry that matches no group is given only when it matches
  neither. An entry matches a group as it was found, before the removal, even when the removal drops
  the group altogether, so narrowing the glob later finds the acceptance still there; the entry is
  not reported as matching no group. The base comparison's rule that a script it skipped holds no
  copy (`src/inherit.rs`, for generated scripts) does not apply to unreported scripts: their copies
  are found and removed as here. With
  `--base`, the base's groups lose their copies in unreported files by the same rule before they are
  compared (the duplication spec's FRESH-1), since the base reads the same declarations (design
  BASE-3); otherwise every reduced group would be marked new on every run. A group still reported
  keeps its fingerprint, since its run is the same; its tokens and lines are worked out again from
  its remaining copies, as the duplication spec's SIZE-1 works them out from a group's copies; and its
  message and property bag name only the remaining copies. So a copy in an unreported file is never
  reported, and neither
  is a copy elsewhere whose only twin is in an unreported file (the session's call recorded after the
  idea review: "a copy elsewhere of its code is not reported"). The same applies to the `duplication` command,
  with or without `--all`. Session's call, for removing the copies after the groups are found rather
  than leaving the files out before: it gives the same groups to report in nearly every case, and it
  lets COUNT-1 count what was left out from one pass. It differs from how a generated file is left out
  (the duplication spec's COPY-4), on purpose.
- **CYCLE-1** A `circular-dependency` finding is left out only when every file in its set is
  unreported. Otherwise it is reported as the dead-code spec's KIND-4 reports it, with one change:
  where KIND-4 uses the set's first file by path, this uses the first file by path that is not
  unreported. That file's first import, by line, that leads to another file of the set is the
  finding's first location; every other file of the set, unreported ones included, is a related
  location at its own first such import, in path order; and the message's loop runs through that
  file. Its fingerprint is unchanged, since it is made from every path of the set. With `--base`, a
  new cycle is placed as the dead-code spec's INH-3 places it, at the first import that makes an edge
  the branch added, taking only imports in files that are not unreported; when every added edge
  starts in an unreported file, or the branch added none (INH-3's last case), it is placed by this
  requirement's rule. Session's call: a cycle that reaches into the code a person maintains is still theirs to
  break, and its location should be in that code.

### Saying what was left out

- **COUNT-1** `audit` gives one notice of the kind `findings-unreported` for each distinct
  `unreported` glob that matches a file: the glob, the stacks that state it, and how many findings it
  left out, as "The `unreported` glob `docs/research/**` that the project states for the stack
  `typescript` left out 12 findings." (owner). A glob that two stacks state, written the same, gives
  one notice naming both stacks, and its findings are counted once. A glob that matches files but
  left out no finding still gives its notice, with 0, so a glob with no effect is seen. When findings
  in files the globs do not match are no longer reported because only unreported files use what they
  name (COUNT-2), one more notice of the same kind says how many, for all the globs together, as "12
  more findings outside the `unreported` files are not reported, since only those files use what they
  name." Session's call, for the 0 and the extra notice.
- **COUNT-2** A finding is left out by the key when the run would report it without the key and does
  not report it with the key. For the dead-code kinds, the run works out what is reached and used
  twice over the same reading: once with the unreported files as entry points, which is what it
  reports, and once without them, which is what it would report without the key; nothing is read
  again. For complexity, duplication and cycles, the files are reached the same either way, and a
  finding is left out when DUPE-1, CPLX-1 or CYCLE-1 leaves it out. A finding left out counts against
  each glob that matches its file: for a finding with one file, that file; for a duplication group,
  any removed copy's file; for a cycle, any file of the set. A finding that two globs match counts
  against both, so the counts can add up to more than the findings left out. A left-out finding that
  no glob matches (an unused file or export in maintained code that only an unreported file used)
  counts in COUNT-1's extra notice. So a glob over a script nothing reached counts the script's own
  `unused-file` finding and those of every file only it reaches. Not counted: a finding the key brings
  out (READ-2's example in Goals), which is reported; a duplication group still reported with copies
  removed; a group the second pass of SAME-4 drops, whose copies are still reported in the wider
  group; and a group acceptance leaves out (DUPE-1). Findings `data` or `generated` already leave out
  are not counted either, since the run would not report them without the key; but a cycle of files
  that are all unreported, some of them generated, is counted, since `generated` does not leave out
  cycles. Session's call: the count says what the key hides from the report, against the report
  without it.
- **COUNT-3** With `--base`, the counts are the branch's alone. The base reads the same declarations
  (design BASE-3), so its findings are left out by the same rules, and nothing about the base is
  counted. Session's call.
- **COUNT-4** The `complexity` and `duplication` commands give the notices of COUNT-1 too, each
  counting only the findings of its own check. `--all` does not change the counts: it lists more of
  what is reported (every function, or the accepted groups), never what the key left out. Session's call: they leave
  out the same files as `audit`, and a person reading one of them should see that too.
- **COUNT-5** `graph` prints, for each file, whether it is unreported, beside its `data` and
  `generated` flags, and lists the `unreported` reason among an entry point's reasons (READ-2). What
  `graph` prints is outside the interface version (CONTRIBUTING.md).

### Refusing a key the analyzer doesn't know

- **REFUSE-1** The key is added before any project writes it: the analyzer release that knows
  `unreported` comes first, the pack pins that release, and only then may a project state the key.
  A project that writes it under an older analyzer has its run stopped by the reading spec's KEY-2.
- **REFUSE-2** The line that refuses a key the analyzer doesn't know (the reading spec's KEY-2, in the
  shape KEY-7 quotes), whether the pack or the project states the key, gains a second fix after the
  first: `, or, if the key is newer than this analyzer, update the analyzer's version pinned in the
  pack`. Session's call: the line today tells a project to change its
  own declarations, which is the wrong fix for a key that is right but too new, and this item is the
  first that adds a key a project is expected to write.
- **REFUSE-3** `interface` also prints the declaration keys the analyzer knows, as a list of their
  names. It only adds to what `interface` prints, so the interface version is not raised. With it,
  yak can tell, before it runs the analyzer, that a project's `yak.json` states a key the analyzer its
  pack pins does not know, and say to update the pack, on a machine or in a CI job whose pack lags
  the commit that added the key (OWED-1). Session's call: REFUSE-1 orders releases, not installs, and
  without this the older analyzer's line tells the person to delete a key that is right.

### The interface version

- **VERS-1** Interface version 1 gains the key `unreported`. It only adds, so the interface version is
  not raised (the complexity spec's VER-1). `CONTRIBUTING.md` gets a paragraph for it, as duplication
  got one for its keys, after the duplication paragraph: the key, that only a project states it, and
  the notice `findings-unreported`. The notice's kind is not part of the interface, as no notice kind
  is today; yak shows notices as text. The entry-point reason `unreported` is shown only by `graph`,
  which is outside the interface (COUNT-5).

### Fixtures

- **TEST-1** Made-up projects (CONTRIBUTING.md's rules) show, each with its expected findings written
  in the test:
  - an unreported file with an unused export, an unresolved import and a function over the limits,
    none reported;
  - a file and an export used only by an unreported file, neither reported as unused; and an export
    that an unreported file only re-exports, still reported unused (READ-2);
  - a duplication group of three copies with one in an unreported file, reported with two; one of two
    copies, not reported; with the counts of COUNT-1 (DUPE-1, COUNT-2);
  - a cycle of an unreported and an ordinary file, reported at the ordinary file with the same
    fingerprint as without the key; a cycle of unreported files only, not reported (CYCLE-1);
  - two globs matching one file, each counting its findings, and a glob matching a file with no
    finding, counting 0 (COUNT-1, COUNT-2);
  - an `unreported` glob that matches no file, giving `glob-matches-nothing` (KEY-6); and one glob
    that two stacks state, giving one notice naming both (COUNT-1);
  - an accepted group that removal drops altogether, its entry given no notice (DUPE-1);
  - a pack stating `unreported`, refused (KEY-7); a file both `data` and unreported, not read (KEY-8);
  - an unknown key, refused with both fixes (REFUSE-2);
  - a branch that adds an unreported glob against a base without it: findings it hides are left out
    on both sides, and the notice counts them (COUNT-3);
  - an unreported script nothing reaches, which imports a file only it uses: neither reported, both
    `unused-file` findings counted, the second in the extra notice (COUNT-1, COUNT-2); and a file that
    an unreported script makes used, whose other export is now reported (Goals);
  - an unreported file that is also a package's `main`, whose re-exports still pass use (READ-2);
  - a package whose only entry point is an unreported file, keeping ENTRY-8's hint; and a package
    holding only unreported files, giving no `kinds-not-reported` notice (READ-2, READ-3);
  - a group that the second pass of SAME-4 drops; an accepted group that loses a copy, still accepted
    by its old entry and by a new one (DUPE-1);
  - a new cycle with `--base` whose first added edge starts in an unreported file and a later one in
    an ordinary file, and a new cycle with no added edge (CYCLE-1);
  - `interface` listing the declaration keys, `unreported` among them (REFUSE-3); and a base group that loses its unreported copy, marked unchanged
    (DUPE-1);
  - a file both `generated` and unreported, an entry point and in a cycle of unreported files left out
    and counted (KEY-8, COUNT-2);
  - the `complexity` and `duplication` commands, leaving the files out and giving their notices
    (CPLX-1, DUPE-1, COUNT-4), and `graph`'s flag and reason (COUNT-5).

### Owed to yakshaver

- **OWED-1** These go to yakshaver through the work item
  `2026-11-18-analyzer-design-spec-stage-2-amendments`, or its successor if it has closed, and not in
  this repository:
  - a design-spec amendment adding `unreported` to the declarations: a list of globs that only a
    project states, its files read but nothing reported in them, and the `findings-unreported` notice
    (KEY-6, KEY-7, COUNT-1);
  - that design DECL-2's `yak stack` commands, when built, accept `unreported` for a project and
    refuse it for a pack;
  - that the engine shows the `findings-unreported` notices to the person at the gate, as it shows the
    run's other notices, and that its existing question when a work item changes a project's
    declarations covers a new `unreported` glob (design HAND-7);
  - a check that yak passes a project's `unreported` list from its `yak.json` into the hand-off file
    under the project's declarations for the stack, as it passes `data` and `generated`;
  - the release order of REFUSE-1: the pack pins the analyzer release with this key before any project
    is told it may write it;
  - that the engine reads REFUSE-3's list of keys from `interface` and, when a project states a key
    the pinned analyzer does not know, stops before running it with a line that says to update the
    pack, not to remove the key.

  The item closes only once each is filed in that yakshaver item or written in the judged list as
  still owed there. Session's call, as the duplication spec's OWED-1.

## Alternatives considered

- **Tell people to declare such files `generated`.** It needs nothing built and gives nearly the same
  result. Rejected: the declaration says something untrue about the file, it still reports cycles
  through it, a pack declares the key too, and nothing would count what it hides.
- **Declare them `data`.** Rejected: a data file is not read, so whatever only it imports becomes an
  unused file or export, the second anti-goal.
- **One list per check** (`duplicationUnreported`, `complexityUnreported` and so on). Rejected by the
  owner: one list for every check is what was asked for, and no case of wanting one check off but not
  the others came up in the brainstorm.
- **A list a pack may declare as a default, which a project adds to.** Rejected by the owner: ignoring
  is a judgment about one project's code; a pack default could not be taken back by a project, since
  lists only join; and a pack's globs are matched from each folder yak's detection looked in, which
  makes a broad ignore glob broader still.
- **A configuration file of the analyzer's own, so it runs without yak.** Weighed first and dropped
  by the owner (Intent).
- **No count in the report, relying on yak's gate question when declarations change** (design
  HAND-7). Rejected by the owner: the gate asks once, when the glob is added, and the report should
  show the effect on every run.
- **Leave unreported files out of duplication before the groups are found**, as `generated` files are.
  Rejected, session's call: COUNT-1 would then need a second duplication pass to know what was left
  out, and the results differ only in rare cases (Risks).
- **Name the key `ignore` or `ignored`.** Rejected, session's call (KEY-6).

## Risks

- **A broad glob hides real findings.** `src/**` in a project's declarations would empty the report.
  Handled by COUNT-1, which shows the count on every run, and by the engine's question when
  declarations change (OWED-1). The analyzer does not judge a glob too broad: it has no rule to judge
  by.
- **Ignoring tests makes code used only by tests look used.** By READ-2 a helper only an unreported
  test file uses is not reported unused. That is the point of reading the file (owner), and a project
  that wants such helpers found leaves its tests reported.
- **DUPE-1 differs from leaving the files out first.** Runs are grown to their longest across all
  files, so a run whose longest form is shared with an unreported file may be reported at that length
  among the remaining copies rather than at a length found among them alone. The group is still a
  real duplicate, and the fixtures fix what is reported; handled by accepting the difference, written
  in DUPE-1.
- **A project writes the key before its analyzer knows it.** Its run stops, and the older analyzer
  prints today's line, which tells the project to change its own declarations. This happens on any
  machine or CI job whose pack still pins an older analyzer once the key is committed. Handled by
  REFUSE-1's order, and by REFUSE-3 with the engine's check (OWED-1), which say to update the pack;
  REFUSE-2's line helps only with keys added after this one.
- **Ignoring brings out findings.** A file used only by an ignored script was one `unused-file`
  finding; with the script an entry point, the file is used and its other exports are reported. They
  are true findings, reported (Goals); a person adding a glob may see the report grow before it
  shrinks.
- **A cycle's location moves.** CYCLE-1 puts a partly unreported cycle at its first file not
  unreported. The fingerprint is made from all the set's paths, not the location, so the finding
  matches its base and any recorded reason across the change.
- **The count costs a second pass.** Findings are worked out in unreported files and then dropped,
  which is the work done today, when those files are reported on. COUNT-2 adds a second pass of
  reach and uses over the same reading, without the unreported entry points; nothing is parsed again.
  It runs only when an `unreported` glob matches a file, and the cold and warm measurements
  (`docs/specs/2026-11-19-cold-warm-measurements-design.md`) show whether it needs more care.

## Left to the builder

- Where the `unreported` set is built (it belongs with `src/marks.rs`'s other two) and how each check
  reaches it.
- Where the counts are gathered: from the full set of findings before filtering, or by each check
  counting what it drops, so long as COUNT-2's definition holds.
- The wording of the `findings-unreported` notices, keeping a glob's notice to its three parts (glob,
  stacks, count) and the extra notice to its count. Session's call for the order: the globs' notices
  in their order in the hand-off file, then the extra notice.
- How `graph` names the per-file flag and the entry source in its output.
- The fixtures' file names and how many projects TEST-1's cases are spread across.
