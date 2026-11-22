# Complexity, part 2: the finding in audit, new against inherited: implementation plan

**Goal:** `audit` reports each function over a declared limit as one `complexity` finding, with a
message to act on and the function's scores and limits in its property bag, and with a base marks it
new or inherited by comparing the function's scores there, as design BASE-5 says.

**Architecture:** the reading keeps the run's limits (`Reading::limits`), so `findings::find` can
make complexity findings from the same `complexity::measured` the command uses, and give the
LIMIT-2 notice. A `Finding` carries a complexity finding's `Detail`, which `audit` writes as the
result's property bag. `inherit::mark` compares complexity findings apart from the dead-code kinds:
it reads every function of the base with its scores, pairs them by fingerprint in source order, and
does so whatever files the base skipped.

**Spec:** docs/specs/2026-11-21-complexity-findings-design.md

**Implements:** CPX-1, CPX-2, CPX-3, CPX-5, LIMIT-2, NEW-1 to NEW-5, FN-6, VER-1, VER-2, CASE-1

The second of the item's three plans, on its own branch, `feat/complexity-2-audit`, stacked on plan
1's `feat/complexity-findings` (pull request #39), whose pull request goes against that branch until
#39 merges. Plan 3 is the comparison with fallow (VS-1 to VS-7), after SCORE-4's amendment.

## Global constraints

- One finding per function, under the rule `complexity`, whichever measures are over (CPX-1).
- A complexity finding is `unchanged` only when the base has a function of the same fingerprint, its
  path moved through git's renames, and for each measure the finding is over, the base's score is
  over the same limit and at least as high; otherwise `new`. Both sides use the limits in force now
  (NEW-1; design BASE-3, BASE-5).
- A file skipped at the base leaves complexity findings compared; the dead-code kinds keep COMMIT-4's
  rule that every finding is then new (NEW-4).
- No dead-code finding, fingerprint or message changes, and nothing `graph` prints changes (the
  spec's anti-goals).
- The interface version stays 1 (VER-1).

## The decisions this plan makes

1. **The limits travel with the reading.** `Reading::limits` is read in `reading::read` from the
   hand-off file it is given, so the working tree and the base read the same limits, those in force
   now (design BASE-3), and `findings::find` needs nothing more. `complexity::run` reads them from
   the reading too.
2. **The message.** Each measure over, as "cognitive complexity 23, over the limit of 15", joined by
   ", and " where both are, then the other measure's score in parentheses where only one is (CPX-2).
   A new finding adds, for each measure over, "At the base its cognitive complexity was 18.", with
   ", under the limit" where the base's score was below it, and ", at the limit" where it was equal
   to it, since a score equal to its limit is not over it and "under" would misstate it (NEW-3).
3. **The property bag** is built in `audit.rs` from the finding's `Detail`, its keys sorted as every
   object of the report is. `base` is absent without a base, `null` where the base lacks the
   function, and the base's two scores otherwise (CPX-3).
4. **Pairing in source order.** The base's functions of one fingerprint are queued in path, line and
   column order, and the branch's findings take them in the report's order, which is the same order
   within a file (NEW-5). Only findings take a base function; FN-5 makes a repeated fingerprint a
   fault anyway.
5. **The notice for a file skipped at the base** keeps its kind, `base-files-skipped`, and says that
   complexity findings are still compared, and those in a skipped script new (NEW-4).
6. **The fixtures for the base** go in `tests/audit.test.rs`, beside the dead-code base tests whose
   helpers (`branched`, `audited`, `marked`) they use. A function's scores are made with a body of
   `if`s, `ifs(n)`, which scores `n` cognitive and `n + 1` cyclomatic, against the limits 10 and 3.
7. **The existing tests that list `audit`'s notifications** each gain the LIMIT-2 notice, since
   their projects declare no complexity limit; the one that compares them with `graph`'s notices
   leaves it out, since `graph` reads no limits.

## Review focus

- **A base complexity finding counted as a dead-code one.** `mark` leaves `Rule::Complexity` out of
  the base's fingerprint counts and skips it in the dead-code loop; otherwise a function over at the
  base would be inherited by fingerprint alone, whatever its scores. Pinned by `worse` in
  `a_complexity_finding_is_inherited_only_where_the_base_was_over_with_a_score_as_high`, Task 2.
- **A renamed file's functions.** The base's path is moved through the renames before the
  fingerprint is made. Pinned by `kept` in the same test.
- **The early return for a skipped base file.** Complexity findings are marked before it, so none
  is left without a state. Pinned by `a_file_skipped_at_the_base_leaves_the_other_complexity_findings_compared`,
  Task 2.
- **A measure with no limit.** `inherited` and `at_base` read only the measures a finding is over,
  each of which has a limit; `limit(...).unwrap_or_default()` is never reached for one without. No
  test needed beyond the one-limit case of Task 1's test, where the cyclomatic limit 10 is never
  reached.

### Task 1: The finding and its notice in audit

**Implements:** CPX-1, CPX-2, CPX-3, CPX-5, LIMIT-2

**Test:** `cargo test --locked --test audit --test cli --test complexity`

**Files:**
- Modify: `src/reading.rs`, `src/graph.rs`, `src/complexity.rs`, `src/findings.rs`, `src/audit.rs`,
  `tests/audit.test.rs`, `tests/cli.test.rs`

**Interfaces:**
- Consumes: plan 1's `complexity::{Limits, Measured, measured}` and `findings::fingerprint`.
- Produces: `Reading::limits`; `complexity::{Scores, Detail, parts, message}` and `Limits::limit`;
  `Finding::complexity`; the result's `properties.complexity`. With a base, a complexity finding is
  still marked by fingerprint alone here; Task 2 compares its scores.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/audit.test.rs
<<<<<<< find
    assert_eq!(
        notifications(&run),
        [
            ("no-hand-off-file", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
=======
    assert_eq!(
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("no-hand-off-file", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
>>>>>>> replace
<<<<<<< find
    assert_eq!(
        notifications(&run)
            .iter()
            .map(|(kind, _)| *kind)
            .collect::<Vec<&str>>(),
        notices(&document)
    );
=======
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
>>>>>>> replace
<<<<<<< find
        notifications(&audit(&project, Some(&hand_off))),
        [
            ("glob-matches-nothing", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
=======
        notifications(&audit(&project, Some(&hand_off))),
        [
            ("complexity-not-checked", None),
            ("glob-matches-nothing", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
>>>>>>> replace
<<<<<<< find
    assert_eq!(
        notifications(&run),
        [("kinds-not-reported", None), ("kinds-not-reported", None),]
    );
=======
    assert_eq!(
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("kinds-not-reported", None),
            ("kinds-not-reported", None),
        ]
    );
>>>>>>> replace
<<<<<<< find
    assert_eq!(
        sentences,
        [
            "1 file(s) the analyzer reads are in no package, so their unused files, exports and types are \
             not reported.",
=======
    assert_eq!(
        sentences[1..],
        [
            "1 file(s) the analyzer reads are in no package, so their unused files, exports and types are \
             not reported.",
>>>>>>> replace
<<<<<<< find
        notifications(&run),
        [
            ("no-hand-off-file", None),
            ("computed-import", Some("src/index.ts")),
        ]
    );
    assert_eq!(
        run["invocations"][0]["toolExecutionNotifications"][1]["message"]["text"],
=======
        notifications(&run),
        [
            ("complexity-not-checked", None),
            ("no-hand-off-file", None),
            ("computed-import", Some("src/index.ts")),
        ]
    );
    assert_eq!(
        run["invocations"][0]["toolExecutionNotifications"][2]["message"]["text"],
>>>>>>> replace
<<<<<<< find
        [
            ("base", None, false),
            ("no-hand-off-file", None, false),
            (
                "tsconfig-extends-left-out",
=======
        [
            ("base", None, false),
            ("complexity-not-checked", None, false),
            ("no-hand-off-file", None, false),
            (
                "tsconfig-extends-left-out",
>>>>>>> replace
<<<<<<< find
            ("base", None, false),
            ("base-files-skipped", None, false),
            ("no-hand-off-file", None, false),
=======
            ("base", None, false),
            ("base-files-skipped", None, false),
            ("complexity-not-checked", None, false),
            ("no-hand-off-file", None, false),
>>>>>>> replace
<<<<<<< find
        message(&run, "circular-dependency", "src/q.ts"),
        "These files import each other in a loop: `src/p.ts` → `src/q.ts` → `src/p.ts`. The loop is part \
         of a set of 3 files that each reach all the others. The branch added the import of `./p` on line \
         2 of `src/q.ts`, which leads to `src/p.ts`."
    );
}
=======
        message(&run, "circular-dependency", "src/q.ts"),
        "These files import each other in a loop: `src/p.ts` → `src/q.ts` → `src/p.ts`. The loop is part \
         of a set of 3 files that each reach all the others. The branch added the import of `./p` on line \
         2 of `src/q.ts`, which leads to `src/p.ts`."
    );
}

/// A function's body of as many `if`s as given, which scores that many for cognitive complexity and
/// one more for cyclomatic.
fn ifs(count: usize) -> String {
    "if (a) {} ".repeat(count)
}

/// A hand-off file that gives the cyclomatic limit 10 and the cognitive limit 3, so that a body of
/// `ifs(4)` or more is over only the cognitive one.
fn limited(project: &Project) -> String {
    project.hand_off(
        &project.root,
        &["."],
        stack(
            json!({}),
            json!({ "cyclomaticLimit": 10, "cognitiveLimit": 3 }),
        ),
    )
}

/// The complexity findings of a run, in its order: each with its file as the report writes it, its
/// function's readable name path, and whether it is new or inherited where the run says.
fn complexities(run: &Value) -> Vec<(&str, &str, Option<&str>)> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .filter(|result| result["ruleId"] == json!("complexity"))
        .map(|result| {
            (
                result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .expect("a file"),
                result["properties"]["complexity"]["function"]
                    .as_str()
                    .expect("a name path"),
                result["baselineState"].as_str(),
            )
        })
        .collect()
}

/// The result of the complexity finding of the function given.
fn complexity_of<'r>(run: &'r Value, function: &str) -> &'r Value {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .find(|result| result["properties"]["complexity"]["function"] == json!(function))
        .unwrap_or_else(|| panic!("a finding of {function}"))
}

#[test]
fn reports_a_function_over_a_limit_once_with_its_scores_and_limits() {
    let project = Project::new("audit-complexity");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            &format!(
                "export function under(a) {{ {} }}\nexport function over(a) {{ {} }}\nexport function both(a) {{ {} }}\n",
                ifs(3),
                ifs(4),
                ifs(10)
            ),
        )
        .commit();

    let hand_off = limited(&project);
    let run = audit(&project, Some(&hand_off));
    let over = complexity_of(&run, "over");

    // One finding per function, whichever measures are over, at the function's name (complexity spec
    // CPX-1, CPX-2).
    assert_eq!(
        complexities(&run),
        [
            ("src/index.ts", "over", None),
            ("src/index.ts", "both", None),
        ]
    );
    assert_eq!(
        over["locations"][0]["physicalLocation"]["region"],
        json!({ "startLine": 2, "startColumn": 17 })
    );
    assert_eq!(
        over["message"]["text"],
        json!("`over` has cognitive complexity 4, over the limit of 3 (cyclomatic 5).")
    );
    assert_eq!(
        complexity_of(&run, "both")["message"]["text"],
        json!(
            "`both` has cyclomatic complexity 11, over the limit of 10, and cognitive complexity 10, over \
             the limit of 3."
        )
    );
    // Without a base, the property bag holds no `base` (complexity spec CPX-3).
    assert_eq!(
        over["properties"]["complexity"],
        json!({
            "function": "over",
            "cyclomatic": 5,
            "cognitive": 4,
            "cyclomaticLimit": 10,
            "cognitiveLimit": 3,
            "over": ["cognitive"],
        })
    );
    assert!(!notifications(&run).contains(&("complexity-not-checked", None)));

    // With no limit declared, no complexity finding is reported, and a notice says so (complexity spec
    // LIMIT-2).
    let run = audit(&project, None);

    assert_eq!(complexities(&run), []);
    assert!(notifications(&run).contains(&("complexity-not-checked", None)));
}
>>>>>>> replace
```

```rust edit path=tests/cli.test.rs
<<<<<<< find
    assert_eq!(notifications.as_array().map(Vec::len), Some(1));
    assert_eq!(
        notifications[0]["descriptor"]["id"],
        json!("no-hand-off-file")
    );
=======
    assert_eq!(notifications.as_array().map(Vec::len), Some(2));
    assert_eq!(
        notifications[1]["descriptor"]["id"],
        json!("no-hand-off-file")
    );
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test audit --test cli --test complexity
```

Expected: the audit and cli tests that list notifications fail without `complexity-not-checked`,
and the new test finds no complexity result.

- [ ] **Step 3: Write the code**

```rust edit path=src/reading.rs
<<<<<<< find
use crate::declarations;
=======
use crate::complexity::Limits;
use crate::declarations;
>>>>>>> replace
<<<<<<< find
    /// What a person should know about the run, in the order it was found.
    pub notices: Vec<Notice>,
}

/// What every reading
=======
    /// What a person should know about the run, in the order it was found.
    pub notices: Vec<Notice>,
    /// The limit of each measure of complexity the declarations give, one for the whole run
    /// (complexity spec LIMIT-1).
    pub limits: Limits,
}

/// What every reading
>>>>>>> replace
<<<<<<< find
        entries,
        uses,
        notices,
    })
}
=======
        entries,
        uses,
        notices,
        limits: Limits::of(handed),
    })
}
>>>>>>> replace
```

```rust edit path=src/graph.rs
<<<<<<< find
        uses,
        notices,
    } = reading;
=======
        uses,
        notices,
        limits: _,
    } = reading;
>>>>>>> replace
```

```rust edit path=src/complexity.rs
<<<<<<< find
//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint of its
//! finding, and the document the command prints (complexity spec LIMIT-1, LIMIT-2, CPX-4, SOLO-1 to
//! SOLO-5).

use std::path::Path;
=======
//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint and message
//! of its finding, and the document the command prints (complexity spec LIMIT-1, LIMIT-2, CPX-2 to
//! CPX-4, SOLO-1 to SOLO-5).

use std::fmt::Write as _;
use std::path::Path;
>>>>>>> replace
<<<<<<< find
use crate::functions::{self, Function};
=======
use crate::functions::{self, Function, Scored};
>>>>>>> replace
<<<<<<< find
/// The name of the cognitive measure in what is printed (spec CPX-3).
const COGNITIVE: &str = "cognitive";
=======
/// The name of the cognitive measure in what is printed (spec CPX-3).
const COGNITIVE: &str = "cognitive";

/// The measures, in the order a finding names them.
const MEASURES: [&str; 2] = [CYCLOMATIC, COGNITIVE];
>>>>>>> replace
<<<<<<< find
    /// The measures a function is over, cyclomatic first: a score greater than its limit, and not one
    /// equal to it (spec CPX-3; the spec's terms).
=======
    /// The limit of the measure named, where one is declared.
    pub fn limit(self, measure: &str) -> Option<u64> {
        if measure == CYCLOMATIC {
            self.cyclomatic
        } else {
            self.cognitive
        }
    }

    /// The measures a function is over, cyclomatic first: a score greater than its limit, and not one
    /// equal to it (spec CPX-3; the spec's terms).
>>>>>>> replace
<<<<<<< find
/// The value of a limit's key that wins:
=======
/// A function's two scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scores {
    /// Its cyclomatic complexity.
    pub cyclomatic: u64,
    /// Its cognitive complexity.
    pub cognitive: u64,
}

impl Scores {
    /// The score of the measure named.
    pub fn score(self, measure: &str) -> u64 {
        if measure == CYCLOMATIC {
            self.cyclomatic
        } else {
            self.cognitive
        }
    }
}

impl From<Scored> for Scores {
    fn from(scored: Scored) -> Self {
        Self {
            cyclomatic: scored.cyclomatic,
            cognitive: scored.cognitive,
        }
    }
}

/// What a complexity finding carries in its property bag (spec CPX-3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detail {
    /// The function's readable name path.
    pub function: String,
    /// Its scores.
    pub scores: Scores,
    /// The limits in force.
    pub limits: Limits,
    /// The measures it is over, cyclomatic first.
    pub over: Vec<&'static str>,
    /// Its scores at the base: none without a base, and `Some(None)` where the base does not have
    /// the function.
    pub base: Option<Option<Scores>>,
}

/// The value of a limit's key that wins:
>>>>>>> replace
<<<<<<< find
/// The fingerprint of a function's finding: its rule, its file's path, and for each part of its name
/// path `n` for a named part or `u` for an unnamed one, its name, and its number in decimal digits
/// (spec CPX-4; design IF-4).
pub fn fingerprint(path: &str, function: &Function) -> String {
    let mut parts = vec![path.to_owned()];

    for part in &function.path {
        parts.push(if part.name.is_some() { "n" } else { "u" }.to_owned());
        parts.push(part.name.clone().unwrap_or_default());
        parts.push(part.number.to_string());
    }

    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

    findings::fingerprint(Rule::Complexity, &parts)
}
=======
/// What a function's fingerprint is made from after its rule: its file's path, and for each part of
/// its name path `n` for a named part or `u` for an unnamed one, its name, and its number in decimal
/// digits (spec CPX-4; design IF-4).
pub fn parts(path: &str, function: &Function) -> Vec<String> {
    let mut parts = vec![path.to_owned()];

    for part in &function.path {
        parts.push(if part.name.is_some() { "n" } else { "u" }.to_owned());
        parts.push(part.name.clone().unwrap_or_default());
        parts.push(part.number.to_string());
    }

    parts
}

/// The fingerprint of a function's finding (spec CPX-4).
pub fn fingerprint(path: &str, function: &Function) -> String {
    let parts = parts(path, function);
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

    findings::fingerprint(Rule::Complexity, &parts)
}

/// The message of a function's finding: its readable name path, each measure it is over with its
/// score and limit, then the other measure's score, as in "`Parser > read` has cognitive complexity
/// 23, over the limit of 15 (cyclomatic 12)." (spec CPX-2).
pub fn message(measured: &Measured, limits: Limits) -> String {
    let scores = Scores::from(measured.function.scored);
    let over: Vec<String> = measured
        .over
        .iter()
        .map(|measure| {
            format!(
                "{measure} complexity {}, over the limit of {}",
                scores.score(measure),
                limits.limit(measure).unwrap_or_default()
            )
        })
        .collect();
    let others: Vec<String> = MEASURES
        .iter()
        .filter(|measure| !measured.over.contains(measure))
        .map(|measure| format!("{measure} {}", scores.score(measure)))
        .collect();
    let mut message = format!(
        "`{}` has {}",
        measured.function.readable(),
        over.join(", and ")
    );

    if !others.is_empty() {
        let _ = write!(message, " ({})", others.join(", "));
    }

    message.push('.');
    message
}
>>>>>>> replace
<<<<<<< find
    let listing = files::list(&root, &mut notices)?;
    let limits = Limits::of(handed.as_ref());
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
=======
    let listing = files::list(&root, &mut notices)?;
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
    let limits = reading.limits;
>>>>>>> replace
```

```rust edit path=src/findings.rs
<<<<<<< find
use sha2::{Digest, Sha256};

use crate::entries::Fault;
=======
use sha2::{Digest, Sha256};

use crate::complexity::{self, Detail, Scores};
use crate::entries::Fault;
>>>>>>> replace
<<<<<<< find
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
}
=======
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
    /// What a complexity finding carries in its property bag; none for any other (complexity spec
    /// CPX-3).
    pub complexity: Option<Detail>,
}
>>>>>>> replace
<<<<<<< find
            parts: parts.iter().map(|part| (*part).to_owned()).collect(),
            baseline: None,
        }
=======
            parts: parts.iter().map(|part| (*part).to_owned()).collect(),
            baseline: None,
            complexity: None,
        }
>>>>>>> replace
<<<<<<< find
    circular(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
=======
    circular(reading, &mut found);
    complex(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
    notices.extend(reading.limits.notice());
>>>>>>> replace
<<<<<<< find
/// One notice for each file with an `import()` or `require()` it cannot follow:
=======
/// One finding for each function over a limit, whichever measures are over (complexity spec CPX-1 to
/// CPX-5).
fn complex(reading: &Reading, found: &mut Vec<Finding>) {
    let limits = reading.limits;

    for measured in complexity::measured(reading, limits) {
        if measured.over.is_empty() {
            continue;
        }

        let parts = complexity::parts(&measured.path, &measured.function);
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
        let scored = measured.function.scored;
        let mut finding = Finding::new(
            Rule::Complexity,
            complexity::message(&measured, limits),
            Location {
                path: measured.path.clone(),
                line: Some(scored.line),
                column: Some(scored.column),
            },
            &parts,
        );

        finding.complexity = Some(Detail {
            function: measured.function.readable(),
            scores: Scores::from(scored),
            limits,
            over: measured.over,
            base: None,
        });
        found.push(finding);
    }
}

/// One notice for each file with an `import()` or `require()` it cannot follow:
>>>>>>> replace
```

```rust edit path=src/audit.rs
<<<<<<< find
use crate::files;
use crate::findings::{self, Finding, Location, Rule};
=======
use crate::complexity::Detail;
use crate::files;
use crate::findings::{self, Finding, Location, Rule};
>>>>>>> replace
<<<<<<< find
    if let Some(baseline) = finding.baseline {
        result["baselineState"] = json!(baseline.name());
    }
=======
    if let Some(baseline) = finding.baseline {
        result["baselineState"] = json!(baseline.name());
    }

    if let Some(detail) = &finding.complexity {
        result["properties"] = json!({ "complexity": bag(detail) });
    }
>>>>>>> replace
<<<<<<< find
/// A location as SARIF writes one:
=======
/// The property bag of a complexity finding: the function, its scores, the limits, the measures it is
/// over, and with a base its scores there, or `null` where the base does not have it (complexity spec
/// CPX-3).
fn bag(detail: &Detail) -> Value {
    let mut bag = json!({
        "cognitive": detail.scores.cognitive,
        "cognitiveLimit": detail.limits.cognitive,
        "cyclomatic": detail.scores.cyclomatic,
        "cyclomaticLimit": detail.limits.cyclomatic,
        "function": detail.function,
        "over": detail.over,
    });

    if let Some(base) = detail.base {
        bag["base"] = base.map_or(
            Value::Null,
            |base| json!({ "cognitive": base.cognitive, "cyclomatic": base.cyclomatic }),
        );
    }

    bag
}

/// A location as SARIF writes one:
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --locked --test audit --test cli --test complexity
cargo clippy --locked --all-targets
```

Expected: every test passes; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
git add -A && git commit -m "feat(complexity): report functions over a limit in audit"
```

Expected: one commit.

### Task 2: New against inherited

**Implements:** NEW-1 to NEW-5, FN-6, CASE-1

**Test:** `cargo test --locked --test audit`

**Files:**
- Modify: `src/complexity.rs`, `src/inherit.rs`, `tests/audit.test.rs`

**Interfaces:**
- Consumes: Task 1's `Detail`, `Scores`, `Limits::limit` and `complexity::measured`.
- Produces: `complexity::{inherited, at_base}`; `inherit::complex`, which marks every complexity
  finding and fills its `Detail::base`.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/audit.test.rs
<<<<<<< find
//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4).
=======
//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3);
//! and, with a base as a commit and a branch as later commits and edits, which findings are new and
//! which inherited (dead-code spec CMD-2, COMMIT-1 to COMMIT-5, INH-1, FIX-4), complexity findings by
//! their scores (complexity spec CPX-1 to CPX-3, NEW-1 to NEW-5, FN-6, CASE-1).
>>>>>>> replace
<<<<<<< find
        "2 file(s) were skipped at the base, so no finding is marked unchanged: `tsconfig.json`, \
         `src/broken.ts`."
=======
        "2 file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
         unchanged; complexity findings are still compared, and those in a skipped script are new: \
         `tsconfig.json`, `src/broken.ts`."
>>>>>>> replace
<<<<<<< find
    assert!(notifications(&run).contains(&("complexity-not-checked", None)));
}
=======
    assert!(notifications(&run).contains(&("complexity-not-checked", None)));
}

#[test]
fn a_complexity_finding_is_inherited_only_where_the_base_was_over_with_a_score_as_high() {
    let body = |count| format!("{{ {} }}", ifs(count));
    let project = branched(
        "base-complexity",
        &[
            (
                "package.json",
                r#"{ "name": "app", "main": "./src/index.ts" }"#,
            ),
            (
                "src/index.ts",
                &format!(
                    "import './moved';\nexport function under(a) {}\nexport function worse(a) {}\nexport function same(a) {}\nexport function better(a) {}\n",
                    body(2),
                    body(4),
                    body(4),
                    body(6)
                ),
            ),
            (
                "src/moved.ts",
                &format!("export function kept(a) {}\n", body(4)),
            ),
        ],
    );

    project.git(&["mv", "src/moved.ts", "src/elsewhere.ts"]);
    project
        .write(
            "src/index.ts",
            &format!(
                "import './elsewhere';\nexport function under(a) {}\nexport function worse(a) {}\nexport function same(a) {}\nexport function better(a) {}\nexport function added(a) {}\n",
                body(4),
                body(5),
                body(4),
                body(5),
                body(4)
            ),
        )
        .commit();

    let hand_off = limited(&project);
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // Under at the base and over now, worse, and added are new; the same, better, and a function in a
    // renamed file are inherited (complexity spec NEW-1; design BASE-5).
    assert_eq!(
        complexities(&run),
        [
            ("src/elsewhere.ts", "kept", Some("unchanged")),
            ("src/index.ts", "under", Some("new")),
            ("src/index.ts", "worse", Some("new")),
            ("src/index.ts", "same", Some("unchanged")),
            ("src/index.ts", "better", Some("unchanged")),
            ("src/index.ts", "added", Some("new")),
        ]
    );
    // A new finding's message says what the base had, and the property bag holds the base's scores,
    // or `null` where the base does not have the function (complexity spec NEW-3, CPX-3).
    assert_eq!(
        complexity_of(&run, "under")["message"]["text"],
        json!(
            "`under` has cognitive complexity 4, over the limit of 3 (cyclomatic 5). At the base its \
             cognitive complexity was 2, under the limit."
        )
    );
    assert_eq!(
        complexity_of(&run, "worse")["message"]["text"],
        json!(
            "`worse` has cognitive complexity 5, over the limit of 3 (cyclomatic 6). At the base its \
             cognitive complexity was 4."
        )
    );
    assert_eq!(
        complexity_of(&run, "added")["message"]["text"],
        json!("`added` has cognitive complexity 4, over the limit of 3 (cyclomatic 5).")
    );
    assert_eq!(
        complexity_of(&run, "worse")["properties"]["complexity"]["base"],
        json!({ "cyclomatic": 5, "cognitive": 4 })
    );
    assert_eq!(
        complexity_of(&run, "added")["properties"]["complexity"]["base"],
        json!(null)
    );
    assert_eq!(
        complexity_of(&run, "same")["message"]["text"],
        json!("`same` has cognitive complexity 4, over the limit of 3 (cyclomatic 5).")
    );
}

#[test]
fn a_file_skipped_at_the_base_leaves_the_other_complexity_findings_compared() {
    let over = format!("{{ {} }}", ifs(4));
    let project = branched(
        "base-complexity-skipped",
        &[
            ("package.json", "{ broken"),
            ("pnpm-workspace.yaml", "packages: ["),
            ("tsconfig.json", "{ broken"),
            (
                "src/index.ts",
                &format!("export function steady(a) {over}\n"),
            ),
            ("src/broken.ts", "const = ;\n"),
        ],
    );

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("pnpm-workspace.yaml", "packages: []\n")
        .write("tsconfig.json", "{}")
        .write(
            "src/broken.ts",
            &format!("export function mended(a) {over}\n"),
        )
        .commit();

    let hand_off = limited(&project);
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);
    let skipped: Vec<Option<&str>> = marked(&run)
        .into_iter()
        .filter(|(kind, _, _)| *kind == "file-skipped")
        .map(|(_, file, _)| file)
        .collect();

    // The skipped `package.json`, workspace list and tsconfig change no function's scores, and the
    // skipped script had no functions there (complexity spec NEW-4).
    assert_eq!(
        skipped,
        [
            Some("package.json"),
            Some("pnpm-workspace.yaml"),
            Some("src/broken.ts"),
            Some("tsconfig.json"),
        ]
    );
    assert_eq!(
        complexities(&run),
        [
            ("src/broken.ts", "mended", Some("new")),
            ("src/index.ts", "steady", Some("unchanged")),
        ]
    );
}

#[test]
fn a_function_added_or_removed_before_another_can_take_its_place() {
    let body = |count| format!("{{ {} }}", ifs(count));
    let project = branched(
        "base-complexity-shifts",
        &[
            (
                "src/unnamed.ts",
                &format!("export const handlers = [(a) => {}];\n", body(4)),
            ),
            (
                "src/repeated.js",
                &format!("export const named = {{ go(a) {} }};\n", body(4)),
            ),
            (
                "src/renamed.ts",
                &format!("export function before(a) {}\n", body(4)),
            ),
            (
                "src/gained.ts",
                &format!(
                    "export const rows = [{{ size: 1 }}, {{ run(a) {} }}];\n",
                    body(4)
                ),
            ),
            (
                "src/removed.ts",
                &format!(
                    "export const removed = [(a) => {}, (a) => {}];\n",
                    body(6),
                    body(4)
                ),
            ),
        ],
    );

    project
        .write(
            "src/unnamed.ts",
            &format!(
                "export const handlers = [(a) => {}, (a) => {}];\n",
                body(4),
                body(4)
            ),
        )
        .write(
            "src/repeated.js",
            &format!(
                "export const named = {{ go(a) {}, go(a) {} }};\n",
                body(4),
                body(4)
            ),
        )
        .write(
            "src/renamed.ts",
            &format!("export function after(a) {}\n", body(4)),
        )
        .write(
            "src/gained.ts",
            &format!(
                "export const rows = [{{ size: 1, go() {{}} }}, {{ run(a) {} }}];\n",
                body(4)
            ),
        )
        .write(
            "src/removed.ts",
            &format!("export const removed = [(a) => {}];\n", body(5)),
        )
        .commit();

    let hand_off = limited(&project);
    let run = audited(&project, &["--hand-off", &hand_off, "--base", "main"]);

    // The accepted consequences of naming a function by its place among the unnamed ones or the ones
    // of its name: the function added first takes the place, and the base's scores, of the one that
    // was there; a renamed function is a new one; and after an earlier one is removed, a later one
    // made worse is compared with the removed one's scores (complexity spec FN-6).
    assert_eq!(
        complexities(&run),
        [
            ("src/gained.ts", "<anonymous 2> > run", Some("new")),
            ("src/removed.ts", "<anonymous 1>", Some("unchanged")),
            ("src/renamed.ts", "after", Some("new")),
            ("src/repeated.js", "named > go", Some("unchanged")),
            ("src/repeated.js", "named > go #2", Some("new")),
            ("src/unnamed.ts", "<anonymous 1>", Some("unchanged")),
            ("src/unnamed.ts", "<anonymous 2>", Some("new")),
        ]
    );

    let removed = run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .find(|result| {
            result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                == json!("src/removed.ts")
        })
        .expect("the finding in src/removed.ts");

    // The one left is compared with the removed one's scores, which were higher.
    assert_eq!(
        removed["properties"]["complexity"]["base"],
        json!({ "cyclomatic": 7, "cognitive": 6 })
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```text
cargo test --locked --test audit
```

Expected: the three new tests fail, complexity findings marked by fingerprint alone and the bag
holding no `base`, and the skipped-file test on the notice's old sentence.

- [ ] **Step 3: Write the code**

```rust edit path=src/complexity.rs
<<<<<<< find
//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint and message
//! of its finding, and the document the command prints (complexity spec LIMIT-1, LIMIT-2, CPX-2 to
//! CPX-4, SOLO-1 to SOLO-5).
=======
//! The `complexity` command, and the complexity of a reading's functions, which `audit` reports too:
//! the limit of each measure, each function's scores against them with the fingerprint and message
//! of its finding and how it compares with the base, and the document the command prints
//! (complexity spec LIMIT-1, LIMIT-2, CPX-2 to CPX-4, NEW-1, NEW-3, SOLO-1 to SOLO-5).
>>>>>>> replace
<<<<<<< find
use std::fmt::Write as _;
=======
use std::cmp::Ordering;
use std::fmt::Write as _;
>>>>>>> replace
<<<<<<< find
    message.push('.');
    message
}
=======
    message.push('.');
    message
}

/// Whether a finding is inherited from the base's scores of its function: for each measure it is
/// over, the base's score is over the same limit and at least as high (spec NEW-1; design BASE-5).
pub fn inherited(over: &[&str], now: Scores, base: Scores, limits: Limits) -> bool {
    over.iter().all(|measure| {
        limits
            .limit(measure)
            .is_some_and(|limit| base.score(measure) > limit)
            && base.score(measure) >= now.score(measure)
    })
}

/// What a new finding's message adds of the base: for each measure it is over, the base's score, and
/// where that was not over the limit, so (spec NEW-3).
pub fn at_base(over: &[&str], base: Scores, limits: Limits) -> String {
    over.iter()
        .map(|measure| {
            let was = base.score(measure);
            let against = match was.cmp(&limits.limit(measure).unwrap_or_default()) {
                Ordering::Greater => "",
                Ordering::Equal => ", at the limit",
                Ordering::Less => ", under the limit",
            };

            format!(" At the base its {measure} complexity was {was}{against}.")
        })
        .collect()
}
>>>>>>> replace
```

```rust edit path=src/inherit.rs
<<<<<<< find
//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too, through the files git finds renamed between them (dead-code spec INH-1 to INH-3,
//! COMMIT-4, REPORT-4).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::findings::{self, Baseline, Finding, Graph, Location, Rule};
=======
//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too, through the files git finds renamed between them (dead-code spec INH-1 to INH-3,
//! COMMIT-4, REPORT-4); a complexity finding by its function's scores there (complexity spec NEW-1 to
//! NEW-5).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use crate::complexity::{self, Scores};
use crate::findings::{self, Baseline, Finding, Graph, Location, Rule};
>>>>>>> replace
<<<<<<< find
    // A file skipped at the base can leave it with findings it never had, which would make the same
    // findings on the branch look inherited: so then every finding is new (dead-code spec COMMIT-4).
    if !skipped.is_empty() {
        notices.push(Notice::new(
            "base-files-skipped",
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding is marked unchanged: `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
        ));

        for finding in found.iter_mut() {
            finding.baseline = Some(Baseline::New);
        }

        return notices;
    }

    let moved = |path: &str| {
        base.renames
            .get(path)
            .cloned()
            .unwrap_or_else(|| path.to_owned())
    };
    let (base_found, _) = findings::find(&base.reading);
    // How many findings of each fingerprint the base has, with its paths moved to the branch's.
    let mut held: BTreeMap<String, usize> = BTreeMap::new();

    for finding in base_found
        .iter()
        .filter(|finding| finding.rule != Rule::CircularDependency)
    {
=======
    let moved = |path: &str| {
        base.renames
            .get(path)
            .cloned()
            .unwrap_or_else(|| path.to_owned())
    };

    // One function's scores depend on no other file, so a file skipped at the base leaves the
    // complexity findings compared; a skipped script has no functions there (complexity spec NEW-4).
    complex(working, found, base, moved);

    // A file skipped at the base can leave it with findings it never had, which would make the same
    // findings on the branch look inherited: so then every other finding is new (dead-code spec
    // COMMIT-4).
    if !skipped.is_empty() {
        notices.push(Notice::new(
            "base-files-skipped",
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
                 unchanged; complexity findings are still compared, and those in a skipped script are new: \
                 `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
        ));

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
>>>>>>> replace
<<<<<<< find
        if finding.rule == Rule::CircularDependency {
            cycle(working, &working_graph, finding, &base_pairs, &base_sets);

            continue;
        }
=======
        if finding.rule == Rule::CircularDependency {
            cycle(working, &working_graph, finding, &base_pairs, &base_sets);

            continue;
        }

        if finding.rule == Rule::Complexity {
            continue;
        }
>>>>>>> replace
<<<<<<< find
/// Marks a circular dependency:
=======
/// Marks each complexity finding: unchanged where the base has its function, the path moved through
/// the renames, over each limit the finding is over with a score at least as high, and otherwise new,
/// with what the base had in its message. Every function of the base is read, and functions of one
/// fingerprint pair in source order (complexity spec NEW-1 to NEW-3, NEW-5).
fn complex(working: &Reading, found: &mut [Finding], base: &Base, moved: impl Fn(&str) -> String) {
    let limits = working.limits;
    let mut held: BTreeMap<String, VecDeque<Scores>> = BTreeMap::new();

    for measured in complexity::measured(&base.reading, limits) {
        held.entry(complexity::fingerprint(
            &moved(&measured.path),
            &measured.function,
        ))
        .or_default()
        .push_back(Scores::from(measured.function.scored));
    }

    for finding in found
        .iter_mut()
        .filter(|finding| finding.rule == Rule::Complexity)
    {
        let Some(detail) = &mut finding.complexity else {
            continue;
        };
        let at_base = held
            .get_mut(&finding.fingerprint)
            .and_then(VecDeque::pop_front);
        let inherited = at_base.is_some_and(|scores| {
            complexity::inherited(&detail.over, detail.scores, scores, limits)
        });

        if let Some(scores) = at_base
            && !inherited
        {
            finding
                .message
                .push_str(&complexity::at_base(&detail.over, scores, limits));
        }

        detail.base = Some(at_base);
        finding.baseline = Some(if inherited {
            Baseline::Unchanged
        } else {
            Baseline::New
        });
    }
}

/// Marks a circular dependency:
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```text
cargo test --locked --no-fail-fast
cargo clippy --locked --all-targets
```

Expected: every test of every file passes; clippy prints no warning.

- [ ] **Step 5: Commit**

```text
yak stack format
git add -A && git commit -m "feat(complexity): mark a finding inherited by its scores at the base"
```

Expected: one commit.

### Task 3: The guide

**Implements:** VER-1, VER-2

**Test:** `node --test scripts/contributing.test.mjs`

**Files:**
- Modify: `CONTRIBUTING.md`, `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: Task 1's property bag.
- Produces: the guide's `audit` reporting six kinds, and the property bag's names among what
  interface version 1 covers, a draft until the swap.

- [ ] **Step 1: Write the failing test**

```js edit path=scripts/contributing.test.mjs
<<<<<<< find
    '`yak-analyzer audit` reads the project as `graph` does',
=======
    '`yak-analyzer audit` reads the project as `graph` does',
    'six\n  kinds, the five dead-code kinds and complexity',
>>>>>>> replace
<<<<<<< find
    'a draft until the swap',
=======
    'a draft until the swap',
    'its property bag, `complexity`',
    '`cyclomaticLimit`, `cognitiveLimit` and `over`',
>>>>>>> replace
```

- [ ] **Step 2: Run the test to see it fail**

```text
node --test scripts/contributing.test.mjs
```

Expected: the commands test and the interface-version test fail on sentences the guide does not hold
yet.

- [ ] **Step 3: Write the guide**

````markdown edit path=CONTRIBUTING.md
<<<<<<< find
- `yak-analyzer audit` reads the project as `graph` does and prints one SARIF 2.1.0 report of the
  five dead-code kinds, with the notices as the run's notifications.
=======
- `yak-analyzer audit` reads the project as `graph` does and prints one SARIF 2.1.0 report of six
  kinds, the five dead-code kinds and complexity, with the notices as the run's notifications.
>>>>>>> replace
<<<<<<< find
The rules that name a function in a `complexity` finding's fingerprint, its name path, are part of
the scheme and a draft until the swap, the pack release that replaces fallow with the analyzer in
yak: nothing yak runs reads them before then, so changing them raises nothing. From the swap they
are fixed, and a change that breaks them raises the interface version.
=======
A `complexity` finding carries its function's scores in its property bag, `complexity`:
`function`, `cyclomatic`, `cognitive`, `cyclomaticLimit`, `cognitiveLimit` and `over`, and with a
base `base`. Interface version 1 covers those names and meanings, since the engine will read them,
and the rules that name a function in the finding's fingerprint, its name path, as part of the
scheme. Both are a draft until the swap, the pack release that replaces fallow with the analyzer in
yak: nothing yak runs reads them before then, so changing them raises nothing. From the swap they
are fixed, and a change that breaks them raises the interface version.
>>>>>>> replace
````

- [ ] **Step 4: Run the test to see it pass**

```text
node --test scripts/contributing.test.mjs
```

Expected: every test passes.

- [ ] **Step 5: Commit**

```text
yak stack format
git add -A && git commit -m "docs: describe the complexity finding's property bag"
```

Expected: one commit.
