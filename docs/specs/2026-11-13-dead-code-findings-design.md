# The dead-code findings: entry points, five kinds, SARIF, and new against inherited

**Status:** draft.

**Work item:** `2026-11-14-dead-code-findings`, the third and last item of the analyzer's stage 1.

**Amends:** `docs/specs/2026-11-11-reading-a-project-design.md`: IMP-5 (the names an import brings
in are now recorded, USE-1), OUT-1 (`graph` gains the entry points and the uses, CMD-4), KEY-2 (the
two opaque keys get their shapes, ENTRY-6 to ENTRY-7, and a wrong shape now stops the run, ENTRY-6a), LIST-9 (this item says how a commit is
read, COMMIT-1 to COMMIT-5), PACK-6 (two more fields of a `package.json` are read, `bin` and
`scripts`, ENTRY-1), RUN-1 (a third command, `audit`, CMD-1) and RUN-4 (one more stop, CMD-3). It amends
`docs/specs/2026-11-11-skeleton-and-release-pipeline-design.md`, its PKG-5: the launcher tells the
binary it was started by a launcher (LIFE-1). It also answers the question that spec passed on in its non-goals: what
happens to the binary when the launcher is stopped (LIFE-1).

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. From it, this item builds FIND-1 to FIND-7 for the five dead-code kinds, FIND-10
to FIND-12, ENT-1 to ENT-6, the analyzer's side of IF-1 to IF-4 for the audit (not the MCP server,
and not IF-6a's check command), BASE-1 to BASE-6, the dead-code parts of AN-3 and AN-7, the patterns
of PROOF-1 listed in FIX-1, and the stage-1 parts of STAGE-1. "(design FIND-3)" after a sentence
names the requirement it follows from.

How to read the marks: **owner** marks what the owner decided in the brainstorm. Everything else not
taken from the design spec is the **session's call**: the session that wrote this spec decided it,
and the owner has not ruled on it.

## Intent

The analyzer reads a project and resolves every import (0.2.0), and it reports nothing. yakshaver
gates each work item on a dead-code audit, which today is fallow's, and the design spec's plan is to
replace fallow with this analyzer. That needs findings: which files nothing starts or imports,
which exports nothing uses, which imports lead nowhere, and which files import each other in a loop.
It needs them in SARIF, the format yak already reads, and it needs them marked **new** or
**inherited** against a base commit, since the gate weighs only what a branch adds.

This item builds all of that, and ends by running it on the named repositories. The owner's bar for
the analyzer is the design spec's (PROOF-4): no finding known to be false, and no true finding that
fallow reports and the analyzer misses (owner). Matching fallow exactly, false findings included, is
the floor: not the aim, and not a total failure (owner). This item checks the analyzer against its
own bar on real code; stage 2, the comparison with fallow, judges every disagreement between the two
and ends in the owner's decision to go on or stop.

Where this goes wrong unseen is a finding that is wrong in the quiet direction. A file that looks
used because something wrongly reaches it is never reported, and an inherited finding wrongly
matched to a new one passes the gate in silence. So the entry points, and every use the analyzer
counts, are printed by `graph` where a fixture can pin them down, and the end check audits real
branches against their bases, not only whole trees.

Terms. Every term the reading spec's Intent defines holds here as it does there, among them the
owner, a session, the builder, the plan, yak, a pack, a stack, the root, a package, the hand-off
file, a declaration, a notice, the named repositories, the stages, the comparison, the swap, the
launcher, the five targets and the report folder. Added here:

- **fallow** is the dead-code tool yak runs at the gate today, which the analyzer is to replace.
- **An entry point** is a file something outside the project's imports starts or loads: a program a
  `package.json` names, a test file a runner finds, a page a browser opens.
- **A reachable file** is an entry point, or a file an import of a reachable file leads to.
- **An export** is a name a file's `export` statements offer. **A use** of an export is a place
  another file reads it (USE-2).
- **Detection folders** are the folders the reading spec's PACK-4 names: with a hand-off file, the
  folders yak looked in for stacks; without one, the root and the workspace's folders. **The three
  kinds that depend on entry points** are unused file, unused export and unused type; they are
  reported only in packages whose own folder is a detection folder (ENTRY-9).
- **A finding** has a **rule**, a message, a location and a **fingerprint**: a short digest that
  names the same finding on the base and on the branch.
- **SARIF** is the Static Analysis Results Interchange Format, version 2.1.0, a JSON format that yak
  reads. A **notification** is SARIF's place for a notice from a run that went on.
- **A strongly connected set** is a largest set of files in which each file reaches each other file
  by imports.

## Goals, non-goals and anti-goals

**Goals:**

- The analyzer reports the five dead-code kinds of the design spec (FIND-1) as one SARIF report,
  from an `audit` command yak can run where it runs fallow's today.
- Given a base, every finding is marked new or inherited, with uncommitted edits and new files
  counted as the branch's. The one thing the working tree cannot show is a move git was not told of
  (INH-2).
- Entry points come from a project's standard files and from declarations, never from a tool's name
  written into the code (design ENT-5).
- The entry points and the uses behind every finding can be seen in `graph`'s output, and fixtures
  pin them down.
- On the named repositories, the analyzer runs on every root and on real merged branches, and the
  findings it gives are judged by reading the code.

**Non-goals:**

- Complexity and duplication (design FIND-8, FIND-9: stage 4). The two limit keys and the
  duplication minimum are still read and checked, and never used.
- The MCP server, and the command-line forms of its tools (design STAGE-5). The entry-point listing
  is part of `graph` until then (CMD-4).
- The check command of design IF-6a, a cache (design CACHE-1), and anything in yak's engine, the
  pack or the pack SDK (design ORDER-1).
- The judged comparison with fallow, the audits of merged branches with both tools, and the owner's
  decision to go on (design PROOF-2, PROOF-2a, PROOF-6). Those are stage 2.
- Fixing the speed. The end check measures it (LIVE-5); the design spec puts any work on it in
  stage 4.
- A release. As in the reading item, release-please opens a release pull request, and merging it is
  the owner's choice.

**Anti-goals,** failures even if it technically works:

- A finding marked inherited that the branch made. The gate would pass it in silence.
- A file kept alive, or an export counted as used, by something the output does not show.
- A tool's name, or a tool's convention, written into the code where a declaration could state it.
- A finding whose fingerprint holds a line number, or changes between platforms or runs.
- Output that differs between Windows, macOS and Linux, or with and without `node_modules` and build
  output.
- An audit that is killed leaving a process or a lock behind.
- A private repository's name or code committed here.

## Requirements

### Commands

- **CMD-1** The binary gains one command, `audit`. It takes `--hand-off <file>`, as `graph` does,
  and `--base <commit>`. It reads the project, finds the five kinds, and prints one SARIF 2.1.0
  report on standard output (REPORT-1). With `--base`, it compares with that commit (INH-1); without
  it, every finding is printed with no `baselineState` (design IF-1). The `base` field of a hand-off
  file is not used by `audit`: it is the MCP server's default (design MCP-1a), and yak hands the
  audit its base as an argument, as it fills `{base}` in fallow's command today. Session's call.
- **CMD-2** `--base` takes anything git names a commit by. A value that begins with `-`, or holds a
  control character, is refused before git sees it, as a wrong option is (exit 2): git would read
  it as an option, and the oldest git the analyzer supports has no way to mark the end of options.
  The analyzer asks git for the full name and uses only that afterwards, and prints it in the
  report (REPORT-4). A base that is not a commit in the repository stops the run (design BASE-6),
  with the name given and `git fetch` as the fix.
- **CMD-3** `audit` exits 0 when it ran, whether or not it found anything; a run that cannot go on
  stops as the reading spec's RUN-3 and RUN-4 say, exit 1 and one line on standard error. RUN-4's
  list gains two stops after its 7a: 7b, the base is not a commit (CMD-2); 7c, a partial clone
  with a git too old to read the base without the network (COMMIT-2). A wrong command or option
  exits 2, a fault in the analyzer exits 3, as for `graph`.
- **CMD-4** `graph`, with or without `--hand-off`, adds to what it prints (reading OUT-1): each entry
  point with every source that made it one (ENTRY-1); for each package, whether the three kinds
  that depend on entry points are reported in it (ENTRY-9); for each file, its exports, and for each import
  the names it brings in (USE-1); and each use the analyzer counted, with where it counts from: the
  file and line of the reference, of the import for a use of every export (USE-2), or the entry
  point and its source for an entry point's exports (USE-3). It does not print findings: those are `audit`'s. `graph` stays outside the interface
  version (reading OUT-2), so stage 5's entry-point tool may take its own form. Session's call: one
  builder's command shows the whole basis of every finding, and no command is fixed before the MCP
  tools are designed.
- **CMD-5** `interface` prints what it printed: the interface version stays 1. The rule names
  (REPORT-2), the fingerprint scheme (REPORT-3), the two opaque keys' shapes (ENTRY-6, ENTRY-7) and
  `audit`'s options join the drafts of the reading spec's RUN-7, which may change until the end of
  stage 2 without raising it.

### Entry points

- **ENTRY-1** A file is an entry point when one of these sources names it, and the analyzer keeps
  every source that did (design ENT-1):
  - the `package.json` of its package: `main`, `module`, `types` and `typings`, `bin` (a path or
    each value of an object), and every target of `exports`, each followed through every condition
    as an import is (reading RES-5, RES-6) and mapped back to source (reading MAP-1); a package
    declared **internal** (ENTRY-5) adds none from `exports`. A key of `exports` whose target is
    `null` hides its subpaths, as Node has it: no file is an entry point through a subpath a `null`
    key matches, also where a wider pattern would otherwise offer it;
  - a script of any `package.json` that runs it (ENTRY-7): the word is read from that
    `package.json`'s folder, and the file it names is an entry point wherever under the root it
    lies, in that package or another, as a root script's `node packages/a/run.mjs` names one in a
    workspace package. Session's call;
  - the `files` list of a tsconfig that is not data (design ENT-2); `include` makes nothing an entry
    point;
  - an HTML page, whose `<script src>` targets are entry points (design ENT-2); the page itself, of
    the kind "html", is never reported;
  - an `entries` glob, from a pack or from the project (design ENT-4);
  - a tool's configuration file read by a `configEntries` declaration (ENTRY-6).

  `bin` and `scripts` join the fields the reading spec's PACK-6 reads, in every form npm gives them
  (`bin` as a path or an object of paths, `scripts` as an object of strings); a value of another
  form stops the run at RUN-4's step 11, as any field of the wrong form does.

  An MDX document is not an entry point by itself: which documents a documentation tool loads is
  that tool's convention, stated by an `entries` declaration in the tool's stack (design ENT-5).
  Where one is declared, the files it imports are reachable through it (design AN-3).
- **ENTRY-2** An `exports` target with a `*` is mapped as a pattern (reading MAP-3): its folder and
  extension are rewritten to the source's, and every listed file the rewritten pattern matches is an
  entry point. The `*` has Node's meaning, not a glob's: it stands for any text, `/` included, and a
  `*` written twice stands for the same text both times. (`bin` has no patterns; a `*` in it is a
  character of the path.) A target that
  maps to no source adds no entry point, and the package is marked as having one (ENTRY-8).
- **ENTRY-3** An `entries` glob may name build output (design ENT-3). A glob that matches no listed
  file is mapped back, and the files the mapped glob matches are entry points. Mapping rewrites two
  parts of the glob and leaves the rest as written:
  - its folder: the glob's leading segments that hold no wildcard or brace. When that folder lies
    in the `outDir` of a tsconfig in the folder of the package the folder lies in, the `outDir` part
    is replaced as the reading spec's MAP-1 step 1 replaces it; otherwise its first segment is
    replaced by `src` (MAP-1 step 2). A glob whose first segment holds a wildcard has no folder to
    rewrite, and is not mapped;
  - its ending: when the last segment ends in an output form (`.js`, `.mjs`, `.cjs`, or a type
    declaration's ending), written alone or among braces, that ending is replaced by the source
    extensions of MAP-1, `{ts,tsx,mts,cts,js,jsx,mjs,cjs}`. Any other ending is left.

  A glob that matches neither way gives a notice naming the glob and its source. Session's call.
- **ENTRY-4** A pack's glob applies in each detection folder, relative to that folder; a project's
  glob applies once, relative to the root (design ENT-4, reading KEY-3).
- **ENTRY-5** `internal` names package folders, as paths relative to the root (project) or to each
  detection folder (pack). A package whose folder it names adds no entry points from its `exports`,
  so its exports' use is counted like any other file's (design FIND-5). `main`, `module`, `types`,
  `typings` and `bin` still make entry points in an internal package. Session's call: `bin` is a program, which is
  started and not imported.
- **ENTRY-6** `configEntries` is a list of objects, one for each configuration file a tool reads
  (design ENT-4). Each object holds:
  - `file`: a glob for the configuration file, matched relative to each detection folder (ENTRY-4),
    such as `vitest.config.{ts,mts,js,mjs}` for one in the folder itself or
    `**/vitest.config.{ts,mts,js,mjs}` for one at any depth. A file is matched once, from the
    deepest detection folder that holds it, so detection folders inside one another never apply one
    file twice;
  - `property`: the path to the value in the object the file exports, as a list of keys, such as
    `["test", "include"]`;
  - `value`: `"globs"` when the value is a list of globs or one glob, or `"folder"` when it is a
    folder to which `pattern` is joined;
  - `pattern`: with `"folder"`, the glob joined to the folder; absent otherwise;
  - `relativeTo`: `"config"` or `"detection"`: the value is relative to the configuration file's own
    folder, or to the detection folder it was matched from;
  - `default`: the list of globs used when the file states no readable value, relative to the same
    folder; it may be empty.

  Every field is required except `pattern`, which is required with `"folder"` and not allowed
  otherwise. The globs the declaration itself holds, `file`, `pattern` and `default`, are a
  declaration's globs: one the reading spec's KEY-5 cannot read stops the run as KEY-2 says
  (ENTRY-6a). A glob read from the configuration file is the project's tool's, not a declaration,
  so it never stops a run: one that KEY-5 cannot read, that is absolute or steps out with `..`, that
  begins with `!`, or that holds an extended pattern of the form `?(…)`, `*(…)`, `+(…)`, `@(…)` or
  `!(…)`, which some tools accept and KEY-5 does not, is passed over with a notice naming the file
  and the glob, and the rest are used. A value that leaves no glob uses `default`. A pack writes its
  `default` in KEY-5's form, braces in place of extended patterns.

  The value is read only when it is written as a string, or a list of strings, in the object that is
  the file's default export (`export default {…}`, `module.exports = {…}`) or in the one object
  passed to the call that is the default export (`export default defineConfig({…})`), reached
  through the property path by object literals alone. Anything else, a variable, a spread, a
  function, a regular expression, is not read, and `default` stands (design ENT-4). The file itself
  is an entry point, so a helper it imports is reachable (design PROOF-1, pattern 6). A matched file
  that does not parse is treated as any file is (reading PARSE-2). Session's call for the shape.
- **ENTRY-6a** A `configEntries` or `scriptCommands` value not of the shape ENTRY-6 or ENTRY-7 gives,
  an object missing a field or holding one it does not allow included, is a declaration that cannot
  be used (reading KEY-2): it stops the run at the reading spec's RUN-4 step 5, naming the stack, the
  key and the field. This replaces KEY-2's "accepted whatever it holds". A fixture holds each case.
- **ENTRY-7** A script of a `package.json` makes a file an entry point when a program that runs files
  is given the file as an argument. `node` is such a program for every package (design ENT-2); a
  `scriptCommands` declaration, a list of program names, adds others (design ENT-4). A script is read
  as a shell would split it, and only that far:
  - quotes are found first, from the left: text between a pair of single quotes, or of double
    quotes, is quoted, and nothing in it splits;
  - it is cut into commands at unquoted `&&`, `||`, `;`, `|` and `&`;
  - a command is cut into words at unquoted white space, and the quotes are removed;
  - leading words of the form `NAME=value` are passed over;
  - when the first remaining word is `node` or a declared program, every later word of the command
    that names a listed file, from the package's folder, with the reading spec's RES-3 and MAP-1
    applied, is an entry point; a word of the form `--option=value` is tried by its value.

  So `cross-env NODE_ENV=test node scripts/run.mjs` makes `scripts/run.mjs` an entry point when the
  pack declares `cross-env`, since every later word of a declared program is tried. Nothing else of
  shell syntax is read: no variables, no substitutions, no globs in words. A word that names no file
  makes nothing. Session's call for the shape and the reading.
- **ENTRY-8** A package with no entry point at all, and a package with a `bin` or `exports` target
  that maps to no source (ENTRY-2), is marked so, and each unused-file finding in it says that the
  likely fault is a missing `entries` declaration (design FIND-2). The message names the key, not a
  yak command: the commands are the engine's (design FAIL-1).
- **ENTRY-9** Unused files, exports and types are reported only in files whose package carries the
  reading spec's PACK-4 mark: a package whose own folder is a detection folder (design AN-4a). A file
  in no package is in none. For each package that holds analyzed files and is not marked, one notice
  names it and says the three kinds are not reported there and why; when files in no package are
  analyzed, one notice says how many and the same. These notices are given whether or not the
  hand-off file says detection was cut short, which has its own notice (reading HO-2). The other two kinds are
  reported everywhere under the root.
- **ENTRY-10** A computed `import()` or `require()` cannot be followed (design ENT-6). Each file's
  count is in `graph`'s output already; `audit` adds one notice for each file that has any, saying
  that what it loads must be declared in `entries`.

### Uses

- **USE-1** Each script, and each script block of a Vue component, is read for its names (this
  replaces the reading spec's IMP-5): what it exports, each with whether it is a type (an interface
  or a type alias) or a value (anything else: a function, a class, a variable, an enum, a namespace
  with values; a namespace that holds only types is a type; a name declared both as a value and as
  a type, as merged declarations are, is one value export, used when either is used), a name passed on from another file
  taking its kind from its origin (USE-2a); for each import, the names it brings in and
  the local binding of each, or that it brings in the namespace, or nothing (`import './x'`); and
  each `export … from`, with its names or `*`. `oxc_semantic`, the oxc crate that ties each
  reference to its binding, is added for this, pinned with the oxc group (design AN-2, reading
  DEP-3). Session's call: the binding analysis is oxc's, not a second writing of scope rules.
- **USE-2** A use is a reference, not a mention (design FIND-4):
  - a named or default import is a use of that export of the target only when its local binding is
    referenced somewhere in the importing file, in code or in a type; where the import leads to
    several files (reading RES-1), it is a use of that export in each of them;
  - an import whose binding is never referenced is not a use;
  - through `import * as ns`, `ns.name` and `ns["name"]` with a literal key use `name`; any other
    reference of `ns`, passing it on included, uses every export of the target;
  - `export { a } from './x'` and `export * from './x'` pass use through: a use of the re-exporting
    file's `a`, or of a name `*` covers, is a use of `./x`'s export, and so on through a chain;
  - `import { a } from './x'; export { a }`, an imported binding exported again, and `export
    default a` of one, pass use through the same way; the `export` statement itself is not a
    reference that uses `./x`'s `a`;
  - a TypeScript import type, `import('./x').T`, uses `./x`'s `T`;
  - a use counts from any file, reachable or not: an export an unused file uses is used until that
    file goes, and the unused-file finding says the file can go. Session's call;
  - `export * as ns from './x'` passes a use of `ns` on to every export of `./x`;
  - `import x = require('./x')`, `require('./x')` and `import('./x')` with a literal argument use
    every export of the target. Session's call: the safe direction, since what is read off the
    result is not followed;
  - an export used in its own file is noted, for FIND-3's message, and is not a use by another file.
- **USE-2a** A name a file offers only by passing it on, through `export … from`, `export *`, or an
  imported binding exported again, is not that file's own export and is never reported there. The
  name's origin is the file that declares it, found by following the chain; that is where an unused
  name is reported, once. Whether it is a value or a type is decided at the origin, whatever an
  `export type` along the way says. Session's call: one finding for one unused name.
- **USE-3** The exports of an entry point are all used: whatever starts the file consumes them
  (design FIND-3). That use passes through its re-exports as USE-2 says, so a package's public API
  reaches the files it re-exports from.
- **USE-4** An exported binding of a Vue component's script block counts as used when the
  component's template names it: as a tag, in PascalCase or in kebab-case, or as an identifier
  anywhere in the template's text (design FIND-11). The scan takes every identifier-shaped word in
  the template and is not a compiler; it can count more uses than a compiler would, never fewer.
  The same holds for an import's binding in `<script setup>`, which the template uses directly. A
  component's default export is never reported. Session's call.
- **USE-5** An MDX document's `import` statements (reading PARSE-4) are read for their names as a
  script's are (USE-1), and its `export … from` statements pass use through as a script's do
  (USE-2). Its imported binding is used when the document's text, outside its
  `import` and `export` statements and outside code blocks, names it as an identifier-shaped word
  (design AN-3). An MDX document's own exports are never reported.
- **USE-6** A file imported with a query, such as `?raw`, is read as text (design AN-3). It is
  reachable, so it is never an unused file, and when any import with a query leads to it, its
  exports are never reported. Its own imports are followed only when an import without a query, or
  an entry point source, also reaches it: read as text, it loads nothing.

### The five kinds

- **KIND-1** **Unused file** (design FIND-2). A listed script or Vue component that is not
  reachable, in a package ENTRY-9 reports in. Reachability follows every import that
  leads to files, of every form: type imports, `require`, literal `import()`, re-exports, a page's
  script, a Vue block's `src`. Not reported: a data file, a generated file, a type declaration
  file, an MDX document, an HTML page, a file of the kind "other" (design FIND-10). The location is
  the file, with no line.
- **KIND-2** **Unused export** and **unused type** (design FIND-3). An export of a reachable file
  that no other file uses (USE-2 to USE-6), in a package ENTRY-9 reports in, and in a file that is
  not data, not generated and not a type declaration file. A value is an unused export; a type is an
  unused type. The message says which of two cases it is: used only inside its own file, where the
  fix is dropping `export`, or used nowhere. The exports of an unreachable file are not reported:
  the unused-file finding covers them. Session's call. The location is the export's name.
- **KIND-3** **Unresolved import** (design FIND-6). An import whose outcome is unresolved (reading
  RES-1), in any file under the root that is not data, generated or a type declaration file. The
  location is the import's specifier. An ignored import, an outside import and a computed import
  are never this kind.
- **KIND-4** **Circular dependency** (design FIND-7). A strongly connected set of two or more files,
  or one file that imports itself, over the edges of static value imports: `import` and `export …
  from` statements that are not type-only, `import x = require`, and `require()` with a literal
  argument. Not an edge: `import type`, `export type`, an import all of whose names are marked
  `type`, a TypeScript import type, `import()`, and any import from or to a type declaration file,
  which loads nothing at run time. Session's call: `require()` loads on the spot and
  can cause the same load-order fault. One finding is one set. Its first location is the first import,
  by line, of the set's first file by path, that leads to another file of the set; its related locations are the
  set's other files, each at its first such import (design IF-2). The message names one cycle
  through the set's first file by path: the shortest, found by a breadth-first walk over the set's
  edges that takes each file's targets in path order, so the same set gives the same words on every
  platform. It names up to ten files, then how many more the set holds.
- **KIND-5** Nothing else is reported: the design spec's FIND-12 list holds, and complexity and
  duplication wait for stage 4.

### The report

- **REPORT-1** `audit` prints one SARIF 2.1.0 document with one run. The run names the tool, its
  version and the interface version; it lists the five rules (REPORT-2); and it holds one result per
  finding, ordered by rule in REPORT-2's order, then path, then line, then column, then fingerprint, so the same project gives the
  same bytes on every platform (reading RUN-8). Each result carries its rule, a message a person can
  act on, a first location with a path relative to the root, with forward slashes, and a line where
  the kind has one, related locations where it has them, `partialFingerprints` (REPORT-3), and with
  a base a `baselineState` of `new` or `unchanged` (design IF-2). Inherited findings are included,
  marked `unchanged`.
- **REPORT-2** The rule names are `unused-file`, `unused-export`, `unused-type`,
  `unresolved-import` and `circular-dependency`. Session's call, as drafts (CMD-5).
- **REPORT-3** A fingerprint is a SHA-256 digest, written as the first 32 hexadecimal digits, of the
  rule and the parts the design spec's IF-4 names, each part written with its length in bytes before
  it, so no two lists of parts give the same input whatever characters they hold:
  for an unused file its path; for an unused export or type its path and name; for an unresolved
  import its path and specifier; for a cycle its files' paths, sorted. It holds no line number. It is
  carried under one `partialFingerprints` key the builder names, with a version in the key, so a
  later scheme can sit beside it. Session's call for the digest.
- **REPORT-4** Every notice of the run is a notification of the run's invocation, with its kind, its
  sentence and its file where it has one (design IF-3). With a base, one notification names the base
  commit's full name, and every notice the base's reading gives, of any kind, COMMIT-4's included,
  is a notification marked as the base's.
  yak does not read notifications; a person reads them in the report, or in `graph`'s output, which
  prints the same notices.

### Reading a commit

- **COMMIT-1** With a base, the analyzer reads the project twice: the working tree, as it does now,
  and the base commit, from git's stored files (design BASE-2). Nothing is checked out, no worktree is
  made, and nothing is written. The reading spec's LIST-9 already makes everything after the file
  list work from a file list and a way to read a listed file; this item makes the readers that do
  not yet follow it, the `package.json`, script and tsconfig readers that read from disk directly,
  read through that one way, and supplies a second way for a commit.
- **COMMIT-2** The base's file list is the files git stores at that commit under the root, with
  every one tracked: the same paths the working tree's list would hold if it were checked out clean.
  A stored symbolic link and a submodule are left out, as the working tree's are (reading LIST-3).
  Contents are read in one batch from git, never with one start of git per file, as git stores them,
  with no filter applied. A line-ending filter changes nothing the analyzer reads but line endings.
  A file stored in another form than the working tree's, through a large-file pointer or a
  `working-tree-encoding` that is not UTF-8, does not parse or is not UTF-8 at the base, and is
  skipped (COMMIT-4). Lazy fetching is turned off for every start of git, so a partial clone that
  lacks a stored file makes git fail, which stops the run with git's line, and never fetches over
  the network (reading RUN-6). The git floor of the reading spec's DEP-2 stays where it is for this:
  when an audit with a base runs in a partial clone (one whose settings name a promisor remote) and
  the git found is older than the first version that honours the switch, the run stops before the
  base is read, with a line that names both versions and says a partial clone needs the newer git.
  A run without a base, and a run in a full clone, need nothing more. Session's call: it keeps the
  promise of no network without raising the floor for everyone. The plan names the oldest git version the commands this item adds work with,
  and raises the reading spec's DEP-2 floor if it must. A tsconfig above
  the root (reading CFG-3) is read from the same commit.
- **COMMIT-3** At the base, "git ignores this path" (reading RES-4, LIST-5) is answered by the
  working tree's ignore rules, the same question asked of git as for the working tree. Session's
  call: both sides then judge an import of a generated file the same way, as both sides use the
  declarations in force now (design BASE-3); the cost is that an ignore rule added on the branch also
  applies to the base.
- **COMMIT-4** At the base, the project's own files stop nothing (design AN-7): nobody can fix the
  base. Every check of the reading spec's RUN-4 from step 8 on, which on the working tree stops the
  run, at the base skips the file it is about, with a notice marked as the base's:
  - a `package.json` that is skipped, whether it cannot be read or a field has the wrong form, makes
    no package, and its files belong to the next package up;
  - a workspace list that is skipped adds no workspace folders;
  - a tsconfig that is skipped, or whose `extends` target is missing, governs nothing, and the search
    for a governing tsconfig goes on upward (reading CFG-2);
  - a source file that does not parse or cannot be read has no imports and no exports.

  A skipped file can leave the base with findings it never had, and their reach is not confined to
  one package: a skipped tsconfig changes how other packages' imports resolve, and a skipped source
  file's lost imports make the files it imported look unused. Those would make the same findings on
  the branch look inherited, which the gate passes in silence. So when the base skipped any file,
  no finding is marked inherited: every finding is new, and one notification names the skipped
  files and says why. Session's call: a broken file at the base is rare, since a file broken on
  purpose is declared data, and a gate that asks about every finding once is the safe direction.
  What still stops the run is what
  is not the project's: a base that is not a commit (CMD-2), and git failing while the base is read,
  which stops it as RUN-4's step 7 does, with git's own first line.
- **COMMIT-5** The hand-off file's declarations apply to both sides (design BASE-3), with the base's
  detection folders taken from the same file. Without a hand-off file, each side works out its own
  detection folders from its own workspace list (reading PACK-4), as a run without one does.

### New against inherited

- **INH-1** A finding of the working tree is inherited, `unchanged`, when the base has a finding of
  the same fingerprint, with INH-2's renames; otherwise it is `new` (design BASE-1). Several findings
  can share a fingerprint, such as two unresolved imports of one specifier in one file: when the
  working tree has more of one fingerprint than the base, the base's count of them, first by line
  and column, are inherited and the rest are new. A cycle is decided by INH-3 alone, not by its fingerprint. A
  finding only the base has is not printed.
- **INH-2** Renames (design BASE-4). The analyzer asks git for the files renamed between the base
  commit and the working tree, with git's own rename detection and its default similarity, for the
  files git tracks and the index's staged additions. Detection is turned on, and its limit on the
  number of files set to a fixed value the plan names, for every start of git, whatever the user's
  or repository's settings say (reading RUN-6a), so the answer is the same on every machine; when
  git reports that it gave up for the limit, a notification says so. Before fingerprints are compared, each base
  path git reports renamed is rewritten to its new path, so the base's fingerprints are made with the
  working tree's paths. An untracked file is never the new side of a rename: git cannot pair it
  without writing to the index. Session's call.
- **INH-3** Cycles (design BASE-5). A working-tree cycle is inherited when the base, after INH-2's
  rewriting, has a strongly connected set that contains all of its files, and the branch added no
  edge of KIND-4 between two of its files. An edge is a pair of files, the importing file and the
  file it leads to, with the base's paths rewritten by INH-2; so a specifier rewritten without
  changing its target, such as `./a` becoming `./a.js`, adds no edge. Otherwise the cycle is new. A
  new cycle's first location is the first import, by path and line, that makes an edge the branch
  added, and its message names that import and the files. A new cycle with no added edge, which
  happens only when the base's set was cut by a file COMMIT-4 skipped, takes KIND-4's first location.
- **INH-4** The two sides are read in parallel where the builder finds it worth it; the output is
  the same bytes either way (reading RUN-8).

### Life of a run

- **LIFE-1** When the npm launcher (skeleton spec) that started the binary ends, as when yak stops
  it, the binary ends soon after, on every target, writing nothing. A binary started directly is
  stopped directly, and its behaviour does not change: nothing about it depends on what its
  standard input is. The launcher still passes standard input, output and error through as the
  skeleton spec's PKG-5 says; what it adds is only what lets the binary know it was started by a
  launcher that has since gone, such as the launcher's process id in a variable of the binary's
  environment, which the binary checks while it runs. The builder chooses how, and a test stops a
  launcher during a long audit and checks that its binary is gone within five seconds. This amends
  the skeleton spec's PKG-5 by that addition. Session's
  call; the reading spec passed this question to the first command yak starts and stops.
- **LIFE-2** Every git process the analyzer starts ends when the analyzer does: each is read to its
  end or has its input closed, and git's optional locks stay off (reading RUN-6a), so a killed audit
  leaves no lock. A git process that is working when the analyzer is killed may run on until it
  next reads or writes, and then ends. A test kills an audit while it reads a base and checks that
  nothing under the root changed and that within thirty seconds no git it started is still running.
- **LIFE-3** No project's size or shape, and no ordinary depth of nesting in a file, runs the
  analyzer out of stack. Windows' main thread has 1 MB of stack, and a process that runs out of
  stack writes nothing (reading RUN-3). Fixtures, generated by the test and not committed, hold an
  import chain, a cycle and a re-export chain of 10,000 files each, and a file nested 1,000 levels
  deep in expressions and blocks, and pass on every target.

### Fixtures

- **FIX-1** Each pattern of the design spec's PROOF-1 that this item's findings depend on has a
  fixture, written as the reading spec's FIX-1 writes one, with its expected `graph` facts and its
  expected SARIF findings: 1 (tests compiled into build output, found by a runner given a glob of
  built files, declared through `entries`), 2 (tests found by a project's runner script named in a
  `test` script), 3 (a `bin` that points at bundled output, settled by an `entries` declaration), 4
  (programs started by a path string from a test, settled by `entries`), 6 (a configuration file
  found by name that imports a helper), 7 (a wildcard `exports` pattern expanding one `*` twice, and
  the same package internal), 8 (an `exports` target with no source), 11 (a file live only through a
  tsconfig's `files`), 16 (Vue components used only in a template, an import used nowhere, and an
  MDX document naming one export and not another), 17 (an export used only in its file, one used
  nowhere, an unreferenced import, a same-name local elsewhere, namespace use per member), 18 (a
  static cycle, a loop closed only by `import()` or `import type`, and on a branch an inherited
  cycle, a file joining one and an import added inside one), 19 (a computed `import()`), 22 as far
  as these kinds go (an inherited finding, a new one, and a renamed file), and 25 (each form of
  ENTRY-6).
- **FIX-2** Further fixtures hold what this spec adds: each form of ENTRY-7's script reading, a
  quoted path, an assignment before the program, a declared wrapper and a word that names no file;
  re-export chains with `export *` and `export * as`; an unused file's exports not reported; ENTRY-8's
  message; ENTRY-9's notice for an unmarked package; a base whose tsconfig does not parse (COMMIT-4);
  a base that is not a commit (CMD-2); and the tests of LIFE-1 to LIFE-3.
- **FIX-3** Every SARIF report a fixture prints is checked against SARIF 2.1.0's JSON Schema, a copy
  of which lies among the tests with where it came from and its digest, as the hand-off schema's copy
  does (reading HO-1).
- **FIX-4** Each fixture with a base makes its base as a commit in the fixture's repository and its
  branch as later commits and uncommitted edits, so the working tree and the base are both read as
  the audit reads them.

### The check on real repositories

- **LIVE-1** When the fixtures pass, the session runs the analyzer on the named repositories, on the
  owner's machine, on clean worktrees with nothing installed and nothing built, as the reading
  spec's LIVE-1 does: the same repositories that check covered, and each web app in a sub-folder a
  second time with the sub-folder as the root (owner). The names come from the record of the design
  spec's work item. Worktrees are made in a temporary folder outside every repository and removed
  when their runs are judged.
- **LIVE-2** Each run has a hand-off file written by hand, kept in the report folder as the reading
  spec's LIVE-1 keeps one. It holds the declarations a pack would state, in a stack of their own, and
  those the repository would state, as the project's, kept apart as yak keeps them (design PROOF-2):
  `data` and `generated` from the reading item's files, and `entries` and `internal`, from the
  session's reading of each repository. `configEntries` and `scriptCommands` describe what a tool
  does, so only the pack's stack holds them (reading KEY-2), as the pack will in stage 3. The report lists each
  declaration with its reason and the number of files it matches, and the owner reads that list when
  validating the plan that carries this check.
- **LIVE-3** Each root gets an `audit` with no base, and an `audit` of one recent merged pull request
  with changes to code. Where the repository keeps the pull request's last commit, through a merge
  commit's second parent or the pull request's ref on the remote, the worktree is made at that commit
  and the base is its merge base with the default branch as it stood before the merge. Where it does
  not, as after a squash or a rebase, the worktree is made at the last commit the merge put on the
  default branch and the base is the commit just before the first. The
  session judges every finding of the run with no base by reading the code, kind by kind: true, or
  false with why. Findings with the same cause are one case and are judged once. Every finding the
  branch audit marks new is judged too, and so is every finding it marks inherited whose file the
  branch changed, since that is where a wrong match would hide.
- **LIVE-4** The counts of fallow's findings on the same roots, by kind, with fallow's own entry
  configuration as the reading item ran it, are set beside the analyzer's, and not judged here:
  judging the differences is stage 2.
- **LIVE-5** The time of each run is recorded, split into the working tree's reading, the base's,
  and the rest, and set beside the design spec's bars of 3 seconds after an edit and 10 seconds at
  the gate (design SPEED-1, SPEED-2). A miss is reported to the owner and not fixed here.
- **LIVE-6** Each false finding becomes a made-up fixture of its shape before it is fixed (design
  PROOF-5). One that cannot be fixed without something a later stage builds is written down with
  the stage it waits for. Fixtures copy no code from the repositories.
- **LIVE-7** The judged lists stay in the work item's report folder and are committed nowhere
  (design PROOF-3). The record carries the counts, the times and the outcome.
- **LIVE-8** The item is done when LIVE-3 leaves no finding known to be false, other than those
  LIVE-6 wrote down, and the fixtures pass on every target.

### Dependencies and the repository's own text

- **DEP-1** Each crate added is pinned to one exact version, with its reason in `Cargo.toml`, as the
  reading spec's DEP-1 asks: `oxc_semantic` with the oxc group (USE-1), and a SHA-256 crate
  (REPORT-3). A crate for SARIF's types is not needed; the report is written with `serde_json`.
- **DOC-1** `CONTRIBUTING.md` gains `audit` and what `graph` now prints. The README's status says
  that the analyzer reports the five dead-code kinds, and the test that holds the README to its
  status text changes with it.

## Alternatives considered

- **Several work items, one per kind.** Each would be smaller. Rejected: the kinds share entry
  points, uses and the report, and the end check needs all five to judge the analyzer against its
  bar. The work splits into plans instead, as the reading item's did.
- **A separate `entries` command now.** The design spec's stage 1 names an entry-point listing.
  Rejected: its fixed form belongs to the MCP tool of stage 5, and a command under the interface
  version fixed before that tool is designed would be changed. `graph` shows the entry points with
  their sources meanwhile (CMD-4).
- **Taking the base from the hand-off file when `--base` is absent.** One source of the base for
  yak. Rejected: the design spec's IF-1 has an audit without a base print no `baselineState`, and an
  audit that silently compared would make a whole-tree run look like a branch run.
- **Checking the base out into a temporary worktree.** It reuses the reading as it is. Rejected by
  the design spec's BASE-2 and AN-6: the analyzer writes nothing, and registering a worktree is a
  write to `.git`.
- **Answering "ignored" at the base from the base's own ignore files.** More faithful to the base.
  Rejected: git answers that question only from a working tree, and both sides already use the
  declarations in force now (design BASE-3); COMMIT-3 keeps the two consistent.
- **Leaving renames or the cycle rule to stage 2.** It would shrink this item. Rejected: without
  INH-3 an import added inside an old cycle passes the gate in silence, and without INH-2 a rename
  turns every inherited finding new, which the end check's branch audits would show at once.
- **Counting uses by name matching, without `oxc_semantic`.** No new crate. Rejected: a same-name
  local in another file, or a shadowed name, would count as a use, which is the quiet direction the
  anti-goals name.
- **Following what is read off a `require()` or an `import()` result.** `const { a } = require('./x')`
  uses only `a`. Rejected for now: a destructuring is the easy case, and anything else would fall back
  to all exports anyway; counting all of them is the safe direction and the fixtures show it. Stage 2
  shows whether it hides true findings.
- **Reporting the exports of an unused file too.** fallow does. Rejected: the file finding already
  says the whole file can go, and each export finding beside it is noise. For the bar against fallow
  (design PROOF-4), an export fallow reports in a file the analyzer reports unused counts as found:
  the unused-file finding covers it.
- **Judging a sample of the findings in LIVE-3.** Faster on a repository with hundreds. Rejected: a
  false finding of a kind the sample missed would pass, and the owner's bar has no false finding.

## Risks

- **Reading twice is too slow.** The reading item measured about 2.0 seconds for one reading of the
  largest named repository, so an audit with a base starts near 4 seconds, over the 3-second bar
  after an edit. LIVE-5 measures it; INH-4 lets the two readings run in parallel; a cache is stage
  4's, if the numbers call for one (design CACHE-1). The bars are not conditions of the swap by
  themselves (design SPEED-4).
- **A wrong match marks a new finding inherited.** The fingerprint holds no line, so two findings
  with the same parts are one: an unused export removed and another of the same name added in the
  same file is inherited. That is the design spec's scheme (IF-4). LIVE-3 judges every inherited
  finding in a file the branch changed.
- **git pairs renames wrongly.** A rename detected between two unrelated files of similar content
  carries an old finding onto a new file. git's default similarity is used, and the branch audits
  of LIVE-3 show how it behaves on real changes.
- **The ignore rules of the working tree differ from the base's.** COMMIT-3 accepts it; an import
  the branch's new ignore rule makes ignored is ignored on both sides, so it is neither new nor
  inherited, which is what the design spec's BASE-3 asks of declarations too.
- **A use counted that a compiler would not see.** The template and MDX scans (USE-4, USE-5) and the
  rule for `require()` and `import()` (USE-2) count more uses than are real, so some true unused
  exports go unreported. That is the safe direction; stage 2 compares with fallow and shows the cost.
- **A script read wrongly.** ENTRY-7 reads only a small part of shell syntax. A file a script starts
  in a form it does not read looks unused; LIVE-3 shows each one, and the fix is a declaration, or a
  form added with its fixture.
- **A configuration value the analyzer cannot read.** ENTRY-6 falls back to the declared default, so
  a project with a computed `include` gets the tool's default. A finding that follows is settled by
  an `entries` declaration.
- **The binary outlives the launcher.** A killed launcher on Windows does not end its child by
  itself. LIFE-1 makes the binary end on its own and tests it on every target.
- **A stack overflow writes nothing.** yak would see an audit that cannot run, with no line. LIFE-3
  removes recursion over the graph and tests the depth.
- **The draft keys and rule names change in stage 2.** Only hand-written files and fixtures hold them
  until then (CMD-5).
- **A git too old to turn lazy fetching off.** git's switch for it is recent; an older git in a
  partial clone could fetch a missing file. COMMIT-2 stops an audit with a base in that case, the
  plan names the version, and the README says partial clones need it.
- **A value from a caller turned into a git option.** The base is the first value from outside the
  analyzer that reaches git's command line; one beginning with `-`, such as `--output=…`, could make
  git write a file. CMD-2 refuses such a value before git sees it, and only git's own full name for
  the commit is used afterwards. Every other path given to git comes from git's own listings or the
  hand-off file's checked fields (reading HO-2), and the analyzer still writes nothing and uses no
  network (reading RUN-6).
- **The SARIF schema's copy goes stale.** It records its source and digest (FIX-3); SARIF 2.1.0 is a
  fixed standard.

## Left to the builder

- How the work is split into plans, and their order. The session's guess, which the owner agreed
  with as a shape: entry points and their listing in `graph`; names and uses with unused files;
  unused exports and types with Vue and MDX; unresolved imports and cycles with the report and
  `audit`; reading a commit and new against inherited, with LIFE-1 to LIFE-3; the check on real
  repositories.
- Where in SARIF's run the interface version and the base's mark on a notification are written,
  such as the run's or the notification's property bag.
- The `partialFingerprints` key, the separator of REPORT-3, the field names `graph` adds, the notice
  kinds, and the wording of every message and line.
- How the readers that read from disk are moved behind the reading spec's LIST-9 (COMMIT-1), and how
  git is asked for a commit's files and contents and for renames, within a small, fixed number of
  starts.
- How LIFE-1 and LIFE-3 are met, such as walks written as loops over an explicit stack and work run
  on a thread with a larger stack.
- Which merged pull request each repository's branch audit uses (LIVE-3), named in the report.
