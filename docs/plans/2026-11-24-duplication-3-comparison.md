# Duplication, part 3: the comparison with fallow: implementation plan

**Goal:** the analyzer's duplication groups are compared with fallow's on the named repositories,
every difference is judged, the owner judges a sample, and the comparison proposes the pack's
defaults, so the owner holds one judged list and chooses the defaults from it (YARD-1 to YARD-7), and
what this item owes yakshaver is filed there (OWED-1).

**Architecture:** Task 1 renames the report test plan 1 left named for six rules, the plan's one
planned change of code. Task 2 runs both tools on pinned commits of the named roots,
matches their groups by place, judges each group only one tool finds, tries the eight settings sets,
runs the branch audits, and measures `audit --base`, all in the item's report folder, which is
committed nowhere. A refactor candidate the analyzer misses, or a group it reports that is no
candidate, is fixed on this branch with its fixture failing first, or written in the judged list as
left, with the owner's reason (YARD-5). Task 3 files OWED-1's list in the yakshaver item it names.

**Spec:** docs/specs/2026-11-23-duplication-findings-design.md

**Implements:** YARD-1 to YARD-7, OWED-1, FIXT-1

The third of the item's three plans, on its own branch, `feat/duplication-3-comparison`, stacked on
plan 2's `feat/duplication-audit-finding` (pull request #46), whose pull request goes against that
branch until #46 merges.

## Global constraints

- Nothing committed names a machine or a private repository other than yakshaver: the judged list,
  the hand-off files, fallow's configurations and every run stay in the report folder,
  `.git/yak-reports/2026-11-23-duplication-findings/cmp/`, committed nowhere (YARD-1; comparison spec
  JDG-6, CMP-2).
- fallow keeps each repository's own duplication settings, with `ignoredClones` emptied, and runs
  `fallow dupes --format json`, or the name and flags its pinned version gives (YARD-1).
- The analyzer runs `yak-analyzer duplication --hand-off <file> --all` with each settings set tried
  and no `duplicationAccepted`, so neither tool silences anything (YARD-1).
- Two groups match when at least two copies of one each share a line with a different copy of the
  other, in the same file; each group is counted once, in its own tool's counts (YARD-2).
- Each fallow-only and analyzer-only group is judged by reading its code, with a reason; the owner
  judges at least ten of each kind per repository, or all where there are fewer, and the owner's
  judgment stands (YARD-3).
- The eight sets are fallow's starting set (`strings` loosened, 50 tokens, 5 lines, 2 copies) and each
  set changing one setting: tokens 35 or 70; lines 3 or 8; `strings` not loosened; `names` loosened;
  `numbers` loosened. A further set only where two changes each lowered the misses, then the two
  together. A group is judged once (YARD-4).
- Each fix has a failing fixture of its shape first, and fixtures copy no code from fallow or the
  named repositories (YARD-5; FIXT-2; design PROOF-5).

## The decisions this plan makes

1. **The comparison reuses the complexity comparison's.** Task 2 copies
   `.git/yak-reports/2026-11-22-complexity-findings/cmp/`'s list of named roots, its scripts that pin,
   run and branch-audit (`run5.sh`, `branch5.sh`, `branches.sh`, `handoffs5.py`), and its fallow
   configurations, into this item's `cmp/`. In each configuration the `duplicates` section is set
   back to its repository's own, with `ignoredClones` emptied and the emptied entries kept in
   `cmp/configs/IGNORED.md`. `match5.py` is replaced by `matchdupes.py`, which matches groups as
   YARD-2 says, and `judged.py` keeps each group's judgment by its copies, so a group another set
   already had keeps it (YARD-4).
2. **The settings sets go in the hand-off files.** One hand-off file per root and per set, in
   `cmp/hand-off/<set>/`, its project part holding `duplicationMinimum`, `duplicationMinimumLines`,
   `duplicationCopies` and `duplicationIgnore`, and the pack's test callees: each form of `it` and
   `test` the named repositories call, read from their test files, which Task 3 then owes the pack
   (OWED-1).
3. **The misses are counted per set** as YARD-4 says, analyzer-only groups judged no refactor
   candidate plus fallow-only groups judged one, summed over every root, each repository counted
   once. The proposal is the set with the fewest; a tie goes to the set with fewer settings changed
   from fallow's. The owner chooses, and the choice goes on the record.
4. **What Task 2 finds is fixed on this branch.** A fallow-only group the owner judges a refactor
   candidate, or an analyzer-only group the owner judges none, is cured here, its fixture failing
   first in `tests/duplication.test.rs` or `tests/audit.test.rs`, as an amendment with a ruling. A
   cure that changes a rule of the spec is a further amendment of the spec, approved before the
   judged list is final. One that cannot be cured without a design change is written in the judged
   list as left, with the owner's reason (YARD-5).
5. **The times** are `audit --base` on the named root with the most files, on its pinned commit
   against its first parent, five warm runs each with the binary of the default branch where this
   item started (`0de083d`) and with this branch's, as the cold and warm measurements spec defines
   them (YARD-6). Both runs declare the starting set, so the second measures duplication's cost.
6. **The counts and the outcome go on the record** as a ruling on Task 2, since a live-check task has
   no test command, as the complexity comparison's did.
7. **The report test's name.** `prints_one_report_that_names_the_tool_the_interface_version_and_the_six_rules`
   checks seven rules since plan 1 added `duplication`; two readers raised it, and the owner chose on
   2026-11-24 to rename it here, so this plan has code to carry it as a pull request. Its check runs the
   test by its new name and passes once the name is in place; a rename has nothing after it to write,
   so the tests-first run passes, and the ruling it takes says so.

## Review focus

- **A group matched twice or not at all.** `matchdupes.py` counts a group once in its own tool's
  counts however many it matches, and tests the match from both sides (YARD-2). Its first run is
  checked by hand on one root against five groups read by eye.
- **A judgment carried to a different group.** `judged.py` carries a judgment only between groups
  that match both ways; a group whose copies grew or shrank across sets is judged afresh (YARD-4).
- **Something silenced.** Each run is checked for an empty `ignoredClones` and an empty
  `duplicationAccepted` before its counts are taken (YARD-1).
- **A branch-audit difference counted as a miss.** One that comes from FRESH-1's stricter rule is
  written down, not counted (YARD-5; design PROOF-2a).

### Task 1: The report test's name

**Implements:** FIXT-1

**Test:** `cargo test --locked --test audit -- --exact prints_one_report_that_names_the_tool_the_interface_version_and_the_seven_rules 2>&1 | grep -q "1 passed"`

**Files:**
- Modify: `tests/audit.test.rs`

**Interfaces:**
- Consumes: plan 1's addition of `duplication` to the report's rules, which the test already checks.
- Produces: the test named for the seven rules it checks.

- [ ] **Step 1: Rename the test**

```rust edit path=tests/audit.test.rs
<<<<<<< find
fn prints_one_report_that_names_the_tool_the_interface_version_and_the_six_rules() {
=======
fn prints_one_report_that_names_the_tool_the_interface_version_and_the_seven_rules() {
>>>>>>> replace
```

- [ ] **Step 2: Run the test**

```
cargo test --locked --test audit -- --exact prints_one_report_that_names_the_tool_the_interface_version_and_the_seven_rules
```

Expected: PASS, 1 test. A rename has no code to write after it, so its tests-first run passes too,
and that run's ruling says so (decision 7).

- [ ] **Step 3: Commit**

```
yak stack format
git add tests/audit.test.rs
git commit -m "test(audit): name the report test for the seven rules it checks"
```

Expected: the format changes nothing, and the commit is made.

### Task 2: The comparison with fallow

**Implements:** YARD-1, YARD-2, YARD-3, YARD-4, YARD-5, YARD-6, YARD-7

**Check:** the owner reads the report folder's `cmp/judged.md`: the pinned commits, fallow's version
and both tools' settings; per root and per set tried, the counts shared, fallow-only and analyzer-only,
with each judgment and the owner's sample; the groups each repository's `ignoredClones` silenced, with
the analyzer groups they match; the settings tried, the misses of each, the set proposed and the set
the owner chose; each fix with its fixture, and each group left with the owner's reason; the branch
audits; and YARD-6's times.

**Files:**
- None in this repository: the report folder, `.git/yak-reports/2026-11-23-duplication-findings/cmp/`,
  is committed nowhere. A cure Task 2 finds changes this branch as decision 4 says.

**Interfaces:**
- Consumes: the binary built from this branch; the complexity comparison's roots, scripts and fallow
  configurations (decision 1).
- Produces: the judged list, the defaults the owner chose, and the counts and outcome on the record.

- [ ] **Step 1: Pin, install and check fallow**

Read `install.version` in yakshaver's `tools/fallow.md` on its `main` that day. Pin each named root's
commit afresh, make two clean worktrees of each, and install fallow's from its lockfile with install
scripts off (YARD-1; comparison spec CMP-1 to CMP-4a, CMP-6). Check the duplication command's name and
flags on that version, and read each repository's duplication settings and `ignoredClones` again.

```
fallow dupes --help
```

Expected: fallow's version, the command and its flags, and each repository's duplication settings
and `ignoredClones` written in the judged list; where the command differs from `fallow dupes --format
json`, the judged list says which was run.

- [ ] **Step 2: Write the settings**

Copy the roots, scripts and configurations (decision 1). Set each configuration's `duplicates` back
to its repository's own, empty its `ignoredClones`, and write each emptied entry in
`cmp/configs/IGNORED.md`. Write one hand-off file per root and per set in `cmp/hand-off/<set>/`
(decision 2), and write `matchdupes.py` and `judged.py`.

Expected: one configuration per root, one hand-off file per root and set, `IGNORED.md` listing every
emptied entry with its repository, and the match checked by hand on one root (Review focus).

- [ ] **Step 3: Run both tools on the starting set**

On each root, from that root: fallow's duplication command with its configuration, and
`yak-analyzer duplication --hand-off <file> --all` with the starting set's hand-off file, saving each
in `cmp/runs/start/`.

Expected: every run exits 0, or the failure is written in the judged list with its output; no run
silences a group.

- [ ] **Step 4: Match, judge, and take the owner's sample**

Match each root's groups into shared, fallow-only and analyzer-only (YARD-2). Judge each fallow-only
and analyzer-only group by reading its code: a refactor candidate or not, with a reason (YARD-3).
Match the groups each `ignoredClones` silenced against the analyzer's, and list them apart. Give the
owner a sample of at least ten of each kind per repository, or all where there are fewer, and record
the owner's judgments; where the owner disagrees, read the session's other judgments of that shape
again.

Expected: each group judged in `cmp/judged.md`, the owner's sample with its judgments, and each
disagreement's shape read again.

- [ ] **Step 5: Try the settings, and propose the defaults**

Run the analyzer with each of the other seven sets, saving each in `cmp/runs/<set>/`, match each
against fallow's run, and judge only the groups no earlier set had (YARD-4). Count each set's misses
(decision 3); where two changes each lowered them, try the two together. Propose the set with the
fewest, and take the owner's sample again on its groups the first sample did not reach. The owner
chooses the defaults.

Expected: each set's counts and misses in the judged list, the set proposed, and the owner's choice
written there and on the record.

- [ ] **Step 6: Fix or leave what the owner judged**

For each fallow-only group the owner judged a refactor candidate, and each analyzer-only group the
owner judged none, under the chosen set: write a fixture of its shape that fails, then the cure, as
decision 4 says, or write it in the judged list as left with the owner's reason. Run Steps 3 to 5's
runs of the chosen set again with the cures.

```
cargo test --release --locked
```

Expected: each cure's fixture failed before its code and passes after, every test passes, and the
judged list holds each fix and each group left (YARD-5; comparison spec CURE-5).

- [ ] **Step 7: Branch audits and times**

Run the branch audits as the comparison spec's BRA-1 to BRA-3 run them, two merged pull requests per
named repository at the least, each audited with both tools under the chosen set, and judge each
difference in what is marked new; one that comes from FRESH-1's stricter rule is written down and not
counted (YARD-5). Measure the times (decision 5).

Expected: each branch audit's agreement judged in the judged list; both medians there, and sent to the
work item `2026-11-19-match-fallow-speed`.

- [ ] **Step 8: Record the counts and the outcome**

Write the judged list's top (YARD-7), and record a ruling on Task 2 that holds, per root, the shared,
fallow-only and analyzer-only counts of the chosen set, its misses, the defaults chosen, the branch
audits' agreement, and the times.

Expected: the ruling is on the record, and the owner reads the judged list at validation.

### Task 3: What is owed to yakshaver

**Implements:** OWED-1

**Check:** the owner reads, in the yakshaver work item `2026-11-18-analyzer-design-spec-stage-2-amendments`
(or its successor where it has closed), each of OWED-1's six entries filed on its record, with the
defaults Task 2's owner chose and the test callees decision 2 read; or, in the judged list, each entry
written as still owed there.

**Files:**
- None in this repository: each entry is filed on the yakshaver item's record.

**Interfaces:**
- Consumes: Task 2's chosen defaults and the test callees read for its hand-off files.
- Produces: OWED-1's entries on the yakshaver item's record.

- [ ] **Step 1: File each entry**

Read the yakshaver item's state. Where it is open, file on its record, each as one entry: FIND-9's and
PROOF-4's amendment; BASE-5's amendment to FRESH-1's rule; DECL-2's amendment for `duplicationIgnore`
and `duplicationTestCallees`; the pack's declarations of the new keys, with the chosen defaults and
the test callees, and the engine's way to add an entry to `duplicationAccepted`; the release-notes
line for a change to the default `duplicationIgnore` or `duplicationTestCallees`; and the check that
yak passes the three lists unchanged, with the hand-off schema's description. Where it has closed,
file them on its successor, or write each in the judged list as still owed.

```
yak work show --item 2026-11-18-analyzer-design-spec-stage-2-amendments
```

Expected: the item's state; then each of the six entries on its record, or each written in the
judged list as still owed there.

- [ ] **Step 2: Record what was filed**

Record a ruling on Task 3 naming each entry and where it was filed.

Expected: the ruling is on the record, and the owner reads it at validation.
