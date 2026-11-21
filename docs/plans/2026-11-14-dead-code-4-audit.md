# Dead-code findings, part 4: the five kinds, the SARIF report and the audit command: implementation plan

**Goal:** a new command, `audit`, reads the project as `graph` does and prints one SARIF 2.1.0 report
of the five dead-code kinds: unused files, exports and types, unresolved imports, and circular
dependencies, each finding with a message to act on, a location and a fingerprint.

**Architecture:** The reading `graph` does moves into its own module, `src/reading.rs`, so `graph`
and `audit` share one reading and differ only in what they print. A new `src/findings.rs` works out
the findings from a reading: the uses and the reach that plan 3 built give the three kinds that
depend on entry points, the resolved imports give the unresolved ones, and the imports that load at
run time give the cycles. A new `src/audit.rs` writes them as SARIF with `serde_json`.

**Spec:** docs/specs/2026-11-13-dead-code-findings-design.md

**Implements:** CMD-1, CMD-3, CMD-5, ENTRY-8, ENTRY-9, ENTRY-10, KIND-1 to KIND-5, REPORT-1 to REPORT-4, FIX-1, FIX-2, FIX-3, DEP-1

The fourth of the item's six plans, listed in plan 1. Plan 3 has merged, so this plan's branch,
`feat/dead-code-4-audit`, is made from `main`, and its pull request goes against `main`. Plan 3 moved
KIND-1, KIND-2, ENTRY-9 and ENTRY-10 here, to come with the command that prints them.

What waits for plan 5: `--base`, so CMD-2, the base's half of CMD-1 and CMD-3 (the stops 7b and 7c),
REPORT-1's `baselineState` and REPORT-4's notifications of the base. Plan 5 adds them to what this
plan builds, and nothing here takes a form that plan 5 would have to change.

## Global constraints

- `audit` exits 0 when it ran, whether or not it found anything; a run that cannot go on stops as
  `graph` does, exit 1 and one line on standard error; a wrong command or option exits 2 (CMD-3).
- The report is SARIF 2.1.0, one run, written with `serde_json`; no crate for SARIF's types (REPORT-1,
  DEP-1).
- The same project gives the same bytes on every platform: results by rule, path, line, column,
  fingerprint; notifications in `graph`'s order of notices (REPORT-1, reading RUN-8).
- A fingerprint is the first 32 hexadecimal digits of a SHA-256 digest, holds no line number, and is
  the same on every platform and run (REPORT-3, the spec's anti-goals).
- Every crate is pinned to one exact version with its reason in `Cargo.toml`: `sha2 = "=0.11.1"`
  (DEP-1).
- No walk over the graph is recursive (LIFE-3, which plan 5 tests at depth).
- `interface` prints what it printed; the interface version stays 1 (CMD-5).

## The decisions this plan makes

1. **Three tasks: the report, then the kinds that depend on entry points, then the other two.** Task 1
   builds `audit` with a report that holds no finding, so the command, the report's frame and the
   schema check come first and each later task adds findings to a report already checked.
2. **One reading for both commands.** `graph`'s reading moves, unchanged, into `reading::read`, which
   returns a `Reading` holding everything it found. `graph::printed` and `audit::printed` each take one.
   `graph` prints exactly what it printed.
3. **What the report holds.** One run, its `columnKind` `utf16CodeUnits`, which is how plan 3 counts an
   export's column. The tool's driver gives its name, version, `informationUri` and the five rules,
   each with an `id` and a one-sentence `shortDescription`. The interface version is the run's
   property `interfaceVersion`. One invocation, `executionSuccessful: true`, holds the notices as
   `toolExecutionNotifications`, each with its kind as `descriptor.id`, its sentence as
   `message.text`, `level: note`, and its file as a location where it has one. A result has
   `ruleId`, `ruleIndex`, `level: warning`, `message.text`, one location, `relatedLocations` where it
   has some, and `partialFingerprints`.
4. **Paths as SARIF writes them.** A path is a relative URI with `uriBaseId` `%SRCROOT%`, the root:
   each byte outside RFC 3986's unreserved characters, sub-delimiters, `@` and `/` is written as `%`
   and two hexadecimal digits, so `src/not done.ts` is `src/not%20done.ts`.
5. **The fingerprint.** Its input is the rule's name and the parts REPORT-3 names, each written as its
   length in bytes, a colon, and its bytes. It is kept under the key `yakAnalyzer/v1`.
6. **Where a kind is.** An unused file is the file, with no region. An unused export or type is its
   name, at the line and column plan 3 prints. An unresolved import and each file of a cycle are the
   import's line, with no column: plan 3 keeps no column for an import, and the specifier's line is
   what a person needs.
7. **The notices of ENTRY-9 and ENTRY-10 are `audit`'s.** They say where findings are not reported
   and what a computed import needs, so they come with the findings; `graph` already prints what they
   are worked out from, each package's `detected` and each file's `computedImports`. Given in `graph`
   too, ENTRY-9's notice would change the notices of nearly every earlier fixture, most of which have
   files in no package. So REPORT-4's "`graph` prints the same notices" holds for every notice of the
   reading, and these two kinds are the report's alone. Their kinds are `kinds-not-reported` and
   `computed-import`.
8. **What a package that holds analyzed files is (ENTRY-9).** One with any file the analyzer reads,
   a script, a type declaration file, a component, a document or a page, that is not data. A package's
   notice names its folder; the files in no package get one notice with their count.
9. **What an unused file is (KIND-1).** A listed script or Vue component, not data, not generated, not
   gone since it was listed, in a reported package, that `reachable` does not hold. Reach follows
   every import, so a file a `<script setup>` block imports and never uses is reached; its exports'
   use is what tells.
10. **What an unused export is (KIND-2).** An export of a reachable script or component, in a reported
    package, not data and not generated, that no use names, by its name or as every export. A use from
    the file's own template counts: it is the component's use (plan 3's decision 12). An unreachable
    file's exports are not reported.
11. **ENTRY-8's message** is a second sentence on an unused file in a package `entryFaults` names: the
    fault, and that the likely fault is a missing `entries` declaration.
12. **The edges of a cycle (KIND-4).** An import of the forms `import`, `export … from`, `export *`,
    `import … = require` and `require()`, that brings in more than types and carries no query, from
    a file that is not a type declaration file to another such file. An import that carries a query
    reads its target as text (USE-6), so it loads nothing. An `import` or `export … from` all of whose
    names are marked `type` now prints `typeOnly: true` in `graph`, which is what that field says.
13. **The sets are found by Tarjan's walk** over the files in path order, written as a loop over a
    stack of its own; each file's targets are taken in path order, so the sets and the loops are the
    same on every platform.
14. **The message of a cycle** names the loop as `` `a` → `b` → `a` ``: up to ten files, then "N more",
    then the first file again; and, when the set holds more files than the loop, how many it holds.
15. **No result is new or inherited yet.** Without `--base` every result has no `baselineState`,
    which is CMD-1's form without a base; `--base` is plan 5's, so `audit --base` is refused here as
    any unknown option is.
16. **The README and `CONTRIBUTING.md` wait for plan 6 (DOC-1),** as plan 1 placed them. The README's
    status line, that the analyzer reports no findings yet, is held by a test and changes with it
    there. A release made between this plan and plan 6 would carry it.
17. **The tests of `audit` are a new test file, `tests/audit.test.rs`.** Its `[[test]]` entry is in
    `Cargo.toml`, which is applied with the code, so task 1's tests-first run cannot run it. Task 1's
    test command therefore runs `cli` first, whose new test of `audit` fails for the right reason, and
    `audit` after it only when `cli` passes. Tasks 2 and 3 add tests to the target task 1 made.

## Review focus

- **Unused files on a project with no hand-off file.** Without declarations, a project's test files,
  scripts and configuration files are entry points only where a standard file names them, so they are
  reported unused. On three local checkouts of 2,600 to 3,800 files, `audit` with no hand-off file
  gave 1, 162 and 600 findings, nearly all unused files of that kind; plan 6 runs with hand-written
  declarations and judges each one (LIVE-2, LIVE-3).
- **A file that only a `src` attribute, an HTML page or a literal `import()` loads in a loop** is not in
  a cycle, since those forms are not edges (KIND-4). No fixture holds a page in one.
- **An import with a query inside a loop** breaks it (decision 12). No fixture holds one.
- **Two unresolved imports of one specifier in one file** share a fingerprint, as REPORT-3 has it;
  INH-1 in plan 5 counts them. The unresolved-import test holds two.
- **A notice about a file in the report** is a location with no region. The notices test holds one,
  with a space in its path.
- **Speed.** `audit` reads once, as `graph` does: about one to two seconds on the checkouts above, the
  findings adding little. LIVE-5 measures it in plan 6.

### Task 1: the audit command and its report

**Implements:** CMD-1, CMD-3, CMD-5, KIND-5, REPORT-1, REPORT-2, REPORT-4, FIX-3

**Test:** `cargo test --release --locked --test cli && cargo test --release --locked --test audit`

**Files:**
- Create: `src/reading.rs`, `src/audit.rs`, `src/findings.rs`
- Modify: `src/graph.rs`, `src/cli.rs`, `src/main.rs`, `src/notice.rs`, `Cargo.toml`
- Test: `tests/cli.test.rs`, `tests/audit.test.rs`, `tests/support/project.rs`,
  `tests/sarif/sarif-schema-2.1.0.json`, `tests/sarif/README.md`, `scripts/sarif-schema.test.mjs`

**Interfaces:**
- Consumes: everything `graph::read` used: `hand_off`, `declarations`, `files`, `marks`, `packages`,
  `tsconfig`, `scripts`, `resolve`, `entries` and `uses`; `notice::Notice`.
- Produces: `reading::read` and `reading::Reading` (`handed`, `files`, `marks`, `packages`, `scripts`,
  `resolved`, `entries`, `uses`, `notices`); `graph::printed`; `audit::printed`; `findings::Rule`
  with `ALL`, `name` and `description`; `cli::Command::Audit`; `Notice::kind`, `file` and `sentence`,
  and `notice::sorted`. Tasks 2 and 3 add the findings; plan 5 adds `--base` to `Command::Audit`.

- [ ] **Step 1: Write the failing tests**

The tests of the command line gain `audit`, and refuse `audit` with an argument it does not take:

```rust edit path=tests/cli.test.rs
<<<<<<< find
//! Starts the built binary and checks its commands and how it refuses (spec RUN-1, RUN-2, RUN-3, RUN-7).

use std::fs;
use std::path::Path;
=======
//! Starts the built binary and checks its commands and how it refuses (spec RUN-1, RUN-2, RUN-3, RUN-7;
//! dead-code spec CMD-1, CMD-3, CMD-5).

use std::fs;
use std::path::Path;
>>>>>>> replace
<<<<<<< find
    assert_eq!(document["notices"][0]["file"], Value::Null);
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    let cases: [&[&str]; 6] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
        &["interface", "extra"],
        &["graph", "extra"],
        &["graph", "--base", "main"],
    ];

    for args in cases {
=======
    assert_eq!(document["notices"][0]["file"], Value::Null);
}

#[test]
fn audit_without_a_hand_off_file_prints_one_sarif_report_and_exits_zero() {
    // An empty repository holds nothing to find: the report still names the tool and lists the rules.
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-audit");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("the test's folder can be made");
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git starts")
            .success()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .arg("audit")
        .current_dir(&root)
        .output()
        .expect("the binary starts");
    let report: Value = serde_json::from_slice(&output.stdout).expect("audit prints JSON");
    let run = &report["runs"][0];
    let notifications = &run["invocations"][0]["toolExecutionNotifications"];

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(report["version"], json!("2.1.0"));
    assert_eq!(report["runs"].as_array().map(Vec::len), Some(1));
    assert_eq!(run["tool"]["driver"]["name"], json!("yak-analyzer"));
    assert_eq!(
        run["tool"]["driver"]["rules"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(run["results"], json!([]));
    assert_eq!(notifications.as_array().map(Vec::len), Some(1));
    assert_eq!(
        notifications[0]["descriptor"]["id"],
        json!("no-hand-off-file")
    );
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    // `audit` takes no base yet: reading a commit comes with the plan after this one.
    let cases: [&[&str]; 8] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
        &["interface", "extra"],
        &["graph", "extra"],
        &["graph", "--base", "main"],
        &["audit", "extra"],
        &["audit", "--base", "main"],
    ];

    for args in cases {
>>>>>>> replace
<<<<<<< find
        assert!(error.starts_with("yak-analyzer: cannot do `"), "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(
            error.contains("--version, interface, and graph"),
            "{args:?}"
        );
    }
=======
        assert!(error.starts_with("yak-analyzer: cannot do `"), "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(
            error.contains("--version, interface, graph [--hand-off <file>], and audit"),
            "{args:?}"
        );
    }
>>>>>>> replace
<<<<<<< find
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version, interface, and graph"));
}

#[test]
=======
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version, interface, graph [--hand-off <file>], and audit"));
}

#[test]
>>>>>>> replace
```

The fixtures' helper runs any command of the binary, not only `graph`:

```rust edit path=tests/support/project.rs
<<<<<<< find
        self
    }

    /// Runs `graph` in the checkout, after the change the test asks for, and returns how it ended.
    pub fn run(&self, change: impl FnOnce(&mut Command)) -> Output {
        let mut binary = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"));

        binary.arg("graph").current_dir(&self.root);
        alone(&mut binary);
        change(&mut binary);

        binary.output().expect("the binary starts")
    }

    /// The document `graph` prints for the checkout. The run must succeed and write no line.
    pub fn graph(&self) -> Value {
        printed(&self.run(|_| {}))
=======
        self
    }

    /// Runs a command of the binary in the checkout, after the change the test asks for, and returns
    /// how it ended.
    pub fn start(&self, command: &str, change: impl FnOnce(&mut Command)) -> Output {
        let mut binary = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"));

        binary.arg(command).current_dir(&self.root);
        alone(&mut binary);
        change(&mut binary);

        binary.output().expect("the binary starts")
    }

    /// Runs `graph` in the checkout, after the change the test asks for, and returns how it ended.
    pub fn run(&self, change: impl FnOnce(&mut Command)) -> Output {
        self.start("graph", change)
    }

    /// The document `graph` prints for the checkout. The run must succeed and write no line.
    pub fn graph(&self) -> Value {
        printed(&self.run(|_| {}))
>>>>>>> replace
<<<<<<< find
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).expect("graph prints JSON")
}

/// The one line a stopped run wrote, after checking that it stopped as a stop does.
=======
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).expect("the binary prints JSON")
}

/// The one line a stopped run wrote, after checking that it stopped as a stop does.
>>>>>>> replace
```

A new test file runs `audit` on made-up projects, and checks every report against SARIF's schema:

```rust path=tests/audit.test.rs
//! Runs the built binary's `audit` on made-up projects and checks the report it prints: its form, its
//! notifications and how it stops (dead-code spec CMD-1, CMD-3, REPORT-1, REPORT-2, REPORT-4, FIX-3).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The copy of SARIF 2.1.0's schema. `tests/sarif/README.md` says where it came from.
const SCHEMA: &str = include_str!("sarif/sarif-schema-2.1.0.json");

/// The one run of the report `audit` prints for the checkout, with the hand-off file given or with
/// none. The run must succeed and write no line, and the schema must accept the report (spec FIX-3).
fn audit(project: &Project, hand_off: Option<&str>) -> Value {
    let report = printed(&project.start("audit", |binary| {
        if let Some(file) = hand_off {
            binary.args(["--hand-off", file]);
        }
    }));
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema's copy is JSON");
    let faults: Vec<String> = jsonschema::validator_for(&schema)
        .expect("the schema's copy is a JSON Schema")
        .iter_errors(&report)
        .map(|fault| format!("{}: {fault}", fault.instance_path()))
        .collect();

    assert_eq!(faults, Vec::<String>::new());
    assert_eq!(report["version"], json!("2.1.0"));
    assert_eq!(report["runs"].as_array().map(Vec::len), Some(1));

    report["runs"][0].clone()
}

/// The notifications of a run, in its order: each with its kind and the file it is about, as the
/// report writes it.
fn notifications(run: &Value) -> Vec<(&str, Option<&str>)> {
    run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .map(|notification| {
            (
                notification["descriptor"]["id"].as_str().expect("a kind"),
                notification["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str(),
            )
        })
        .collect()
}

#[test]
fn prints_one_report_that_names_the_tool_the_interface_version_and_the_five_rules() {
    let project = Project::new("audit-report");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { helper } from './helper';\n\nhelper();\n",
        )
        .write("src/helper.ts", "export const helper = (): void => {};\n")
        .commit();

    let run = audit(&project, None);
    let driver = &run["tool"]["driver"];
    let rules: Vec<&str> = driver["rules"]
        .as_array()
        .expect("a list of rules")
        .iter()
        .map(|rule| rule["id"].as_str().expect("a name"))
        .collect();

    assert_eq!(driver["name"], json!("yak-analyzer"));
    assert_eq!(driver["version"], json!(env!("CARGO_PKG_VERSION")));
    assert_eq!(run["properties"]["interfaceVersion"], json!(1));
    assert_eq!(
        rules,
        [
            "unused-file",
            "unused-export",
            "unused-type",
            "unresolved-import",
            "circular-dependency",
        ]
    );
    assert_eq!(run["invocations"][0]["executionSuccessful"], json!(true));
    // Every file here is reached, every export is used, and every import leads to a file.
    assert_eq!(run["results"], json!([]));
}

#[test]
fn the_notices_of_a_run_are_the_notifications_of_its_invocation() {
    let project = Project::new("audit-notices");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .commit()
        // An untracked script that does not parse is skipped, with a notice about the file.
        .write("src/not done.ts", "const = ;\n");

    let run = audit(&project, None);
    let document = project.graph();

    // The notices are the ones `graph` prints, in its order, and a file is named by its path from the
    // root, written as a URI (spec REPORT-1, REPORT-4).
    assert_eq!(
        notifications(&run),
        [
            ("no-hand-off-file", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
    assert_eq!(
        notifications(&run)
            .iter()
            .map(|(kind, _)| *kind)
            .collect::<Vec<&str>>(),
        notices(&document)
    );
    assert!(paths(&document).contains(&"src/not done.ts"));

    // With a hand-off file, its declarations are read, and what they give notice of is in the report.
    let hand_off = project.hand_off(
        &project.root,
        &["."],
        json!([{
            "name": "typescript",
            "interfaceVersion": 1,
            "pack": { "data": ["**/fixtures/**"] },
            "project": {}
        }]),
    );

    assert_eq!(
        notifications(&audit(&project, Some(&hand_off))),
        [
            ("glob-matches-nothing", None),
            ("untracked-file-skipped", Some("src/not%20done.ts")),
        ]
    );
}

#[test]
fn stops_where_graph_stops_with_the_same_line() {
    let project = Project::new("audit-stops");

    project.write("src/broken.ts", "const = ;\n").commit();

    // A run that cannot go on prints one line on standard error and exits 1, as `graph` does (spec
    // CMD-3).
    let line = stopped(&project.start("audit", |_| {}));

    assert!(line.contains("src/broken.ts"), "{line}");
    assert_eq!(line, stopped(&project.run(|_| {})));
}
```

A copy of SARIF 2.1.0's schema, as OASIS publishes it, with a note on where it came from and a test
that holds the note's digest to the copy, as the hand-off schema's copy has:

```markdown path=tests/sarif/README.md
# SARIF 2.1.0's schema, copied

`sarif-schema-2.1.0.json` is a copy of the JSON Schema of SARIF 2.1.0, the format `audit` prints its
report in. The tests check every report they read against this copy (dead-code spec FIX-3).

- **Published at:** `https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json`
- **Edition:** SARIF Version 2.1.0, OASIS Standard, Errata 01
- **SHA-256 of the file, with line feeds:** `c3b4bb2d6093897483348925aaa73af03b3e3f4bd4ca38cef26dcb4212a2682e`

SARIF 2.1.0 is a fixed standard, so the copy changes only if OASIS publishes another errata. When it
does, copy the file again and write the new address and digest here. `scripts/sarif-schema.test.mjs`
fails when the copy and the digest written here disagree.
```

```js path=scripts/sarif-schema.test.mjs
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The folder that holds the schema's copy and the note on where it came from. */
const FOLDER = join(ROOT, 'tests', 'sarif');

/** A text file's content with line feeds, whatever the checkout wrote. */
const text = (name) => readFileSync(join(FOLDER, name), 'utf8').replaceAll('\r\n', '\n');

test('the copy of the SARIF schema is the file the note beside it describes (spec FIX-3)', () => {
  const note = text('README.md');
  const stated = /SHA-256 of the file, with line feeds:\*\* `(?<digest>[0-9a-f]{64})`/u.exec(note)?.groups?.digest;
  const digest = createHash('sha256').update(text('sarif-schema-2.1.0.json')).digest('hex');

  assert.equal(stated, digest);
});

test('the note says where the copy came from: the address and the edition', () => {
  const note = text('README.md');
  const schema = JSON.parse(text('sarif-schema-2.1.0.json'));

  assert.ok(note.includes(`\`${schema.id}\``));
  assert.ok(note.includes('OASIS Standard, Errata 01'));
});

test('the copy is the schema of SARIF 2.1.0', () => {
  const schema = JSON.parse(text('sarif-schema-2.1.0.json'));

  assert.deepEqual(schema.properties.version.enum, ['2.1.0']);
  assert.deepEqual(schema.required, ['version', 'runs']);
});
```

```json path=tests/sarif/sarif-schema-2.1.0.json
{
  "$schema": "http://json-schema.org/draft-04/schema#",
  "title": "Static Analysis Results Format (SARIF) Version 2.1.0 JSON Schema",
  "id": "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json",
  "description": "Static Analysis Results Format (SARIF) Version 2.1.0 JSON Schema: a standard format for the output of static analysis tools.",
  "additionalProperties": false,
  "type": "object",
  "properties": {

    "$schema": {
      "description": "The URI of the JSON schema corresponding to the version.",
      "type": "string",
      "format": "uri"
    },

    "version": {
      "description": "The SARIF format version of this log file.",
      "enum": [ "2.1.0" ],
      "type": "string"
    },

    "runs": {
      "description": "The set of runs contained in this log file.",
      "type": [ "array", "null" ],
      "minItems": 0,
      "uniqueItems": false,
      "items": {
        "$ref": "#/definitions/run"
      }
    },

    "inlineExternalProperties": {
      "description": "References to external property files that share data between runs.",
      "type": "array",
      "minItems": 0,
      "uniqueItems": true,
      "items": {
        "$ref": "#/definitions/externalProperties"
      }
    },

    "properties": {
      "description": "Key/value pairs that provide additional information about the log file.",
      "$ref": "#/definitions/propertyBag"
    }
  },

  "required": [ "version", "runs" ],

  "definitions": {

    "address": {
      "description": "A physical or virtual address, or a range of addresses, in an 'addressable region' (memory or a binary file).",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "absoluteAddress": {
          "description": "The address expressed as a byte offset from the start of the addressable region.",
          "type": "integer",
          "minimum": -1,
          "default": -1

        },

        "relativeAddress": {
          "description": "The address expressed as a byte offset from the absolute address of the top-most parent object.",
          "type": "integer"

        },

        "length": {
          "description": "The number of bytes in this range of addresses.",
          "type": "integer"
        },

        "kind": {
          "description": "An open-ended string that identifies the address kind. 'data', 'function', 'header','instruction', 'module', 'page', 'section', 'segment', 'stack', 'stackFrame', 'table' are well-known values.",
          "type": "string"
        },

        "name": {
          "description": "A name that is associated with the address, e.g., '.text'.",
          "type": "string"
        },

        "fullyQualifiedName": {
          "description": "A human-readable fully qualified name that is associated with the address.",
          "type": "string"
        },

        "offsetFromParent": {
          "description": "The byte offset of this address from the absolute or relative address of the parent object.",
          "type": "integer"
        },

        "index": {
          "description": "The index within run.addresses of the cached object for this address.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "parentIndex": {
          "description": "The index within run.addresses of the parent object.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the address.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "artifact": {
      "description": "A single artifact. In some cases, this artifact might be nested within another artifact.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "description": {
          "description": "A short description of the artifact.",
          "$ref": "#/definitions/message"
        },

        "location": {
          "description": "The location of the artifact.",
          "$ref": "#/definitions/artifactLocation"
        },

        "parentIndex": {
          "description": "Identifies the index of the immediate parent of the artifact, if this artifact is nested.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "offset": {
          "description": "The offset in bytes of the artifact within its containing artifact.",
          "type": "integer",
          "minimum": 0
        },

        "length": {
          "description": "The length of the artifact in bytes.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "roles": {
          "description": "The role or roles played by the artifact in the analysis.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "enum": [
              "analysisTarget",
              "attachment",
              "responseFile",
              "resultFile",
              "standardStream",
              "tracedFile",
              "unmodified",
              "modified",
              "added",
              "deleted",
              "renamed",
              "uncontrolled",
              "driver",
              "extension",
              "translation",
              "taxonomy",
              "policy",
              "referencedOnCommandLine",
              "memoryContents",
              "directory",
              "userSpecifiedConfiguration",
              "toolSpecifiedConfiguration",
              "debugOutputFile"
            ],
            "type": "string"
          }
        },

        "mimeType": {
          "description": "The MIME type (RFC 2045) of the artifact.",
          "type": "string",
          "pattern": "[^/]+/.+"
        },

        "contents": {
          "description": "The contents of the artifact.",
          "$ref": "#/definitions/artifactContent"
        },

        "encoding": {
          "description": "Specifies the encoding for an artifact object that refers to a text file.",
          "type": "string"
        },

        "sourceLanguage": {
          "description": "Specifies the source language for any artifact object that refers to a text file that contains source code.",
          "type": "string"
        },

        "hashes": {
          "description": "A dictionary, each of whose keys is the name of a hash function and each of whose values is the hashed value of the artifact produced by the specified hash function.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "lastModifiedTimeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which the artifact was most recently modified. See \"Date/time properties\" in the SARIF spec for the required format.",
          "type": "string",
          "format": "date-time"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the artifact.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "artifactChange": {
      "description": "A change to a single artifact.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "artifactLocation": {
          "description": "The location of the artifact to change.",
          "$ref": "#/definitions/artifactLocation"
        },

        "replacements": {
          "description": "An array of replacement objects, each of which represents the replacement of a single region in a single artifact specified by 'artifactLocation'.",
          "type": "array",
          "minItems": 1,
          "uniqueItems": false,
          "items": {
            "$ref": "#/definitions/replacement"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the change.",
          "$ref": "#/definitions/propertyBag"
        }

      },

      "required": [ "artifactLocation", "replacements" ]
    },

    "artifactContent": {
      "description": "Represents the contents of an artifact.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "text": {
          "description": "UTF-8-encoded content from a text artifact.",
          "type": "string"
        },

        "binary": {
          "description": "MIME Base64-encoded content from a binary artifact, or from a text artifact in its original encoding.",
          "type": "string"
        },

        "rendered": {
          "description": "An alternate rendered representation of the artifact (e.g., a decompiled representation of a binary region).",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the artifact content.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "artifactLocation": {
      "description": "Specifies the location of an artifact.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "uri": {
          "description": "A string containing a valid relative or absolute URI.",
          "type": "string",
          "format": "uri-reference"
        },

        "uriBaseId": {
          "description": "A string which indirectly specifies the absolute URI with respect to which a relative URI in the \"uri\" property is interpreted.",
          "type": "string"
        },

        "index": {
          "description": "The index within the run artifacts array of the artifact object associated with the artifact location.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "description": {
          "description": "A short description of the artifact location.",
          "$ref": "#/definitions/message"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the artifact location.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "attachment": {
      "description": "An artifact relevant to a result.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "description": {
          "description": "A message describing the role played by the attachment.",
          "$ref": "#/definitions/message"
        },

        "artifactLocation": {
          "description": "The location of the attachment.",
          "$ref": "#/definitions/artifactLocation"
        },

        "regions": {
          "description": "An array of regions of interest within the attachment.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/region"
          }
        },

        "rectangles": {
          "description": "An array of rectangles specifying areas of interest within the image.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/rectangle"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the attachment.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "artifactLocation" ]
    },

    "codeFlow": {
      "description": "A set of threadFlows which together describe a pattern of code execution relevant to detecting a result.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "message": {
          "description": "A message relevant to the code flow.",
          "$ref": "#/definitions/message"
        },

        "threadFlows": {
          "description": "An array of one or more unique threadFlow objects, each of which describes the progress of a program through a thread of execution.",
          "type": "array",
          "minItems": 1,
          "uniqueItems": false,
          "items": {
            "$ref": "#/definitions/threadFlow"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the code flow.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "threadFlows" ]
    },

    "configurationOverride": {
      "description": "Information about how a specific rule or notification was reconfigured at runtime.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "configuration": {
          "description": "Specifies how the rule or notification was configured during the scan.",
          "$ref": "#/definitions/reportingConfiguration"
        },

        "descriptor": {
          "description": "A reference used to locate the descriptor whose configuration was overridden.",
          "$ref": "#/definitions/reportingDescriptorReference"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the configuration override.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "configuration", "descriptor" ]
    },

    "conversion": {
      "description": "Describes how a converter transformed the output of a static analysis tool from the analysis tool's native output format into the SARIF format.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "tool": {
          "description": "A tool object that describes the converter.",
          "$ref": "#/definitions/tool"
        },

        "invocation": {
          "description": "An invocation object that describes the invocation of the converter.",
          "$ref": "#/definitions/invocation"
        },

        "analysisToolLogFiles": {
          "description": "The locations of the analysis tool's per-run log files.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/artifactLocation"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the conversion.",
          "$ref": "#/definitions/propertyBag"
        }

      },

      "required": [ "tool" ]
    },

    "edge": {
      "description": "Represents a directed edge in a graph.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "id": {
          "description": "A string that uniquely identifies the edge within its graph.",
          "type": "string"
        },

        "label": {
          "description": "A short description of the edge.",
          "$ref": "#/definitions/message"
        },

        "sourceNodeId": {
          "description": "Identifies the source node (the node at which the edge starts).",
          "type": "string"
        },

        "targetNodeId": {
          "description": "Identifies the target node (the node at which the edge ends).",
          "type": "string"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the edge.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "id", "sourceNodeId", "targetNodeId" ]
    },

    "edgeTraversal": {
      "description": "Represents the traversal of a single edge during a graph traversal.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "edgeId": {
          "description": "Identifies the edge being traversed.",
          "type": "string"
        },

        "message": {
          "description": "A message to display to the user as the edge is traversed.",
          "$ref": "#/definitions/message"
        },

        "finalState": {
          "description": "The values of relevant expressions after the edge has been traversed.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "stepOverEdgeCount": {
          "description": "The number of edge traversals necessary to return from a nested graph.",
          "type": "integer",
          "minimum": 0
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the edge traversal.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "edgeId" ]
    },

    "exception": {
      "description": "Describes a runtime exception encountered during the execution of an analysis tool.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "kind": {
          "type": "string",
          "description": "A string that identifies the kind of exception, for example, the fully qualified type name of an object that was thrown, or the symbolic name of a signal."
        },

        "message": {
          "description": "A message that describes the exception.",
          "type": "string"
        },

        "stack": {
          "description": "The sequence of function calls leading to the exception.",
          "$ref": "#/definitions/stack"
        },

        "innerExceptions": {
          "description": "An array of exception objects each of which is considered a cause of this exception.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/exception"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the exception.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "externalProperties": {
      "description": "The top-level element of an external property file.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "schema": {
          "description": "The URI of the JSON schema corresponding to the version of the external property file format.",
          "type": "string",
          "format": "uri"
        },

        "version": {
          "description": "The SARIF format version of this external properties object.",
          "enum": [ "2.1.0" ],
          "type": "string"
        },

        "guid": {
          "description": "A stable, unique identifier for this external properties object, in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "runGuid": {
          "description": "A stable, unique identifier for the run associated with this external properties object, in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "conversion": {
          "description": "A conversion object that will be merged with a separate run.",
          "$ref": "#/definitions/conversion"
        },

        "graphs": {
          "description": "An array of graph objects that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "default": [],
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/graph"
          }
        },

        "externalizedProperties": {
          "description": "Key/value pairs that provide additional information that will be merged with a separate run.",
          "$ref": "#/definitions/propertyBag"
        },

        "artifacts": {
          "description": "An array of artifact objects that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/artifact"
          }
        },

        "invocations": {
          "description": "Describes the invocation of the analysis tool that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/invocation"
          }
        },

        "logicalLocations": {
          "description": "An array of logical locations such as namespaces, types or functions that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/logicalLocation"
          }
        },

        "threadFlowLocations": {
          "description": "An array of threadFlowLocation objects that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/threadFlowLocation"
          }
        },

        "results": {
          "description": "An array of result objects that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/result"
          }
        },

        "taxonomies": {
          "description": "Tool taxonomies that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "driver": {
          "description": "The analysis tool object that will be merged with a separate run.",
          "$ref": "#/definitions/toolComponent"
        },

        "extensions": {
          "description": "Tool extensions that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "policies": {
          "description": "Tool policies that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "translations": {
          "description": "Tool translations that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "addresses": {
          "description": "Addresses that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/address"
          }
        },

        "webRequests": {
          "description": "Requests that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/webRequest"
          }
        },

        "webResponses": {
          "description": "Responses that will be merged with a separate run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/webResponse"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the external properties.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "externalPropertyFileReference": {
      "description": "Contains information that enables a SARIF consumer to locate the external property file that contains the value of an externalized property associated with the run.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "location": {
          "description": "The location of the external property file.",
          "$ref": "#/definitions/artifactLocation"
        },

        "guid": {
          "description": "A stable, unique identifier for the external property file in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "itemCount": {
          "description": "A non-negative integer specifying the number of items contained in the external property file.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the external property file.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "anyOf": [
        { "required": [ "location" ] },
        { "required": [ "guid" ] }
      ]
    },

    "externalPropertyFileReferences": {
      "description": "References to external property files that should be inlined with the content of a root log file.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "conversion": {
          "description": "An external property file containing a run.conversion object to be merged with the root log file.",
          "$ref": "#/definitions/externalPropertyFileReference"
        },

        "graphs": {
          "description": "An array of external property files containing a run.graphs object to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "externalizedProperties": {
          "description": "An external property file containing a run.properties object to be merged with the root log file.",
          "$ref": "#/definitions/externalPropertyFileReference"
        },

        "artifacts": {
          "description": "An array of external property files containing run.artifacts arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "invocations": {
          "description": "An array of external property files containing run.invocations arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "logicalLocations": {
          "description": "An array of external property files containing run.logicalLocations arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "threadFlowLocations": {
          "description": "An array of external property files containing run.threadFlowLocations arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "results": {
          "description": "An array of external property files containing run.results arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "taxonomies": {
          "description": "An array of external property files containing run.taxonomies arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "addresses": {
          "description": "An array of external property files containing run.addresses arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "driver": {
          "description": "An external property file containing a run.driver object to be merged with the root log file.",
          "$ref": "#/definitions/externalPropertyFileReference"
        },

        "extensions": {
          "description": "An array of external property files containing run.extensions arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "policies": {
          "description": "An array of external property files containing run.policies arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "translations": {
          "description": "An array of external property files containing run.translations arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "webRequests": {
          "description": "An array of external property files containing run.requests arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "webResponses": {
          "description": "An array of external property files containing run.responses arrays to be merged with the root log file.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/externalPropertyFileReference"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the external property files.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "fix": {
      "description": "A proposed fix for the problem represented by a result object. A fix specifies a set of artifacts to modify. For each artifact, it specifies a set of bytes to remove, and provides a set of new bytes to replace them.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "description": {
          "description": "A message that describes the proposed fix, enabling viewers to present the proposed change to an end user.",
          "$ref": "#/definitions/message"
        },

        "artifactChanges": {
          "description": "One or more artifact changes that comprise a fix for a result.",
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/artifactChange"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the fix.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "artifactChanges" ]
    },

    "graph": {
      "description": "A network of nodes and directed edges that describes some aspect of the structure of the code (for example, a call graph).",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "description": {
          "description": "A description of the graph.",
          "$ref": "#/definitions/message"
        },

        "nodes": {
          "description": "An array of node objects representing the nodes of the graph.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/node"
          }
        },

        "edges": {
          "description": "An array of edge objects representing the edges of the graph.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/edge"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the graph.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "graphTraversal": {
      "description": "Represents a path through a graph.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "runGraphIndex": {
          "description": "The index within the run.graphs to be associated with the result.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "resultGraphIndex": {
          "description": "The index within the result.graphs to be associated with the result.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "description": {
          "description": "A description of this graph traversal.",
          "$ref": "#/definitions/message"
        },

        "initialState": {
          "description": "Values of relevant expressions at the start of the graph traversal that may change during graph traversal.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "immutableState": {
          "description": "Values of relevant expressions at the start of the graph traversal that remain constant for the graph traversal.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "edgeTraversals": {
          "description": "The sequences of edges traversed by this graph traversal.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/edgeTraversal"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the graph traversal.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "oneOf": [
        { "required": [ "runGraphIndex" ] },
        { "required": [ "resultGraphIndex" ] }
      ]
    },

    "invocation": {
      "description": "The runtime environment of the analysis tool run.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "commandLine": {
          "description": "The command line used to invoke the tool.",
          "type": "string"
        },

        "arguments": {
          "description": "An array of strings, containing in order the command line arguments passed to the tool from the operating system.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "items": {
            "type": "string"
          }
        },

        "responseFiles": {
          "description": "The locations of any response files specified on the tool's command line.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/artifactLocation"
          }
        },

        "startTimeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which the invocation started. See \"Date/time properties\" in the SARIF spec for the required format.",
          "type": "string",
          "format": "date-time"
        },

        "endTimeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which the invocation ended. See \"Date/time properties\" in the SARIF spec for the required format.",
          "type": "string",
          "format": "date-time"
        },

        "exitCode": {
          "description": "The process exit code.",
          "type": "integer"
        },

        "ruleConfigurationOverrides": {
          "description": "An array of configurationOverride objects that describe rules related runtime overrides.",
          "type": "array",
          "minItems": 0,
          "default": [],
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/configurationOverride"
          }
        },

        "notificationConfigurationOverrides": {
          "description": "An array of configurationOverride objects that describe notifications related runtime overrides.",
          "type": "array",
          "minItems": 0,
          "default": [],
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/configurationOverride"
          }
        },

        "toolExecutionNotifications": {
          "description": "A list of runtime conditions detected by the tool during the analysis.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/notification"
          }
        },

        "toolConfigurationNotifications": {
          "description": "A list of conditions detected by the tool that are relevant to the tool's configuration.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/notification"
          }
        },

        "exitCodeDescription": {
          "description": "The reason for the process exit.",
          "type": "string"
        },

        "exitSignalName": {
          "description": "The name of the signal that caused the process to exit.",
          "type": "string"
        },

        "exitSignalNumber": {
          "description": "The numeric value of the signal that caused the process to exit.",
          "type": "integer"
        },

        "processStartFailureMessage": {
          "description": "The reason given by the operating system that the process failed to start.",
          "type": "string"
        },

        "executionSuccessful": {
          "description": "Specifies whether the tool's execution completed successfully.",
          "type": "boolean"
        },

        "machine": {
          "description": "The machine on which the invocation occurred.",
          "type": "string"
        },

        "account": {
          "description": "The account under which the invocation occurred.",
          "type": "string"
        },

        "processId": {
          "description": "The id of the process in which the invocation occurred.",
          "type": "integer"
        },

        "executableLocation": {
          "description": "An absolute URI specifying the location of the executable that was invoked.",
          "$ref": "#/definitions/artifactLocation"
        },

        "workingDirectory": {
          "description": "The working directory for the invocation.",
          "$ref": "#/definitions/artifactLocation"
        },

        "environmentVariables": {
          "description": "The environment variables associated with the analysis tool process, expressed as key/value pairs.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "stdin": {
          "description": "A file containing the standard input stream to the process that was invoked.",
          "$ref": "#/definitions/artifactLocation"
        },

        "stdout": {
          "description": "A file containing the standard output stream from the process that was invoked.",
          "$ref": "#/definitions/artifactLocation"
        },

        "stderr": {
          "description": "A file containing the standard error stream from the process that was invoked.",
          "$ref": "#/definitions/artifactLocation"
        },

        "stdoutStderr": {
          "description": "A file containing the interleaved standard output and standard error stream from the process that was invoked.",
          "$ref": "#/definitions/artifactLocation"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the invocation.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "executionSuccessful" ]
    },

    "location": {
      "description": "A location within a programming artifact.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "id": {
          "description": "Value that distinguishes this location from all other locations within a single result object.",
          "type": "integer",
          "minimum": -1,
          "default": -1
        },

        "physicalLocation": {
          "description": "Identifies the artifact and region.",
          "$ref": "#/definitions/physicalLocation"
        },

        "logicalLocations": {
          "description": "The logical locations associated with the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/logicalLocation"
          }
        },

        "message": {
          "description": "A message relevant to the location.",
          "$ref": "#/definitions/message"
        },

        "annotations": {
          "description": "A set of regions relevant to the location.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/region"
          }
        },

        "relationships": {
          "description": "An array of objects that describe relationships between this location and others.",
          "type": "array",
          "default": [],
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/locationRelationship"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the location.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "locationRelationship": {
      "description": "Information about the relation of one location to another.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "target": {
          "description": "A reference to the related location.",
          "type": "integer",
          "minimum": 0
        },

        "kinds": {
          "description": "A set of distinct strings that categorize the relationship. Well-known kinds include 'includes', 'isIncludedBy' and 'relevant'.",
          "type": "array",
          "default": [ "relevant" ],
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },

        "description": {
          "description": "A description of the location relationship.",
          "$ref": "#/definitions/message"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the location relationship.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "target" ]
    },

    "logicalLocation": {
      "description": "A logical location of a construct that produced a result.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "name": {
          "description": "Identifies the construct in which the result occurred. For example, this property might contain the name of a class or a method.",
          "type": "string"
        },

        "index": {
          "description": "The index within the logical locations array.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "fullyQualifiedName": {
          "description": "The human-readable fully qualified name of the logical location.",
          "type": "string"
        },

        "decoratedName": {
          "description": "The machine-readable name for the logical location, such as a mangled function name provided by a C++ compiler that encodes calling convention, return type and other details along with the function name.",
          "type": "string"
        },

        "parentIndex": {
          "description": "Identifies the index of the immediate parent of the construct in which the result was detected. For example, this property might point to a logical location that represents the namespace that holds a type.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "kind": {
          "description": "The type of construct this logical location component refers to. Should be one of 'function', 'member', 'module', 'namespace', 'parameter', 'resource', 'returnType', 'type', 'variable', 'object', 'array', 'property', 'value', 'element', 'text', 'attribute', 'comment', 'declaration', 'dtd' or 'processingInstruction', if any of those accurately describe the construct.",
          "type": "string"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the logical location.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "message": {
      "description": "Encapsulates a message intended to be read by the end user.",
      "type": "object",
      "additionalProperties": false,

      "properties": {

        "text": {
          "description": "A plain text message string.",
          "type": "string"
        },

        "markdown": {
          "description": "A Markdown message string.",
          "type": "string"
        },

        "id": {
          "description": "The identifier for this message.",
          "type": "string"
        },

        "arguments": {
          "description": "An array of strings to substitute into the message string.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "type": "string"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the message.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "anyOf": [
        { "required": [ "text" ] },
        { "required": [ "id" ] }
      ]
    },

    "multiformatMessageString": {
      "description": "A message string or message format string rendered in multiple formats.",
      "type": "object",
      "additionalProperties": false,

      "properties": {

        "text": {
          "description": "A plain text message string or format string.",
          "type": "string"
        },

        "markdown": {
          "description": "A Markdown message string or format string.",
          "type": "string"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the message.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "text" ]
    },

    "node": {
      "description": "Represents a node in a graph.",
      "type": "object",
      "additionalProperties": false,

      "properties": {

        "id": {
          "description": "A string that uniquely identifies the node within its graph.",
          "type": "string"
        },

        "label": {
          "description": "A short description of the node.",
          "$ref": "#/definitions/message"
        },

        "location": {
          "description": "A code location associated with the node.",
          "$ref": "#/definitions/location"
        },

        "children": {
          "description": "Array of child nodes.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/node"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the node.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "id" ]
    },

    "notification": {
      "description": "Describes a condition relevant to the tool itself, as opposed to being relevant to a target being analyzed by the tool.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "locations": {
          "description": "The locations relevant to this notification.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/location"
          }
        },

        "message": {
          "description": "A message that describes the condition that was encountered.",
          "$ref": "#/definitions/message"
        },

        "level": {
          "description": "A value specifying the severity level of the notification.",
          "default": "warning",
          "enum": [ "none", "note", "warning", "error" ],
          "type": "string"
        },

        "threadId": {
          "description": "The thread identifier of the code that generated the notification.",
          "type": "integer"
        },

        "timeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which the analysis tool generated the notification.",
          "type": "string",
          "format": "date-time"
        },

        "exception": {
          "description": "The runtime exception, if any, relevant to this notification.",
          "$ref": "#/definitions/exception"
        },

        "descriptor": {
          "description": "A reference used to locate the descriptor relevant to this notification.",
          "$ref": "#/definitions/reportingDescriptorReference"
        },

        "associatedRule": {
          "description": "A reference used to locate the rule descriptor associated with this notification.",
          "$ref": "#/definitions/reportingDescriptorReference"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the notification.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "message" ]
    },

    "physicalLocation": {
      "description": "A physical location relevant to a result. Specifies a reference to a programming artifact together with a range of bytes or characters within that artifact.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "address": {
          "description": "The address of the location.",
          "$ref": "#/definitions/address"
        },

        "artifactLocation": {
          "description": "The location of the artifact.",
          "$ref": "#/definitions/artifactLocation"
        },

        "region": {
          "description": "Specifies a portion of the artifact.",
          "$ref": "#/definitions/region"
        },

        "contextRegion": {
          "description": "Specifies a portion of the artifact that encloses the region. Allows a viewer to display additional context around the region.",
          "$ref": "#/definitions/region"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the physical location.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "anyOf": [
        {
          "required": [ "address" ]
        },
        {
          "required": [ "artifactLocation" ]
        }
      ]
    },

    "propertyBag": {
      "description": "Key/value pairs that provide additional information about the object.",
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "tags": {

          "description": "A set of distinct strings that provide additional information.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "type": "string"
          }
        }
      }
    },

    "rectangle": {
      "description": "An area within an image.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "top": {
          "description": "The Y coordinate of the top edge of the rectangle, measured in the image's natural units.",
          "type": "number"
        },

        "left": {
          "description": "The X coordinate of the left edge of the rectangle, measured in the image's natural units.",
          "type": "number"
        },

        "bottom": {
          "description": "The Y coordinate of the bottom edge of the rectangle, measured in the image's natural units.",
          "type": "number"
        },

        "right": {
          "description": "The X coordinate of the right edge of the rectangle, measured in the image's natural units.",
          "type": "number"
        },

        "message": {
          "description": "A message relevant to the rectangle.",
          "$ref": "#/definitions/message"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the rectangle.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "region": {
      "description": "A region within an artifact where a result was detected.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "startLine": {
          "description": "The line number of the first character in the region.",
          "type": "integer",
          "minimum": 1
        },

        "startColumn": {
          "description": "The column number of the first character in the region.",
          "type": "integer",
          "minimum": 1
        },

        "endLine": {
          "description": "The line number of the last character in the region.",
          "type": "integer",
          "minimum": 1
        },

        "endColumn": {
          "description": "The column number of the character following the end of the region.",
          "type": "integer",
          "minimum": 1
        },

        "charOffset": {
          "description": "The zero-based offset from the beginning of the artifact of the first character in the region.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "charLength": {
          "description": "The length of the region in characters.",
          "type": "integer",
          "minimum": 0
        },

        "byteOffset": {
          "description": "The zero-based offset from the beginning of the artifact of the first byte in the region.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "byteLength": {
          "description": "The length of the region in bytes.",
          "type": "integer",
          "minimum": 0
        },

        "snippet": {
          "description": "The portion of the artifact contents within the specified region.",
          "$ref": "#/definitions/artifactContent"
        },

        "message": {
          "description": "A message relevant to the region.",
          "$ref": "#/definitions/message"
        },

        "sourceLanguage": {
          "description": "Specifies the source language, if any, of the portion of the artifact specified by the region object.",
          "type": "string"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the region.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "anyOf": [
        { "required": [ "startLine" ] },
        { "required": [ "charOffset" ] },
        { "required": [ "byteOffset" ] }
      ]
    },

    "replacement": {
      "description": "The replacement of a single region of an artifact.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "deletedRegion": {
          "description": "The region of the artifact to delete.",
          "$ref": "#/definitions/region"
        },

        "insertedContent": {
          "description": "The content to insert at the location specified by the 'deletedRegion' property.",
          "$ref": "#/definitions/artifactContent"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the replacement.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "deletedRegion" ]
    },

    "reportingDescriptor": {
      "description": "Metadata that describes a specific report produced by the tool, as part of the analysis it provides or its runtime reporting.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "id": {
          "description": "A stable, opaque identifier for the report.",
          "type": "string"
        },

        "deprecatedIds": {
          "description": "An array of stable, opaque identifiers by which this report was known in some previous version of the analysis tool.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },

        "guid": {
          "description": "A unique identifier for the reporting descriptor in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "deprecatedGuids": {
          "description": "An array of unique identifies in the form of a GUID by which this report was known in some previous version of the analysis tool.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "type": "string",
            "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
          }
        },

        "name": {
          "description": "A report identifier that is understandable to an end user.",
          "type": "string"
        },

        "deprecatedNames": {
          "description": "An array of readable identifiers by which this report was known in some previous version of the analysis tool.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },

        "shortDescription": {
          "description": "A concise description of the report. Should be a single sentence that is understandable when visible space is limited to a single line of text.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "fullDescription": {
          "description": "A description of the report. Should, as far as possible, provide details sufficient to enable resolution of any problem indicated by the result.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "messageStrings": {
          "description": "A set of name/value pairs with arbitrary names. Each value is a multiformatMessageString object, which holds message strings in plain text and (optionally) Markdown format. The strings can include placeholders, which can be used to construct a message in combination with an arbitrary number of additional string arguments.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "defaultConfiguration": {
          "description": "Default reporting configuration information.",
          "$ref": "#/definitions/reportingConfiguration"
        },

        "helpUri": {
          "description": "A URI where the primary documentation for the report can be found.",
          "type": "string",
          "format": "uri"
        },

        "help": {
          "description": "Provides the primary documentation for the report, useful when there is no online documentation.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "relationships": {
          "description": "An array of objects that describe relationships between this reporting descriptor and others.",
          "type": "array",
          "default": [],
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/reportingDescriptorRelationship"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the report.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "id" ]
    },

    "reportingConfiguration": {
      "description": "Information about a rule or notification that can be configured at runtime.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "enabled": {
          "description": "Specifies whether the report may be produced during the scan.",
          "type": "boolean",
          "default": true
        },

        "level": {
          "description": "Specifies the failure level for the report.",
          "default": "warning",
          "enum": [ "none", "note", "warning", "error" ],
          "type": "string"
        },

        "rank": {
          "description": "Specifies the relative priority of the report. Used for analysis output only.",
          "type": "number",
          "default": -1.0,
          "minimum": -1.0,
          "maximum": 100.0
        },

        "parameters": {
          "description": "Contains configuration information specific to a report.",
          "$ref": "#/definitions/propertyBag"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the reporting configuration.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "reportingDescriptorReference": {
      "description": "Information about how to locate a relevant reporting descriptor.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "id": {
          "description": "The id of the descriptor.",
          "type": "string"
        },

        "index": {
          "description": "The index into an array of descriptors in toolComponent.ruleDescriptors, toolComponent.notificationDescriptors, or toolComponent.taxonomyDescriptors, depending on context.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "guid": {
          "description": "A guid that uniquely identifies the descriptor.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "toolComponent": {
          "description": "A reference used to locate the toolComponent associated with the descriptor.",
          "$ref": "#/definitions/toolComponentReference"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the reporting descriptor reference.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "anyOf": [
        { "required": [ "index" ] },
        { "required": [ "guid" ] },
        { "required": [ "id" ] }
      ]
    },

    "reportingDescriptorRelationship": {
      "description": "Information about the relation of one reporting descriptor to another.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "target": {
          "description": "A reference to the related reporting descriptor.",
          "$ref": "#/definitions/reportingDescriptorReference"
        },

        "kinds": {
          "description": "A set of distinct strings that categorize the relationship. Well-known kinds include 'canPrecede', 'canFollow', 'willPrecede', 'willFollow', 'superset', 'subset', 'equal', 'disjoint', 'relevant', and 'incomparable'.",
          "type": "array",
          "default": [ "relevant" ],
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },

        "description": {
          "description": "A description of the reporting descriptor relationship.",
          "$ref": "#/definitions/message"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the reporting descriptor reference.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "target" ]
    },

    "result": {
      "description": "A result produced by an analysis tool.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "ruleId": {
          "description": "The stable, unique identifier of the rule, if any, to which this result is relevant.",
          "type": "string"
        },

        "ruleIndex": {
          "description": "The index within the tool component rules array of the rule object associated with this result.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "rule": {
          "description": "A reference used to locate the rule descriptor relevant to this result.",
          "$ref": "#/definitions/reportingDescriptorReference"
        },

        "kind": {
          "description": "A value that categorizes results by evaluation state.",
          "default": "fail",
          "enum": [ "notApplicable", "pass", "fail", "review", "open", "informational" ],
          "type": "string"
        },

        "level": {
          "description": "A value specifying the severity level of the result.",
          "default": "warning",
          "enum": [ "none", "note", "warning", "error" ],
          "type": "string"
        },

        "message": {
          "description": "A message that describes the result. The first sentence of the message only will be displayed when visible space is limited.",
          "$ref": "#/definitions/message"
        },

        "analysisTarget": {
          "description": "Identifies the artifact that the analysis tool was instructed to scan. This need not be the same as the artifact where the result actually occurred.",
          "$ref": "#/definitions/artifactLocation"
        },

        "locations": {
          "description": "The set of locations where the result was detected. Specify only one location unless the problem indicated by the result can only be corrected by making a change at every specified location.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/location"
          }
        },

        "guid": {
          "description": "A stable, unique identifier for the result in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "correlationGuid": {
          "description": "A stable, unique identifier for the equivalence class of logically identical results to which this result belongs, in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "occurrenceCount": {
          "description": "A positive integer specifying the number of times this logically unique result was observed in this run.",
          "type": "integer",
          "minimum": 1
        },

        "partialFingerprints": {
          "description": "A set of strings that contribute to the stable, unique identity of the result.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "fingerprints": {
          "description": "A set of strings each of which individually defines a stable, unique identity for the result.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "stacks": {
          "description": "An array of 'stack' objects relevant to the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/stack"
          }
        },

        "codeFlows": {
          "description": "An array of 'codeFlow' objects relevant to the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/codeFlow"
          }
        },

        "graphs": {
          "description": "An array of zero or more unique graph objects associated with the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/graph"
          }
        },

        "graphTraversals": {
          "description": "An array of one or more unique 'graphTraversal' objects.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/graphTraversal"
          }
        },

        "relatedLocations": {
          "description": "A set of locations relevant to this result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/location"
          }
        },

        "suppressions": {
          "description": "A set of suppressions relevant to this result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/suppression"
          }
        },

        "baselineState": {
          "description": "The state of a result relative to a baseline of a previous run.",
          "enum": [
            "new",
            "unchanged",
            "updated",
            "absent"
          ],
          "type": "string"
        },

        "rank": {
          "description": "A number representing the priority or importance of the result.",
          "type": "number",
          "default": -1.0,
          "minimum": -1.0,
          "maximum": 100.0
        },

        "attachments": {
          "description": "A set of artifacts relevant to the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/attachment"
          }
        },

        "hostedViewerUri": {
          "description": "An absolute URI at which the result can be viewed.",
          "type": "string",
          "format": "uri"
        },

        "workItemUris": {
          "description": "The URIs of the work items associated with this result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "type": "string",
            "format": "uri"
          }
        },

        "provenance": {
          "description": "Information about how and when the result was detected.",
          "$ref": "#/definitions/resultProvenance"
        },

        "fixes": {
          "description": "An array of 'fix' objects, each of which represents a proposed fix to the problem indicated by the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/fix"
          }
        },

        "taxa": {
          "description": "An array of references to taxonomy reporting descriptors that are applicable to the result.",
          "type": "array",
          "default": [],
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/reportingDescriptorReference"
          }
        },

        "webRequest": {
          "description": "A web request associated with this result.",
          "$ref": "#/definitions/webRequest"
        },

        "webResponse": {
          "description": "A web response associated with this result.",
          "$ref": "#/definitions/webResponse"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the result.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "message" ]
    },

    "resultProvenance": {
      "description": "Contains information about how and when a result was detected.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "firstDetectionTimeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which the result was first detected. See \"Date/time properties\" in the SARIF spec for the required format.",
          "type": "string",
          "format": "date-time"
        },

        "lastDetectionTimeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which the result was most recently detected. See \"Date/time properties\" in the SARIF spec for the required format.",
          "type": "string",
          "format": "date-time"
        },

        "firstDetectionRunGuid": {
          "description": "A GUID-valued string equal to the automationDetails.guid property of the run in which the result was first detected.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "lastDetectionRunGuid": {
          "description": "A GUID-valued string equal to the automationDetails.guid property of the run in which the result was most recently detected.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "invocationIndex": {
          "description": "The index within the run.invocations array of the invocation object which describes the tool invocation that detected the result.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "conversionSources": {
          "description": "An array of physicalLocation objects which specify the portions of an analysis tool's output that a converter transformed into the result.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/physicalLocation"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the result.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "run": {
      "description": "Describes a single run of an analysis tool, and contains the reported output of that run.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "tool": {
          "description": "Information about the tool or tool pipeline that generated the results in this run. A run can only contain results produced by a single tool or tool pipeline. A run can aggregate results from multiple log files, as long as context around the tool run (tool command-line arguments and the like) is identical for all aggregated files.",
          "$ref": "#/definitions/tool"
        },

        "invocations": {
          "description": "Describes the invocation of the analysis tool.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/invocation"
          }
        },

        "conversion": {
          "description": "A conversion object that describes how a converter transformed an analysis tool's native reporting format into the SARIF format.",
          "$ref": "#/definitions/conversion"
        },

        "language": {
          "description": "The language of the messages emitted into the log file during this run (expressed as an ISO 639-1 two-letter lowercase culture code) and an optional region (expressed as an ISO 3166-1 two-letter uppercase subculture code associated with a country or region). The casing is recommended but not required (in order for this data to conform to RFC5646).",
          "type": "string",
          "default": "en-US",
          "pattern": "^[a-zA-Z]{2}(-[a-zA-Z]{2})?$"
        },

        "versionControlProvenance": {
          "description": "Specifies the revision in version control of the artifacts that were scanned.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/versionControlDetails"
          }
        },

        "originalUriBaseIds": {
          "description": "The artifact location specified by each uriBaseId symbol on the machine where the tool originally ran.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/artifactLocation"
          }
        },

        "artifacts": {
          "description": "An array of artifact objects relevant to the run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/artifact"
          }
        },

        "logicalLocations": {
          "description": "An array of logical locations such as namespaces, types or functions.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/logicalLocation"
          }
        },

        "graphs": {
          "description": "An array of zero or more unique graph objects associated with the run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/graph"
          }
        },

        "results": {
          "description": "The set of results contained in an SARIF log. The results array can be omitted when a run is solely exporting rules metadata. It must be present (but may be empty) if a log file represents an actual scan.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "items": {
            "$ref": "#/definitions/result"
          }
        },

        "automationDetails": {
          "description": "Automation details that describe this run.",
          "$ref": "#/definitions/runAutomationDetails"
        },

        "runAggregates": {
          "description": "Automation details that describe the aggregate of runs to which this run belongs.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/runAutomationDetails"
          }
        },

        "baselineGuid": {
          "description": "The 'guid' property of a previous SARIF 'run' that comprises the baseline that was used to compute result 'baselineState' properties for the run.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "redactionTokens": {
          "description": "An array of strings used to replace sensitive information in a redaction-aware property.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "type": "string"
          }
        },

        "defaultEncoding": {
          "description": "Specifies the default encoding for any artifact object that refers to a text file.",
          "type": "string"
        },

        "defaultSourceLanguage": {
          "description": "Specifies the default source language for any artifact object that refers to a text file that contains source code.",
          "type": "string"
        },

        "newlineSequences": {
          "description": "An ordered list of character sequences that were treated as line breaks when computing region information for the run.",
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "default": [ "\r\n", "\n" ],
          "items": {
            "type": "string"
          }
        },

        "columnKind": {
          "description": "Specifies the unit in which the tool measures columns.",
          "enum": [ "utf16CodeUnits", "unicodeCodePoints" ],
          "type": "string"
        },

        "externalPropertyFileReferences": {
          "description": "References to external property files that should be inlined with the content of a root log file.",
          "$ref": "#/definitions/externalPropertyFileReferences"
        },

        "threadFlowLocations": {
          "description": "An array of threadFlowLocation objects cached at run level.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/threadFlowLocation"
          }
        },

        "taxonomies": {
          "description": "An array of toolComponent objects relevant to a taxonomy in which results are categorized.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "addresses": {
          "description": "Addresses associated with this run instance, if any.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "$ref": "#/definitions/address"
          }
        },

        "translations": {
          "description": "The set of available translations of the localized data provided by the tool.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "policies": {
          "description": "Contains configurations that may potentially override both reportingDescriptor.defaultConfiguration (the tool's default severities) and invocation.configurationOverrides (severities established at run-time from the command line).",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "webRequests": {
          "description": "An array of request objects cached at run level.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/webRequest"
          }
        },

        "webResponses": {
          "description": "An array of response objects cached at run level.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/webResponse"
          }
        },

        "specialLocations": {
          "description": "A specialLocations object that defines locations of special significance to SARIF consumers.",
          "$ref": "#/definitions/specialLocations"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the run.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "tool" ]
    },

    "runAutomationDetails": {
      "description": "Information that describes a run's identity and role within an engineering system process.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "description": {
          "description": "A description of the identity and role played within the engineering system by this object's containing run object.",
          "$ref": "#/definitions/message"
        },

        "id": {
          "description": "A hierarchical string that uniquely identifies this object's containing run object.",
          "type": "string"
        },

        "guid": {
          "description": "A stable, unique identifier for this object's containing run object in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "correlationGuid": {
          "description": "A stable, unique identifier for the equivalence class of runs to which this object's containing run object belongs in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the run automation details.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "specialLocations": {
      "description": "Defines locations of special significance to SARIF consumers.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "displayBase": {
          "description": "Provides a suggestion to SARIF consumers to display file paths relative to the specified location.",
          "$ref": "#/definitions/artifactLocation"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the special locations.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "stack": {
      "description": "A call stack that is relevant to a result.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "message": {
          "description": "A message relevant to this call stack.",
          "$ref": "#/definitions/message"
        },

        "frames": {
          "description": "An array of stack frames that represents a sequence of calls, rendered in reverse chronological order, that comprise the call stack.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "items": {
            "$ref": "#/definitions/stackFrame"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the stack.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "frames" ]
    },

    "stackFrame": {
      "description": "A function call within a stack trace.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "location": {
          "description": "The location to which this stack frame refers.",
          "$ref": "#/definitions/location"
        },

        "module": {
          "description": "The name of the module that contains the code of this stack frame.",
          "type": "string"
        },

        "threadId": {
          "description": "The thread identifier of the stack frame.",
          "type": "integer"
        },

        "parameters": {
          "description": "The parameters of the call that is executing.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": false,
          "default": [],
          "items": {
            "type": "string",
            "default": []
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the stack frame.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "suppression": {
      "description": "A suppression that is relevant to a result.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "guid": {
          "description": "A stable, unique identifier for the suprression in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "kind": {
          "description": "A string that indicates where the suppression is persisted.",
          "enum": [
            "inSource",
            "external"
          ],
          "type": "string"
        },

        "status": {
          "description": "A string that indicates the review status of the suppression.",
          "enum": [
            "accepted",
            "underReview",
            "rejected"
          ],
          "type": "string"
        },

        "justification": {
          "description": "A string representing the justification for the suppression.",
          "type": "string"
        },

        "location": {
          "description": "Identifies the location associated with the suppression.",
          "$ref": "#/definitions/location"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the suppression.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "kind" ]
    },

    "threadFlow": {
      "description": "Describes a sequence of code locations that specify a path through a single thread of execution such as an operating system or fiber.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "id": {
          "description": "An string that uniquely identifies the threadFlow within the codeFlow in which it occurs.",
          "type": "string"
        },

        "message": {
          "description": "A message relevant to the thread flow.",
          "$ref": "#/definitions/message"
        },


        "initialState": {
          "description": "Values of relevant expressions at the start of the thread flow that may change during thread flow execution.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "immutableState": {
          "description": "Values of relevant expressions at the start of the thread flow that remain constant.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "locations": {
          "description": "A temporally ordered array of 'threadFlowLocation' objects, each of which describes a location visited by the tool while producing the result.",
          "type": "array",
          "minItems": 1,
          "uniqueItems": false,
          "items": {
            "$ref": "#/definitions/threadFlowLocation"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the thread flow.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "locations" ]
    },

    "threadFlowLocation": {
      "description": "A location visited by an analysis tool while simulating or monitoring the execution of a program.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "index": {
          "description": "The index within the run threadFlowLocations array.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "location": {
          "description": "The code location.",
          "$ref": "#/definitions/location"
        },

        "stack": {
          "description": "The call stack leading to this location.",
          "$ref": "#/definitions/stack"
        },

        "kinds": {
          "description": "A set of distinct strings that categorize the thread flow location. Well-known kinds include 'acquire', 'release', 'enter', 'exit', 'call', 'return', 'branch', 'implicit', 'false', 'true', 'caution', 'danger', 'unknown', 'unreachable', 'taint', 'function', 'handler', 'lock', 'memory', 'resource', 'scope' and 'value'.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "type": "string"
          }
        },

        "taxa": {
          "description": "An array of references to rule or taxonomy reporting descriptors that are applicable to the thread flow location.",
          "type": "array",
          "default": [],
          "minItems": 0,
          "uniqueItems": true,
          "items": {
            "$ref": "#/definitions/reportingDescriptorReference"
          }
        },

        "module": {
          "description": "The name of the module that contains the code that is executing.",
          "type": "string"
        },

        "state": {
          "description": "A dictionary, each of whose keys specifies a variable or expression, the associated value of which represents the variable or expression value. For an annotation of kind 'continuation', for example, this dictionary might hold the current assumed values of a set of global variables.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "nestingLevel": {
          "description": "An integer representing a containment hierarchy within the thread flow.",
          "type": "integer",
          "minimum": 0
        },

        "executionOrder": {
          "description": "An integer representing the temporal order in which execution reached this location.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "executionTimeUtc": {
          "description": "The Coordinated Universal Time (UTC) date and time at which this location was executed.",
          "type": "string",
          "format": "date-time"
        },

        "importance": {
          "description": "Specifies the importance of this location in understanding the code flow in which it occurs. The order from most to least important is \"essential\", \"important\", \"unimportant\". Default: \"important\".",
          "enum": [ "important", "essential", "unimportant" ],
          "default": "important",
          "type": "string"
        },

        "webRequest": {
          "description": "A web request associated with this thread flow location.",
          "$ref": "#/definitions/webRequest"
        },

        "webResponse": {
          "description": "A web response associated with this thread flow location.",
          "$ref": "#/definitions/webResponse"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the threadflow location.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "tool": {
      "description": "The analysis tool that was run.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "driver": {
          "description": "The analysis tool that was run.",
          "$ref": "#/definitions/toolComponent"
        },

        "extensions": {
          "description": "Tool extensions that contributed to or reconfigured the analysis tool that was run.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponent"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the tool.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "driver" ]
    },

    "toolComponent": {
      "description": "A component, such as a plug-in or the driver, of the analysis tool that was run.",
      "additionalProperties": false,
      "type": "object",
      "properties": {

        "guid": {
          "description": "A unique identifier for the tool component in the form of a GUID.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "name": {
          "description": "The name of the tool component.",
          "type": "string"
        },

        "organization": {
          "description": "The organization or company that produced the tool component.",
          "type": "string"
        },

        "product": {
          "description": "A product suite to which the tool component belongs.",
          "type": "string"
        },

        "productSuite": {
          "description": "A localizable string containing the name of the suite of products to which the tool component belongs.",
          "type": "string"
        },

        "shortDescription": {
          "description": "A brief description of the tool component.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "fullDescription": {
          "description": "A comprehensive description of the tool component.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "fullName": {
          "description": "The name of the tool component along with its version and any other useful identifying information, such as its locale.",
          "type": "string"
        },

        "version": {
          "description": "The tool component version, in whatever format the component natively provides.",
          "type": "string"
        },

        "semanticVersion": {
          "description": "The tool component version in the format specified by Semantic Versioning 2.0.",
          "type": "string"
        },

        "dottedQuadFileVersion": {
          "description": "The binary version of the tool component's primary executable file expressed as four non-negative integers separated by a period (for operating systems that express file versions in this way).",
          "type": "string",
          "pattern": "[0-9]+(\\.[0-9]+){3}"
        },

        "releaseDateUtc": {
          "description": "A string specifying the UTC date (and optionally, the time) of the component's release.",
          "type": "string"
        },

        "downloadUri": {
          "description": "The absolute URI from which the tool component can be downloaded.",
          "type": "string",
          "format": "uri"
        },

        "informationUri": {
          "description": "The absolute URI at which information about this version of the tool component can be found.",
          "type": "string",
          "format": "uri"
        },

        "globalMessageStrings": {
          "description": "A dictionary, each of whose keys is a resource identifier and each of whose values is a multiformatMessageString object, which holds message strings in plain text and (optionally) Markdown format. The strings can include placeholders, which can be used to construct a message in combination with an arbitrary number of additional string arguments.",
          "type": "object",
          "additionalProperties": {
            "$ref": "#/definitions/multiformatMessageString"
          }
        },

        "notifications": {
          "description": "An array of reportingDescriptor objects relevant to the notifications related to the configuration and runtime execution of the tool component.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/reportingDescriptor"
          }
        },

        "rules": {
          "description": "An array of reportingDescriptor objects relevant to the analysis performed by the tool component.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/reportingDescriptor"
          }
        },

        "taxa": {
          "description": "An array of reportingDescriptor objects relevant to the definitions of both standalone and tool-defined taxonomies.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/reportingDescriptor"
          }
        },

        "locations": {
          "description": "An array of the artifactLocation objects associated with the tool component.",
          "type": "array",
          "minItems": 0,
          "default": [],
          "items": {
            "$ref": "#/definitions/artifactLocation"
          }
        },

        "language": {
          "description": "The language of the messages emitted into the log file during this run (expressed as an ISO 639-1 two-letter lowercase language code) and an optional region (expressed as an ISO 3166-1 two-letter uppercase subculture code associated with a country or region). The casing is recommended but not required (in order for this data to conform to RFC5646).",
          "type": "string",
          "default": "en-US",
          "pattern": "^[a-zA-Z]{2}(-[a-zA-Z]{2})?$"
        },

        "contents": {
          "description": "The kinds of data contained in this object.",
          "type": "array",
          "uniqueItems": true,
          "default": [ "localizedData", "nonLocalizedData" ],
          "items": {
            "enum": [
              "localizedData",
              "nonLocalizedData"
            ],
            "type": "string"
          }
        },

        "isComprehensive": {
          "description": "Specifies whether this object contains a complete definition of the localizable and/or non-localizable data for this component, as opposed to including only data that is relevant to the results persisted to this log file.",
          "type": "boolean",
          "default": false
        },

        "localizedDataSemanticVersion": {
          "description": "The semantic version of the localized strings defined in this component; maintained by components that provide translations.",
          "type": "string"
        },

        "minimumRequiredLocalizedDataSemanticVersion": {
          "description": "The minimum value of localizedDataSemanticVersion required in translations consumed by this component; used by components that consume translations.",
          "type": "string"
        },

        "associatedComponent": {
          "description": "The component which is strongly associated with this component. For a translation, this refers to the component which has been translated. For an extension, this is the driver that provides the extension's plugin model.",
          "$ref": "#/definitions/toolComponentReference"
        },

        "translationMetadata": {
          "description": "Translation metadata, required for a translation, not populated by other component types.",
          "$ref": "#/definitions/translationMetadata"
        },

        "supportedTaxonomies": {
          "description": "An array of toolComponentReference objects to declare the taxonomies supported by the tool component.",
          "type": "array",
          "minItems": 0,
          "uniqueItems": true,
          "default": [],
          "items": {
            "$ref": "#/definitions/toolComponentReference"
          }
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the tool component.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "name" ]
    },

    "toolComponentReference": {
      "description": "Identifies a particular toolComponent object, either the driver or an extension.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "name": {
          "description": "The 'name' property of the referenced toolComponent.",
          "type": "string"
        },

        "index": {
          "description": "An index into the referenced toolComponent in tool.extensions.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "guid": {
          "description": "The 'guid' property of the referenced toolComponent.",
          "type": "string",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the toolComponentReference.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "translationMetadata": {
      "description": "Provides additional metadata related to translation.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "name": {
          "description": "The name associated with the translation metadata.",
          "type": "string"
        },

        "fullName": {
          "description": "The full name associated with the translation metadata.",
          "type": "string"
        },

        "shortDescription": {
          "description": "A brief description of the translation metadata.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "fullDescription": {
          "description": "A comprehensive description of the translation metadata.",
          "$ref": "#/definitions/multiformatMessageString"
        },

        "downloadUri": {
          "description": "The absolute URI from which the translation metadata can be downloaded.",
          "type": "string",
          "format": "uri"
        },

        "informationUri": {
          "description": "The absolute URI from which information related to the translation metadata can be downloaded.",
          "type": "string",
          "format": "uri"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the translation metadata.",
          "$ref": "#/definitions/propertyBag"
        }
      },
      "required": [ "name" ]
    },

    "versionControlDetails": {
      "description": "Specifies the information necessary to retrieve a desired revision from a version control system.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "repositoryUri": {
          "description": "The absolute URI of the repository.",
          "type": "string",
          "format": "uri"
        },

        "revisionId": {
          "description": "A string that uniquely and permanently identifies the revision within the repository.",
          "type": "string"
        },

        "branch": {
          "description": "The name of a branch containing the revision.",
          "type": "string"
        },

        "revisionTag": {
          "description": "A tag that has been applied to the revision.",
          "type": "string"
        },

        "asOfTimeUtc": {
          "description": "A Coordinated Universal Time (UTC) date and time that can be used to synchronize an enlistment to the state of the repository at that time.",
          "type": "string",
          "format": "date-time"
        },

        "mappedTo": {
          "description": "The location in the local file system to which the root of the repository was mapped at the time of the analysis.",
          "$ref": "#/definitions/artifactLocation"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the version control details.",
          "$ref": "#/definitions/propertyBag"
        }
      },

      "required": [ "repositoryUri" ]
    },

    "webRequest": {
      "description": "Describes an HTTP request.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "index": {
          "description": "The index within the run.webRequests array of the request object associated with this result.",
          "type": "integer",
          "default": -1,
          "minimum": -1

        },

        "protocol": {
          "description": "The request protocol. Example: 'http'.",
          "type": "string"
        },

        "version": {
          "description": "The request version. Example: '1.1'.",
          "type": "string"
        },

        "target": {
          "description": "The target of the request.",
          "type": "string"
        },

        "method": {
          "description": "The HTTP method. Well-known values are 'GET', 'PUT', 'POST', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS', 'TRACE', 'CONNECT'.",
          "type": "string"
        },

        "headers": {
          "description": "The request headers.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "parameters": {
          "description": "The request parameters.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "body": {
          "description": "The body of the request.",
          "$ref": "#/definitions/artifactContent"
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the request.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    },

    "webResponse": {
      "description": "Describes the response to an HTTP request.",
      "type": "object",
      "additionalProperties": false,
      "properties": {

        "index": {
          "description": "The index within the run.webResponses array of the response object associated with this result.",
          "type": "integer",
          "default": -1,
          "minimum": -1
        },

        "protocol": {
          "description": "The response protocol. Example: 'http'.",
          "type": "string"
        },

        "version": {
          "description": "The response version. Example: '1.1'.",
          "type": "string"
        },

        "statusCode": {
          "description": "The response status code. Example: 451.",
          "type": "integer"
        },

        "reasonPhrase": {
          "description": "The response reason. Example: 'Not found'.",
          "type": "string"
        },

        "headers": {
          "description": "The response headers.",
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        },

        "body": {
          "description": "The body of the response.",
          "$ref": "#/definitions/artifactContent"
        },

        "noResponseReceived": {
          "description": "Specifies whether a response was received from the server.",
          "type": "boolean",
          "default": false
        },

        "properties": {
          "description": "Key/value pairs that provide additional information about the response.",
          "$ref": "#/definitions/propertyBag"
        }
      }
    }
  }
}
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test cli && cargo test --release --locked --test audit
```

Expected: FAIL in `cli`: `audit_without_a_hand_off_file_prints_one_sarif_report_and_exits_zero`
fails, since the binary refuses `audit`, and `refuses_anything_else_with_one_line_on_standard_error`
and `refuses_no_argument_at_all_the_same_way` fail on the list of what it answers. The `audit` target
does not run (decision 17).

- [ ] **Step 3: Write the code**

The reading moves out of `graph` into a module of its own:

```rust path=src/reading.rs
//! Reading a project: its hand-off file, its files, its packages, its scripts' imports and where they
//! lead, its entry points, and which export of which file is used. `graph` prints a reading, and
//! `audit` reports what one shows (spec OUT-1, CMD-1, CMD-4).

use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off::{self, HandOff};
use crate::marks::{self, Marks};
use crate::notice::Notice;
use crate::packages::{self, Packages};
use crate::resolve::{Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;
use crate::uses::{self, Uses};

/// What reading a project found.
pub struct Reading {
    /// The hand-off file, where one was given.
    pub handed: Option<HandOff>,
    /// The files of the list, in its order.
    pub files: Vec<File>,
    /// Which files the declarations mark as data or as generated.
    pub marks: Marks,
    /// The packages, and which package each file belongs to.
    pub packages: Packages,
    /// What each file that was read holds.
    pub scripts: Scripts,
    /// Where each import of each file leads, by the file's path, in the order of its imports.
    pub resolved: BTreeMap<String, Vec<Resolution>>,
    /// The entry points, each with every source that made it one.
    pub entries: Entries,
    /// Every use of an export, and the files the entry points reach.
    pub uses: Uses,
    /// What a person should know about the run, in the order it was found.
    pub notices: Vec<Notice>,
}

/// Reads the project, with the hand-off file at the path given or with none.
pub fn read(hand_off_path: Option<&Path>) -> Result<Reading, Stop> {
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

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match &handed {
        Some(handed) => handed.root.clone(),
        None => env::current_dir().map_err(|error| {
            Stop::new(format!(
                "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
            ))
        })?,
    };
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
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            root: &root,
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed: handed.as_ref(),
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);

    Ok(Reading {
        handed,
        files: listing.files,
        marks,
        packages,
        scripts,
        resolved,
        entries,
        uses,
        notices,
    })
}
```

```rust edit path=src/graph.rs
<<<<<<< find
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports and the names of every script, where each import leads,
//! the entry points with the source of each, every use of an export, and the files the entry points
//! reach (spec CMD-4).

use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::{EVERY, Export, Import, Name};
use crate::marks::{self, Marks};
use crate::notice::{self, Notice};
use crate::packages::{self, Package, Packages};
use crate::resolve::{Outcome, Place, Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;
use crate::uses::{self, Origin, Reach, Use, Uses};

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

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match &handed {
        Some(handed) => handed.root.clone(),
        None => env::current_dir().map_err(|error| {
            Stop::new(format!(
                "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
            ))
        })?,
    };
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
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            root: &root,
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed: handed.as_ref(),
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
=======
//! The `graph` command: prints what reading a project found as one JSON document (spec OUT-1).
//!
//! So far that is the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports and the names of every script, where each import leads,
//! the entry points with the source of each, every use of an export, and the files the entry points
//! reach (spec CMD-4).

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::entries::{self, Entries};
use crate::files::File;
use crate::imports::{EVERY, Export, Import, Name};
use crate::marks::Marks;
use crate::notice;
use crate::packages::{Package, Packages};
use crate::reading::Reading;
use crate::resolve::{Outcome, Place, Resolution};
use crate::scripts::Scripts;
use crate::uses::{Origin, Reach, Use, Uses};

/// The document `graph` prints for a reading.
pub fn printed(reading: Reading) -> String {
    let Reading {
        handed,
        files,
        marks,
        packages,
        scripts,
        resolved,
        entries,
        uses,
        notices,
    } = reading;
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
>>>>>>> replace
<<<<<<< find
    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// The entry points as they are printed, by path, each with every source that made it one (spec
=======
    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    format!("{document:#}")
}

/// The entry points as they are printed, by path, each with every source that made it one (spec
>>>>>>> replace
```

The rules, and the report that lists them:

```rust path=src/findings.rs
//! What `audit` reports: the rules a finding can be of (spec REPORT-2).

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// A script or a Vue component that no entry point reaches.
    UnusedFile,
    /// A value a reachable file exports that no other file uses.
    UnusedExport,
    /// A type a reachable file exports that no other file uses.
    UnusedType,
    /// An import that leads to no file.
    UnresolvedImport,
    /// Files that import each other in a loop.
    CircularDependency,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 5] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
    ];

    /// The rule's name in the report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedFile => "unused-file",
            Self::UnusedExport => "unused-export",
            Self::UnusedType => "unused-type",
            Self::UnresolvedImport => "unresolved-import",
            Self::CircularDependency => "circular-dependency",
        }
    }

    /// What the rule finds, as one sentence.
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnusedFile => "A script or a Vue component that no entry point reaches.",
            Self::UnusedExport => "A value a file exports that no other file uses.",
            Self::UnusedType => "A type a file exports that no other file uses.",
            Self::UnresolvedImport => "An import that leads to no file.",
            Self::CircularDependency => "Files that import each other in a loop.",
        }
    }
}
```

```rust path=src/audit.rs
//! The `audit` command: prints what a reading shows as one SARIF 2.1.0 document with one run (spec
//! CMD-1, REPORT-1).
//!
//! So far the run names the tool and lists the five rules and the notices; it holds no finding yet.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::findings::Rule;
use crate::notice::{self, Notice};
use crate::reading::Reading;

/// Where the schema of SARIF 2.1.0 is published.
const SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

/// The version of SARIF the report is written in.
const SARIF_VERSION: &str = "2.1.0";

/// The name SARIF gives the folder every path of the report is relative to: the root.
const ROOT: &str = "%SRCROOT%";

/// The characters of a path that a URI holds as they are, besides letters and digits: the unreserved
/// ones, the sub-delimiters, `@`, and the `/` between segments (RFC 3986).
const PLAIN: &str = "-._~!$&'()*+,;=@/";

/// The report `audit` prints for a reading.
pub fn printed(reading: Reading) -> String {
    let mut document = json!({
        "$schema": SCHEMA,
        "version": SARIF_VERSION,
        "runs": [{
            "columnKind": "utf16CodeUnits",
            "invocations": [{
                "executionSuccessful": true,
                "toolExecutionNotifications": notice::sorted(reading.notices)
                    .iter()
                    .map(notification)
                    .collect::<Vec<Value>>(),
            }],
            "properties": { "interfaceVersion": INTERFACE_VERSION },
            "results": [],
            "tool": {
                "driver": {
                    "informationUri": env!("CARGO_PKG_REPOSITORY"),
                    "name": env!("CARGO_PKG_NAME"),
                    "rules": Rule::ALL.iter().map(|rule| described(*rule)).collect::<Vec<Value>>(),
                    "version": env!("CARGO_PKG_VERSION"),
                },
            },
        }],
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    format!("{document:#}")
}

/// One rule as the run lists it.
fn described(rule: Rule) -> Value {
    json!({ "id": rule.name(), "shortDescription": { "text": rule.description() } })
}

/// One notice as a notification of the run's invocation: its kind, its sentence, and its file where
/// it has one (spec REPORT-4).
fn notification(notice: &Notice) -> Value {
    let mut notification = json!({
        "descriptor": { "id": notice.kind() },
        "level": "note",
        "message": { "text": notice.sentence() },
    });

    if let Some(file) = notice.file() {
        notification["locations"] =
            json!([{ "physicalLocation": { "artifactLocation": artifact(file) } }]);
    }

    notification
}

/// A file as SARIF names one: its path from the root, as a URI.
fn artifact(path: &str) -> Value {
    json!({ "uri": uri(path), "uriBaseId": ROOT })
}

/// A path as a relative URI: each byte a URI cannot hold as it is, is written as `%` and its two
/// hexadecimal digits (RFC 3986).
fn uri(path: &str) -> String {
    let mut uri = String::with_capacity(path.len());

    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || PLAIN.as_bytes().contains(&byte) {
            uri.push(char::from(byte));
        } else {
            // Writing into a String cannot fail.
            let _ = write!(uri, "%{byte:02X}");
        }
    }

    uri
}
```

A notice tells its parts to the report:

```rust edit path=src/notice.rs
<<<<<<< find
        }
    }

    /// The notice as it is printed.
    fn printed(&self) -> Value {
        json!({ "file": self.file, "kind": self.kind, "sentence": self.sentence })
    }
}

/// The notices as they are printed: those about no file first, then by file, by kind and by sentence
/// (spec RUN-8).
pub fn printed(mut notices: Vec<Notice>) -> Vec<Value> {
    notices.sort_by(|a, b| (&a.file, a.kind, &a.sentence).cmp(&(&b.file, b.kind, &b.sentence)));

    notices.iter().map(Notice::printed).collect()
}
=======
        }
    }

    /// The kind of notice.
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// The file it is about, where there is one.
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// What happened, as a sentence.
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// The notice as `graph` prints it.
    fn printed(&self) -> Value {
        json!({ "file": self.file, "kind": self.kind, "sentence": self.sentence })
    }
}

/// The notices in the order they are printed in: those about no file first, then by file, by kind and
/// by sentence (spec RUN-8).
pub fn sorted(mut notices: Vec<Notice>) -> Vec<Notice> {
    notices.sort_by(|a, b| (&a.file, a.kind, &a.sentence).cmp(&(&b.file, b.kind, &b.sentence)));

    notices
}

/// The notices as `graph` prints them, in their order.
pub fn printed(notices: Vec<Notice>) -> Vec<Value> {
    sorted(notices).iter().map(Notice::printed).collect()
}
>>>>>>> replace
```

The command line, and the new test target:

```rust edit path=src/cli.rs
<<<<<<< find
/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// The option of `graph` that names the hand-off file.
const HAND_OFF_OPTION: &str = "--hand-off";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str = "--version, interface, and graph [--hand-off <file>]";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
=======
/// The command that reads a project and prints what it read.
const GRAPH_COMMAND: &str = "graph";

/// The command that reads a project and prints a report of what it found (spec CMD-1).
const AUDIT_COMMAND: &str = "audit";

/// The option of `graph` and of `audit` that names the hand-off file.
const HAND_OFF_OPTION: &str = "--hand-off";

/// What the binary accepts, for the line that refuses anything else.
const ACCEPTED: &str =
    "--version, interface, graph [--hand-off <file>], and audit [--hand-off <file>]";

/// One thing the binary can be asked to do.
#[derive(Debug, PartialEq, Eq)]
>>>>>>> replace
<<<<<<< find
    Interface,
    /// Read the project and print what was read, with a hand-off file or without one.
    Graph { hand_off: Option<PathBuf> },
}

/// Reads the arguments after the program's name. The error is the text of the refusal.
=======
    Interface,
    /// Read the project and print what was read, with a hand-off file or without one.
    Graph { hand_off: Option<PathBuf> },
    /// Read the project and print a report of what was found, with a hand-off file or without one.
    Audit { hand_off: Option<PathBuf> },
}

/// Reads the arguments after the program's name. The error is the text of the refusal.
>>>>>>> replace
<<<<<<< find
        [GRAPH_COMMAND, HAND_OFF_OPTION, _] => Ok(Command::Graph {
            hand_off: Some(PathBuf::from(&args[2])),
        }),
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => {
            let given = args
=======
        [GRAPH_COMMAND, HAND_OFF_OPTION, _] => Ok(Command::Graph {
            hand_off: Some(PathBuf::from(&args[2])),
        }),
        [AUDIT_COMMAND] => Ok(Command::Audit { hand_off: None }),
        [AUDIT_COMMAND, HAND_OFF_OPTION, _] => Ok(Command::Audit {
            hand_off: Some(PathBuf::from(&args[2])),
        }),
        [] => Err(format!("nothing to do; it answers {ACCEPTED}")),
        _ => {
            let given = args
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: so far its hand-off file, its
//! files, its packages, its scripts' imports and where they lead, its entry points, and which export of
//! which file is used. It reports no findings yet.

mod cli;
mod declarations;
mod entries;
mod files;
mod git;
mod glob;
mod graph;
=======
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, and `audit` prints a report of it, which holds no finding
//! yet.

mod audit;
mod cli;
mod declarations;
mod entries;
mod files;
mod findings;
mod git;
mod glob;
mod graph;
>>>>>>> replace
<<<<<<< find
mod notice;
mod packages;
mod pages;
mod resolve;
mod scripts;
mod stop;
=======
mod notice;
mod packages;
mod pages;
mod reading;
mod resolve;
mod scripts;
mod stop;
>>>>>>> replace
<<<<<<< find
use std::process::ExitCode;

use cli::Command;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;
=======
use std::process::ExitCode;

use cli::Command;
use stop::Stop;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;
>>>>>>> replace
<<<<<<< find

            ExitCode::SUCCESS
        }
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
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

=======

            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => {
            ended(reading::read(hand_off.as_deref()).map(graph::printed))
        }
        Ok(Command::Audit { hand_off }) => {
            ended(reading::read(hand_off.as_deref()).map(audit::printed))
        }
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

>>>>>>> replace
<<<<<<< find
        }
    }
}
=======
        }
    }
}

/// Ends a run that reads a project: the document it made, on standard output, or the one line of a
/// run that could not go on, on standard error (spec RUN-3, CMD-3).
fn ended(document: Result<String, Stop>) -> ExitCode {
    match document {
        Ok(document) => {
            println!("{document}");

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

```toml edit path=Cargo.toml
<<<<<<< find
name = "entries"
path = "tests/entries.test.rs"

[dependencies]
# Parse scripts into syntax trees, walk them, and tie each reference to the binding it names
# (`oxc_semantic`, spec USE-1); released together, so pinned together (spec AN-2, DEP-3).
=======
name = "entries"
path = "tests/entries.test.rs"

[[test]]
name = "audit"
path = "tests/audit.test.rs"

[dependencies]
# Parse scripts into syntax trees, walk them, and tie each reference to the binding it names
# (`oxc_semantic`, spec USE-1); released together, so pinned together (spec AN-2, DEP-3).
>>>>>>> replace
<<<<<<< find
yaml-rust2 = { version = "=0.13.0", default-features = false }

[dev-dependencies]
# Checks the tests' hand-off files against the copy of yakshaver's schema. Never part of the binary.
jsonschema = { version = "=0.61.1", default-features = false }
=======
yaml-rust2 = { version = "=0.13.0", default-features = false }

[dev-dependencies]
# Checks the tests' hand-off files against the copy of yakshaver's schema, and the reports `audit`
# prints against the copy of SARIF's (spec FIX-3). Never part of the binary.
jsonschema = { version = "=0.61.1", default-features = false }
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test cli && cargo test --release --locked --test audit
```

Expected: PASS: 7 tests in `cli`, 3 in `audit`.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two, 52 of them in the node
tests.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add Cargo.toml src/reading.rs src/audit.rs src/findings.rs src/graph.rs src/cli.rs src/main.rs src/notice.rs tests/cli.test.rs tests/audit.test.rs tests/support/project.rs tests/sarif scripts/sarif-schema.test.mjs
git commit -m "feat(audit): add the audit command and its SARIF report"
```

Expected: one commit on `feat/dead-code-4-audit`.

### Task 2: unused files, exports and types

**Implements:** KIND-1, KIND-2, ENTRY-8, ENTRY-9, ENTRY-10, REPORT-1, REPORT-3, FIX-1, FIX-2, DEP-1

**Test:** `cargo test --release --locked --test audit`

**Files:**
- Modify: `src/findings.rs`, `src/audit.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`
- Test: `tests/audit.test.rs`

**Interfaces:**
- Consumes: task 1's `Reading` and `Rule`; plan 3's `Uses` (`list`, `reached`) and each file's
  `exports`; plan 1's `Entries::faults` and `Fault`; `Package::detected` and `Packages::of_file`;
  `scripts::reads`; `sha2::Sha256`.
- Produces: `findings::find`, `Finding` (`rule`, `message`, `location`, `related`, `fingerprint`),
  `Location` (`path`, `line`, `column`), `Rule::index`. Task 3 adds two kinds to `find`; plan 5
  compares fingerprints with the base's.

- [ ] **Step 1: Write the failing tests**

Seven tests join `tests/audit.test.rs`: every way of naming an entry point with only the unreached
file left, the unused exports and types, the kinds never reported, a Vue template and an MDX document,
the packages detection did not look in, a computed import, and the fingerprints:

```rust edit path=tests/audit.test.rs
<<<<<<< find
    assert!(line.contains("src/broken.ts"), "{line}");
    assert_eq!(line, stopped(&project.run(|_| {})));
}
=======
    assert!(line.contains("src/broken.ts"), "{line}");
    assert_eq!(line, stopped(&project.run(|_| {})));
}

/// One stack named `typescript` that holds the declarations given, the pack's and the project's.
fn stack(pack: Value, project: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": project }])
}

/// The findings of a run, in its order: each with its rule, its file as the report writes it, and its
/// line where it has one.
fn found(run: &Value) -> Vec<(&str, &str, Option<u64>)> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .map(|result| {
            let place = &result["locations"][0]["physicalLocation"];

            (
                result["ruleId"].as_str().expect("a rule"),
                place["artifactLocation"]["uri"].as_str().expect("a file"),
                place["region"]["startLine"].as_u64(),
            )
        })
        .collect()
}

/// The message of the finding of the rule given at the file given.
fn message<'r>(run: &'r Value, rule: &str, uri: &str) -> &'r str {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .find(|result| {
            result["ruleId"] == json!(rule)
                && result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    == json!(uri)
        })
        .and_then(|result| result["message"]["text"].as_str())
        .unwrap_or_else(|| panic!("a finding of {rule} at {uri}"))
}

/// The fingerprints of a run's findings, in its order.
fn fingerprints(run: &Value) -> Vec<&str> {
    run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .map(|result| {
            result["partialFingerprints"]["yakAnalyzer/v1"]
                .as_str()
                .expect("a fingerprint")
        })
        .collect()
}

#[test]
fn every_way_a_project_names_its_entry_points_leaves_only_the_file_nothing_reaches() {
    let project = Project::new("audit-entry-patterns");

    project
        .write(
            "package.json",
            r#"{
                "name": "app",
                "main": "./dist/index.js",
                "bin": { "app": "./bundle/app.js" },
                "exports": { ".": "./dist/index.js", "./features/*": "./dist/features/*.js" },
                "scripts": { "test": "node scripts/run-tests.mjs" }
            }"#,
        )
        .write(
            "tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "dist", "rootDir": "src" }, "files": ["src/global.ts"] }"#,
        )
        .write("src/index.ts", "export const start = 1;\n")
        .write("src/features/search.ts", "export const search = 1;\n")
        .write("src/global.ts", "export {};\n")
        .write("src/cli.ts", "export {};\n")
        .write("src/math.test.ts", "export {};\n")
        .write("scripts/run-tests.mjs", "export {};\n")
        .write("tools/program.mjs", "export {};\n")
        .write(
            "runner.config.ts",
            "import { helper } from './config-helper';\n\
             export default { include: ['e2e/**/*.e2e.ts'], setup: helper };\n",
        )
        .write("config-helper.ts", "export const helper = 1;\n")
        .write("e2e/login.e2e.ts", "export {};\n")
        .write("src/orphan.ts", "export const orphan = 1;\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(
            json!({
                "entries": ["dist/**/*.test.js"],
                "configEntries": [{
                    "file": "runner.config.ts",
                    "property": ["include"],
                    "value": "globs",
                    "relativeTo": "config",
                    "default": []
                }]
            }),
            json!({ "entries": ["src/cli.ts", "tools/program.mjs"] }),
        ),
    );
    let run = audit(&project, Some(&hand_off));

    // A `package.json`'s fields and scripts, a tsconfig's `files`, a glob of built tests mapped back, a
    // configuration file and the helper it imports, an `exports` pattern, a bundled `bin` settled by an
    // `entries` declaration, and a program a test starts by its path: each is an entry point, and
    // their exports are all used. Only the file nothing reaches is reported, and not its export (spec
    // KIND-1, KIND-2, FIX-1 patterns 1 to 4, 6, 7, 11 and 25).
    assert_eq!(found(&run), [("unused-file", "src/orphan.ts", None)]);
    // Its package has a `bin` target that maps to no source, so the message says what is likely missing
    // (spec ENTRY-8, FIX-1 pattern 3).
    assert_eq!(
        message(&run, "unused-file", "src/orphan.ts"),
        "No entry point reaches this file through any import, so it can be removed. Its package `.` has a \
         `bin` or `exports` target that maps to no source, so the likely fault is a missing `entries` \
         declaration."
    );
}

#[test]
fn reports_an_export_no_other_file_uses_as_a_value_or_a_type_once_where_it_is_declared() {
    let project = Project::new("audit-unused-exports");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import { used, type Shape } from './shapes';\n\
             import { spare } from './spare';\n\
             import * as tools from './tools';\n\
             import { relayed } from './relay';\n\
             const local = (shape: Shape) => used(shape) + tools.measure() + relayed;\n\
             local({ size: 1 });\n",
        )
        .write(
            "src/shapes.ts",
            "export interface Shape { size: number }\n\
             export type Unseen = string;\n\
             export const used = (shape: Shape): number => inner(shape);\n\
             export function inner(shape: Shape): number { return shape.size; }\n\
             export const nowhere = 1;\n",
        )
        .write("src/spare.ts", "export const spare = 1;\n")
        .write(
            "src/tools.ts",
            "export const measure = (): number => 1;\nexport const cut = (): number => 2;\n",
        )
        .write(
            "src/other.ts",
            "const nowhere = 2;\nexport const elsewhere = nowhere;\n",
        )
        .write("src/relay.ts", "export * from './deep';\n")
        .write(
            "src/deep.ts",
            "export const relayed = 1;\nexport const stranded = 2;\n",
        )
        .commit();

    let run = audit(&project, None);

    // An export used only inside its file and one used nowhere; an import whose binding is never
    // referenced uses nothing; a local of the same name in another file is not a use; a namespace uses
    // the member read off it; and a name passed on through `export *` is reported once, where it is
    // declared (spec KIND-2, USE-2, USE-2a, FIX-1 pattern 17, FIX-2).
    assert_eq!(
        found(&run),
        [
            ("unused-file", "src/other.ts", None),
            ("unused-export", "src/deep.ts", Some(2)),
            ("unused-export", "src/shapes.ts", Some(4)),
            ("unused-export", "src/shapes.ts", Some(5)),
            ("unused-export", "src/spare.ts", Some(1)),
            ("unused-export", "src/tools.ts", Some(2)),
            ("unused-type", "src/shapes.ts", Some(2)),
        ]
    );
    assert_eq!(
        run["results"][2]["message"]["text"],
        json!(
            "The export `inner` is used only inside its own file, so its `export` can be dropped."
        )
    );
    assert_eq!(
        run["results"][3]["message"]["text"],
        json!("The export `nowhere` is used nowhere, so it can be removed.")
    );
    assert_eq!(
        run["results"][6]["message"]["text"],
        json!("The type `Unseen` is used nowhere, so it can be removed.")
    );
    // An export's location is its name, the column counted from one.
    assert_eq!(
        run["results"][2]["locations"][0]["physicalLocation"]["region"],
        json!({ "startLine": 4, "startColumn": 17 })
    );
}

#[test]
fn reports_no_file_of_a_kind_that_is_never_an_unused_file() {
    let project = Project::new("audit-never-unused");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "export {};\n")
        .write("src/env.d.ts", "declare const flag: boolean;\n")
        .write("src/fixtures/broken.ts", "const = ;\n")
        .write("src/generated/api.ts", "export const api = 1;\n")
        .write("docs/guide.mdx", "# Guide\n")
        .write("public/index.html", "<p>page</p>\n")
        .write("notes.txt", "notes\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(
            json!({ "data": ["**/fixtures/**"] }),
            json!({ "generated": ["src/generated/**"] }),
        ),
    );

    // A type declaration file, a data file, a generated file, an MDX document, an HTML page and a file
    // of another kind are never reported unused (spec KIND-1).
    assert_eq!(found(&audit(&project, Some(&hand_off))), []);
}

#[test]
fn a_vue_template_and_an_mdx_document_keep_what_they_name_used() {
    let project = Project::new("audit-vue-mdx");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/main.ts" }"#,
        )
        .write(
            "src/main.ts",
            "import App from './App.vue';\nconsole.log(App);\n",
        )
        .write(
            "src/App.vue",
            "<template>\n  <Card />\n</template>\n\n\
             <script setup lang=\"ts\">\n\
             import Card from './Card';\n\
             import Unused from './Unused';\n\
             </script>\n",
        )
        .write("src/Card.ts", "export default {};\n")
        .write("src/Unused.ts", "export default {};\n")
        .write(
            "docs/guide.mdx",
            "import { Chart, Table } from '../src/widgets'\n\n# Guide\n\n<Chart />\n",
        )
        .write(
            "src/widgets.ts",
            "export const Chart = 1;\nexport const Table = 2;\n",
        )
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(json!({ "entries": ["docs/**/*.mdx"] }), json!({})),
    );

    // A component the template names is used, and one a `<script setup>` block only imports is not,
    // though the import still reaches its file; a document a declaration makes an entry point uses the
    // export its text names, and not the one it only imports (spec KIND-1, KIND-2, USE-4, USE-5, FIX-1
    // pattern 16).
    assert_eq!(
        found(&audit(&project, Some(&hand_off))),
        [
            ("unused-export", "src/Unused.ts", Some(1)),
            ("unused-export", "src/widgets.ts", Some(2)),
        ]
    );
}

#[test]
fn the_kinds_that_depend_on_entry_points_wait_for_a_detection_folder() {
    let project = Project::new("audit-not-detected");

    project
        .write(
            "packages/a/package.json",
            r#"{ "name": "a", "main": "./index.ts" }"#,
        )
        .write("packages/a/index.ts", "export {};\n")
        .write("packages/a/orphan.ts", "export const orphan = 1;\n")
        .write(
            "packages/b/package.json",
            r#"{ "name": "b", "main": "./index.ts" }"#,
        )
        .write("packages/b/index.ts", "import './missing';\n")
        .write("packages/b/orphan.ts", "export const orphan = 1;\n")
        .write("scripts/loose.ts", "export const loose = 1;\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &[".", "packages/a"],
        stack(json!({}), json!({})),
    );
    let run = audit(&project, Some(&hand_off));

    // Only the package whose folder yak's detection looked in has its unused files reported. The others
    // get a notice each, and so do the files in no package (spec ENTRY-9, FIX-2).
    assert_eq!(
        found(&run)
            .into_iter()
            .filter(|(rule, _, _)| rule.starts_with("unused"))
            .collect::<Vec<_>>(),
        [("unused-file", "packages/a/orphan.ts", None)]
    );
    assert_eq!(
        notifications(&run),
        [("kinds-not-reported", None), ("kinds-not-reported", None),]
    );

    let sentences: Vec<&str> = run["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .expect("a list of notifications")
        .iter()
        .map(|notification| {
            notification["message"]["text"]
                .as_str()
                .expect("a sentence")
        })
        .collect();

    assert_eq!(
        sentences,
        [
            "1 file(s) the analyzer reads are in no package, so their unused files, exports and types are \
             not reported.",
            "The package `packages/b` is not in a folder yak's detection looked in, so its unused files, \
             exports and types are not reported.",
        ]
    );
}

#[test]
fn a_computed_import_gives_a_notice_to_declare_what_it_loads() {
    let project = Project::new("audit-computed");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "const name = 'a';\nawait import(`./plugins/${name}`);\nrequire(name);\n",
        )
        .write("src/plugins/a.ts", "export {};\n")
        .commit();

    let run = audit(&project, None);

    // What a computed import loads looks unused until it is declared (spec ENTRY-10, FIX-1 pattern 19).
    assert_eq!(found(&run), [("unused-file", "src/plugins/a.ts", None)]);
    assert_eq!(
        notifications(&run),
        [
            ("no-hand-off-file", None),
            ("computed-import", Some("src/index.ts")),
        ]
    );
    assert_eq!(
        run["invocations"][0]["toolExecutionNotifications"][1]["message"]["text"],
        json!(
            "2 call(s) of `import()` or `require()` load a file the analyzer cannot work out, so what they \
             load must be declared in `entries`."
        )
    );
}

#[test]
fn a_fingerprint_names_a_finding_by_its_parts_and_never_by_its_line() {
    let project = Project::new("audit-fingerprints");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write("src/index.ts", "import './a';\n")
        .write(
            "src/a.ts",
            "export const first = 1;\nexport const second = 2;\n",
        )
        .write("src/b.ts", "export {};\n")
        .commit();

    let before = audit(&project, None);

    // Each fingerprint is 32 hexadecimal digits, and each finding has its own (spec REPORT-3).
    let printed = fingerprints(&before);

    assert_eq!(printed.len(), 3);
    assert!(printed.iter().all(|each| {
        each.len() == 32
            && each
                .chars()
                .all(|digit| digit.is_ascii_digit() || ('a'..='f').contains(&digit))
    }));
    assert_eq!(
        printed
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );

    // Moved down a line, the same findings keep their fingerprints.
    project
        .write(
            "src/a.ts",
            "\nexport const first = 1;\nexport const second = 2;\n",
        )
        .commit();

    let after = audit(&project, None);

    assert_eq!(found(&after)[1], ("unused-export", "src/a.ts", Some(2)));
    assert_eq!(fingerprints(&after), printed);
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test audit
```

Expected: FAIL: six of the seven new tests fail, since the report holds no result and no notice of
ENTRY-9 or ENTRY-10. `reports_no_file_of_a_kind_that_is_never_an_unused_file` passes already, as a
report with no result is what it expects, and so do task 1's three tests.

- [ ] **Step 3: Write the code**

`sha2` joins the dependencies:

```toml edit path=Cargo.toml
<<<<<<< find
json-strip-comments = "=3.1.2"
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"
# Matches the globs of declarations and of the workspace's list (spec KEY-5).
regex = "=1.13.1"
# Tells whether two paths are one file on disk, which Rust gives on Windows only in an unstable form.
=======
json-strip-comments = "=3.1.2"
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"
# Makes the fingerprint of each finding `audit` reports, a SHA-256 digest (dead-code spec REPORT-3,
# DEP-1).
sha2 = { version = "=0.11.1", default-features = false }
# Matches the globs of declarations and of the workspace's list (spec KEY-5).
regex = "=1.13.1"
# Tells whether two paths are one file on disk, which Rust gives on Windows only in an unstable form.
>>>>>>> replace
```

```toml edit path=Cargo.lock
<<<<<<< find
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"

[[package]]
name = "borrow-or-share"
version = "0.2.4"
=======
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"

[[package]]
name = "block-buffer"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2f6c7dbe95a6ed67ad9f18e57daf93a2f034c524b99fd2b76d18fdfeb6660aa"
dependencies = [
 "hybrid-array",
]

[[package]]
name = "borrow-or-share"
version = "0.2.4"
>>>>>>> replace
<<<<<<< find
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "417bef24afe1460300965a25ff4a24b8b45ad011948302ec221e8a0a81eb2c79"

[[package]]
name = "crossbeam-utils"
version = "0.8.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a31eee39dddec8330830986fcd7625edb5a24ec90ea038215273bbc3adb08ac6"

[[package]]
name = "dashmap"
version = "6.2.1"
=======
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "417bef24afe1460300965a25ff4a24b8b45ad011948302ec221e8a0a81eb2c79"

[[package]]
name = "cpufeatures"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5ca28b0ae3115b884660db4118d803791fd6756b6e88f39c0f3f7859060d7566"
dependencies = [
 "libc",
]

[[package]]
name = "crossbeam-utils"
version = "0.8.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a31eee39dddec8330830986fcd7625edb5a24ec90ea038215273bbc3adb08ac6"

[[package]]
name = "crypto-common"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ce6e4c961d6cd6c9a86db418387425e8bdeaf05b3c8bc1411e6dca4c252f1453"
dependencies = [
 "hybrid-array",
]

[[package]]
name = "dashmap"
version = "6.2.1"
>>>>>>> replace
<<<<<<< find
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "displaydoc"
version = "0.2.7"
=======
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "digest"
version = "0.11.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f1dd6dbb5841937940781866fa1281a1ff7bd3bf827091440879f9994983d5c2"
dependencies = [
 "block-buffer",
 "crypto-common",
]

[[package]]
name = "displaydoc"
version = "0.2.7"
>>>>>>> replace
<<<<<<< find
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "icu_collections"
version = "2.4.0"
=======
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "hybrid-array"
version = "0.4.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "27f864f10dfb56725ce5ce5472bc52252c8f93a4ab86327122cebf62c5f59a17"
dependencies = [
 "typenum",
]

[[package]]
name = "icu_collections"
version = "2.4.0"
>>>>>>> replace
<<<<<<< find
 "zmij",
]

[[package]]
name = "simd-json"
version = "0.18.1"
=======
 "zmij",
]

[[package]]
name = "sha2"
version = "0.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47d7069beb7d6ac7b9acd1039986e73443f24234f41074da099d6f994ac9ad19"
dependencies = [
 "cfg-if",
 "cpufeatures",
 "digest",
]

[[package]]
name = "simd-json"
version = "0.18.1"
>>>>>>> replace
<<<<<<< find
 "once_cell",
]

[[package]]
name = "unicode-general-category"
version = "1.1.0"
=======
 "once_cell",
]

[[package]]
name = "typenum"
version = "1.20.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20"

[[package]]
name = "unicode-general-category"
version = "1.1.0"
>>>>>>> replace
<<<<<<< find
 "regex",
 "same-file",
 "serde_json",
 "yaml-rust2",
]

=======
 "regex",
 "same-file",
 "serde_json",
 "sha2",
 "yaml-rust2",
]

>>>>>>> replace
```

The findings of the three kinds that depend on entry points, their notices, and the fingerprint:

```rust path=src/findings.rs
//! What `audit` reports: the findings a reading shows, each with its rule, its message, its location
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::notice::Notice;
use crate::reading::Reading;
use crate::scripts;

/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
const FINGERPRINT_DIGITS: usize = 32;

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// A script or a Vue component that no entry point reaches.
    UnusedFile,
    /// A value a reachable file exports that no other file uses.
    UnusedExport,
    /// A type a reachable file exports that no other file uses.
    UnusedType,
    /// An import that leads to no file.
    UnresolvedImport,
    /// Files that import each other in a loop.
    CircularDependency,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 5] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
    ];

    /// The rule's name in the report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedFile => "unused-file",
            Self::UnusedExport => "unused-export",
            Self::UnusedType => "unused-type",
            Self::UnresolvedImport => "unresolved-import",
            Self::CircularDependency => "circular-dependency",
        }
    }

    /// What the rule finds, as one sentence.
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnusedFile => "A script or a Vue component that no entry point reaches.",
            Self::UnusedExport => "A value a file exports that no other file uses.",
            Self::UnusedType => "A type a file exports that no other file uses.",
            Self::UnresolvedImport => "An import that leads to no file.",
            Self::CircularDependency => "Files that import each other in a loop.",
        }
    }

    /// The rule's place in the report's list.
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|rule| *rule == self)
            .unwrap_or_default()
    }
}

/// Where a finding is: a file, and a line and a column in it where the kind has them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    /// The file, by its path from the root.
    pub path: String,
    /// The line, counted from one.
    pub line: Option<usize>,
    /// The column, counted from one in UTF-16 code units.
    pub column: Option<usize>,
}

/// One finding.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    /// Its rule.
    pub rule: Rule,
    /// What a person can act on, as sentences.
    pub message: String,
    /// Where it is.
    pub location: Location,
    /// The other places it involves.
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
}

impl Finding {
    /// A finding, its fingerprint made from its rule and the parts given.
    fn new(rule: Rule, message: String, location: Location, parts: &[&str]) -> Self {
        Self {
            rule,
            message,
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
        }
    }
}

/// The fingerprint of a finding: the first digits of the SHA-256 digest of its rule and its parts, each
/// written after its length in bytes and a colon, so that no two lists of parts give the same input
/// (spec REPORT-3).
fn fingerprint(rule: Rule, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for part in std::iter::once(rule.name()).chain(parts.iter().copied()) {
        hasher.update(format!("{}:", part.len()).as_bytes());
        hasher.update(part.as_bytes());
    }

    let mut digits = String::with_capacity(FINGERPRINT_DIGITS);

    for byte in hasher.finalize().iter().take(FINGERPRINT_DIGITS / 2) {
        // Writing into a String cannot fail.
        let _ = write!(digits, "{byte:02x}");
    }

    digits
}

/// The findings of a reading, in the report's order, and the notices that come with them.
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = Vec::new();
    let mut notices = Vec::new();

    unused(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

    found.sort_by(|a, b| {
        (a.rule, &a.location, &a.fingerprint).cmp(&(b.rule, &b.location, &b.fingerprint))
    });

    (found, notices)
}

/// Whether the three kinds that depend on entry points are reported for a file: its package's own
/// folder is a detection folder (spec ENTRY-9).
fn reported(reading: &Reading, path: &str) -> bool {
    reading.packages.of_file.get(path).is_some_and(|folder| {
        reading
            .packages
            .list
            .iter()
            .any(|package| package.folder == *folder && package.detected)
    })
}

/// Whether a file is the project's own code to report on: read, and neither data nor generated.
fn own(reading: &Reading, file: &File) -> bool {
    !reading.marks.data.contains(&file.path)
        && !reading.marks.generated.contains(&file.path)
        && !reading.scripts.gone.contains(&file.path)
}

/// The unused files, exports and types (spec KIND-1, KIND-2).
fn unused(reading: &Reading, found: &mut Vec<Finding>) {
    let used: BTreeSet<(&str, Option<&str>)> = reading
        .uses
        .list
        .iter()
        .map(|used| (used.file.as_str(), used.name.as_deref()))
        .collect();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || !own(reading, file)
            || !reported(reading, &file.path)
        {
            continue;
        }

        let path = file.path.as_str();

        if !reading.uses.reached.contains_key(path) {
            found.push(Finding::new(
                Rule::UnusedFile,
                unused_file(reading, path),
                Location {
                    path: path.to_owned(),
                    line: None,
                    column: None,
                },
                &[path],
            ));

            // The unused-file finding covers its exports.
            continue;
        }

        let Some(read) = reading.scripts.read.get(path) else {
            continue;
        };

        for export in &read.exports {
            let name = export.name.as_str();

            if used.contains(&(path, None)) || used.contains(&(path, Some(name))) {
                continue;
            }

            let (rule, what) = if export.is_type {
                (Rule::UnusedType, "type")
            } else {
                (Rule::UnusedExport, "export")
            };
            let message = if export.used_in_file {
                format!(
                    "The {what} `{name}` is used only inside its own file, so its `export` can be dropped."
                )
            } else {
                format!("The {what} `{name}` is used nowhere, so it can be removed.")
            };

            found.push(Finding::new(
                rule,
                message,
                Location {
                    path: path.to_owned(),
                    line: Some(export.line),
                    column: Some(export.column),
                },
                &[path, name],
            ));
        }
    }
}

/// The message of an unused file: that it can go, and, in a package whose entry points are likely
/// incomplete, that the likely fault is a missing `entries` declaration (spec ENTRY-8).
fn unused_file(reading: &Reading, path: &str) -> String {
    let mut message =
        "No entry point reaches this file through any import, so it can be removed.".to_owned();
    let fault = reading.packages.of_file.get(path).and_then(|folder| {
        reading
            .entries
            .faults
            .get(folder)
            .map(|fault| (folder, fault))
    });

    if let Some((folder, fault)) = fault {
        let why = match fault {
            Fault::NoEntryPoint => "has no entry point at all",
            Fault::UnmappedTarget => "has a `bin` or `exports` target that maps to no source",
        };

        let _ = write!(
            message,
            " Its package `{folder}` {why}, so the likely fault is a missing `entries` declaration."
        );
    }

    message
}

/// One notice for each package that holds files the analyzer reads and whose folder is not a detection
/// folder, and one for the files in no package: the three kinds that depend on entry points are not
/// reported there (spec ENTRY-9).
fn not_reported(reading: &Reading, notices: &mut Vec<Notice>) {
    let analyzed = || {
        reading
            .files
            .iter()
            .filter(|file| scripts::reads(file.kind) && own(reading, file))
    };

    for package in reading
        .packages
        .list
        .iter()
        .filter(|package| !package.detected)
    {
        let holds = analyzed()
            .any(|file| reading.packages.of_file.get(&file.path) == Some(&package.folder));

        if holds {
            notices.push(Notice::new(
                "kinds-not-reported",
                None,
                format!(
                    "The package `{}` is not in a folder yak's detection looked in, so its unused files, \
                     exports and types are not reported.",
                    package.folder
                ),
            ));
        }
    }

    let outside = analyzed()
        .filter(|file| !reading.packages.of_file.contains_key(&file.path))
        .count();

    if outside > 0 {
        notices.push(Notice::new(
            "kinds-not-reported",
            None,
            format!(
                "{outside} file(s) the analyzer reads are in no package, so their unused files, exports and \
                 types are not reported."
            ),
        ));
    }
}

/// One notice for each file with an `import()` or `require()` it cannot follow: what it loads must be
/// declared (spec ENTRY-10).
fn computed(reading: &Reading, notices: &mut Vec<Notice>) {
    for (path, read) in &reading.scripts.read {
        if read.computed > 0 {
            notices.push(Notice::new(
                "computed-import",
                Some(path.clone()),
                format!(
                    "{} call(s) of `import()` or `require()` load a file the analyzer cannot work out, so \
                     what they load must be declared in `entries`.",
                    read.computed
                ),
            ));
        }
    }
}
```

The report writes each finding as a result:

```rust edit path=src/audit.rs
<<<<<<< find
//! The `audit` command: prints what a reading shows as one SARIF 2.1.0 document with one run (spec
//! CMD-1, REPORT-1).
//!
//! So far the run names the tool and lists the five rules and the notices; it holds no finding yet.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::findings::Rule;
use crate::notice::{self, Notice};
use crate::reading::Reading;

/// Where the schema of SARIF 2.1.0 is published.
const SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
=======
//! The `audit` command: prints what a reading shows as one SARIF 2.1.0 document with one run (spec
//! CMD-1, REPORT-1).
//!
//! The run names the tool and lists the five rules, the findings and the notices.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::findings::{self, Finding, Location, Rule};
use crate::notice::{self, Notice};
use crate::reading::Reading;

/// The key of `partialFingerprints` that holds a finding's fingerprint, with the scheme's version in
/// it, so that a later scheme can sit beside it (spec REPORT-3).
const FINGERPRINT_KEY: &str = "yakAnalyzer/v1";

/// Where the schema of SARIF 2.1.0 is published.
const SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
>>>>>>> replace
<<<<<<< find
const PLAIN: &str = "-._~!$&'()*+,;=@/";

/// The report `audit` prints for a reading.
pub fn printed(reading: Reading) -> String {
    let mut document = json!({
        "$schema": SCHEMA,
        "version": SARIF_VERSION,
=======
const PLAIN: &str = "-._~!$&'()*+,;=@/";

/// The report `audit` prints for a reading.
pub fn printed(mut reading: Reading) -> String {
    let (found, notices) = findings::find(&reading);

    reading.notices.extend(notices);

    let mut document = json!({
        "$schema": SCHEMA,
        "version": SARIF_VERSION,
>>>>>>> replace
<<<<<<< find
                    .collect::<Vec<Value>>(),
            }],
            "properties": { "interfaceVersion": INTERFACE_VERSION },
            "results": [],
            "tool": {
                "driver": {
                    "informationUri": env!("CARGO_PKG_REPOSITORY"),
=======
                    .collect::<Vec<Value>>(),
            }],
            "properties": { "interfaceVersion": INTERFACE_VERSION },
            "results": found.iter().map(result).collect::<Vec<Value>>(),
            "tool": {
                "driver": {
                    "informationUri": env!("CARGO_PKG_REPOSITORY"),
>>>>>>> replace
<<<<<<< find
    json!({ "id": rule.name(), "shortDescription": { "text": rule.description() } })
}

/// One notice as a notification of the run's invocation: its kind, its sentence, and its file where
/// it has one (spec REPORT-4).
fn notification(notice: &Notice) -> Value {
=======
    json!({ "id": rule.name(), "shortDescription": { "text": rule.description() } })
}

/// One finding as a result of the run: its rule, its message, its location, the other places it
/// involves, and its fingerprint (spec REPORT-1).
fn result(finding: &Finding) -> Value {
    let mut result = json!({
        "level": "warning",
        "locations": [placed(&finding.location)],
        "message": { "text": finding.message },
        "partialFingerprints": { FINGERPRINT_KEY: finding.fingerprint },
        "ruleId": finding.rule.name(),
        "ruleIndex": finding.rule.index(),
    });

    if !finding.related.is_empty() {
        result["relatedLocations"] = finding
            .related
            .iter()
            .enumerate()
            .map(|(id, location)| {
                let mut related = placed(location);

                related["id"] = json!(id);
                related
            })
            .collect();
    }

    result
}

/// A location as SARIF writes one: the file, and the line and column where there are some.
fn placed(location: &Location) -> Value {
    let mut physical = json!({ "artifactLocation": artifact(&location.path) });

    if let Some(line) = location.line {
        physical["region"] = json!({ "startLine": line });

        if let Some(column) = location.column {
            physical["region"]["startColumn"] = json!(column);
        }
    }

    json!({ "physicalLocation": physical })
}

/// One notice as a notification of the run's invocation: its kind, its sentence, and its file where
/// it has one (spec REPORT-4).
fn notification(notice: &Notice) -> Value {
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, and `audit` prints a report of it, which holds no finding
//! yet.

mod audit;
mod cli;
=======
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, and `audit` prints a report of the findings it shows.

mod audit;
mod cli;
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test audit
```

Expected: PASS: 10 tests.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add Cargo.toml Cargo.lock src/findings.rs src/audit.rs src/main.rs tests/audit.test.rs
git commit -m "feat(audit): report unused files, exports and types"
```

Expected: one commit on `feat/dead-code-4-audit`.

### Task 3: unresolved imports and circular dependencies

**Implements:** KIND-3, KIND-4, REPORT-3, FIX-1

**Test:** `cargo test --release --locked --test audit`

**Files:**
- Modify: `src/findings.rs`, `src/imports.rs`
- Test: `tests/audit.test.rs`

**Interfaces:**
- Consumes: task 2's `find`, `Finding` and `Location`; `resolve::Outcome`; `imports::Form` and
  `Import` with `type_only` and `has_query`.
- Produces: the unresolved-import and circular-dependency findings; `Import::type_only` true for an
  `import` or `export … from` all of whose names are marked `type`. Plan 5 compares a cycle's edges with
  the base's (INH-3) from the same edges.

- [ ] **Step 1: Write the failing tests**

Three tests join `tests/audit.test.rs`: the unresolved imports, the cycles and what is not one, and
the message of a long loop:

```rust edit path=tests/audit.test.rs
<<<<<<< find
    assert_eq!(found(&after)[1], ("unused-export", "src/a.ts", Some(2)));
    assert_eq!(fingerprints(&after), printed);
}
=======
    assert_eq!(found(&after)[1], ("unused-export", "src/a.ts", Some(2)));
    assert_eq!(fingerprints(&after), printed);
}

#[test]
fn reports_each_import_that_leads_to_no_file_in_any_package() {
    let project = Project::new("audit-unresolved");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts", "dependencies": { "left-pad": "1.0.0" } }"#,
        )
        .write(
            "src/index.ts",
            "import './missing';\nimport 'left-pad';\nimport 'node:fs';\nimport './missing';\n\
             const name = 'x';\nawait import(name);\n",
        )
        .write("src/env.d.ts", "import './nowhere';\n")
        .write("src/generated/api.ts", "import './nowhere';\n")
        .write("tools/package.json", r#"{ "name": "tools" }"#)
        .write("tools/run.ts", "require('./gone');\n")
        .commit();

    let hand_off = project.hand_off(
        &project.root,
        &["."],
        stack(json!({}), json!({ "generated": ["src/generated/**"] })),
    );
    let run = audit(&project, Some(&hand_off));

    // Each import that leads nowhere is one finding at its line, in a package detection looked in or
    // not; one of a package, a built-in module or a computed specifier is none, and nor is one in a type
    // declaration file or a generated file. Two of one specifier in one file share a fingerprint (spec
    // KIND-3, REPORT-3).
    let unresolved: Vec<_> = found(&run)
        .into_iter()
        .filter(|(rule, _, _)| *rule == "unresolved-import")
        .collect();

    assert_eq!(
        unresolved,
        [
            ("unresolved-import", "src/index.ts", Some(1)),
            ("unresolved-import", "src/index.ts", Some(4)),
            ("unresolved-import", "tools/run.ts", Some(1)),
        ]
    );
    assert_eq!(
        message(&run, "unresolved-import", "tools/run.ts"),
        "The import of `./gone` leads to no file, no package and no built-in module."
    );

    let printed = fingerprints(&run);
    let first = found(&run)
        .iter()
        .position(|(rule, _, _)| *rule == "unresolved-import")
        .expect("an unresolved import");

    assert_eq!(printed[first], printed[first + 1]);
    assert_ne!(printed[first], printed[first + 2]);
}

#[test]
fn reports_each_set_of_files_that_load_each_other_once_over_static_imports_alone() {
    let project = Project::new("audit-cycles");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "import './a';\nimport './lazy';\nimport './typed';\nimport './self';\nimport './one';\n",
        )
        // A static cycle of three, with a shorter way back from `b`.
        .write(
            "src/a.ts",
            "export const a = 1;\nimport { b } from './b';\nconsole.log(b);\n",
        )
        .write(
            "src/b.ts",
            "import './c';\nexport { a } from './a';\nexport const b = 2;\n",
        )
        .write(
            "src/c.ts",
            "const loaded = require('./b');\nconsole.log(loaded);\n",
        )
        // A loop closed only by `import()`, or only by imports of types, loads nothing in a loop.
        .write("src/lazy.ts", "import './lazy-back';\n")
        .write("src/lazy-back.ts", "await import('./lazy');\n")
        .write("src/typed.ts", "import './typed-back';\nexport type T = 1;\n")
        .write(
            "src/typed-back.ts",
            "import type { T } from './typed';\nimport { type T as U } from './typed';\n\
             export type { T } from './typed';\nexport type V = [T, U];\n",
        )
        // A file that imports itself.
        .write("src/self.ts", "import './self';\n")
        // A loop through a type declaration file is none.
        .write("src/one.ts", "import './two';\n")
        .write("src/two.d.ts", "import './one';\n")
        .commit();

    let run = audit(&project, None);
    let cycles: Vec<&Value> = run["results"]
        .as_array()
        .expect("a list of results")
        .iter()
        .filter(|result| result["ruleId"] == json!("circular-dependency"))
        .collect();
    let related = |result: &Value| -> Vec<(String, u64)> {
        result["relatedLocations"]
            .as_array()
            .map(|locations| {
                locations
                    .iter()
                    .map(|location| {
                        let place = &location["physicalLocation"];

                        (
                            place["artifactLocation"]["uri"]
                                .as_str()
                                .expect("a file")
                                .to_owned(),
                            place["region"]["startLine"].as_u64().expect("a line"),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    // One finding for each set, at the first import of its first file by path that loads another file
    // of the set; the set's other files are related, each at its first such import. The message names
    // the shortest loop through the first file (spec KIND-4, FIX-1 pattern 18).
    assert_eq!(
        found(&run)
            .into_iter()
            .filter(|(rule, _, _)| *rule == "circular-dependency")
            .collect::<Vec<_>>(),
        [
            ("circular-dependency", "src/a.ts", Some(2)),
            ("circular-dependency", "src/self.ts", Some(1)),
        ]
    );
    assert_eq!(
        related(cycles[0]),
        [("src/b.ts".to_owned(), 1), ("src/c.ts".to_owned(), 1)]
    );
    assert_eq!(
        cycles[0]["message"]["text"],
        json!(
            "These files import each other in a loop: `src/a.ts` → `src/b.ts` → `src/a.ts`. The loop is part \
             of a set of 3 files that each reach all the others."
        )
    );
    assert_eq!(related(cycles[1]), []);
    assert_eq!(
        cycles[1]["message"]["text"],
        json!("These files import each other in a loop: `src/self.ts` → `src/self.ts`.")
    );

    // An import all of whose names are marked `type` brings in types alone, as `graph` prints it.
    let document = project.graph();
    let typed_back = document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!("src/typed-back.ts"))
        .expect("the file is listed");

    assert_eq!(
        typed_back["imports"]
            .as_array()
            .expect("a list of imports")
            .iter()
            .map(|import| import["typeOnly"].as_bool().expect("a flag"))
            .collect::<Vec<bool>>(),
        [true, true, true]
    );
}

#[test]
fn names_ten_files_of_a_long_loop_and_how_many_more() {
    let project = Project::new("audit-long-cycle");

    project.write(
        "package.json",
        r#"{ "name": "app", "main": "./src/m00.ts" }"#,
    );

    for link in 0..12 {
        project.write(
            &format!("src/m{link:02}.ts"),
            &format!("import './m{:02}';\n", (link + 1) % 12),
        );
    }

    project.commit();

    let run = audit(&project, None);

    // A loop of twelve files names the first ten, then how many more, then the first again (spec
    // KIND-4).
    assert_eq!(
        run["results"][0]["message"]["text"],
        json!(
            "These files import each other in a loop: `src/m00.ts` → `src/m01.ts` → `src/m02.ts` → \
             `src/m03.ts` → `src/m04.ts` → `src/m05.ts` → `src/m06.ts` → `src/m07.ts` → `src/m08.ts` → \
             `src/m09.ts` → 2 more → `src/m00.ts`."
        )
    );
    assert_eq!(
        run["results"][0]["relatedLocations"]
            .as_array()
            .map(Vec::len),
        Some(11)
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test audit
```

Expected: FAIL: the three new tests fail, since the report holds no unresolved import and no
cycle. The other ten pass.

- [ ] **Step 3: Write the code**

An import of types alone by its names:

```rust edit path=src/imports.rs
<<<<<<< find
    pub specifier: String,
    /// How it is written.
    pub form: Form,
    /// Whether it brings in types alone: `import type`, `export type`, or a type import.
    pub type_only: bool,
    /// The line it begins on, counted from one.
    pub line: usize,
=======
    pub specifier: String,
    /// How it is written.
    pub form: Form,
    /// Whether it brings in types alone: `import type`, `export type`, a type import, or an `import`
    /// or `export … from` all of whose names are marked `type`.
    pub type_only: bool,
    /// The line it begins on, counted from one.
    pub line: usize,
>>>>>>> replace
<<<<<<< find

impl<'a> Visit<'a> for Finder<'_, 'a> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        let import = self.found(
            Self::text(&declaration.source),
            Form::Import,
            declaration.import_kind.is_type(),
            declaration.span,
        );

=======

impl<'a> Visit<'a> for Finder<'_, 'a> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        // `import { type A, type B }` brings in types alone, as `import type` does.
        let types_alone = declaration.specifiers.as_ref().is_some_and(|specifiers| {
            !specifiers.is_empty()
                && specifiers.iter().all(|specifier| {
                    matches!(
                        specifier,
                        ImportDeclarationSpecifier::ImportSpecifier(named)
                            if named.import_kind.is_type()
                    )
                })
        });
        let import = self.found(
            Self::text(&declaration.source),
            Form::Import,
            declaration.import_kind.is_type() || types_alone,
            declaration.span,
        );

>>>>>>> replace
<<<<<<< find
    }

    fn visit_export_from_declaration(&mut self, declaration: &ExportFromDeclaration<'a>) {
        let import = self.found(
            Self::text(&declaration.source),
            Form::Export,
            declaration.export_kind.is_type(),
            declaration.span,
        );

=======
    }

    fn visit_export_from_declaration(&mut self, declaration: &ExportFromDeclaration<'a>) {
        // `export { type A, type B } from` passes types alone on, as `export type` does.
        let types_alone = !declaration.specifiers.is_empty()
            && declaration
                .specifiers
                .iter()
                .all(|specifier| specifier.export_kind.is_type());
        let import = self.found(
            Self::text(&declaration.source),
            Form::Export,
            declaration.export_kind.is_type() || types_alone,
            declaration.span,
        );

>>>>>>> replace
```

The two kinds:

```rust edit path=src/findings.rs
<<<<<<< find
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::notice::Notice;
use crate::reading::Reading;
use crate::scripts;

/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
const FINGERPRINT_DIGITS: usize = 32;

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
=======
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::imports::{Form, Import};
use crate::notice::Notice;
use crate::reading::Reading;
use crate::resolve::Outcome;
use crate::scripts;

/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
const FINGERPRINT_DIGITS: usize = 32;

/// How many files of a loop a circular dependency's message names before it says how many more there
/// are (spec KIND-4).
const LOOP_NAMED: usize = 10;

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
>>>>>>> replace
<<<<<<< find
    let mut notices = Vec::new();

    unused(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

=======
    let mut notices = Vec::new();

    unused(reading, &mut found);
    unresolved(reading, &mut found);
    circular(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

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

/// The kind of each listed file, by its path.
fn kinds(reading: &Reading) -> BTreeMap<&str, Kind> {
    reading
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.kind))
        .collect()
}

/// The imports that lead to no file, in any file under the root that is read and is neither generated
/// nor a type declaration file (spec KIND-3).
fn unresolved(reading: &Reading, found: &mut Vec<Finding>) {
    let kinds = kinds(reading);

    for (path, read) in &reading.scripts.read {
        if kinds.get(path.as_str()) == Some(&Kind::Declaration)
            || reading.marks.generated.contains(path)
        {
            continue;
        }

        let Some(resolutions) = reading.resolved.get(path) else {
            continue;
        };

        for (import, resolution) in read.imports.iter().zip(resolutions) {
            if resolution.outcome == Outcome::Unresolved {
                found.push(Finding::new(
                    Rule::UnresolvedImport,
                    format!(
                        "The import of `{}` leads to no file, no package and no built-in module.",
                        import.specifier
                    ),
                    Location {
                        path: path.clone(),
                        line: Some(import.line),
                        column: None,
                    },
                    &[path, &import.specifier],
                ));
            }
        }
    }
}

/// Whether an import loads its target when the script runs, and so is an edge of a cycle: a static
/// import, `export … from`, `import … = require` or `require()`, not of types alone, and not read as
/// text (spec KIND-4, USE-6).
fn loads(import: &Import) -> bool {
    matches!(
        import.form,
        Form::Import | Form::Export | Form::ExportAll | Form::ImportEquals | Form::Require
    ) && !import.type_only
        && !import.has_query()
}

/// For each file, by its place in a list of files, each file it loads, by its place, with the line of
/// its first import that does and that import's place among its imports.
type Edges = Vec<BTreeMap<usize, (usize, usize)>>;

/// The edges cycles are found over: for each file, by its place in the list of files, each file it
/// loads, by its place, with the line of its first import that does, and that import's place. A type
/// declaration file loads nothing and is loaded by nothing (spec KIND-4).
fn edges(reading: &Reading) -> (Vec<&str>, Edges) {
    let kinds = kinds(reading);
    let nodes: Vec<&str> = reading
        .scripts
        .read
        .keys()
        .map(String::as_str)
        .filter(|path| kinds.get(path) != Some(&Kind::Declaration))
        .collect();
    let places: BTreeMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(place, path)| (*path, place))
        .collect();
    let mut edges = vec![BTreeMap::new(); nodes.len()];

    for (from, path) in nodes.iter().enumerate() {
        let (Some(read), Some(resolutions)) =
            (reading.scripts.read.get(*path), reading.resolved.get(*path))
        else {
            continue;
        };

        for (index, (import, resolution)) in read.imports.iter().zip(resolutions).enumerate() {
            let Outcome::Files(targets) = &resolution.outcome else {
                continue;
            };

            if !loads(import) {
                continue;
            }

            for target in targets {
                if let Some(to) = places.get(target.as_str()) {
                    let first: &mut (usize, usize) =
                        edges[from].entry(*to).or_insert((import.line, index));

                    *first = (*first).min((import.line, index));
                }
            }
        }
    }

    (nodes, edges)
}

/// The strongly connected sets of a graph, each in the order of its nodes, found by Tarjan's walk
/// written as a loop over a stack of its own, so that no chain of files is too long for the stack (spec
/// LIFE-3).
fn strongly_connected(edges: &Edges) -> Vec<Vec<usize>> {
    let unvisited = usize::MAX;
    let targets: Vec<Vec<usize>> = edges
        .iter()
        .map(|from| from.keys().copied().collect())
        .collect();
    let mut index = vec![unvisited; edges.len()];
    let mut low = vec![0; edges.len()];
    let mut held = vec![false; edges.len()];
    let mut stack: Vec<usize> = Vec::new();
    let mut next = 0;
    let mut sets = Vec::new();

    for start in 0..edges.len() {
        if index[start] != unvisited {
            continue;
        }

        // Each node being walked, with how many of its edges have been followed.
        let mut walk: Vec<(usize, usize)> = vec![(start, 0)];

        index[start] = next;
        low[start] = next;
        next += 1;
        stack.push(start);
        held[start] = true;

        while let Some(&(node, followed)) = walk.last() {
            if let Some(target) = targets[node].get(followed).copied() {
                if let Some(top) = walk.last_mut() {
                    top.1 += 1;
                }

                if index[target] == unvisited {
                    index[target] = next;
                    low[target] = next;
                    next += 1;
                    stack.push(target);
                    held[target] = true;
                    walk.push((target, 0));
                } else if held[target] {
                    low[node] = low[node].min(index[target]);
                }

                continue;
            }

            walk.pop();

            if let Some(&(parent, _)) = walk.last() {
                low[parent] = low[parent].min(low[node]);
            }

            if low[node] == index[node] {
                let mut set = Vec::new();

                while let Some(member) = stack.pop() {
                    held[member] = false;
                    set.push(member);

                    if member == node {
                        break;
                    }
                }

                set.sort_unstable();
                sets.push(set);
            }
        }
    }

    sets
}

/// The circular dependencies: each strongly connected set of two or more files, or a file that loads
/// itself, as one finding (spec KIND-4).
fn circular(reading: &Reading, found: &mut Vec<Finding>) {
    let (nodes, edges) = edges(reading);

    for set in strongly_connected(&edges) {
        let first = set[0];

        if set.len() == 1 && !edges[first].contains_key(&first) {
            continue;
        }

        let members: BTreeSet<usize> = set.iter().copied().collect();
        // Where a file of the set is: its first import, by line, that loads another file of the set, or
        // itself in a set of one.
        let placed = |node: usize| Location {
            path: nodes[node].to_owned(),
            line: edges[node]
                .iter()
                .filter(|(to, _)| members.contains(to) && (set.len() == 1 || **to != node))
                .map(|(_, (line, index))| (*line, *index))
                .min()
                .map(|(line, _)| line),
            column: None,
        };
        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let mut finding = Finding::new(
            Rule::CircularDependency,
            looped(&nodes, &edges, &members, first),
            placed(first),
            &paths,
        );

        finding.related = set[1..].iter().map(|node| placed(*node)).collect();
        found.push(finding);
    }
}

/// The message of a circular dependency: the shortest loop through the set's first file, found by a
/// walk in breadth that takes each file's targets in the order of their paths, up to ten files and
/// then how many more; and how many files the set holds, where the loop is not all of them (spec
/// KIND-4).
fn looped(nodes: &[&str], edges: &Edges, members: &BTreeSet<usize>, first: usize) -> String {
    let mut before: BTreeMap<usize, usize> = BTreeMap::new();
    let mut queue = VecDeque::from([first]);
    let mut last = first;

    'walk: while let Some(node) = queue.pop_front() {
        for target in edges[node].keys().filter(|to| members.contains(to)) {
            if *target == first {
                last = node;

                break 'walk;
            }

            if !before.contains_key(target) {
                before.insert(*target, node);
                queue.push_back(*target);
            }
        }
    }

    let mut cycle = vec![last];

    while let Some(node) = cycle.last().and_then(|node| before.get(node)) {
        cycle.push(*node);
    }

    cycle.reverse();

    let mut named: Vec<String> = cycle
        .iter()
        .take(LOOP_NAMED)
        .map(|node| format!("`{}`", nodes[*node]))
        .collect();

    if cycle.len() > LOOP_NAMED {
        named.push(format!("{} more", cycle.len() - LOOP_NAMED));
    }

    named.push(format!("`{}`", nodes[first]));

    let mut message = format!(
        "These files import each other in a loop: {}.",
        named.join(" → ")
    );

    if members.len() > cycle.len() {
        let _ = write!(
            message,
            " The loop is part of a set of {} files that each reach all the others.",
            members.len()
        );
    }

    message
}

/// One notice for each file with an `import()` or `require()` it cannot follow: what it loads must be
/// declared (spec ENTRY-10).
fn computed(reading: &Reading, notices: &mut Vec<Notice>) {
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test audit
```

Expected: PASS: 13 tests.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add src/findings.rs src/imports.rs tests/audit.test.rs
git commit -m "feat(audit): report unresolved imports and circular dependencies"
```

Expected: one commit on `feat/dead-code-4-audit`.
