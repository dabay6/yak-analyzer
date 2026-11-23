# Complexity, part 3: fallow's React name, and the comparison with fallow: implementation plan

**Goal:** the analyzer gives a function the React name fallow's walk gives it, so a hook's call adds
to the same functions' scores in both tools (SCORE-10, SCORE-12), and the comparison with fallow on
the named repositories is run and judged, so the owner holds one judged list (VS-1 to VS-7).

**Architecture:** `src/functions.rs`'s scorer keeps one held name, as fallow keeps its
`pending_identity`: a variable declared to a plain name, an object's or a class's key, and
`export default` each hold their name while the walk is beneath them, the first function the walk
reaches takes it, and nothing is held after. This replaces the climb from a function to what
declares it, so FN-4's name path, which the fingerprint reads, does not change. Task 2 is the
owner's check of the judged list, which stays in the item's report folder.

**Spec:** docs/specs/2026-11-21-complexity-findings-design.md

**Implements:** SCORE-4 to SCORE-14, CASE-1, VS-1 to VS-7

The third of the item's three plans, on its own branch, `feat/complexity-3-comparison`, stacked on
plan 2's `feat/complexity-2-audit` (pull request #40), whose pull request goes against that branch
until #40 merges. It builds on SCORE-4's amendment of 2026-11-22, `amendment:1` on the record,
approved by the owner.

## Global constraints

- Nothing committed names a machine or a private repository other than yakshaver: the judged list,
  the hand-off files, fallow's configurations and every run stay in the report folder,
  `.git/yak-reports/2026-11-22-complexity-findings/cmp/`, committed nowhere (VS-1; comparison spec
  JDG-6, CMP-2).
- The React name never changes a function's name path or its fingerprint (SCORE-10; FN-4, FN-5,
  CPX-4).
- Both tools use the limits 20 cyclomatic and 15 cognitive: the hand-off files declare them, and
  fallow's `health.maxCyclomatic` and `health.maxCognitive` are set to them; fallow's
  `health.cyclomaticPolicy` and `thresholdOverrides` are left out of its best-case configuration
  (VS-1, SCORE-11, SCORE-14).
- fallow's CRAP findings are switched off with `--max-crap 1000000` on every `fallow health` and
  `fallow audit` run (VS-1).
- Each false finding or miss becomes a failing fixture before it is fixed (VS-3; design PROOF-5).
- Fixtures are made up and copy no code from fallow or the named repositories (CASE-2).

## The decisions this plan makes

1. **One held name, set and cleared around the five holders.** `Scorer::hold` sets the held name,
   walks, and clears it, whether or not a function took it, which is SCORE-10's "after it nothing
   is held". A computed key holds nothing, and so clears what an outer holder held, as fallow's
   slot does. `enter` takes the held name for each function, and a function's own identifier wins
   over it. A class takes nothing, so a class's methods take their own keys.
2. **An accessor's React name is its key,** with no `get ` or `set `, since the held name comes from
   `key_name` and not from `declared`, which keeps the prefix for FN-4's name path.
3. **A name from a call's string argument is never held,** so it never makes a function a
   component's or a hook's (SCORE-10). The scorer reads no React name from a call at all.
4. **The new fixtures go in one test,** `a_function_takes_the_react_name_fallow_s_walk_holds_for_it`
   in `tests/complexity.test.rs`, beside the React test plan 1 wrote. It holds SCORE-12's seven cases,
   which fail first, and the cases SCORE-12 adds to CASE-1 that plan 1's code already counts as
   SCORE-6 and SCORE-10 say: a class field's value, `React.useState`, a capitalised name from a
   string argument, a destructuring declarator, defaulted, nested and rest properties, and JSX only in
   a nested function. Those pass before the fix and pin it after.
5. **The comparison reuses stage 2's.** Task 2 copies the scripts of the dead-code comparison's
   report folder (`.git/yak-reports/2026-11-16-dead-code-comparison/cmp/scripts/`) into this item's
   `cmp/scripts/`, and changes the matching to complexity findings: the same file and the same start
   line, and the same column where two functions start on one line (VS-2). Where decorators or
   `export` make the two tools place a function differently, the match moves fallow's start to the
   function's first character as the analyzer places it (CPX-2), and the judged list says how many
   matches needed it.
6. **What Task 2 finds is fixed on this branch.** Each false finding or miss of the analyzer's is
   cured here, its fixture failing first, as an amendment with a ruling; a counting rule a cure
   changes is a further amendment of the spec, approved before the judged list is final (SCORE-5).
   One that cannot be cured without a design change is a case left (comparison spec BAR-4).
7. **The counts and the outcome go on the record** as a ruling on Task 2, since a live-check task has
   no test command, as stage 2's comparison did.

## Review focus

- **A held name left over after its holder.** A function after a variable whose value held no
  function, as `const size = 1; f(() => { useState(0); })`, must not take `size`. `hold` clears the
  name after every holder; the test's `wrap({ size: 1 }, …)` and `make(…)` cases pin the clear inside a
  value, and the file's later lines pin it between declarations.
- **A function's parameter default.** `function Panel(a = () => useX())`: the arrow in the default
  takes nothing, since `Panel`'s own identifier took the held name's place when `enter` ran. No
  fixture holds it, and no hook is called in a default in the named repositories that plan 1 found.
- **A computed key inside a held value**, as `const Panel = { [k]: () => { useState(0); } }`: the
  arrow takes nothing, since the computed key held none. Decision 1 follows fallow's slot; Task 2's
  comparison would show it if fallow differs.
- **An `export default` of a class.** Its methods take their own keys, since a class takes nothing
  and each method's key holds its own name.

### Task 1: The React name fallow's walk holds

**Implements:** SCORE-6, SCORE-10, SCORE-12, SCORE-13, CASE-1

**Test:** `cargo test --locked --test complexity`

**Files:**
- Modify: `src/functions.rs`, `tests/complexity.test.rs`

**Interfaces:**
- Consumes: plan 1's `Scorer`, `Frame::react`, `key_name`, `hook` and `DEFAULT`.
- Produces: `Scorer::held` and `Scorer::hold`; the free function `react(name)`; the scorer's
  `visit_variable_declarator`, `visit_object_property`, `visit_method_definition`,
  `visit_property_definition` and `visit_export_default_declaration`. `Scorer::react` is removed.
  Nothing outside `src/functions.rs` changes: Task 2 runs the binary.

- [ ] **Step 1: Write the failing test**

```rust edit path=tests/complexity.test.rs
<<<<<<< find
            ("memo() > useMemo()", 1, 0),
            ("helper", 1, 0),
        ]
    );
}
=======
            ("memo() > useMemo()", 1, 0),
            ("helper", 1, 0),
        ]
    );
}

#[test]
fn a_function_takes_the_react_name_fallow_s_walk_holds_for_it() {
    let project = Project::new("complexity-react-names");

    project
        .write(
            "src/panels.tsx",
            r#"export const Wrapped = wrap(() => 1, () => { useState(0); });
export const Sized = pad({ size: 1 }, () => { useState(0); });
export const Picked = flag ? () => { useState(0); } : Fallback;
export const Either = fallback || (() => { useState(0); });
export const Listed = [() => { useState(0); }];
export const accessors = { get Panel() { useState(0); return 1; } };
export class Box { get Panel() { useState(0); return 1; } }
export function Counter() { React.useState(0); }
it("Renders", () => { useState(0); });
export const { Grid } = make(() => { useState(0); });
"#,
        )
        .write(
            "src/props.tsx",
            r#"export function Defaulted({ a, b, c, d, e } = {}) { return <i />; }
export function Nested({ a: { b, c, d, e, f } }) { return <i />; }
export function Rest({ a, b, c, d, ...rest }) { return <i />; }
export function Outer({ a, b, c, d, e }) { return () => <i />; }
export function fields(a) { return class { x = a ? 1 : 2; }; }
"#,
        )
        .commit();

    let document = complexity(&project, &["--all"]);

    // The name a declaration, a key or `export default` holds goes to the first function the walk
    // reaches beneath it, and a key inside takes its place; an accessor's name has no `get `; a name
    // from a call's string argument, a destructuring and a member callee make no component (spec
    // SCORE-10, SCORE-12).
    assert_eq!(
        scores(&document, "src/panels.tsx"),
        [
            ("wrap()", 1, 0),
            ("wrap() #2", 1, 0),
            ("pad()", 1, 0),
            ("<anonymous 1>", 1, 1),
            ("<anonymous 2>", 1, 1),
            ("<anonymous 3>", 1, 1),
            ("accessors > get Panel", 1, 1),
            ("Box > get Panel", 1, 1),
            ("Counter", 1, 0),
            ("it(\"Renders\")", 1, 0),
            ("make()", 1, 0),
        ]
    );
    // A first parameter's top-level properties count, with or without a default and never a rest
    // element, where the function itself renders JSX; a class field's value counts toward the
    // function around the class (spec SCORE-6, SCORE-10).
    assert_eq!(
        scores(&document, "src/props.tsx"),
        [
            ("Defaulted", 1, 1),
            ("Nested", 1, 0),
            ("Rest", 1, 0),
            ("Outer", 1, 0),
            ("Outer > <anonymous 1>", 1, 0),
            ("fields", 2, 1),
        ]
    );
}
>>>>>>> replace
```

- [ ] **Step 2: Run the test to see it fail**

```text
cargo test --locked --test complexity
```

Expected: `a_function_takes_the_react_name_fallow_s_walk_holds_for_it` fails on its first assertion,
with `wrap() #2` and `pad()` at cognitive 1, the three `<anonymous …>` arrows and both
`get Panel`s at 0; every other line of both lists already matches. The other tests pass.

- [ ] **Step 3: Write the code**

```rust edit path=src/functions.rs
<<<<<<< find
    ChainExpression, Class, ConditionalExpression, ContinueStatement, DoWhileStatement, Expression,
    ForInStatement, ForOfStatement, ForStatement, FormalParameters, IfStatement, JSXElement,
    JSXFragment, LogicalExpression, LogicalOperator, MethodDefinitionKind, ObjectExpression,
    Program, PropertyKey, PropertyKind, Statement, SwitchCase, SwitchStatement, WhileStatement,
};
=======
    ChainExpression, Class, ConditionalExpression, ContinueStatement, DoWhileStatement,
    ExportDefaultDeclaration, Expression, ForInStatement, ForOfStatement, ForStatement,
    FormalParameters, IfStatement, JSXElement, JSXFragment, LogicalExpression, LogicalOperator,
    MethodDefinition, MethodDefinitionKind, ObjectExpression, ObjectProperty, Program,
    PropertyDefinition, PropertyKey, PropertyKind, Statement, SwitchCase, SwitchStatement,
    VariableDeclarator, WhileStatement,
};
>>>>>>> replace
<<<<<<< find
        open: Vec::new(),
        frames: Vec::new(),
    };
=======
        open: Vec::new(),
        frames: Vec::new(),
        held: None,
    };
>>>>>>> replace
<<<<<<< find
    /// The functions the walk is inside, innermost last.
    frames: Vec<Frame>,
}
=======
    /// The functions the walk is inside, innermost last.
    frames: Vec<Frame>,
    /// The React name a declaration, a key or `export default` holds for the first function the walk
    /// reaches beneath it (fallow's `pending_identity`; spec SCORE-10).
    held: Option<String>,
}
>>>>>>> replace
<<<<<<< find
            let react = self.react(node, span, own);
=======
            // A function's own identifier wins over a held name, which is lost either way.
            let held = self.held.take();
            let react = own
                .map(|identifier| identifier.name.to_string())
                .or(held)
                .is_some_and(|name| react(&name));
>>>>>>> replace
<<<<<<< find
    /// Whether a function's name is shaped like a React component's or hook's, read as fallow reads
    /// it: its own identifier, or else what declares it, or the call it is passed to, by name; never a
    /// name taken from a call (fallow's `frame_name_is_react_shaped`; spec SCORE-2).
    fn react(&self, node: NodeId, span: Span, own: Option<&BindingIdentifier<'a>>) -> bool {
        let mut node = node;
        let mut span = span;
        let name = own
            .map(|identifier| identifier.name.to_string())
            .or_else(|| {
                loop {
                    let (outer, holder) = self.holder(node, span);

                    if let Some((name, _)) = declared(holder, outer) {
                        break Some(name);
                    }

                    match holder {
                        AstKind::CallExpression(call) if passed(&call.arguments, outer) => {
                            node = call.node_id.get();
                            span = call.span;
                        }
                        _ => break None,
                    }
                }
            });

        name.is_some_and(|name| {
            name.starts_with(|first: char| first.is_ascii_uppercase()) || hook(&name)
        })
    }
=======
    /// Walks what the closure walks with the React name given held, and nothing held after it, whether
    /// or not a function took it (spec SCORE-10).
    fn hold(&mut self, name: Option<String>, walk: impl FnOnce(&mut Self)) {
        self.held = name;
        walk(self);
        self.held = None;
    }
>>>>>>> replace
<<<<<<< find
/// Whether a name is a React hook's: `use` and then a capital letter (fallow's `is_hook_callee`).
=======
/// Whether a React name is a component's or a hook's: it begins with a capital letter, or is a hook's
/// name (fallow's `frame_name_is_react_shaped`; spec SCORE-10).
fn react(name: &str) -> bool {
    name.starts_with(|first: char| first.is_ascii_uppercase()) || hook(name)
}

/// Whether a name is a React hook's: `use` and then a capital letter (fallow's `is_hook_callee`).
>>>>>>> replace
<<<<<<< find
    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
=======
    // A variable declared to a plain name, an object's or a class's key, and `export default` each
    // hold a React name for the first function beneath them (fallow's `visit_variable_declarator`,
    // `visit_object_property`, `visit_method_definition`, `visit_property_definition` and
    // `visit_export_default_declaration`; spec SCORE-10).
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(identifier) = &declarator.id {
            self.hold(Some(identifier.name.to_string()), |scorer| {
                walk::walk_variable_declarator(scorer, declarator);
            });
        } else {
            walk::walk_variable_declarator(self, declarator);
        }
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        self.hold(key_name(&property.key, property.computed), |scorer| {
            walk::walk_object_property(scorer, property);
        });
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        self.hold(key_name(&method.key, method.computed), |scorer| {
            walk::walk_method_definition(scorer, method);
        });
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        self.hold(key_name(&property.key, property.computed), |scorer| {
            walk::walk_property_definition(scorer, property);
        });
    }

    fn visit_export_default_declaration(&mut self, declaration: &ExportDefaultDeclaration<'a>) {
        self.hold(Some(DEFAULT.to_owned()), |scorer| {
            walk::walk_export_default_declaration(scorer, declaration);
        });
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
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
git add -A && git commit -m "fix(complexity): give a function the React name fallow's walk holds"
```

Expected: one commit.

### Task 2: The comparison with fallow

**Implements:** SCORE-4, SCORE-5, SCORE-7, SCORE-8, SCORE-9, SCORE-11, SCORE-14, VS-1, VS-2, VS-3, VS-4, VS-5, VS-6, VS-7

**Check:** the owner reads the report folder's `cmp/judged.md`: which of the comparison spec's
BAR-1 to BAR-3 holds at its top, and each case left with why; the pinned commits, fallow's version
and both tools' settings; per root, the counts shared, analyzer-only and fallow-only, and the
judgment of each finding only one tool reports and of each shared one whose scores differ; VS-4's
section apart, with SCORE-14's four kinds counted; the branch audits; VS-6's times; and each fix
with its fixture. The item closes only once SOLO-5's amendment, with VS-4's and SCORE-14's
release-notes lines where they are owed, is in the yakshaver item
`2026-11-18-analyzer-design-spec-stage-2-amendments` or written in the judged list as still owed
there (VS-7).

**Files:**
- None in this repository: the report folder, `.git/yak-reports/2026-11-22-complexity-findings/cmp/`,
  is committed nowhere. A cure Task 2 finds changes this branch as decision 6 says.

**Interfaces:**
- Consumes: the binary built from this branch after Task 1; the dead-code comparison's scripts and
  best-case fallow configurations (decision 5).
- Produces: the judged list, and the counts and outcome on the record.

- [ ] **Step 1: Pin, install and check fallow**

Read `install.version` in yakshaver's `tools/fallow.md` on its `main` that day. Where it is not
3.39.0, read what changed in fallow's counting between `v3.39.0` and that tag, and write each rule
that changed as a further amendment of the spec, approved before Step 3 (SCORE-5). Pin each named
root's commit afresh, make two clean worktrees of each, and install fallow's from its lockfile with
install scripts off (VS-1; comparison spec CMP-1 to CMP-4a, CMP-6). Check that
`fallow health --max-crap 1000000` is accepted, and on yakshaver's root count the findings over the
CRAP threshold alone at its default, and whether `fallow audit --format json` includes them (VS-4).

Expected: fallow's version written in the judged list, with any amendment approved; the flag
accepted, or the CRAP findings set apart by their fields where it is not (the spec's risk); the
CRAP count, and whether the gate shows them today, written in VS-4's section.

- [ ] **Step 2: Write the settings**

Copy the dead-code comparison's best-case fallow configurations into `cmp/configs/`, set
`health.maxCyclomatic` to 20 and `health.maxCognitive` to 15 in each, remove
`health.cyclomaticPolicy` and `health.thresholdOverrides` wherever they appear, and write each
change, and which named roots had either key, in `cmp/configs/CHANGES.md`. Write each root's hand-off
file in `cmp/hand-off/` with `cyclomaticLimit` 20 and `cognitiveLimit` 15 in the project's part, and
copy the scripts (decision 5).

Expected: one configuration and one hand-off file per root; `CHANGES.md` lists every change with
its reason (VS-1, SCORE-11, SCORE-14).

- [ ] **Step 3: Run both tools on every root**

On each root, from that root: `fallow health --format json --max-crap 1000000` with its
configuration, `yak-analyzer audit --hand-off <file>`, and `yak-analyzer complexity --hand-off
<file> --all`, saving each in `cmp/runs/`.

Expected: every run exits 0, or the failure is written in the judged list with its output.

- [ ] **Step 4: Match and judge**

Match each root's findings (decision 5) into shared, analyzer-only and fallow-only, with the
scores of each shared finding. Trace each finding only one tool reports, and each shared one whose
scores differ, to the counting rule that differs, and judge it by SCORE-2 and SCORE-3 (VS-3); read
the analyzer's scores of a function only fallow reports from the `--all` run. Check SCORE-7's
optional chains and SCORE-10's defaulted object pattern against fallow's own scores, in the named
roots or in a made-up file run through both tools. Set apart VS-4's kinds and SCORE-14's four, each
counted: fallow's suppressed functions, its threshold overrides, its findings in Svelte, Astro and
Angular files, and its findings in a `.ts` file holding JSX, with the file named.

Expected: each finding judged in `cmp/judged.md`; each false finding or miss of the analyzer's cured
as decision 6 says, its fixture failing first, and the runs of Step 3 made again with the cure
(comparison spec CURE-5).

- [ ] **Step 5: Branch audits and times**

Run the branch audits as the comparison spec's BRA-1 to BRA-3 run them, two merged pull requests per
named repository at the least, with `fallow audit --base <ref> --format json --max-crap 1000000` and
`yak-analyzer audit --hand-off <file> --base <ref>`, choosing, where the merges hold one, a branch
that makes an over-limit function worse, one that takes a function from under to over, and one that
adds or removes an unnamed function before another (VS-5). Measure `audit --base` on the named root
with the most files, on its pinned commit against its first parent, five warm runs each with
plan 2's binary and with this branch's, as the cold and warm measurements spec defines them (VS-6).

Expected: each branch audit's agreement judged, with each difference that comes only from fallow's
rule for "introduced" and design BASE-5 written down and not counted; both medians in the judged
list, and sent to the work item `2026-11-19-match-fallow-speed`.

- [ ] **Step 6: Record the counts and the outcome**

Write the judged list's top (VS-7), and record a ruling on Task 2 that holds, for each root, the
shared, analyzer-only and fallow-only counts, the counts set apart by kind, the branch audits'
agreement, the times, which of BAR-1 to BAR-3 holds, and what is owed in yakshaver.

Expected: the ruling is on the record, and the owner reads the judged list at validation.
