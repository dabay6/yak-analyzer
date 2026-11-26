# Duplication findings, with fallow as a yardstick

**Status:** draft.

**Work item:** `2026-11-23-duplication-findings`, the second half of the analyzer's stage 4. It
answers no issue.

**Amends:**

- `docs/specs/2026-11-11-reading-a-project-design.md` (**the reading spec** below), its KEY-1 and
  KEY-3: `duplicationMinimum` gets its unit, five keys are added, three of them lists of text that
  are neither globs nor paths, and one list key replaces where the others join (TUNE-1 to TUNE-6 below).
- `docs/specs/2026-11-21-complexity-findings-design.md` (**the complexity spec** below), its
  Non-goals: duplication, left there to "a later item", is this item.

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository (**the design
spec** below). This item builds its FIND-9, the duplication half of STAGE-4, and PROOF-2, PROOF-2a
and PROOF-5 for duplication. It follows FIND-1, FIND-10, IF-2 to IF-6 and BASE-1 to BASE-4. It
departs from three of the design spec's requirements, by the owner's decisions, and the amendments
are owed there (OWED-1): FIND-9 and PROOF-4, which make fallow the floor for duplication, and
BASE-5's rule for a duplicated group. "(design FIND-9)" after a sentence names the requirement it
follows from.

How to read the marks: **owner** marks what the owner decided in this item's brainstorm, which is on
the work item's record. Everything else not taken from the design spec or an earlier spec of this
repository is the **session's call**: the session that wrote this spec decided it, and the owner has
not ruled on it.

## Intent

The analyzer reports the five dead-code kinds and complexity through `audit`. It doesn't report
duplication yet, so yak still runs fallow for it at the gate. Duplicated code is worth finding
because each group of copies is a refactor candidate (owner): a change to one copy usually needs
the same change in the others, and nothing reminds the person making it.

This item adds duplication to the analyzer. As with complexity, the analyzer only identifies: it
finds the groups of copies and says which a branch made new; holding the branch, and collecting
refactor candidates, are yakshaver's (owner). A new group holds the branch at the gate like any new
finding. There is no warning level (owner).

Unlike complexity, fallow is not the bar here. Duplication is more a matter of judgment than a
score is, so the analyzer has its own definition of a copy, built from the ground up: whole
statements and declarations, never a run that starts or ends mid-line (owner). fallow is a
yardstick: the comparison shows what fallow finds that the analyzer doesn't, and the owner judges
the differences, which are not failures by themselves (owner). How much code has to match, and how
loosely, is tunable: the pack sets a default, and a project can change it (owner).

Terms:

- **yakshaver**, **yak**, **the engine**, **the pack**, **the gate**, **the base**, **inherited**,
  **the swap**, **the plan**, **the hand-off file**, **the named repositories**, **the judged list**
  and **the report folder** are as the complexity spec defines them.
- **The dead-code spec** is `docs/specs/2026-11-13-dead-code-findings-design.md`. Its COMMIT-4
  notice is the one `src/inherit.rs` gives today, as the complexity spec's NEW-4 changed it.
- **A source** of a declaration is the pack or the project: the hand-off file keeps the two apart for
  each stack.
- **fallow** is the tool yak runs today for dead code, complexity and duplication (open source,
  `fallow-rs/fallow`). Its duplication finder reports **clone groups**: runs of tokens, which can
  start and end mid-line, that appear in two or more places. Its `duplicates.mode` setting says how
  loosely tokens match; all four named repositories set `weak`, which ignores the values of string
  literals, and keep fallow's defaults otherwise: at least 50 tokens, 5 lines and 2 copies. (Read
  by this item's brainstorm from the four repositories' fallow configurations and fallow 3.39.0's
  defaults on 2026-11-23, and on the work item's record; YARD-1's first step reads them again on
  the pinned commits.) Its `ignoredClones` setting silences a group a person has reviewed. Its
  tokens are its own lexer's, which need not count as SIZE-1 does, so "50 tokens" in the two tools
  is not one size.
- **A statement list** is a list of statements in the source: a file's top level, a block's body
  (a function's, a loop's, an `if`'s and so on), a `switch` case's statements, or a class's body,
  whose members count as its statements. A Vue component's `<script>` and `<script setup>` blocks
  are one file for finding copies, but each block's top level is a statement list of its own, so no
  run crosses from one block into the other.
- **A run** is one or more statements next to each other in one statement list. (One execution of
  the analyzer is called an invocation here, so the word isn't used twice.) **A copy** is a run
  that is the same as another run (SAME-1 to SAME-4). **A group** is all the copies of one run,
  two or more.
- **A token** is a piece of a statement as ECMAScript's lexical grammar counts it: an identifier or
  keyword, a punctuator, a literal, each piece of a template literal, and a JSX text that holds more
  than white space. Tokens are counted on the statement's syntax tree written out in one fixed
  form, not on its source: so nothing a formatter may add or drop counts, such as semicolons,
  trailing commas, parentheses that change nothing (SAME-1), the parentheses around a lone arrow
  parameter, or the empty `()` of `new Foo`. Comments and white space are not tokens. Session's
  call: a formatter's pass then moves no count.
- **Names** are identifiers: the names of variables, parameters, functions, classes, properties,
  object keys, members and labels, type names and type parameters, and JSX tag and attribute names.
  Keywords, `this` and operators are not names.
- **Types are code** for this item: type annotations, type arguments and `as` or `satisfies`
  expressions are part of a statement's shape and its tokens, and type-only statements (`type`
  aliases, interfaces, `declare` statements) join runs like any other. Session's call: a repeated
  type is as much a refactor candidate as repeated code, and leaving types out would make two runs
  that differ only in types one group.
- **A callee chain** is what a call calls, written as the complexity spec's FN-4 writes a callee:
  its names joined by `.`, a call inside the chain as `()` without its arguments, and a tagged
  template inside the chain as ``` `` ``` without its contents. So `it.each(rows)("x", fn)` calls
  the chain `it.each()`, ``it.each`a | b`("x", fn)`` the chain ``it.each`` ``, and
  `test.only("x", fn)` the chain `test.only`.
- **A test callee** is a call that runs a test, such as `it(…)` or `test(…)`, named by a
  declaration (TUNE-5), never by the analyzer's code (CONTRIBUTING.md: a tool's convention is a
  declaration).
- IDs with no prefix word, such as COPY-1, are this spec's. No prefix of this spec is used by the
  design spec or an earlier spec of this repository. Design IDs carry "design", as "design DECL-2";
  the complexity spec's, the reading spec's and the dead-code spec's carry those words.

## Goals, non-goals and anti-goals

**Goals:**

- A `duplication` finding in `audit` for each group, with every copy's place on it, marked new or
  unchanged by FRESH-1 (design FIND-1, FIND-9; owner).
- Detection tuned by declarations: whether names, strings and numbers must match, the smallest run
  in tokens and in lines, and how many copies make a group (owner).
- A way to accept a group a person has reviewed, so it stops being reported until its content or
  its number of copies changes (owner, after fallow's `ignoredClones`).
- A command of its own, `yak-analyzer duplication`, listing the groups (owner, as complexity has
  one).
- A comparison with fallow on the named repositories that shows what fallow finds and the analyzer
  doesn't, judged by the owner, and sets the pack's defaults (owner).

**Non-goals:**

- Near-miss matching (copies that differ by an added or removed statement), a project-wide
  duplication percentage, and reporting only copies in different folders. Each is a different
  algorithm or a gate policy. They are not in this item, and may never be (owner).
- Anything in yakshaver or the pack: the pack's default values, how a person sets a declaration or
  accepts a group, the engine reading the result's details, and collecting refactor candidates
  (owner). What this item owes there is in OWED-1.
- Duplicated test bodies. In a test, only code outside the test callees' callbacks counts: setup
  and teardown in hooks, and helpers (owner: "copy-paste in tests, while preferably not allowed,
  isn't as important").
- MCP-1's duplicates tool. The MCP server is stage 5 (SHOW-5).
- Speed. This item measures what duplication adds to `audit` (YARD-6), and sets no bar.

**Anti-goals,** failures even if it technically works:

- A group wrongly marked inherited. It passes the gate in silence (design PROOF-2a).
- A group the branch never touched, in none of its copies and in no copy the base had, marked new,
  so the branch has to refactor code outside its work.
- A finding that carries the copied code. A copy can hold a string that should not leave the
  machine in a SARIF file, so a finding names places only (session's call).
- A minimum built into the analyzer. Where none is declared, nothing is reported, and a notice says
  so (design FIND-8).
- Any change to the other six kinds' findings, their fingerprints, or what `graph` prints.

## Requirements

### What a copy is

- **COPY-1** A copy is always a run of whole statements of one statement list (Terms), so it never
  starts or ends inside a statement or mid-line (owner). Runs are looked for in every statement list
  at every depth, so the bodies of two functions that differ only in their names are found as a
  group of their bodies' runs. Session's call for statement lists and class bodies: an object
  literal's properties and an array's elements are not statements, and a repeated configuration
  object is a declaration of data, not a refactor candidate.
- **COPY-2** Import declarations and `export … from` declarations never join a run. A file's
  imports repeat across files by nature, and the fix for them is not a shared function. Session's
  call.
- **COPY-3** A statement that holds a call to a test callee (TUNE-5) anywhere inside it never joins
  a run as a whole, and nothing inside a test callee's callback is looked at. A test callee's
  callback is any function passed as an argument to the call, or to any call in its callee chain:
  in `it.each(rows)("x", (row) => {…})` the arrow is one. Runs are still looked
  for in the statement's other statement lists, outside those callbacks: so in
  `describe("x", () => { beforeEach(() => {…}); it("y", () => {…}); })`, the `beforeEach` call can
  join a run with another `describe`'s, and its callback's body is searched, while the `it` call and
  its callback are not (owner: hooks count, test bodies don't). A function a test file declares
  outside every test callee counts like any code.
- **COPY-4** Runs are found in every file whose code the analyzer parses whole: scripts, and the
  script blocks of Vue components (the reading spec's PARSE-1 and PARSE-3). MDX documents and HTML
  pages are read only for their imports, so nothing in them is a copy. No run is found in a file
  declared data or generated, or in a `.d.ts` file (design FIND-10). The same holds at the base.
  Session's call, as the complexity spec's FN-2 and CPX-5.
- **COPY-5** The runs are read in the same parse that finds the file's imports and scores its
  functions; the file is not parsed again. The reading keeps, for each statement list, what
  SAME-1 compares and SIZE-1 counts for each statement, and its lines. `graph` prints none of it.
  Session's call, as the complexity spec's FN-3.

### When two runs are the same

- **SAME-1** Two runs are the same when they have the same number of statements and each pair of
  statements has the same syntax: the same kinds of node in the same shape, whatever the
  formatting, comments, quotes, optional semicolons or trailing commas. Parentheses that change
  nothing are not part of the shape: `(a + b)` and `a + b` alone as a statement are the same, while
  `(a + b) * c` and `a + b * c` are not, since their trees differ. A property key or a member is
  its name, written bare or quoted (`{ a: 1 }` and `{ "a": 1 }` are the same); a computed key
  (`{ [a]: 1 }`) is not a name. Within that, by default:
  each name must be the same name, each string the same value, and each number the same value
  (design FIND-9: "the same once whitespace and comments are dropped"). A string is its value after
  escapes are read; a template piece is its cooked value, or its raw text where the cooked value is
  undefined (an invalid escape in a tagged template). A JSX text is compared as JSX renders it: each
  line trimmed of the white space at its ends, lines that hold only white space dropped, and the
  rest joined by one space, so re-indenting JSX changes nothing; a JSX text that holds only white
  space is not part of the shape. The arrow and `new` forms a formatter chooses between
  (`x => x` and `(x) => x`, `new Foo` and `new Foo()`) are the same shape. Session's call for syntax
  over tokens: a formatter's rewrap or a quote change then never splits a group.
- **SAME-2** Each word `duplicationIgnore` lists (TUNE-4) loosens SAME-1: with `names`, any name
  matches any name; with `strings`, any string literal, template piece or JSX text matches any
  other; with `numbers`, any number literal, a `bigint` among them, matches any other. A regular
  expression literal counts as a string. Session's call for the three words (owner: those three
  switches).
- **SAME-3** Two copies of one group never overlap. Where a run repeats back to back (`a; a; a;`),
  the copies are taken from the start of the list without overlap. Session's call.
- **SAME-4** A group is reported only with its largest runs: no reported group's run can be made
  longer by a statement next to it, on the same side in every copy, and still have the same copies.
  A group whose every copy lies inside a distinct copy of another reported group that has at least
  as many copies is not reported; so the body of a duplicated function isn't reported apart from the
  function. A group with a copy the larger group lacks is reported, so two groups can share lines.
  Session's call.

### How big a run must be

- **SIZE-1** A run's size is its tokens and its lines. Its tokens are counted in each copy, and the
  group's is the fewest of its copies. Its lines are those of each copy from the first statement's
  first line to the last statement's last line that hold a token, and the group's is the fewest of
  its copies. Session's call: a blank or comment line doesn't make a run bigger. So a copy's span,
  from its first line to its last, can be longer than the group's line count.
- **SIZE-2** A group is reported when its tokens are at least `duplicationMinimum`, its lines are at
  least `duplicationMinimumLines` where that is declared, and it has at least `duplicationCopies`
  copies (TUNE-1 to TUNE-3). Session's call.

### The declarations

- **TUNE-1** `duplicationMinimum`, a key the reading spec's KEY-1 already names, is the smallest run
  reported, in tokens. No pack declares it today (a search of the yakshaver pack's repository for
  the key, on 2026-11-23 while this spec was written, found none), so giving it a unit breaks
  nothing. Without it, no
  duplication is reported at all, and a notice names the key, as the complexity spec's LIMIT-2 does
  for a limit (design FIND-8).
- **TUNE-2** `duplicationMinimumLines`, a positive whole number, is the smallest run reported, in
  lines. Without it, a run has no smallest number of lines. Session's call.
- **TUNE-3** `duplicationCopies`, a whole number of 2 or more, is the fewest copies a group needs.
  Without it, a group needs 2. A value of 1 can't be used, and the line that refuses it says so
  (the reading spec's KEY-2). Session's call: 2 is the definition of a duplicate, not a limit.
- **TUNE-4** `duplicationIgnore`, a list holding any of the words `names`, `strings` and `numbers`,
  says what SAME-2 loosens. Without it, nothing is loosened. Another word can't be used, and the line
  that refuses it names the three. Where the project states it, the project's list replaces the
  pack's, an empty list included, unlike every other list key but TUNE-5's, which join (the reading
  spec's KEY-3). Among stacks of one source (Terms), the later stack's list wins, as for a number (the complexity
  spec's LIMIT-1). Session's call: the pack's default is expected to loosen `strings`, and a list
  that joined could never take it back. A list of words is used, not three true-or-false keys,
  because the hand-off format lets a project state only a list of text or a positive whole number.
- **TUNE-5** `duplicationTestCallees`, a list of callee chains (Terms), names the test callees
  (COPY-3). A call is a test callee's when its callee chain is exactly one in the list: with `it`,
  `it.only` and `it.each()` listed, `it(…)`, `it.only(…)` and `it.each(rows)(…)` are, and `it.skip(…)`
  is not. So `test.beforeEach(…)` and `test.describe(…)` are not test callees unless listed, and hooks
  keep counting. An entry that is not a callee chain as Terms writes one can't be used. The lists
  of one source join across stacks, unlike TUNE-4's, since two stacks can each bring a test
  framework of their own. Where the project states it in any stack, the project's joined list
  replaces the pack's joined list, so a project can take out a name it uses for its own function.
  Session's call for the joining. Without it, no call is a test callee's. Session's call,
  for the rule that a tool's convention is a declaration: the pack lists each form its test
  frameworks use.
- **TUNE-6** `duplicationAccepted`, a list of text, holds the groups a person has accepted (QUIET-1).
  Each entry is a group's fingerprint, as `audit` writes one (lower-case hexadecimal digits, as many
  as `src/findings.rs` gives every fingerprint), a `/`, and its number of copies, a whole number of 2
  or more, as `1f0a…9b/3`. An entry of another shape can't be used, and the line that refuses it
  gives the shape. The lists join, as lists do. Session's call for the shape (owner: keyed
  by the fingerprint and the copy count).
- **TUNE-7** Every number key takes one value for the invocation: the project's over the pack's, and among
  values from the same source the later stack's (the complexity spec's LIMIT-1). Both sides of a
  comparison use the declarations in force now (design BASE-3).

### The finding

- **DUP-1** `audit` reports each group as one finding under the rule `duplication` (design FIND-1).
  The SARIF report's list of rules gains `duplication`, after `complexity`.
- **DUP-2** Its first location is the first of its copies by path and line; for a new group, the copy
  FRESH-3 names. Its related locations are the other copies, by path and line (design IF-2). Each
  location has the copy's first line; its region runs to the copy's last line. Its message says how
  many copies there are, the size (SIZE-1's counts), and where each copy is, as its span from first
  line to last, with no code: "3 copies of a 12-line run (82 tokens): `src/a.ts` lines 10–23,
  `src/b.ts` lines 40–53, `src/c.ts` lines 7–20." Session's call.
- **DUP-3** Its result carries a SARIF property bag, `properties.duplication`, with `tokens` and
  `lines` (the group's size, SIZE-1), and `copies`, a list of `{ "path", "startLine", "endLine" }` in
  the order of DUP-2's locations. Session's call for the names. Like the complexity spec's CPX-3, the names are a draft until the swap (VERS-1).
- **DUP-4** Its fingerprint is made, as design IF-4 says, from the rule and the run itself, under the
  existing scheme `yakAnalyzer/v1`: SAME-1's form of the run's statements, with what SAME-2 loosens
  written as a placeholder, so two runs SAME-1 and SAME-2 count the same give the same fingerprint
  and no others do. It holds no path and no line, so the group keeps its fingerprint when its copies
  move or a copy is added. The form written into the digest is the builder's (Left to the builder);
  once the swap is made it is part of the scheme, and changing it breaks every recorded reason and
  every accepted group.
- **DUP-5** The fingerprint and every exact form (FRESH-1) are the same on every machine for the same
  code: they are built from the syntax tree's values, never from slices of the source, and every
  line ending inside a string, template piece or JSX text is written as one line feed, so a checkout
  with CRLF line endings and one with LF give the same fingerprints. Session's call: accepted groups
  are shared through the project's declarations, and a reason through the work item, across
  machines.

### Accepted groups

- **QUIET-1** A group whose fingerprint and number of copies are an entry of `duplicationAccepted` is
  not reported by `audit`, at the base or on the branch. A group of the same run with another number
  of copies, or a run changed so that its fingerprint changes, is reported again (owner: "keyed by
  the group's fingerprint and copy count"). Acceptance applies after SAME-4: an accepted group still
  keeps the smaller groups inside it from being reported, so accepting a function's group doesn't
  bring its body's group to light. Session's call.
- **QUIET-2** An entry that matches no group found is named in a notice, with its source (Terms), so
  a person can remove it from the project's declarations, or ask for it to leave the pack's.
  Session's call.
- **QUIET-3** Where a test's setup can't be shared, a person accepts its group the same way (owner).
- **QUIET-4** A reason given at the gate doesn't do this job. It covers a finding on its own branch
  (design IF-4), and under FRESH-1 any later branch that touches a copy of a reviewed group is held
  again, with a new reason asked for each time. An accepted group stays silent on every branch while
  its run and its number of copies stay as they were, and the decision is in the project's
  declarations, where a reader of the code can see it. The owner chose it after fallow's
  `ignoredClones` (owner).

### New against inherited

- **FRESH-1** With a base, a group is `new` when the branch added or changed it, and `unchanged`
  otherwise (owner: "changing a copy should count too, because if you have to change a copy you
  would most likely have to change the other copy as well"). It is `unchanged` when both of these
  hold, and `new` otherwise:
  - the base has a group with the same fingerprint (DUP-4) and the same number of copies, among the
    base's groups as SAME-3, SAME-4 and SIZE-2 leave them, accepted ones included (QUIET-1): the
    groups `audit` would report at the base before acceptance;
  - each of its copies pairs with a different copy of that base group in the same file, its path
    moved through git's renames (design BASE-4), and with the same **exact form**: SAME-1's form
    with nothing loosened, whatever `duplicationIgnore` says.

  So pasting a further copy is new (the count differs); fixing one line in one of two old copies is
  new (the copies split into shorter runs, whose fingerprints the base lacks); removing one of three
  copies leaves the other two new (the count differs); changing a string in one copy while
  `strings` is loosened is new by the second test, though the fingerprint stays; and editing one copy
  out of the group while pasting another elsewhere is new, since the pasted copy has no partner in
  its file (owner). Moving a copy to another file makes the group new too. A group the branch never
  touched is unchanged, even where the branch's edits around it moved its lines, since neither test
  holds a line. An edit next to every copy that makes the run longer (SAME-4) touches the group, and
  the longer group is new. A comment, or a formatter's pass or codemod that leaves the syntax as it was,
  changes no exact form and no token count (SIZE-1), so it makes no group new, with one exception:
  a rewrap can change a copy's line count, and a group that a rewrap takes over
  `duplicationMinimumLines` is new, since the base had no such group (Risks). This departs from design BASE-5, under which shortening an
  old duplicate is not new and inheritance is decided by a base run that contains the group; the
  amendment is owed (OWED-1). Session's call for matching by fingerprint, count and exact forms,
  close to the scheme `audit` already uses, over a containment test on the base's groups or the
  lines git finds changed: the first maps lines between the two trees, and the second marks every
  group new after a formatter's pass. Duplication's fingerprint holds no path, so this comparison
  doesn't go through the dead-code kinds' count of fingerprints with their paths moved
  (`Finding::moved` in `src/findings.rs`), as complexity's doesn't either.
- **FRESH-2** FRESH-1 reads no line differences: it needs only the base's groups, found from the
  base's files as `audit --base` already reads them, with the declarations in force now (design
  BASE-2, BASE-3). Session's call.
- **FRESH-3** A new group's first location is its first copy, by path and line, whose exact form no
  copy of the base's group of the same fingerprint has, or its first copy where there is none. Its
  message adds which copies are new or changed, or, where none is, what the base had: "At the base
  this run had 3 copies." or "The base reports no group of this run." Session's call.
- **FRESH-4** A file skipped at the base leaves the duplication findings compared, as the complexity
  spec's NEW-4 does for complexity. A script skipped at the base has no runs there, so a group with a
  copy in it has fewer copies at the base, or none, and is new. A script skipped at the base that
  the branch removed or changed is different: a copy the branch took out of it can't be seen at the
  base, so a group that lost it would look unchanged. So where the branch removed or changed any
  script skipped at the base (its content differs from the base's), every duplication group is new,
  and the notice says why and names the scripts. Where the branch left each skipped script as it
  was, the groups are compared as FRESH-1 says. A skipped `package.json`, workspace list or tsconfig
  changes no run. The notice the dead-code spec's COMMIT-4 gives says that duplication
  findings were compared too. Session's call: under COMMIT-4 one broken base file would make every
  group new, the second anti-goal.

### The command

- **SHOW-1** `yak-analyzer duplication [--hand-off <file>] [--all]` reads the project as `audit` does
  and prints one JSON document on standard output. Each option is taken at most once, in either order.
  It takes no base. Session's call, after the complexity spec's SOLO-1.
- **SHOW-2** The document holds `interfaceVersion`, `settings` (`minimumTokens`, `minimumLines`,
  `minimumCopies`, `ignore`, `testCallees` and `accepted`, each the value in force: as declared, or where a
  key isn't declared, what TUNE-2 to TUNE-6 say holds without it, so `minimumCopies` is 2 and the lists are
  empty; `minimumTokens` and `minimumLines` are `null` where undeclared), `groups` and `notices`. Each group holds `fingerprint`, `tokens`, `lines`, `copies` (as DUP-3) and
  `accepted`. Groups are sorted by their first copy's path and line. Notices are written as `graph`
  writes them. Session's call.
- **SHOW-3** By default, `groups` lists the groups `audit` would report. With `--all`, it also lists
  the accepted ones, with `accepted` true. Session's call.
- **SHOW-4** It exits as `audit` does: 0 when it ran, whatever it found; 1 with one line on standard
  error when it couldn't run; 2 for a command line it refuses. The refusal's list of what the binary
  accepts names it.
- **SHOW-5** It is a thin command over a function that `audit` also uses, and that stage 5's
  duplicates tool (design MCP-1) can use. Whether that tool also lists every group is stage 5's to
  decide; nothing is owed to MCP-1 now. The command's name, options and document are drafts until
  stage 5; yak never runs the command, so a change before then breaks nothing. Session's call, as the
  complexity spec's SOLO-5.

### The interface version

- **VERS-1** The interface version stays 1: a rule name, keys, a property bag and a command are
  added, and giving `duplicationMinimum` a unit breaks nothing, since no pack declares it (design
  IF-6; the complexity spec's VER-1). `CONTRIBUTING.md`'s paragraph on the interface version names
  the seventh rule, `duplication`, and the new keys, and says that DUP-3's property bag and DUP-4's
  form of a run are drafts until the swap and fixed from it. "The commands" gains the `duplication`
  command, and `audit`'s description says it reports seven kinds. The module comment of
  `src/declarations.rs` and `scripts/contributing.test.mjs` change to match.

### Fixtures

- **FIXT-1** Each rule above has a made-up project in `tests/`, with its expected output written in
  the test (CONTRIBUTING.md, "Fixtures"). These are at least:
  - COPY-1's runs at several depths, two functions differing only in name, and a class body;
  - COPY-2's imports, COPY-3's `describe` with hooks and `it` calls, an `it.each(rows)(…)`, and a
    helper in a test file;
  - COPY-4's Vue blocks, an MDX document, and data, generated and `.d.ts` files;
  - SAME-1's formatter rewrap, quote change, and added trailing comma; each word of SAME-2, alone and
    together; SAME-3's back-to-back runs; SAME-4's nested and overlapping groups;
  - SIZE-1's blank and comment lines, and copies of different line counts; each minimum of SIZE-2
    just met and just missed;
  - each key of TUNE-1 to TUNE-7, among them no `duplicationMinimum`, a `duplicationCopies` of 1, an
    unknown word in `duplicationIgnore`, a project's empty `duplicationIgnore` replacing the pack's,
    and a malformed `duplicationAccepted` entry;
  - QUIET-1's accepted group, the same run with one more copy, and QUIET-2's stale entry;
  - each case of FRESH-1: a copy pasted, a copy edited, one of three copies removed, one copy edited
    away while another is pasted, a copy moved to another file, a group with edits only around it, a
    group in a renamed file, a rewrap that leaves a group as it was and one that takes it over the
    line minimum, and a group only one side has; FRESH-4's skipped script;
  - DUP-5's same project with CRLF and with LF line endings, giving the same fingerprints;
  - JSX re-indented and rewrapped, an arrow's parameter with and without parentheses, `new Foo` and
    `new Foo()`, each giving the same group, fingerprint and counts; a template with an invalid
    escape in a tagged template;
  - FRESH-4's skipped script the branch changed, and one it left as it was; a project's
    `duplicationTestCallees` replacing the pack's;
  - types as code: two runs differing only in a type annotation, and repeated interfaces; a tagged
    template callee (``it.each`…`(…)``);
  - the command, with and without `--all`, and with and without the declarations.
- **FIXT-2** No fixture copies code from fallow or from the named repositories (CONTRIBUTING.md).

### The comparison with fallow

- **YARD-1** The comparison runs as the complexity spec's VS-1 runs: pinned commits, the same roots,
  clean worktrees, hand-written hand-off files, fallow's version the one the pack pins that day, each
  root run from that root. fallow keeps each repository's own duplication settings (`weak`, with
  fallow's defaults today) with its `ignoredClones` emptied, and runs its duplication command with
  JSON output (`fallow dupes --format json`; the plan's first step checks the name and flags on the
  pinned version, and reads each repository's duplication settings and `ignoredClones` again). The
  analyzer runs `yak-analyzer duplication --hand-off <file> --all` with the settings YARD-4 tries and
  no accepted groups, so neither tool silences anything and the counts compare like for like. The
  groups each repository's `ignoredClones` silenced are listed apart in the judged list, with the
  analyzer groups they match (YARD-2), since they are what the gate shows today. Session's call: the
  yardstick is fallow's detection, unsilenced.
- **YARD-2** Two groups are matched by place, not by tokens: a fallow group and an analyzer group
  match when at least two copies of one each share a line with a different copy of the other, in the
  same file. The test is the same from either side. A group that matches one or more groups of the
  other tool is shared; one that matches none is fallow-only or analyzer-only. Each group is counted
  once, in its own tool's counts, however many it matches. Session's call: the two tools' copies
  start and end at different places by design (COPY-1), and two shared copies are what make a
  duplicate.
- **YARD-3** Each fallow-only and analyzer-only group is judged by reading the code, with a reason: a
  refactor candidate, or not one. The judged list holds every judgment. The owner reads them and
  judges a sample of at least ten of each kind per repository, or all where there are fewer; where the
  owner and the comparing session (YARD-4) disagree, the owner's judgment stands, and its other judgments of
  that shape are read again (owner: the owner judges a sample, and differences are not failures).
- **YARD-4** The defaults come from the comparison: the agent session that runs it (not the session
  that wrote this spec) tries settings, starting from fallow's (`strings` loosened, 50 tokens, 5
  lines, 2 copies), and proposes the set with the fewest judged misses: analyzer-only groups judged
  no refactor candidate, plus fallow-only groups judged one. A tie goes to the set nearer fallow's.
  The owner chooses.

  The sets tried are the starting set and each set that changes one setting from it: tokens 35 or
  70; lines 3 or 8; `strings` not loosened; `names` loosened; `numbers` loosened. That is eight
  sets; copies stay 2. A further set is tried only where two changes each lowered the misses, and
  then the two together. A group is judged once: a group of another set with the same copies
  (YARD-2's match, both ways) keeps its judgment, and only groups no earlier set had are judged
  afresh. The owner's sample (YARD-3) is taken on the starting set, and again on the chosen set's
  groups the first sample didn't reach. Session's call: one change at a time shows what each
  setting does, and judging each group once keeps the work to the groups that differ. The values chosen go to the pack through OWED-1, and are written in the judged list.
  Session's call (owner: defaults set from the comparison).
- **YARD-5** A fallow-only group the owner judges a refactor candidate is either fixed, with a
  failing fixture of its shape first (design PROOF-5), or written in the judged list as left as it
  is, with the owner's reason, such as a copy that starts mid-statement. (This is not a
  `duplicationAccepted` entry, which is a project's.) An analyzer-only group the owner judges no refactor
  candidate is handled the same way. Nothing else is a failure. Branch audits run as the complexity
  spec's VS-5 runs them, two merged pull requests per repository at the least, each audited with both
  tools; each difference in what is marked new is judged, and one that comes from FRESH-1's stricter
  rule is written down, not counted (design PROOF-2a).
- **YARD-6** The time `audit --base` takes on the named root with the most files is measured before
  and after this item's change, as the complexity spec's VS-6 measures it, and written in the judged
  list. No bar is set.
- **YARD-7** The judged list holds the pinned commits, fallow's version and both tools' settings; per
  root the shared, fallow-only and analyzer-only counts with each judgment; the owner's sample; the
  settings tried and the defaults chosen; the branch audits; YARD-6's times; and each fix with its
  fixture. The owner reads it before the item closes (design PROOF-3).

### Owed to yakshaver

- **OWED-1** These go to yakshaver through the work item `2026-11-18-analyzer-design-spec-stage-2-amendments`,
  or its successor if it has closed, and not in this repository:
  - a design-spec amendment to FIND-9: the analyzer's own definition of a copy (COPY-1, SAME-1),
    its keys (TUNE-1 to TUNE-6), and fallow as a yardstick; and to PROOF-4: for duplication, the swap
    waits on YARD-5, not on matching fallow (owner);
  - an amendment to BASE-5's rule for a duplicated group, to FRESH-1's (owner);
  - an amendment to design DECL-2, which says a project's list adds to the stack's:
    `duplicationIgnore` and `duplicationTestCallees` are lists the project's value replaces
    (TUNE-4, TUNE-5);
  - the pack's declarations of the new keys, with YARD-4's defaults and each form of `it` and `test` as test
    callees, and the engine's way for a person to add an entry to `duplicationAccepted`;
  - a line in the notes of any pack release that changes the default `duplicationIgnore` or
    `duplicationTestCallees`, saying that accepted groups and recorded reasons for duplication can
    stop matching (Risks);
  - a check that yak passes a project's `duplicationIgnore`, `duplicationTestCallees` and
    `duplicationAccepted` lists to the analyzer unchanged, an empty list included, though their
    entries are neither globs nor paths, and a change to the hand-off schema's description, which
    says a project's list holds "globs or paths" (`tests/hand-off/hand-off-1.schema.json`). The
    schema's type already allows any list of text, so the format version stays 1.

  The item closes only once each is filed in that yakshaver item or written in the judged list as
  still owed there. Session's call, as the complexity spec's VS-7.

## Alternatives considered

- **fallow's definition as the floor, token runs matched token for token.** The gate's findings
  would not move at the swap, but copies would start and end mid-line, and every place the analyzer
  found a better group would count as a fault. The owner chose the analyzer's own definition, with
  fallow as a yardstick (owner).
- **A warning for two copies, a stop for three or more.** The owner's first answer. A warning that
  asks nothing of anyone piles up unread, and what the gate does with a finding is yakshaver's.
  Dropped for one kind of finding with no levels (owner).
- **A group new only when the branch pastes a copy, as fallow's audit and design BASE-5 count it.**
  It is gentler to a one-line fix in an old copy, but that fix is the moment the other copies need
  the same change. Rejected (owner), with accepted groups as the way out.
- **Leaving test files out, or counting them like any code.** Out misses repeated setup; in fills the
  gate with test bodies nobody will share. Hooks and helpers only (owner). Finding the test bodies by
  file was weighed and dropped: the hand-off has no key that marks test files, and the callees are
  what tell a test body apart.
- **Three true-or-false keys for the loosening.** Clearer to read, but the hand-off format lets a
  project state only a list of text or a positive whole number, so a project couldn't set them
  without a format change in yakshaver. Rejected for TUNE-4's list (session's call).
- **Lines the branch changed, read from git, for FRESH-1.** It catches every edit inside a copy, but
  a formatter's pass or a codemod would mark every group it touched new, though SAME-1 sees no
  change. Rejected for FRESH-1's exact forms (session's call).
- **Including the copied code in the finding.** It would make the message easier to act on, but it
  can carry a string the project would not want in a report. Rejected (session's call): the places
  are enough to open.

## Risks

- **The definition finds too little or too much on real code.** It is new, and nothing but the
  comparison tests it. YARD-3 judges every difference, the owner judges a sample, and YARD-4 sets the
  defaults from what was judged.
- **Statement-level copies miss duplication fallow's mid-line runs find,** such as a long repeated
  expression inside otherwise different statements. YARD-2 matches by area so these show as
  fallow-only, and YARD-5 has the owner decide whether each shape is worth a fix.
- **FRESH-1 marks many groups new on ordinary branches,** since any edit inside a copy does. That is
  the owner's choice; the gate's count of new duplication is in the branch audits (YARD-5), and an
  accepted group is the way out.
- **A group wrongly marked inherited,** where FRESH-1's fingerprint, count and exact forms match a
  base group that a person would call a different one. Each case of FRESH-1 has a fixture, and the
  branch audits look for it.
- **Code that calls its own function named like a test callee** (`test(…)` in a script that is no
  test) has that call's callbacks left out of detection (COPY-3). It is rare, and a project can
  state its own `duplicationTestCallees` without the name, which replaces the pack's (TUNE-5); the
  comparison notes any it meets.
- **The reading holds more for each file.** COPY-5 keeps something for every statement at every
  depth, which grows with deep nesting, on both trees at each audit. The builder keeps a digest
  and counts per statement, not its written form, and YARD-6's measurement includes the memory the
  largest named root takes before and after.
- **Duplication doubles work at each audit,** since both trees are read and grouped. COPY-5 keeps it
  inside the parse, and YARD-6 measures it. A cost found too high goes to the speed work, not here.
- **The fingerprint's form is hard to change after the swap,** since accepted groups and reasons hang
  on it. DUP-4 leaves the form a draft until the swap, and the comparison exercises it first.
- **A rewrap takes a group over the line minimum** and makes it new on a branch that changed no
  code (FRESH-1's exception). Only a group within a line or two of `duplicationMinimumLines` can be
  moved this way, and the branch audits (YARD-5) count how often it happens; if it is common, the
  line minimum is measured on the base's line count as well, a further amendment.
- **Changing `duplicationIgnore` or `duplicationTestCallees` changes groups and fingerprints,** since
  what is loosened is part of the run (DUP-4), and which statements join runs depends on the test
  callees (COPY-3). A change to the pack's default or a project's list then reports every accepted group
  again and voids the reasons recorded on branches in flight. It can't be avoided: copies that match
  only once names are loosened have no common run without the loosening. QUIET-2's notice names
  every entry that stopped matching, and the pack release that changes the default says so in its
  notes, owed with the defaults (OWED-1).
- **Work owed outside this repository is forgotten.** OWED-1 makes the item's close wait for it.

## Left to the builder

- The names of the modules, types and functions, and where in the parse the runs are read.
- How groups are found (hashing, a suffix array or another way), within SAME-3 and SAME-4.
- The fixed form a statement's tree is written out in to count its tokens (Terms, "A token"), within
  the rule that nothing a formatter may add or drop counts.
- The form of a run written into the fingerprint's digest (DUP-4).
- The form of a copy's exact form (FRESH-1): the same form as DUP-4's, with nothing loosened, or a
  digest of it.
- The exact wording of messages and notices, within DUP-2, FRESH-3, TUNE-1 and QUIET-2.
- The order of keys in the command's JSON.
