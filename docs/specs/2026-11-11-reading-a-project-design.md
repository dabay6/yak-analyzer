# Reading a project: its files, their imports, and the hand-off file

**Status:** draft.

**Work item:** `2026-11-12-reading-a-project`, the second item of the analyzer's stage 1.

**Amends:** `docs/specs/2026-11-11-skeleton-and-release-pipeline-design.md`, its section "The
binary". BIN-2 and BIN-4 there describe the skeleton: a binary with one command, which reads no
file and starts no program. RUN-1, RUN-2 and RUN-6 below say what holds from this item on. That
spec also passed one question to "the item that adds the first long-running command": what happens
to the binary when the launcher is stopped. This item passes it on again, and the non-goals say why.

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. That repository is not public yet, so this spec says in its own words
everything a builder needs from it. From it, this item builds AN-3 to AN-7 (what is read, from
where, and what stops a run), the parts of ENT-2 and ENT-3 that resolving an import needs, FIND-10
as far as it changes what is read, the analyzer's side of HAND-2 and HAND-6 (reading the hand-off
file), IF-3 (how a run fails), IF-5 (a run with no hand-off file), IF-6 and IF-6a (the interface
version and the check of a hand-off file), and the first draft of ENT-4's keys.

How to read the marks: "(design AN-4)" after a sentence names the design spec's requirement that
sentence follows from. Everything else is the session's call: the session that wrote this spec
decided it, and the owner has not ruled on it. The owner was offered questions and answered "use
the approved analyzer design", so no sentence here is marked as the owner's.

## Intent

The analyzer is published and installs through yak, and it analyzes nothing: the binary answers its
version. Every finding the design spec asks for (an unused file, an unused export, an unresolved
import, a cycle) is a statement about a graph: which files a project has, and which file each
import leads to. That graph does not exist yet, and nothing can be found until it does.

Building it is where an analyzer of this kind goes wrong without anyone seeing it. An import that
is resolved to the wrong place, or quietly to nowhere, does not fail. It makes a live file look
dead, or a dead file look alive, in a later item's findings. So this item builds the reading alone
and makes it visible: one command prints everything that was read and what every import resolved
to, made-up projects pin that output down, and at the end the command is run on real repositories,
where each import it could not resolve is judged by reading the code.

Three rules of the design spec shape all of it:

- **The result is the same whether or not dependencies are installed and the project is built**
  (design AN-5). yak runs the analyzer in fresh worktrees that have neither `node_modules` nor
  build output.
- **It does not guess through what it cannot read** (design AN-7). A broken file that git tracks
  stops the run with one line saying what to fix.
- **A tool's convention is a declaration, never code in the analyzer** (design ENT-5). What a test
  runner or a bundler starts reaches the analyzer as data in a file yak hands it.

Terms:

- **The owner** is the person who owns this repository and approves its specs and plans. **A
  session** is an AI agent's working session on the owner's behalf.
- **yak** is the command of yakshaver, the developer tool the analyzer is built for. A **pack** is
  a package of content yak loads. A **stack** is a pack's description of one technology, such as
  TypeScript, which yak detects in a project.
- **The root** is the folder the analyzer reads. For yak it is a checkout's root.
- **A package** is a folder that holds a `package.json`, with the files under it that no nearer
  `package.json` claims.
- **The hand-off file** is a JSON file yak writes for one checkout and hands to the analyzer. It
  says where the root is, where yak looked for stacks, and which **declarations** are in force.
- **A declaration** is a key and a value that states a convention, such as "these files are test
  fixtures". A pack states declarations for a stack; a project may add its own.
- **A notice** is something a person should know about a run that went on, such as a file that was
  skipped.
- **The named repositories** are the real repositories the analyzer must be right on. They are
  named on the record of the design spec's work item in yakshaver and nowhere in this repository.

## Goals, non-goals and anti-goals

**Goals:**

- Given a root, the analyzer knows the project's files as git knows them, its packages, and for
  every import in every file it reads, exactly one outcome: the file it leads to, a package
  outside the project, a path git ignores, or nothing.
- That knowledge is the same on every platform, and the same with and without installed
  dependencies and build output on disk.
- It reads every kind of file the design spec names as an importer: scripts, Vue components, MDX
  documents and HTML pages.
- It reads the hand-off file yak will write, refuses one it cannot use with a line that says what
  to change, and applies the two declarations that change what is read.
- One command prints all of it, so a wrong reading can be seen and tested before any finding is
  built on it.
- On the named repositories, every import the analyzer cannot resolve has been judged, and none is
  known to be false.

**Non-goals:**

- Any finding, any report in SARIF, and any audit command. Each comes in a later item of stage 1.
  This item's output shows unresolved imports, but they are not yet findings: they carry no
  fingerprint and no rule, and nothing compares them with a base.
- Entry points: which files something starts. The next item reads them. This item resolves an
  import that passes through a package's `exports`, and stops there.
- Whether a name is used: bindings, references, a file's exports, the names a Vue template or an
  MDX document's text mentions. `oxc_semantic` is not used until an item counts uses.
- The value shapes of the two declarations that describe what a tool does (KEY-1). The item that
  reads entry points fixes them.
- A cache (design CACHE-1), the MCP server, and new against inherited.
- A release. Nothing installs the analyzer from npm until the design spec's swap, and the
  comparison runs a built binary. release-please will open a release pull request for this item's
  commits; whether to merge it is the owner's choice, and it needs trusted publishing set up first,
  as `CONTRIBUTING.md` says.
- What happens to the binary when the launcher is stopped. The command this item adds is the
  builder's own and yak never starts it. The audit is the first command yak starts and stops, and
  its item states and tests this.

**Anti-goals,** failures even if it technically works:

- An import that silently resolves to the wrong place or to nowhere. Every import the analyzer
  reads has an outcome the output shows.
- Output that differs between Windows, macOS and Linux for the same project, or between a fresh
  worktree and an installed, built checkout.
- A file in `node_modules` or in build output that git ignores changing any result.
- The analyzer running a project's code, writing inside the root, or using the network.
- A tool's name in the analyzer's code.
- A real project's code or a private repository's name committed here.

## Requirements

### Commands, and how a run fails

- **RUN-1** The binary has four commands. The version option stays as it is. The three new ones:
  - `interface` prints one line of JSON holding the interface version (RUN-7) and the list of
    hand-off formats the analyzer reads, which is `[1]` (design IF-1).
  - `check --hand-off <file>` reads a hand-off file and says whether the analyzer can use every
    declaration in it, without reading the project (HO-6).
  - `graph` reads the project and prints what it read (OUT-1). It takes `--hand-off <file>`, and
    runs without one (HO-5).
- **RUN-2** Given a command or an option it does not have, or a command without an argument it
  needs, the binary writes one line on standard error naming what it was given and what it
  accepts, and exits 2, as the skeleton did. This replaces the skeleton spec's BIN-2.
- **RUN-3** A run that cannot go on exits 1, prints nothing on standard output, and writes exactly
  one line on standard error: what failed, and what fixes it (design IF-3). yak relays the first
  line of standard error as the reason an audit could not run, so nothing else is ever written
  there. The `check` command is the one exception: it writes one line for each problem it finds
  (HO-6).
- **RUN-4** These stop a run, each with its own line:
  - the root is not inside a git work tree, or git cannot be started or fails: the root, and git's
    own first line where it gave one;
  - the hand-off file cannot be read, is not JSON, or lacks a field format 1 requires: the file,
    and the field;
  - the hand-off file states a format the analyzer does not read (HO-3);
  - a stack's declarations were written for another interface version (HO-4);
  - a declaration cannot be used (KEY-2);
  - a `package.json` that makes a package cannot be read as a JSON object: the file (PACK-2);
  - a tsconfig that governs a file the analyzer reads cannot be read: the file (CFG-4);
  - a file git tracks does not parse: the file and the line, and that a file broken on purpose is
    declared as data (PARSE-2).
- **RUN-5** A run that goes on collects notices. Each has a kind, the file it is about where there
  is one, and a sentence. They are part of the output (OUT-1) and are never written on standard
  error. Each requirement below that says "with a notice" adds a kind.
- **RUN-6** The analyzer never runs a project's code, which includes its configuration files. It
  writes nothing inside the root and uses no network. The one program it starts is `git`, and only
  to read (design AN-6). This replaces the skeleton spec's BIN-4.
- **RUN-7** The interface version is the whole number 1. The design spec makes it cover the
  commands and options, the declaration keys and their shapes, the rule names, the fingerprints
  and the MCP tools, and raises it when a release breaks one of them (design IF-6). Until the end
  of the design spec's stage 2 the keys of KEY-1 and the commands of RUN-1 are drafts: changing a
  draft does not raise the number. Nothing but hand-written files states an interface version
  before then.
- **RUN-8** For the same project and the same hand-off file, the output is the same bytes on every
  platform. Every path in it is relative to the root and written with forward slashes, and every
  list is sorted.

### The file list

- **LIST-1** The root is the folder the hand-off file names, whatever folder the analyzer was
  started in (design HAND-2). Without a hand-off file it is the folder the analyzer was started in.
  The root must be inside a git work tree. It may be a sub-folder of one; the analyzer then reads
  only what lies under it.
- **LIST-2** The file list is every file under the root that git tracks, and every file there that
  is untracked and that git does not ignore. What is ignored is what git itself answers, from
  every `.gitignore`, from `.git/info/exclude` and from the user's global excludes (design AN-4).
  The analyzer asks git for the list and keeps, for each file, whether it is tracked.
- **LIST-3** The list never reaches into a nested repository, a submodule or another worktree
  (design AN-4). A tracked file that is missing from the working tree is not in it. A symbolic
  link is never followed and never read.
- **LIST-4** Whether a path exists is asked of the file list, never of the file system. Only the
  content of a listed file is read from disk. So a file git ignores cannot change a result by
  being on disk, which is what makes the result the same with and without `node_modules` and
  build output (design AN-5), and a path matches only when its letters' case matches, on every
  platform (RES-10).
- **LIST-5** For a path that is not in the list, the analyzer can tell whether git ignores it, with
  git's own answer, whether or not the path is on disk. RES-1 needs this to tell an ignored
  target from a missing one.
- **LIST-6** A listed file has one kind, from its name: a script (`.ts`, `.tsx`, `.mts`, `.cts`,
  `.js`, `.jsx`, `.mjs`, `.cjs`); a type declaration file (`.d.ts`, `.d.mts`, `.d.cts`); a Vue
  component (`.vue`); an MDX document (`.mdx`); an HTML page (`.html`); a package manifest
  (`package.json`); a tsconfig (`tsconfig.json`, any `tsconfig.*.json`, `jsconfig.json`); or
  other. A file of the kind "other", such as CSS or JSON, is never read (design AN-3).
- **LIST-7** A path that is not valid UTF-8 is left out of the list, with a notice.

### Packages

- **PACK-1** Every `package.json` in the file list that is not data (KEY-4) makes a package,
  whether or not a workspace lists it (design AN-4). A file belongs to the nearest package at or
  above its folder. A file with no `package.json` above it under the root belongs to no package,
  which is normal for a repository whose root has none.
- **PACK-2** A `package.json` that makes a package and cannot be read as a JSON object stops the
  run (RUN-4, design AN-7).
- **PACK-3** The analyzer knows the folders in which the design spec reports the three kinds of
  finding that depend on entry points (design AN-4a). With a hand-off file they are the folders it
  lists under `detection`. Without one they are the root and the folders its workspace names: the
  root `package.json`'s `workspaces`, as a list or as an object with `packages`, and the
  `packages` list of a `pnpm-workspace.yaml` at the root, with their globs and their exclusions.
  The output marks each package that lies in one of those folders. This item reports nothing by
  it; the item that reads entry points does.
- **PACK-4** A package is known by the `name` in its `package.json`. When two packages under the
  root have one name, an import by that name resolves to the one PACK-3 marks; when that does not
  settle it, to the first by path. A notice names both.

### Parsing

- **PARSE-1** A script is parsed with `oxc_parser` (design AN-2). Its language comes from its
  extension: TypeScript for `.ts`, `.mts` and `.cts`; TypeScript with JSX for `.tsx`; JavaScript
  with JSX allowed for `.js`, `.jsx`, `.mjs` and `.cjs`. A type declaration file is parsed as
  TypeScript.
- **PARSE-2** A file does not parse when the parser reports a syntax error in it, or when it is
  not valid UTF-8. A tracked file that does not parse stops the run, with the file and the line
  (RUN-4). An untracked file that does not parse is skipped with a notice, so a scratch file
  cannot stop a run; its imports are then not read (design AN-7).
- **PARSE-3** In a Vue component the script blocks are read: `<script>` and `<script setup>`, each
  in the language its `lang` attribute names, JavaScript when it names none. A script block's
  `src` attribute is an import. A block that does not parse is treated as PARSE-2 treats its file
  (design AN-3, FIND-11).
- **PARSE-4** In an MDX document the `import` and `export … from` statements are read, and nothing
  else (design AN-3). A statement inside a code block is text, not an import.
- **PARSE-5** In an HTML page every `<script>` element's `src` is an import, and nothing else is
  read: not the content of a script element, and no other element (design ENT-2).
- **PARSE-6** A data file (KEY-4) is never parsed.

### What an import is

- **IMP-1** These are imports, each recorded with its specifier, its form and its line:
  - an `import` statement, marked when the whole statement is `import type`;
  - an `export … from` statement, marked the same way for `export type`, and marked when it is
    `export *`;
  - `import x = require('…')`;
  - a call of `import(…)` whose one argument is a string literal, or a template literal with no
    substitution;
  - a call of `require(…)`, where `require` is the bare name, with the same kind of argument;
  - a Vue script block's `src`, and an HTML page's script `src` (PARSE-3, PARSE-5).
- **IMP-2** A call of `import(…)` or `require(…)` whose argument is anything else cannot be
  followed. The analyzer counts them for each file (design ENT-6).
- **IMP-3** Nothing else is an import: not a string passed to any other function, such as a
  worker's file or a test helper that replaces a module; not a `/// <reference>` comment; not a
  type written in a JSDoc comment. What a tool loads by its own convention is a declaration's to
  state (design ENT-5).
- **IMP-4** A specifier may carry a query, such as `?raw`, or a fragment. The import uses the file
  the specifier names without them, and is marked as carrying one, since such a file is read as
  text (design AN-3).
- **IMP-5** Which names an import brings in is not recorded in this item.

### Resolving an import

- **RES-1** Every import gets exactly one outcome:
  - **a file**: a file in the list, by its path;
  - **outside**: a package that is not in the project, or a module built into Node;
  - **ignored**: a path git ignores, such as a generated file. It adds nothing to the graph and is
    never a fault, whether or not the file is on disk (design AN-5);
  - **unresolved**: nothing.
- **RES-2** A specifier is tried against these in order, and the first that applies decides:
  1. a relative specifier (`./`, `../`) names a path from the importing file's folder (RES-3);
  2. a pattern in the `paths` of the tsconfig that governs the importing file (CFG-2, RES-4);
  3. a specifier that begins with `#` is looked up in the `imports` of the importing file's
     package (RES-5);
  4. where the governing tsconfig states a `baseUrl`, a path from that folder, when it leads to a
     file;
  5. the name of a package under the root, the importing file's own package included, alone or
     followed by a subpath (RES-6);
  6. anything else is outside. So is a module built into Node, with or without the `node:` prefix
     (design FIND-6).
- **RES-3** A path leads to a file when one of these is in the file list, tried in this order: the
  path itself; the path with a script extension written as its output form replaced by the source
  form (`.js` by `.ts`, `.tsx` or `.d.ts`; `.jsx` by `.tsx`; `.mjs` by `.mts` or `.d.mts`; `.cjs`
  by `.cts` or `.d.cts`); the path with each script extension added, TypeScript before JavaScript,
  then `.d.ts`; the path as a folder with an `index` file found the same way. A folder that is a
  package resolves through its `package.json` as RES-6 does. A path that leads to a file of the
  kind "other", or to a data file, resolves to that file, and nothing is followed from it (design
  AN-3, FIND-10).
- **RES-4** When a relative specifier, a `#` specifier or a subpath of a package under the root
  leads to no file, the import is ignored if git ignores the path it names, and unresolved if not
  (design FIND-6). When a `paths` pattern matches and none of its targets leads to a file, the
  import is unresolved or ignored in the same way, with two exceptions that send the specifier on
  down RES-2's order: the pattern is the bare `*`, which matches everything; or the specifier
  names a package under the root or a package that a `package.json` at or above the importing
  file lists as a dependency of any kind. Without the exceptions, a project that aliases some
  names of a scope and installs others would have every installed one reported.
- **RES-5** A `#` specifier is matched against the keys of the `imports` map in the importing
  file's package, a key with `*` included. Where the matched value is a set of conditions, every
  condition's target is tried in the order written, and the first that leads to a file, directly
  or through MAP-1, decides. No condition is preferred by name: each names build output of the
  same source. A `#` specifier that matches no key is unresolved.
- **RES-6** An import of a package under the root by its name resolves through that package's
  `package.json`. With an `exports` field, the subpath, or `.` for the bare name, is matched
  against its keys, patterns included, where a `*` may be written more than once in a target;
  conditions are tried as in RES-5. Without one, the bare name resolves through `module`, then
  `main`, then an `index` file in the package's folder, and a subpath is a path inside the
  package's folder (RES-3). Each target passes through MAP-1. A subpath that `exports` does not
  offer is unresolved.
- **RES-7** A script `src` in an HTML page is tried from the page's folder, then from the folder
  of the page's package, then from the root; one that begins with `/` skips the first. One that
  names another host, or leads to no file, is outside: a page may load a script from a server
  (design ENT-2).
- **RES-8** An import that is outside records the package's name, and whether a `package.json` at
  or above the importing file lists that package as a dependency of any kind. An import that is
  outside and not listed is the one way a wrong resolution can hide, so the output lets them be
  counted (LIVE-2).
- **RES-9** `oxc_resolver` is used where it gives these rules (design AN-2). The plan begins with
  a trial that shows which rules it covers under LIST-4, and which the analyzer adds around it. If
  it cannot be kept from asking the file system, the design spec's AN-2 is amended before code is
  written.
- **RES-10** Matching a path against the file list is exact on letter case on every platform. An
  import whose case differs from the file's name is unresolved everywhere, as it is on a
  case-sensitive file system.

### tsconfig files

- **CFG-1** Every tsconfig in the file list that is not data is read, as JSON that may hold
  comments and trailing commas. This item uses `extends`, `files`, `include`, `exclude`,
  `references`, and from `compilerOptions`: `paths`, `baseUrl`, `outDir` and `rootDir` (design
  ENT-2).
- **CFG-2** The tsconfig that governs a file is found from the file's folder upward: in the
  nearest folder that holds a `tsconfig.json` or a `jsconfig.json`, that config governs when its
  `files`, `include` and `exclude` cover the file, which they do by default for everything under
  its folder. When it does not cover the file and has `references`, the first referenced config
  that covers the file governs. When none does, the search goes on upward. A file with no
  governing tsconfig has no `paths` and no `baseUrl`.
- **CFG-3** `extends` is followed. A relative target, and a target inside a package under the
  root, is read, and its `paths` and `baseUrl` mean what they mean in the file that states them.
  A target in a package outside the project is not read: the config is used without it, with a
  notice, so that the result is the same with no `node_modules` (design AN-5). A relative target
  that git ignores is treated the same way.
- **CFG-4** A tsconfig that governs at least one file the analyzer reads, or that such a config
  extends from inside the project, and that cannot be read, stops the run (RUN-4, design AN-7).
  One that governs nothing is skipped with a notice.

### From build output back to source

- **MAP-1** A target in `exports`, `main`, `module` or `imports` often names build output, which
  a worktree does not have. When a target is not in the file list, the analyzer looks for the
  source it is built from, in this order (design ENT-3):
  1. for each tsconfig in the package's folder that states an `outDir` the target lies in, the
     same path under that config's `rootDir`; where it states no `rootDir`, under `src` in the
     config's folder, then under the config's folder itself;
  2. the same path with its first folder replaced by `src`.

  In each place the extension is replaced by its source forms as in RES-3, and `.d.ts` by `.ts`
  and `.tsx`.
- **MAP-2** A target that is in the file list is that file: a committed JavaScript file is its own
  source. A target git ignores is never used, on disk or not (LIST-4).
- **MAP-3** A target with a `*` is mapped as a pattern, by rewriting its folder and extension,
  never by listing build output (design ENT-3).
- **MAP-4** A target that maps to nothing leads to no file, and RES-4 decides between ignored and
  unresolved. That is normal for a file a generator writes.

### The hand-off file

- **HO-1** The analyzer reads hand-off format 1. Its schema is a JSON Schema in the yakshaver
  repository, `packages/core/src/stacks/hand-off/hand-off-1.schema.json`, on a branch that is not
  merged yet. This repository holds a copy among its tests, with the yakshaver commit it was
  copied from written beside it. Every hand-off file among the fixtures is checked against the
  copy by a test, except those made wrong on purpose (design HAND-6a).
- **HO-2** From the file the analyzer takes: the format version; `root`, an absolute path, which
  is the root (LIST-1); under `detection`, the folders and whether yak's limit cut the list
  short, which gives a notice; and `stacks`, in order, each with its name, the interface version
  its declarations were written for, the pack's declarations and the project's, kept apart. It
  reads `base` and `siblings` and checks their shape; this item uses neither. It ignores a
  top-level field it does not know (design HAND-2, HAND-6).
- **HO-3** A format version other than 1 stops the run. The line names the format the file
  states, the formats the analyzer reads, and which side to update: yak, when the file's format
  is older than any the analyzer reads, and the analyzer's version pinned in the pack, when it is
  newer (design HAND-6).
- **HO-4** A stack that states an interface version other than the analyzer's stops the run. The
  line names the stack, both versions, and which side to update: the analyzer's version pinned in
  the pack, when the declarations are newer, and the stack's declarations, when they are older. A
  stack that states none must hold no declarations (design IF-6a).
- **HO-5** Without a hand-off file the analyzer reads the project from what its own standard files
  state, with no declarations, and says so in a notice (design IF-5).
- **HO-6** `check --hand-off <file>` applies HO-1 to HO-4 and KEY-1 to KEY-3 and reads nothing
  else. It exits 0 and prints nothing when the analyzer can use the whole file. Otherwise it
  exits 1 and writes one line for each problem, so that a pack's own checks show all of them at
  once (design IF-6a).

### The declaration keys

- **KEY-1** These are the keys the analyzer knows, as drafts (RUN-7). The design spec lists what
  each states (design ENT-4); this item gives them names:

  | Key | Value | States |
  |---|---|---|
  | `entries` | a list of globs | files that are entry points |
  | `data` | a list of globs | files that are not the project's code, such as test fixtures |
  | `generated` | a list of globs | committed files that a generator writes |
  | `internal` | a list of paths | package folders whose `exports` add no entry points |
  | `cyclomaticLimit` | a positive whole number | the limit for cyclomatic complexity |
  | `cognitiveLimit` | a positive whole number | the limit for cognitive complexity |
  | `duplicationMinimum` | a positive whole number | the smallest duplicated run reported |
  | `configEntries` | opaque | entries read from a tool's configuration file |
  | `scriptCommands` | opaque | programs whose file arguments in scripts are entries |

  A key that is not in the table is a declaration the analyzer cannot use.
- **KEY-2** A declaration cannot be used when its key is unknown, when its value is not of its
  key's kind, when a glob is not a glob the analyzer can read, when a glob or a path is absolute
  or steps out of its folder with `..`, or when the project states an opaque key, which only a
  pack may. The line names the stack, the key, and whether the pack or the project states it
  (design FAIL-1). The value of an opaque key is accepted whatever it holds, until the item that
  reads entry points fixes its shape.
- **KEY-3** The analyzer combines what the file keeps apart (design ENT-4). The lists of one key
  are joined across the stacks, the pack's and the project's. A pack's glob or path is relative
  to each folder PACK-3 names, so it applies in each of them. A project's is relative to the
  root. For a number the project's value wins; where the project states none, the value of the
  stack that comes later in the file wins.
- **KEY-4** This item uses two keys, because they change what is read (design FIND-10):
  - A file a `data` glob matches is data. It is never parsed, a `package.json` among data makes no
    package, and a tsconfig among data is never read, so data cannot stop a run by being broken.
    An import that leads to a data file resolves to it, and nothing is followed from it.
  - A file a `generated` glob matches is read like any other, and marked.

  The other keys are read, checked by KEY-2, and used by later items.

### The output

- **OUT-1** `graph` prints one JSON document on standard output and exits 0. It holds: the root;
  whether a hand-off file was used; the packages, each with its folder, its name and PACK-3's
  mark; the files, each with its path, its kind, whether it is tracked, its package, and whether
  it is data, generated or a type declaration file; for each file that was read, its imports,
  each with the specifier, the form, the line, the mark of IMP-4, the outcome, and with the
  outcome the target's path or the outside package's name and RES-8's mark; the count of IMP-2;
  and the notices.
- **OUT-2** The output is the builder's and the tests' view of what was read. It is not part of
  the interface version, and a later item may change it freely.

### The fixtures

- **FIX-1** The tests hold made-up projects, one small project for each pattern, each with the
  output it must give written down beside it (design PROOF-1). No code is copied from a real
  project. A test makes the project a git repository of its own in a temporary folder, with the
  tracked, untracked and ignored files the fixture states, runs the built binary on it, and
  compares the output. The fixtures run on all five targets, as every test does.
- **FIX-2** These patterns of the design spec's list have a fixture here, as far as reading goes,
  under the design spec's numbers:
  - 5: fixture folders that are data, holding their own tsconfig files, one of them broken;
  - 9: one alias that means `src` in a tsconfig's `paths` and build output in `package.json`
    `imports`;
  - 10: a workspace package imported by name, resolving through `exports` to build output;
  - 12: relative imports written with `.js` for `.ts`, with an explicit `.ts`, and to a `.d.ts`
    alone;
  - 13: a module imported with `?raw`;
  - 14: an HTML page that is not `index.html`, loading one script by a path relative to the page
    and one by a path from the server's root;
  - 15: a web app in a sub-folder of a repository whose root has no `package.json`;
  - 16: Vue components with `<script setup>` and with a plain script, and a file only an MDX
    document imports;
  - 19: a computed `import()`;
  - 20: nested worktrees ignored only through `.git/info/exclude`, and an untracked scratch file
    that does not parse;
  - 21: committed generated sources, one of them a type declaration file that many files import;
  - 23: a pnpm workspace, and a workspace whose folders a hand-off file lists;
  - 24: a tsconfig that extends a dependency's, and an import of a file git ignores.
- **FIX-3** Fixtures 10 and 24 are each run twice, with and without `node_modules` and build
  output on disk, and must give the same output (design AN-5).
- **FIX-4** Further fixtures cover what this spec adds: a tsconfig whose `references` govern a
  file (CFG-2); a `paths` pattern that misses, in each of RES-4's cases; an `exports` pattern
  whose `*` is written twice in its target; conditions whose first target has no source; two
  packages with one name; an import whose letter case differs from the file's; a symbolic link;
  and a source file that is not UTF-8.
- **FIX-5** Every line of RUN-4 and every problem of KEY-2 has a fixture that checks the exact
  line and the exit code (design PROOF-1, pattern 26, as far as this item fails).

### The check on real repositories

- **LIVE-1** When the fixtures pass, the session runs `graph` on each named repository, on the
  owner's machine: on a clean worktree with no dependencies installed and nothing built, once
  with no hand-off file and once with a hand-off file written by hand that declares the
  repository's data and generated files. A repository with a web app in a sub-folder is also run
  from that sub-folder.
- **LIVE-2** For each run the session checks that it finishes, that the file list is git's own,
  and records the time it took. It judges, by reading the code, every import that is unresolved,
  and every import that is outside and not listed as a dependency (RES-8): true, or false.
- **LIVE-3** Each false one becomes a made-up fixture of its shape first, and is then fixed
  (design PROOF-5). One that cannot be fixed without something a later item builds is written
  down with the item it waits for.
- **LIVE-4** The judged lists stay in the work item's report folder on the owner's machine and are
  committed nowhere, since they name private repositories and their files (design PROOF-3). The
  work item's record carries the counts and the times.
- **LIVE-5** The item is done when LIVE-2 leaves no import known to be false, other than those
  LIVE-3 wrote down.

### Dependencies and the repository's own text

- **DEP-1** Every crate this item adds is pinned to one exact version, and the plan lists each
  with the reason it is needed, as the skeleton spec's GATE-1 and GATE-4 ask. The released binary
  still asks no more of a system than the skeleton spec's PKG-2a allows, which CI already checks
  on every target.
- **DOC-1** `CONTRIBUTING.md` gains how a fixture is written and run, and the three commands. The
  README's status says what the analyzer reads and that it reports no findings yet.

## Alternatives considered

- **Walking the folders with a library that reads ignore files.** It needs no `git` program. It
  was rejected because it only approximates git: the global excludes, `.git/info/exclude`, nested
  repositories and worktrees each need their own handling, and the design spec asks for what git
  ignores, and allows starting `git` to read.
- **Resolving by asking the file system, as Node and bundlers do.** That is what a resolver
  library does by default. It was rejected because the answer then depends on what is on disk: a
  built checkout resolves into build output, a fresh worktree does not, and a file system that
  ignores case resolves what a case-sensitive one refuses.
- **Reading `node_modules` when it is there, for a better answer.** Rejected by the design spec's
  AN-5: yak's worktrees never have it, so a better answer on one checkout is a different answer.
- **Going on past a syntax error.** The parser can hand back a partial tree. Rejected by the
  design spec's AN-7: an import list read from half a file is a guess.
- **A separate item for Vue, MDX and HTML.** It would make this item smaller. Rejected because the
  named repositories hold all three, so the check of LIVE-1 would be judging imports the analyzer
  cannot yet read. The plan may still build them last.
- **Reporting unresolved imports as findings now.** They fall out of resolving. Rejected because
  a finding needs a rule name, a fingerprint, SARIF and a base to compare with, which are the
  next items' work; the output already shows them.
- **Leaving the hand-off file to the item that reads entry points.** Rejected because data and
  generated declarations change what is read, and LIVE-1 needs them on real repositories.
- **Fixing the shape of every declaration key now.** Rejected for the two opaque keys: nothing
  reads their values until entry points are read, and a shape drafted without its reader would be
  changed.

## Risks

- **`oxc_resolver` cannot work from a file list.** The trial of RES-9 finds out before code is
  written. If it cannot, the analyzer carries its own resolution for these rules and the design
  spec is amended.
- **The parser refuses code a project's own tools accept.** A tracked file then stops the run.
  LIVE-1 shows whether any file of the named repositories does; a file can be declared as data;
  and the parser's version is pinned, so the answer changes only in a pull request that the
  fixtures judge.
- **A wrong "outside".** An alias that only a bundler's configuration states, and no tsconfig,
  looks like a package name, so its import is outside and its target file loses an importer.
  Nothing fails. RES-8 and LIVE-2 make these countable and judged. If a named repository needs
  such an alias, it is a new declaration, by an amendment to the design spec.
- **The rule for the governing tsconfig differs from TypeScript's in an unusual layout.** `paths`
  then resolve wrongly. The fixtures cover the layouts the design spec's recon found, and LIVE-2
  judges the rest.
- **Exact case reports imports that work on the owner's machine.** They fail on any
  case-sensitive system, so the reports are true; the cost is that a team on Windows alone sees
  them.
- **git is slow, missing, or refuses.** One run starts git a small, fixed number of times, never
  once per file, and LIVE-2 records the time. A missing git, or one that refuses a folder it does
  not trust, stops the run with git's own line (RUN-4).
- **The schema changes in yakshaver before its branch merges.** The copy records the commit it
  came from, and a changed schema is copied again with the fixtures that depend on it.
- **The draft keys change.** Only hand-written files hold them until stage 2 ends.
- **The hand-rolled reading of Vue blocks and MDX statements misses a form.** An import is then
  not read and a file may look unused later. The fixtures cover the forms the design spec names,
  and LIVE-2 would show a file whose imports were missed only indirectly, so the plan's review
  reads these two readers closely.
- **A large repository.** The output is one document holding every file and import. It is the
  builder's command, and its size is recorded in LIVE-2; if it is unwieldy the plan may add a way
  to narrow it.
- **Reading what it should not.** The analyzer reads any root a hand-off file names. It only
  reads, prints paths and specifiers and never file content, and writes nothing.

## Left to the builder

- The crate layout, and which crates read JSON, JSON with comments, YAML and globs, parse the
  command line, and run work in parallel, each with its reason (DEP-1).
- How git is asked for the file list and for what it ignores, within LIST-2 to LIST-5 and a
  small, fixed number of starts.
- The field names of the output, the names of the notice kinds, and the wording of each line.
- How the script blocks of a Vue component and the statements of an MDX document are found.
- The details of CFG-2 that TypeScript defines and this spec does not repeat, such as how
  `include` globs match.
- Where the fixtures and the schema's copy live, and how a fixture states its tracked, untracked
  and ignored files.
- How the work is split into plans and pull requests, and the order in which the readers are
  built.
