# Stage 2: the dead-code comparison with fallow, and the owner's go

**Status:** draft.

**Work item:** `2026-11-16-dead-code-comparison`, the analyzer's stage 2. It answers no issue.

**Amends:** `docs/specs/2026-11-13-dead-code-findings-design.md` (the stage-1 spec, called **the
stage-1 spec** below):

- its USE-2, the rule that `require('./x')` and `import('./x')` with a literal argument use every
  export of the target: DYN-1 to DYN-4 replace it where what is read off the result can be followed;
- its ENTRY-6, the shape of a `configEntries` declaration: CONF-1 adds one optional field;
- its amendment of 2026-11-15, which left a configuration file named by a script option to stage 3:
  CONF-1 to CONF-3 bring it into this stage;
- its CMD-5, and the reading spec's RUN-7 and KEY-1, which keep the declaration keys, rule names,
  fingerprints and `audit`'s options as drafts until the end of stage 2: FREEZE-1 ends the drafts.

**Builds from:** the analyzer's design spec, `docs/specs/2026-11-10-in-house-code-analyzer-design.md`
in the yakshaver repository, called **the design spec** below. This item builds its STAGE-2:
PROOF-2, PROOF-2a, PROOF-3, PROOF-4 and PROOF-5 for the five dead-code kinds, PROOF-6, the end of
stage 2 that STAGE-1 names for ENT-4's keys, and SPEED-4's measured times. "(design PROOF-4)" after
a sentence names the requirement it follows from.

How to read the marks: **owner** marks what the owner decided in this item's brainstorm, which is on
the work item's record. Everything else not taken from the design spec or the stage-1 spec is the
**session's call**: the session that wrote this spec decided it, and the owner has not ruled on it.

## Intent

Stage 1 is done. The analyzer reports the five dead-code kinds (unused file, unused export, unused
type, unresolved import, circular dependency) as SARIF, from entry points declared in a hand-off
file, and marks each finding new or unchanged against a base commit. Its last plan, the live check
of 2026-11-15 (the stage-1 spec's LIVE-1 to LIVE-8), ran it on the named repositories and judged
every finding it reported by reading the code; after that plan's fixes, no finding it reports is
known to be false, apart from one case left for later. What the live check did not do is look at
fallow's side: the findings fallow reports and the analyzer does not were counted, and not judged.

So nobody yet knows whether the analyzer misses real dead code that fallow finds. The design spec's
bar for replacing fallow (design PROOF-4) is two-sided: no false finding, and no true finding fallow
reports that the analyzer misses. This item answers the second half, finishes the first on fresh
commits and on branches, fixes the analyzer until it meets the bar or until what is left is written
down with why, and ends with the owner's decision to go on or to stay with fallow (design PROOF-6).
Nothing in yak's engine, the pack SDK or the pack is built before that decision (design ORDER-1), so
until then the analyzer's work is the only thing at stake.

The owner's bar, in their words: "the floor is identical to fallow, the ceiling would be no false
positives, and no missed" (owner). A false finding fallow shares does not stop progress; what stops
it is the analyzer "report[ing] things that just aren't real and there isn't an obvious way to
correct it" (owner). And a miss the analyzer makes on purpose is still a miss: "if something is
truly unused isn't [it] dead code?" (owner).

Terms. Every term the stage-1 spec's Intent defines holds here as it does there, among them fallow,
an entry point, a reachable file, an export, a use, a finding, SARIF and the detection folders, and
through it the reading spec's terms: the owner, a session, the builder, the plan, yak, a pack, a
stack, the root, a package, the hand-off file, a declaration, a notice, the named repositories, the
comparison, the swap and the report folder. Added here:

- **A root** is one folder a tool is run from: a named repository's checkout, or a web app's
  sub-folder in it that is compared a second time from that sub-folder (design PROOF-2).
- **The pinned commit** of a named repository is the commit this item runs both tools on (CMP-1).
- **The live check** is the stage-1 spec's LIVE-1 to LIVE-8, as run on 2026-11-15, and its report
  folder is `yak-reports/2026-11-14-dead-code-findings/live/` under the git common directory of this
  repository's main checkout.
- **A shared finding** is one both tools report (JDG-2); **an analyzer-only** or **a fallow-only
  finding** is one only that tool reports.
- **True** and **false**: a finding is true when the code bears it out under the design spec's
  definition of its kind (design FIND-2 to FIND-7), and false otherwise. **A miss** is a true
  finding fallow reports that the analyzer does not.
- **The check of 2026-11-11** was run by the design spec's work item,
  `2026-11-10-in-house-code-intelligence-engine` in the yakshaver repository, on the owner's
  machine; the design spec's Intent gives its findings. **fallow's best-case configuration** for a
  repository is the entry configuration that check wrote by hand for fallow (design PROOF-2): globs
  and paths that tell fallow each repository's entry points. Its files would be in that work item's
  report folder, `yak-reports/2026-11-10-in-house-code-intelligence-engine/` under the git common
  directory of yakshaver's main checkout; on the machine this spec was written on, that folder does
  not exist (CMP-5).
- **The reading spec** is `docs/specs/2026-11-11-reading-a-project-design.md` in this repository,
  the analyzer's first spec, which the stage-1 spec builds on.

## Goals, non-goals and anti-goals

**Goals:**

- Every finding only one of the two tools reports, on every root of the named repositories, is
  judged true or false by reading the code, with why (design PROOF-2).
- The same for the findings each tool marks new on a few merged branches of each repository (design
  PROOF-2a).
- The analyzer is fixed until it has no false finding and no miss there, or until each case left is
  written down with why it is not fixed (design PROOF-4, owner).
- The analyzer follows what is read off an `import()` or a `require()`, so an export nothing reads is
  reported even when its file is loaded that way (owner).
- A configuration file a script names by an option is read as the tool reads it, here and not in
  stage 3 (session's call).
- The declaration keys, rule names, fingerprints and `audit`'s options stop being drafts (design
  STAGE-1, ENT-4).
- The owner reads one judged list and decides go or no-go, and the decision is on the record (design
  PROOF-6).

**Non-goals:**

- Complexity and duplication (design STAGE-4). They are judged the same way there, before any swap.
- Anything in yak's engine, the pack SDK or the pack (design ORDER-1): the hand-off files stay
  written by hand. Stage 3 writes them and repeats the comparison on yak's own files (design
  PROOF-8).
- Kinds the analyzer does not report by design (JDG-5): fallow's duplicate exports, dependency
  findings, and unused files that are not scripts or components, such as CSS.
- Speed work. Times are measured and written in the judged list (design SPEED-4); any work on them
  is stage 4's.
- A release. release-please opens its pull request as usual; merging it is the owner's choice.

**Anti-goals,** failures even if it technically works:

- A miss written off as "by design". What is truly unused is dead code (owner); only JDG-5's kinds
  and the definitions the design spec writes down are set apart, and each one set apart is listed
  for the owner to read.
- A false or missed finding cured by a declaration in the hand-off file, where the analyzer could
  have known it (owner, CURE-1).
- A difference between the two tools that comes from running them on different trees, rather than
  from the tools.
- A private repository's name, files or code committed here, in a fixture, a message or this
  spec's text (CONTRIBUTING.md). The judged list stays in the report folder (design PROOF-3).
- Judging that never ends because the repositories keep moving (CMP-1).

## Requirements

### The runs

- **CMP-1** Each named repository is pinned at one commit when the comparison starts: the head of its
  default branch that day (owner). Every run, both tools' and every rerun after a fix, is on the
  pinned commits, until the final run of GO-1. The pinned commits are written at the top of the
  judged list.
- **CMP-2** The named repositories and roots are the live check's: yakshaver, bricklog with its web
  app's sub-folder as a second root, and the two private repositories, the smaller with its frontend
  sub-folder as a second root; six roots in four repositories. Their names come from the record of
  the design spec's work item, and the live check's report folder names them; this spec does not
  (CONTRIBUTING.md).
- **CMP-3** The analyzer runs on a clean worktree of the pinned commit, with no dependencies installed
  and nothing built, given a hand-off file written by hand (design PROOF-2). The hand-off files start
  as the live check's, one for each root. A hand-off file has two parts (the reading spec's HO-2):
  the fields that say where the tree is, `root` and `detection`, which are rewritten for each
  worktree a run uses and are not a change to the file in this spec's sense; and the declarations,
  `stacks`, which change only as CURE-2 allows, each change written in the judged list with its
  reason. Wherever this spec speaks of a hand-off file changing, it means its declarations.
- **CMP-4** fallow runs on a second worktree of the same commit, with dependencies installed and
  nothing built (owner). The install runs in each folder of the worktree that holds a lockfile (for
  a workspace, its root; a sub-folder root with a lockfile of its own is installed there too), with
  the package manager that lockfile belongs to, at the version the nearest `package.json`'s
  `packageManager` field names, or else the version installed on the machine, written in the judged
  list. Every command installs from the lockfile alone, with install scripts off:
  - `package-lock.json`: `npm ci --ignore-scripts`;
  - `pnpm-lock.yaml`: `pnpm install --frozen-lockfile --ignore-scripts`;
  - `yarn.lock`, Yarn 1: `yarn install --frozen-lockfile --ignore-scripts`; Yarn 2 or later:
    `yarn install --immutable --mode=skip-build`.

  No install script of any package runs; a package manager or version that offers no way to turn
  them off is not used, and that folder is run by fallow without dependencies. So is a folder with
  no lockfile, or whose install fails; the judged list says so beside its counts. This holds for
  every commit fallow runs on: the pinned commits, each branch audit's commit (BRA-1) and the final
  run's (GO-1). All worktrees are made in a temporary folder outside every repository, and removed
  once the owner has decided (GO-3). Session's call: two worktrees, so the analyzer's worktree never
  holds `node_modules`. The analyzer's findings should not change with `node_modules` present (an
  anti-goal of the stage-1 spec), but PROOF-2 asks for a clean worktree, and two worktrees give it
  without relying on that.
- **CMP-4a** Before installing in a folder, the session checks it for code a package manager would run
  or fetch despite scripts being off: a `.pnpmfile.cjs` (pnpm), a `.yarnrc.yml` with `yarnPath` or
  `plugins` (Yarn 2 or later), and a `packageManager` field that Corepack would download. Where one
  is present, it tells the owner what it found and installs there only on the owner's word; without
  it, that folder is run by fallow without dependencies, as CMP-4 says. Session's call.
- **CMP-5** fallow runs with its best-case configuration for each repository (design PROOF-2). The
  session looks for the 2026-11-11 configuration in the report folder the Terms name, on the machine
  it runs on, and asks the owner whether another of their machines holds it; it copies what it finds
  into this item's report folder. The live check's report folder does not hold it: the live check
  ran fallow with each repository's own configuration (JDG-4). Where it cannot be found, or names
  paths that no longer exist at the pinned commit, the session writes or corrects it from the design
  spec's account of that check and the repository's code, and writes each change with its reason
  beside the copy. A fallow finding caused by a stale or rewritten configuration is still judged,
  since the judgment is of the finding against the code (design PROOF-2).
- **CMP-6** One fallow version is used for every run: 3.39.0, the version the live check ran, as its
  fallow output records. fallow's `audit --help` (read from 3.15.0, the version installed on the
  machine, on 2026-11-15) gives `--base` as an alias of `--changed-since`, says the audit reports
  findings in the changed files only, and marks each JSON finding `introduced: true` or `false`; the
  plan's first step confirms each of these on 3.39.0 before any branch audit, and a difference is
  brought to the owner (BRA-1). It is installed once, at that
  exact version, into a folder of its own in the temporary folder, with `npm install --ignore-scripts
  fallow@3.39.0`, and run from there (should fallow not run without its install script, the session
  tells the owner before running that script); a fallow a
  repository's own dependencies install, and one installed on the machine for every project, are
  not used. Each run asks for JSON (`--format json`); fallow exits 1 when its own gate fails on
  findings, as the live check saw, so exit 1 with JSON printed is a run that worked, and any other
  failure stops that run to be looked at. Session's call.
- **CMP-7** Both tools are run on each root from that root, as the gate runs (design PROOF-2). A
  sub-folder root runs fallow from the sub-folder too, so each pair of counts in the judged list is
  of the same root. The live check set fallow's root counts for bricklog and the smaller private
  repository beside the analyzer's without that rule; those counts are redone.

### Judging

- **JDG-1** For each root, the analyzer's findings and fallow's are matched, and every finding only
  one tool reports is judged true or false by reading the code, with why, by the session (design
  PROOF-2). Findings with the same cause are one case and are judged once, with the count. Shared
  findings are judged too where the live check has not judged them, since a shared false finding is
  still short of the ceiling (BAR-3).
- **JDG-2** Two findings are the same when they are of the same kind and name the same thing: an
  unused file by its path; an unused export or type by its file and name; an unresolved import by
  its file and specifier; a circular dependency by its set of files. Three cases are shared though
  they are not written the same:
  - an unused export or type fallow reports in a file the analyzer reports as an unused file: the
    file finding covers it (the stage-1 spec's Alternatives);
  - an unused name fallow reports where it is passed on (a file that re-exports it) and the analyzer
    reports at its origin (the stage-1 spec's USE-2a): matched by following the chain of re-exports
    as `graph` prints it;
  - one or more cycles fallow reports whose files all lie in one strongly connected set the analyzer
    reports: all of them match that one finding (the design spec's FIND-7, one finding per set). fallow's kinds are mapped to the five by the builder (left to
  the builder). Session's call.
- **JDG-3** The live check's judgments carry over (owner: start from the live check's judged lists and
  judge only what is new, plus fallow's side). A finding the analyzer reported in the live check, and
  reports at the pinned commit with the same fingerprint, keeps its judgment, marked as carried with
  the live check's reason, when nothing it depends on changed between the two commits. The live
  check's commit for each repository is in the "Roots and commits" table of its `report.md`. Its
  judged lists, `<repository>/judged.md`, name findings by kind, file and name, not by fingerprint;
  each is joined to its fingerprint through the live check's last SARIF run of that root,
  `runs/<root>.final.sarif`, where `<root>` is that root's own folder name in the live check's report
  folder, as it stands there (six folders, one per root; a sub-folder root's folder is named for the
  repository and the sub-folder, as `bricklog-web` is). A repository's `judged.md` covers all its
  roots and says which root each finding is from. The join uses JDG-2's matching rule. The live check's branch audits are not carried: BRA-1 chooses branches
  afresh. Carried, when none of these changed:
  - for every finding of a root: in any `package.json` under the root, a field the analyzer reads for
    entry points or resolution (`name`, `main`, `module`, `types`, `typings`, `bin`, `exports`,
    `imports`, `scripts`, `workspaces`), not a version or a dependency list, so a release's version
    bump reopens nothing; any tsconfig, or configuration file a `configEntries` declaration reads,
    under the root; and the root's hand-off declarations;
  - an unused export or type: its file; every file that imports it, at either commit; and every file
    along a chain of re-exports to or from it (the stage-1 spec's USE-2, USE-2a, USE-3);
  - an unresolved import: its file, and the folder or package its specifier points into;
  - a circular dependency: every file of its set;
  - an unused file: the file; and every file under the root that holds a computed `import()` or
    `require()` (the stage-1 spec's ENTRY-10), at either commit, since such a loader can start the
    file where no import shows it.

  Any one of them changing reopens the judgment, and the finding is judged afresh. A carried judgment
  is also reopened when a fix (CURE-3) changes the rule it was judged under. The rule assumes the
  analyzer still reports correctly that the finding exists, which is what the live check judged;
  it does not assume the finding is still true where something it rests on changed. Session's call
  for what each kind depends on.
- **JDG-4** The live check's fallow-only counts are redone in full from its fallow output, not taken
  from its report's prose, which left some out: in yakshaver, fallow reported 19 unused files where
  the report accounts for 3. The live check ran fallow with each repository's own fallow
  configuration, or fallow's defaults where it has none (the stage-1 spec's LIVE-4), not the best
  case, so its list is only a start: it shows the shapes to expect and the size of the work, and the
  best-case runs of CMP-5 on the pinned commits give the list that is judged.
- **JDG-5** Set apart and not counted as misses, each listed with its count in a section of its own
  so the owner can read what was set apart:
  - the kinds the design spec's FIND-12 puts out of scope: unused class members, unused store
    members, unused, unlisted or misplaced dependencies, duplicate exports and architecture
    boundaries (in the live check's fallow output, 96 class members in the larger private repository
    and 2 store members in bricklog, among others). Each is listed by kind and count, so the owner
    sees how much unused code of these kinds fallow reports;
  - unused files that are not scripts or Vue components, such as CSS, which the stage-1 spec's
    KIND-1 does not report;
  - a cycle closed only by an `import()` or an `import type`, which the design spec's FIND-7 says is
    not a circular dependency because neither can cause the load-order fault the finding warns of;
  - an unused export of a package's public surface: an export of a file a `package.json` names as
    an entry point, through `main`, `module`, `types`, `typings` or `bin` in any package, or through
    `exports` in a package not declared internal (the stage-1 spec's ENTRY-1, ENTRY-5 and USE-3),
    which the design spec's FIND-5 counts as used, a rule that spec marks as the owner's. Session's call to
    apply it here, written beside the owner's decision on misses; the owner has not ruled on it
    against "truly unused is dead code". Since it rests on the hand-written `internal` list, the
    judged list names every package that offers files through `exports` and is not declared internal,
    with whether its `package.json` marks it `private`, so a package that is never published and
    should be internal shows.

  Nothing else is set apart. In particular a miss caused by a rule the stage-1 spec chose for safety,
  such as USE-2's for `import()`, is a miss (owner).
- **JDG-5a** A kind fallow reports that neither the five nor FIND-12 names, such as unused enum members
  or unused component props, is neither set apart nor counted as a miss yet: whether the analyzer
  should report it is a question of scope the design spec never weighed. Each such kind is listed by
  kind and count, ten of its findings (or all, when fewer) are judged true or false, and the kind is
  brought to the owner with that sample before GO-3. The owner's answer decides it: a kind the owner
  rules in is a design-spec amendment and new work for a later stage, and its true findings are
  listed at GO-2 as misses of a kind ruled in, apart from the misses within the five kinds, and do
  not decide BAR-1 in this stage, which could not reach them by construction; a kind the owner rules
  out joins JDG-5. Session's call.
- **JDG-6** Each judgment names the finding, the tool or tools that report it, true or false, the
  reason in a sentence, and for a false finding or a miss, its cause as a shape the analyzer can be
  fixed against. The judged list is written to the report folder, never committed (design PROOF-3,
  GO-2).

### The bar

- **BAR-1** **The floor** (owner): the analyzer is no worse than fallow. It is at the floor when every
  analyzer-only finding is true and it has no miss. A false finding both tools report does not put it
  below the floor.
- **BAR-2** **The stop** (owner): an analyzer-only false finding with no obvious way to correct it:
  the session can neither fix it in the analyzer nor cure it by a declaration CURE-1 allows (a fact
  the analyzer could never know, stated by a pack or the repository). A case a declaration CURE-1
  allows does cure is not a stop: it is cured, and the declaration is written as CURE-2 says. It is what the owner named as stopping progress: a case of it left at GO-2 is a reason for
  no-go that the judged list states at its top. The session does not stop the comparison on finding
  one: it tells the owner at once, with why it sees no fix, and goes on judging and fixing the rest,
  so the owner decides with the whole list in front of them (design PROOF-6). Session's call on going
  on.
- **BAR-3** **The ceiling** (owner, design PROOF-4): no false finding, shared or not, and no miss. It
  is the target the fixes work towards.
- **BAR-4** The fixes end when the ceiling holds on the pinned commits, or when every case left is
  written down with why it is not fixed and where it leaves the analyzer: a miss leaves it below the
  floor, an analyzer-only false finding is a stop (BAR-2), and a shared false finding leaves it short
  of the ceiling. Each case left also says whether a later stage takes it, and which, or that none
  does. Then GO-1 runs, and GO-2 states the cases left at its top. Session's call.

### Branch audits

- **BRA-1** In each named repository, the comparison audits a few merged pull requests with changes to
  code, each against its base, with both tools: the analyzer's `audit --base <commit>` and fallow's
  `fallow audit --base <ref>` (design PROOF-2a). The worktree and the base are chosen as the
  stage-1 spec's LIVE-3 chooses them, from merges before the pinned commit; the live check's branch
  is one of them where its merge is still before it. Two per repository at the least (session's
  call). Each branch audit runs on every root of its repository, a sub-folder root included, with
  that root's hand-off file. fallow's worktree at the branch's commit is installed as CMP-4 says.
  fallow's audit reports only findings in the files the branch changed, with `introduced` true or
  false, as its help says (CMP-6); the analyzer's reports every finding, marked new or unchanged, so
  the analyzer's are compared only where they lie in a changed file, and its new findings in other
  files are judged on their own. A finding the branch causes in a file it left untouched, such as a
  file made unused by a removed import, is one fallow's audit cannot see: it is judged for the
  analyzer alone and is never a miss.
- **BRA-2** The findings each tool marks new are matched as JDG-2 matches findings, and every finding
  marked new by only one tool is judged as JDG-1 judges. So is every finding the analyzer marks
  unchanged in a file the branch changed, since that is where a wrong match would hide (the stage-1
  spec's LIVE-3).
- **BRA-3** fallow's way of deciding new may differ from the analyzer's fingerprint (design IF-4). A
  difference that comes only from the two ways of matching, and not from either tool reporting
  wrongly, is written down as such and is not a false finding or a miss; a finding wrongly marked
  unchanged by the analyzer is a false finding of the analyzer's, judged as the ceiling asks.

### Fixes

- **CURE-1** A false finding or a miss is fixed in the analyzer (owner). A declaration in a pack's part
  of the hand-off file states only what the analyzer cannot know on its own, such as a tool's fixed
  facts ("vitest reads `test.include`"); a declaration in the repository's part states only that
  repository's own choices. Neither is added to make one finding right, unless the cure is a piece
  of information the analyzer could never know (owner).
- **CURE-2** A change to a hand-off file that CURE-1 allows is written in the judged list with its
  reason, and the declarations of each root's last judged run are kept there, since stage 3 takes
  the pack's part for the pack's declarations and the swap takes the repository's part for its
  `yak.json` (design PROOF-2, PROOF-3, SWAP-5).
- **CURE-3** Each false finding or miss becomes a made-up fixture of its shape, failing, before it is
  fixed (design PROOF-5). Fixtures copy no code from the repositories (CONTRIBUTING.md).
- **CURE-4** A fix that changes what an approved spec says is an amendment to that spec, written in
  this item before the fix is planned, as this spec's DYN and CONF requirements amend the stage-1
  spec. A fix that changes what the design spec says comes back to it as an amendment there (design
  ORDER-1).
- **CURE-5** After each fix, the analyzer is rerun on every root's pinned commit and every branch
  audit, and every finding the fix added or removed is judged; the judged list is updated. Session's
  call: a fix for one repository's shape can change another's findings.

### Following what is read off `import()` and `require()`

The stage-1 spec counted every export of a file loaded by a literal `import()` or `require()` as
used, in the safe direction, and said stage 2 would show what it hides. The live check found one
case: a file loaded only by `(await import('…')).name` has two exports used only in their own file,
which fallow reports and the analyzer does not. The owner ruled that a miss (owner).

- **DYN-1** A literal `import('./x')` or `require('./x')` uses only the exports read off its result,
  when every use of the result is one of these forms:
  - a member read with a name or a literal key, `(await import('./x')).a`, `require('./x').a`,
    `(await import('./x'))["a"]`;
  - a destructuring with names only, `const { a, b: c } = await import('./x')`, or
    `const { a } = require('./x')`; a default value (`{ a = 1 }`) or a nested pattern
    (`{ a: { b } }`) still uses only the top-level names, here `a`; a rest element (`{ a, ...r }`)
    falls under DYN-2;
  - a binding that holds the module, `const m = await import('./x')` or `const m = require('./x')`,
    whose every reference is a member read with a name or a literal key, as USE-2 reads a namespace
    import's `ns.name`;
  - a `.then` callback on the `import()`, whose parameter is a binding used as the third form says,
    or a destructuring with names only, `.then(({ a }) => …)`;
  - a binding that holds the promise, `const p = import('./x')`, whose every reference is
    `(await p)` read in the first or second form, or `p.then(…)` with a callback of the fourth form.

  The binding forms hold only for a binding that has at least one reference in code, is not
  exported, and is not named in a Vue component's template (the stage-1 spec's USE-4). An exported
  binding, or one a template names, passes the module on and falls under DYN-2; a binding with no
  reference at all uses nothing, as an unused result does.

  A member read straight off a promise, `import('./x').a` or `p.a` where `p` holds the promise, is
  not a read of the module and falls under DYN-2; `.then` on a promise is never a use of an export
  named `then`. Session's call for the forms.
- **DYN-2** Any other use of the result uses every export of the target, as USE-2 says today: the
  result passed on, returned, spread, read with a computed key, destructured with a rest element,
  reassigned, or used in a form DYN-1 does not list (owner's decision, session's call on the
  boundary). An `import()` or `require()` whose result is not used at all uses nothing.
- **DYN-3** `import x = require('./x')` is read as `import * as x from './x'` is (USE-2): `x.name`
  uses `name`, and any other reference of `x` uses every export. Session's call: it is the same
  binding with another syntax.
- **DYN-4** A file loaded this way is still reachable (KIND-1), and an export of an entry point is
  still used (USE-3). `graph` prints each use DYN-1 counts at the reference, as it prints a namespace
  member's (the stage-1 spec's CMD-4). Fixtures hold each form of DYN-1, each case of DYN-2 and
  DYN-3, and the live check's shape (CURE-3).

### A configuration file a script names

The live check left one false finding for stage 3: a package's script runs its test runner with
`--config` and a second configuration file, whose name the pack's `configEntries` glob does not
match, so the test files that configuration includes were reported unused. Reading the script's
words is the analyzer's work, and by CURE-1 a fix the analyzer can make is made here (session's
call, on the owner's rule).

- **CONF-1** A `configEntries` object (the stage-1 spec's ENTRY-6) may hold one more field,
  `options`: an object with `program`, one program name, and `names`, a list of option names, such
  as `{ "program": "vitest", "names": ["--config", "-c"] }`, and, optionally, `relativeTo`: what a
  value read from a file a script names is relative to, `"config"`, `"detection"`, or `"script"`,
  the folder of the `package.json` whose script names the file, which is the working directory a
  package manager runs the script in. Absent, the declaration's own `relativeTo` holds. A tool may
  read a value against its working directory when its configuration file is given on the command
  line, and only the pack can say whether it does. The field is optional; an object holding it in
  another shape stops the run as ENTRY-6a says. Which option a tool reads its
  configuration file from is a fact of the tool, so a pack states it, as it states `file` (CURE-1,
  design ENT-5).
- **CONF-2** A script of a `package.json` is cut into commands and words as the stage-1 spec's ENTRY-7
  cuts it. In a command, the program is the first word, after the leading `NAME=value` words, that
  is the declaration's `program`, compared by the word's last path segment with a `.cmd`, `.exe`
  or `.ps1` ending dropped, so `./node_modules/.bin/vitest` and `vitest.cmd` are `vitest`; it need
  not be the command's first word, so
  `npx vitest --config a.ts`, `pnpm exec vitest -c a.ts` and `cross-env A=1 vitest --config a.ts`
  all name `vitest`, with no declaration of `npx`, `pnpm` or `cross-env`. Among the words after the
  program, the word after one of `names`, or the value of a word `<name>=value` where `<name>` is one
  of `names` (so `--config=a.ts` and `-c=a.ts`), names a configuration file. Each such word does,
  when the option is given more than once. Session's call: ENTRY-7 looks at the first word only because it must know which program runs
  files; here the declaration names the program, so a wrapper in front of it changes nothing. That
  file, found from the folder of the `package.json` whose script it is, is
  read by the declaration as a file its `file` glob matched is, with `relativeTo: "config"` meaning
  that file's folder and `"detection"` the deepest detection folder that holds the named file, the
  same folder ENTRY-6 would have matched the file from. Every `configEntries` declaration with
  `options` applies to the scripts of every `package.json` under the root, as a `scriptCommands`
  declaration does (ENTRY-7), since the hand-off file cannot say which packages use a tool (design
  ENT-4). Every file under the root lies in a detection folder, since the root is always the first
  (the reading spec's HO-2). A word that names no listed file gives a notice naming the script and
  the word, and adds nothing. A
  named file that is listed but does not parse is treated as ENTRY-6 treats a matched one: it is an
  entry point (CONF-3), read as any file is (the reading spec's PARSE-2), and the declaration's
  `default` stands, since no value could be read.
- **CONF-3** A configuration file named this way is an entry point, as a matched one is (ENTRY-6),
  whether or not `program` is also in a `scriptCommands` declaration: ENTRY-7 would make the word's
  file an entry point only when it is, and this declaration already says the program reads the
  file. A file both matched by `file` and named by a script is one entry point; its value is read
  with the declaration's own `relativeTo` as a matched file, and, when `options.relativeTo` differs,
  read again with that one as a named file, and the globs of both readings are used, since the tool
  reads it each way in a different command. Session's call. A fixture holds a script naming
  a second configuration file, by a separate word and by `=`, and one naming a file that does not
  exist (CURE-3).

### The draft keys

- **FREEZE-1** Once the owner decides to go on (GO-3), the drafts end: the declaration keys of the
  reading spec's KEY-1 and their shapes (the stage-1 spec's ENTRY-6, ENTRY-7 and CONF-1), the rule
  names, the fingerprint scheme, `audit`'s options and the `interface` command's answer are fixed
  under interface version 1 (design STAGE-1, ENT-4; the reading spec's RUN-7). From then a change to
  one of them raises the interface version (design IF-6). Neither `CONTRIBUTING.md` nor the README
  says this today; the change that fixes the drafts adds to `CONTRIBUTING.md`, under "The commands",
  a paragraph saying what the interface version covers and that a change to any of it raises the
  number. The README's status text is not changed, so the test that holds it (the stage-1 spec's
  DOC-1) is untouched. `scripts/contributing.test.mjs`, which holds the guide's other command text,
  gains a check that the paragraph is there. On a no-go, nothing is fixed.
- **FREEZE-2** Before they are fixed, the session lists in the judged list each key, shape or rule
  name the comparison found awkward or wrong, with what it would change, and any change the owner
  approves is made first, as an amendment (CURE-4). The list goes to the owner with the judged list,
  before GO-3; a change the owner approves with the go is made after it and before FREEZE-1, and
  CURE-5's rerun follows it like any fix; FREEZE-1's drafts end only once that is done, and any
  finding the rerun changes is added to the judged list and to the record, and told to the owner. Session's call: the comparison is the only test real
  repositories give the keys before they are fixed.

### The owner's decision

- **GO-1** When the fixes end (BAR-4), a final run of both tools on each repository's current
  default-branch head, made the same way (CMP-3 to CMP-7), checks that nothing has moved under the
  pinned results (owner). Findings it adds are judged; findings it removes are written down; and a
  finding both runs report keeps its judgment only as JDG-3 carries one, between the pinned commit
  and the final one, and is judged afresh otherwise. It runs once for each repository: an
  analyzer-only false finding or a miss it turns up is fixed as CURE-1 to CURE-5 say, and the fix is
  then checked once on that repository's final commit, where the case was found. Anything that
  second look, or that fix's CURE-5 rerun on the pinned commits and branches, turns up goes to the
  owner as BAR-4 writes a case left, and is not fixed in this item. Session's call: the anti-goal on judging that never ends.
- **GO-2** The judged list in the report folder holds, for the owner to read (design PROOF-3):
  - at the top, which of BAR-1 to BAR-3 holds, and each case that keeps the analyzer from the
    ceiling, with why;
  - the pinned and final commits, the fallow version, and the configurations both tools used, with
    each change and its reason (CMP-5, CURE-2);
  - per root, the counts of each kind for each tool, shared, analyzer-only and fallow-only, and the
    judgment of each finding JDG-1 judges, with JDG-5's section apart;
  - the branch audits, as BRA-2 and BRA-3 judge them;
  - the analyzer's times on each root, as the live check measured them (design SPEED-4);
  - each fix, with its fixture and the cases it cured.
- **GO-3** The owner reads the judged list and decides go or no-go (design PROOF-6). The decision is
  recorded on the work item's record in the owner's words, with the counts and the bar that held.
  On a go, FREEZE-1 is done and stage 3 may begin in yakshaver. On a no-go, yak keeps fallow, and the
  owner decides what becomes of the analyzer's repository and the hand-off schema (design PROOF-6).
  The worktrees of CMP-4 are removed after the decision.

## Alternatives considered

- **Judge everything again from fresh commits.** Each finding would be judged on one tree. Rejected
  (owner): the live check judged about 120 findings already, most in files that have not changed, and
  judging them again costs hours without new information. JDG-3 carries a judgment only where
  nothing the finding rests on has changed.
- **Run every pass on the latest commits.** Always current. Rejected (owner): the repositories move
  while the analyzer is fixed, so judging would never end. CMP-1 pins them, and GO-1 checks the
  latest once at the end.
- **Run fallow on a clean worktree, as the analyzer runs.** The same tree state for both. Rejected by
  the design spec's PROOF-2: fallow resolves through `node_modules`, so a clean tree is not its best
  case, and the comparison is against fallow at its best.
- **Accept the stage-1 spec's safe-side rule for `import()`.** No analyzer change. Rejected (owner): a
  truly unused export is dead code, and the rule hid one in the live check.
- **Follow every use of an `import()` result, with a flow analysis.** It would count fewest uses.
  Rejected (session's call): it is a type checker's work, and DYN-1's forms are the ones code writes;
  everything else falls back to the safe side, where a miss shows in the comparison as a fixture's
  shape to add.
- **Leave the `--config` case to stage 3, where the pack is written.** The stage-1 amendment's plan.
  Rejected (session's call, on CURE-1): reading a script's words is already the analyzer's work
  (ENTRY-7), and the fact a pack must state, which option names the configuration, is one field.
  Leaving it would leave a known false finding in the list the owner decides on.
- **Widen the pack's `configEntries` glob to match the second configuration file.** No analyzer
  change. Rejected (owner's rule, CURE-1): it would be a declaration written to make one finding
  right, and it would match any file of that name whether a script names it or not.
- **Count fallow's extra kinds as misses.** A stricter floor. Rejected: the design spec chose the kinds
  (FIND-12, FIND-7, FIND-5) on purpose, and the owner's decision on misses was about what is unused,
  not about other kinds. JDG-5 lists them so the owner sees what was set apart.
- **Compare branch audits by diffing two whole runs of fallow.** Works without fallow's audit command.
  Not needed: fallow's `audit` takes `--base` (an alias of `--changed-since`) and marks each JSON
  finding `introduced: true` or `false`, as its own help says (read from fallow 3.15.0 on
  2026-11-15; the plan's first step confirms it on 3.39.0), and its idea of new is what the gate
  would have weighed.

## Risks

- **fallow's best-case configuration is lost or stale.** Then fallow's list fills with findings its
  configuration causes. CMP-5 has the session find it, or write it again, with each change recorded;
  and every fallow-only finding is judged against the code, so a stale configuration lengthens the
  list and does not move the bar (design PROOF-2).
- **Different trees, different findings.** Two worktrees of one pinned commit hold the same tracked
  files; the installed one adds only ignored `node_modules`. CMP-4 and CMP-1 keep the commit the same,
  and the analyzer's findings do not depend on `node_modules` (the stage-1 spec's anti-goals).
- **Installing runs a private repository's dependencies.** Turning scripts off (owner) stops
  packages' install scripts, but not everything a package manager may run or fetch: pnpm loads a
  `.pnpmfile.cjs`, Yarn 2 or later runs the release a `yarnPath` names and its plugins, and Corepack
  may download the package manager a `packageManager` field names. Before installing in a folder,
  the session looks for each of these, and where one is present it tells the owner and installs
  only on their word (CMP-4a). Nothing is built, the worktrees are outside every repository, and
  fallow reads the installed files and runs none of them. A Yarn Plug'n'Play install leaves no
  `node_modules`; the judged list says so beside that root's fallow counts.
- **The two tools match findings differently.** JDG-2 fixes one matching rule for both, and BRA-3
  sets apart differences that come only from how each decides new.
- **DYN-1 counts a use that is not there.** A form read wrongly would hide a true unused export, the
  quiet direction. Each form has a fixture with a used and an unused export, and DYN-2's fall-back is
  the old rule, so no case is worse than stage 1.
- **The likeliest miss is left on the safe side.** A lazy-load wrapper, `() => import('./x')` passed
  to a router or to a `lazy` function, passes the result on, so DYN-2 counts every export of `./x`
  as used, and an unused one is a miss if fallow reports it. The comparison shows how many such
  misses there are; each is a miss under JDG-5's last sentence and is fixed or written down as BAR-4
  says, with its own shape as a fixture.
- **DYN-1 drops a use that is there.** A promise taken for a module, or a module taken for a
  promise, would count the wrong names: `p.then` as an export named `then`, or `p.a` as a use of
  `a`. DYN-1 tells the module binding from the promise binding and sends a member read off a
  promise to DYN-2, which counts every export; fixtures hold both bindings, and the rerun of CURE-5
  judges every finding a fix adds.
- **A fix for one repository breaks another.** CURE-5 reruns every root and branch after each fix.
- **Carried judgments hide a change.** A finding depends on files other than its own: an export on
  its importers, a cycle on its whole set, an unused file on any computed loader. JDG-3 carries a
  judgment only where every file the kind depends on is unchanged, and reopens it when the root's
  hand-off declarations, an entry-point field of a `package.json`, or the rule it was judged under
  changes. The cost is more judging where the repositories moved most; version bumps reopen
  nothing, since only entry-point and resolution fields count.
- **The keys are fixed too early.** FREEZE-1 waits for the owner's go and FREEZE-2 lists what the
  comparison found awkward first; a later change raises the interface version, which is the design
  spec's way (IF-6).
- **The judged list is lost.** It lives in one report folder under one machine's `.git`, and the
  2026-11-11 configuration has already been lost that way (CMP-5). Stage 3 takes the pack's
  declarations from it, and the swap the repository's (CURE-2). The pack's part names tools, not
  private files, so at GO-3 it is also written onto the work item's record, which every machine
  syncs; the counts, the bar that held and the cases left go there too. The repository's part names
  private paths and stays in the report folder; if it is lost, the swap's declarations are written
  again from each repository's code, as the live check wrote them, at the cost of that work.
- **A private name reaches a commit.** Fixtures are made up (CURE-3), the judged list stays in the
  report folder, and this spec and every amendment name no private repository (CMP-2).
- **The judging takes longer than planned.** The largest private repository had 365 fallow-only
  unused files in the live check; findings with one cause are one case (JDG-1), as the live check
  judged hundreds that way.

## Left to the builder

- How the work splits into plans and their order. The session's guess: the comparison runs and the
  judged list on the pinned commits; DYN-1 to DYN-4; CONF-1 to CONF-3; the fixes the judging turns
  up, each with its fixture; the final run, the owner's decision and FREEZE-1.
- How fallow's rule names map to the five kinds (JDG-2), written in the judged list.
- Which merged pull requests each branch audit uses (BRA-1), named in the judged list.
- The judged list's layout, and the scripts that match the two tools' output, kept in the report
  folder.
- The notice kind and wording of CONF-2's notice, and the field names `graph` uses for DYN-1's uses.
- Whether a fix the judging turns up needs its own amendment (CURE-4), and its wording.
