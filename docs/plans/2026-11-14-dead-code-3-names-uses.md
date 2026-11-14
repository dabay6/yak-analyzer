# Dead-code findings, part 3: the names of each file, the uses of each export, and what the entry points reach: implementation plan

**Goal:** `graph` prints what every script, component and document exports, the names each import
brings in, every use of an export with where it counts from, and the files the entry points reach:
everything the unused-file, unused-export and unused-type findings are worked out from.

**Architecture:** `src/imports.rs` gains oxc's binding analysis, `oxc_semantic`, and with it reads each
script's own exports, the names of each import, and which of those names the script refers to.
`src/pages.rs` adds what a Vue component's template and an MDX document's text name. A new
`src/uses.rs` passes each use on through re-exports to the file that declares the name, counts an
entry point's exports as used, and walks what the entry points reach. `graph` prints all of it.

**Spec:** docs/specs/2026-11-13-dead-code-findings-design.md

**Implements:** USE-1, USE-2, USE-2a, USE-3, USE-4, USE-5, USE-6, CMD-4, DEP-1, FIX-1, FIX-2

The third of the item's six plans, listed in plan 1. Plan 2 has merged, so this plan's branch,
`feat/dead-code-3-names-uses`, is made from `main`, and its pull request goes against `main`.

Plan 1's list gave this plan the unused files, exports and types as well, with two notices (KIND-1,
KIND-2, ENTRY-9, ENTRY-10). Decision 1 moves those four to plan 4, and says why.

## Global constraints

- A use is a reference, never a mention: which binding a name refers to is decided by `oxc_semantic`,
  not by matching names (USE-1, USE-2, and the spec's rejected alternative).
- Where the analyzer cannot follow what is read, it counts more uses, never fewer: a namespace passed
  on, `require()`, `import()`, a template, a document's text (USE-2, USE-4, USE-5).
- Nothing reaches a file or uses an export that `graph` does not print (the spec's anti-goals, CMD-4).
- `graph` stays outside the interface version, so what it adds may change form later; the facts the
  fixtures hold do not (CMD-4, reading OUT-2).
- The same project gives the same bytes on every platform: every list `graph` adds is sorted or in
  the order written (reading RUN-8).
- Every crate is pinned to one exact version with its reason in `Cargo.toml`: `oxc_semantic =
  "=0.158.0"`, with the oxc group (DEP-1, reading DEP-3).

## The decisions this plan makes

1. **The findings wait for `audit`, in plan 4.** The spec has `graph` print the basis of every finding
   and never a finding (CMD-4), and `audit` is plan 4's. Unused files, exports and types built here
   would have no command that prints them, so no fixture could hold them. This plan builds and prints
   everything they are worked out from: exports, uses and reach. Plan 4 then reports all five kinds
   with the report: KIND-1, KIND-2, ENTRY-9's notices and ENTRY-10's notice move there. ENTRY-9's
   notices would also change the notices nearly every earlier fixture checks, which belongs with the
   change that makes them mean something.
2. **What `graph` adds.** A file that was read gains `exports`, each `{ name, kind, line, column,
   usedInFile }`, `kind` being `value` or `type`. An import gains `names`, each `{ name, local,
   exported }`: `name` is the target's name for it, `default`, or `*` for the namespace; `local` its
   binding; `exported` the names the importing file passes it on under. Two top-level lists join
   plan 1's: `uses`, each `{ file, name, by, line, entry }`, where `name` is `*` for every export and
   the use counts from `by` and `line`, or from `entry`, an entry point's source as `entries` prints
   one; and `reachable`, each `{ path, text }`.
3. **One use for each binding, from its first reference.** `graph` prints where a use counts from, not
   every reference: the line of the first one. A namespace's binding gives one use for each member
   read off it and one of every export when it is referred to in any other way.
4. **A use is printed where it lands and on the way there.** A file that passes a name on has the use
   printed too, under the name it offers, so a chain can be read hop by hop. A name a file does not
   offer is printed nowhere.
5. **`export *` never passes `default`** (USE-2). When every export of a file is used and it holds an
   `export *`, the target's names are printed one by one and not as `*`, so its `default` stays unused.
6. **An import with a query** gives one use of every export of its target, from the import's line,
   whatever names it brings in, and passes nothing on (USE-6). `reachable` marks a file only such
   imports reach as `text`, and its own imports are not followed.
7. **What an export is.** A name an `export` statement at the top of a script offers. `export =`, a
   CommonJS `module.exports`, and an `export` inside a `declare module` or a namespace are none. A
   file with none has nothing to report, and `require()` and `import … = require` use every export
   anyway.
8. **Value or type is the binding's.** oxc's flags for the binding decide: a value when any
   declaration of the name is one, so an interface merged with a constant is one value export, and a
   namespace is a type only when it holds types alone.
9. **`usedInFile`** is true when anything in the file refers to the binding other than the `export`
   statement that names it. A function that calls itself counts.
10. **Where an export is.** The line and column of its name, the column counted from one in UTF-16
    code units, which is SARIF's own counting, so plan 4 prints it as it is. A default export with no
    name is where its declaration or expression begins.
11. **A type import** uses the first name read off the file: `import('./x').A.B` uses `A`; with none,
    every export.
12. **A Vue component.** Its default export is the component, never reported (USE-4), so it is not
    among its exports, and `export default` of an imported binding uses that binding there, where in
    a script it would pass it on. The template's words are every run of letters, digits, `_` and `$`
    that does not begin with a digit; a word written with hyphens is also read in camelCase and in
    PascalCase. Only a `<script setup>` block's imports are the template's to use. An export the
    template names is a use that counts from the component itself.
13. **An MDX document.** Its text is every line outside its front matter, its statements and its
    fenced code blocks; a statement that does not parse is text (reading PARSE-4). The words are found
    as a template's are, without the hyphen rule.
14. **A `src`**, of a component's script block or of a page's script, uses every export of the file
    it names: that file is the block's or the page's own script.
15. **No new test file.** A new file needs a `[[test]]` entry in `Cargo.toml`, which is applied with
    the code, so its tests could not fail first for the right reason (plan 1's ruling). The fixtures
    join the tests of the area they check: `imports` for names and uses, `entries` for an entry
    point's uses and reach, `pages` for Vue and MDX.
16. **`oxc_semantic` runs with no syntax checks of its own,** so what parses is what parsed before
    (reading PARSE-2), and with its table of nodes, which the reading of a namespace's members needs.

## Review focus

- **A binding an MDX document uses only in another statement of its own** (`import { Chart } from
  './chart'`, a blank line, then `export const meta = { chart: Chart }`). Each statement is parsed by
  itself (reading PARSE-4), and USE-5 counts the text outside statements, so this use is not seen and
  `Chart` could later be reported unused. No fixture holds it; the check on the named repositories
  (plan 6) shows whether real documents are written so.
- **A default import of a file that only has `export =` or `module.exports`** uses a `default` the
  file does not offer. Nothing is reported for such a file, since it has no exports. Only the `.cjs`
  fixture holds the form.
- **A namespace that is destructured or passed on** (`const { a } = ns`, `f(ns)`) uses every export.
  The fixture holds the passed-on case.
- **A word of a template inside a comment or an attribute's text** counts as a use, and so does a
  template in another language, such as Pug, which is scanned as text. Both are the safe direction.
- **`export import a = require('./x')`** passes `a` on to every export of `./x`. No fixture holds it.
- **Speed.** Every script now gets oxc's binding analysis. On three real checkouts of 2,600 to 3,800
  files a whole `graph` run took about one to two seconds on the session's machine; the bars are
  measured in plan 6 (LIVE-5).
- **The size of `uses`.** One entry for each binding of each import that is referred to, and one for
  each hop of a chain: about 12,000 on the checkout with the most.

### Task 1: names, uses and reach, in graph

**Implements:** USE-1, USE-2, USE-2a, USE-3, USE-4, USE-5, USE-6, CMD-4, DEP-1, FIX-1, FIX-2

**Test:** `cargo test --release --locked --test imports --test entries --test pages --test file-list --test hand-off`

**Files:**
- Create: `src/uses.rs`
- Modify: `src/imports.rs`, `src/pages.rs`, `src/graph.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`
- Test: `tests/imports.test.rs`, `tests/entries.test.rs`, `tests/pages.test.rs`,
  `tests/file-list.test.rs`, `tests/hand-off.test.rs`

**Interfaces:**
- Consumes: `scripts::Scripts`; `resolve::Resolution` and `Outcome`; plan 1's `entries::Entries` and
  `Source`, with `Source::printed`; `oxc_semantic`'s `SemanticBuilder`, `Semantic`, `SymbolId` and
  `NodeId`.
- Produces: `imports::Name`, `Used` and `Export`, with `EVERY` and `DEFAULT`; `Import::names` and
  `uses`; `Read::exports` and `named`; `uses::find`, `Uses` (`list`, `reached`), `Use`, `Origin` and
  `Reach`. Plan 4 reads `Uses` and each file's `exports` to report the unused files, exports and
  types, and prints each export's `line` and `column` as its location.

- [ ] **Step 1: Write the failing tests**

Five tests join `tests/imports.test.rs`, for a script's exports, the names of its imports, and its
uses, through re-exports too:

```rust edit path=tests/imports.test.rs
<<<<<<< find
//! Runs the built binary on made-up projects and checks the imports it finds in each script, and how it
//! treats a script it cannot parse (spec PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-5, RUN-4).

#[path = "support/project.rs"]
mod project;

=======
//! Runs the built binary on made-up projects and checks the imports it finds in each script, the names
//! each brings in, what a script exports and what it uses of the files it imports, and how it treats a
//! script it cannot parse (spec PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-4, RUN-4; dead-code spec USE-1,
//! USE-2, USE-2a, FIX-1, FIX-2).

#[path = "support/project.rs"]
mod project;

>>>>>>> replace
<<<<<<< find
        line.starts_with("yak-analyzer: the field `name` in package.json"),
        "{line}"
    );
}
=======
        line.starts_with("yak-analyzer: the field `name` in package.json"),
        "{line}"
    );
}

/// The exports of a file as `graph` prints them: each with its name, its kind, the line and the column
/// of its name, and whether its own file refers to it.
fn exports<'a>(document: &'a Value, path: &str) -> Vec<(&'a str, &'a str, u64, u64, bool)> {
    file(document, path)["exports"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} was read"))
        .iter()
        .map(|export| {
            (
                export["name"].as_str().expect("a name"),
                export["kind"].as_str().expect("a kind"),
                export["line"].as_u64().expect("a line"),
                export["column"].as_u64().expect("a column"),
                export["usedInFile"].as_bool().expect("a mark"),
            )
        })
        .collect()
}

/// One name an import brings in, as `graph` prints it: the target's name for it, its local binding,
/// and the names the importing file passes it on under.
fn name(name: &str, local: Option<&str>, exported: &[&str]) -> Value {
    json!({ "exported": exported, "local": local, "name": name })
}

/// The uses of a file's exports that count from a reference, in the document's order: each with the
/// export's name, `*` for every export, and the file and line it counts from.
fn uses<'a>(document: &'a Value, path: &str) -> Vec<(&'a str, &'a str, u64)> {
    document["uses"]
        .as_array()
        .expect("a list of uses")
        .iter()
        .filter(|used| used["file"] == json!(path) && used["entry"].is_null())
        .map(|used| {
            (
                used["name"].as_str().expect("a name"),
                used["by"].as_str().expect("a file"),
                used["line"].as_u64().expect("a line"),
            )
        })
        .collect()
}

#[test]
fn reads_what_a_script_exports_each_once_as_a_value_or_a_type() {
    let project = Project::new("imports-exports");

    project
        .write(
            "src/names.ts",
            "function helper() {\n\
             \x20 return 1;\n\
             }\n\
             export function used() {\n\
             \x20 return helper();\n\
             }\n\
             export const alone = used();\n\
             export interface Shape {}\n\
             export type Alias = Shape;\n\
             export enum Color {\n\
             \x20 Red,\n\
             }\n\
             export namespace OnlyTypes {\n\
             \x20 export type Inner = string;\n\
             }\n\
             export namespace WithValues {\n\
             \x20 export const inner = 1;\n\
             }\n\
             export class Widget {}\n\
             export const { first, rest: [second] } = { first: 1, rest: [2] };\n\
             const hidden = 1;\n\
             export { hidden as shown };\n\
             export function over(value: string): void;\n\
             export function over(value: number): void;\n\
             export function over(value: unknown): void {\n\
             \x20 void value;\n\
             }\n\
             export interface Merged {}\n\
             export const Merged = 1;\n\
             /* \u{1F600} */ export const [\u{3C0}, after] = [1, 2];\n\
             export default class {}\n",
        )
        .write(
            "src/common.cjs",
            "module.exports = { a: 1 };\nexports.b = 2;\n",
        )
        .commit();

    let document = project.graph();

    // A type is an interface, a type alias or a namespace of types alone; anything else is a value. A
    // name offered more than once, as overloads and merged declarations are, is one export, and a value
    // when any of them is. Each is where its name is written, the column in UTF-16 code units (spec
    // USE-1).
    assert_eq!(
        exports(&document, "src/names.ts"),
        [
            ("used", "value", 4, 17, true),
            ("alone", "value", 7, 14, false),
            ("Shape", "type", 8, 18, true),
            ("Alias", "type", 9, 13, false),
            ("Color", "value", 10, 13, false),
            ("OnlyTypes", "type", 13, 18, false),
            ("WithValues", "value", 16, 18, false),
            ("Widget", "value", 19, 14, false),
            ("first", "value", 20, 16, false),
            ("second", "value", 20, 30, false),
            ("shown", "value", 22, 20, false),
            ("over", "value", 23, 17, false),
            ("Merged", "value", 28, 18, false),
            ("\u{3C0}", "value", 30, 24, false),
            ("after", "value", 30, 27, false),
            ("default", "value", 31, 16, false),
        ]
    );
    // An export is a name an `export` statement offers: CommonJS offers none.
    assert_eq!(exports(&document, "src/common.cjs"), []);
}

#[test]
fn reads_the_names_each_import_brings_in_and_the_names_it_passes_on() {
    let project = Project::new("imports-names");

    project
        .write(
            "src/main.ts",
            "import fallback, { a, b as bee, type T } from './lib';\n\
             import * as all from './lib';\n\
             import './lib';\n\
             export { c, d as dee } from './lib';\n\
             export * from './lib';\n\
             export * as grouped from './lib';\n\
             import old = require('./lib');\n\
             type Later = import('./lib').Deep.Deeper;\n\
             const lazy = import('./lib');\n\
             const now = require('./lib');\n\
             import { again, twice } from './lib';\n\
             export { again as onceMore, twice, twice as thrice };\n\
             export default fallback;\n\
             export { all };\n",
        )
        .write("src/lib.ts", "")
        .commit();

    let document = project.graph();
    let names: Vec<Value> = file(&document, "src/main.ts")["imports"]
        .as_array()
        .expect("a list of imports")
        .iter()
        .map(|import| import["names"].clone())
        .collect();

    // A default, a named and a namespace import each make a binding; `export … from` makes none and
    // offers the name again; an imported binding exported again is passed on under each name it is
    // exported by (spec USE-1, USE-2a).
    assert_eq!(
        names,
        [
            json!([
                name("default", Some("fallback"), &["default"]),
                name("a", Some("a"), &[]),
                name("b", Some("bee"), &[]),
                name("T", Some("T"), &[]),
            ]),
            json!([name("*", Some("all"), &["all"])]),
            json!([]),
            json!([name("c", None, &["c"]), name("d", None, &["dee"])]),
            json!([name("*", None, &[])]),
            json!([name("*", None, &["grouped"])]),
            json!([name("*", Some("old"), &[])]),
            json!([name("Deep", None, &[])]),
            json!([]),
            json!([]),
            json!([
                name("again", Some("again"), &["onceMore"]),
                name("twice", Some("twice"), &["twice", "thrice"]),
            ]),
        ]
    );
    // A name a file only passes on is not its own export (spec USE-2a).
    assert_eq!(exports(&document, "src/main.ts"), []);
}

#[test]
fn counts_a_use_where_a_binding_is_referred_to_and_not_where_it_is_only_imported() {
    let project = Project::new("imports-uses");

    project
        .write(
            "src/lib.ts",
            "export const used = 1;\n\
             export const unreferenced = 2;\n\
             export const inFileOnly = 3;\n\
             export const nowhere = inFileOnly;\n\
             export const shadowed = 5;\n\
             export interface Shape {}\n\
             export const one = 1;\n\
             export const two = 2;\n\
             export const three = 3;\n\
             export default function main() {}\n",
        )
        .write(
            "src/main.ts",
            "import main, { used, unreferenced, shadowed, type Shape } from './lib';\n\
             import * as lib from './lib';\n\
             \n\
             function inner(shadowed: number): Shape | number {\n\
             \x20 return shadowed + used;\n\
             }\n\
             \n\
             main();\n\
             inner(lib.one + lib['two']);\n",
        )
        // The same names as locals of another file are no use of the exports.
        .write(
            "src/other.ts",
            "const nowhere = 1;\nconst lib = { three: 3 };\nconsole.log(nowhere, lib.three);\n",
        )
        .write("src/bag.ts", "export const x = 1;\nexport const y = 2;\n")
        .write(
            "src/whole.ts",
            "import * as bag from './bag';\nconsole.log(bag);\n",
        )
        .write("src/thing.js", "export const twin = 1;\n")
        .write("src/thing.d.ts", "export declare const twin: number;\n")
        .write(
            "src/twins.ts",
            "import { twin } from './thing';\nconsole.log(twin);\n",
        )
        .commit();

    let document = project.graph();

    // A use is a reference, in code or in a type, from the line of the first one: an import whose
    // binding nothing refers to, or that a parameter of the same name hides, is none. A namespace's
    // member read by a name the script writes uses that export (spec USE-2, FIX-1 pattern 17).
    assert_eq!(
        uses(&document, "src/lib.ts"),
        [
            ("Shape", "src/main.ts", 4),
            ("default", "src/main.ts", 8),
            ("one", "src/main.ts", 9),
            ("two", "src/main.ts", 9),
            ("used", "src/main.ts", 5),
        ]
    );
    // An export its own file refers to is marked, and is not used by another file.
    assert_eq!(
        exports(&document, "src/lib.ts")[2..4],
        [
            ("inFileOnly", "value", 3, 14, true),
            ("nowhere", "value", 4, 14, false),
        ]
    );
    // A namespace referred to in any other way uses every export.
    assert_eq!(uses(&document, "src/bag.ts"), [("*", "src/whole.ts", 2)]);
    // An import that leads to several files uses the export in each.
    assert_eq!(
        uses(&document, "src/thing.d.ts"),
        [("twin", "src/twins.ts", 2)]
    );
    assert_eq!(
        uses(&document, "src/thing.js"),
        [("twin", "src/twins.ts", 2)]
    );
}

#[test]
fn passes_a_use_on_through_re_exports_to_the_file_that_declares_the_name() {
    let project = Project::new("imports-chains");

    project
        .write(
            "src/index.ts",
            "export { leaf as renamed } from './middle';\n\
             export * as group from './group';\n\
             import { kept } from './middle';\n\
             export { kept };\n",
        )
        .write(
            "src/middle.ts",
            "export * from './leaf';\nexport * from './loop';\n",
        )
        // Two files that pass each other's names on end the walk all the same.
        .write(
            "src/loop.ts",
            "export * from './middle';\nexport const looped = 1;\n",
        )
        .write(
            "src/leaf.ts",
            "export const leaf = 1;\nexport const kept = 2;\nexport const spare = 3;\nexport default 4;\n",
        )
        .write(
            "src/group.ts",
            "export const g1 = 1;\nexport const g2 = 2;\n",
        )
        .write(
            "src/main.ts",
            "import { renamed, group, kept, missing } from './index';\n\
             import fallback from './middle';\n\
             console.log(renamed, group, kept, missing, fallback);\n",
        )
        .write(
            "src/all.ts",
            "import * as middle from './middle';\nconsole.log(middle);\n",
        )
        .commit();

    let document = project.graph();

    // A use of a name a file passes on is a use of the name it comes from, through `export … from`, an
    // imported binding exported again, and `export *`, to the file that declares it. `export * as`
    // passes a use on to every export, and `export *` never passes `default` (spec USE-2, FIX-2).
    assert_eq!(
        uses(&document, "src/index.ts"),
        [
            ("group", "src/main.ts", 3),
            ("kept", "src/main.ts", 3),
            ("renamed", "src/main.ts", 3),
        ]
    );
    assert_eq!(uses(&document, "src/middle.ts"), [("*", "src/all.ts", 2)]);
    assert_eq!(
        uses(&document, "src/leaf.ts"),
        [
            ("kept", "src/all.ts", 2),
            ("kept", "src/main.ts", 3),
            ("leaf", "src/all.ts", 2),
            ("leaf", "src/main.ts", 3),
            ("spare", "src/all.ts", 2),
        ]
    );
    assert_eq!(
        uses(&document, "src/loop.ts"),
        [("looped", "src/all.ts", 2)]
    );
    assert_eq!(uses(&document, "src/group.ts"), [("*", "src/main.ts", 3)]);
    // Neither file that only passes names on has an export of its own (spec USE-2a).
    assert_eq!(exports(&document, "src/index.ts"), []);
    assert_eq!(exports(&document, "src/middle.ts"), []);
}

#[test]
fn uses_every_export_of_a_file_a_call_loads_and_the_name_a_type_import_reads() {
    let project = Project::new("imports-loads");

    project
        .write(
            "src/main.ts",
            "import old = require('./old');\n\
             const now = require('./now');\n\
             const later = import('./later');\n\
             type Picked = import('./types').Picked;\n\
             type Whole = typeof import('./shapes');\n\
             console.log(old, now, later);\n",
        )
        .write("src/old.ts", "export const a = 1;\n")
        .write("src/now.ts", "export const b = 1;\n")
        .write("src/later.ts", "export const c = 1;\n")
        .write(
            "src/types.ts",
            "export type Picked = string;\nexport type Left = number;\n",
        )
        .write("src/shapes.ts", "export const d = 1;\n")
        .commit();

    let document = project.graph();

    // What is read off the result of `require`, `import()` and `import … = require` is not followed, so
    // each uses every export; a type import uses the name it reads (spec USE-2).
    assert_eq!(uses(&document, "src/old.ts"), [("*", "src/main.ts", 1)]);
    assert_eq!(uses(&document, "src/now.ts"), [("*", "src/main.ts", 2)]);
    assert_eq!(uses(&document, "src/later.ts"), [("*", "src/main.ts", 3)]);
    assert_eq!(
        uses(&document, "src/types.ts"),
        [("Picked", "src/main.ts", 4)]
    );
    assert_eq!(uses(&document, "src/shapes.ts"), [("*", "src/main.ts", 5)]);
}
>>>>>>> replace
```

Two join `tests/entries.test.rs`, for an entry point's own exports and for what the entry points
reach:

```rust edit path=tests/entries.test.rs
<<<<<<< find
//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! and the packages whose entry points are likely incomplete (spec ENTRY-1 to ENTRY-8, CMD-4, FIX-1,
//! FIX-2).

#[path = "support/project.rs"]
mod project;
=======
//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! the packages whose entry points are likely incomplete, the uses an entry point makes of its own
//! exports, and the files the entry points reach (spec ENTRY-1 to ENTRY-8, USE-3, USE-6, CMD-4, FIX-1,
//! FIX-2).

#[path = "support/project.rs"]
mod project;
>>>>>>> replace
<<<<<<< find
             entry point."
        )
    );
}
=======
             entry point."
        )
    );
}

/// The uses a document holds that count from an entry point: each with the file whose exports are
/// used, the export's name, `*` for every export, and the field or declaration that made the entry
/// point one.
fn entry_uses(document: &Value) -> Vec<(&str, &str, &str)> {
    document["uses"]
        .as_array()
        .expect("a list of uses")
        .iter()
        .filter(|used| !used["entry"].is_null())
        .map(|used| {
            (
                used["file"].as_str().expect("a file"),
                used["name"].as_str().expect("a name"),
                used["entry"]["from"].as_str().expect("a source"),
            )
        })
        .collect()
}

/// The files a document says the entry points reach, each with whether it is reached only as text.
fn reachable(document: &Value) -> Vec<(&str, bool)> {
    document["reachable"]
        .as_array()
        .expect("a list of files")
        .iter()
        .map(|file| {
            (
                file["path"].as_str().expect("a path"),
                file["text"].as_bool().expect("a mark"),
            )
        })
        .collect()
}

#[test]
fn an_entry_points_exports_are_all_used_through_its_re_exports_too() {
    let project = Project::new("entries-uses");

    project
        .write(
            "package.json",
            r#"{ "name": "lib", "main": "src/index.ts", "bin": "src/index.ts" }"#,
        )
        .write(
            "src/index.ts",
            "export { helper } from './helper';\nexport * from './all';\nexport const own = 1;\n",
        )
        .write(
            "src/helper.ts",
            "export const helper = 1;\nexport const internal = 2;\n",
        )
        .write("src/all.ts", "export const a = 1;\nexport default 2;\n")
        .write("src/unused.ts", "export const dead = 1;\n")
        .commit();

    let document = project.graph();

    // Whatever starts an entry point consumes its exports, once for each source that made it one, and
    // that use passes on through its re-exports: to the one name of `./helper`, and to every name of
    // `./all` but `default` (spec USE-3).
    assert_eq!(
        entry_uses(&document),
        [
            ("src/all.ts", "a", "bin"),
            ("src/all.ts", "a", "main"),
            ("src/helper.ts", "helper", "bin"),
            ("src/helper.ts", "helper", "main"),
            ("src/index.ts", "*", "bin"),
            ("src/index.ts", "*", "main"),
        ]
    );
    assert_eq!(
        reachable(&document),
        [
            ("src/all.ts", false),
            ("src/helper.ts", false),
            ("src/index.ts", false),
        ]
    );
}

#[test]
fn reaches_files_through_every_form_of_import_and_reads_one_imported_with_a_query_as_text() {
    let project = Project::new("entries-reach");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "src/main.ts" }"#,
        )
        .write(
            "src/main.ts",
            "import type { T } from './types';\n\
             import raw from './shader.ts?raw';\n\
             import both from './both.ts?raw';\n\
             import './both';\n\
             import App from './App.vue';\n\
             const lazy = () => import('./lazy');\n\
             const old = require('./old.cjs');\n\
             export * from './again';\n\
             export const value: T = 'x';\n\
             console.log(raw, both, App, lazy, old);\n",
        )
        .write("src/types.ts", "export type T = string;\n")
        .write(
            "src/shader.ts",
            "export * from './never';\nexport const shade = 1;\n",
        )
        .write("src/never.ts", "export const never = 1;\n")
        .write("src/both.ts", "import './deep';\nexport const both = 1;\n")
        .write("src/deep.ts", "")
        .write(
            "src/App.vue",
            "<script src=\"./logic.ts\"></script>\n<template><p /></template>\n",
        )
        .write("src/logic.ts", "export default {};\n")
        .write("src/lazy.ts", "")
        .write("src/old.cjs", "")
        .write("src/again.ts", "")
        .write(
            "index.html",
            "<script type=\"module\" src=\"./src/page.ts\"></script>\n",
        )
        .write("src/page.ts", "")
        .write("src/orphan.ts", "import './island';\n")
        .write("src/island.ts", "")
        .commit();

    let document = project.graph();

    // Every import that leads to files is followed, of every form: a type import, a call, a re-export,
    // a component's `src` and a page's script. A file only imports with a query reach is read as text,
    // so the file it imports is not reached; one an import without a query also reaches is read as
    // code. A file no entry point reaches is not here (spec USE-6).
    assert_eq!(
        reachable(&document),
        [
            ("src/App.vue", false),
            ("src/again.ts", false),
            ("src/both.ts", false),
            ("src/deep.ts", false),
            ("src/lazy.ts", false),
            ("src/logic.ts", false),
            ("src/main.ts", false),
            ("src/old.cjs", false),
            ("src/page.ts", false),
            ("src/shader.ts", true),
            ("src/types.ts", false),
        ]
    );

    let uses = |path: &str| -> Vec<Value> {
        document["uses"]
            .as_array()
            .expect("a list of uses")
            .iter()
            .filter(|used| used["file"] == json!(path))
            .map(|used| json!([used["name"], used["by"], used["line"]]))
            .collect()
    };

    // Read as text, a file's exports are all used, and it passes nothing on.
    assert_eq!(uses("src/shader.ts"), [json!(["*", "src/main.ts", 2])]);
    assert_eq!(uses("src/never.ts"), Vec::<Value>::new());
    // The script a component's block names is the component's own.
    assert_eq!(uses("src/logic.ts"), [json!(["*", "src/App.vue", 1])]);
}
>>>>>>> replace
```

Two join `tests/pages.test.rs`, for a Vue component's template and an MDX document's text:

````rust edit path=tests/pages.test.rs
<<<<<<< find
//! Runs the built binary on made-up projects and checks the imports it finds in Vue components, MDX
//! documents and HTML pages, and where a page's scripts lead (spec PARSE-2 to PARSE-5, IMP-1, RES-7,
//! CFG-2, FIX-2, FIX-4).

#[path = "support/project.rs"]
mod project;
=======
//! Runs the built binary on made-up projects and checks the imports it finds in Vue components, MDX
//! documents and HTML pages, where a page's scripts lead, and the bindings a component's template and a
//! document's text use (spec PARSE-2 to PARSE-5, IMP-1, RES-7, CFG-2, FIX-2, FIX-4; dead-code spec
//! USE-4, USE-5, FIX-1).

#[path = "support/project.rs"]
mod project;
>>>>>>> replace
<<<<<<< find
        assert_eq!(file(&document, path).get("imports"), None, "{path}");
    }
}
=======
        assert_eq!(file(&document, path).get("imports"), None, "{path}");
    }
}

/// The uses of a file's exports a document holds, in its order: each with the export's name, `*` for
/// every export, and the file and line it counts from.
fn uses<'a>(document: &'a Value, path: &str) -> Vec<(&'a str, &'a str, u64)> {
    document["uses"]
        .as_array()
        .expect("a list of uses")
        .iter()
        .filter(|used| used["file"] == json!(path))
        .map(|used| {
            (
                used["name"].as_str().expect("a name"),
                used["by"].as_str().expect("a file"),
                used["line"].as_u64().expect("a line"),
            )
        })
        .collect()
}

/// The names of the exports `graph` prints for a file.
fn exported<'a>(document: &'a Value, path: &str) -> Vec<&'a str> {
    file(document, path)["exports"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} was read"))
        .iter()
        .map(|export| export["name"].as_str().expect("a name"))
        .collect()
}

#[test]
fn a_vue_template_uses_the_bindings_it_names() {
    let project = Project::new("pages-vue-uses");

    project
        .write(
            "src/App.vue",
            "<template>\n\
             \x20 <Child :value=\"format(count)\" />\n\
             \x20 <other-child class=\"plain\" />\n\
             \x20 <p>{{ label }} {{ texts.title }}</p>\n\
             </template>\n\
             \n\
             <script setup lang=\"ts\">\n\
             import Child from './Child';\n\
             import OtherChild from './OtherChild';\n\
             import Unused from './Unused';\n\
             import { format, spare } from './format';\n\
             import * as texts from './texts';\n\
             const count = 1;\n\
             </script>\n\
             \n\
             <script lang=\"ts\">\n\
             import plain from './plain';\n\
             export const label = 'x';\n\
             export const quiet = 'y';\n\
             export default { name: 'App' };\n\
             </script>\n",
        )
        .write("src/Child.ts", "export default {};\n")
        .write("src/OtherChild.ts", "export default {};\n")
        .write("src/Unused.ts", "export default {};\n")
        .write(
            "src/format.ts",
            "export const format = String;\nexport const spare = 1;\n",
        )
        .write("src/texts.ts", "export const title = 't';\n")
        .write("src/plain.ts", "export default 1;\n")
        .commit();

    let document = project.graph();

    // A binding a `<script setup>` block imports is used where the template names it: as a tag, in
    // PascalCase or in kebab-case, or as a word anywhere in its text. One it does not name is not used
    // (spec USE-4, FIX-1 pattern 16).
    assert_eq!(
        uses(&document, "src/Child.ts"),
        [("default", "src/App.vue", 2)]
    );
    assert_eq!(
        uses(&document, "src/OtherChild.ts"),
        [("default", "src/App.vue", 3)]
    );
    assert_eq!(uses(&document, "src/Unused.ts"), []);
    assert_eq!(
        uses(&document, "src/format.ts"),
        [("format", "src/App.vue", 2)]
    );
    assert_eq!(uses(&document, "src/texts.ts"), [("*", "src/App.vue", 4)]);
    // A plain block's import is not the template's to use, though the template holds the word.
    assert_eq!(uses(&document, "src/plain.ts"), []);
    // An export of a block is used when the template names it, and the component's default export is
    // the component, never among its exports.
    assert_eq!(exported(&document, "src/App.vue"), ["label", "quiet"]);
    assert_eq!(
        uses(&document, "src/App.vue"),
        [("label", "src/App.vue", 4)]
    );
}

#[test]
fn an_mdx_document_uses_the_bindings_its_text_names() {
    let project = Project::new("pages-mdx-uses");

    project
        .write(
            "docs/guide.mdx",
            "import { Chart, Table, Hidden } from './widgets'\n\
             export { extra } from './extra'\n\
             \n\
             # Guide\n\
             \n\
             <Chart />\n\
             \n\
             ```jsx\n\
             <Table />\n\
             ```\n",
        )
        .write(
            "docs/widgets.ts",
            "export const Chart = 1;\nexport const Table = 2;\nexport const Hidden = 3;\n",
        )
        .write("docs/extra.ts", "export const extra = 1;\n")
        .write(
            "src/main.ts",
            "import { extra } from '../docs/guide.mdx';\nconsole.log(extra);\n",
        )
        .commit();

    let document = project.graph();

    // A binding the document's text names is used, from the line that names it; one named only in its
    // statements or in a fenced code block is not (spec USE-5, FIX-1 pattern 16).
    assert_eq!(
        uses(&document, "docs/widgets.ts"),
        [("Chart", "docs/guide.mdx", 6)]
    );
    // Its `export … from` passes a use on as a script's does, and it has no export of its own.
    assert_eq!(
        uses(&document, "docs/extra.ts"),
        [("extra", "src/main.ts", 2)]
    );
    assert_eq!(
        uses(&document, "docs/guide.mdx"),
        [("extra", "src/main.ts", 2)]
    );
    assert_eq!(exported(&document, "docs/guide.mdx"), Vec::<&str>::new());
}
>>>>>>> replace
````

The two tests that compare whole objects gain the new fields:

```rust edit path=tests/file-list.test.rs
<<<<<<< find
        .write(".git/info/exclude", "local/\n");

    let document = project.graph();
    // A script, a component, a document and a page are read, so each has its imports, which in these
    // empty files are none.
    let file = |path: &str, kind: &str, tracked: bool| {
        let mut expected = json!({
            "data": false,
=======
        .write(".git/info/exclude", "local/\n");

    let document = project.graph();
    // A script, a component, a document and a page are read, so each has its imports and its exports,
    // which in these empty files are none.
    let file = |path: &str, kind: &str, tracked: bool| {
        let mut expected = json!({
            "data": false,
>>>>>>> replace
<<<<<<< find

        if ["script", "declaration", "vue", "mdx", "page"].contains(&kind) {
            expected["computedImports"] = json!(0);
            expected["imports"] = json!([]);
        }

=======

        if ["script", "declaration", "vue", "mdx", "page"].contains(&kind) {
            expected["computedImports"] = json!(0);
            expected["exports"] = json!([]);
            expected["imports"] = json!([]);
        }

>>>>>>> replace
```

```rust edit path=tests/hand-off.test.rs
<<<<<<< find
                nothing("generated", "src/generated/**", "project"),
            ],
            "packages": [],
        })
    );
}
=======
                nothing("generated", "src/generated/**", "project"),
            ],
            "packages": [],
            "reachable": [],
            "uses": [],
        })
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test imports --test entries --test pages --test file-list --test hand-off
```

Expected: FAIL. The nine new tests fail, five in `imports`, two in `entries` and two in `pages`, since
`graph` prints no `exports`, `names`, `uses` or `reachable` yet; and the two tests that compare whole
objects fail on the fields they now expect. The rest pass.

- [ ] **Step 3: Write the code**

`oxc_semantic` joins the oxc group:

```toml edit path=Cargo.toml
<<<<<<< find
path = "tests/entries.test.rs"

[dependencies]
# Parse scripts into syntax trees and walk them; released together, so pinned together (spec AN-2, DEP-3).
oxc_allocator = "=0.158.0"
oxc_ast = "=0.158.0"
oxc_ast_visit = "=0.158.0"
oxc_parser = "=0.158.0"
oxc_span = "=0.158.0"
# Finds the tsconfig that governs a file, through its `references` and `extends`, with its `paths` and
# `baseUrl`, from a file system of the analyzer's own (spec RES-9).
=======
path = "tests/entries.test.rs"

[dependencies]
# Parse scripts into syntax trees, walk them, and tie each reference to the binding it names
# (`oxc_semantic`, spec USE-1); released together, so pinned together (spec AN-2, DEP-3).
oxc_allocator = "=0.158.0"
oxc_ast = "=0.158.0"
oxc_ast_visit = "=0.158.0"
oxc_parser = "=0.158.0"
oxc_semantic = "=0.158.0"
oxc_span = "=0.158.0"
# Finds the tsconfig that governs a file, through its `references` and `extends`, with its `paths` and
# `baseUrl`, from a file system of the analyzer's own (spec RES-9).
>>>>>>> replace
```

```toml edit path=Cargo.lock
<<<<<<< find
 "syn 3.0.10",
]

[[package]]
name = "email_address"
version = "0.2.9"
=======
 "syn 3.0.10",
]

[[package]]
name = "either"
version = "1.19.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0e9c71c2167ca323c882b99918929403426e2373ea17242ff5653e0d5e1058be"

[[package]]
name = "email_address"
version = "0.2.9"
>>>>>>> replace
<<<<<<< find
 "serde_core",
]

[[package]]
name = "itoa"
version = "1.0.19"
=======
 "serde_core",
]

[[package]]
name = "itertools"
version = "0.15.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b4baf93f58d4425749ca49a51c50ebab072c5df6994d08fed93541c331481dc"
dependencies = [
 "either",
]

[[package]]
name = "itoa"
version = "1.0.19"
>>>>>>> replace
<<<<<<< find
 "windows",
]

[[package]]
name = "oxc_span"
version = "0.158.0"
=======
 "windows",
]

[[package]]
name = "oxc_semantic"
version = "0.158.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "36c8301783f312c94916a56eab2dcd1e3493f2d8d59bbd7fbc9169425502bb97"
dependencies = [
 "itertools",
 "memchr",
 "oxc_allocator",
 "oxc_ast",
 "oxc_ast_visit",
 "oxc_data_structures",
 "oxc_diagnostics",
 "oxc_ecmascript",
 "oxc_index",
 "oxc_span",
 "oxc_str",
 "oxc_syntax",
 "rustc-hash",
 "self_cell",
 "smallvec",
]

[[package]]
name = "oxc_span"
version = "0.158.0"
>>>>>>> replace
<<<<<<< find
 "oxc_ast_visit",
 "oxc_parser",
 "oxc_resolver",
 "oxc_span",
 "regex",
 "same-file",
=======
 "oxc_ast_visit",
 "oxc_parser",
 "oxc_resolver",
 "oxc_semantic",
 "oxc_span",
 "regex",
 "same-file",
>>>>>>> replace
```

The reading of a script, with its names:

```rust path=src/imports.rs
//! What an import is, and what a script's names are: parsing a script and finding every import in it,
//! the names each brings in, what the script exports, and what of each import's target it uses (spec
//! PARSE-1 to PARSE-4, IMP-1 to IMP-4, USE-1, USE-2).

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, CallExpression, Declaration, ExportAllDeclaration,
    ExportDefaultDeclarationKind, ExportFromDeclaration, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, ImportExpression, JSXMemberExpressionObject, ModuleExportName,
    Program, Statement, StringLiteral, TSImportEqualsDeclaration, TSImportType,
    TSImportTypeQualifier, TSModuleReference, TSTypeName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{NodeId, Semantic, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};

/// The name an import or an export has for the whole of a file: its namespace, or all it exports.
pub const EVERY: &str = "*";

/// The name of a file's default export.
pub const DEFAULT: &str = "default";

/// How an import is written (spec IMP-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// `import … from '…'`, or `import '…'`.
    Import,
    /// `export … from '…'`.
    Export,
    /// `export * from '…'`, and `export * as name from '…'`.
    ExportAll,
    /// `import name = require('…')`.
    ImportEquals,
    /// `import('…')` with a literal.
    Dynamic,
    /// `require('…')` with a literal, where `require` is the bare name.
    Require,
    /// `import('…')` written where a type is expected.
    TypeImport,
    /// The `src` of a Vue component's script block, or of an HTML page's script element.
    Src,
}

impl Form {
    /// The form's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Export => "export",
            Self::ExportAll => "export-all",
            Self::ImportEquals => "import-equals",
            Self::Dynamic => "dynamic",
            Self::Require => "require",
            Self::TypeImport => "type-import",
            Self::Src => "src",
        }
    }
}

/// One name an import brings in (spec USE-1).
#[derive(Debug, PartialEq, Eq)]
pub struct Name {
    /// The export of the target it names: the export's name, `default`, or `*` for the namespace.
    pub name: String,
    /// The binding it makes in the importing file, where it makes one.
    pub local: Option<String>,
    /// The names the importing file offers it under again, each of which passes a use on to the target
    /// (spec USE-2, USE-2a).
    pub exported: Vec<String>,
}

/// One use an import makes of its target (spec USE-2).
#[derive(Debug, PartialEq, Eq)]
pub struct Used {
    /// The export used, or none for every export of the target.
    pub name: Option<String>,
    /// The line the use counts from, counted from one.
    pub line: usize,
}

/// One import.
#[derive(Debug, PartialEq, Eq)]
pub struct Import {
    /// The specifier as it is written.
    pub specifier: String,
    /// How it is written.
    pub form: Form,
    /// Whether it brings in types alone: `import type`, `export type`, or a type import.
    pub type_only: bool,
    /// The line it begins on, counted from one.
    pub line: usize,
    /// The names it brings in, in the order they are written (spec USE-1).
    pub names: Vec<Name>,
    /// What of its target the importing file uses through it (spec USE-2).
    pub uses: Vec<Used>,
}

impl Import {
    /// Whether the specifier carries a query or a fragment: a `?`, or a `#` that is not its first
    /// character (spec IMP-4).
    pub fn has_query(&self) -> bool {
        self.specifier.contains('?')
            || self
                .specifier
                .get(1..)
                .is_some_and(|rest| rest.contains('#'))
    }
}

/// One name a file's own `export` statements offer, declared in the file (spec USE-1).
#[derive(Debug, PartialEq, Eq)]
pub struct Export {
    /// Its name; `default` for the default export.
    pub name: String,
    /// Whether it is a type, an interface or a type alias or a namespace of types alone, and not a
    /// value.
    pub is_type: bool,
    /// The line its name is on, counted from one.
    pub line: usize,
    /// The column its name begins at, counted from one in UTF-16 code units.
    pub column: usize,
    /// Whether the file itself refers to it, apart from exporting it (spec USE-2).
    pub used_in_file: bool,
}

/// What reading a script found.
#[derive(Debug, Default)]
pub struct Read {
    /// Its imports, in the order they are written.
    pub imports: Vec<Import>,
    /// How many calls of `import(…)` or `require(…)` have an argument that cannot be followed
    /// (spec IMP-2).
    pub computed: usize,
    /// The names its own `export` statements offer, in the order they are written (spec USE-1).
    pub exports: Vec<Export>,
    /// The uses a file makes of its own exports from outside its scripts, as a Vue component's
    /// template does (spec USE-4).
    pub named: Vec<Used>,
}

/// Why a script did not parse: the line of the first syntax error, where there is one, and its message.
#[derive(Debug)]
pub struct Broken {
    /// The line, counted from one.
    pub line: Option<usize>,
    /// What the parser said.
    pub message: String,
}

/// Parses a script, as a module first and then as a script, and finds its imports and its names (spec
/// PARSE-1).
pub fn read(path: &str, text: &str) -> Result<Read, Broken> {
    read_as(text, source_type(path), false)
}

/// Parses a Vue component's script block in the language its `lang` names: `ts`, `tsx`, `js` or
/// `jsx`, and JavaScript when it names none; nothing for a block in any other language (spec PARSE-3).
pub fn read_block(text: &str, lang: Option<&str>) -> Option<Result<Read, Broken>> {
    let source_type = match lang {
        None | Some("js" | "jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some(_) => return None,
    };

    Some(read_as(text, source_type, true))
}

/// Parses statements of an MDX document as a JavaScript module and keeps the `import` and `export …
/// from` statements among them, and nothing else (spec PARSE-4).
pub fn read_statements(text: &str) -> Result<Vec<Import>, Broken> {
    parse(text, SourceType::jsx().with_module(true), false).map(|read| {
        read.imports
            .into_iter()
            .filter(|import| matches!(import.form, Form::Import | Form::Export | Form::ExportAll))
            .collect()
    })
}

/// Parses as a module first and then as a script.
fn read_as(text: &str, source_type: SourceType, component: bool) -> Result<Read, Broken> {
    let module = source_type.with_module(true);

    match parse(text, module, component) {
        Ok(read) => Ok(read),
        Err(broken) => parse(text, module.with_script(true), component).map_err(|_| broken),
    }
}

/// The language of a script, from its name. A type declaration file is TypeScript, and JavaScript allows
/// JSX whatever its ending.
fn source_type(path: &str) -> SourceType {
    let typescript = [".ts", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending));
    let declaration = [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|ending| path.ends_with(ending));

    if path.ends_with(".tsx") {
        SourceType::tsx()
    } else if typescript {
        SourceType::ts().with_typescript_definition(declaration)
    } else {
        SourceType::jsx()
    }
}

/// Parses once with the source type given. In a Vue component's block, the default export is the
/// component itself, so what it names is used and not passed on (spec USE-4).
fn parse(text: &str, source_type: SourceType, component: bool) -> Result<Read, Broken> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
    let lines = Lines::of(text);

    if let Some(error) = parsed.diagnostics.iter().next() {
        let line = error.labels.first().map(|label| lines.at(label.offset()));

        return Err(Broken {
            line,
            message: error.message.to_string(),
        });
    }

    if parsed.fatal_error {
        return Err(Broken {
            line: None,
            message: "the parser stopped".to_owned(),
        });
    }

    // The binding analysis is oxc's: each reference tied to the binding it names (spec USE-1).
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let mut finder = Finder {
        lines: &lines,
        semantic: &semantic,
        component,
        imports: Vec::new(),
        computed: 0,
        bound: Vec::new(),
    };

    finder.visit_program(&parsed.program);

    let exports = finder.exports(&parsed.program);

    finder.used();

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
        exports,
        named: Vec::new(),
    })
}

/// Where each line of a text begins, to turn an offset into a line and a column.
struct Lines<'t> {
    /// The text.
    text: &'t str,
    /// The offset each line begins at.
    starts: Vec<usize>,
}

impl<'t> Lines<'t> {
    /// The starts of the lines of a text.
    fn of(text: &'t str) -> Self {
        Self {
            text,
            starts: std::iter::once(0)
                .chain(text.match_indices('\n').map(|(at, _)| at + 1))
                .collect(),
        }
    }

    /// The line an offset lies on, counted from one.
    fn at(&self, offset: u32) -> usize {
        let offset = usize::try_from(offset).unwrap_or(usize::MAX);

        self.starts.partition_point(|start| *start <= offset)
    }

    /// The line an offset lies on and its column there, each counted from one, the column in UTF-16
    /// code units, as SARIF counts one.
    fn place(&self, offset: u32) -> (usize, usize) {
        let line = self.at(offset);
        let start = self.starts[line - 1];
        let before = usize::try_from(offset)
            .ok()
            .and_then(|offset| self.text.get(start..offset))
            .unwrap_or_default();

        (line, before.encode_utf16().count() + 1)
    }
}

/// Walks a program and collects its imports, with the names each brings in.
struct Finder<'s, 'a> {
    lines: &'s Lines<'s>,
    semantic: &'s Semantic<'a>,
    component: bool,
    imports: Vec<Import>,
    computed: usize,
    /// The binding each name of an import makes: its symbol, and the places of the import and of the
    /// name in their lists.
    bound: Vec<(SymbolId, usize, usize)>,
}

impl<'a> Finder<'_, 'a> {
    /// Records one import and returns its place in the list.
    fn found(&mut self, specifier: String, form: Form, type_only: bool, span: Span) -> usize {
        self.imports.push(Import {
            specifier,
            form,
            type_only,
            line: self.lines.at(span.start),
            names: Vec::new(),
            uses: Vec::new(),
        });

        self.imports.len() - 1
    }

    /// Records a name the import at the place given brings in, with the binding it makes, or the
    /// name an `export … from` offers it under.
    fn brings(
        &mut self,
        import: usize,
        name: String,
        local: Option<&BindingIdentifier<'a>>,
        exported: Option<String>,
    ) {
        let names = &mut self.imports[import].names;

        if let Some(local) = local {
            self.bound.push((local.symbol_id(), import, names.len()));
        }

        names.push(Name {
            name,
            local: local.map(|local| local.name.to_string()),
            exported: exported.into_iter().collect(),
        });
    }

    /// Records that the import at the place given uses every export of its target, from its own line.
    fn uses_every(&mut self, import: usize) {
        let line = self.imports[import].line;

        self.imports[import].uses.push(Used { name: None, line });
    }

    /// The text of a string literal, as JavaScript reads it.
    fn text(literal: &StringLiteral<'_>) -> String {
        literal.value.display().to_string()
    }

    /// The text of an argument that is a string or a template with no substitution, or nothing.
    fn literal(expression: &Expression<'_>) -> Option<String> {
        match expression {
            Expression::StringLiteral(literal) => Some(Self::text(literal)),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
                .quasis
                .first()
                .map(|quasi| match quasi.value.cooked {
                    Some(cooked) => cooked.display().to_string(),
                    None => quasi.value.raw.to_string(),
                }),
            _ => None,
        }
    }

    /// The name an `import` or `export` statement writes for an export.
    fn written(name: &ModuleExportName<'_>) -> String {
        match name {
            ModuleExportName::IdentifierName(identifier) => identifier.name.to_string(),
            ModuleExportName::IdentifierReference(identifier) => identifier.name.to_string(),
            ModuleExportName::StringLiteral(literal) => Self::text(literal),
        }
    }

    /// The import and the name a symbol is the binding of, where it is one.
    fn binding(&self, symbol: SymbolId) -> Option<(usize, usize)> {
        self.bound
            .iter()
            .find(|(bound, _, _)| *bound == symbol)
            .map(|(_, import, name)| (*import, *name))
    }

    /// Whether a reference is only the naming of a binding in an `export` statement, which passes a
    /// use on and is not one (spec USE-2). A component's default export is the component, so naming a
    /// binding there uses it (spec USE-4).
    fn exporting(&self, reference: NodeId) -> bool {
        match self.semantic.nodes().parent_kind(reference) {
            AstKind::ExportSpecifier(_) => true,
            AstKind::ExportDefaultDeclaration(_) => !self.component,
            _ => false,
        }
    }

    /// The names the script's own `export` statements offer, each once. A name that is an import's
    /// binding is passed on, and recorded on the import (spec USE-1, USE-2a). Only the statements at
    /// the top of the script are a module's exports.
    fn exports(&mut self, program: &Program<'a>) -> Vec<Export> {
        let mut found: Vec<Export> = Vec::new();

        for statement in &program.body {
            match statement {
                Statement::ExportDeclaration(export) => {
                    let mut declared = Vec::new();

                    Self::declared(&export.declaration, &mut declared);

                    for identifier in declared {
                        self.offered(
                            identifier.name.to_string(),
                            identifier.span,
                            Some(identifier.symbol_id()),
                            &mut found,
                        );
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    for specifier in &export.specifiers {
                        if let ModuleExportName::IdentifierReference(local) = &specifier.local {
                            let symbol = self
                                .semantic
                                .scoping()
                                .get_reference(local.reference_id())
                                .symbol_id();

                            self.offered(
                                Self::written(&specifier.exported),
                                specifier.exported.span(),
                                symbol,
                                &mut found,
                            );
                        }
                    }
                }
                Statement::ExportDefaultDeclaration(export) => {
                    let symbol = match &export.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                            function.id.as_ref().map(BindingIdentifier::symbol_id)
                        }
                        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                            class.id.as_ref().map(BindingIdentifier::symbol_id)
                        }
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
                            Some(interface.id.symbol_id())
                        }
                        ExportDefaultDeclarationKind::Identifier(identifier) => self
                            .semantic
                            .scoping()
                            .get_reference(identifier.reference_id())
                            .symbol_id(),
                        _ => None,
                    };

                    self.offered(
                        DEFAULT.to_owned(),
                        export.declaration.span(),
                        symbol,
                        &mut found,
                    );
                }
                _ => {}
            }
        }

        found
    }

    /// The bindings a declaration makes, which an `export` before it offers by their names.
    fn declared<'d>(declaration: &'d Declaration<'a>, found: &mut Vec<&'d BindingIdentifier<'a>>) {
        match declaration {
            Declaration::VariableDeclaration(variables) => {
                for declarator in &variables.declarations {
                    Self::patterned(&declarator.id, found);
                }
            }
            Declaration::FunctionDeclaration(function) => found.extend(&function.id),
            Declaration::ClassDeclaration(class) => found.extend(&class.id),
            Declaration::TSTypeAliasDeclaration(alias) => found.push(&alias.id),
            Declaration::TSInterfaceDeclaration(interface) => found.push(&interface.id),
            Declaration::TSEnumDeclaration(enumeration) => found.push(&enumeration.id),
            Declaration::TSNamespaceDeclaration(namespace) => found.push(&namespace.id),
            Declaration::TSImportEqualsDeclaration(import) => found.push(&import.id),
            Declaration::TSExternalModuleDeclaration(_) | Declaration::TSGlobalDeclaration(_) => {}
        }
    }

    /// The bindings a pattern makes: a name, or each name a destructuring holds.
    fn patterned<'d>(pattern: &'d BindingPattern<'a>, found: &mut Vec<&'d BindingIdentifier<'a>>) {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => found.push(identifier),
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    Self::patterned(&property.value, found);
                }

                if let Some(rest) = &object.rest {
                    Self::patterned(&rest.argument, found);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    Self::patterned(element, found);
                }

                if let Some(rest) = &array.rest {
                    Self::patterned(&rest.argument, found);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                Self::patterned(&assignment.left, found);
            }
        }
    }

    /// Records one name an `export` statement offers. The binding of an import is passed on under the
    /// name; anything else is the script's own export, a value unless its binding is a type alone, and
    /// one export when the name is offered more than once, as overloads and merged declarations are.
    fn offered(
        &mut self,
        name: String,
        span: Span,
        symbol: Option<SymbolId>,
        found: &mut Vec<Export>,
    ) {
        let passed = symbol
            .and_then(|symbol| self.binding(symbol))
            .filter(|_| !(self.component && name == DEFAULT));

        if let Some((import, bound)) = passed {
            self.imports[import].names[bound].exported.push(name);

            return;
        }

        let scoping = self.semantic.scoping();
        let is_type = symbol.is_some_and(|symbol| !scoping.symbol_flags(symbol).is_value());
        let used_in_file = symbol.is_some_and(|symbol| {
            scoping
                .get_resolved_references(symbol)
                .any(|reference| !self.exporting(reference.node_id()))
        });

        match found.iter_mut().find(|export| export.name == name) {
            Some(export) => {
                export.is_type &= is_type;
                export.used_in_file |= used_in_file;
            }
            None => {
                let (line, column) = self.lines.place(span.start);

                found.push(Export {
                    name,
                    is_type,
                    line,
                    column,
                    used_in_file,
                });
            }
        }
    }

    /// Records what each import's bindings are used for: a named or default binding uses its export
    /// where the script refers to it, in code or in a type; a namespace's binding uses the member read
    /// off it, and every export where it is referred to in any other way (spec USE-2).
    fn used(&mut self) {
        let scoping = self.semantic.scoping();
        let nodes = self.semantic.nodes();

        for (symbol, import, name) in self.bound.clone() {
            // `import name = require('…')` uses every export already, whatever is read off the name.
            if self.imports[import].form == Form::ImportEquals {
                continue;
            }

            let imported = self.imports[import].names[name].name.clone();
            let mut uses: Vec<Used> = Vec::new();

            for reference in scoping.get_resolved_references(symbol) {
                let node = reference.node_id();

                if self.exporting(node) {
                    continue;
                }

                let used = if imported == EVERY {
                    member(nodes.parent_kind(node), node)
                } else {
                    Some(imported.clone())
                };

                if uses.iter().all(|earlier| earlier.name != used) {
                    uses.push(Used {
                        name: used,
                        line: self.lines.at(nodes.kind(node).span().start),
                    });
                }
            }

            self.imports[import].uses.extend(uses);
        }
    }
}

/// The member a reference to a namespace reads off it, where its parent reads one by a name the
/// script writes: `ns.name`, `ns["name"]`, `ns.Name` in a type, or `<ns.Name />` (spec USE-2).
fn member(parent: AstKind<'_>, reference: NodeId) -> Option<String> {
    match parent {
        AstKind::StaticMemberExpression(member) => {
            is_reference(&member.object, reference).then(|| member.property.name.to_string())
        }
        AstKind::ComputedMemberExpression(member) => is_reference(&member.object, reference)
            .then(|| Finder::literal(&member.expression))
            .flatten(),
        AstKind::TSQualifiedName(qualified) => match &qualified.left {
            TSTypeName::IdentifierReference(identifier)
                if identifier.node_id.get() == reference =>
            {
                Some(qualified.right.name.to_string())
            }
            _ => None,
        },
        AstKind::JSXMemberExpression(member) => match &member.object {
            JSXMemberExpressionObject::IdentifierReference(identifier)
                if identifier.node_id.get() == reference =>
            {
                Some(member.property.name.to_string())
            }
            _ => None,
        },
        _ => None,
    }
}

/// Whether an expression is the reference with the node given.
fn is_reference(expression: &Expression<'_>, reference: NodeId) -> bool {
    matches!(expression, Expression::Identifier(identifier) if identifier.node_id.get() == reference)
}

impl<'a> Visit<'a> for Finder<'_, 'a> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        let import = self.found(
            Self::text(&declaration.source),
            Form::Import,
            declaration.import_kind.is_type(),
            declaration.span,
        );

        for specifier in declaration.specifiers.iter().flatten() {
            let name = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    Self::written(&specifier.imported)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => DEFAULT.to_owned(),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => EVERY.to_owned(),
            };

            self.brings(import, name, Some(specifier.local()), None);
        }

        walk::walk_import_declaration(self, declaration);
    }

    fn visit_export_from_declaration(&mut self, declaration: &ExportFromDeclaration<'a>) {
        let import = self.found(
            Self::text(&declaration.source),
            Form::Export,
            declaration.export_kind.is_type(),
            declaration.span,
        );

        for specifier in &declaration.specifiers {
            self.brings(
                import,
                Self::written(&specifier.local),
                None,
                Some(Self::written(&specifier.exported)),
            );
        }

        walk::walk_export_from_declaration(self, declaration);
    }

    fn visit_export_all_declaration(&mut self, declaration: &ExportAllDeclaration<'a>) {
        let import = self.found(
            Self::text(&declaration.source),
            Form::ExportAll,
            declaration.export_kind.is_type(),
            declaration.span,
        );

        self.brings(
            import,
            EVERY.to_owned(),
            None,
            declaration.exported.as_ref().map(Self::written),
        );

        walk::walk_export_all_declaration(self, declaration);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        if let TSModuleReference::ExternalModuleReference(reference) = &declaration.module_reference
        {
            let import = self.found(
                Self::text(&reference.expression),
                Form::ImportEquals,
                declaration.import_kind.is_type(),
                declaration.span,
            );

            self.brings(import, EVERY.to_owned(), Some(&declaration.id), None);
            self.uses_every(import);
        }

        walk::walk_ts_import_equals_declaration(self, declaration);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        match Self::literal(&expression.source) {
            Some(specifier) => {
                let import = self.found(specifier, Form::Dynamic, false, expression.span);

                self.uses_every(import);
            }
            None => self.computed += 1,
        }

        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && callee.name == "require"
        {
            match call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(Self::literal)
            {
                Some(specifier) => {
                    let import = self.found(specifier, Form::Require, false, call.span);

                    self.uses_every(import);
                }
                None => self.computed += 1,
            }
        }

        walk::walk_call_expression(self, call);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        let found = self.found(
            Self::text(&import.source),
            Form::TypeImport,
            true,
            import.span,
        );
        // `import('./x').A.B` reads `A` off the file; with nothing read off it, it is the whole file.
        let mut qualifier = import.qualifier.as_ref();

        while let Some(TSImportTypeQualifier::QualifiedName(qualified)) = qualifier {
            qualifier = Some(&qualified.left);
        }

        match qualifier {
            Some(TSImportTypeQualifier::Identifier(identifier)) => {
                let line = self.imports[found].line;

                self.brings(found, identifier.name.to_string(), None, None);
                self.imports[found].uses.push(Used {
                    name: Some(identifier.name.to_string()),
                    line,
                });
            }
            _ => self.uses_every(found),
        }

        walk::walk_ts_import_type(self, import);
    }
}
```

What a template and a document's text name:

```rust edit path=src/pages.rs
<<<<<<< find
//! Each reader is the analyzer's own and small. It finds only what the analyzer needs from its format,
//! and the scripts it finds are parsed as any script is.

use std::ops::Range;

use crate::imports::{self, Broken, Form, Import, Read};

/// The elements of an HTML page whose content is text a browser does not read as markup.
const RAW_TEXT: [&str; 9] = [
=======
//! Each reader is the analyzer's own and small. It finds only what the analyzer needs from its format,
//! and the scripts it finds are parsed as any script is.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::imports::{self, Broken, DEFAULT, EVERY, Form, Import, Name, Read, Used};

/// The elements of an HTML page whose content is text a browser does not read as markup.
const RAW_TEXT: [&str; 9] = [
>>>>>>> replace
<<<<<<< find
    Statement { line: usize },
}

/// The script blocks of a Vue component: each block's `src` and the imports of its content, in the
/// language its `lang` names; a block in another language is passed over. A block that does not parse
/// makes the component not parse, at the block's line in the file (spec PARSE-3).
pub fn vue(text: &str) -> Result<(Read, Vec<Skipped>), Broken> {
    let mut read = Read {
        imports: Vec::new(),
        computed: 0,
    };
    let mut skipped = Vec::new();

    for block in blocks(text)
        .into_iter()
        .filter(|block| block.tag.name == "script")
    {
        let line = line_at(text, block.start);

        if let Some(src) = block.tag.attribute("src") {
=======
    Statement { line: usize },
}

/// The script blocks of a Vue component: each block's `src` and the imports and names of its content,
/// in the language its `lang` names; a block in another language is passed over. A block that does not
/// parse makes the component not parse, at the block's line in the file (spec PARSE-3).
///
/// The component's default export is the component, which is never reported, so it is not among its
/// exports. A name its template holds uses the export of that name, and the binding of that name that
/// an import of a `<script setup>` block makes (spec USE-4).
pub fn vue(text: &str) -> Result<(Read, Vec<Skipped>), Broken> {
    let mut read = Read::default();
    let mut skipped = Vec::new();
    let blocks = blocks(text);
    let mut named: BTreeMap<String, usize> = BTreeMap::new();

    for block in blocks.iter().filter(|block| block.tag.name == "template") {
        words(
            &text[block.content.clone()],
            line_at(text, block.content.start),
            true,
            &mut named,
        );
    }

    for block in blocks.iter().filter(|block| block.tag.name == "script") {
        let line = line_at(text, block.start);

        if let Some(src) = block.tag.attribute("src") {
>>>>>>> replace
<<<<<<< find
                language: lang.unwrap_or_default().to_owned(),
            }),
            Some(Ok(found)) => {
                read.imports
                    .extend(found.imports.into_iter().map(|import| Import {
                        line: import.line + above,
                        ..import
                    }));
                read.computed += found.computed;
            }
            Some(Err(broken)) => {
                return Err(Broken {
=======
                language: lang.unwrap_or_default().to_owned(),
            }),
            Some(Ok(found)) => {
                let setup = block.tag.attribute("setup").is_some();

                read.imports.extend(found.imports.into_iter().map(|import| {
                    let mut import = moved(import, above);

                    if setup {
                        name_bindings(&mut import, &named);
                    }

                    import
                }));
                read.computed += found.computed;

                for export in found.exports {
                    if export.name != DEFAULT
                        && read
                            .exports
                            .iter()
                            .all(|earlier| earlier.name != export.name)
                    {
                        read.exports.push(imports::Export {
                            line: export.line + above,
                            ..export
                        });
                    }
                }
            }
            Some(Err(broken)) => {
                return Err(Broken {
>>>>>>> replace
<<<<<<< find
        }
    }

    Ok((read, skipped))
}

/// The `import` and `export … from` statements of an MDX document, and nothing else (spec PARSE-4). A
/// statement begins at the start of a line with `import` or `export`, outside the front matter and
/// outside a fenced code block, and runs to the next blank line. One that does not parse is text.
pub fn mdx(text: &str) -> (Read, Vec<Skipped>) {
    let lines: Vec<(usize, &str)> = text
        .split_inclusive('\n')
=======
        }
    }

    read.named = read
        .exports
        .iter()
        .filter_map(|export| {
            named.get(&export.name).map(|line| Used {
                name: Some(export.name.clone()),
                line: *line,
            })
        })
        .collect();

    Ok((read, skipped))
}

/// An import of a block, with its lines counted in the file: the lines above the block are added.
fn moved(import: Import, above: usize) -> Import {
    Import {
        line: import.line + above,
        uses: import
            .uses
            .into_iter()
            .map(|used| Used {
                line: used.line + above,
                ..used
            })
            .collect(),
        ..import
    }
}

/// Records, for each binding of an import that a text names, the use the text makes of it, from the
/// line of the name's first place in the text, unless a script already uses it (spec USE-4, USE-5).
fn name_bindings(import: &mut Import, named: &BTreeMap<String, usize>) {
    for Name { name, local, .. } in &import.names {
        let Some(line) = local.as_ref().and_then(|local| named.get(local)) else {
            continue;
        };
        let used = (name != EVERY).then(|| name.clone());

        if import.uses.iter().all(|earlier| earlier.name != used) {
            import.uses.push(Used {
                name: used,
                line: *line,
            });
        }
    }
}

/// Adds each identifier-shaped word of a text to the words given, with the line of its first place;
/// `first` is the line the text begins on. With `tags`, a word written with hyphens is also added in
/// camelCase and in PascalCase, as a template names a component by its tag (spec USE-4). This is a
/// scan and not a compiler: it finds more names than a compiler would, never fewer.
fn words(text: &str, first: usize, tags: bool, found: &mut BTreeMap<String, usize>) {
    let is_word = |each: char| each.is_alphanumeric() || each == '_' || each == '$';
    let mut add = |word: String, line: usize| {
        if word.chars().next().is_some_and(|start| !start.is_numeric()) {
            found.entry(word).or_insert(line);
        }
    };

    for (index, line) in text.split('\n').enumerate() {
        for chain in line.split(|each: char| !(is_word(each) || each == '-')) {
            let parts: Vec<&str> = chain.split('-').filter(|part| !part.is_empty()).collect();

            for part in &parts {
                add((*part).to_owned(), first + index);
            }

            if tags && parts.len() > 1 {
                let pascal: String = parts.iter().map(|part| capitalized(part)).collect();
                let camel = parts[0].to_owned()
                    + &parts[1..]
                        .iter()
                        .map(|part| capitalized(part))
                        .collect::<String>();

                add(pascal, first + index);
                add(camel, first + index);
            }
        }
    }
}

/// A word with its first letter in upper case.
fn capitalized(word: &str) -> String {
    let mut letters = word.chars();

    letters.next().map_or_else(String::new, |start| {
        start.to_uppercase().chain(letters).collect()
    })
}

/// The `import` and `export … from` statements of an MDX document, and nothing else (spec PARSE-4). A
/// statement begins at the start of a line with `import` or `export`, outside the front matter and
/// outside a fenced code block, and runs to the next blank line. One that does not parse is text.
///
/// A binding an import makes is used where the document's text names it, outside its statements and
/// its fenced code blocks (spec USE-5).
pub fn mdx(text: &str) -> (Read, Vec<Skipped>) {
    let lines: Vec<(usize, &str)> = text
        .split_inclusive('\n')
>>>>>>> replace
<<<<<<< find
    let mut skipped = Vec::new();
    let mut index = front_matter(&lines);
    let mut fence: Option<(char, usize)> = None;

    while let Some((start, line)) = lines.get(index).copied() {
        if let Some((mark, size)) = fence {
=======
    let mut skipped = Vec::new();
    let mut index = front_matter(&lines);
    let mut fence: Option<(char, usize)> = None;
    let mut named: BTreeMap<String, usize> = BTreeMap::new();

    while let Some((start, line)) = lines.get(index).copied() {
        if let Some((mark, size)) = fence {
>>>>>>> replace
<<<<<<< find
        }

        if !begins_statement(line) {
            index += 1;

            continue;
=======
        }

        if !begins_statement(line) {
            words(line, index + 1, false, &mut named);
            index += 1;

            continue;
>>>>>>> replace
<<<<<<< find
        let stop = lines.get(end).map_or(text.len(), |(at, _)| *at);

        match imports::read_statements(&text[start..stop]) {
            Ok(found) => imports.extend(found.into_iter().map(|import| Import {
                line: import.line + index,
                ..import
            })),
            Err(_) => skipped.push(Skipped::Statement { line: index + 1 }),
        }

        index = end;
    }

    (
        Read {
            imports,
            computed: 0,
        },
        skipped,
    )
=======
        let stop = lines.get(end).map_or(text.len(), |(at, _)| *at);

        match imports::read_statements(&text[start..stop]) {
            Ok(found) => imports.extend(found.into_iter().map(|import| moved(import, index))),
            Err(_) => {
                skipped.push(Skipped::Statement { line: index + 1 });

                // A statement that does not parse is text, so its words are the text's.
                for (offset, (_, line)) in lines[index..end].iter().enumerate() {
                    words(line, index + offset + 1, false, &mut named);
                }
            }
        }

        index = end;
    }

    for import in &mut imports {
        name_bindings(import, &named);
    }

    (
        Read {
            imports,
            ..Read::default()
        },
        skipped,
    )
>>>>>>> replace
<<<<<<< find

    Read {
        imports,
        computed: 0,
    }
}

/// The import a `src` attribute makes.
fn source(src: &str, line: usize) -> Import {
    Import {
        specifier: src.to_owned(),
        form: Form::Src,
        type_only: false,
        line,
    }
}

=======

    Read {
        imports,
        ..Read::default()
    }
}

/// The import a `src` attribute makes. The script it names is the block's or the page's own, so every
/// export of it is used (spec USE-2).
fn source(src: &str, line: usize) -> Import {
    Import {
        specifier: src.to_owned(),
        form: Form::Src,
        type_only: false,
        line,
        names: Vec::new(),
        uses: vec![Used { name: None, line }],
    }
}

>>>>>>> replace
```

The uses, passed on to the file that declares each name, and what the entry points reach:

```rust path=src/uses.rs
//! Uses and reach: which export of which file is used, and from where; and which files the entry
//! points reach (spec USE-2 to USE-6).
//!
//! A use begins at a reference in a file, at an import that uses every export of its target, or at an
//! entry point, whose exports whatever starts it consumes. From there it is passed on through each
//! `export … from`, `export *` and imported binding exported again, to the file that declares the
//! name. Every walk here is a loop over a list of work, so no chain of files is too long for the stack
//! (spec LIFE-3).

use std::collections::{BTreeMap, BTreeSet};

use crate::entries::{Entries, Source};
use crate::imports::{DEFAULT, EVERY, Form, Read};
use crate::resolve::{Outcome, Resolution};
use crate::scripts::Scripts;

/// Where a use counts from (spec CMD-4).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// A reference in a file, or an import that uses every export, at a line.
    Reference { file: String, line: usize },
    /// An entry point, whose exports whatever starts it consumes, with the source that made it one.
    Entry(Source),
}

/// One use of an export.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Use {
    /// The file whose export is used.
    pub file: String,
    /// The export's name, or none for every export of the file.
    pub name: Option<String>,
    /// Where it counts from.
    pub origin: Origin,
}

/// How far a file is reached from the entry points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// By an entry point's source or an import: its own imports are followed.
    Code,
    /// Only by imports that carry a query, which read it as text: it loads nothing (spec USE-6).
    Text,
}

/// The uses of a project and what its entry points reach.
#[derive(Debug, Default)]
pub struct Uses {
    /// Every use that was counted, in order.
    pub list: BTreeSet<Use>,
    /// Each file an entry point reaches, by path, with how far.
    pub reached: BTreeMap<String, Reach>,
}

/// What a file offers under the names it exports.
#[derive(Default)]
struct Offer<'s> {
    /// The names it declares itself.
    own: BTreeSet<&'s str>,
    /// The names it passes on, each with the import it comes through and the target's name for it,
    /// none for the target's namespace.
    passed: BTreeMap<&'s str, Vec<(usize, Option<&'s str>)>>,
    /// The imports whose every name it passes on under the same name: `export * from`.
    stars: Vec<usize>,
}

impl<'s> Offer<'s> {
    /// What a file that was read offers.
    fn of(read: &'s Read) -> Self {
        let mut offer = Self {
            own: read
                .exports
                .iter()
                .map(|export| export.name.as_str())
                .collect(),
            ..Self::default()
        };

        for (index, import) in read.imports.iter().enumerate() {
            for name in &import.names {
                let from = (name.name != EVERY).then_some(name.name.as_str());

                for exported in &name.exported {
                    offer
                        .passed
                        .entry(exported)
                        .or_default()
                        .push((index, from));
                }

                if import.form == Form::ExportAll && name.exported.is_empty() {
                    offer.stars.push(index);
                }
            }
        }

        offer
    }
}

/// What of a file a use is of.
#[derive(Clone, Copy)]
enum Wanted<'s> {
    /// Every export.
    Every,
    /// Every name an `export *` passes on: every export but `default`.
    Starred,
    /// One export, by its name.
    One(&'s str),
}

impl<'s> Wanted<'s> {
    /// What a name passed on is a use of in the file it comes from: that file's name for it, or its
    /// every export where the name is its namespace.
    fn through(from: Option<&'s str>) -> Self {
        from.map_or(Self::Every, Self::One)
    }
}

/// One use still to be followed: the file it reaches, what of it, where it counts from by its place in
/// the list of origins, and whether it reads the file as text.
struct Pending<'s> {
    file: &'s str,
    wanted: Wanted<'s>,
    origin: usize,
    text: bool,
}

/// Finds every use and what the entry points reach.
pub fn find(
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
    entries: &Entries,
) -> Uses {
    Uses {
        list: uses(scripts, resolved, entries),
        reached: reached(scripts, resolved, entries),
    }
}

/// The files an import of a file leads to.
fn targets<'s>(
    resolved: &'s BTreeMap<String, Vec<Resolution>>,
    file: &str,
    import: usize,
) -> &'s [String] {
    match resolved.get(file).and_then(|imports| imports.get(import)) {
        Some(Resolution {
            outcome: Outcome::Files(files),
            ..
        }) => files,
        _ => &[],
    }
}

/// The uses that begin in the files and at the entry points, each with where it counts from: what an
/// import uses of each file it leads to, what a file's own template names, and every export of an
/// entry point (spec USE-2, USE-3, USE-4, USE-6).
fn begun<'s>(
    scripts: &'s Scripts,
    resolved: &'s BTreeMap<String, Vec<Resolution>>,
    entries: &'s Entries,
) -> (Vec<Origin>, Vec<Pending<'s>>) {
    let mut origins: Vec<Origin> = Vec::new();
    let mut work: Vec<Pending<'s>> = Vec::new();

    for (path, read) in &scripts.read {
        for (index, import) in read.imports.iter().enumerate() {
            // A file imported with a query is read as text: its exports are never reported, and it
            // passes nothing on (spec USE-6).
            let text = import.has_query();
            let uses: Vec<(Wanted<'s>, usize)> = if text {
                vec![(Wanted::Every, import.line)]
            } else {
                import
                    .uses
                    .iter()
                    .map(|used| (Wanted::through(used.name.as_deref()), used.line))
                    .collect()
            };

            for (wanted, line) in uses {
                origins.push(Origin::Reference {
                    file: path.clone(),
                    line,
                });

                for file in targets(resolved, path, index) {
                    work.push(Pending {
                        file,
                        wanted,
                        origin: origins.len() - 1,
                        text,
                    });
                }
            }
        }

        for used in &read.named {
            origins.push(Origin::Reference {
                file: path.clone(),
                line: used.line,
            });
            work.push(Pending {
                file: path,
                wanted: Wanted::through(used.name.as_deref()),
                origin: origins.len() - 1,
                text: false,
            });
        }
    }

    for (path, sources) in &entries.points {
        for source in sources {
            origins.push(Origin::Entry(source.clone()));
            work.push(Pending {
                file: path,
                wanted: Wanted::Every,
                origin: origins.len() - 1,
                text: false,
            });
        }
    }

    (origins, work)
}

/// Every use, passed on to the file that declares the name (spec USE-2, USE-2a, USE-3).
fn uses(
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
    entries: &Entries,
) -> BTreeSet<Use> {
    let offers: BTreeMap<&str, Offer<'_>> = scripts
        .read
        .iter()
        .map(|(path, read)| (path.as_str(), Offer::of(read)))
        .collect();
    let (origins, mut work) = begun(scripts, resolved, entries);
    // What was counted, and what was only looked through on the way to the names behind an `export *`.
    let mut counted: BTreeSet<(&str, Option<&str>, usize)> = BTreeSet::new();
    let mut looked: BTreeSet<(&str, &str, usize)> = BTreeSet::new();

    while let Some(Pending {
        file,
        wanted,
        origin,
        text,
    }) = work.pop()
    {
        // A file that was not read has no exports.
        let Some(offer) = offers.get(file) else {
            continue;
        };
        // The imports the use is passed on through, each with what it is a use of in the import's
        // target.
        let mut on: Vec<(usize, Wanted<'_>)> = Vec::new();
        let starred = |wanted| offer.stars.iter().map(move |import| (*import, wanted));

        match wanted {
            Wanted::Every => {
                if counted.insert((file, None, origin)) && !text {
                    on.extend(
                        offer
                            .passed
                            .values()
                            .flatten()
                            .map(|(import, from)| (*import, Wanted::through(*from))),
                    );
                    on.extend(starred(Wanted::Starred));
                }
            }
            Wanted::Starred => {
                if looked.insert((file, EVERY, origin)) {
                    for name in offer.own.iter().filter(|name| **name != DEFAULT) {
                        counted.insert((file, Some(name), origin));
                    }

                    for (name, passed) in &offer.passed {
                        if *name != DEFAULT && counted.insert((file, Some(name), origin)) {
                            on.extend(
                                passed
                                    .iter()
                                    .map(|(import, from)| (*import, Wanted::through(*from))),
                            );
                        }
                    }

                    on.extend(starred(Wanted::Starred));
                }
            }
            Wanted::One(name) => {
                if offer.own.contains(name) {
                    counted.insert((file, Some(name), origin));
                } else if let Some(passed) = offer.passed.get(name) {
                    if counted.insert((file, Some(name), origin)) {
                        on.extend(
                            passed
                                .iter()
                                .map(|(import, from)| (*import, Wanted::through(*from))),
                        );
                    }
                } else if name != DEFAULT && looked.insert((file, name, origin)) {
                    // `export *` passes every name on but `default`.
                    on.extend(starred(Wanted::One(name)));
                }
            }
        }

        for (import, wanted) in on {
            for target in targets(resolved, file, import) {
                work.push(Pending {
                    file: target,
                    wanted,
                    origin,
                    text: false,
                });
            }
        }
    }

    counted
        .into_iter()
        .map(|(file, name, origin)| Use {
            file: file.to_owned(),
            name: name.map(str::to_owned),
            origin: origins[origin].clone(),
        })
        .collect()
}

/// The files the entry points reach, through every import that leads to files, of every form. A file
/// that only imports with a query reach is read as text, and its own imports are not followed (spec
/// USE-6).
fn reached(
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
    entries: &Entries,
) -> BTreeMap<String, Reach> {
    let mut reached: BTreeMap<String, Reach> = BTreeMap::new();
    let mut work: Vec<&str> = Vec::new();

    for path in entries.points.keys() {
        reached.insert(path.clone(), Reach::Code);
        work.push(path);
    }

    while let Some(path) = work.pop() {
        let Some(read) = scripts.read.get(path) else {
            continue;
        };

        for (index, import) in read.imports.iter().enumerate() {
            let reach = if import.has_query() {
                Reach::Text
            } else {
                Reach::Code
            };

            for target in targets(resolved, path, index) {
                match (reached.get(target), reach) {
                    (Some(Reach::Code), _) | (Some(Reach::Text), Reach::Text) => {}
                    (_, Reach::Code) => {
                        reached.insert(target.clone(), Reach::Code);
                        work.push(target);
                    }
                    (None, Reach::Text) => {
                        reached.insert(target.clone(), Reach::Text);
                    }
                }
            }
        }
    }

    reached
}
```

`graph` prints them:

```rust edit path=src/graph.rs
<<<<<<< find
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports of every script, where each import leads, and the entry
//! points with the source of each (spec CMD-4).

use std::collections::BTreeMap;
use std::env;
=======
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports and the names of every script, where each import leads,
//! the entry points with the source of each, every use of an export, and the files the entry points
//! reach (spec CMD-4).

use std::collections::BTreeMap;
use std::env;
>>>>>>> replace
<<<<<<< find
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::Import;
use crate::marks::{self, Marks};
use crate::notice::{self, Notice};
use crate::packages::{self, Package, Packages};
=======
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::{EVERY, Export, Import, Name};
use crate::marks::{self, Marks};
use crate::notice::{self, Notice};
use crate::packages::{self, Package, Packages};
>>>>>>> replace
<<<<<<< find
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
=======
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;
use crate::uses::{self, Origin, Reach, Use, Uses};

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
>>>>>>> replace
<<<<<<< find
        },
        &mut notices,
    )?;
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
=======
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
>>>>>>> replace
<<<<<<< find
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    // An object's keys in order, whatever order they were added in.
=======
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
        "reachable": reachable(&uses),
        "uses": uses.list.iter().map(used).collect::<Vec<Value>>(),
    });

    // An object's keys in order, whatever order they were added in.
>>>>>>> replace
<<<<<<< find
        .collect()
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, and the
/// count of those that cannot be followed.
fn file(
    file: &File,
    marks: &Marks,
=======
        .collect()
}

/// The files the entry points reach, by path, each with whether it is reached only as text, by
/// imports that carry a query (spec USE-6).
fn reachable(uses: &Uses) -> Vec<Value> {
    uses.reached
        .iter()
        .map(|(path, reach)| json!({ "path": path, "text": *reach == Reach::Text }))
        .collect()
}

/// One use as it is printed: the file whose export is used, the export's name, `*` for every export,
/// and where it counts from, the file and line of a reference or an entry point's source (spec CMD-4).
fn used(used: &Use) -> Value {
    let (by, line, entry) = match &used.origin {
        Origin::Reference { file, line } => (Some(file), Some(line), None),
        Origin::Entry(source) => (None, None, Some(source.printed())),
    };

    json!({
        "by": by,
        "entry": entry,
        "file": used.file,
        "line": line,
        "name": used.name.as_deref().unwrap_or(EVERY),
    })
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, the
/// count of those that cannot be followed, and its own exports.
fn file(
    file: &File,
    marks: &Marks,
>>>>>>> replace
<<<<<<< find
        (scripts.read.get(&file.path), resolved.get(&file.path))
    {
        printed["computedImports"] = json!(read.computed);
        printed["imports"] = read
            .imports
            .iter()
=======
        (scripts.read.get(&file.path), resolved.get(&file.path))
    {
        printed["computedImports"] = json!(read.computed);
        printed["exports"] = read.exports.iter().map(export).collect();
        printed["imports"] = read
            .imports
            .iter()
>>>>>>> replace
<<<<<<< find
    printed
}

/// One import as it is printed, with where it leads and the rule that decided it (spec OUT-1).
fn import(import: &Import, resolution: &Resolution) -> Value {
    let mut printed = json!({
        "form": import.form.name(),
        "line": import.line,
        "query": import.has_query(),
        "rule": resolution.rule.name(),
        "specifier": import.specifier,
=======
    printed
}

/// One export of a file as it is printed: its name, whether it is a value or a type, where its name
/// is written, and whether its own file refers to it (spec USE-1, USE-2).
fn export(export: &Export) -> Value {
    json!({
        "column": export.column,
        "kind": if export.is_type { "type" } else { "value" },
        "line": export.line,
        "name": export.name,
        "usedInFile": export.used_in_file,
    })
}

/// One name an import brings in as it is printed: the target's name for it, `*` for the namespace,
/// its local binding, and the names the importing file passes it on under (spec USE-1, USE-2a).
fn name(name: &Name) -> Value {
    json!({ "exported": name.exported, "local": name.local, "name": name.name })
}

/// One import as it is printed, with where it leads and the rule that decided it (spec OUT-1).
fn import(import: &Import, resolution: &Resolution) -> Value {
    let mut printed = json!({
        "form": import.form.name(),
        "line": import.line,
        "names": import.names.iter().map(name).collect::<Vec<Value>>(),
        "query": import.has_query(),
        "rule": resolution.rule.name(),
        "specifier": import.specifier,
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: so far its hand-off file, its
//! files, its packages, its scripts' imports and where they lead. It reports no findings yet.

mod cli;
mod declarations;
=======
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: so far its hand-off file, its
//! files, its packages, its scripts' imports and where they lead, its entry points, and which export of
//! which file is used. It reports no findings yet.

mod cli;
mod declarations;
>>>>>>> replace
<<<<<<< find
mod stop;
mod tools;
mod tsconfig;

use std::env;
use std::ffi::OsString;
=======
mod stop;
mod tools;
mod tsconfig;
mod uses;

use std::env;
use std::ffi::OsString;
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test imports --test entries --test pages --test file-list --test hand-off
```

Expected: PASS: 16 tests in `imports`, 11 in `entries`, 7 in `pages`, 16 in `file-list`, 10 in
`hand-off`.

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
git add Cargo.toml Cargo.lock src/imports.rs src/pages.rs src/uses.rs src/graph.rs src/main.rs tests/imports.test.rs tests/entries.test.rs tests/pages.test.rs tests/file-list.test.rs tests/hand-off.test.rs
git commit -m "feat(uses): read each file's names and count the uses of its exports"
```

Expected: one commit on `feat/dead-code-3-names-uses`.
