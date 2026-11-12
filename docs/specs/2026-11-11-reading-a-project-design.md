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
file), IF-3 (how a run fails), IF-5 (a run with no hand-off file), IF-6 (the interface version)
with the part of IF-6a that refuses declarations written for another version, and the first draft
of ENT-4's keys.

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
- **The stages** are the design spec's order of work. Stage 1 builds the analyzer's dead-code
  findings, and this item is part of it. Stage 2 is **the comparison**: the analyzer and fallow,
  the tool yak uses today, are both run on the named repositories, and every finding only one of
  them reports is judged true or false by reading the code. Stage 3 builds yak's side: the
  hand-off file and the declarations in the pack. **The swap** is the last stage, where yak
  starts using the analyzer in fallow's place. **The recon** is the reading of the named
  repositories that the design spec's list of fixture patterns came from.
- **A finding** is one fault the analyzer reports, under a **rule**, its kind's fixed name. **An
  audit** is the run yak asks for, which reports findings; its report is in **SARIF**, a standard
  JSON format for such reports. **A base** is a commit an audit compares the working tree with,
  to tell findings that are **new** from those the base already had, which are **inherited**; a
  **fingerprint** is what identifies a finding across the two. **The MCP server** is a later
  form of the analyzer that answers an agent's questions. None of these is built in this item.
- **The builder** is whoever carries this spec out, usually a session. **The plan** is the
  builder's written list of tasks, which the owner approves before any code is written. **The
  work item's record** is what yak keeps about this work: its decisions, its plans and their
  results. Its **report folder** is a folder yak keeps for the item beside the repository's git
  data, outside every tracked file.
- **Rust** is the language the analyzer is written in, and a **crate** is a Rust library. **oxc**
  is a set of Rust crates for JavaScript and TypeScript: `oxc_parser` reads a file into a syntax
  tree, and `oxc_resolver` finds the file an import names.
- **The launcher**, **the five targets**, **release-please** and **trusted publishing** are the
  skeleton spec's: the small JavaScript program npm installs that starts the binary; the five
  pairs of operating system and processor a binary is built for; the tool that opens a release
  pull request; and npm's way of publishing without a stored token. **The version pinned in the
  pack** is the one exact analyzer version a pack tells yak to install.

## Goals, non-goals and anti-goals

**Goals:**

- Given a root, the analyzer knows the project's files as git knows them, its packages, and for
  every import in every file it reads, exactly one outcome: the file it leads to, a package
  outside the project, a path git ignores, or nothing.
- That knowledge is the same on every platform, and the same with and without installed
  dependencies and build output that git ignores on disk.
- It reads every kind of file the design spec names as an importer: scripts, Vue components, MDX
  documents and HTML pages.
- It reads the hand-off file yak will write, refuses one it cannot use with a line that says what
  to change, and applies the one declaration that changes what is read: which files are data.
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
- The value shapes of the two declarations that describe what a tool does (KEY-1), and how two
  stacks' numbers are combined. The items that first read them fix them.
- The command that checks a hand-off file without analyzing anything (design IF-6a). Its first
  caller is the pack's own checks, in the design spec's stage 3. Until then `graph` refuses a file
  it cannot use, with the same lines.
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
- A tool's convention written into the analyzer's code where a declaration could state it. The
  formats the analyzer does read itself are the project's own standard files, which the design
  spec names: `package.json`, tsconfig files, and a workspace's list of packages.
- A real project's code or a private repository's name committed here.

## Requirements

### Commands, and how a run fails

- **RUN-1** The version option stays as it is, and the binary gains two commands:
  - `interface` prints one line of JSON with two fields: `interfaceVersion`, the interface
    version (RUN-7), and `handOffFormats`, the list of hand-off formats the analyzer reads, which
    is `[1]` (design IF-1).
  - `graph` reads the project and prints what it read (OUT-1). It takes one option,
    `--hand-off <file>`, and runs without it (HO-5).
- **RUN-2** Given nothing, a command or an option it does not have, or `--hand-off` with no file
  after it, the binary writes one line on standard error naming what it was given and what it
  accepts, and exits 2, as the skeleton did. This replaces the skeleton spec's BIN-2, and the
  skeleton's tests of it change with it.
- **RUN-3** A run that cannot go on exits 1, prints nothing on standard output, and writes exactly
  one line on standard error: what failed, and what fixes it (design IF-3). yak relays the first
  line of standard error as the reason an audit could not run, so nothing else is ever written
  there. A file's name is written into a line with any line break or control character in it
  escaped, so one line stays one line. A fault in the analyzer itself, which Rust calls a panic,
  is not a stop of RUN-4: the binary then exits 3 and writes one line that says the fault is the
  analyzer's and what it was. No test can cause one, so a reviewer reads this in the code. A
  process that the operating system ends, for want of memory or stack, writes nothing, and a
  run whose output can no longer be written, because its reader has gone, ends with exit 1 and
  no line.
- **RUN-4** These stop a run, each with its own line, and nothing else does. They are checked in
  this order, and the first that fails is the one reported; where several files fail the same
  check, the first by path is:
  1. the hand-off file cannot be read or is not JSON: the file;
  2. the hand-off file states a format the analyzer does not read, which is asked before any
     other field is looked at (HO-3);
  3. the hand-off file lacks a field format 1 requires, holds a field of the wrong shape, or
     holds a field the format does not allow where it allows none: the file, and the field
     (HO-2);
  4. a stack's declarations were written for another interface version, or a stack holds
     declarations and states no interface version (HO-4);
  5. a declaration cannot be used, the stacks taken in the file's order and the pack's before
     the project's (KEY-2);
  6. git cannot be started, or is older than the oldest version the analyzer works with: both
     versions (DEP-2);
  7. the root is not inside a git work tree, or git fails: the root, and git's own first line
     where it gave one;
  8. a `package.json` that makes a package cannot be read as a JSON object: the file (PACK-2);
  9. the workspace's list at the root, `pnpm-workspace.yaml`, cannot be read: the file (PACK-3);
  10. a tsconfig that CFG-4 names cannot be read, or extends a file that is missing: the file,
      and the missing one (CFG-4);
  11. a field the analyzer reads in a `package.json`, a tsconfig or the workspace's list holds a
      value of a form that field does not have, such as `workspaces` that is a number: the file,
      and the field (PACK-6);
  12. a file git tracks does not parse or cannot be read: the file and the line, and that a file
      broken on purpose is declared as data (PARSE-2).
- **RUN-5** A run that goes on collects notices. Each has a kind, the file it is about where there
  is one, and a sentence. They are part of the output (OUT-1) and are never written on standard
  error. Each requirement below that says "with a notice" adds a kind.
- **RUN-6** The analyzer never runs a project's code, which includes its configuration files. It
  writes nothing inside the root and uses no network. The one program it starts is `git`, and only
  to read (design AN-6). This replaces the skeleton spec's BIN-4.
- **RUN-6a** How git is started is part of that rule:
  - It is given only commands that read, with git's optional locks turned off, so that a run
    writes nothing inside `.git` either, and a run that is killed leaves no lock behind.
  - The variables by which a caller's environment can point git at another repository, work
    tree or index are cleared for git, so that the root decides which repository is read. yak
    may start the analyzer from inside a git hook, where they are set.
  - It is started with the settings that would make git start another program turned off, a
    file-system monitor and a pager among them. The plan lists each setting.
  - It is started so that letter case counts in an ignore pattern on every platform, whatever
    the repository's own setting says. Without that, a pattern whose case differs from a path
    would ignore the path on Windows and macOS and not on Linux, and RUN-8 would not hold. The
    plan's first task tries what this does to the file list on a file system that ignores case,
    and a fixture holds such a pattern (FIX-4).
  - What git writes on standard error is kept from the analyzer's own standard error (RUN-3).
    When git succeeds and still warns, the first line of the warning becomes a notice.
- **RUN-7** The interface version is the whole number 1. The design spec makes it cover the
  commands and options, the declaration keys and their shapes, the rule names, the fingerprints
  and the MCP tools, and raises it when a release breaks one of them (design IF-6). Until the end
  of the design spec's stage 2 the keys of KEY-1 and the `interface` command are drafts: changing
  a draft does not raise the number. Nothing but hand-written files states an interface version
  before then. `graph`, as a command and in what it prints, is never under the interface version
  (OUT-2).
- **RUN-8** For the same project and the same hand-off file, the output is the same bytes on every
  platform. One thing a machine may still change is the user's own excludes there, which the design
  spec has the analyzer honor (LIST-2): a file one user's git ignores and another's does not is read
  on one machine only. Every path in it is relative to the root and written with forward slashes.
  Packages and files are listed by path, compared byte by byte, which is also what "first by path"
  means elsewhere in this spec; a file's imports are listed in the order they are written, and the
  files one import leads to by path; notices are listed by file, those about no file first, then by
  kind, then by their sentence.

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
  (design AN-4). The analyzer knows where each one lies, from what git lists, and never asks git
  about a path inside one; an import that leads into one is outside (RES-1). A tracked file that is
  missing from the working tree is not in the list, and one that is gone by the time it is read is
  treated as missing, with a notice, and not as a file that cannot be read. A symbolic link is not
  in it either: it is never followed and never read, one notice says how many were left out, and an
  import that names one, or passes through a linked folder, is outside (RES-1). A tracked path is a
  link when git records it as one, whatever the working tree holds, so that a system that checks
  links out as ordinary files gives the same result; an untracked path is one when the file system
  says so.
- **LIST-4** Whether a path exists is asked of the file list, never of the file system. Only the
  content of a listed file is read from disk. So a file git ignores cannot change a result by
  being on disk, which is what makes the result the same with and without `node_modules` and
  build output that git ignores (design AN-5), and a path matches only when its letters' case
  matches, on every platform (RES-10). Build output that git does not ignore is in the list like
  any file, and is part of the project. The one file asked about outside the list is a tsconfig
  above the root (CFG-3): the analyzer asks git the same question about that one path, whether
  it is a file of the work tree that git does not ignore, and then reads it.
- **LIST-5** For a path that is not in the list, the analyzer can tell whether git ignores it, with
  git's own answer, whether or not the path is on disk. RES-1 needs this to tell an ignored
  target from a missing one.
- **LIST-6** A listed file has one kind, from its name: a script (`.ts`, `.tsx`, `.mts`, `.cts`,
  `.js`, `.jsx`, `.mjs`, `.cjs`); a type declaration file (`.d.ts`, `.d.mts`, `.d.cts`); a Vue
  component (`.vue`); an MDX document (`.mdx`); an HTML page (`.html`); a package manifest
  (`package.json`); a tsconfig (`jsconfig.json`, and any file whose name begins with `tsconfig`
  and ends with `.json`, as the design spec's ENT-2 has it); or other. A file of the kind
  "other", such as CSS or JSON, is never read (design AN-3), with one exception: a
  `pnpm-workspace.yaml` at the root, when it is in the list (PACK-3).
- **LIST-7** A path that is not valid UTF-8 is left out of the list, with a notice.
- **LIST-8** Nothing above the root is part of the project, also when the root is a sub-folder of
  a work tree. Every search that goes upward from a file, for its package, its tsconfig or a
  `package.json` that lists a dependency, ends at the root. An import that leads above the root
  is outside (RES-1). The one thing read above the root is a tsconfig that a tsconfig under the
  root extends (CFG-3).
- **LIST-9** Everything after the file list works from three things only: the list; a way to
  read the content of a listed file; and LIST-4's way to ask about and read one tsconfig above
  the root. The file system `oxc_resolver` is given (RES-9) answers from the same three. The
  item that tells new findings from inherited ones reads a commit and not a working tree (design
  BASE-1), and supplies all three from git's stored files. How "git ignores it" is answered for
  a commit is that item's to say.

### Packages

- **PACK-1** Every `package.json` in the file list that is not data (KEY-4) makes a package,
  whether or not a workspace lists it (design AN-4). A file belongs to the nearest package at or
  above its folder. A file with no `package.json` above it under the root belongs to no package,
  which is normal for a repository whose root has none.
- **PACK-2** A `package.json` that makes a package and cannot be read as a JSON object stops the
  run (RUN-4, design AN-7).
- **PACK-3** A **workspace package** is a package in a folder the project's workspace names: the
  root `package.json`'s `workspaces`, as a list or as an object with `packages`, and the
  `packages` list of a `pnpm-workspace.yaml` at the root, with their globs and their exclusions;
  and, with a hand-off file, any folder it lists under `detection` that holds a `package.json`.
  The package in the root's own folder is always one.
  The analyzer always reads the workspace's own list, with or without a hand-off file, because
  resolving needs it: an import by a package's name leads to a workspace package (RES-6). A
  `package.json` in any other folder, such as an example project, makes a package for the files
  under it, and an import by its name leads to it in one case only, which RES-2's step 5 states.
  Without that rule, a stray folder whose `package.json` carries the name of an installed
  dependency would take every import of it. An entry of a workspace's list is a glob (KEY-5); an
  entry that begins with `!` takes folders out again. A `pnpm-workspace.yaml` that is in the
  file list and cannot be read stops the run (RUN-4).
- **PACK-4** The analyzer also knows the folders in which the design spec reports the three kinds of
  finding that depend on entry points: an unused file, an unused export and an unused type (design
  AN-4a). With a hand-off file they are the folders it lists under `detection`, which is the list
  yak's detection made; without one they are the root and the workspace's folders of PACK-3. The
  output marks each package whose own folder is one of them; a package deeper inside such a folder
  is not marked. This item reports nothing by the mark; the item that reads entry points does.
- **PACK-5** A package is known by the `name` in its `package.json`. When two workspace packages
  have one name, an import by that name resolves to the first by path, and a notice names both.
- **PACK-6** The analyzer reads these fields of a `package.json`: `name`, `workspaces`,
  `exports`, `imports`, `types` and its older name `typings`, `module`, `main`, and the four
  lists of dependencies. The plan lists the forms each field may take, from each tool's own
  documentation, and a fixture holds each form. From the
  workspace's list it reads `packages`, and from a tsconfig the fields of CFG-1. Each is read in
  every form npm, Node, pnpm or TypeScript gives it; a target in `exports` or `imports` that is
  a list is followed entry by entry, as a set of conditions is (RES-5). A field that holds a
  value of no such form stops the run (RUN-4): treating it as absent would be a guess, and would
  hide what the file was meant to say (design AN-7). A field the analyzer does not read is never
  looked at.

### Parsing

- **PARSE-1** A script is parsed with `oxc_parser` (design AN-2). Its language comes from its
  extension: TypeScript for `.ts`, `.mts` and `.cts`; TypeScript with JSX for `.tsx`; JavaScript
  with JSX allowed for `.js`, `.jsx`, `.mjs` and `.cjs`. A type declaration file is parsed as
  TypeScript. Every file is parsed as a module first, and one that does not parse as a module is
  tried as a script before PARSE-2 calls it broken, whatever its extension. A byte-order mark at
  the start of any file the analyzer reads, a `package.json` and a tsconfig included, is passed
  over.
- **PARSE-2** A file does not parse when the parser reports a syntax error in it, or when it is
  not valid UTF-8. A tracked file that does not parse stops the run, with the file and the line
  (RUN-4), the line where there is one. An untracked file that does not parse is skipped with a
  notice, so a scratch file cannot stop a run; its imports are then not read (design AN-7). A
  listed file whose content cannot be read from disk is treated the same way, by whether it is
  tracked. This holds for scripts and for the script blocks of Vue components. An MDX document
  and an HTML page can fail only by not being readable from disk or not being valid UTF-8:
  nothing written in one is a syntax error (PARSE-4, PARSE-5). A `package.json` and a tsconfig
  are different on purpose: one that cannot be read stops the run
  whether or not git tracks it (PACK-2, CFG-4), because every file under it is read by it.
- **PARSE-3** In a Vue component the script blocks are read: `<script>` and `<script setup>`, each
  in the language its `lang` attribute names (`ts`, `tsx`, `js` or `jsx`), JavaScript when it
  names none. A block in any other language is not read, with a notice. A script block's
  `src` attribute is an import. A block that does not parse is treated as PARSE-2 treats its file
  (design AN-3, FIND-11).
- **PARSE-4** In an MDX document the `import` and `export … from` statements are read, and nothing
  else (design AN-3). A statement inside a code block is text, not an import. A line that looks
  like the start of such a statement and does not parse as one is text too, with a notice, and
  never stops a run: prose may begin with the word "import", and this reader is the analyzer's
  own.
- **PARSE-5** In an HTML page every `<script>` element's `src` is an import, and nothing else is
  read: not the content of a script element, and no other element (design ENT-2). The page is
  read leniently, as a browser reads it, and nothing in its markup stops a run.
- **PARSE-6** A data file (KEY-4) is never parsed.

### What an import is

- **IMP-1** These are imports, each recorded with its specifier, its form and its line:
  - an `import` statement, marked when the whole statement is `import type`;
  - a TypeScript import type, `import('…')` written where a type is expected, marked as a type
    import;
  - an `export … from` statement, marked the same way for `export type`, and marked when it is
    `export *`;
  - `import x = require('…')`;
  - a call of `import(…)` whose first argument is a string literal, or a template literal with
    no substitution; a second argument, which holds options, changes nothing;
  - a call of `require(…)`, where `require` is the bare name, with the same kind of argument;
  - a Vue script block's `src`, and an HTML page's script `src` (PARSE-3, PARSE-5).
- **IMP-2** A call of `import(…)` or `require(…)` whose argument is anything else cannot be
  followed. The analyzer counts them for each file (design ENT-6).
- **IMP-3** Nothing else is an import: not a string passed to any other function, such as a
  worker's file or a test helper that replaces a module; not a `/// <reference>` comment; not a
  type written in a JSDoc comment. What a tool loads by its own convention is a declaration's to
  state (design ENT-5).
- **IMP-4** A specifier may carry a query, such as `?raw`, or a fragment, which begins at a `#`
  that is not the specifier's first character. The import uses the file the specifier names
  without them, and is marked as carrying one, since such a file is read as text (design AN-3).
  A specifier that begins with `#` is a name in a package's `imports` (RES-2), not a fragment.
- **IMP-5** Which names an import brings in is not recorded in this item.

### Resolving an import

- **RES-1** Every import gets exactly one outcome. Where two could apply, such as a path inside a
  nested worktree that git also ignores, the one earlier in this list holds:
  - **files**: one file in the list, by its path, or several, where a package offers different
    files for different conditions (RES-5), or a type declaration file lies beside the script it
    describes (RES-3);
  - **outside**: somewhere the analyzer does not look. There are six such places: a package that
    is not in the project; a module built into Node; a path above the root (LIST-8); a path
    inside a nested repository, a submodule or another worktree (LIST-3); a symbolic link, or a
    path through one (LIST-3); and something loaded from elsewhere, such as a script a page
    takes from a server (RES-7);
  - **ignored**: a path git ignores, such as a generated file. It adds nothing to the graph and is
    never a fault, whether or not the file is on disk (design AN-5);
  - **unresolved**: nothing.
- **RES-2** A specifier is tried against these in order, and the first that applies decides:
  1. a relative specifier, which is `.` or `..` or begins with `./` or `../`, names a path from
     the importing file's folder (RES-3);
  2. a pattern in the `paths` of the tsconfig that governs the importing file (CFG-2, RES-4);
  3. a specifier that begins with `#` is looked up in the `imports` of the importing file's
     package (RES-5);
  4. where the governing tsconfig states a `baseUrl`, a path from that folder, when it leads to a
     file;
  5. the name of a workspace package (PACK-3), alone or followed by a subpath (RES-6). The name
     is the specifier's first segment, or its first two when it begins with `@`. Any package,
     workspace package or not, may import itself by its own name. Node allows that only for a
     package with `exports`; TypeScript and bundlers are often set up to allow it without, so
     the analyzer allows it for any. Otherwise a package that is not a workspace package is
     imported by name only where a `package.json` at or above the importing file lists that
     name as a dependency whose version begins with `workspace:`, `file:`, `link:` or
     `portal:`. After `file:`, `link:` or `portal:` comes a path from the folder of the
     `package.json` that lists it, and the package in that folder is the one meant; after
     `workspace:`, it is the package under the root with that name, the first by path;
  6. anything else is outside. So is a module built into Node, with or without the `node:` prefix
     (design FIND-6): any specifier with the prefix, and the names Node itself lists as built in,
     which the plan takes from one named Node version. A specifier that is an absolute path or a
     URL is outside too, as loaded from elsewhere.
- **RES-3** A path leads to a file as TypeScript finds one, with the file list in place of the
  file system. That is, when one of these is in the list, tried in this order: for a path that
  ends in a script extension's output form, the same path in its source forms (`.js` as `.ts`,
  `.tsx` or `.d.ts`; `.jsx` as `.tsx`; `.mjs` as `.mts` or `.d.mts`; `.cjs` as `.cts` or
  `.d.cts`); the path itself; the path with each script extension added, TypeScript before
  JavaScript, then `.d.ts`; the path as a folder with an `index` file found the same way. A
  script ending in `.js`, `.mjs` or `.cjs` and a type declaration file of the same name in the
  same folder are a pair, and an import that finds either one leads to both, however the
  specifier is written: TypeScript reads the declaration and the program runs the script, and
  an import that led to one alone would leave the other without an importer. A
  folder that holds a `package.json` is tried through its `types`, `module` and `main` first,
  and leads to every file they lead to, as in RES-6. A path that leads to a file of the kind
  "other", or to a data file, resolves to that file, and nothing is followed from it (design
  AN-3, FIND-10).
- **RES-4** When a relative specifier, a `#` specifier or a subpath of a workspace package leads
  to no file, the import is ignored or unresolved (design FIND-6). It is ignored when git
  ignores the path as it is written, or one of the source forms RES-3 and MAP-1 tried for it:
  the paths that end in `.ts`, `.tsx`, `.mts`, `.cts` or a type declaration's ending. The path as
  it is written counts whatever its ending, since the design spec's AN-5 never reports an import
  whose target git ignores. An output form that RES-3 added and the specifier did not write,
  such as the path with `.js` put on, does not count. So in a repository that compiles beside
  its sources and ignores every `.js` file, a mistyped `./fooo` is unresolved, and a mistyped
  `./fooo.js` is ignored: the analyzer cannot tell that one from an import of a generated file,
  the project's own compiler reports it, and a false finding here would be one no declaration
  can settle. Otherwise the import is unresolved. When a `paths` pattern matches and none of its
  targets leads to a file, the import is unresolved or ignored in the same way, with two
  exceptions that send the specifier on down RES-2's order: the pattern is the bare `*`, which
  matches everything; or the specifier names a workspace package or a package that a
  `package.json` at or above the importing file lists as a dependency of any kind. Without the
  exceptions, a project that aliases some names of a scope and installs others would have every
  installed one reported.
- **RES-5** A `#` specifier is matched against the keys of the `imports` map in the importing
  file's package, a key with `*` included, as Node matches them. Where the matched value is a set
  of conditions, every condition's target is followed, each through MAP-1, and the import leads
  to every file found. The analyzer cannot know which condition a program will run under, so no
  condition is preferred: often they all name build output of one source, and the import then
  leads to that one file; where they name different sources, such as one file for a browser and
  one for Node, each of them is used by the import. A `#` specifier that matches no key is
  unresolved, and so is one whose conditions lead to no file at all, unless RES-4 makes it
  ignored.
- **RES-6** An import of a package by its name (RES-2, step 5) resolves through that package's
  `package.json`. With an `exports` field, the subpath, or `.` for the bare name, is matched against
  its keys, patterns included, where a `*` may be written more than once in a target; conditions are
  followed as in RES-5. An `exports` that is one text value, or a set of conditions with no key that
  begins with `.`, is the target for the bare name. Without an `exports` field, the bare name leads
  to every file that `types`, `module` and `main` lead to, or, where none of them is stated, to an
  `index` file in the package's folder; and a subpath is a path inside the package's folder (RES-3).
  Each target passes through MAP-1. A subpath that `exports` does not offer is unresolved.
- **RES-7** A script `src` in an HTML page is tried from the page's folder, then from the folder
  of the page's package, then from the root; one that begins with `/` skips the first. In each
  place the file is looked for as RES-3 says. One that names another host, or leads to no file,
  is outside, as loaded from elsewhere: a page may load a script from a server (design ENT-2).
- **RES-8** An import that is outside records which of RES-1's six places it is. For a package it
  records the package's name, and whether a `package.json` at or above the importing file lists
  that package as a dependency of any kind. An import of a package that is outside and not
  listed is where a wrong resolution hides most easily, so the output lets them be counted
  (LIVE-2).
- **RES-9** What is built is `oxc_resolver` with a file system of the analyzer's own, and the
  analyzer's own code around it (design AN-2). The library takes its file system as a part the
  caller supplies (its `FileSystem` trait, read in its documentation at version 11.24.3 on
  2026-11-11), so the analyzer gives it one that answers from the file list, which is how LIST-4
  and RES-10 hold inside the library. The split:
  - the library finds a file for a path (RES-3), and reads tsconfig files with their `extends`,
    `references`, `paths` and `baseUrl` (CFG-1 to CFG-3);
  - the analyzer's own code does what differs from Node's and TypeScript's rules or is not
    resolution at all: the order and the outcomes (RES-1, RES-2, RES-4), workspace packages by
    name with no `node_modules` (RES-6), following every condition (RES-5), build output back to
    source (MAP-1), HTML pages (RES-7), and the mark of RES-8.

  The plan's first task tries the library's half against this spec's fixtures on the version it
  pins. A rule the library does not give as written here moves to the analyzer's own code, and
  the plan says so; the rule and its fixture do not change. If the trial leaves the library
  nothing but finding a file for a path, or shows it cannot be used at all, it is not worth a
  dependency that changes every week: the design spec's AN-2 is then amended, before any other
  code is written, and the analyzer resolves in its own code.
- **RES-10** Matching a path against the file list is exact on letter case on every platform. An
  import whose case differs from the file's name is unresolved everywhere, as it is on a
  case-sensitive file system.

### tsconfig files

- **CFG-1** Every tsconfig in the file list that is not data is read, as JSON that may hold
  comments and trailing commas. This item uses `extends`, `files`, `include`, `exclude`,
  `references`, and from `compilerOptions`: `paths`, `baseUrl`, `outDir` and `rootDir`. The
  design spec's ENT-2 names six of these; `include`, `exclude` and `baseUrl` are this spec's
  additions, needed to tell which config governs a file and to resolve as TypeScript does.
- **CFG-2** The tsconfig that governs a file is found from the file's folder upward: in the
  nearest folder that holds a `tsconfig.json` or a `jsconfig.json`, the `tsconfig.json` where it
  holds both, that config governs when its
  `files`, `include` and `exclude` cover the file, which they do by default for everything under
  its folder. When it does not cover the file and has `references`, the first referenced config
  that covers the file governs. When none does, the search goes on upward. A file with no
  governing tsconfig has no `paths` and no `baseUrl`. Whether a config covers a file is asked as
  if the file were TypeScript: a `.js`, `.vue`, `.mdx` or `.html` file is covered when a `.ts`
  file of the same name in the same folder would be, and `allowJs` is not read. Otherwise a Vue
  component or an MDX document would have no `paths`, and its aliased imports would be outside.
- **CFG-3** `extends` is followed. A relative target, and a target inside a workspace package, is
  read, and its `paths` and `baseUrl` mean what they mean in the file that states them. A
  relative target above the root is read too, when it is a file of the same work tree that git
  does not ignore: a web app in a sub-folder often extends a config at the repository's root. A
  `paths` target that leads above the root makes the import outside (LIST-8). A target in a
  package outside the project is not read: the config is used without it, with a notice, so that
  the result is the same with no `node_modules` (design AN-5). A relative target that git ignores
  is treated the same way.
- **CFG-4** A tsconfig that cannot be read stops the run when a file the analyzer reads depends on
  it (RUN-4, design AN-7). Whether one does is told without reading it, by where it lies and who
  names it: it is a `tsconfig.json` or a `jsconfig.json` in any folder at or above a file the
  analyzer reads, up to the root, since CFG-2's search may pass the nearest one and go on
  upward; or a tsconfig that such a config, or one it names in turn, names in
  `extends` or `references`, a config above the root included (CFG-3). Any other tsconfig that
  cannot be read is skipped with a notice. An `extends` target that should be read and is not
  there stops the run too, naming both files. A `references` entry that names no tsconfig in
  the list is passed over with a notice.

### From build output back to source

- **MAP-1** A target in `exports`, `types`, `module`, `main` or `imports` often names build
  output, which a worktree does not have. When a target is not in the file list, the analyzer
  looks for the source it is built from, in this order. The same is done for any other path an
  import names that leads to no file: a subpath of a workspace package that has no `exports`,
  and a relative specifier that reaches into a build folder. In every case this comes before
  RES-4 decides between ignored and unresolved. The order is the design spec's ENT-3; what is
  tried where a config states no `rootDir`, and applying it to a path that no `package.json`
  states, are this spec's:
  1. for each tsconfig that lies directly in the folder of the package the target or path lies
     in, `tsconfig.json` first and the others by path, and that states an
     `outDir` the target lies in: the same path under that config's `rootDir`; where it states
     no `rootDir`, under `src` in the config's folder, then under the config's folder itself;
  2. the same path with its first folder replaced by `src`.

  In each place the target's ending, whether a script's output form (`.js`, `.mjs`, `.cjs`) or a
  type declaration's, is replaced by each source extension in turn: `.ts`, `.tsx`, `.mts`,
  `.cts`, then `.js`, `.jsx`, `.mjs`, `.cjs`. This is wider than RES-3's pairs on purpose: a
  bundler writes `.mjs` and `.cjs` files from one `.ts` source.
- **MAP-2** A target that is in the file list is that file: a committed JavaScript file is its own
  source. A target git ignores is never used, on disk or not (LIST-4).
- **MAP-3** A target with a `*` is mapped as a pattern, by rewriting its folder and extension,
  never by listing build output (design ENT-3). For an import, RES-5 and RES-6 have filled the
  `*` in from the specifier before MAP-1 is applied, so nothing in this item maps a pattern; the
  rule is stated here because the item that reads entry points maps whole patterns the same way.
  An `outDir` and a `rootDir` in MAP-1 are a tsconfig's own or the ones it takes through
  `extends` (CFG-3).
- **MAP-4** A target that maps to nothing leads to no file, and RES-4 decides between ignored and
  unresolved. That is normal for a file a generator writes. It is also where a mapping that
  should have found a source and did not ends up, as ignored, since build folders are ignored; so
  an ignored import records the target it came from, and LIVE-2 judges every one.

### The hand-off file

- **HO-1** The analyzer reads hand-off format 1. Its schema is a JSON Schema in the yakshaver
  repository, `packages/core/src/stacks/hand-off/hand-off-1.schema.json`, on the branch of that
  repository's pull request #213, which is not merged yet. This item adds a copy of it among this
  repository's tests; HO-2 says what it holds, for a reader who cannot open the original. Beside the
  copy are written the path, the pull request, the commit it was copied from, and a SHA-256 digest
  of the file's content, so the copy can be checked against the original even after that branch is
  rewritten. Every hand-off file among the fixtures is checked against the copy by a test, except
  those made wrong on purpose (design HAND-6a). The check needs a JSON Schema validator, which is a
  dependency of the tests alone and never part of the binary (DEP-1).
- **HO-2** A hand-off file of format 1 is one JSON object with these fields (design HAND-2):

  | Field | Needed | Value | The analyzer |
  |---|---|---|---|
  | `formatVersion` | yes | the number 1 | refuses any other (HO-3) |
  | `root` | yes | an absolute path, with symbolic links resolved, as the operating system spells it | reads this folder (LIST-1) |
  | `detection.folders` | yes | a list of at least one path, each relative to `root` with forward slashes; the first is `.` | refuses a list whose first is not `.`, or a path that is absolute or steps out with `..` (RUN-4); uses them in PACK-3 and PACK-4 |
  | `detection.truncated` | yes | true or false: whether yak stopped at its limit of 64 folders, so the list is not the whole workspace | gives a notice when true |
  | `stacks` | yes | a list, in the order yak applies the stacks | reads each (HO-4, KEY-1) |
  | `stacks[].name` | yes | the stack's name | names it in a line |
  | `stacks[].interfaceVersion` | no | a positive whole number: the interface version the stack's declarations were written for | checks it (HO-4) |
  | `stacks[].pack` | yes | an object: each of the pack's declaration keys with its value, any JSON | reads it (KEY-1) |
  | `stacks[].project` | yes | an object: each of the project's declaration keys with a list of text values or a positive whole number | reads it (KEY-1) |
  | `siblings` | yes | an absolute path: the folder the hand-off file lies in, which holds the files of the repository's other checkouts | checks its shape; a later item uses it |
  | `base` | no | a commit's full name, 40 or 64 hexadecimal digits: the commit a later item compares the checkout with | checks its shape; a later item uses it |

  The analyzer ignores a top-level field it does not know (design HAND-6). Inside `detection`
  and inside a stack the format allows no other field, and the analyzer refuses one there
  (RUN-4).
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

### The declaration keys

- **KEY-1** These are the keys the analyzer knows, as drafts (RUN-7). The design spec lists what
  each states (design ENT-4); this item gives them names. It names all nine, though it uses one,
  because the analyzer refuses a key it does not know: a hand-off file written by hand for the
  comparison holds keys that later items read, and must not be refused meanwhile:

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
- **KEY-3** The analyzer combines the lists the file keeps apart (design ENT-4). The lists of one
  key are joined across the stacks, the pack's and the project's. A pack's glob or path is
  relative to each folder PACK-4 names, so it applies in each of them. A project's is relative to
  the root. A number is only checked by KEY-2 in this item: when two stacks, or a pack and the
  project, state the same number key, each value is checked and none is used. The design spec's
  ENT-4 already says which wins, the project's and otherwise the later stack's, and the item
  that first reads a limit builds that.
- **KEY-4** This item uses `data`, because it changes what is read (design FIND-10). A file a
  `data` glob matches is data. It is never parsed, a `package.json` among data makes no package,
  and a tsconfig among data is never read, so data cannot stop a run by being broken. An import
  that leads to a data file resolves to it, and nothing is followed from it. A file a `generated`
  glob matches is read like any other; the output marks it, for the item that reports findings.
  The other keys are read, checked by KEY-2, and used by later items.
- **KEY-5** A glob is a pattern for the paths of files, written with forward slashes and matched
  with exact letter case against paths relative to the glob's folder (KEY-3). In it, `*` stands
  for any run of characters inside one path segment, `**` for any number of whole segments, `?`
  for one character, `[abc]` for one character of a set, which may hold a range such as `a-z`
  and may begin with `!` to mean any character not in it, and `{a,b}` for one of several
  alternatives, which may hold further braces. A backslash before a character makes it stand for
  itself, as every other character does. A glob the analyzer cannot read is one with a bracket
  or a brace left open. A glob names files: a folder's name alone matches nothing, and
  `folder/**` matches every file under it. A declaration's glob has no way to exclude; an entry
  of a workspace's list is a glob for folders, and may begin with `!` (PACK-3). A `data` or
  `generated` glob that matches no file gives a notice, since it is likely a mistake.

### The output

- **OUT-1** `graph` prints one JSON document on standard output and exits 0. It holds no absolute
  path, the root's included (RUN-8). It holds: whether a hand-off file was used; the packages,
  each with its folder, its name, whether it is a workspace package (PACK-3) and PACK-4's mark;
  the files, each with its path, its kind, whether it is tracked, its package, and whether it is
  data, generated or a type declaration file; for each file that was read, its imports, each
  with the specifier, the form, the line, the mark of IMP-4, the outcome, and with the outcome
  the paths of the files it leads to, or which of the six places it is outside in (RES-8), with
  a package's name and mark, or the ignored path and the
  target it came from (MAP-4), and which rule decided, a step of RES-2 or RES-7; the count of
  IMP-2; and the notices.
- **OUT-2** The output is the builder's and the tests' view of what was read. It is not part of
  the interface version, and a later item may change its form. The facts the fixtures hold, each
  file and each import's outcome, are what this spec fixes: an item that changes the form
  rewrites the fixtures' written-down output in the same change and changes none of those facts.

### The fixtures

- **FIX-1** The tests hold made-up projects, one small project for each pattern, each with the
  output it must give written down beside it (design PROOF-1). No code is copied from a real
  project. A test makes the project a git repository of its own in a temporary folder, with the
  tracked, untracked and ignored files the fixture states, runs the built binary on it, and
  compares the output. The fixtures run on all five targets, as every test does. A test runs git
  and the binary with the machine's and the user's own git settings shut out, global excludes
  included, so that a fixture's result does not depend on the machine. A fixture that needs
  something an operating system does not let a test make, such as an untracked symbolic link on
  Windows, runs on the targets that allow it, and says which.
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
  - 15: a web app in a sub-folder of a repository whose root has no `package.json`, run from the
    repository's root and with the sub-folder as the root, where one import and one tsconfig's
    `extends` lead above the sub-folder (LIST-8, CFG-3);
  - 16: Vue components with `<script setup>` and with a plain script, and a file only an MDX
    document imports;
  - 19: a computed `import()`;
  - 20: nested worktrees ignored only through `.git/info/exclude`, and an untracked scratch file
    that does not parse;
  - 21: committed generated sources, one of them a type declaration file that many files import;
  - 23: a pnpm workspace, and a workspace whose folders a hand-off file lists;
  - 24: a tsconfig that extends a dependency's, and an import of a file git ignores.
- **FIX-3** Fixtures 10 and 24 are each run twice, with and without `node_modules` and ignored
  build output on disk, and must give the same output (design AN-5). The test writes those
  folders into the temporary project itself: this repository ignores `node_modules`, so they
  cannot be committed as a fixture's files.
- **FIX-4** Further fixtures cover what this spec adds: a tsconfig whose `references` govern a
  file (CFG-2); a `paths` pattern that misses, in each of RES-4's cases; an `exports` pattern
  whose `*` is written twice in its target; conditions that name one source, conditions that
  name two different sources, and conditions whose first target has no source; a folder outside
  the workspace whose `package.json` carries the name of an installed dependency, and a package
  outside the workspace listed with `file:`; two workspace packages with one name; an import
  whose letter case differs from the file's; an ignore pattern whose letter case differs from a
  path's (RUN-6a); a symbolic link, an import that names it and one that passes through a
  linked folder; an import that leads into a submodule and one into a nested repository
  (LIST-3); a source file that is not UTF-8; a package target in an ignored build folder that
  maps to no source (MAP-4); an import with no extension of a file git ignores by its ending,
  and a mistyped import in a project that ignores every `.js` file, written with no extension
  and written with `.js` (RES-4); an MDX document
  whose prose begins a line with "import" (PARSE-4); and a `package.json` whose `exports` holds
  a list (PACK-6).
- **FIX-5** Every stop of RUN-4 and every case of KEY-2 has a fixture that checks the line and
  the exit code, and one fixture breaks two things at once to check RUN-4's order (design
  PROOF-1, pattern 26, as far as this item fails). The comparison leaves out the parts of a line
  that vary by machine: the temporary folder, and git's or the operating system's own words. A
  stop that a target does not let a test cause, such as a file that cannot be read where the
  tests run as an administrator, is checked on the targets that allow it, and the test says
  which.
- **FIX-6** Three of the rules about what the analyzer may touch are tested, on a fixture of
  their own. After a run, nothing under the root has changed, the `.git` folder included, by a
  comparison of every file's content before and after. A run started with the environment's
  variables pointing git at another repository, work tree and index gives the output it gives
  without them. And a run on a repository whose own settings name a program for git to start
  does not start it, which the test sees because the program would leave a file behind. That
  the analyzer uses no network and runs no project code cannot be tested; a reviewer reads the
  code for it, as the skeleton spec's GATE-4 says. Also among the fixtures: a script with a type
  declaration file of the same name beside it, imported with and without an extension (RES-3).

### The check on real repositories

- **LIVE-1** When the fixtures pass, the session runs `graph` on each named repository, on the
  owner's machine: on a clean worktree with no dependencies installed and nothing built, once
  with no hand-off file and once with a hand-off file written by hand that declares the
  repository's data and generated files. A repository with a web app in a sub-folder is also run
  from that sub-folder. The session reads the repositories' names from the record of the design
  spec's work item, with yak, in the yakshaver checkout on that machine. A hand-written hand-off
  file is kept in this work item's report folder, in a folder of its own for each repository;
  its `detection.folders` are the root and the workspace's folders as PACK-3 finds them, its
  `siblings` is the folder it is kept in, and it has one stack, holding the repository's `data`
  and `generated` globs as the project's. The session writes those globs from its own reading of
  the repository: the folders that hold test fixtures and other files that are not the
  project's code, and the committed files a generator writes. Each glob is written in the report
  with the reason for it and the number of files it matches, because a `data` glob takes files
  out of the reading and a wrong one would hide a fault. The owner reads that list when
  validating the plan that carries this check, which is before the item can close, and a glob
  the owner rejects is removed and the run judged again. The same globs are the starting point
  for the files the comparison writes in stage 2, where they are judged a second time. The run
  with no hand-off file has no declarations, so
  on a repository that holds a file broken on purpose it stops, as PARSE-2 says it must; the
  line it stops with is then that run's result, and is recorded. Each worktree is made with git
  in a temporary folder outside every repository and removed when its runs are judged. What a
  run prints is saved in the report folder, apart from the folders that hold the hand-off
  files, which hold nothing else.
- **LIVE-2** For each run with a hand-off file the session checks that it finishes, and that the
  file list is git's own: the paths in the output are the paths git itself prints, asked separately
  with the command the plan names, for the tracked files and the untracked files it does not ignore
  under the root, less the links.
  It records the time the run took. The time is set beside what the design spec measured for the
  same work with a throwaway script, about 1.8 seconds on one thread on the two largest of these
  repositories, and beside its bar of 3 seconds for an audit after an edit. An audit reads twice,
  the working tree and the base, and stage 1 has no cache, so twice the measured time is what is
  held against the bar; a result over it is reported to the owner and not fixed in this item. It
  judges, by reading the code, every
  import that is unresolved, every import that is ignored (MAP-4), and every import that is outside
  in a package no `package.json` lists, in another repository, through a link, or at an absolute
  path (RES-8): true, or false. Imports that name the same thing with the same outcome are one case
  and are judged once: the same path for a relative specifier, the same specifier under the same
  governing tsconfig for an alias, and the same specifier in the same package for a `#` specifier
  or a package's name. An
  import that led to the wrong file shows in none of these lists, so it also checks against the code
  one import of each kind that could: one for each workspace package imported by name, one for each
  `paths` pattern and each `imports` key that led to files, and each case that led to more than one
  file.
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
  still asks no more of a system's libraries than the skeleton spec's PKG-2a allows, which CI
  already checks on every target.
- **DEP-2** `git` on the `PATH` is a new need of the binary at run time, for `graph` and for
  every later command that reads a project. The plan names the oldest git version the commands it
  uses work with, the README states it, and a git older than that stops the run with a line
  (RUN-4). The images CI builds and tests the Linux targets in must hold a git at least that
  new, since the fixtures run there (FIX-1).
- **DEP-3** The oxc crates are released together and must move together. The repository's
  Dependabot settings put them in one group, so one pull request moves all their pins; moved one
  at a time, each pull request would fail to build.
- **DOC-1** `CONTRIBUTING.md` gains how a fixture is written and run, and the two new commands. The
  README's status says what the analyzer reads and that it reports no findings yet, and the test
  that holds the README to its present status text changes with it.

## Alternatives considered

- **Walking the folders with a library that reads ignore files.** It needs no `git` program. It
  was rejected because it only approximates git: the global excludes, `.git/info/exclude`, nested
  repositories and worktrees each need their own handling, and the design spec asks for what git
  ignores, and allows starting `git` to read.
- **Reading the repository through a git library, with no `git` program.** The binary would need
  nothing on the `PATH` and no oldest git version. Rejected because such a library is a second
  writing of git's rules for what is ignored, which is the very answer the design spec asks git
  for, and it is a large dependency in a binary that is otherwise small. Every machine that has
  a checkout has git.
- **Resolving by asking the file system, as Node and bundlers do.** That is what a resolver
  library does by default. It was rejected because the answer then depends on what is on disk: a
  built checkout resolves into build output, a fresh worktree does not, and a file system that
  ignores case resolves what a case-sensitive one refuses.
- **Reading `node_modules` when it is there, for a better answer.** Rejected by the design spec's
  AN-5: yak's worktrees never have it, so a better answer on one checkout is a different answer.
- **Writing all of the resolution in the analyzer's own code.** One set of rules in one place,
  with no library to bend. Rejected because how TypeScript finds a file for a path, and how
  tsconfig files extend and refer to each other, are intricate and already written and tested in
  `oxc_resolver`; a second writing of them is where silent mistakes would come from. The
  analyzer's own code is kept to what differs from those rules (RES-9).
- **Preferring a condition by name, or taking the first that leads to a file.** One import then
  leads to one file. Rejected because where conditions name different sources, the file not
  chosen loses an importer and would later be reported as unused, with nothing to show why.
- **Going on past a syntax error.** The parser can hand back a partial tree. Rejected by the
  design spec's AN-7: an import list read from half a file is a guess.
- **An existing parser for Vue components, MDX documents and HTML pages.** Each would read its
  format whole and correctly. Rejected because the analyzer needs three small things from them:
  the script blocks, the import statements and the `src` of a script element; the compilers Vue
  and MDX themselves ship are JavaScript programs, not Rust crates; and each crate added is a
  pinned dependency to keep moving. The fixtures hold the forms the design spec names, and the
  two readers cannot stop a run by being wrong (PARSE-4, PARSE-5).
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

- **`oxc_resolver` does less of RES-9's half than its documentation suggests.** Its file system
  is a part the caller supplies, which was read and not yet tried. The plan's first task tries
  it. A rule it does not give moves to the analyzer's own code with its fixture unchanged, which
  costs code and not correctness; if it cannot be used at all, the design spec is amended first.
- **An import that follows every condition leads to more files than any one program loads.** A
  file that only a never-used condition names then stays alive, and a later item will not report
  it as unused. That is the safe direction: the other choice reports live files.
- **A workspace the analyzer does not see.** A project whose packages are linked by something
  other than `workspaces`, `pnpm-workspace.yaml` or a `workspace:`, `file:` or `link:`
  dependency has its imports by name counted as outside. They show among the imports LIVE-2
  judges, when no `package.json` lists the name; where one does, the hand-off file's folders
  make the package a workspace package (PACK-3).
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
- **A mistyped import that is called ignored.** RES-4 calls an import ignored when git ignores
  the path as written or one of the source forms tried for it. In a repository that ignores an
  ending outright, a mistyped import written with that ending, or with no extension where the
  ending is a source form such as a type declaration's, is then ignored and not unresolved, and
  a later item will not report it. The other choice reports every import of
  a generated file. LIVE-2 judges every ignored import, so the named repositories show how often
  it happens.
- **A standard file with an odd but valid form stops the run.** PACK-6 stops on a field whose
  form the analyzer does not know. If npm, Node, pnpm or TypeScript accept a form this spec's
  builder did not think of, a project that is fine cannot be read until the analyzer learns it.
  LIVE-1 runs every named repository, and a stop there is fixed with a fixture, as LIVE-3 says.
- **Letter case in ignore patterns.** RUN-6a makes case count on every platform, so on Windows
  or macOS the analyzer may read a file that the user's own git there ignores through a pattern
  of another case. It then reads more than the user expects, the same on every machine. Whether
  git can be asked this way without changing which tracked files it lists is tried first in the
  plan; if it cannot, the owner is asked before the rule is given up, since RUN-8 depends on it.
- **Exact case reports imports that work on the owner's machine.** They fail on any
  case-sensitive system, so the reports are true; the cost is that a team on Windows alone sees
  them.
- **git is slow, missing, or refuses.** One run starts git a small, fixed number of times, never
  once per file, and LIVE-2 records the time. A missing git, or one that refuses a folder it does
  not trust, stops the run with git's own line (RUN-4).
- **The schema changes in yakshaver before its branch merges.** The copy records where it came
  from and a digest of its content (HO-1), and a changed schema is copied again with the fixtures
  that depend on it.
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

- The crate layout, and which crates read JSON, JSON with comments, YAML and globs, and parse the
  command line, each with its reason (DEP-1). Nothing in this spec asks for work in parallel; if
  the times of LIVE-2 call for it, RUN-8 still holds.
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

## Amendments

### 2026-11-12: An untracked name that differs from a tracked name only in letter case

The first plan's trial of git was carried out on 2026-11-12; its report, `trial-git.md`, is in the
work item's report folder. It found that letter case can count in ignore patterns without changing
which tracked files git lists, so RUN-6a stands as written. It also found one thing this spec did
not foresee.

On a file system that ignores letter case, a tracked file can lie on disk under a name that differs
from the name git holds only in letter case: git holds `docs/Guide.md`, and the disk holds
`docs/guide.md`. That happens when a file is renamed on such a system without telling git. With the
repository's own setting, git lists the file once, under the name it holds. With letter case made
to count, as RUN-6a asks, git lists it twice: the tracked name, and the name on disk as an untracked
file that nothing ignores. LIST-2 would then put one file into the file list under two names, and
only on the machines whose disk holds the other spelling, which is the difference between machines
that RUN-8 forbids.

- **LIST-10** An untracked path is left out of the file list when it differs only in letter case
  from the path of a tracked file that is itself in the list. The tracked path stands for the file.
  This narrows LIST-2, which otherwise holds. Three things make the rule exact:
  - A path here is the whole path from the root, folders included, so a folder whose name differs
    in case between git and the disk is covered: each file under it is one such pair.
  - Two paths differ only in letter case when they are equal once every letter of both is put in
    lower case by Unicode's own mapping. Nothing else is folded: not accents, and not the different
    ways Unicode can write one character. The rule is the analyzer's own and is the same on every
    file system.
  - The tracked file must be in the list. A tracked file that is missing from the working tree is
    not in it (LIST-3), and then the untracked path is kept. That is the case of a file renamed in
    letter case alone on a file system that keeps case apart: git calls the old name missing and
    the new one untracked, and the file is read under its new name.

  One notice says how many paths were left out and names the first pair, so a renamed folder of
  many files gives one notice and not hundreds.

  The rule applies on every platform. On a file system that keeps case apart, a tracked file and an
  untracked file can differ only in letter case and both be real; the untracked one is left out
  there too, and is not read until it is tracked or renamed, which the notice is there to say. What
  the rule does not make the same on every platform is a working tree whose disk holds another
  spelling than git: where the file system ignores case the file is listed under git's spelling,
  and where it keeps case apart, under the disk's. The two working trees differ, so RUN-8, which
  speaks of the same project, is not broken by it.
- **FIX-7** Fixtures hold the cases of LIST-10: a tracked file and an untracked path that differs
  from it only in letter case, where the list holds the tracked path alone and the notice names
  the pair; the same for a folder whose name differs in case; and a tracked file renamed in letter
  case alone. On a target whose file system ignores letter case a pair is one file on disk, made by
  renaming it, and the renamed file is listed under git's spelling; on a target that keeps case
  apart a pair is two files, and the renamed file is listed under its new name. Each test makes
  what its target allows and says which.
- **LIVE-6** LIVE-2's check that the file list is git's own takes out the paths LIST-10 leaves out,
  as it takes out the links, and the report says how many there were in each named repository.
