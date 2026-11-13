# Reading a project, part 6: the repository's own text, and the check on the named repositories: implementation plan

**Goal:** the README and the contributing guide say what the analyzer now does and needs, and the
check on the repositories named on the design item's record shows that the reading is right there.

**Architecture:** no code changes. Task 1 changes `README.md`, `CONTRIBUTING.md` and the two tests
that hold them to what they must say. Task 2 is the owner's check of the live run of `graph` on
the named repositories, whose results stay in the item's report folder.

**Spec:** docs/specs/2026-11-11-reading-a-project-design.md

**Implements:** DOC-1, DEP-2, OUT-2, LIVE-1 to LIVE-6

This is the sixth and last of the item's plans. It is worked on its own branch,
`feat/reading-6-live-check`, made from the fifth plan's branch, and its pull request is stacked on
the fifth plan's. It is opened ready for review, not as a draft: the owner said on 2026-11-13 that
pull requests in this repository need not be drafts.

## Global constraints

- Nothing committed names a machine, or a private repository other than yakshaver (the
  repository's rule): the named repositories, their files and the judged lists stay in the report
  folder, which is committed nowhere (LIVE-4).
- The live check runs on clean worktrees with nothing installed and nothing built, made with git
  in a temporary folder outside every repository (LIVE-1).
- A false import found by the check becomes a made-up fixture first, then a fix (LIVE-3).

## The decisions this plan makes

1. **The check was run while this plan was written,** with the fifth plan's build (`82d6f4d`), on
   2026-11-13. This plan changes no code, so the result holds for its branch; Task 2 runs it again
   with the branch's own build before handing over, and compares.
2. **The repositories and their commits:** each repository's local `main`, made into a detached
   worktree in one temporary folder. The two web apps in sub-folders are also run with the
   sub-folder as the root, with a hand-off file of their own.
3. **The report folder's layout:** `live/<run>/hand-off.json`, one folder per run holding only its
   hand-off file; `live/runs/` holding what each run printed, with and without a hand-off file; and
   `live/report.md`, which lists each glob with its reason and its count, the judged imports, the
   samples, the file-list comparison and the times.
4. **Git's own file list** (LIVE-2) is asked with RUN-6a's settings: `git --no-optional-locks
   --no-pager -c core.fsmonitor=false -c core.untrackedCache=false -c core.ignorecase=false
   ls-files -z --stage`, less modes 120000 and 160000 and the paths `ls-files -z --deleted` names,
   together with `ls-files -z --others --exclude-standard` less the folders it ends with `/`.
5. **The time** is the settled one of three warm runs with a hand-off file. The check found twice
   that time over the 3-second bar on the largest repository and at it on the second; LIVE-2 has
   this reported to the owner and not fixed in this item.
6. **The counts and the times go on the record** as a ruling of Task 2, which is how this item's
   record carries a result (LIVE-4).
7. **The worktrees are removed once the owner has validated the plan,** since a glob the owner
   rejects means a run again (LIVE-1).
8. **The README names git 2.15,** the oldest git the second plan's version check accepts (DEP-2),
   and says the run stops naming both versions when the git it finds is older.
9. **The contributing guide says that `graph`'s output is not part of the interface version,**
   and that a change of its form rewrites the fixtures' written-down output and none of their
   facts (OUT-2).

## Review focus

- **The `data` globs** take files out of the reading, so a wrong one hides a fault. Each is in the
  report with why and how many files it matches; the owner reads them when validating (LIVE-1).
- **The speed** is over the design spec's bar for an audit on the largest repository; it is the
  owner's to weigh, for the item that builds the audit.

### Task 1: the README and the contributing guide

**Implements:** DOC-1, DEP-2, OUT-2

**Test:** `node --test scripts/readme.test.mjs scripts/contributing.test.mjs`

**Files:**
- Modify: `README.md`, `CONTRIBUTING.md`
- Test: `scripts/readme.test.mjs`, `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: the commands of the first plan, the fixtures' helpers in `tests/support/project.rs`,
  and the second plan's oldest git, 2.15.
- Produces: the README's status that says what the analyzer reads and that it reports no findings
  yet, and the guide's sections on the commands and the fixtures.

- [ ] **Step 1: Write the failing tests**

```js path=scripts/readme.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { MAIN_DIR, manifest, ROOT } from './lib/run.mjs';

/** The README's text. */
const readme = readFileSync(join(ROOT, 'README.md'), 'utf8').replaceAll('\r\n', '\n');

test('the README says how to install and run the released binary (spec DOC-1)', () => {
  assert.ok(readme.includes('\n## Install\n'));
  assert.ok(readme.includes(`npm install ${manifest(MAIN_DIR).name}\n`));
  assert.ok(readme.includes('npx yak-analyzer --version\n'));
});

test('its status says what the analyzer reads, and that it reports no findings yet (spec DOC-1)', () => {
  assert.ok(readme.includes('reports\nno findings yet'));
  assert.ok(readme.includes('`yak-analyzer graph` prints what it read'));
  assert.ok(!readme.includes('analyzes nothing yet'));
});

test('it names the oldest git the analyzer reads a project with (spec DEP-2)', () => {
  assert.ok(readme.includes('`git` 2.15 or later on the `PATH`'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
  assert.ok(readme.includes('npm config delete @dabay6:registry --location=user\n'));
});
```

```js path=scripts/contributing.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** The contributing guide's text. */
const guide = readFileSync(join(ROOT, 'CONTRIBUTING.md'), 'utf8');

test('the guide says how a release is made, and what to do when one fails (spec PUB-12, PUB-14)', () => {
  for (const said of [
    '## Releasing',
    '### When a release fails',
    'docs.npmjs.com/trusted-publishers',
    'the environment `npm`',
    'the workflow file\n   `ci.yml`',
    '`NPM_TOKEN`',
    '"Bypass 2FA"',
    'refuse tokens',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
});

test('the guide names the commands, and says how a fixture is written and run (spec DOC-1, OUT-2)', () => {
  for (const said of [
    '## The commands',
    '`yak-analyzer interface`',
    '`yak-analyzer graph --hand-off <file>`',
    'not part of the interface\nversion',
    '## Fixtures',
    '`tests/support/project.rs`',
    'cargo test --release --locked --test resolve',
  ]) {
    assert.ok(guide.replaceAll('\r\n', '\n').includes(said), said);
  }
});

test("the gates it names run the scripts' tests as well as the launcher's (spec GATE-2)", () => {
  assert.ok(guide.includes('node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"'));
});
```

- [ ] **Step 2: Run them and see them fail**

```bash
node --test scripts/readme.test.mjs scripts/contributing.test.mjs
```

Expected: the tests of the status, of git's version and of the commands and fixtures fail, because
the README and the guide do not say those things yet; the other four pass.

- [ ] **Step 3: Write the README and the guide**

````markdown path=README.md
# yak-analyzer

A static analyzer for TypeScript and JavaScript, built for
[yakshaver](https://github.com/dabay6/yakshaver). It is being written to take the place of the
third-party analyzer yakshaver wraps today, so that yakshaver depends on an interface it designs.

## Status

The first release, 0.1.0, is a walking skeleton: a binary that answers its version, built and
tested on five targets and published to npm with provenance.

The code since then reads a project. It lists the files git lists, reads the packages and the
tsconfig files, finds every import of the scripts, Vue components, MDX documents and HTML pages,
and works out where each import leads, with build output mapped back to its source. It reports
no findings yet: `yak-analyzer graph` prints what it read, as JSON, and `yak-analyzer interface`
prints the interface version and the hand-off formats it reads.

## Install

yakshaver will install the analyzer by itself, at one pinned version, once its content pack
declares it. To try the released binary by hand:

```bash
npm install @dabay6/yak-analyzer
npx yak-analyzer --version
```

It needs Node 22.18 or later, and there is a binary for Windows on x64, macOS on arm64 and x64,
and Linux with glibc on x64 and arm64. No package runs a script when it is installed.

To read a project it needs `git` 2.15 or later on the `PATH`: it asks git for the files and for
what git ignores, and stops with one line naming both versions when the git it finds is older.

If your own npm settings map the `@dabay6` scope to another registry, npm looks for the package
there and does not find it. Remove that setting first:

```bash
npm config delete @dabay6:registry --location=user
```

## What it will do

It reports seven kinds of finding, each marked new on a branch or inherited from the branch's
base:

- unused files, unused exports and unused types
- unresolved imports
- circular dependencies
- functions over a complexity limit
- duplicated code

yakshaver runs it in two places: once over a whole branch before work is handed over for
validation, and in the background after each edit. It also serves its findings to an agent over
the Model Context Protocol (MCP).

## How it is designed

- **Rust, on the oxc libraries** (parser, semantic analysis, module resolver), each pinned to an
  exact version.
- **Entry points are data.** What a tool starts (a test runner's files, a bundler's entries, a
  configuration file found by name) is declared in yakshaver's content packs, and what one project
  wires by hand is declared by that project. yakshaver hands both to the analyzer in a hand-off
  file on every run. The analyzer holds no list of tools and has no plugin registry.
- **No configuration file in a project,** and no setting a person edits by hand.
- **The same findings everywhere.** Results do not depend on whether dependencies are installed,
  the project is built, or a cache is warm.
- **It only reads.** It never runs a project's code, writes nothing inside a checkout, and uses
  no network. The one program it may start is `git`.
- **SARIF 2.1.0** is its report format.
- **Shipped through npm:** one main package and one package per platform holding the binary.
- **TypeScript and JavaScript only.** It reads syntax, scopes and imports; it is not type-aware.

## What it will not do

- Fix code.
- Analyze other languages.
- Report unused class members, unused or unlisted dependencies, duplicate exports, or
  architecture boundaries.

## Where the design lives

The spec is `docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver
repository, which is not public yet. Until it is, this README is the public summary of it.

## License

[MIT](LICENSE).
````

````markdown path=CONTRIBUTING.md
# Contributing

This file holds the rules that shape every change here, the toolchain, and the gates a change
passes before it is reviewable.

## The design comes first

The analyzer is built from a written spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the
[yakshaver](https://github.com/dabay6/yakshaver) repository. Every requirement there has an ID,
and a change here names the IDs it implements. The spec fixes the contract between three
repositories: this one, yakshaver's engine, and yakshaver's content pack. A change to that
contract is made in the spec first.

## Rules that shape every change

- **A tool's convention is a declaration, never code here.** When a declaration can state what a
  tool starts, the analyzer does not learn the tool's name.
- **Tests use made-up projects.** Each fixture is a small project written for the test, with its
  expected findings written down. No code is copied from a real project.
- **No private names.** Nothing committed here names a machine, or a private repository other
  than yakshaver, which this program is built for; and nothing copies a private repository's
  code. That includes the repositories the analyzer is checked against.
- **It only reads.** A change that makes the analyzer run a project's code, write inside a
  checkout, or use the network is not accepted.
- **Dependencies are pinned to exact versions.** oxc changes in ordinary weekly releases.

## Toolchain

- Rust, at the one exact version `rust-toolchain.toml` names. With `rustup` installed, `cargo`
  fetches that version by itself the first time it runs here.
- Node 22.18 or later, for the launcher, its tests and the scripts in `scripts/`. They need no
  package beyond Node itself.

Every dependency in `Cargo.toml` is pinned to one exact version, written with `=`, and
`Cargo.lock` is committed.

## The gates

Four commands. All four pass before a change is reviewable.

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

CI runs them on all five targets, each on a runner of its own operating system and processor. It
then packs the npm packages, installs the packed files into a new folder with no registry to
reach, and runs the launcher there. The same steps run locally:

```bash
cargo build --release
node scripts/pack.mjs main
node scripts/pack.mjs platform
node scripts/packed-install.mjs
node scripts/check-floor.mjs
```

`check-floor.mjs` fails when the binary asks for a newer system than the Node yak needs: glibc
2.28 on Linux, macOS 11.0, and no separately installed C runtime on Windows.

## The commands

- `yak-analyzer --version` prints the version.
- `yak-analyzer interface` prints one line of JSON: the interface version, and the hand-off
  formats the analyzer reads.
- `yak-analyzer graph` reads the project in the folder it is started in and prints what it read,
  as one JSON document: the packages, the files, each file's imports and where each leads, and the
  notices. `yak-analyzer graph --hand-off <file>` reads the project the hand-off file names, with
  its declarations. A run that cannot go on prints one line on standard error and exits 1.

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
version; its form may change. When it does, the fixtures' written-down output changes in the same
change, and none of the facts they hold: each file, and each import's outcome.

## Fixtures

The tests in `tests/` run the built binary on made-up projects, one small project for each
pattern a real repository wires imports with. A test writes its project into a temporary folder,
makes it a git repository of its own there, with the tracked, untracked and ignored files it
needs, runs `graph` on it, and compares what is printed with what it expects, written in the
test. `tests/support/project.rs` holds the helpers: `Project::new`, `write`, `commit`, `git`,
`hand_off`, `graph` and `run`. Git and the binary run with the machine's and the user's own git
settings shut out, so a result does not depend on the machine.

Each test file is named in `Cargo.toml` with a `[[test]]` entry; a new file gets one. One file's
tests run with its name:

```bash
cargo test --release --locked --test resolve
```

## Releasing

A release is two acts of the owner's: merging the release pull request, and approving the
publishing job. Nothing else publishes. release-please keeps the release pull request open and up
to date from the commits on `main`; it holds the next version, written into every file that
carries it, and the notes.

1. Read npm's page on trusted publishers again (docs.npmjs.com/trusted-publishers), and its page
   on access tokens. If a rule has changed so that the steps below no longer work, stop: the work
   item that makes the release amends the spec first. One change is already announced: npm stops
   publishing with an access token in January 2027, and from then the rescue below needs another
   way.
2. Merge the release pull request. On `main`, release-please makes the tag and the GitHub
   release, the five target jobs build and test the binaries, and the publishing job then waits
   for the owner's approval.
3. While it waits, set up trusted publishing on npmjs.com for each package that has not yet been
   published that way; the notes of earlier releases say which have. A setup npm has voided is
   deleted first. Each setup names the repository `dabay6/yak-analyzer`, the workflow file
   `ci.yml` and the environment `npm`, and allows direct publishing. Read each setting back on
   all six packages.
4. Approve the publishing job. It publishes the platform packages first and the main package
   last, and adds a line to the release's notes saying which packages it published and whether a
   token was stored. Five more jobs then install the published packages from npmjs.com, one on
   each target, and check their provenance.

A package's setup stands once that package has been published by trusted publishing. A package
that is new, a later platform's for one, cannot start this way, since a setup needs a package
that exists: it is first published with a token, as the first six were.

### When a release fails

- **A target's job fails after the merge.** Nothing is published. Run the failed job again. If
  it passes, the publishing job follows. If it fails again, the fault is in the code: edit the
  GitHub release to say the version was not published to npm, and fix it in the next version.
- **The publishing job fails part of the way.** Run it again: it skips what is on npmjs.com and
  publishes the rest. It never publishes the main package before all five platform packages are
  there.
- **Trusted publishing fails.** A wrong setting on npmjs.com is put right and the job run again.
  When that does not cure it: let the packages still unpublished accept tokens again, make an
  access token that can write to `@dabay6`, with "Bypass 2FA" chosen and a life of days, store it
  as the secret `NPM_TOKEN` of the environment `npm`, and run the job again. If npm still does
  not fall back to the token, delete the setup of those packages and run the job once more.
  Afterwards delete the token, on GitHub and on npmjs.com, and set the packages to refuse tokens
  again. Those packages were not published by trusted publishing, so step 3 is done for them
  again at the next release.
- **A published package cannot be installed.** One of the five check jobs fails. Mark that
  version deprecated on npmjs.com with a line saying why, and fix it in the next version.
## Commits and pull requests

- Commits follow [Conventional Commits](https://www.conventionalcommits.org/):
  `type(scope): description`, in the imperative, with no trailing period.
- Work reaches `main` through a pull request, from a topic branch.
- Pull requests are merged with a merge commit or a rebase, never a squash, so each commit's type
  stays visible to the release tooling.

## License

By contributing you agree that your contribution is licensed under the [MIT license](LICENSE).
````

- [ ] **Step 4: Run every gate**

```bash
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0, and the Node tests report 49 passed.

- [ ] **Step 5: Commit, push, and open the pull request on the fifth plan's**

```bash
git add README.md CONTRIBUTING.md scripts/readme.test.mjs scripts/contributing.test.mjs
git commit -m "docs: say what the analyzer reads and how a fixture is written" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -u origin feat/reading-6-live-check
gh pr create --base feat/reading-5-pages --head feat/reading-6-live-check --title "docs: say what the analyzer reads and how a fixture is written" --body "The sixth part. Its description is written out in full when the plan is handed over for validation."
gh pr checks --watch
```

Expected: one commit on `feat/reading-6-live-check`, and the pull request's seven checks pass.

### Task 2: the check on the named repositories

**Implements:** LIVE-1, LIVE-2, LIVE-3, LIVE-4, LIVE-5, LIVE-6

**Check:** the owner reads the report folder's `live/report.md`: that each `data` and `generated`
glob is right, with its reason and its count; that every run finished and its file list is git's
own; that no import is unresolved, and that each ignored import and each sample judged true is
true; and the times beside the bar. A glob the owner rejects is removed and the runs judged again.

**Files:**
- None in this repository: the report folder, `.git/yak-reports/2026-11-12-reading-a-project/live/`,
  is committed nowhere (LIVE-4).

**Interfaces:**
- Consumes: the binary built from this branch; the hand-off files and the worktrees of decision 2.
- Produces: the report, and the counts and times on the record.

- [ ] **Step 1: Run `graph` again with this branch's build**

On each worktree, with no hand-off file and with its hand-off file, saving what it prints beside
the earlier runs, and comparing the two.

Expected: each run exits 0 and prints what the run of decision 1 printed.

- [ ] **Step 2: Record the counts and the times**

Record a ruling of this task that holds, for each repository, its files, its imports, the
unresolved, ignored and outside-unlisted counts, the times, and the speed against the bar.

Expected: the ruling is on the record.

- [ ] **Step 3: Hand the report to the owner, and remove the worktrees after validation**

```bash
git -C <repository> worktree remove <worktree>
```

Expected: after the owner validates the plan, each worktree is gone and each repository's
`git worktree list` no longer names it.
