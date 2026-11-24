# Duplication, part 2: the finding in audit, new against inherited: implementation plan

**Goal:** `audit` reports each group of copies a person has not accepted as one `duplication` finding,
placed at its copies with their spans, its size in its property bag, and with a base marks it new or
unchanged by comparing its copies with the base's groups of the same fingerprint, as FRESH-1 says.

**Architecture:** `findings::find` makes duplication findings from the same `duplication::groups` the
command uses (SHOW-5), each carrying its `Group`. A `Location` gains the last line a copy runs to,
which `audit` writes as the region's `endLine`, and the property bag is built from the finding's
locations, so its copies follow DUP-2's order. Each statement keeps a second digest, of its exact
form with nothing loosened, written beside the loosened one in the same walk only where the lens
loosens something. Each copy carries the exact form of its run. `inherit::mark` compares duplication
findings apart from the other kinds: it groups the base's reading, pairs copies by file and exact
form, and places a new group at its first copy the base lacks.

**Spec:** docs/specs/2026-11-23-duplication-findings-design.md

**Implements:** DUP-1, DUP-2, DUP-3, FRESH-1 to FRESH-4, VERS-1

The second of the item's three plans, on its own branch, `feat/duplication-audit-finding`, stacked on
plan 1's `feat/duplication-findings` (pull request #45), whose pull request goes against that branch
until #45 merges. Plan 1's text says this plan is written from `main` once plan 1 has merged; the
owner chose on 2026-11-24 to stack it instead. Plan 3 is the comparison with fallow (YARD-1 to YARD-7,
OWED-1).

## Global constraints

- One finding per group `audit` would report, under the rule `duplication`; an accepted group gives
  none, and still hides the groups inside it (DUP-1, QUIET-1).
- A finding names places only, never the copied code (the spec's anti-goals).
- A duplication finding is `unchanged` only when the base has a group of the same fingerprint and the
  same number of copies, accepted ones included, whose copies pair one to one with its own, each in
  the same file, the base's path moved through git's renames, with the same exact form; otherwise
  `new` (FRESH-1).
- FRESH-1 reads no line differences: only the base's groups, from the base's reading, with the
  declarations in force now (FRESH-2; design BASE-3).
- A file skipped at the base leaves duplication findings compared, unless it is a script; the
  dead-code kinds keep COMMIT-4's rule (FRESH-4).
- No finding of the other six kinds, no fingerprint and no message of theirs changes, and nothing
  `graph` prints changes (the spec's anti-goals).
- The interface version stays 1 (VERS-1).

## The decisions this plan makes

1. **The exact form is written in the same walk.** `Former` keeps a second stream, written only where
   the lens loosens a word: each piece goes to both, a loosened value goes to the form as its
   placeholder and to the exact form as itself. A statement's `exact` digest is its digest where
   nothing is loosened, so a project that loosens nothing pays nothing for it. A copy's `exact` is
   the fingerprint its run would have from those digests, made the same way as the group's (Left to
   the builder: "the same form as DUP-4's, with nothing loosened, or a digest of it").
2. **A script skipped at the base is always one the branch changed or removed.** A tracked file that
   fails to parse stops the working tree's reading, and one the branch made untracked shows as
   removed. So no branch whose run goes on has a skipped script left as it was. FRESH-4's test,
   "where the branch removed or changed any script skipped at the base", is therefore "where any
   script was skipped at the base", with no read of git's changed files. Its fixture for a skipped
   file that leaves the groups compared is a skipped `tsconfig.json`.
3. **The notice for that case** is a kind of its own, `duplication-not-compared`, naming the scripts.
   `base-files-skipped` says that complexity and duplication findings are still compared, as FRESH-4
   asks of COMMIT-4's notice. Where no `duplicationMinimum` is declared there is nothing to compare,
   and no such notice is given.
4. **The message.** "3 copies of a 5-line run (20 tokens): `src/a.ts` lines 2–6, `src/b.ts` lines
   2–6, `src/c.ts` lines 2–6.", the copies by path and line. A span is always written "lines a–b"
   and a size "n-line", even for one line, since a run of one line is rare and the form stays one.
   A new group's message adds, by FRESH-3: "New or changed since the base: …" with the copies that
   pair with none of the base's group in their file with their exact form; or, where every copy
   pairs, "At the base this run had 3 copies."; or, where the base has no group of the fingerprint,
   "The base reports no group of this run." rather than naming every copy again.
5. **A new group's first location** is its first copy, by path and line, whose exact form no copy of
   the base's groups of its fingerprint has, else its first copy (FRESH-3). The other copies follow
   in path and line order. The finding's locations are moved and the findings sorted again; the
   property bag is built from the locations, so its copies follow.
6. **The base's dead-code findings are found alone.** `inherit::mark` counted fingerprints over the
   base's whole `findings::find`, then left out cycles and complexity. `findings::counted` finds only
   the unused files, exports and types and the unresolved imports, which `find` now starts from, so
   the base's groups are found once, by the duplication comparison.
7. **The fixtures for the base** go in `tests/audit.test.rs`, beside the base tests whose helpers
   (`branched`, `audited`, `marked`, `sentence`) they use. A group's copies are made with
   `counting(name, number)`, a function whose body differs from another group's only in its number,
   so groups in one project never meet; `labelled(name, label)` holds a string, for loosening.
8. **The existing tests that list `audit`'s notifications** each gain the TUNE-1 notice,
   `duplication-not-checked`, since their projects declare no `duplicationMinimum`; the one that
   compares them with `graph`'s notices leaves out both "not checked" notices, and the two that read a
   notification or a sentence by its place move one along.

## Review focus

- **A base group counted as a dead-code finding.** `mark` counts only `findings::counted`, and skips
  `Rule::Duplication` in the dead-code loop; otherwise a group would be inherited by fingerprint alone
  or always new. Pinned by the unchanged groups of
  `a_group_is_unchanged_only_where_the_base_has_each_of_its_copies_in_its_file_as_it_was`, Task 2.
- **The early return for a skipped base file.** Duplication findings are marked before it, and the
  findings sorted again on that path too, since a new group may move its first location. Pinned by
  `a_file_skipped_at_the_base_leaves_the_groups_compared_unless_it_is_a_script`, Task 2.
- **The exact stream's ranges.** Each statement notes where its exact form starts and ends as the walk
  enters and leaves it, as it does for its form; a range read from the wrong stream would give every
  copy the same exact form. Pinned by `label-2` in the first Task 2 test, where only a string differs
  and strings are loosened.
- **Two base groups of one fingerprint.** `held` keeps every base group of a fingerprint, and a group
  is unchanged where any one of them pairs; FRESH-3's partners are taken from all of them. No fixture
  makes two such base groups, which SAME-4 leaves only for runs whose copies lie apart; the code reads
  each, so none is lost.
- **A copy's line endings.** The exact form is built from the tree's values, with every line ending
  written as a line feed, as the form is (DUP-5). Pinned by the CRLF copy in
  `a_rewrap_leaves_a_group_as_it_was_unless_it_takes_it_over_the_line_minimum`, Task 2.

### Task 1: The finding and its property bag in audit

**Implements:** DUP-1, DUP-2, DUP-3

**Test:** `cargo test --locked --test audit --test cli`

**Files:**
- Modify: `src/findings.rs`, `src/duplication.rs`, `src/audit.rs`, `src/inherit.rs`,
  `tests/audit.test.rs`, `tests/cli.test.rs`

**Interfaces:**
- Consumes: plan 1's `duplication::{groups, Group, Place}`, `Settings::notice` and
  `findings::fingerprint`.
- Produces: `Location::end_line` and `impl From<&Place> for Location`; `Finding::group`;
  `duplication::{message, spans}`; the result's `properties.duplication` and each location's
  `endLine`; the TUNE-1 and QUIET-2 notices in `audit`. With a base, a duplication finding is marked
  new here, since the base's are counted under a fingerprint of no parts; Task 2 compares its copies.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/audit.test.rs
<<<<<<< find
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4), complexity findings by
//! their scores (complexity spec CPX-1 to CPX-3, NEW-1 to NEW-5, FN-6, CASE-1).

#[path = "support/project.rs"]
mod project;
=======
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4), complexity findings by
//! their scores (complexity spec CPX-1 to CPX-3, NEW-1 to NEW-5, FN-6, CASE-1); and duplication findings
//! (duplication spec DUP-1 to DUP-3, QUIET-1, QUIET-2).

#[path = "support/project.rs"]
mod project;
>>>>>>> replace
<<<<<<< find
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("no-hand-off-file", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
    // Besides the notice of the limits it checks complexity against, which `graph` does not read
    // (complexity spec LIMIT-2).
    assert_eq!(
        notifications(&run)
            .iter()
            .map(|(kind, _)| *kind)
            .filter(|kind| *kind != "complexity-not-checked")
            .collect::<Vec<&str>>(),
        notices(&document)
    );
=======
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("duplication-not-checked", None),
            ("no-hand-off-file", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
    // Besides the notices of the limits it checks complexity against and of the smallest run it looks
    // for, which `graph` does not read (complexity spec LIMIT-2; duplication spec TUNE-1).
    assert_eq!(
        notifications(&run)
            .iter()
            .map(|(kind, _)| *kind)
            .filter(|kind| !kind.ends_with("-not-checked"))
            .collect::<Vec<&str>>(),
        notices(&document)
    );
>>>>>>> replace
<<<<<<< find
        notifications(&audit(&project, Some(&hand_off))),
        [
            ("complexity-not-checked", None),
            ("glob-matches-nothing", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
=======
        notifications(&audit(&project, Some(&hand_off))),
        [
            ("complexity-not-checked", None),
            ("duplication-not-checked", None),
            ("glob-matches-nothing", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
>>>>>>> replace
<<<<<<< find
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("kinds-not-reported", None),
            ("kinds-not-reported", None),
        ]
=======
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("duplication-not-checked", None),
            ("kinds-not-reported", None),
            ("kinds-not-reported", None),
        ]
>>>>>>> replace
<<<<<<< find
        .collect();

    assert_eq!(
        sentences[1..],
        [
            "1 file(s) the analyzer reads are in no package, so their unused files, exports and types are \
             not reported.",
=======
        .collect();

    assert_eq!(
        sentences[2..],
        [
            "1 file(s) the analyzer reads are in no package, so their unused files, exports and types are \
             not reported.",
>>>>>>> replace
<<<<<<< find
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("no-hand-off-file", None),
            ("computed-import", Some("src/index.ts")),
        ]
    );
    assert_eq!(
        run["invocations"][0]["toolExecutionNotifications"][2]["message"]["text"],
        json!(
            "2 call(s) of `import()` or `require()` load a file the analyzer cannot work out, so what they \
             load must be declared in `entries`."
=======
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("duplication-not-checked", None),
            ("no-hand-off-file", None),
            ("computed-import", Some("src/index.ts")),
        ]
    );
    assert_eq!(
        run["invocations"][0]["toolExecutionNotifications"][3]["message"]["text"],
        json!(
            "2 call(s) of `import()` or `require()` load a file the analyzer cannot work out, so what they \
             load must be declared in `entries`."
>>>>>>> replace
<<<<<<< find
        [
            ("base", None, false),
            ("complexity-not-checked", None, false),
            ("no-hand-off-file", None, false),
            (
                "tsconfig-extends-left-out",
=======
        [
            ("base", None, false),
            ("complexity-not-checked", None, false),
            ("duplication-not-checked", None, false),
            ("no-hand-off-file", None, false),
            (
                "tsconfig-extends-left-out",
>>>>>>> replace
<<<<<<< find
            ("base", None, false),
            ("base-files-skipped", None, false),
            ("complexity-not-checked", None, false),
            ("no-hand-off-file", None, false),
            ("file-skipped", Some("src/broken.ts"), true),
            ("file-skipped", Some("tsconfig.json"), true),
=======
            ("base", None, false),
            ("base-files-skipped", None, false),
            ("complexity-not-checked", None, false),
            ("duplication-not-checked", None, false),
            ("no-hand-off-file", None, false),
            ("file-skipped", Some("src/broken.ts"), true),
            ("file-skipped", Some("tsconfig.json"), true),
>>>>>>> replace
<<<<<<< find
        json!({ "cyclomatic": 7, "cognitive": 6 })
    );
}
=======
        json!({ "cyclomatic": 7, "cognitive": 6 })
    );
}

/// A function whose body is the same whatever its name: its statements are lines 2 to 6 of its file.
fn body(name: &str) -> String {
    format!(
        "export function {name}(items: number[]) {{\n  const total = items.length;\n  if (total > 10) {{\n    return \
         total * 2;\n  }}\n  return total;\n}}\n"
    )
}

/// A hand-off file whose project declares what is given.
fn declaring(project: &Project, declared: Value) -> String {
    project.hand_off(&project.root, &["."], stack(json!({}), declared))
}

/// A copy of a group as a run's finding names it: its file as the report writes it, with its first and
/// last lines.
type Copy<'r> = (&'r str, u64, u64);

/// The duplication findings of a run, in its order: each with its copies, its first location first, and
/// whether it is new or inherited where the run says.
fn duplicates(run: &Value) -> Vec<(Vec<Copy<'_>>, Option<&str>)> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .filter(|result| result["ruleId"] == json!("duplication"))
        .map(|result| {
            let copies = result["locations"]
                .as_array()
                .into_iter()
                .chain(result["relatedLocations"].as_array())
                .flatten()
                .map(|location| {
                    let place = &location["physicalLocation"];

                    (
                        place["artifactLocation"]["uri"].as_str().expect("a file"),
                        place["region"]["startLine"].as_u64().expect("a first line"),
                        place["region"]["endLine"].as_u64().expect("a last line"),
                    )
                })
                .collect();

            (copies, result["baselineState"].as_str())
        })
        .collect()
}

#[test]
fn reports_each_group_once_at_its_copies_with_its_size_in_its_property_bag() {
    let project = Project::new("duplication-findings");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { a } from './a';\nimport { b } from './b';\nimport { c } from './c';\n\na([]);\nb([]);\nc([]);\n",
        )
        .write("src/a.ts", &body("a"))
        .write("src/b.ts", &body("b"))
        .write("src/c.ts", &body("c"))
        .commit();

    let hand_off = declaring(&project, json!({ "duplicationMinimum": 10 }));
    let run = audit(&project, Some(&hand_off));
    let result = run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .find(|result| result["ruleId"] == json!("duplication"))
        .expect("a duplication finding");

    // One finding for the group, at its first copy by path and line, with the others related; each
    // location runs from the copy's first line to its last (duplication spec DUP-1, DUP-2).
    assert_eq!(
        duplicates(&run),
        [(
            vec![("src/a.ts", 2, 6), ("src/b.ts", 2, 6), ("src/c.ts", 2, 6)],
            None
        )]
    );
    assert_eq!(
        result["message"]["text"],
        json!(
            "3 copies of a 5-line run (20 tokens): `src/a.ts` lines 2–6, `src/b.ts` lines 2–6, \
             `src/c.ts` lines 2–6."
        )
    );
    // Its property bag holds its size and its copies in the order of its locations (duplication spec
    // DUP-3).
    assert_eq!(
        result["properties"]["duplication"],
        json!({
            "copies": [
                { "endLine": 6, "path": "src/a.ts", "startLine": 2 },
                { "endLine": 6, "path": "src/b.ts", "startLine": 2 },
                { "endLine": 6, "path": "src/c.ts", "startLine": 2 },
            ],
            "lines": 5,
            "tokens": 20,
        })
    );
    // `audit` and `duplication` find the groups with the same function (duplication spec SHOW-5).
    let document = printed(&project.start("duplication", |binary| {
        binary.args(["--hand-off", &hand_off]);
    }));
    let fingerprint = result["partialFingerprints"]["yakAnalyzer/v1"]
        .as_str()
        .expect("a fingerprint");

    assert_eq!(document["groups"][0]["fingerprint"], json!(fingerprint));
    assert!(!notifications(&run).contains(&("duplication-not-checked", None)));

    // An accepted group is not reported; an entry with another number of copies matches nothing, so the
    // group is reported and the entry named in a notice (duplication spec QUIET-1, QUIET-2).
    let accepting = |copies: usize| {
        let hand_off = declaring(
            &project,
            json!({
                "duplicationMinimum": 10,
                "duplicationAccepted": [format!("{fingerprint}/{copies}")],
            }),
        );

        audit(&project, Some(&hand_off))
    };
    let accepted = accepting(3);
    let stale = accepting(2);

    assert!(duplicates(&accepted).is_empty());
    assert!(!notifications(&accepted).contains(&("accepted-group-not-found", None)));
    assert_eq!(duplicates(&stale).len(), 1);
    assert!(notifications(&stale).contains(&("accepted-group-not-found", None)));
}
>>>>>>> replace
```

```rust edit path=tests/cli.test.rs
<<<<<<< find
        Some(7)
    );
    assert_eq!(run["results"], json!([]));
    assert_eq!(notifications.as_array().map(Vec::len), Some(2));
    assert_eq!(
        notifications[1]["descriptor"]["id"],
        json!("no-hand-off-file")
    );
}
=======
        Some(7)
    );
    assert_eq!(run["results"], json!([]));
    assert_eq!(notifications.as_array().map(Vec::len), Some(3));
    assert_eq!(
        notifications[2]["descriptor"]["id"],
        json!("no-hand-off-file")
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```
cargo test --locked --test audit --test cli
```

Expected: FAIL. `reports_each_group_once_at_its_copies_with_its_size_in_its_property_bag` finds no
duplication finding, and each test that lists the notifications lacks `duplication-not-checked`.

- [ ] **Step 3: Write the code**

```rust edit path=src/findings.rs
<<<<<<< find
//! What `audit` reports: the findings a reading shows, each with its rule, its message, its location
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
=======
//! What `audit` reports: the findings a reading shows, each with its rule, its message, its location
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3; duplication spec DUP-1, DUP-2).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
>>>>>>> replace
<<<<<<< find
use sha2::{Digest, Sha256};

use crate::complexity::{self, Detail, Scores};
use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::imports::{Form, Import};
=======
use sha2::{Digest, Sha256};

use crate::complexity::{self, Detail, Scores};
use crate::duplication::{self, Group, Place};
use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::imports::{Form, Import};
>>>>>>> replace
<<<<<<< find
    pub line: Option<usize>,
    /// The column, counted from one in UTF-16 code units.
    pub column: Option<usize>,
}

/// Whether a finding is new on the branch or the base has it too (spec REPORT-1, INH-1).
=======
    pub line: Option<usize>,
    /// The column, counted from one in UTF-16 code units.
    pub column: Option<usize>,
    /// The last line, where what it names runs over lines: a copy of a duplication group's (duplication
    /// spec DUP-2).
    pub end_line: Option<usize>,
}

impl From<&Place> for Location {
    /// Where a copy of a group is: its file, from its first line to its last (duplication spec DUP-2).
    fn from(copy: &Place) -> Self {
        Self {
            path: copy.path.clone(),
            line: Some(copy.start_line),
            column: None,
            end_line: Some(copy.end_line),
        }
    }
}

/// Whether a finding is new on the branch or the base has it too (spec REPORT-1, INH-1).
>>>>>>> replace
<<<<<<< find
    /// What a complexity finding carries in its property bag; none for any other (complexity spec
    /// CPX-3).
    pub complexity: Option<Detail>,
}

impl Finding {
=======
    /// What a complexity finding carries in its property bag; none for any other (complexity spec
    /// CPX-3).
    pub complexity: Option<Detail>,
    /// The group of a duplication finding, whose size its property bag carries; none for any other
    /// (duplication spec DUP-3).
    pub group: Option<Group>,
}

impl Finding {
>>>>>>> replace
<<<<<<< find
            parts: parts.iter().map(|part| (*part).to_owned()).collect(),
            baseline: None,
            complexity: None,
        }
    }

    /// The fingerprint the finding would have with each path it names moved as `moved` says, as a
    /// rename moves a base's file to its path on the branch (spec INH-2). A cycle's paths are sorted
    /// again after they move.
=======
            parts: parts.iter().map(|part| (*part).to_owned()).collect(),
            baseline: None,
            complexity: None,
            group: None,
        }
    }

    /// A group's finding: at its first copy, with its other copies as related locations, and the
    /// fingerprint the group was given, which holds no path; none for a group with no copy (duplication
    /// spec DUP-1, DUP-2, DUP-4).
    fn grouped(group: Group) -> Option<Self> {
        let mut locations = group.copies.iter().map(Location::from);
        let location = locations.next()?;
        let related = locations.collect();

        Some(Self {
            rule: Rule::Duplication,
            message: duplication::message(&group),
            location,
            related,
            fingerprint: group.fingerprint.clone(),
            parts: Vec::new(),
            baseline: None,
            complexity: None,
            group: Some(group),
        })
    }

    /// The fingerprint the finding would have with each path it names moved as `moved` says, as a
    /// rename moves a base's file to its path on the branch (spec INH-2). A cycle's paths are sorted
    /// again after they move.
>>>>>>> replace
<<<<<<< find
    unresolved(reading, &mut found);
    circular(reading, &mut found);
    complex(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
    notices.extend(reading.limits.notice());
=======
    unresolved(reading, &mut found);
    circular(reading, &mut found);
    complex(reading, &mut found);
    duplicated(reading, &mut found, &mut notices);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
    notices.extend(reading.limits.notice());
>>>>>>> replace
<<<<<<< find
                    path: path.to_owned(),
                    line: None,
                    column: None,
                },
                &[path],
            ));
=======
                    path: path.to_owned(),
                    line: None,
                    column: None,
                    end_line: None,
                },
                &[path],
            ));
>>>>>>> replace
<<<<<<< find
                    path: path.to_owned(),
                    line: Some(export.line),
                    column: Some(export.column),
                },
                &[path, name],
            ));
=======
                    path: path.to_owned(),
                    line: Some(export.line),
                    column: Some(export.column),
                    end_line: None,
                },
                &[path, name],
            ));
>>>>>>> replace
<<<<<<< find
                        path: path.clone(),
                        line: Some(import.line),
                        column: None,
                    },
                    &[path, &import.specifier],
                ));
=======
                        path: path.clone(),
                        line: Some(import.line),
                        column: None,
                        end_line: None,
                    },
                    &[path, &import.specifier],
                ));
>>>>>>> replace
<<<<<<< find
                .min()
                .map(|(line, _)| line),
            column: None,
        };
        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let mut finding = Finding::new(
=======
                .min()
                .map(|(line, _)| line),
            column: None,
            end_line: None,
        };
        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let mut finding = Finding::new(
>>>>>>> replace
<<<<<<< find
                path: measured.path.clone(),
                line: Some(scored.line),
                column: Some(scored.column),
            },
            &parts,
        );
=======
                path: measured.path.clone(),
                line: Some(scored.line),
                column: Some(scored.column),
                end_line: None,
            },
            &parts,
        );
>>>>>>> replace
<<<<<<< find
    }
}

/// One notice for each file with an `import()` or `require()` it cannot follow: what it loads must be
/// declared (spec ENTRY-10).
fn computed(reading: &Reading, notices: &mut Vec<Notice>) {
=======
    }
}

/// One finding for each group of copies a person has not accepted, with the notices for an entry of
/// `duplicationAccepted` that matches no group and for a run with no `duplicationMinimum` (duplication
/// spec DUP-1, TUNE-1, QUIET-1, QUIET-2).
fn duplicated(reading: &Reading, found: &mut Vec<Finding>, notices: &mut Vec<Notice>) {
    let (groups, accepted) = duplication::groups(reading);

    found.extend(
        groups
            .into_iter()
            .filter(|group| !group.accepted)
            .filter_map(Finding::grouped),
    );
    notices.extend(accepted);
    notices.extend(reading.duplication.notice());
}

/// One notice for each file with an `import()` or `require()` it cannot follow: what it loads must be
/// declared (spec ENTRY-10).
fn computed(reading: &Reading, notices: &mut Vec<Notice>) {
>>>>>>> replace
```

```rust edit path=src/duplication.rs
<<<<<<< find
//! The `duplication` command, and the groups of copies a reading holds, which `audit` will report too:
//! the settings the declarations give, the runs of statements that repeat, the groups they make with
//! their sizes and fingerprints, the groups a person accepted, and the document the command prints
//! (duplication spec SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7, DUP-4, QUIET-1 to QUIET-4,
//! SHOW-1 to SHOW-5).

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
=======
//! The `duplication` command, and the groups of copies a reading holds, which `audit` reports too:
//! the settings the declarations give, the runs of statements that repeat, the groups they make with
//! their sizes and fingerprints, the groups a person accepted, the message of a group's finding, and the
//! document the command prints (duplication spec SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7,
//! DUP-2, DUP-4, QUIET-1 to QUIET-4, SHOW-1 to SHOW-5).

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
>>>>>>> replace
<<<<<<< find
    (groups, notices)
}

/// The entry of `duplicationAccepted` that accepts a group: its fingerprint, `/`, and its number of
/// copies (spec TUNE-6).
fn accepting(group: &Group) -> String {
=======
    (groups, notices)
}

/// The message of a group's finding: how many copies it has, its size, and where each copy is, with no
/// code (spec DUP-2).
pub fn message(group: &Group) -> String {
    format!(
        "{} copies of a {}-line run ({} tokens): {}.",
        group.copies.len(),
        group.lines,
        group.tokens,
        spans(&group.copies)
    )
}

/// Where each copy is, as its file and its span from its first line to its last, joined by commas.
pub fn spans<'p>(copies: impl IntoIterator<Item = &'p Place>) -> String {
    copies
        .into_iter()
        .map(|copy| {
            format!(
                "`{}` lines {}–{}",
                copy.path, copy.start_line, copy.end_line
            )
        })
        .collect::<Vec<String>>()
        .join(", ")
}

/// The entry of `duplicationAccepted` that accepts a group: its fingerprint, `/`, and its number of
/// copies (spec TUNE-6).
fn accepting(group: &Group) -> String {
>>>>>>> replace
```

```rust edit path=src/audit.rs
<<<<<<< find
        result["properties"] = json!({ "complexity": bag(detail) });
    }

    if !finding.related.is_empty() {
        result["relatedLocations"] = finding
            .related
=======
        result["properties"] = json!({ "complexity": bag(detail) });
    }

    // A duplication finding's copies are its locations, in their order (duplication spec DUP-3).
    if let Some(group) = &finding.group {
        result["properties"] = json!({ "duplication": {
            "copies": std::iter::once(&finding.location)
                .chain(&finding.related)
                .map(|copy| json!({ "endLine": copy.end_line, "path": copy.path, "startLine": copy.line }))
                .collect::<Vec<Value>>(),
            "lines": group.lines,
            "tokens": group.tokens,
        } });
    }

    if !finding.related.is_empty() {
        result["relatedLocations"] = finding
            .related
>>>>>>> replace
<<<<<<< find
    bag
}

/// A location as SARIF writes one: the file, and the line and column where there are some.
fn placed(location: &Location) -> Value {
    let mut physical = json!({ "artifactLocation": artifact(&location.path) });

=======
    bag
}

/// A location as SARIF writes one: the file, and the line, the column and the last line where there are
/// some.
fn placed(location: &Location) -> Value {
    let mut physical = json!({ "artifactLocation": artifact(&location.path) });

>>>>>>> replace
<<<<<<< find
        if let Some(column) = location.column {
            physical["region"]["startColumn"] = json!(column);
        }
    }

    json!({ "physicalLocation": physical })
=======
        if let Some(column) = location.column {
            physical["region"]["startColumn"] = json!(column);
        }

        if let Some(end) = location.end_line {
            physical["region"]["endLine"] = json!(end);
        }
    }

    json!({ "physicalLocation": physical })
>>>>>>> replace
```

```rust edit path=src/inherit.rs
<<<<<<< find
        path: from.to_owned(),
        line: Some(line),
        column: None,
    };
    finding.related = locations;
    let _ = write!(
=======
        path: from.to_owned(),
        line: Some(line),
        column: None,
        end_line: None,
    };
    finding.related = locations;
    let _ = write!(
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```
cargo test --locked --test audit --test cli --test duplication
```

Expected: PASS, every test of the three targets.

- [ ] **Step 5: Format, check and commit**

```
yak stack format
cargo clippy --release --all-targets --locked -- -D warnings
git add src/findings.rs src/duplication.rs src/audit.rs src/inherit.rs tests/audit.test.rs tests/cli.test.rs
git commit -m "feat(duplication): report each group as a finding in audit"
```

Expected: the format changes nothing, clippy prints no warning, and the commit is made.

### Task 2: New against inherited

**Implements:** FRESH-1, FRESH-2, FRESH-3, FRESH-4

**Test:** `cargo test --locked --test audit`

**Files:**
- Modify: `src/forms.rs`, `src/duplication.rs`, `src/findings.rs`, `src/inherit.rs`,
  `tests/audit.test.rs`

**Interfaces:**
- Consumes: Task 1's `Finding::group`, `Location::from(&Place)` and `duplication::spans`; plan 1's
  `forms::Item`, `Former` and `duplication::groups`.
- Produces: `forms::Item::exact`; `duplication::Place::exact`; `findings::counted`; duplication
  findings marked new or unchanged by `inherit::mark`, a new one placed and its message added to by
  FRESH-3; the `duplication-not-compared` notice; `base-files-skipped`'s sentence naming duplication.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/audit.test.rs
<<<<<<< find
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4), complexity findings by
//! their scores (complexity spec CPX-1 to CPX-3, NEW-1 to NEW-5, FN-6, CASE-1); and duplication findings
//! (duplication spec DUP-1 to DUP-3, QUIET-1, QUIET-2).

#[path = "support/project.rs"]
mod project;
=======
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4), complexity findings by
//! their scores (complexity spec CPX-1 to CPX-3, NEW-1 to NEW-5, FN-6, CASE-1), duplication findings by
//! their groups' copies (duplication spec DUP-1 to DUP-3, QUIET-1, QUIET-2, FRESH-1 to FRESH-4).

#[path = "support/project.rs"]
mod project;
>>>>>>> replace
<<<<<<< find
    assert_eq!(
        sentence(&run, "base-files-skipped"),
        "2 file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
         unchanged; complexity findings are still compared, and those in a skipped script are new: \
         `tsconfig.json`, `src/broken.ts`."
    );
}

=======
    assert_eq!(
        sentence(&run, "base-files-skipped"),
        "2 file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
         unchanged; complexity and duplication findings are still compared, and those in a skipped \
         script are new: `tsconfig.json`, `src/broken.ts`."
    );
}

>>>>>>> replace
<<<<<<< find
    assert_eq!(duplicates(&stale).len(), 1);
    assert!(notifications(&stale).contains(&("accepted-group-not-found", None)));
}
=======
    assert_eq!(duplicates(&stale).len(), 1);
    assert!(notifications(&stale).contains(&("accepted-group-not-found", None)));
}

/// A function whose body differs from another's only where its number does: its statements are lines 2
/// to 6 of the function, a run of 20 tokens.
fn counting(name: &str, number: u32) -> String {
    format!(
        "export function {name}(items: number[]) {{\n  const total = items.length + {number};\n  if (total > \
         {number}) {{\n    return total * {number};\n  }}\n  return total;\n}}\n"
    )
}

/// A function whose body holds a string: its statements are lines 2 to 4 of the function.
fn labelled(name: &str, label: &str) -> String {
    format!(
        "export function {name}(items: number[]) {{\n  const total = items.length + 40;\n  console.log(\"{label}\", \
         total);\n  return total * 40;\n}}\n"
    )
}

#[test]
fn a_group_is_unchanged_only_where_the_base_has_each_of_its_copies_in_its_file_as_it_was() {
    let project = branched(
        "base-duplication",
        &[
            ("src/pasted-1.ts", &counting("p1", 1)),
            ("src/pasted-2.ts", &counting("p2", 1)),
            ("src/removed-1.ts", &counting("r1", 2)),
            ("src/removed-2.ts", &counting("r2", 2)),
            ("src/removed-3.ts", &counting("r3", 2)),
            ("src/around-1.ts", &counting("a1", 3)),
            ("src/around-2.ts", &counting("a2", 3)),
            ("src/renamed-1.ts", &counting("n1", 4)),
            ("src/renamed-2.ts", &counting("n2", 4)),
            ("src/moved-1.ts", &counting("m1", 5)),
            ("src/moved-2.ts", &counting("m2", 5)),
            ("src/edited-1.ts", &counting("e1", 6)),
            ("src/edited-2.ts", &counting("e2", 6)),
            ("src/swapped-1.ts", &counting("s1", 7)),
            ("src/swapped-2.ts", &counting("s2", 7)),
            ("src/label-1.ts", &labelled("l1", "first")),
            ("src/label-2.ts", &labelled("l2", "first")),
        ],
    );

    project.git(&["rm", "--quiet", "src/removed-3.ts"]);
    project.git(&["mv", "src/renamed-2.ts", "src/kept-2.ts"]);
    project
        // A copy pasted: the group has one more copy than the base's.
        .write("src/pasted-3.ts", &counting("p3", 1))
        // Edits only around a copy move its lines, not its run.
        .write(
            "src/around-1.ts",
            &format!("export const before = 1;\n{}", counting("a1", 3)),
        )
        // A copy moved to another file.
        .write("src/moved-2.ts", "export const gone = 1;\n")
        .write("src/moved-3.ts", &counting("m3", 5))
        // A line fixed in one of two copies: no run of the size looked for is left.
        .write(
            "src/edited-2.ts",
            &counting("e2", 6).replace("return total * 6", "return total * 60"),
        )
        // One copy edited away while another is pasted elsewhere: as many copies as at the base.
        .write("src/swapped-2.ts", "export const gone = 2;\n")
        .write("src/swapped-3.ts", &counting("s3", 7))
        // With strings loosened, a string changed in one copy leaves the fingerprint as it was.
        .write("src/label-2.ts", &labelled("l2", "second"))
        // A group only the branch has.
        .write("src/only-1.ts", &counting("o1", 8))
        .write("src/only-2.ts", &counting("o2", 8))
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "duplicationMinimum": 10, "duplicationIgnore": ["strings"] }),
    );
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);
    let messages: Vec<&str> = run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .filter(|result| {
            result["ruleId"] == json!("duplication") && result["baselineState"] == json!("new")
        })
        .filter_map(|result| result["message"]["text"].as_str())
        // What the comparison adds follows the sentence every group's message has.
        .filter_map(|message| message.split_once(". ").map(|(_, added)| added))
        .collect();

    // A group is unchanged where the base has a group of its fingerprint whose copies pair with its own,
    // each in the same file, its path moved through git's renames, with the same exact form; a new
    // group is placed at its first copy whose exact form the base lacks, and its message says what
    // changed (duplication spec FRESH-1, FRESH-3).
    assert_eq!(
        duplicates(&run),
        [
            (
                vec![("src/around-1.ts", 3, 7), ("src/around-2.ts", 2, 6)],
                Some("unchanged")
            ),
            (
                vec![("src/kept-2.ts", 2, 6), ("src/renamed-1.ts", 2, 6)],
                Some("unchanged")
            ),
            (
                vec![("src/label-2.ts", 2, 4), ("src/label-1.ts", 2, 4)],
                Some("new")
            ),
            (
                vec![("src/moved-1.ts", 2, 6), ("src/moved-3.ts", 2, 6)],
                Some("new")
            ),
            (
                vec![("src/only-1.ts", 2, 6), ("src/only-2.ts", 2, 6)],
                Some("new")
            ),
            (
                vec![
                    ("src/pasted-1.ts", 2, 6),
                    ("src/pasted-2.ts", 2, 6),
                    ("src/pasted-3.ts", 2, 6)
                ],
                Some("new")
            ),
            (
                vec![("src/removed-1.ts", 2, 6), ("src/removed-2.ts", 2, 6)],
                Some("new")
            ),
            (
                vec![("src/swapped-1.ts", 2, 6), ("src/swapped-3.ts", 2, 6)],
                Some("new")
            ),
        ]
    );
    assert_eq!(
        messages,
        [
            "New or changed since the base: `src/label-2.ts` lines 2–4.",
            "New or changed since the base: `src/moved-3.ts` lines 2–6.",
            "The base reports no group of this run.",
            "New or changed since the base: `src/pasted-3.ts` lines 2–6.",
            "At the base this run had 3 copies.",
            "New or changed since the base: `src/swapped-3.ts` lines 2–6.",
        ]
    );
}

#[test]
fn a_rewrap_leaves_a_group_as_it_was_unless_it_takes_it_over_the_line_minimum() {
    let project = branched(
        "base-duplication-rewrap",
        &[
            ("src/long-1.ts", &counting("g1", 1)),
            ("src/long-2.ts", &counting("g2", 1)),
            ("src/short-1.ts", &labelled("h1", "same")),
            ("src/short-2.ts", &labelled("h2", "same")),
        ],
    );

    // A formatter's pass re-indents one long copy and rewraps both short ones over more lines, and the
    // other long copy is checked out with CRLF line endings.
    project
        .write(
            "src/long-1.ts",
            &counting("g1", 1).replace("\n  ", "\n      "),
        )
        .write("src/long-2.ts", &counting("g2", 1).replace('\n', "\r\n"))
        .write(
            "src/short-1.ts",
            &labelled("h1", "same").replace(
                "console.log(\"same\", ",
                "console.log(\n    \"same\",\n    ",
            ),
        )
        .write(
            "src/short-2.ts",
            &labelled("h2", "same").replace(
                "console.log(\"same\", ",
                "console.log(\n    \"same\",\n    ",
            ),
        )
        .commit();

    let hand_off = declaring(
        &project,
        json!({ "duplicationMinimum": 10, "duplicationMinimumLines": 4 }),
    );
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // The re-indented group has the same run in the same files, its exact forms the same whatever the
    // line endings, so it is unchanged (duplication spec DUP-5); the rewrapped one
    // has its fewest lines over the line minimum only on the branch, so the base reports no such group
    // and it is new (duplication spec FRESH-1).
    assert_eq!(
        duplicates(&run),
        [
            (
                vec![("src/long-1.ts", 2, 6), ("src/long-2.ts", 2, 6)],
                Some("unchanged")
            ),
            (
                vec![("src/short-1.ts", 2, 6), ("src/short-2.ts", 2, 6)],
                Some("new")
            ),
        ]
    );
}

/// The report `audit` prints with a base whose file at the path given does not parse and which the branch
/// mends, beside two copies of a group.
fn skipping(name: &str, path: &str, broken: &str, mended: &str) -> Value {
    let project = branched(
        name,
        &[
            (path, broken),
            ("src/copy-1.ts", &counting("k1", 1)),
            ("src/copy-2.ts", &counting("k2", 1)),
        ],
    );

    project.write(path, mended).commit();

    let hand_off = declaring(&project, json!({ "duplicationMinimum": 10 }));

    audited(&project, &["--hand-off", &hand_off, "--base", "main"])
}

#[test]
fn a_file_skipped_at_the_base_leaves_the_groups_compared_unless_it_is_a_script() {
    let kept = skipping(
        "base-duplication-tsconfig",
        "tsconfig.json",
        "{ broken",
        "{}",
    );

    // A skipped tsconfig changes no run, so the group is compared (duplication spec FRESH-4).
    assert_eq!(
        duplicates(&kept),
        [(
            vec![("src/copy-1.ts", 2, 6), ("src/copy-2.ts", 2, 6)],
            Some("unchanged")
        )]
    );
    assert!(!marked(&kept).contains(&("duplication-not-compared", None, false)));

    // A script skipped at the base is one the branch changed or removed, since one left as it was would
    // stop the working tree's reading.
    let changed = skipping(
        "base-duplication-script",
        "src/broken.ts",
        "const = ;
",
        "export const mended = 1;
",
    );

    // The branch changed the skipped script, where a copy it took out could not be seen at the base, so
    // every group is new, and a notice says why (duplication spec FRESH-4).
    assert_eq!(
        duplicates(&changed),
        [(
            vec![("src/copy-1.ts", 2, 6), ("src/copy-2.ts", 2, 6)],
            Some("new")
        )]
    );
    assert_eq!(
        sentence(&changed, "duplication-not-compared"),
        "The branch changed or removed 1 script(s) skipped at the base, where a copy it took out cannot be \
         seen, so every duplication finding is new: `src/broken.ts`."
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```
cargo test --locked --test audit
```

Expected: FAIL. Every group of the three new tests is new, the re-indented, renamed and untouched ones
included, since Task 1 marks each by a fingerprint the base's never have; `base-files-skipped`'s
sentence does not name duplication; and no `duplication-not-compared` notice is given.

- [ ] **Step 3: Write the code**

```rust edit path=src/forms.rs
<<<<<<< find
//! A statement's form is written as the walk meets its syntax tree: the name of each node, and each
//! value that makes two nodes of one kind differ. Formatting, comments and quotes are not in the tree,
//! and parentheses that change nothing are passed over, so they leave the form as it is. What the lens
//! loosens is written as a placeholder.

use std::collections::BTreeSet;
use std::fmt;
=======
//! A statement's form is written as the walk meets its syntax tree: the name of each node, and each
//! value that makes two nodes of one kind differ. Formatting, comments and quotes are not in the tree,
//! and parentheses that change nothing are passed over, so they leave the form as it is. What the lens
//! loosens is written as a placeholder. Where it loosens anything, the exact form, with nothing
//! loosened, is written beside it (duplication spec FRESH-1).

use std::collections::BTreeSet;
use std::fmt;
>>>>>>> replace
<<<<<<< find
    /// The first bytes of the SHA-256 digest of its form, with what the lens loosens as placeholders
    /// (spec SAME-1, SAME-2).
    pub digest: [u8; DIGEST],
    /// Whether it can join a run: an import cannot, nor a statement that holds a test callee's call
    /// (spec COPY-2, COPY-3).
    pub joins: bool,
=======
    /// The first bytes of the SHA-256 digest of its form, with what the lens loosens as placeholders
    /// (spec SAME-1, SAME-2).
    pub digest: [u8; DIGEST],
    /// The same of its exact form, with nothing loosened; its digest where the lens loosens nothing
    /// (spec FRESH-1).
    pub exact: [u8; DIGEST],
    /// Whether it can join a run: an import cannot, nor a statement that holds a test callee's call
    /// (spec COPY-2, COPY-3).
    pub joins: bool,
>>>>>>> replace
<<<<<<< find
    let mut former = Former {
        lens,
        stream: Vec::new(),
        lists: Vec::new(),
        open: Vec::new(),
        chained: false,
=======
    let mut former = Former {
        lens,
        stream: Vec::new(),
        exact: (lens.names || lens.strings || lens.numbers).then(Vec::new),
        lists: Vec::new(),
        open: Vec::new(),
        chained: false,
>>>>>>> replace
<<<<<<< find
        .map(|raws| List {
            items: raws
                .iter()
                .map(|raw| counted.item(raw, &former.stream, lines))
                .collect(),
        })
        .collect()
=======
        .map(|raws| List {
            items: raws
                .iter()
                .map(|raw| counted.item(raw, &former.stream, former.exact.as_deref(), lines))
                .collect(),
        })
        .collect()
>>>>>>> replace
<<<<<<< find
    lone: BTreeSet<u32>,
}

/// One statement as the walk found it.
struct Raw {
    /// Where its form starts in the stream.
    from: usize,
    /// Where it ends.
    to: usize,
    /// Where the statement is in the text.
    span: Span,
    /// Whether it can join a run.
=======
    lone: BTreeSet<u32>,
}

/// Writes a piece of a form to the form given: its tag, then its bytes after their length.
fn piece(form: &mut Vec<u8>, tag: u8, bytes: &[u8]) {
    form.push(tag);
    form.extend_from_slice(&u32::try_from(bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    form.extend_from_slice(bytes);
}

/// The first bytes of the SHA-256 digest of a form.
fn kept(form: &[u8]) -> [u8; DIGEST] {
    let digest = Sha256::digest(form);
    let mut kept = [0; DIGEST];

    kept.copy_from_slice(&digest[..DIGEST]);
    kept
}

/// One statement as the walk found it.
struct Raw {
    /// Where its form starts in the stream.
    from: usize,
    /// Where it ends.
    to: usize,
    /// Where its exact form starts in the exact stream, where there is one.
    exact_from: usize,
    /// Where it ends.
    exact_to: usize,
    /// Where the statement is in the text.
    span: Span,
    /// Whether it can join a run.
>>>>>>> replace
<<<<<<< find
    lens: &'l Lens,
    /// The forms, end to end.
    stream: Vec<u8>,
    /// Each list's statements, in the order the walk met the lists.
    lists: Vec<Vec<Raw>>,
    /// The statements the walk is inside, by their list and their place in it.
=======
    lens: &'l Lens,
    /// The forms, end to end.
    stream: Vec<u8>,
    /// The exact forms, end to end, where the lens loosens anything.
    exact: Option<Vec<u8>>,
    /// Each list's statements, in the order the walk met the lists.
    lists: Vec<Vec<Raw>>,
    /// The statements the walk is inside, by their list and their place in it.
>>>>>>> replace
<<<<<<< find
    /// Walks one statement of a list, noting where its form starts and ends in the stream.
    fn item(&mut self, list: usize, span: Span, joins: bool, walk: impl FnOnce(&mut Self)) {
        let index = self.lists[list].len();

        self.lists[list].push(Raw {
            from: self.stream.len(),
            to: self.stream.len(),
            span,
            joins,
        });
=======
    /// Walks one statement of a list, noting where its form starts and ends in the stream.
    fn item(&mut self, list: usize, span: Span, joins: bool, walk: impl FnOnce(&mut Self)) {
        let index = self.lists[list].len();
        let exact = self.exact.as_ref().map_or(0, Vec::len);

        self.lists[list].push(Raw {
            from: self.stream.len(),
            to: self.stream.len(),
            exact_from: exact,
            exact_to: exact,
            span,
            joins,
        });
>>>>>>> replace
<<<<<<< find
        walk(self);
        self.open.pop();
        self.lists[list][index].to = self.stream.len();
    }

    /// Writes a piece of a form: its tag, then its bytes after their length.
    fn put(&mut self, tag: u8, bytes: &[u8]) {
        self.stream.push(tag);
        self.stream
            .extend_from_slice(&u32::try_from(bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
        self.stream.extend_from_slice(bytes);
    }

    /// Writes a name, or its placeholder where names are loosened (spec SAME-2).
    fn name(&mut self, name: &str) {
        if self.lens.names {
            self.stream.push(b'n');
        } else {
            self.put(b'N', name.as_bytes());
        }
    }

    /// Writes a string's value, or its placeholder where strings are loosened (spec SAME-2).
    fn string(&mut self, value: &str) {
        if self.lens.strings {
            self.stream.push(b's');
        } else {
            self.put(b'S', value.as_bytes());
        }
    }

    /// Writes a number's value, or its placeholder where numbers are loosened (spec SAME-2).
    fn number(&mut self, value: &[u8]) {
        if self.lens.numbers {
            self.stream.push(b'd');
        } else {
            self.put(b'D', value);
        }
    }

    /// Writes the values of a node that are neither names, strings nor numbers.
=======
        walk(self);
        self.open.pop();
        self.lists[list][index].to = self.stream.len();
        self.lists[list][index].exact_to = self.exact.as_ref().map_or(0, Vec::len);
    }

    /// Writes a piece of both forms: its tag, then its bytes after their length.
    fn put(&mut self, tag: u8, bytes: &[u8]) {
        piece(&mut self.stream, tag, bytes);

        if let Some(exact) = &mut self.exact {
            piece(exact, tag, bytes);
        }
    }

    /// Writes the end of a node to both forms.
    fn close(&mut self) {
        self.stream.push(CLOSE);

        if let Some(exact) = &mut self.exact {
            exact.push(CLOSE);
        }
    }

    /// Writes a value the lens may loosen: where it is loosened, its placeholder to the form and the
    /// value to the exact form; where not, the value to both (spec SAME-2, FRESH-1).
    fn loose(&mut self, loosened: bool, placeholder: u8, tag: u8, bytes: &[u8]) {
        if !loosened {
            self.put(tag, bytes);

            return;
        }

        self.stream.push(placeholder);

        if let Some(exact) = &mut self.exact {
            piece(exact, tag, bytes);
        }
    }

    /// Writes a name, or its placeholder where names are loosened (spec SAME-2).
    fn name(&mut self, name: &str) {
        self.loose(self.lens.names, b'n', b'N', name.as_bytes());
    }

    /// Writes a string's value, or its placeholder where strings are loosened (spec SAME-2).
    fn string(&mut self, value: &str) {
        self.loose(self.lens.strings, b's', b'S', value.as_bytes());
    }

    /// Writes a number's value, or its placeholder where numbers are loosened (spec SAME-2).
    fn number(&mut self, value: &[u8]) {
        self.loose(self.lens.numbers, b'd', b'D', value);
    }

    /// Writes the values of a node that are neither names, strings nor numbers.
>>>>>>> replace
<<<<<<< find
                    self.keys.pop();
                }

                self.stream.push(CLOSE);
            }
        }
    }
=======
                    self.keys.pop();
                }

                self.close();
            }
        }
    }
>>>>>>> replace
<<<<<<< find
        {
            self.put(b'w', b"IdentifierName");
            self.name(&literal.value.display().to_string());
            self.stream.push(CLOSE);
        } else {
            walk::walk_property_key(self, key);
        }
=======
        {
            self.put(b'w', b"IdentifierName");
            self.name(&literal.value.display().to_string());
            self.close();
        } else {
            walk::walk_property_key(self, key);
        }
>>>>>>> replace
<<<<<<< find
        }
    }

    /// One statement, with its digest, its size and its place.
    fn item(&self, raw: &Raw, stream: &[u8], lines: &Lines<'_>) -> Item {
        let first = self.starts.partition_point(|start| *start < raw.span.start);
        let after = self.starts.partition_point(|start| *start < raw.span.end);
        let digest = Sha256::digest(&stream[raw.from..raw.to]);
        let mut kept = [0; DIGEST];

        kept.copy_from_slice(&digest[..DIGEST]);

        Item {
            digest: kept,
            joins: raw.joins,
            tokens: u64::try_from(after - first).unwrap_or(u64::MAX),
            lines: (after > first).then(|| Held {
=======
        }
    }

    /// One statement, with its digests, its size and its place.
    fn item(&self, raw: &Raw, stream: &[u8], exact: Option<&[u8]>, lines: &Lines<'_>) -> Item {
        let first = self.starts.partition_point(|start| *start < raw.span.start);
        let after = self.starts.partition_point(|start| *start < raw.span.end);
        let digest = kept(&stream[raw.from..raw.to]);

        Item {
            digest,
            exact: exact.map_or(digest, |exact| kept(&exact[raw.exact_from..raw.exact_to])),
            joins: raw.joins,
            tokens: u64::try_from(after - first).unwrap_or(u64::MAX),
            lines: (after > first).then(|| Held {
>>>>>>> replace
```

```rust edit path=src/duplication.rs
<<<<<<< find
    pub start: u32,
    /// The byte after its last.
    pub end: u32,
}

/// One group of copies (spec SAME-1 to SAME-4, SIZE-1).
=======
    pub start: u32,
    /// The byte after its last.
    pub end: u32,
    /// Its exact form: the fingerprint its run would have with nothing loosened (spec FRESH-1).
    pub exact: String,
}

/// One group of copies (spec SAME-1 to SAME-4, SIZE-1).
>>>>>>> replace
<<<<<<< find
            return None;
        }

        let parts: Vec<String> = runs
            .first()?
            .1
            .iter()
            .map(|item| findings::hex(&item.digest))
            .collect();
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

        Some(Group {
            fingerprint: findings::fingerprint(Rule::Duplication, &parts),
            tokens,
            lines,
            copies: runs.iter().map(|(path, run)| place(path, run)).collect(),
=======
            return None;
        }

        Some(Group {
            fingerprint: fingerprint(runs.first()?.1, |item| &item.digest),
            tokens,
            lines,
            copies: runs.iter().map(|(path, run)| place(path, run)).collect(),
>>>>>>> replace
<<<<<<< find
    count
}

/// Where a run is.
fn place(path: &str, run: &[Item]) -> Place {
    let first = run.first();
    let last = run.last();
=======
    count
}

/// The fingerprint of a run, made from one digest of each of its statements (spec DUP-4, FRESH-1).
fn fingerprint(run: &[Item], digest: impl Fn(&Item) -> &[u8; DIGEST]) -> String {
    let parts: Vec<String> = run.iter().map(|item| findings::hex(digest(item))).collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

    findings::fingerprint(Rule::Duplication, &parts)
}

/// Where a run is, with its exact form.
fn place(path: &str, run: &[Item]) -> Place {
    let first = run.first();
    let last = run.last();
>>>>>>> replace
<<<<<<< find
        end_line: last.map_or(0, |item| item.end_line),
        start: first.map_or(0, |item| item.start),
        end: last.map_or(0, |item| item.end),
    }
}

=======
        end_line: last.map_or(0, |item| item.end_line),
        start: first.map_or(0, |item| item.start),
        end: last.map_or(0, |item| item.end),
        exact: fingerprint(run, |item| &item.exact),
    }
}

>>>>>>> replace
```

```rust edit path=src/findings.rs
<<<<<<< find

/// The findings of a reading, in the report's order, and the notices that come with them.
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = Vec::new();
    let mut notices = Vec::new();

    unused(reading, &mut found);
    unresolved(reading, &mut found);
    circular(reading, &mut found);
    complex(reading, &mut found);
    duplicated(reading, &mut found, &mut notices);
=======

/// The findings of a reading, in the report's order, and the notices that come with them.
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = counted(reading);
    let mut notices = Vec::new();

    circular(reading, &mut found);
    complex(reading, &mut found);
    duplicated(reading, &mut found, &mut notices);
>>>>>>> replace
<<<<<<< find
    (found, notices)
}

/// Puts findings in the report's order: by rule, then path, line and column, then fingerprint (spec
/// REPORT-1).
pub fn sort(found: &mut [Finding]) {
=======
    (found, notices)
}

/// The findings of the kinds a base marks by counting their fingerprints: the unused files, exports and
/// types, and the unresolved imports (spec INH-1).
pub fn counted(reading: &Reading) -> Vec<Finding> {
    let mut found = Vec::new();

    unused(reading, &mut found);
    unresolved(reading, &mut found);

    found
}

/// Puts findings in the report's order: by rule, then path, line and column, then fingerprint (spec
/// REPORT-1).
pub fn sort(found: &mut [Finding]) {
>>>>>>> replace
```

```rust edit path=src/inherit.rs
<<<<<<< find
//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too, through the files git finds renamed between them (dead-code spec INH-1 to INH-3,
//! COMMIT-4, REPORT-4); a complexity finding by its function's scores there (complexity spec NEW-1 to
//! NEW-5).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use crate::complexity::{self, Scores};
use crate::findings::{self, Baseline, Finding, Graph, Location, Rule};
use crate::git;
use crate::notice::{Notice, SKIPPED};
=======
//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too, through the files git finds renamed between them (dead-code spec INH-1 to INH-3,
//! COMMIT-4, REPORT-4); a complexity finding by its function's scores there (complexity spec NEW-1 to
//! NEW-5); a duplication finding by its group's copies there (duplication spec FRESH-1 to FRESH-4).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use crate::complexity::{self, Scores};
use crate::duplication::{self, Place};
use crate::files::Kind;
use crate::findings::{self, Baseline, Finding, Graph, Location, Rule};
use crate::git;
use crate::notice::{Notice, SKIPPED};
>>>>>>> replace
<<<<<<< find
            .unwrap_or_else(|| path.to_owned())
    };

    // One function's scores depend on no other file, so a file skipped at the base leaves the
    // complexity findings compared; a skipped script has no functions there (complexity spec NEW-4).
    complex(working, found, base, moved);
=======
            .unwrap_or_else(|| path.to_owned())
    };

    // A copy depends on no other file, so a file skipped at the base leaves the duplication findings
    // compared, unless it is a script (duplication spec FRESH-4).
    notices.extend(duplicated(found, base, moved, &skipped));

    // One function's scores depend on no other file, so a file skipped at the base leaves the
    // complexity findings compared; a skipped script has no functions there (complexity spec NEW-4).
    complex(working, found, base, moved);
>>>>>>> replace
<<<<<<< find
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
                 unchanged; complexity findings are still compared, and those in a skipped script are new: \
                 `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
=======
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
                 unchanged; complexity and duplication findings are still compared, and those in a skipped \
                 script are new: `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
>>>>>>> replace
<<<<<<< find

        for finding in found
            .iter_mut()
            .filter(|finding| finding.rule != Rule::Complexity)
        {
            finding.baseline = Some(Baseline::New);
        }

        return notices;
    }

    let (base_found, _) = findings::find(&base.reading);
    // How many findings of each fingerprint the base has, with its paths moved to the branch's.
    let mut held: BTreeMap<String, usize> = BTreeMap::new();

    for finding in base_found
        .iter()
        .filter(|finding| !matches!(finding.rule, Rule::CircularDependency | Rule::Complexity))
    {
        *held.entry(finding.moved(moved)).or_default() += 1;
    }

=======

        for finding in found
            .iter_mut()
            .filter(|finding| !matches!(finding.rule, Rule::Complexity | Rule::Duplication))
        {
            finding.baseline = Some(Baseline::New);
        }

        // A new duplication finding may have moved to another of its copies.
        findings::sort(found);

        return notices;
    }

    // How many findings of each fingerprint the base has, with its paths moved to the branch's.
    let mut held: BTreeMap<String, usize> = BTreeMap::new();

    for finding in findings::counted(&base.reading) {
        *held.entry(finding.moved(moved)).or_default() += 1;
    }

>>>>>>> replace
<<<<<<< find
            continue;
        }

        if finding.rule == Rule::Complexity {
            continue;
        }

=======
            continue;
        }

        if matches!(finding.rule, Rule::Complexity | Rule::Duplication) {
            continue;
        }

>>>>>>> replace
<<<<<<< find
    }
}

/// Marks a circular dependency: inherited when one strongly connected set of the base holds all its
/// files and the branch added no edge between two of them; otherwise new, placed at the first import, by
/// path and line, that makes an edge the branch added (dead-code spec INH-3).
=======
    }
}

/// Marks each duplication finding: unchanged where the base has a group of the same fingerprint whose
/// copies pair one to one with its own, each in the same file, its path moved through the renames, with
/// the same exact form; and otherwise new, with what changed in its message. Where a script was skipped
/// at the base, a copy the branch took out of it cannot be seen there, so every duplication finding is
/// new, and the notice says why. A script skipped at the base is always one the branch changed or
/// removed: one left as it was would stop the working tree's reading (duplication spec FRESH-1 to
/// FRESH-4).
fn duplicated(
    found: &mut [Finding],
    base: &Base,
    moved: impl Fn(&str) -> String,
    skipped: &[&str],
) -> Option<Notice> {
    // Where no duplication is looked for, there is none to compare.
    base.reading.duplication.lens()?;

    let kinds: BTreeMap<&str, Kind> = base
        .reading
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.kind))
        .collect();
    let changed: Vec<&str> = skipped
        .iter()
        .copied()
        .filter(|path| matches!(kinds.get(path), Some(Kind::Script | Kind::Vue)))
        .collect();
    let duplicates = found
        .iter_mut()
        .filter(|finding| finding.rule == Rule::Duplication);

    if !changed.is_empty() {
        for finding in duplicates {
            finding.baseline = Some(Baseline::New);
        }

        return Some(Notice::new(
            "duplication-not-compared",
            None,
            format!(
                "The branch changed or removed {} script(s) skipped at the base, where a copy it took out \
                 cannot be seen, so every duplication finding is new: `{}`.",
                changed.len(),
                changed.join("`, `")
            ),
        ));
    }

    // The base's groups before acceptance, by fingerprint: each one's copies as their files on the
    // branch with their exact forms, sorted (duplication spec FRESH-1).
    let mut held: BTreeMap<String, Vec<Vec<(String, String)>>> = BTreeMap::new();

    for group in duplication::groups(&base.reading).0 {
        let mut copies: Vec<(String, String)> = group
            .copies
            .iter()
            .map(|copy| (moved(&copy.path), copy.exact.clone()))
            .collect();

        copies.sort();
        held.entry(group.fingerprint).or_default().push(copies);
    }

    for finding in duplicates {
        let Some(group) = &finding.group else {
            continue;
        };
        let at_base = held
            .get(&finding.fingerprint)
            .map_or(&[][..], Vec::as_slice);
        let mut copies: Vec<(String, String)> = group
            .copies
            .iter()
            .map(|copy| (copy.path.clone(), copy.exact.clone()))
            .collect();

        copies.sort();

        if at_base.contains(&copies) {
            finding.baseline = Some(Baseline::Unchanged);

            continue;
        }

        let (placed, said) = changes(&group.copies, at_base);
        let mut locations = placed.into_iter().map(Location::from);

        if let Some(location) = locations.next() {
            finding.location = location;
        }

        finding.related = locations.collect();
        finding.message.push_str(&said);
        finding.baseline = Some(Baseline::New);
    }

    None
}

/// Where a new group is placed and what its message adds: its copies, the first whose exact form no copy
/// of the base's groups of its fingerprint has put first; and the copies that pair with none of theirs in
/// the same file with the same exact form, or, where every copy does, how many copies the base's group
/// had, or that the base has no such group (duplication spec FRESH-3).
fn changes<'g>(copies: &'g [Place], at_base: &[Vec<(String, String)>]) -> (Vec<&'g Place>, String) {
    let mut placed: Vec<&Place> = copies.iter().collect();
    let Some(had) = at_base.first() else {
        return (placed, " The base reports no group of this run.".to_owned());
    };
    let exact: BTreeSet<&str> = at_base
        .iter()
        .flatten()
        .map(|(_, exact)| exact.as_str())
        .collect();
    let mut partners: Vec<&(String, String)> = at_base.iter().flatten().collect();
    let fresh: Vec<&Place> = copies
        .iter()
        .filter(|copy| {
            partners
                .iter()
                .position(|(path, exact)| *path == copy.path && *exact == copy.exact)
                .map(|at| partners.swap_remove(at))
                .is_none()
        })
        .collect();

    if let Some(at) = placed
        .iter()
        .position(|copy| !exact.contains(copy.exact.as_str()))
    {
        let first = placed.remove(at);

        placed.insert(0, first);
    }

    let said = if fresh.is_empty() {
        format!(" At the base this run had {} copies.", had.len())
    } else {
        format!(
            " New or changed since the base: {}.",
            duplication::spans(fresh)
        )
    };

    (placed, said)
}

/// Marks a circular dependency: inherited when one strongly connected set of the base holds all its
/// files and the branch added no edge between two of them; otherwise new, placed at the first import, by
/// path and line, that makes an edge the branch added (dead-code spec INH-3).
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```
cargo test --locked --test audit --test duplication --test complexity
```

Expected: PASS, every test of the three targets: the exact form changes no group, fingerprint or
count that `duplication` prints.

- [ ] **Step 5: Format, check and commit**

```
yak stack format
cargo clippy --release --all-targets --locked -- -D warnings
git add src/forms.rs src/duplication.rs src/findings.rs src/inherit.rs tests/audit.test.rs
git commit -m "feat(duplication): mark a group new unless the base has its copies"
```

Expected: the format changes nothing, clippy prints no warning, and the commit is made.

### Task 3: The guide

**Implements:** VERS-1

**Test:** `node --test scripts/contributing.test.mjs`

**Files:**
- Modify: `CONTRIBUTING.md`, `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: Tasks 1 and 2's finding and property bag.
- Produces: the guide's description of `audit` as reporting seven kinds, and its paragraph on what
  duplication adds to interface version 1, the property bag's names among them.

- [ ] **Step 1: Write the failing test**

```js edit path=scripts/contributing.test.mjs
<<<<<<< find
    '`yak-analyzer graph --hand-off <file>`',
    'entry points with what made each one',
    '`yak-analyzer audit` reads the project as `graph` does',
    'six\n  kinds, the five dead-code kinds and complexity',
    '--hand-off <file> --base <commit>`',
    '`yak-analyzer complexity` reads the project as `audit` does',
    '`yak-analyzer complexity --hand-off <file> --all`',
=======
    '`yak-analyzer graph --hand-off <file>`',
    'entry points with what made each one',
    '`yak-analyzer audit` reads the project as `graph` does',
    'seven\n  kinds, the five dead-code kinds, complexity and duplication',
    '--hand-off <file> --base <commit>`',
    '`yak-analyzer complexity` reads the project as `audit` does',
    '`yak-analyzer complexity --hand-off <file> --all`',
>>>>>>> replace
<<<<<<< find
    'Duplication adds to interface version 1 and breaks nothing in it',
    'the rule name `duplication`',
    '`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and',
    'The form of a run that a group\'s fingerprint is made from is part of the fingerprint',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
=======
    'Duplication adds to interface version 1 and breaks nothing in it',
    'the rule name `duplication`',
    '`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and',
    'a `duplication` finding\'s property bag, `duplication`',
    'each copy\'s `path`, `startLine` and `endLine`',
    'the form of a run that a group\'s fingerprint is made from, which is part',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
>>>>>>> replace
```

- [ ] **Step 2: Run the test to see it fail**

```
node --test scripts/contributing.test.mjs
```

Expected: FAIL: the guide still says `audit` reports six kinds, and names no property bag of
duplication.

- [ ] **Step 3: Write the guide**

```markdown edit path=CONTRIBUTING.md
<<<<<<< find
  entry points with what made each one, the files they reach, the uses of each export, and the
  notices. `yak-analyzer graph --hand-off <file>` reads the project the hand-off file names, with
  its declarations. A run that cannot go on prints one line on standard error and exits 1.
- `yak-analyzer audit` reads the project as `graph` does and prints one SARIF 2.1.0 report of six
  kinds, the five dead-code kinds and complexity, with the notices as the run's notifications. `yak-analyzer audit
  --hand-off <file> --base <commit>` reads with the hand-off file's declarations, reads the base
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.
=======
  entry points with what made each one, the files they reach, the uses of each export, and the
  notices. `yak-analyzer graph --hand-off <file>` reads the project the hand-off file names, with
  its declarations. A run that cannot go on prints one line on standard error and exits 1.
- `yak-analyzer audit` reads the project as `graph` does and prints one SARIF 2.1.0 report of seven
  kinds, the five dead-code kinds, complexity and duplication, with the notices as the run's notifications. `yak-analyzer audit
  --hand-off <file> --base <commit>` reads with the hand-off file's declarations, reads the base
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.
>>>>>>> replace
<<<<<<< find
are fixed, and a change that breaks them raises the interface version.

Duplication adds to interface version 1 and breaks nothing in it: the rule name `duplication`; the
unit of `duplicationMinimum`, tokens, which no pack declared before; and the keys
`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and
`duplicationAccepted`.
The form of a run that a group's fingerprint is made from is part of the fingerprint scheme, and a
draft until the swap: from then, a change that breaks it raises the interface version, since it stops
every accepted group and recorded reason from matching.

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
version; its form may change. When it does, the fixtures' written-down output changes in the same
=======
are fixed, and a change that breaks them raises the interface version.

Duplication adds to interface version 1 and breaks nothing in it: the rule name `duplication`; the
unit of `duplicationMinimum`, tokens, which no pack declared before; the keys
`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and
`duplicationAccepted`; and a `duplication` finding's property bag, `duplication`: `tokens`, `lines`
and `copies`, each copy's `path`, `startLine` and `endLine`, in the order of the finding's locations.
The property bag's names, and the form of a run that a group's fingerprint is made from, which is part
of the fingerprint scheme, are a draft until the swap: from then, a change that breaks either raises
the interface version, and a change to the form stops every accepted group and recorded reason from
matching.

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
version; its form may change. When it does, the fixtures' written-down output changes in the same
>>>>>>> replace
```

- [ ] **Step 4: Run the test to see it pass**

```
node --test scripts/contributing.test.mjs
```

Expected: PASS, 4 tests.

- [ ] **Step 5: Run every gate, and commit**

```
yak stack format
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
git add CONTRIBUTING.md scripts/contributing.test.mjs
git commit -m "docs(guide): name the duplication finding and its property bag"
```

Expected: every gate passes, and the commit is made.
