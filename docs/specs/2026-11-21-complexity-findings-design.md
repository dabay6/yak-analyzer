# Complexity findings, compared with fallow

**Status:** draft.

**Work item:** `2026-11-22-complexity-findings`, the first half of the analyzer's stage 4. It
answers no issue.

**Amends:**

- `docs/specs/2026-11-13-dead-code-findings-design.md` (**the dead-code spec** below), its
  COMMIT-4: a file skipped at the base no longer makes a complexity finding new where it is not that
  file's (NEW-4 below). Its rule for the five dead-code kinds stands.
- `docs/specs/2026-11-15-dead-code-comparison-design.md` (**the comparison spec** below), its
  FREEZE-1: "a change to one of them raises the interface version (design IF-6)" means a change that
  breaks one of them, as IF-6 says. A change that only adds, such as a new rule name, does not raise
  it (VER-1 below). The fingerprint scheme FREEZE-1 fixed keeps every part it has. What this item
  adds to it for the new rule, FN-4's and FN-5's rules for a function's name path, stays a draft
  until the swap (CPX-4, VER-1).
- `docs/specs/2026-11-11-reading-a-project-design.md` (**the reading spec** below), its KEY-3: the
  limit keys are now used, and which value wins is built (LIMIT-1 below).

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository (**the design
spec** below). This item builds its FIND-8, the complexity half of STAGE-4, and PROOF-2, PROOF-2a,
PROOF-4 and PROOF-5 for complexity. It follows FIND-1, FIND-10, IF-1 to IF-4, IF-6 and BASE-5. "(design
FIND-8)" after a sentence names the requirement it follows from.

How to read the marks: **owner** marks what the owner decided in this item's brainstorm, which is on
the work item's record. Everything else not taken from the design spec or an earlier spec of this
repository is the **session's call**: the session that wrote this spec decided it, and the owner has
not ruled on it.

## Intent

The analyzer reports the five dead-code kinds through `audit`. It doesn't report complexity yet.
yak still runs fallow for it. The pack's `typescript` stack runs `fallow audit --base {base}
--format json` and maps each of fallow's complexity findings to a file, a line and a message. The
function's scores, and the limits they passed, don't reach yak's finding.

This item adds complexity to the analyzer. The reason is integration with yakshaver, not a fault in
fallow (owner). fallow is the bar to match (design PROOF-4), not the problem. A complexity finding of
the analyzer's own carries the function, both scores and both limits, in a form yak can read. That
lets yakshaver later track functions over a limit and queue refactoring work for them (owner; that
tracking is yakshaver's work, not this item's).

A complexity finding new on a branch holds the branch at the gate until it is fixed or given a
reason, as the dead-code kinds do (owner). Which findings are new follows the design spec's BASE-5
(owner): a function over a limit is new unless the base has it over the same limit with a score at
least as high. So a function the branch adds over a limit is new. So is a function that was under at
the base and is over now, and one that was already over and got worse.

From this item on, each new check also gets a command of its own beside `audit`, for the owner at a
terminal or an agent; the engine runs `audit` only (owner). Complexity's command sets that pattern.
It is the command-line form of a complexity MCP tool that stage 5 adds (owner; design IF-1).

Terms:

- **yakshaver** is the developer tool this analyzer is built for, and **yak** is its command. **The
  engine** is yak's own program, which runs the analyzer's `audit` at the gate and after each edit.
  **The pack** is yakshaver's content pack. It declares the stacks, the analyzer's declarations among
  them, and the commands yak runs.
- **The gate** is yak's check before a branch's pull request. A finding the analyzer marks `new`
  holds the branch until it is fixed or a person gives a reason; one marked `unchanged` is
  **inherited** and doesn't.
- **The base** is the commit a branch is compared with (`audit --base <commit>`).
- **fallow** is the tool yak runs today for dead code and complexity. It is open source on GitHub
  (`fallow-rs/fallow`). Its `fallow health` command lists **health findings**: functions over its
  complexity settings. It also scores a Vue `<template>` block as if it were a function, and makes a
  **`<component>` rollup** finding that adds a component's worst method to its template. Its **CRAP
  score** combines a function's cyclomatic complexity with its estimated test coverage, and has a
  threshold of its own (30 by default).
- **The swap** is the pack release that replaces fallow with the analyzer in yak, at the end of the
  analyzer's stages (design SWAP-1, STAGE-6). Until it, nothing yak runs uses the analyzer (design
  PROOF-6).
- **The plan** is the implementation plan the builder writes from this spec once it is approved, a
  file in this repository's `docs/plans/`, which the owner approves before building starts.
- **The hand-off file** is the file yak writes to tell the analyzer which folder to read and what the
  stacks and the project declare (`--hand-off <file>`; CONTRIBUTING.md, "The commands").
- **Cyclomatic complexity** counts the independent paths through a function: one, plus one for each
  place the path can split. **Cognitive complexity** is SonarSource's measure of how hard a function
  is to read. It adds for each break in linear flow, and adds more the deeper that break is nested.
  SCORE-1 and SCORE-2 say how each is counted here.
- **A limit** is a positive whole number a declaration states: `cyclomaticLimit` or
  `cognitiveLimit` (the reading spec's KEY-1). A function is **over** a limit when its score is
  greater than the limit; a score equal to the limit is not over it. That is fallow's comparison too:
  in the run VS-4 describes, nine functions scoring exactly 15 cognitive were listed (over the CRAP
  threshold) and none was flagged for cognitive, and two scoring exactly 20 cyclomatic were listed
  and flagged only for cognitive. The pack's fallow configuration states the limits as
  `health.maxCyclomatic` and `health.maxCognitive`.
- **The named repositories** are the four repositories, six roots, the comparison spec's CMP-2
  names. Two are private, so this spec doesn't name them (CONTRIBUTING.md).
- **The judged list** and **the report folder** are as in the comparison spec: the list of judged
  findings, written to `yak-reports/<item id>` in git's common folder on the owner's machine, and
  committed nowhere (design PROOF-3).
- IDs with no prefix word, such as CPX-1, are this spec's. No prefix of this spec is used by the
  design spec or an earlier spec (NEW is new against inherited, VS the comparison, SOLO the command
  of its own, CASE the fixtures). Design IDs carry "design". The dead-code spec's and the comparison
  spec's carry those words.

## Goals, non-goals and anti-goals

**Goals:**

- A `complexity` finding in `audit` for each function over a declared limit, with its scores and
  limits on the result, marked new or unchanged by design BASE-5 (design FIND-8; owner).
- A command of its own, `yak-analyzer complexity`, that lists the functions over a limit, or every
  function with `--all` (owner).
- No known false finding and no miss against fallow on the named repositories, judged as stage 2
  judged the dead-code kinds (design PROOF-4).

**Non-goals:**

- Duplication (design FIND-9). It is the second half of stage 4, a later item.
- Speed. Matching fallow's speed is `2026-11-19-match-fallow-speed`'s. This item only measures what
  complexity adds to `audit` (VS-6).
- The MCP tool. The MCP server is stage 5. That stage adds the complexity tool over the same function
  as the command (owner).
- A command of its own for dead code. It comes at the end of the analyzer's stages (owner).
- Anything in yakshaver or the pack: the pack's default limits, the engine reading the result's
  scores, and tracking or queuing refactoring work (owner).
- Complexity of a Vue `<template>`, fallow's whole-component rollups, and fallow's CRAP score, which
  mixes complexity with estimated test coverage. Design FIND-8 names a function's cyclomatic and
  cognitive complexity only (VS-4).

**Anti-goals,** failures even if it technically works:

- A function wrongly marked inherited. It passes the gate in silence (design PROOF-2a).
- A finding of a function the branch never touched marked new, so the branch has to refactor code
  outside its work. One way this could happen is a file that couldn't be read at the base (NEW-4).
- A limit built into the analyzer. Where none is declared, nothing is reported, and a notice says so
  (design FIND-8).
- Any change to the dead-code findings, their fingerprints, or what `graph` reports about imports.

## Requirements

### What a function is, and its name

- **FN-1** A function is a function declaration, a function expression, an arrow function, or a
  method of a class or an object literal. Getters, setters and constructors count as methods. Code
  outside every function, such as a module's top level or a class's static block, is not a function
  and is never reported. A declaration with no body is not a function either: a TypeScript overload
  signature, a `declare function`, an abstract method, or a method of an interface or type. It takes
  no place under FN-5, so adding an overload moves no fingerprint. Session's call.
- **FN-2** Functions are found in every file whose code the analyzer parses whole: scripts, and the
  `<script>` and `<script setup>` blocks of Vue components (the reading spec's PARSE-1 and PARSE-3).
  MDX documents and HTML pages are read only for their import statements, so their functions are not
  scored. A fallow finding in one of them is set apart (VS-4). Session's call: none of fallow's 178
  health findings on yakshaver (the run VS-4 describes) was in such a file.
- **FN-3** A function is scored in the same parse that finds the file's imports. The file is not
  parsed a second time. For each function, the reading (what the analyzer holds of a project once it
  has read it, `Reading` in `src/reading.rs`) keeps only the function's name path, location and two
  scores. `graph` prints none of it, so no `graph` fixture changes; the `complexity` command is where
  the functions and their scores are read. Session's call: the cost is the parse, and it is already
  paid.
- **FN-4** A function's own name is the first of these that applies: the name of a function
  declaration or a named function expression; the variable a function is assigned to where it is
  declared (`const read = () => …`); the key of the object property or class member it is (a computed
  key has no name), with `get ` or `set ` before an accessor's (`get value`, `set value`); the whole
  left side it is assigned to (`a.b.read = function () {…}` gives `a.b.read`); `default` for an
  anonymous default export. Otherwise it is unnamed.

  A function passed as an argument to a call, with no name by those rules, takes its name from the
  call: the callee, with `()`, and inside the parentheses the call's first argument when that is a
  string literal or a template literal with no substitutions. So `computed(() => …)` is `computed()`, `items.map(item => …)` is
  `items.map()`, and `it("reads a file", () => …)` is `it("reads a file")`. A test's callback, a Vue
  `computed` or `watch`, and an array method's callback are the common unnamed functions, and this
  keeps an added one from shifting the others' places. Only calls of the same callee with no string
  first argument still share a count.

  A left side or a callee is written as its chain of names joined by `.`, with `[]` for a computed
  member, whatever spacing, line breaks, comments or optional chaining the source has. A call inside
  the chain is written as `()`, without its arguments, and a `new` keeps its keyword: so
  `promise.then(…).catch(() => …)` is `promise.then().catch()`, `describe.each(rows)("x", () => …)`
  is `describe.each()("x")`, and `new Promise((resolve) => …)` is `new Promise()`. A string
  argument is written as its value in double quotes, whatever quotes the source uses. So a
  formatter's rewrap or quote change moves no fingerprint. Session's call: design IF-4 says a function's own name or its
  place, and the call it is passed to is the nearest thing to a name such a function has.
- **FN-5** A function's name path is the names of what encloses it, from the outside in, then its
  own. What encloses it is each class and function around it, and each object literal around it. A
  class expression or an object literal takes its name by FN-4's rules for a function (the variable,
  property or left side it is assigned to), so `const Parser = class {…}` is `Parser`. A class
  declaration's name is its own. A Vue component's script blocks are one file for this count, in
  source order.
  Only a class or object literal with a function somewhere inside it takes part in a name path; one
  with none, such as a plain data argument, is passed over and takes no place.
  Each part of a name path is a name and a number. Within one enclosing class, function, object
  literal or file, the parts that take part are counted in source order, and functions, classes and
  object literals share the count. A name used once has the number 1. A name used more than once
  numbers its uses 1, 2 and on. The unnamed ones have no name and are numbered among themselves the
  same way. So no two functions of one file have the same name path, even where a string key is
  spelled like a readable place (`'read #2'` is the name `read #2` with the number 1). This is design
  IF-4's fingerprint for a function: its path, the names that enclose it, and its own name or place,
  with no line number. The readable form joins the parts with ` > `, writes a number above 1 as
  ` #2`, and writes an unnamed part as `<anonymous 1>`, as in `Parser > read #2 > <anonymous 1>`.
  Session's call, with the readable form.
- **FN-6** Two consequences of FN-5 are accepted and written here, so nobody takes them for faults.
  A renamed function, or one moved under another enclosing name, is a different function. An unnamed
  function added before an existing one changes the existing one's place, so the two can swap
  fingerprints. The same holds for a function of a repeated name, and for an unnamed class or object
  literal that gains its first function. Removing an earlier sibling shifts the later ones the same
  way. With NEW-1's comparison of scores, a shifted function can be compared with a removed
  sibling's base scores, and a function that got worse can be marked inherited. Each has a fixture
  (CASE-1) and
  is looked for among the branch audits (VS-5). A file
  git finds renamed keeps its functions (the dead-code spec's INH-2). Session's call: a fingerprint
  that guessed past these would be design IF-4 changed, and nothing yet shows the cost is worth it.

### The scores

- **SCORE-1** These are the starting rules. SCORE-2 may replace any of them, and SCORE-4 brings the
  final list back here. Session's call: the owner handed the counting over.
  - **Both measures** count only the function's own body. A function nested inside it is scored on
    its own and adds nothing to the outer function's score, nor to its nesting. This departs from
    SonarSource 1.7, which counts a nested function's structures in the enclosing function's score
    with the nesting the function adds. Session's call: fallow reports a nested arrow as a function of
    its own (178 findings, 27 of them `<arrow>`), and SCORE-2 settles whether it also counts it in the
    outer one.
  - **Cyclomatic** is 1, plus 1 for each `if`, `for`, `for…in`, `for…of`, `while`, `do`, non-default
    `case`, `catch`, ternary `?:`, and each `&&`, `||` and `??`, their assignment forms (`&&=`, `||=`,
    `??=`) included. Optional chaining (`?.`) and default parameters add nothing.
  - **Cognitive** follows SonarSource's published definition ("Cognitive Complexity", G. Ann
    Campbell, version 1.7). It adds 1 for each `if`, `else if`, `else`, ternary, `switch`, loop and
    `catch`, and for each run of the same boolean operator (`a && b && c` adds 1, `a && b || c` adds
    2). It adds 1 for a `break` or `continue` to a label, and 1 for a direct recursive call. An `if`,
    ternary, `switch`, loop or `catch` also adds its nesting depth: how many of these blocks already
    enclose it inside the function: the body of an `if`, `else if` or `else`, a ternary's branches, a
    `switch`, a loop, or a `catch`. An `else if` or `else` adds 1 but no nesting depth. A `try` adds
    nothing and is not a level of nesting.
- **SCORE-2** Where fallow's counting departs from SCORE-1, fallow's counting wins, rule by rule, so
  the gate's findings stay the same across the swap. Optional chaining, `??`, default parameters,
  `case` fall-through, labelled jumps, nested functions and JSX are the likely places. The plan's
  first step reads fallow's counting from its documentation and source, at the version VS-1 uses,
  and how it compares a score with its limit.
  It writes each rule the analyzer will count, with where it comes from: SonarSource's
  text, or fallow's file and function. fallow's code is read, not copied (CONTRIBUTING.md), and its
  license is checked before its source is read. If the license doesn't allow reading, the plan uses
  fallow's documentation and fixtures run through fallow instead. Session's call: PROOF-4 measures
  the analyzer against fallow, and a score has no truth in the code apart from the rule chosen.
- **SCORE-3** Where fallow's code departs from its own documentation, its code wins, as SCORE-2 says,
  and the difference is written in the plan's rule list. Where fallow counts one construct
  differently in different places, the analyzer follows the way it counts it in most of the
  comparison's cases, and the others are written down in the judged list, not counted as misses.
  Session's call.
- **SCORE-4** The rule list SCORE-2's reading yields comes back to this spec as an amendment, with
  each rule's source, and is approved before the comparison starts. So the rules that decide new
  against inherited are reviewed as requirements, not left as plan notes. A rule changed later, by a
  fix during the comparison, is a further amendment to this spec, written before the fix is planned
  (the comparison spec's CURE-4). Session's call.

### The limits

- **LIMIT-1** Each limit has one value for the whole run. The value a project states wins over the
  pack's. Among values from the same source, the stack later in the hand-off file wins (design
  ENT-4). This builds what the reading spec's KEY-3 left to the first item that reads a limit.
  Session's call: a limit for each stack's folders is not what fallow's single setting does, and no
  repository has asked for one.
- **LIMIT-2** A measure whose limit is declared is checked. A measure whose limit isn't is not,
  and a notice names the missing key. With neither limit declared, as when the analyzer runs with no
  hand-off file, no complexity finding is reported, and the notice says so (design FIND-8, IF-5).
  Session's call for the one-limit case.

### The finding

- **CPX-1** `audit` reports a function over either limit as one finding under the rule `complexity`
  (design FIND-1). It is one finding per function, whichever measures are over. So a reason given for
  it covers the function as it later stands on the same branch, whichever measure moves (design
  IF-4). Session's call: design FIND-1 lists one kind. fallow's three rules (one per measure, and one
  for a function over both) all describe one function.
- **CPX-2** Its location is the function's start: its name where the name is the function's own
  (a declaration, a named expression, a method or property key), otherwise the first character of
  the function, as for a callback named from its call. Its message names the function's readable name path and each measure
  that is over, with its score and limit, then the other measure's score. For example: "`Parser >
  read` has cognitive complexity 23, over the limit of 15 (cyclomatic 12)." Session's call.
- **CPX-3** Its result carries a SARIF property bag, `properties.complexity`, with `function` (the
  readable name path), `cyclomatic` and `cognitive` (the scores), `cyclomaticLimit` and
  `cognitiveLimit` (each limit, or `null` where it isn't declared), and `over`, a list of the
  strings `"cyclomatic"` and `"cognitive"` naming the measures that are over, `cyclomatic` first. With
  a base, it also carries `base`: an object `{ "cyclomatic": <number>, "cognitive": <number> }`
  holding the function's scores at the base, or `null` where the base doesn't have the function.
  Without a base, `base` is absent. This is what the engine will read for yakshaver's tracking (owner:
  the scores and the limit reach yak). Session's call for the names.
- **CPX-4** Its fingerprint is made, as design IF-4 says, from the rule, the file's path and the
  function's name path (FN-5), under the existing scheme `yakAnalyzer/v1`. Each part of the name path
  gives three parts of the digest: `n` for a named part or `u` for an unnamed one, its name (empty
  for an unnamed part), and its number written in decimal digits, each length-prefixed as
  `src/findings.rs` does today. So a name holding ` > ` or ` #2`, or an empty string key, can't
  collide with another path. FN-4's and FN-5's rules are part of the scheme, and are drafts until the swap
  (VER-1). The run's list of rules gains `complexity`, after the five.
- **CPX-5** No function in a file declared data or generated, or in a `.d.ts` file, is reported
  (design FIND-10). The same holds at the base.

### New against inherited

- **NEW-1** With a base, a complexity finding is `unchanged` when the base has a function with the
  same fingerprint, its path moved through git's renames, and for each measure the finding is over,
  the base's score is over the same limit and at least as high. Otherwise it is `new` (design BASE-5;
  owner). Both sides use the limits in force now (design BASE-3), so "the same limit" is one value.
- **NEW-2** So the comparison reads every function of the base and its scores, not only the base's
  findings. A function under the limit at the base must be found there for its message to say so.
  The existing comparison (`src/inherit.rs`) matches the base's findings by fingerprint alone. It
  gains a comparison of scores for this rule. The five dead-code kinds keep theirs.
- **NEW-3** A new finding's message adds, for each measure that is over, what the base had: "At the
  base its cognitive complexity was 18." or "At the base its cognitive complexity was 12, under the
  limit." Where the base doesn't have the function, the message adds
  nothing. Session's call.
- **NEW-4** Any file skipped at the base leaves the other complexity findings compared as NEW-1
  says, unlike the dead-code kinds. A skipped script has no functions at the base, so its findings
  are new. A skipped `package.json`, workspace list or tsconfig (each a `file-skipped` notice, like a
  script) affects no function's scores. One function's scores don't depend on any other file. This amends the
  dead-code spec's COMMIT-4 for this rule only, and the notice COMMIT-4 gives adds that complexity
  findings were still compared, those in a skipped script marked new. Session's call: under COMMIT-4 as it
  stands, one broken file at the base would make every over-limit function in the repository new,
  which is the anti-goal.
- **NEW-5** FN-5 gives each function of a file a name path of its own, so no two functions on one
  side share a fingerprint. Should a case FN-5 missed give two, the comparison pairs them in source
  order, as `src/inherit.rs` pairs findings of one fingerprint today, and the case gets a fixture and
  a fix of FN-5. Session's call.

### The command

- **SOLO-1** `yak-analyzer complexity [--hand-off <file>] [--all]` reads the project as `audit` does,
  and prints one JSON document on standard output. Each option is taken at most once, in either
  order. It takes no base. The gate's view of new and inherited is `audit`'s, and the MCP tool stage 5
  adds may add a base later without breaking anything. Session's call for the name and the options;
  owner for `--all`.
- **SOLO-2** The document holds `interfaceVersion`, `limits` (`cyclomatic` and `cognitive`, each a
  number or `null`), `functions` and `notices`. Each entry of `functions` holds `path`, `line`,
  `column`, `function` (the readable name path), `fingerprint` (the value `audit` gives, or would give, the
  function's finding, so it is there for a function `--all` lists that is over nothing), `cyclomatic`, `cognitive` and `over`. Entries are sorted by path, line and
  column. Notices are written as `graph` writes them. Session's call.
- **SOLO-3** By default, `functions` lists the functions over a limit. With `--all`, it lists every
  function scored, each with `over` empty where it is over nothing. That includes every function when
  no limit is declared (owner). Files declared data or generated, and `.d.ts` files, are left out
  either way (CPX-5).
- **SOLO-4** It exits as `audit` does: 0 when it ran, whatever it found; 1 with one line on standard
  error when it couldn't run; 2 for a command line it refuses. The refusal's list of what the binary
  accepts names it.
- **SOLO-5** It is a thin command over a function that `audit` and stage 5's MCP tool also use. The
  MCP tool's addition to design MCP-1, a seventh tool, is owed to the design spec as an amendment
  (owner). The same amendment changes STAGE-5's "six tools" to seven, and SWAP-2's line for "too
  complex" to name the new tool. It is made through the yakshaver work item `2026-11-18-analyzer-design-spec-stage-2-amendments`,
  or its successor if that item has closed, and not in this repository. The command's name, options and document are drafts until
  stage 5 releases the MCP tool, as the declaration keys were drafts until stage 2 ended (the
  comparison spec's FREEZE-1). That stage fixes them with the tool, and may change them first. yak
  never runs the command (owner), so a change before then breaks nothing yak runs. Session's call.

### The interface version

- **VER-1** The interface version stays 1. A new rule name, a new property on a result and a new
  command only add, and design IF-6 raises the version only for a change that breaks. The comparison
  spec's FREEZE-1 says the same through its "(design IF-6)". `CONTRIBUTING.md`'s paragraph, which says
  "a change to any of them", becomes "a change that breaks any of them". It names the sixth rule,
  `complexity`. It adds to what the version covers the names and meanings of CPX-3's property bag,
  since the engine will read it, and FN-4's and FN-5's rules, as part of the fingerprint scheme. Both
  are drafts until the swap: nothing yak runs reads them before then (design PROOF-6), so changing
  them raises nothing. From the swap they are fixed, and a change that breaks them raises the
  version. The paragraph also says the `complexity` command is a draft until stage 5 (SOLO-5). The
  module comment of `src/declarations.rs` and the test that holds the paragraph
  (`scripts/contributing.test.mjs`) change to match. Session's call: the design spec decides it.
- **VER-2** `CONTRIBUTING.md`'s "The commands" also gains the `complexity` command, and its
  description of `audit` says it reports six kinds, the five dead-code kinds and complexity.

### Fixtures

- **CASE-1** Each rule above has a made-up project in `tests/`, with its expected output written in
  the test (CONTRIBUTING.md, "Fixtures"). These are at least:
  - each counting rule of SCORE-1, as SCORE-4's amendment leaves the list;
  - each naming case of FN-4 and FN-5, among them a getter and setter pair, methods of two sibling
    object literals, two assignments ending in the same member, one name declared in two sibling
    blocks, a class expression assigned to a variable, a function with overload signatures, callbacks
    named from their call (with and without a string first argument), and an empty string key;
  - FN-6's accepted cases: a rename, an unnamed function added before another, a function of a
    repeated name added before another, and an unnamed object literal gaining its first function;
  - FN-5's passed-over object literal with no function in it, and a string key spelled like a
    readable place (`'read #2'`);
  - each base case: a function absent at the base, one under at the base, one worse, one the same,
    one better, one in a renamed file, one in a script skipped at the base, and a `package.json`, a
    workspace list and a tsconfig skipped at the base;
  - a removed earlier sibling, unnamed and of a repeated name, with the later function made worse;
  - a formatter's rewrap of an assignment and a callee, and a quote change in a string argument;
  - an anonymous default export;
  - callbacks whose callee chain holds a call or a `new`;
  - LIMIT-1's precedence, LIMIT-2's one limit and no limit, and a score equal to its limit;
  - CPX-5's data, generated and `.d.ts` files, and FN-2's Vue blocks and MDX document;
  - the command, with and without `--all` and with and without limits.
- **CASE-2** No fixture copies code from fallow or from the named repositories (CONTRIBUTING.md).

### The comparison with fallow

- **VS-1** The comparison runs as the comparison spec's ran, and its requirements apply with
  "complexity" for "the five dead-code kinds": pinned commits (CMP-1), the same roots (CMP-2), clean
  worktrees and hand-written hand-off files (CMP-3), fallow's worktree installed with scripts off
  (CMP-4, CMP-4a), and each root run from that root (CMP-7). The commits are pinned afresh on the day
  this comparison starts. fallow's version is the one the pack pins that day in `tools/fallow.md`
  (`install.version`; 3.39.0 when this spec was written, the version stage 2 used), installed as the
  comparison spec's CMP-6 installs it. SCORE-2 reads that same version. The whole-tree runs use
  `fallow health --format json`, since `fallow audit` reports only the files a branch changed. On the analyzer's side, the whole-tree runs use `yak-analyzer audit --hand-off <file>` for its
  findings, and `yak-analyzer complexity --hand-off <file> --all` to read its scores of a function
  only fallow reports. The branch audits use `fallow audit --base <ref> --format json`, as the gate
  does, and the analyzer's `audit --base` (VS-5). fallow runs with its best-case configuration
  for each repository (the comparison spec's CMP-5), except that both tools use limits of 20
  cyclomatic and 15 cognitive whatever that configuration says: the hand-off files declare them, and
  fallow's `health.maxCyclomatic` and `health.maxCognitive` are set to them (design FIND-8).
  Session's call for the version: the gate's findings are the ones to keep.
- **VS-2** Two findings are the same when they are in the same file and name the same function. A
  function is matched by its start line, and by its column where two start on one line, since fallow names an unnamed function `<arrow>` and the
  analyzer by its place. A function both tools report with different scores is shared, and the
  difference is judged too, since a score decides new against inherited (NEW-1). Session's call.
- **VS-3** Each finding only one tool reports is traced to the counting rule that differs and judged
  by SCORE-2 and SCORE-3. It is the analyzer's false finding or miss where the analyzer departs from
  the rule. It is written down, not counted, where fallow departs from its own rule. The comparison
  spec's judging (JDG-1, JDG-6), bar (BAR-1 to BAR-4) and fixes (CURE-1 to CURE-5) apply. Each false
  finding or miss becomes a failing fixture before it is fixed (design PROOF-5).
- **VS-4** Set apart and not counted as misses, each listed with its count, as the comparison spec's
  JDG-5 lists what it sets apart: fallow's findings on a Vue `<template>`, its `<component>` rollups,
  its findings in MDX documents and HTML pages (FN-2), and its findings over its CRAP threshold alone.
  On yakshaver's `main` on 2026-11-21, `fallow health --format json` (fallow 3.15.0, yakshaver's own
  configuration) gave 178 health findings, and 105 of them were over the CRAP threshold alone. That
  count is from fallow 3.15.0, the machine's install. The plan's first step counts again with the
  pinned version, and checks whether `fallow audit`'s JSON, which the pack maps today, includes such
  findings. If it does, the gate shows them today, and the judged list says how many leave the gate
  at the swap. A line in the swap's release notes (design SWAP-3) is then owed through the same
  yakshaver item as SOLO-5's amendment. Session's call, on design FIND-8's wording; the owner rules
  on it before this spec is approved.
- **VS-5** Branch audits run as the comparison spec's BRA-1 to BRA-3 run them, two merged pull
  requests per repository at the least. They include, where the merges hold one, a branch that makes
  an over-limit function worse, one that takes a function from under to over, and one that adds an
  unnamed function before another, or removes one (FN-6). The audits also check how fallow decides that a complexity
  finding was introduced. A difference that comes only from fallow's rule and design BASE-5 is written down,
  as BRA-3 says, and is not a miss.
- **VS-6** The time `audit --base` takes on the named root with the most files is measured before and
  after this item's change, on its pinned commit with that commit's first parent as the base. Each is
  five warm runs: runs straight after a first run, with the files in the operating system's cache, as
  `docs/specs/2026-11-19-cold-warm-measurements-design.md` defines them. The median goes in the
  judged list. No bar is set. The numbers go to this repository's work item
  `2026-11-19-match-fallow-speed`. Session's call.
- **VS-7** The judged list holds, at the top, which bar holds and each case left (the comparison
  spec's BAR-4); the pinned commits, fallow's version and both tools' settings; per root, the counts
  shared, analyzer-only and fallow-only, and each judgment; VS-4's section apart; the branch audits;
  VS-6's times; and each fix with its fixture. The owner reads it before the item closes (design
  PROOF-3, PROOF-4). The item also closes only once the owner's ruling of VS-4 is on the record, and
  SOLO-5's amendment, with VS-4's release-notes line where it is owed, is in the yakshaver item or
  written in the judged list as still owed there.

## Alternatives considered

- **The owner's first gate rule: only a function the branch adds holds the branch.** The owner
  chose it first, so a branch couldn't be made to refactor code outside its work. It was dropped for
  design BASE-5 (owner). Until yakshaver tracks inherited functions, it let a branch make an existing
  function arbitrarily worse with nothing at the gate. It also needed a design-spec amendment, and it
  would have ruled differently from design BASE-5's rules for cycles and duplication.
- **A rule name per measure, as fallow has (fallow adds a third for both).** A reason given for the cognitive finding
  wouldn't cover the same function going over on cyclomatic, and design FIND-1 lists one kind.
  Rejected (session's call).
- **SonarSource's definition as the only arbiter.** Every place fallow differs would move findings at
  the swap. The comparison couldn't call a fallow-only finding a miss without first ruling on the
  definition. Rejected for SCORE-2 (session's call).
- **Callbacks named only by their place, as design IF-4 reads at its narrowest.** Every added test
  case or `computed` would shift the places of those after it, marking them new or wrongly inherited
  on ordinary branches. Rejected for FN-4's call-site names (session's call).
- **A separate parse for complexity.** It is simpler to keep apart, but it pays the parse twice on
  both trees at every audit, after each edit included. Rejected for FN-3 (session's call).
- **A base option on the command.** The gate's view is `audit`'s, and the owner's use of the command
  is looking at one thing. It can be added later without breaking anything (SOLO-1). Rejected for now
  (session's call).
- **Raising the interface version to 2.** It costs little before the swap, but design IF-6 says an
  addition doesn't raise it. Raising it would teach the pack to update for changes that break
  nothing. Rejected (session's call).
- **Keeping COMMIT-4's all-new rule for complexity.** It is consistent with the dead-code kinds, but
  one broken base file would make every over-limit function new. Rejected for NEW-4 (session's
  call).

## Risks

- **Complexity is off at the gate when no limit is declared, and the notice goes where yak doesn't
  look.** yak reads no notifications (design IF-3). Until the swap, the gate doesn't run the analyzer
  (design PROOF-6). The swap's pack release carries the default limits (design FIND-8, DECL-1a, stage
  3's work in yakshaver). The command's `limits` field and its notice show the state to a person or
  an agent.
- **Unnamed functions are misread when a sibling is added before them.** FN-4 names callbacks from
  their call, which leaves the shift only among calls of the same callee with no string argument. A
  removed sibling shifts places too. A wrongly inherited finding passes in silence. FN-6 writes the case down, CASE-1 fixes its behaviour, and VS-5 looks for it on
  real branches. If it shows up there, the judged list says so, and changing design IF-4 is the
  owner's call.
- **fallow's counting can't be read, or changes between versions.** SCORE-2 falls back to its
  documentation and black-box fixtures. VS-1 pins one version, the gate's.
- **Matching by start line misses where the two tools place a function differently**, for example
  at a decorator or an `export` keyword. Such pairs show as one finding on each side. Judging traces
  them, and the builder normalizes the start where they recur (left to the builder).
- **The cost of scoring on both trees at every audit.** FN-3 keeps it inside the parse, and VS-6
  measures it. A cost found too high goes to the speed item, not here.
- **The speed item may change the same parse.** `2026-11-19-match-fallow-speed` is still an idea.
  Whichever item lands second rebases onto the other. Neither changes the other's requirements.
- **Work owed outside this repository is forgotten.** SOLO-5's amendment and VS-4's release-notes
  line are yakshaver's. VS-7 makes the item's close wait for them to be filed or written down as owed.
- **CRAP findings leave the gate at the swap.** If VS-4 finds the gate shows them today, the owner
  learns how many before the swap, from the judged list.

## Left to the builder

- The names of the modules, types and functions, and where the scoring sits in the parse.
- The exact wording of messages and notices, within CPX-2, NEW-3 and LIMIT-2.
- The order of keys in the command's JSON, and whether `notices` there carries the base's notices
  (it has no base today).
- How VS-2 normalizes a function's start line, where decorators or `export` make the two tools
  differ.
- How the plan's first step reads fallow's counting (SCORE-2). The rules it yields come back to this
  spec as SCORE-4's amendment, not left in the plan.
