# Stage 2, part 1: following what is read off import() and require(): implementation plan

**Goal:** a file loaded by a literal `import()`, `require()` or `import … = require` has used only the
exports its loader reads off the result, where every use of the result is a form the spec lists, so an
export nothing reads is reported unused (DYN-1 to DYN-4).

**Architecture:** `src/imports.rs` stops counting every export as used the moment it meets a literal
`import()` or `require()`. It notes each one, and once oxc's binding analysis has run it walks up from
the call through the table of nodes: a member read, a destructuring, a binding of the module or of its
promise, an `await`, a `then` callback. What it can follow becomes uses by name; anything else falls back
to every export, as before. `import … = require` is read by the code that already reads a namespace
import. Nothing downstream changes: `src/uses.rs` and the findings take `Import::uses` as they do today.

**Spec:** docs/specs/2026-11-15-dead-code-comparison-design.md

**Implements:** DYN-1, DYN-2, DYN-3, DYN-4

The first of the item's four plans. The spec leaves the split to the builder; this is the session's
call, and each later plan is written and approved after the one before it:

1. **This plan**, DYN-1 to DYN-4, on the item's branch, `feat/dead-code-comparison`, which also holds
   the spec.
2. **CONF-1 to CONF-3**: a configuration file a script names by an option.
3. **The comparison** on the pinned commits: CMP-1 to CMP-7, JDG-1 to JDG-6 with JDG-5a, BRA-1 to
   BRA-3, BAR-1 to BAR-4, and CURE-1 to CURE-5 for each fix the judging turns up.
4. **The owner's decision**: GO-1 to GO-3, FREEZE-1 and FREEZE-2.

The two analyzer changes the spec already names come first, so the comparison judges the analyzer
with them in and doesn't have to judge everything again after them. Each later plan's branch is made
from `main` once the plan before it has merged, as stage 1's plans were.

## Global constraints

- Where what is read off a result can't be followed, every export of the target is used, as the
  stage-1 spec's USE-2 says today: a case this plan reads wrongly is never worse than stage 1 (DYN-2,
  the spec's Risks).
- A use is a reference, decided by `oxc_semantic`, never a match of names (stage-1 USE-1).
- A file loaded this way is still reachable, and an entry point's exports are still used (DYN-4).
- Each form of DYN-1, each case of DYN-2 and DYN-3, and the live check's shape is a made-up fixture,
  failing before the code (DYN-4, CURE-3). Fixtures copy no code from the named repositories
  (CONTRIBUTING.md).
- `graph` prints each use DYN-1 counts at the read, as it prints a namespace member (DYN-4, stage-1
  CMD-4).

## The decisions this plan makes

1. **Where a use counts from.** A name read off the result counts from the line of the read, as a
   namespace member does; every export counts from the line of the `import()` or `require()`, as it
   did. A use of `import … = require` counts from its reference, as a namespace import's does, so
   `console.log(old)` now counts from its own line and no longer from the import's.
2. **What "not used at all" is (DYN-2).** A result is thrown away when the call, or the `await` of
   it, is a statement of its own (`await import('./x');`, `require('./polyfill');`) or the argument of
   `void`. An arrow function's expression body returns its value, so `() => import('./x')` passes it
   on (DYN-2) and uses every export.
3. **Parentheses are seen through; type assertions are not.** `(await import('./x')).a` reads `a`.
   `(await import('./x') as T).a`, `!` and `satisfies` aren't forms DYN-1 lists, so they use every
   export.
4. **A binding of the module or of its promise is recorded as a name of the import,** `*` with the
   binding as its `local`, as a namespace import's is. `graph` prints it under the import's `names`,
   and a `<script setup>` block's template that names it uses every export through the code that
   already does this for imports (stage-1 USE-4, DYN-1's template clause). A `then` callback's
   parameter isn't recorded: a template can't see it.
5. **A binding of the promise is followed only where it is awaited or given a `then`** (DYN-1's fifth
   form). Copying it to another binding (`const q = p`) is another use, and uses every export.
6. **A `then` callback is a function written in place,** an arrow function or a function expression.
   Its first parameter holds the module. `.then(handle)` and `.then()` use every export; a callback
   with no parameter uses nothing; one with only a rest parameter uses every export.
7. **An exported binding** is one an `export` declaration makes (`export const m = await
   import('./x')`), or one any `export` statement names (`export { m }`, `export default m`). The first
   is found from the declaration; the second is a reference whose parent isn't a member read, so it
   uses every export like any other passing-on.
8. **`import x = require('./x')` that nothing refers to uses nothing** (DYN-3: read as `import * as
   x` is). `export import x = require('./x')` passes `x` on as a namespace, as `import * as x` with
   `export { x }` does.
9. **A destructuring's keys.** A name, a string or a template with no substitution, read as JavaScript
   reads it; a computed key that isn't a literal uses every export, as a rest element does.
10. **No new test file.** The fixtures join the tests of their area: `imports` for the forms, `pages`
    for the template, `audit` for the finding the live check missed (stage 1's ruling: a new file needs
    a `[[test]]` entry applied with the code).

## Review focus

- **A lazy-load wrapper** (`() => import('./page')` handed to a router or a `lazy` function) passes
  the result on, so every export of the page is used. This is the miss the spec's Risks names as
  likeliest. The DYN-2 fixture holds the wrapper; plan 3's comparison counts how many misses it leaves.
- **`.then(handle)` with a named callback** uses every export. The DYN-2 fixture holds it.
- **`export import x = require('./x')`** passes `x` on (decision 8). No fixture holds it, and it is
  not new behaviour: it goes through the same code `import * as x` with `export { x }` does.
- **A template word that matches a binding's name inside a comment or an attribute's text** uses every
  export, which is the safe direction (stage-1 USE-4's scan).
- **Optional chaining.** `require('./x')?.a` reads `a`: the member's object is still the call. No
  fixture holds it.
- **A binding redeclared with `var`, or declared in one scope and shadowed in another,** is told apart
  by `oxc_semantic`'s symbols, never by names. No fixture holds it beyond the reassigned `let` of DYN-2.

### Task 1: follow what is read off import(), require() and import … = require

**Implements:** DYN-1, DYN-2, DYN-3, DYN-4

**Test:** `cargo test --release --locked --test imports --test pages --test audit`

**Files:**
- Modify: `src/imports.rs`
- Test: `tests/imports.test.rs`, `tests/pages.test.rs`, `tests/audit.test.rs`

**Interfaces:**
- Consumes: `oxc_semantic`'s `Semantic::nodes` (`parent_id`, `parent_kind`, `kind`) and
  `Scoping::get_resolved_references`; `oxc_ast`'s `AstKind`, `VariableDeclarator`, `ObjectPattern`,
  `UnaryOperator`; the existing `member` and `Finder::literal` helpers; `pages::name_bindings`, which
  already turns a template's word into a use of every export of the import whose `local` it names.
- Produces: `Import::uses` for `Form::Dynamic`, `Form::Require` and `Form::ImportEquals` holding only
  the names read where DYN-1 or DYN-3 can follow them; `Import::names` holding `*` with the binding as
  `local` for a binding of the module or its promise. `uses::find`, `graph` and `audit` read them
  unchanged; plan 3's comparison runs `audit` with them.

- [ ] **Step 1: Write the failing tests**

The `imports` tests: each form of DYN-1 with a used and an unused export, each case of DYN-2, DYN-3,
and the two older tests whose expectations change (a binding of an `import()` or a `require()` is now
one of its names; the old "every export" test becomes the type-import test it also was).

```rust edit path=tests/imports.test.rs
<<<<<<< find
//! script it cannot parse (spec PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-4, RUN-4; dead-code spec USE-1,
//! USE-2, USE-2a, FIX-1, FIX-2).
=======
//! script it cannot parse (spec PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-4, RUN-4; dead-code spec USE-1,
//! USE-2, USE-2a, FIX-1, FIX-2; comparison spec DYN-1 to DYN-4).
>>>>>>> replace
<<<<<<< find
    // A default, a named and a namespace import each make a binding; `export … from` makes none and
    // offers the name again; an imported binding exported again is passed on under each name it is
    // exported by (spec USE-1, USE-2a).
=======
    // A default, a named and a namespace import each make a binding; `export … from` makes none and
    // offers the name again; an imported binding exported again is passed on under each name it is
    // exported by (spec USE-1, USE-2a). A binding that holds what an `import()` or a `require()`
    // loads is one of its names too, so a template can name it (comparison spec DYN-1).
>>>>>>> replace
<<<<<<< find
            json!([name("Deep", None, &[])]),
            json!([]),
            json!([]),
=======
            json!([name("Deep", None, &[])]),
            json!([name("*", Some("lazy"), &[])]),
            json!([name("*", Some("now"), &[])]),
>>>>>>> replace
<<<<<<< find
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
=======
#[test]
fn uses_the_name_a_type_import_reads_and_every_export_where_it_reads_none() {
    let project = Project::new("imports-type-loads");

    project
        .write(
            "src/main.ts",
            "type Picked = import('./types').Picked;\n\
             type Whole = typeof import('./shapes');\n",
        )
        .write(
            "src/types.ts",
            "export type Picked = string;\nexport type Left = number;\n",
        )
        .write("src/shapes.ts", "export const d = 1;\n")
        .commit();

    let document = project.graph();

    // A type import uses the name it reads, and every export where it reads none (spec USE-2).
    assert_eq!(
        uses(&document, "src/types.ts"),
        [("Picked", "src/main.ts", 1)]
    );
    assert_eq!(uses(&document, "src/shapes.ts"), [("*", "src/main.ts", 2)]);
}

#[test]
fn follows_the_names_read_off_what_an_import_call_or_a_require_loads() {
    let project = Project::new("imports-dynamic-reads");
    let three = "export const used = 1;\nexport const also = 2;\nexport const spare = 3;\n";

    project
        .write(
            "src/main.ts",
            "const a = (await import('./member')).used;\n\
             const b = require('./required').used;\n\
             const c = (await import('./keyed'))['used'];\n\
             const { used: d, other = 1, nested: { deep } } = await import('./destructured');\n\
             const { used: e } = require('./destructured-required');\n\
             const held = await import('./held');\n\
             console.log(held.used, held['also']);\n\
             const old = require('./held-required');\n\
             console.log(old.used);\n\
             import('./then').then((loaded) => loaded.used);\n\
             import('./then-destructured').then(({ used }) => used);\n\
             const promise = import('./promise');\n\
             console.log((await promise).used);\n\
             promise.then(({ also }) => also);\n\
             const unreferenced = await import('./unreferenced');\n\
             await import('./side-effect');\n\
             void require('./side-effect-required');\n\
             console.log(a, b, c, d, e, other, deep);\n",
        )
        .write("src/member.ts", three)
        .write("src/required.ts", three)
        .write("src/keyed.ts", three)
        .write(
            "src/destructured.ts",
            "export const used = 1;\nexport const other = 2;\nexport const nested = { deep: 3 };\n\
             export const deep = 4;\nexport const spare = 5;\n",
        )
        .write("src/destructured-required.ts", three)
        .write("src/held.ts", three)
        .write("src/held-required.ts", three)
        .write("src/then.ts", three)
        .write("src/then-destructured.ts", three)
        .write("src/promise.ts", three)
        .write("src/unreferenced.ts", three)
        .write("src/side-effect.ts", three)
        .write("src/side-effect-required.ts", three)
        .commit();

    let document = project.graph();
    let main = "src/main.ts";

    // A member read by a name or a literal key; a destructuring with names only, whose default values
    // and nested patterns use only the top-level name; a binding whose every reference reads a member;
    // a `then` callback; and a binding of the promise that is awaited or given a `then`: each uses only
    // the names read, each from the line of its read (comparison spec DYN-1, DYN-4).
    assert_eq!(uses(&document, "src/member.ts"), [("used", main, 1)]);
    assert_eq!(uses(&document, "src/required.ts"), [("used", main, 2)]);
    assert_eq!(uses(&document, "src/keyed.ts"), [("used", main, 3)]);
    assert_eq!(
        uses(&document, "src/destructured.ts"),
        [("nested", main, 4), ("other", main, 4), ("used", main, 4)]
    );
    assert_eq!(
        uses(&document, "src/destructured-required.ts"),
        [("used", main, 5)]
    );
    assert_eq!(
        uses(&document, "src/held.ts"),
        [("also", main, 7), ("used", main, 7)]
    );
    assert_eq!(uses(&document, "src/held-required.ts"), [("used", main, 9)]);
    assert_eq!(uses(&document, "src/then.ts"), [("used", main, 10)]);
    assert_eq!(
        uses(&document, "src/then-destructured.ts"),
        [("used", main, 11)]
    );
    assert_eq!(
        uses(&document, "src/promise.ts"),
        [("also", main, 14), ("used", main, 13)]
    );
    // A binding nothing refers to, and a result thrown away, use nothing (comparison spec DYN-1,
    // DYN-2).
    assert_eq!(uses(&document, "src/unreferenced.ts"), []);
    assert_eq!(uses(&document, "src/side-effect.ts"), []);
    assert_eq!(uses(&document, "src/side-effect-required.ts"), []);
}

#[test]
fn uses_every_export_where_what_a_call_loads_is_passed_on_or_read_another_way() {
    let project = Project::new("imports-dynamic-escapes");
    let two = "export const used = 1;\nexport const spare = 2;\n";
    let escaped: [(&str, u64); 11] = [
        ("src/exported.ts", 1),
        ("src/spread.ts", 2),
        ("src/computed.ts", 4),
        ("src/rest.ts", 5),
        ("src/reassigned.ts", 6),
        ("src/lazy.ts", 8),
        ("src/off-promise.ts", 9),
        ("src/promised.ts", 10),
        ("src/argument.ts", 12),
        ("src/named.ts", 13),
        ("src/handed.ts", 15),
    ];

    project.write(
        "src/main.ts",
        "export const passed = await import('./exported');\n\
         const spread = { ...(await import('./spread')) };\n\
         const key = 'used';\n\
         const computed = (await import('./computed'))[key];\n\
         const { used, ...rest } = await import('./rest');\n\
         let reassigned = require('./reassigned');\n\
         reassigned = {};\n\
         const lazy = () => import('./lazy');\n\
         const off = import('./off-promise').used;\n\
         const promised = import('./promised');\n\
         console.log(promised.used);\n\
         register(require('./argument'));\n\
         const named = await import('./named');\n\
         export { named };\n\
         import('./handed').then(handle);\n\
         console.log(spread, computed, used, rest, lazy, off);\n",
    );

    for (path, _) in escaped {
        project.write(path, two);
    }

    project.commit();

    let document = project.graph();

    // An exported binding, a spread, a computed key, a rest element, a reassigned binding, a result
    // returned or passed on, a member read straight off a promise, a binding of the promise read in
    // another way, and a `then` given a callback it can't see: each uses every export, from the line of
    // the call (comparison spec DYN-2, DYN-4).
    for (path, line) in escaped {
        assert_eq!(uses(&document, path), [("*", "src/main.ts", line)], "{path}");
    }
}

#[test]
fn reads_an_import_of_require_as_a_namespace_import_is_read() {
    let project = Project::new("imports-equals");

    project
        .write(
            "src/main.ts",
            "import whole = require('./whole');\n\
             import picked = require('./picked');\n\
             import unused = require('./unreferenced');\n\
             type Shape = picked.Shape;\n\
             console.log(whole, picked.used);\n",
        )
        .write("src/whole.ts", "export const used = 1;\n")
        .write(
            "src/picked.ts",
            "export const used = 1;\nexport const spare = 2;\nexport type Shape = string;\n",
        )
        .write("src/unreferenced.ts", "export const used = 1;\n")
        .commit();

    let document = project.graph();

    // `x.name` uses `name`, in code or in a type; any other reference uses every export, from its own
    // line; and a binding nothing refers to uses nothing (comparison spec DYN-3).
    assert_eq!(uses(&document, "src/whole.ts"), [("*", "src/main.ts", 5)]);
    assert_eq!(
        uses(&document, "src/picked.ts"),
        [("Shape", "src/main.ts", 4), ("used", "src/main.ts", 5)]
    );
    assert_eq!(uses(&document, "src/unreferenced.ts"), []);
}
>>>>>>> replace
```

The `pages` test: a `<script setup>` binding of an `import()` that the template names.

```rust edit path=tests/pages.test.rs
<<<<<<< find
//! document's text use (spec PARSE-2 to PARSE-5, IMP-1, RES-7, CFG-2, FIX-2, FIX-4; dead-code spec
//! USE-4, USE-5, FIX-1).
=======
//! document's text use (spec PARSE-2 to PARSE-5, IMP-1, RES-7, CFG-2, FIX-2, FIX-4; dead-code spec
//! USE-4, USE-5, FIX-1; comparison spec DYN-1).
>>>>>>> replace
<<<<<<< find
#[test]
fn an_mdx_document_uses_the_bindings_its_text_names() {
=======
#[test]
fn a_vue_template_that_names_a_binding_of_an_import_call_uses_every_export() {
    let project = Project::new("pages-vue-dynamic");

    project
        .write(
            "src/App.vue",
            "<template>\n\
             \x20 <p>{{ texts.title }}</p>\n\
             </template>\n\
             <script setup lang=\"ts\">\n\
             const texts = await import('./texts');\n\
             const quiet = await import('./quiet');\n\
             console.log(quiet.used);\n\
             </script>\n",
        )
        .write(
            "src/texts.ts",
            "export const title = 't';\nexport const spare = 1;\n",
        )
        .write(
            "src/quiet.ts",
            "export const used = 1;\nexport const spare = 2;\n",
        )
        .commit();

    let document = project.graph();

    // A binding that holds what an `import()` loads passes the module on where the template names it,
    // so every export is used, from the template's line; one the template doesn't name uses only what
    // its references read (comparison spec DYN-1, dead-code spec USE-4).
    assert_eq!(uses(&document, "src/texts.ts"), [("*", "src/App.vue", 2)]);
    assert_eq!(uses(&document, "src/quiet.ts"), [("used", "src/App.vue", 7)]);
}

#[test]
fn an_mdx_document_uses_the_bindings_its_text_names() {
>>>>>>> replace
```

The `audit` test: the live check's shape, made up. A file loaded only by an `import()` whose one read
is a member has two exports used only inside it, which fallow reported and the analyzer didn't.

```rust edit path=tests/audit.test.rs
<<<<<<< find
#[test]
fn reports_no_file_of_a_kind_that_is_never_an_unused_file() {
=======
#[test]
fn reports_the_exports_nothing_reads_off_a_file_an_import_call_loads() {
    let project = Project::new("audit-dynamic-reads");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "main": "./src/cli.ts" }"#,
        )
        .write(
            "src/cli.ts",
            "export async function run(): Promise<number> {\n\
             \x20 return (await import('./features/wait.js')).wait();\n\
             }\n",
        )
        .write(
            "src/features/wait.ts",
            "export const pollEvery = 1000;\n\
             export function settle(): number {\n\
             \x20 return pollEvery;\n\
             }\n\
             export async function wait(): Promise<number> {\n\
             \x20 return settle();\n\
             }\n",
        )
        .commit();

    let run = audit(&project, None);

    // The two exports used only inside their own file are reported, and the one member read off the
    // `import()` is not (comparison spec DYN-1, DYN-4).
    assert_eq!(
        found(&run),
        [
            ("unused-export", "src/features/wait.ts", Some(1)),
            ("unused-export", "src/features/wait.ts", Some(2)),
        ]
    );
}

#[test]
fn reports_no_file_of_a_kind_that_is_never_an_unused_file() {
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```
cargo test --release --locked --test imports --test pages --test audit
```

Expected: FAIL. The five new tests and the changed names test fail: each file a call loads has
`("*", …)` where a name is expected, the bindings of `lazy` and `now` are missing from `names`, and
`audit` reports no unused export in `src/features/wait.ts`. Every other test passes.

- [ ] **Step 3: Follow what is read off each result**

```rust edit path=src/imports.rs
<<<<<<< find
//! What an import is, and what a script's names are: parsing a script and finding every import in it,
//! the names each brings in, what the script exports, and what of each import's target it uses (spec
//! PARSE-1 to PARSE-4, IMP-1 to IMP-4, USE-1, USE-2).
=======
//! What an import is, and what a script's names are: parsing a script and finding every import in it,
//! the names each brings in, what the script exports, and what of each import's target it uses (spec
//! PARSE-1 to PARSE-4, IMP-1 to IMP-4, USE-1, USE-2; comparison spec DYN-1 to DYN-3).
>>>>>>> replace
<<<<<<< find
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, CallExpression, Declaration, ExportAllDeclaration,
    ExportDefaultDeclarationKind, ExportFromDeclaration, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, ImportExpression, JSXMemberExpressionObject, ModuleExportName,
    Program, Statement, StringLiteral, TSImportEqualsDeclaration, TSImportType,
    TSImportTypeQualifier, TSModuleReference, TSTypeName,
};
=======
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, CallExpression, Declaration, ExportAllDeclaration,
    ExportDefaultDeclarationKind, ExportFromDeclaration, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, ImportExpression, JSXMemberExpressionObject, ModuleExportName,
    ObjectPattern, Program, Statement, StringLiteral, TSImportEqualsDeclaration, TSImportType,
    TSImportTypeQualifier, TSModuleReference, TSTypeName, UnaryOperator, VariableDeclarator,
};
>>>>>>> replace
<<<<<<< find
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
=======
    let mut finder = Finder {
        lines: &lines,
        semantic: &semantic,
        component,
        imports: Vec::new(),
        computed: 0,
        bound: Vec::new(),
        loads: Vec::new(),
    };

    finder.visit_program(&parsed.program);

    let exports = finder.exports(&parsed.program);

    finder.followed();
    finder.used();
>>>>>>> replace
<<<<<<< find
    /// The binding each name of an import makes: its symbol, and the places of the import and of the
    /// name in their lists.
    bound: Vec<(SymbolId, usize, usize)>,
}
=======
    /// The binding each name of an import makes: its symbol, and the places of the import and of the
    /// name in their lists.
    bound: Vec<(SymbolId, usize, usize)>,
    /// Each `import()` and `require()` with a literal: its place in the list of imports, its node, and
    /// whether its result is a promise of the module, as an `import()`'s is, rather than the module
    /// (spec DYN-1).
    loads: Vec<(usize, NodeId, bool)>,
}

/// What the uses of what an `import()` or a `require()` loads read off it (spec DYN-1, DYN-2).
#[derive(Default)]
struct Reads {
    /// Each export read by its name, with the offset of the read.
    names: Vec<(String, u32)>,
    /// Whether a use passes the result on, or reads it in a way that can't be followed, so that it
    /// uses every export.
    every: bool,
    /// The bindings that hold the module or its promise, which a Vue component's template may name
    /// (spec USE-4).
    bindings: Vec<String>,
}
>>>>>>> replace
<<<<<<< find
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
=======
    /// Records what each import's bindings are used for: a named or default binding uses its export
    /// where the script refers to it, in code or in a type; a namespace's binding, and the binding of
    /// `import name = require('…')`, uses the member read off it, and every export where it is referred
    /// to in any other way (spec USE-2, DYN-3).
    fn used(&mut self) {
        let scoping = self.semantic.scoping();
        let nodes = self.semantic.nodes();

        for (symbol, import, name) in self.bound.clone() {
            let imported = self.imports[import].names[name].name.clone();
>>>>>>> replace
<<<<<<< find
            self.imports[import].uses.extend(uses);
        }
    }
}
=======
            self.imports[import].uses.extend(uses);
        }
    }

    /// Records what each literal `import()` and `require()` uses of its target: the exports read off
    /// its result where every use of it is a form DYN-1 lists, each from the line of its first read;
    /// nothing where the result is not used; and every export, from the line of the call, otherwise
    /// (spec DYN-1, DYN-2). A binding of the module or of its promise is recorded as a name of the
    /// import, so a template that names it uses every export (spec USE-4).
    fn followed(&mut self) {
        for (import, node, promise) in std::mem::take(&mut self.loads) {
            let mut reads = Reads::default();

            if promise {
                self.promise_at(node, &mut reads);
            } else {
                self.module_at(node, &mut reads);
            }

            for binding in reads.bindings {
                self.imports[import].names.push(Name {
                    name: EVERY.to_owned(),
                    local: Some(binding),
                    exported: Vec::new(),
                });
            }

            if reads.every {
                self.uses_every(import);
                continue;
            }

            for (name, offset) in reads.names {
                let line = self.lines.at(offset);
                let uses = &mut self.imports[import].uses;

                if uses
                    .iter()
                    .all(|earlier| earlier.name.as_deref() != Some(name.as_str()))
                {
                    uses.push(Used {
                        name: Some(name),
                        line,
                    });
                }
            }
        }
    }

    /// What uses the value of the node given: the place of the node that holds it, past any
    /// parentheses around it, and that node's parent, by its id and its kind.
    fn user(&self, node: NodeId) -> (Span, NodeId, AstKind<'a>) {
        let nodes = self.semantic.nodes();
        let mut at = node;

        loop {
            let parent = nodes.parent_id(at);

            match nodes.kind(parent) {
                AstKind::ParenthesizedExpression(_) => at = parent,
                kind => return (nodes.kind(at).span(), parent, kind),
            }
        }
    }

    /// Follows the result of an `import()`, a promise of the module: awaited or given a `then`, held
    /// by a binding, or thrown away (spec DYN-1, DYN-2).
    fn promise_at(&self, node: NodeId, reads: &mut Reads) {
        if self.promised(node, reads) {
            return;
        }

        match self.user(node) {
            (_, declarator, AstKind::VariableDeclarator(variable)) => {
                self.bound(variable, declarator, true, reads);
            }
            (_, _, kind) if discarded(kind) => {}
            _ => reads.every = true,
        }
    }

    /// Follows a promise of the module where the node given is awaited or given a `then`, and says
    /// whether it is. A member read straight off the promise is neither (spec DYN-1).
    fn promised(&self, node: NodeId, reads: &mut Reads) -> bool {
        match self.user(node) {
            (_, awaited, AstKind::AwaitExpression(_)) => self.module_at(awaited, reads),
            (span, member, AstKind::StaticMemberExpression(then))
                if then.object.span() == span && then.property.name == "then" =>
            {
                match self.user(member) {
                    (callee, _, AstKind::CallExpression(call)) if call.callee.span() == callee => {
                        self.callback(call.arguments.first(), reads);
                    }
                    _ => reads.every = true,
                }
            }
            _ => return false,
        }

        true
    }

    /// Follows the module itself, the result of a `require()` or of an awaited `import()`: a member
    /// read by a name or a literal key, a binding, or thrown away (spec DYN-1, DYN-2).
    fn module_at(&self, node: NodeId, reads: &mut Reads) {
        match self.user(node) {
            (span, _, AstKind::StaticMemberExpression(member)) if member.object.span() == span => {
                reads
                    .names
                    .push((member.property.name.to_string(), member.property.span.start));
            }
            (span, _, AstKind::ComputedMemberExpression(member)) if member.object.span() == span => {
                match Self::literal(&member.expression) {
                    Some(name) => reads.names.push((name, member.expression.span().start)),
                    None => reads.every = true,
                }
            }
            (_, declarator, AstKind::VariableDeclarator(variable)) => {
                self.bound(variable, declarator, false, reads);
            }
            (_, _, kind) if discarded(kind) => {}
            _ => reads.every = true,
        }
    }

    /// Follows a declaration's binding of the result: a name that holds the module or its promise, and
    /// is not exported; or, for the module, a destructuring (spec DYN-1, DYN-2).
    fn bound(
        &self,
        variable: &VariableDeclarator<'a>,
        declarator: NodeId,
        promise: bool,
        reads: &mut Reads,
    ) {
        match &variable.id {
            BindingPattern::BindingIdentifier(identifier) => {
                let nodes = self.semantic.nodes();
                let exported = matches!(
                    nodes.parent_kind(nodes.parent_id(declarator)),
                    AstKind::ExportDeclaration(_)
                );

                if exported {
                    reads.every = true;
                    return;
                }

                reads.bindings.push(identifier.name.to_string());
                self.held(identifier, promise, reads);
            }
            BindingPattern::ObjectPattern(object) if !promise => Self::destructured(object, reads),
            _ => reads.every = true,
        }
    }

    /// Follows every reference of a binding: of the module, each must read a member by a name or a
    /// literal key; of its promise, each must await it or give it a `then`. Any other reference passes
    /// it on, and uses every export (spec DYN-1, DYN-2).
    fn held(&self, identifier: &BindingIdentifier<'a>, promise: bool, reads: &mut Reads) {
        let nodes = self.semantic.nodes();

        for reference in self
            .semantic
            .scoping()
            .get_resolved_references(identifier.symbol_id())
        {
            let node = reference.node_id();
            let followed = if promise {
                self.promised(node, reads)
            } else {
                match member(nodes.parent_kind(node), node) {
                    Some(name) => {
                        reads.names.push((name, nodes.kind(node).span().start));
                        true
                    }
                    None => false,
                }
            };

            if !followed {
                reads.every = true;
            }
        }
    }

    /// Follows the callback a `then` on a promise of the module is given, when it is a function
    /// written in place: its first parameter holds the module, as a name or as a destructuring (spec
    /// DYN-1).
    fn callback(&self, argument: Option<&Argument<'a>>, reads: &mut Reads) {
        let parameters = match argument.and_then(Argument::as_expression) {
            Some(Expression::ArrowFunctionExpression(arrow)) => &arrow.params,
            Some(Expression::FunctionExpression(function)) => &function.params,
            _ => {
                reads.every = true;
                return;
            }
        };

        match parameters.items.first().map(|parameter| &parameter.pattern) {
            Some(BindingPattern::BindingIdentifier(identifier)) => {
                self.held(identifier, false, reads);
            }
            Some(BindingPattern::ObjectPattern(object)) => Self::destructured(object, reads),
            Some(_) => reads.every = true,
            None if parameters.rest.is_some() => reads.every = true,
            None => {}
        }
    }

    /// Records the names a destructuring of the module reads: each key at its top, whatever its value's
    /// pattern or default. A rest element, or a key that is not a name or a literal, uses every export
    /// (spec DYN-1, DYN-2).
    fn destructured(object: &ObjectPattern<'_>, reads: &mut Reads) {
        if object.rest.is_some() {
            reads.every = true;
            return;
        }

        for property in &object.properties {
            match property.key.static_name() {
                Some(name) => reads
                    .names
                    .push((name.display().to_string(), property.key.span().start)),
                None => reads.every = true,
            }
        }
    }
}

/// Whether a node of this kind throws away the value it holds: a statement of its own, or `void`
/// (spec DYN-2).
fn discarded(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::ExpressionStatement(_) => true,
        AstKind::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        _ => false,
    }
}
>>>>>>> replace
<<<<<<< find
            self.brings(import, EVERY.to_owned(), Some(&declaration.id), None);
            self.uses_every(import);
        }

        walk::walk_ts_import_equals_declaration(self, declaration);
=======
            self.brings(import, EVERY.to_owned(), Some(&declaration.id), None);
        }

        walk::walk_ts_import_equals_declaration(self, declaration);
>>>>>>> replace
<<<<<<< find
            Some(specifier) => {
                let import = self.found(specifier, Form::Dynamic, false, expression.span);

                self.uses_every(import);
            }
=======
            Some(specifier) => {
                let import = self.found(specifier, Form::Dynamic, false, expression.span);

                self.loads.push((import, expression.node_id.get(), true));
            }
>>>>>>> replace
<<<<<<< find
                Some(specifier) => {
                    let import = self.found(specifier, Form::Require, false, call.span);

                    self.uses_every(import);
                }
=======
                Some(specifier) => {
                    let import = self.found(specifier, Form::Require, false, call.span);

                    self.loads.push((import, call.node_id.get(), false));
                }
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```
cargo test --release --locked --test imports --test pages --test audit
```

Expected: PASS, every test in the three files.

- [ ] **Step 5: Run the gates**

```
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: each exits 0. `cargo test` runs every test file, so a fixture elsewhere that relied on a
`require()` or an `import()` using every export would show here; the session's reading of the other
test files found none whose expected uses change.

- [ ] **Step 6: Commit**

```
yak stack format
git add src/imports.rs tests/imports.test.rs tests/pages.test.rs tests/audit.test.rs
git commit -m "feat(uses): follow what is read off import() and require()"
```

Expected: one commit on `feat/dead-code-comparison`, with the trailer lines the session's attribution
asks for.
