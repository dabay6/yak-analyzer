# The skeleton, part 2: the release machinery: implementation plan

**Goal:** Everything a release needs is in the repository and on GitHub before the item's pull request merges:
release-please, a publishing job that waits for the owner's approval, the jobs that rehearse the publishing and
check what was published, the pin pull requests, and the rules on `main` and on the release tags.

**Architecture:** The one workflow gains its remaining jobs. On a push to `main`, release-please keeps the release
pull request and makes the tag; a small job then says, from the tag and from npmjs.com, whether this commit has
something to publish; only then does the publishing job ask for the owner's approval and publish the files the
target jobs packed. The publish and the dry run are one script. Settings that are not files (two environments, two
rulesets, one secret) are made in a task of their own and read back.

**Spec:** docs/specs/2026-11-11-skeleton-and-release-pipeline-design.md

**Implements:** CI-1, CI-2, CI-3, CI-4, CI-5, PUB-1, PUB-1a, PUB-2, PUB-2a, PUB-3, PUB-4, PUB-5, PUB-7, PUB-9, PUB-10, PUB-12, PUB-13, PUB-14, PUB-15, PUB-16, PUB-17, PROT-1, PROT-1a, PROT-2, PROT-3, DOC-1

This is the second of three plans, and it shares the first plan's branch and pull request: both are merged by
one merge. It publishes nothing. The third plan is the work after that merge (ORDER-1): the first release with a
token, the install check through yak, deleting the token and closing the packages to tokens, and the README's
install section (PUB-6, PUB-8, PUB-11, NAME-2, NAME-2a, CHK-1 to CHK-4).

## Global constraints

- The publishing job decides from two facts that are still true when a job is run again: a tag for the version in
  the tree points at the commit the run built, and that version is not yet whole on npmjs.com. It never publishes
  files built from another commit, and nothing is built again for publishing (PUB-2, PUB-2a).
- The platform packages are published first and the main package last, and the main package is not published
  until all five platform packages are on npmjs.com at that version (PUB-4, PUB-16). A package version already
  published is skipped (PUB-5).
- Every publish asks for provenance outright (PUB-15).
- The publishing job runs in an environment only `main` can use, and waits for the owner's approval (PROT-2). It
  always has the right to an identity token, and hands npm the token's secret when the environment has one; npm
  tries trusted publishing first and falls back to the token (PUB-13).
- The publish and the dry run are one script, `scripts/dry-run-publish.mjs`. Its way without an argument is the
  dry run; its publishing way must be asked for by name and runs only where the publishing job has set its
  marker (CI-5).
- The publishing job runs on Node 22.18, with npm fetched by `npx` at one exact version (CI-5).
- A pull request runs only jobs that publish nothing, with a read-only token and no secret: the job that packs
  the main package, the five target jobs, and the rehearsal. Those seven are the checks required on `main`
  (CI-2, PUB-17, PROT-1).
- The release pull request gets a real CI run with no step beyond the merge (PUB-1a).
- Every action is pinned to a full commit hash with its version beside it (CI-4).
- Committed text names no machine, and no private repository other than yakshaver.

## The decisions this plan makes

1. **A token of the owner's opens the release pull request.** GitHub starts no workflow for a pull request that
   a workflow opened with its own token, so release-please is given a fine-grained personal access token: this
   repository only, contents and pull requests read and write, nothing else. It is the secret
   `RELEASE_PLEASE_TOKEN` of an environment named `release` that only `main` can use and that asks for no
   approval. GitHub now also offers a second way, with no extra token: a pull request opened with the workflow's
   own token gets its checks once a person presses "Approve workflows to run". That is a step beyond the merge,
   which PUB-1a rules out, and GitHub says it is still rolling out; so the token is used.
2. **What the rules cannot say.** The token is the owner's own identity, so no rule can tell it from the owner:
   "no credential a workflow holds can merge" (PROT-1) stays a habit. What holds instead: the owner is the only
   person who can write to the repository; the token is readable only by the one job on `main`; and nothing is
   published without the owner's approval in the `npm` environment (PROT-2). The rule on release tags lets only
   the repository's administrators create, move or delete a tag named `v*`, and release-please makes the tag as
   the owner (PROT-1a).
3. **The repository's own ruleset on `main` holds the required checks alone.** The organization's ruleset
   already requires a pull request and forbids deleting or rewriting the branch, for every repository; the rules
   add up.
4. **A job of its own answers "is there something to publish".** The publishing job would otherwise ask for the
   owner's approval on every push to `main`. `scripts/release-facts.mjs` reads the tag from GitHub and the six
   versions from npmjs.com, and the publishing job starts only when a package is missing.
5. **The line in the release's notes is written by a job of its own,** the only one that can write the
   repository's contents; the publishing job itself can only read them.
6. **The jobs that check the published packages run on the runners themselves,** with no build image, since
   they install as a user would.
7. **Runs on `main` are no longer queued behind each other.** A release run may wait a long time for the owner's
   approval, and a queue would hold every later push.
8. **The six `package.json` files are rewritten in the layout `JSON.stringify` gives.** release-please writes a
   file back whole in that layout, so the release pull request then changes version numbers and nothing else.
9. **release-please reads its settings from two files:** `release-please-config.json`, with the `rust` kind for
   the crate and its lock file and the six manifests as extra files, and `.release-please-manifest.json`, which
   starts at 0.0.0. The first release is then 0.1.0 with no further setting, and its tag is `v0.1.0`.
10. **Dependabot opens the pin pull requests,** weekly, for the Cargo dependencies, the workflow's actions and
    the Rust version in `rust-toolchain.toml`.
11. **The rulesets are committed as files** under `.github/rulesets/`, so that what GitHub was given can be read
    and compared. GitHub does not read them from there; Task 5 sends them.
12. **The steps for releasing go into `CONTRIBUTING.md`** as the amendment asks, with one warning npm's pages
    give that the spec does not: npm stops publishing with an access token in January 2027. The first release
    comes before that; the rescue of PUB-14 will need another way afterwards, which is the kind of change
    PUB-12 already sends back to the spec.

## Review focus

- **Nothing here has run against a real release.** release-please's settings were run through its own code
  against a copy of this layout, with no repository; the publishing steps run as a dry run; the publishing with a
  token and provenance, the tag rule, and the conditions that start the publishing job and the check jobs are
  first used by the third plan's release.
- **The two ruleset bodies were not sent to GitHub before the plan was written,** and the administrators'
  role is named by a number GitHub's own pages do not list. Task 5 sends them and reads them back.
- **The conditions on the jobs that run only on `main`.** On a pull request they must all be skipped, and the
  seven required checks must still report. Task 4's run on the pull request is the test of that half.
- **A required check that never reports blocks every merge.** The seven names in the ruleset are the jobs'
  names as Task 4's run shows them; Task 5 compares them before it sends the ruleset.
- **An empty secret.** When the `npm` environment holds no token, the job hands npm an empty one. npm then
  publishes by trusted publishing where a setup matches, and says it needs a login where none does. This was
  tried as a dry run only.
- **`scripts/dry-run-publish.mjs --publish` on a person's machine** stops at its first check, before it reads
  or sends anything. A test covers it.

### Task 1: the six manifests in the layout release-please keeps

**Implements:** PUB-1

**Test:** `node --test "npm/**/*.test.mjs"`

**Files:**
- Modify: `npm/yak-analyzer/package.json`, `npm/platforms/win32-x64/package.json`,
  `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`,
  `npm/platforms/linux-x64/package.json`, `npm/platforms/linux-arm64/package.json`

**Interfaces:**
- Consumes: the first plan's manifests and their tests.
- Produces: the same manifests, with nothing changed but their layout.

This task has no test file of its own: the first plan's tests read these files, and pass before and after.

- [ ] **Step 1: Rewrite the manifests**

```json path=npm/yak-analyzer/package.json
{
  "name": "@dabay6/yak-analyzer",
  "version": "0.0.0",
  "description": "A static analyzer for TypeScript and JavaScript, built for yakshaver",
  "keywords": [
    "static-analysis",
    "dead-code",
    "typescript",
    "javascript",
    "sarif",
    "yakshaver"
  ],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": {
    "url": "https://github.com/dabay6/yak-analyzer/issues"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/yak-analyzer"
  },
  "license": "MIT",
  "type": "module",
  "imports": {
    "#yak-analyzer/*": "./lib/*"
  },
  "bin": {
    "yak-analyzer": "bin/yak-analyzer.js"
  },
  "files": [
    "bin",
    "lib"
  ],
  "engines": {
    "node": ">=22.18.0"
  },
  "optionalDependencies": {
    "@dabay6/yak-analyzer-darwin-arm64": "0.0.0",
    "@dabay6/yak-analyzer-darwin-x64": "0.0.0",
    "@dabay6/yak-analyzer-linux-arm64": "0.0.0",
    "@dabay6/yak-analyzer-linux-x64": "0.0.0",
    "@dabay6/yak-analyzer-win32-x64": "0.0.0"
  },
  "publishConfig": {
    "access": "public"
  }
}
```

```json path=npm/platforms/win32-x64/package.json
{
  "name": "@dabay6/yak-analyzer-win32-x64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for Windows on x64",
  "keywords": [
    "yak-analyzer",
    "static-analysis"
  ],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": {
    "url": "https://github.com/dabay6/yak-analyzer/issues"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/win32-x64"
  },
  "license": "MIT",
  "os": [
    "win32"
  ],
  "cpu": [
    "x64"
  ],
  "files": [
    "yak-analyzer.exe"
  ],
  "publishConfig": {
    "access": "public"
  }
}
```

```json path=npm/platforms/darwin-arm64/package.json
{
  "name": "@dabay6/yak-analyzer-darwin-arm64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for macOS on arm64",
  "keywords": [
    "yak-analyzer",
    "static-analysis"
  ],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": {
    "url": "https://github.com/dabay6/yak-analyzer/issues"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/darwin-arm64"
  },
  "license": "MIT",
  "os": [
    "darwin"
  ],
  "cpu": [
    "arm64"
  ],
  "files": [
    "yak-analyzer"
  ],
  "publishConfig": {
    "access": "public"
  }
}
```

```json path=npm/platforms/darwin-x64/package.json
{
  "name": "@dabay6/yak-analyzer-darwin-x64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for macOS on x64",
  "keywords": [
    "yak-analyzer",
    "static-analysis"
  ],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": {
    "url": "https://github.com/dabay6/yak-analyzer/issues"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/darwin-x64"
  },
  "license": "MIT",
  "os": [
    "darwin"
  ],
  "cpu": [
    "x64"
  ],
  "files": [
    "yak-analyzer"
  ],
  "publishConfig": {
    "access": "public"
  }
}
```

```json path=npm/platforms/linux-x64/package.json
{
  "name": "@dabay6/yak-analyzer-linux-x64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for Linux on x64, for glibc",
  "keywords": [
    "yak-analyzer",
    "static-analysis"
  ],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": {
    "url": "https://github.com/dabay6/yak-analyzer/issues"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/linux-x64"
  },
  "license": "MIT",
  "os": [
    "linux"
  ],
  "cpu": [
    "x64"
  ],
  "libc": [
    "glibc"
  ],
  "files": [
    "yak-analyzer"
  ],
  "publishConfig": {
    "access": "public"
  }
}
```

```json path=npm/platforms/linux-arm64/package.json
{
  "name": "@dabay6/yak-analyzer-linux-arm64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for Linux on arm64, for glibc",
  "keywords": [
    "yak-analyzer",
    "static-analysis"
  ],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": {
    "url": "https://github.com/dabay6/yak-analyzer/issues"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/linux-arm64"
  },
  "license": "MIT",
  "os": [
    "linux"
  ],
  "cpu": [
    "arm64"
  ],
  "libc": [
    "glibc"
  ],
  "files": [
    "yak-analyzer"
  ],
  "publishConfig": {
    "access": "public"
  }
}
```

- [ ] **Step 2: Run the tests**

```bash
node --test "npm/**/*.test.mjs"
```

Expected: 17 tests pass, none fail.

- [ ] **Step 3: Commit**

```bash
git add npm && git commit -m "style: lay the manifests out as release-please writes them"
```

Expected: the commit is made.

### Task 2: one script that publishes or rehearses, and the two that read npmjs.com

**Implements:** CI-5, PUB-2a, PUB-4, PUB-5, PUB-9, PUB-10, PUB-13, PUB-15, PUB-16, PUB-17

**Test:** `node --test "scripts/**/*.test.mjs"`

**Files:**
- Modify: `scripts/lib/run.mjs`, `scripts/dry-run-publish.mjs`
- Create: `scripts/release-facts.mjs`, `scripts/check-published.mjs`
- Test: `scripts/publishing.test.mjs`

**Interfaces:**
- Consumes: the first plan's `scripts/lib/run.mjs` (`manifest`, `packedFile`, `npm`, `fail`) and its packages.
- Produces: `TARGETS`, `PACKAGE_DIRS`, `PUBLISHER`, `REGISTRY`, `isPublished(name, version)`, `sleep(seconds)`
  and `output(name, value)` from `scripts/lib/run.mjs`; `scripts/dry-run-publish.mjs [--all] [--publish]`, which
  hands the workflow `tag`, `published` and `line`; `scripts/release-facts.mjs`, which hands it `tagged` and
  `publish`; `scripts/check-published.mjs`; the marker `YAK_ANALYZER_PUBLISHING_JOB`.

- [ ] **Step 1: Write the failing test**

```js path=scripts/publishing.test.mjs
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { basename, join } from 'node:path';
import { test } from 'node:test';

import { MAIN_DIR, PACKAGE_DIRS, PUBLISHER, REGISTRY, ROOT, TARGETS } from './lib/run.mjs';

/** The script that publishes, and tries the publishing. */
const SCRIPT = join(ROOT, 'scripts', 'dry-run-publish.mjs');

/**
 * Runs the script with the environment given, and no marker of the publishing job unless it is given one.
 *
 * @param {readonly string[]} args - The script's arguments.
 * @param {Record<string, string>} [env] - What to add to the environment.
 * @returns {import('node:child_process').SpawnSyncReturns<string>} How it ended.
 */
const script = (args, env = {}) => {
  const { YAK_ANALYZER_PUBLISHING_JOB: _marker, ...rest } = process.env;

  return spawnSync(process.execPath, [SCRIPT, ...args], { encoding: 'utf8', env: { ...rest, ...env } });
};

test('the publishing way is refused outside the publishing job, before anything is read or sent (spec CI-5)', () => {
  const refused = script(['--all', '--publish']);

  assert.equal(refused.status, 1);
  assert.equal(refused.stdout, '');
  assert.equal(refused.stderr, 'only the publishing job of the workflow publishes; run this script without --publish for a dry run\n');
});

test('a marker that is not exactly the publishing job\'s is refused too', () => {
  assert.equal(script(['--publish'], { YAK_ANALYZER_PUBLISHING_JOB: '1' }).status, 1);
});

test('an argument the script does not know stops it', () => {
  const stopped = script(['--force']);

  assert.equal(stopped.status, 1);
  assert.match(stopped.stderr, /^unknown argument --force; /u);
});

test('the packages are published platform packages first and the main package last (spec PUB-4)', () => {
  assert.equal(PACKAGE_DIRS.length, 6);
  assert.equal(PACKAGE_DIRS.at(-1), MAIN_DIR);
  assert.deepEqual(PACKAGE_DIRS.slice(0, 5).map((dir) => basename(dir)), TARGETS);
});

test('the targets are the folders under npm/platforms', () => {
  assert.deepEqual([...TARGETS].sort(), readdirSync(join(ROOT, 'npm', 'platforms')).sort());
});

test('the npm that publishes is one exact version of 11.5.1 or later that runs on Node 22.18 (spec CI-5)', () => {
  const [, major, minor, patch] = /^npm@(\d+)\.(\d+)\.(\d+)$/u.exec(PUBLISHER).map(Number);

  assert.equal(major, 11);
  assert.ok(minor > 5 || (minor === 5 && patch >= 1));
});

test('every command names npmjs.com as the registry of the scope', () => {
  assert.equal(REGISTRY, '--@dabay6:registry=https://registry.npmjs.org/');
});
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test "scripts/**/*.test.mjs"
```

Expected: the file fails, because `scripts/lib/run.mjs` does not export `PACKAGE_DIRS` yet.

- [ ] **Step 3: Write the scripts**

```js path=scripts/lib/run.mjs
import { spawnSync } from 'node:child_process';
import { appendFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

/** The repository's root. */
export const ROOT = join(import.meta.dirname, '..', '..');

/** Where the packed files go. */
export const DIST = join(ROOT, 'dist');

/** This machine's target, as Node names the operating system and the processor. */
export const TARGET = `${process.platform}-${process.arch}`;

/** The targets a platform package exists for. */
export const TARGETS = ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64'];

/** The main package's folder. */
export const MAIN_DIR = join(ROOT, 'npm', 'yak-analyzer');

/** The folder of this machine's platform package. */
export const PLATFORM_DIR = join(ROOT, 'npm', 'platforms', TARGET);

/**
 * Every package's folder in the order the packages are published: the platform packages first and the main
 * package last, so a main package never exists without its platform packages (spec PUB-4).
 */
export const PACKAGE_DIRS = [
  ...TARGETS.map((target) => {
    return join(ROOT, 'npm', 'platforms', target);
  }),
  MAIN_DIR,
];

/** The binary's file name on this machine. */
export const BINARY = process.platform === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer';

/** The release build of the binary. */
export const BUILT = join(ROOT, 'target', 'release', BINARY);

/**
 * The npm that publishes, fetched by `npx` at this exact version (spec CI-5). Trusted publishing needs 11.5.1 or
 * later, and npm 12 does not run on Node 22.18. It is moved by hand.
 */
export const PUBLISHER = 'npm@11.21.2';

/** Where the packages are published, said on each command so that a machine's own settings cannot change it. */
export const REGISTRY = '--@dabay6:registry=https://registry.npmjs.org/';

/**
 * Reads a package's manifest.
 *
 * @param {string} dir - The package's folder.
 * @returns {{ bin?: Record<string, string>, name: string, version: string }} Its `package.json`.
 */
export function manifest(dir) {
  return JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
}

/**
 * The file `npm pack` writes for a package: its name without the `@`, with `/` as `-`, then its version.
 *
 * @param {string} dir - The package's folder.
 * @returns {string} The packed file, in `dist/`.
 */
export function packedFile(dir) {
  const { name, version } = manifest(dir);

  return join(DIST, `${name.replace('@', '').replace('/', '-')}-${version}.tgz`);
}

/**
 * Runs npm or npx and waits for it. Both are batch files on Windows, which Node starts only through a shell, and
 * the shell splits an argument at its spaces unless it is quoted.
 *
 * @param {'npm' | 'npx'} program - The program.
 * @param {readonly string[]} args - Its arguments.
 * @param {import('node:child_process').SpawnSyncOptions} [options] - Where to run it, and where its output goes.
 * @returns {import('node:child_process').SpawnSyncReturns<string>} How it ended.
 */
export function npm(program, args, options = {}) {
  const shell = process.platform === 'win32';
  const quoted = args.map((arg) => {
    return shell && arg.includes(' ') ? `"${arg}"` : arg;
  });

  return spawnSync(program, quoted, { encoding: 'utf8', shell, ...options });
}

/**
 * Stops the script with one line.
 *
 * @param {string} message - What went wrong, and what to do.
 * @returns {never}
 */
export function fail(message) {
  console.error(message);
  process.exit(1);
}

/**
 * Asks npmjs.com whether a package's version is there for anyone to install. npm answers a version that is not
 * there, and a package that is not there, with the code E404; anything else is a failure of the question itself.
 *
 * @param {string} name - The package.
 * @param {string} version - The version.
 * @returns {boolean} True when it is published.
 */
export function isPublished(name, version) {
  const asked = npm('npx', ['--yes', PUBLISHER, 'view', `${name}@${version}`, 'version', '--json', REGISTRY]);

  if (asked.status === 0) {
    return true;
  }

  let code;

  try {
    code = JSON.parse(asked.stdout).error?.code;
  } catch {
    code = undefined;
  }

  if (code !== 'E404') {
    fail(`npmjs.com could not be asked about ${name}@${version}: ${`${asked.stderr}${asked.stdout}`.trim()}`);
  }

  return false;
}

/**
 * Waits, without starting a process.
 *
 * @param {number} seconds - How long.
 */
export function sleep(seconds) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, seconds * 1000);
}

/**
 * Hands a value to the workflow's later jobs. It does nothing outside a workflow.
 *
 * @param {string} name - The output's name.
 * @param {string} value - Its value, on one line.
 */
export function output(name, value) {
  console.log(`${name}=${value}`);

  if (process.env.GITHUB_OUTPUT !== undefined) {
    appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
  }
}
```

```js path=scripts/dry-run-publish.mjs
/**
 * Publishes the packed files in `dist/`, or tries the publishing without publishing (spec PUB-4, PUB-10, PUB-13,
 * PUB-15 to PUB-17, CI-5). One script does both, so the npm, the registry, the order and the answer for a version
 * that is already published are written once.
 *
 * `node scripts/dry-run-publish.mjs` is the dry run, over whichever packed files `dist/` holds.
 * `--all` asks for all six to be there, as the publishing job has them.
 * `--publish` publishes. It runs only inside the publishing job of the workflow.
 */
import { existsSync } from 'node:fs';

import {
  DIST,
  fail,
  isPublished,
  MAIN_DIR,
  manifest,
  npm,
  output,
  PACKAGE_DIRS,
  packedFile,
  PUBLISHER,
  REGISTRY,
  sleep,
} from './lib/run.mjs';

/** The marker the publishing job sets for this script, and nothing else does. */
const PUBLISHING_JOB = 'YAK_ANALYZER_PUBLISHING_JOB';

/** What npm's dry run says when the version is already published. */
const ALREADY_PUBLISHED = 'cannot publish over the previously published versions';

/** How many times, a quarter of a minute apart, npmjs.com is asked for a package that was just published. */
const TRIES = 20;

/** The arguments this script knows. */
const KNOWN = ['--all', '--publish'];

/** The arguments it was given. */
const given = process.argv.slice(2);

/** Whether to publish for real. */
const publishing = given.includes('--publish');

for (const arg of given) {
  if (!KNOWN.includes(arg)) {
    fail(`unknown argument ${arg}; usage: node scripts/dry-run-publish.mjs [--all] [--publish]`);
  }
}

if (publishing && process.env[PUBLISHING_JOB] !== 'true') {
  fail('only the publishing job of the workflow publishes; run this script without --publish for a dry run');
}

/** The six packages in publishing order: the platform packages, then the main package (spec PUB-4). */
const packages = PACKAGE_DIRS.map((dir) => {
  const { name, version } = manifest(dir);

  return { file: packedFile(dir), isMain: dir === MAIN_DIR, name, version };
});

/** The packages whose packed file is in `dist/`. */
const packed = packages.filter(({ file }) => {
  return existsSync(file);
});

if (packed.length === 0) {
  fail(`${DIST} holds no packed file; run scripts/pack.mjs first`);
}

if ((publishing || given.includes('--all')) && packed.length !== packages.length) {
  const missing = packages.filter(({ file }) => {
    return !existsSync(file);
  });

  fail(`${DIST} must hold all six packed files; missing: ${missing.map(({ name }) => name).join(', ')}`);
}

/**
 * Stops unless every platform package is on npmjs.com at the version, for anyone to install, so that the main
 * package is never published without one (spec PUB-16). A package just published can take a moment to show.
 *
 * @param {string} version - The version being published.
 */
const platformsArePublished = (version) => {
  for (const { isMain, name } of packages) {
    if (isMain) {
      continue;
    }

    let found = false;

    for (let i = 0; i < TRIES && !found; i += 1) {
      if (i > 0) {
        sleep(15);
      }

      found = isPublished(name, version);
    }

    if (!found) {
      fail(`${name}@${version} is not on npmjs.com, so the main package is not published; put it right and run the job again`);
    }
  }
};

/** The packages this run published. */
const published = [];

for (const { file, isMain, name, version } of packed) {
  if (!publishing) {
    const tried = npm('npx', ['--yes', PUBLISHER, 'publish', '--dry-run', '--access', 'public', REGISTRY, file]);
    const said = `${tried.stdout ?? ''}${tried.stderr ?? ''}`;

    if (tried.status === 0) {
      console.log(`the dry run passed for ${name}@${version}`);
    } else if (said.includes(ALREADY_PUBLISHED)) {
      console.log(`the dry run passed for ${name}@${version}; that version is already published`);
    } else {
      console.error(said);
      fail(`npm's dry run refused ${name}@${version}`);
    }

    continue;
  }

  if (isPublished(name, version)) {
    console.log(`${name}@${version} is already published; skipped`);

    continue;
  }

  if (isMain) {
    platformsArePublished(version);
  }

  // Provenance is asked for outright: npm writes it by itself only for a trusted publish (spec PUB-15).
  const sent = npm('npx', ['--yes', PUBLISHER, 'publish', file, '--provenance', '--access', 'public', REGISTRY], {
    stdio: 'inherit',
  });

  if (sent.status !== 0) {
    fail(`npm could not publish ${name}@${version}; put it right and run the job again`);
  }

  published.push(name);
}

if (publishing) {
  const stored = (process.env.NODE_AUTH_TOKEN ?? '') !== '';
  const way = stored
    ? 'A token was stored for this run, so it does not show which way npm took.'
    : 'No token was stored for this run, so they were published by trusted publishing.';

  output('tag', `v${packages[0].version}`);
  output('published', published.join(' '));
  output('line', published.length === 0 ? '' : `Published to npm by one run of the job: ${published.join(', ')}. ${way}`);
}
```

```js path=scripts/release-facts.mjs
/**
 * Says whether this commit is a release with something left to publish, from two facts that are still true when
 * a job is run again (spec PUB-2a): a tag for the version in the tree points at this very commit, and that
 * version is not yet whole on npmjs.com. It never asks release-please whether a release was just made.
 *
 * It hands the workflow `tagged` (the tag is on this commit) and `publish` (and a package is still missing).
 */
import { spawnSync } from 'node:child_process';

import { fail, isPublished, MAIN_DIR, manifest, output, PACKAGE_DIRS } from './lib/run.mjs';

/** The version in the tree. */
const { version } = manifest(MAIN_DIR);

/** The tag a release of that version has. */
const tag = `v${version}`;

/**
 * Runs git and gives what it printed.
 *
 * @param {readonly string[]} args - git's arguments.
 * @returns {string} Its standard output, trimmed.
 */
const git = (args) => {
  const ended = spawnSync('git', args, { encoding: 'utf8' });

  if (ended.status !== 0) {
    fail(`git ${args.join(' ')} failed: ${ended.error?.message ?? ended.stderr}`);
  }

  return ended.stdout.trim();
};

/** The commit this run built. */
const commit = process.env.GITHUB_SHA ?? git(['rev-parse', 'HEAD']);

/** The tag as the repository on GitHub has it: its own line, and a second line when the tag is an object. */
const listed = git(['ls-remote', '--tags', 'origin', `refs/tags/${tag}`, `refs/tags/${tag}^{}`])
  .split('\n')
  .filter((line) => {
    return line.length > 0;
  })
  .map((line) => {
    const [sha, ref] = line.split('\t');

    return { ref, sha };
  });

/** The commit the tag points at, through a tag object when there is one. */
const tagged = (listed.find(({ ref }) => ref.endsWith('^{}')) ?? listed[0])?.sha;

/** Whether the tag is on this commit. */
const isTagged = tagged === commit;

/** Whether a package of the version is still missing from npmjs.com. Asked only for a tagged commit. */
const isMissingOne =
  isTagged &&
  PACKAGE_DIRS.some((dir) => {
    return !isPublished(manifest(dir).name, version);
  });

console.log(
  isTagged
    ? `${tag} is on this commit; ${isMissingOne ? 'a package is still to be published' : 'all six packages are published'}`
    : `${tag} is ${tagged === undefined ? 'not a tag yet' : 'on another commit'}; nothing to publish`
);
output('tagged', String(isTagged));
output('publish', String(isMissingOne));
```

```js path=scripts/check-published.mjs
/**
 * Installs the published main package by its name and version from npmjs.com into a new folder, as a user's npm
 * will, and runs the launcher's version answer (spec PUB-9). It then reads from npmjs.com that the main package
 * and this machine's platform package carry provenance at that version (spec PUB-15). It needs no secret.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { fail, MAIN_DIR, manifest, npm, PLATFORM_DIR, PUBLISHER, REGISTRY, sleep } from './lib/run.mjs';

/** How many times, half a minute apart, the install is tried while npmjs.com catches up with a publish. */
const TRIES = 10;

/** The main package's name, version and `bin` map. */
const { bin, name, version } = manifest(MAIN_DIR);

/** This machine's platform package. */
const platform = manifest(PLATFORM_DIR).name;

/** The line the binary prints for its version (spec BIN-1). */
const expected = `yak-analyzer ${version}\n`;

/** A new folder to install into. */
const folder = mkdtempSync(join(tmpdir(), 'yak-analyzer-published-'));

/** What went wrong, if anything did. */
let problem;

try {
  // The folder needs a manifest of its own, or npm installs into the nearest folder above that has one.
  writeFileSync(join(folder, 'package.json'), '{"private":true}\n');

  let installed = false;

  for (let i = 0; i < TRIES && !installed; i += 1) {
    if (i > 0) {
      sleep(30);
    }

    installed =
      npm('npm', ['install', '--ignore-scripts', '--no-audit', '--no-fund', '--no-update-notifier', REGISTRY, `${name}@${version}`], {
        cwd: folder,
        stdio: 'inherit',
      }).status === 0;
  }

  if (!installed) {
    problem = `npm could not install ${name}@${version} from npmjs.com`;
  } else if (existsSync(join(folder, 'node_modules', platform))) {
    // yak starts a tool by running its `bin` file with Node, through no wrapper script.
    const answered = spawnSync(process.execPath, [join(folder, 'node_modules', name, bin['yak-analyzer']), '--version'], {
      encoding: 'utf8',
    });

    if (answered.status !== 0 || answered.stdout !== expected) {
      problem = `the installed launcher answered ${JSON.stringify(answered.stdout)} and exited ${answered.status} (${answered.stderr.trim()}); expected ${JSON.stringify(expected)}`;
    }
  } else {
    problem = `npm installed ${name}@${version} without ${platform}`;
  }
} finally {
  rmSync(folder, { force: true, recursive: true });
}

if (problem !== undefined) {
  fail(problem);
}

console.log(`${name}@${version}, installed from npmjs.com, answers ${expected.trim()}`);

for (const published of [name, platform]) {
  const asked = npm('npx', ['--yes', PUBLISHER, 'view', `${published}@${version}`, 'dist.attestations', '--json', REGISTRY]);
  let kind;

  try {
    kind = JSON.parse(asked.stdout).provenance?.predicateType;
  } catch {
    kind = undefined;
  }

  // npm prints nothing for a version that carries no provenance.
  if (asked.status !== 0 || typeof kind !== 'string' || !kind.startsWith('https://slsa.dev/provenance/')) {
    fail(`${published}@${version} carries no provenance on npmjs.com: ${`${asked.stdout}${asked.stderr}`.trim()}`);
  }

  console.log(`${published}@${version} carries provenance`);
}
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
node --test "scripts/**/*.test.mjs" "npm/**/*.test.mjs"
```

Expected: 24 tests pass, none fail.

- [ ] **Step 5: Run the scripts' safe ways on this machine**

```bash
cargo build --release && node scripts/pack.mjs main && node scripts/pack.mjs platform
node scripts/dry-run-publish.mjs
node scripts/dry-run-publish.mjs --all
node scripts/dry-run-publish.mjs --all --publish
node scripts/release-facts.mjs
```

Expected: the plain dry run passes for this machine's platform package and then for the main package, in that
order. With `--all` it stops, naming the four platform packages this machine did not pack. With `--publish` it
stops with `only the publishing job of the workflow publishes; run this script without --publish for a dry run`
and sends nothing. `release-facts.mjs` prints `v0.0.0 is not a tag yet; nothing to publish`, then
`tagged=false` and `publish=false`.

- [ ] **Step 6: Commit**

```bash
git add scripts && git commit -m "build: publish and rehearse with one script, and read npmjs.com back"
```

Expected: the commit is made.

### Task 3: release-please's settings and the pin pull requests

**Implements:** PUB-1, PUB-7

**Test:** `node --test scripts/release-settings.test.mjs`

**Files:**
- Create: `release-please-config.json`, `.release-please-manifest.json`, `.github/dependabot.yml`
- Test: `scripts/release-settings.test.mjs`

**Interfaces:**
- Consumes: Task 2's `PACKAGE_DIRS`; the paths of the crate and the six manifests.
- Produces: the settings Task 4's release-please job reads; Dependabot's weekly pull requests.

release-please reads these files only on `main`, and Dependabot only once they are there, so the test checks
what the files say; the third plan's release is their first real use.

- [ ] **Step 1: Write the failing test**

```js path=scripts/release-settings.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { test } from 'node:test';

import { MAIN_DIR, manifest, PACKAGE_DIRS, ROOT } from './lib/run.mjs';

/**
 * Reads a file under the repository's root.
 *
 * @param {string} path - The file, from the root.
 * @returns {string} Its text.
 */
const read = (path) => readFileSync(join(ROOT, path), 'utf8');

/** release-please's settings for the one thing it releases, the repository's root. */
const settings = JSON.parse(read('release-please-config.json')).packages['.'];

test('release-please bumps the crate itself, and each of the six manifests as an extra file (spec PKG-8)', () => {
  const manifests = PACKAGE_DIRS.map((dir) => relative(ROOT, join(dir, 'package.json')).replaceAll('\\', '/'));

  assert.equal(settings['release-type'], 'rust');
  assert.deepEqual(
    settings['extra-files'].filter((file) => typeof file === 'string').sort(),
    [...manifests].sort()
  );
});

test("it bumps the main package's optional dependencies too, and nothing else by a path of its own", () => {
  assert.deepEqual(
    settings['extra-files'].filter((file) => typeof file !== 'string'),
    [{ jsonpath: '$.optionalDependencies.*', path: 'npm/yak-analyzer/package.json', type: 'json' }]
  );
});

test('the tag of a release is `v` and the version, with no name before it (spec PUB-2a)', () => {
  assert.equal(settings['include-component-in-tag'], false);
  assert.equal(settings['include-v-in-tag'], true);
});

test("release-please's own record of the version is the version in the tree", () => {
  assert.deepEqual(JSON.parse(read('.release-please-manifest.json')), { '.': manifest(MAIN_DIR).version });
});

test('pull requests move the Cargo pins, the actions and the Rust version (spec PUB-7)', () => {
  const dependabot = read('.github/dependabot.yml');

  for (const ecosystem of ['cargo', 'github-actions', 'rust-toolchain']) {
    assert.ok(dependabot.includes(`package-ecosystem: ${ecosystem}`), ecosystem);
  }
});
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test scripts/release-settings.test.mjs
```

Expected: the file fails, because `release-please-config.json` does not exist.

- [ ] **Step 3: Write the settings**

```json path=release-please-config.json
{
  "$schema": "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json",
  "packages": {
    ".": {
      "release-type": "rust",
      "package-name": "yak-analyzer",
      "include-component-in-tag": false,
      "include-v-in-tag": true,
      "bump-minor-pre-major": true,
      "extra-files": [
        "npm/yak-analyzer/package.json",
        {
          "type": "json",
          "path": "npm/yak-analyzer/package.json",
          "jsonpath": "$.optionalDependencies.*"
        },
        "npm/platforms/win32-x64/package.json",
        "npm/platforms/darwin-arm64/package.json",
        "npm/platforms/darwin-x64/package.json",
        "npm/platforms/linux-x64/package.json",
        "npm/platforms/linux-arm64/package.json"
      ]
    }
  }
}
```

```json path=.release-please-manifest.json
{
  ".": "0.0.0"
}
```

```yaml path=.github/dependabot.yml
# Pull requests that move each pin, opened without anyone asking (spec PUB-7). CI decides whether one can
# merge; none merges by itself. The npm that publishes is not here: it is moved by hand (spec CI-5).
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: rust-toolchain
    directory: /
    schedule:
      interval: weekly
```

- [ ] **Step 4: Run it and see it pass, and commit**

```bash
node --test scripts/release-settings.test.mjs
git add release-please-config.json .release-please-manifest.json .github/dependabot.yml scripts && git commit -m "build: add release-please's settings and the pin pull requests"
```

Expected: 5 tests pass, and the commit is made.

### Task 4: the workflow's remaining jobs

**Implements:** CI-1, CI-2, CI-3, CI-4, CI-5, PUB-1a, PUB-2, PUB-2a, PUB-9, PUB-13, PUB-17, PROT-1, PROT-1a, PROT-2

**Test:** `node --test scripts/workflow.test.mjs`

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `.github/rulesets/main.json`, `.github/rulesets/release-tags.json`
- Test: `scripts/workflow.test.mjs`

**Interfaces:**
- Consumes: Task 2's scripts and what they hand the workflow; Task 3's settings; the secrets
  `RELEASE_PLEASE_TOKEN` (environment `release`) and `NPM_TOKEN` (environment `npm`), which Task 5 and the third
  plan make.
- Produces: the jobs `rehearse`, `release-please`, `release-facts`, `publish`, `release-notes` and
  `check-published`; the seven check names a pull request reports; the two ruleset bodies Task 5 sends.

The test reads the workflow as text and checks what must hold in it: the required checks' names, the pinned
actions, and which job alone holds each right and each secret. A workflow is still only tested by running it:
Step 4's run on the pull request tests the jobs a pull request runs, and that the others are skipped there.

- [ ] **Step 1: Write the failing test**

```js path=scripts/workflow.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT, TARGETS } from './lib/run.mjs';

/** The workflow's text. It is read as text: nothing here parses YAML, and nothing needs to. */
const workflow = readFileSync(join(ROOT, '.github', 'workflows', 'ci.yml'), 'utf8');

/**
 * Reads a ruleset that is sent to GitHub.
 *
 * @param {string} name - The file's name under `.github/rulesets/`.
 * @returns {Record<string, any>} The ruleset.
 */
const ruleset = (name) => JSON.parse(readFileSync(join(ROOT, '.github', 'rulesets', name), 'utf8'));

/**
 * How many times a piece of text is in the workflow.
 *
 * @param {string} text - The text.
 * @returns {number} The count.
 */
const count = (text) => workflow.split(text).length - 1;

/** The two jobs a pull request runs that are not a target's. */
const NAMED = ['Pack the main package', 'Rehearse the publishing'];

test('the checks required on main are the seven jobs a pull request runs (spec PROT-1, PUB-17)', () => {
  const required = ruleset('main.json').rules[0].parameters.required_status_checks.map(({ context }) => context);
  const built = [...workflow.matchAll(/- \{ target: ([\w-]+), runner: [\w.-]+, image: /gu)].map(([, target]) => target);

  assert.deepEqual(built, TARGETS);
  assert.deepEqual([...required].sort(), [...NAMED, ...TARGETS].sort());

  for (const name of NAMED) {
    assert.match(workflow, new RegExp(`\\n    name: ${name}\\r?\\n`, 'u'), name);
  }
});

test('the published packages are checked on the same five targets (spec PUB-9)', () => {
  const checked = [...workflow.matchAll(/- \{ target: ([\w-]+), runner: [\w.-]+ \}/gu)].map(([, target]) => target);

  assert.deepEqual(checked, TARGETS);
});

test('every action is pinned to a full commit hash, with its version beside it (spec CI-4)', () => {
  const uses = [...workflow.matchAll(/uses: (\S+)(.*)/gu)];

  assert.ok(uses.length > 0);

  for (const [, action, rest] of uses) {
    assert.match(action, /^[\w./-]+@[0-9a-f]{40}$/u, action);
    assert.match(rest.trimEnd(), /^ # v\d+\.\d+\.\d+$/u, action);
  }
});

test("no trigger runs a fork's code with the repository's rights (spec CI-2)", () => {
  assert.equal(count('pull_request_target'), 0);
  assert.equal(count('workflow_run'), 0);
});

test('one job can ask for an identity token, and one other can write the contents (spec CI-3)', () => {
  assert.equal(count('id-token: write'), 1);
  assert.equal(count('contents: write'), 1);
  assert.equal(count('\npermissions:\n  contents: read\n') + count('\r\npermissions:\r\n  contents: read\r\n'), 1);
});

test('the publishing job alone has the marker, the npm token and the publishing environment (spec CI-5, PROT-2)', () => {
  assert.equal(count("YAK_ANALYZER_PUBLISHING_JOB: 'true'"), 1);
  assert.equal(count('secrets.NPM_TOKEN'), 1);
  assert.equal(count('environment: npm'), 1);
  assert.equal(count('--publish'), 1);
});

test("release-please alone has the token that opens the release pull request, in its own environment (spec PUB-1a)", () => {
  assert.equal(count('secrets.RELEASE_PLEASE_TOKEN'), 1);
  assert.equal(count('environment: release'), 1);
});

test('only administrators can make, move or delete a release tag (spec PROT-1a)', () => {
  const tags = ruleset('release-tags.json');

  assert.equal(tags.target, 'tag');
  assert.equal(tags.enforcement, 'active');
  assert.deepEqual(tags.conditions.ref_name.include, ['refs/tags/v*']);
  assert.deepEqual(tags.bypass_actors, [{ actor_id: 5, actor_type: 'RepositoryRole', bypass_mode: 'always' }]);

  for (const rule of ['creation', 'update', 'deletion']) {
    assert.ok(
      tags.rules.some(({ type }) => type === rule),
      rule
    );
  }
});
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test scripts/workflow.test.mjs
```

Expected: the tests fail, because `.github/rulesets/main.json` does not exist and the workflow has no rehearsal.

- [ ] **Step 3: Write the workflow and the rulesets**

```yaml path=.github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

# A job can read the repository and nothing more unless it says otherwise (spec CI-2, CI-3).
permissions:
  contents: read

# A pull request's newer run replaces its older one. Runs on main are never cancelled or queued behind each
# other: one of them may be a release waiting for the owner's approval.
concurrency:
  group: ci-${{ github.event_name == 'pull_request' && github.ref || github.sha }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

env:
  # The oldest macOS the Node yak needs runs on (spec PKG-2a). It means nothing off macOS.
  MACOSX_DEPLOYMENT_TARGET: '11.0'

jobs:
  pack-main:
    name: Pack the main package
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
      # The main package holds no binary. It is packed once, and every target tests this very file (spec PKG-9).
      - run: node scripts/pack.mjs main
      - run: node scripts/dry-run-publish.mjs
      - uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          name: pkg-main
          path: dist/*.tgz
          if-no-files-found: error

  target:
    name: ${{ matrix.target }}
    needs: pack-main
    strategy:
      fail-fast: false
      matrix:
        include:
          - { target: win32-x64, runner: windows-2025, image: '' }
          - { target: darwin-arm64, runner: macos-15, image: '' }
          - { target: darwin-x64, runner: macos-15-intel, image: '' }
          # The Linux binaries are built on glibc 2.28, the oldest the Node yak needs runs on (spec PKG-2a).
          - { target: linux-x64, runner: ubuntu-24.04, image: quay.io/pypa/manylinux_2_28_x86_64 }
          - { target: linux-arm64, runner: ubuntu-24.04-arm, image: quay.io/pypa/manylinux_2_28_aarch64 }
    runs-on: ${{ matrix.runner }}
    container: ${{ matrix.image }}
    defaults:
      run:
        shell: bash
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      # The oldest Node yak supports, kept as a floor on purpose (spec CI-1a).
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
      - name: Install rustup where the image has none
        if: matrix.image != ''
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
          echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
      - name: Formatting
        run: cargo fmt --check
      - name: Lint
        run: cargo clippy --release --all-targets --locked -- -D warnings
      - name: Tests, on the release build that is packed
        run: cargo test --release --locked
      - name: What the binary asks of the system
        run: node scripts/check-floor.mjs
      - name: The launcher's tests and the scripts' tests
        run: node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
      - name: Pack this target's platform package
        run: node scripts/pack.mjs platform
      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
        with:
          name: pkg-main
          path: dist
      - name: Install the packed files and probe them
        run: node scripts/packed-install.mjs
      - name: Try the publishing, without publishing
        run: node scripts/dry-run-publish.mjs
      - uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          name: pkg-${{ matrix.target }}
          path: dist/dabay6-yak-analyzer-${{ matrix.target }}-*.tgz
          if-no-files-found: error

  # The publishing job's own steps, on every pull request and push, without publishing (spec PUB-17): the six
  # packed files gathered as that job gathers them, and the dry run over all six in the order they are published.
  rehearse:
    name: Rehearse the publishing
    needs: [pack-main, target]
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
        with:
          pattern: pkg-*
          path: dist
          merge-multiple: true
      - run: node scripts/dry-run-publish.mjs --all

  # From here on the jobs run only on a push to main (spec CI-1).

  # Keeps the release pull request up to date, and makes the tag and the GitHub release when it is merged. It
  # uses a token of the owner's, kept in an environment only main can use, so that the release pull request gets
  # a CI run of its own (spec PUB-1a).
  release-please:
    name: The release pull request and the tag
    if: github.event_name == 'push'
    runs-on: ubuntu-24.04
    environment: release
    steps:
      - uses: googleapis/release-please-action@45996ed1f6d02564a971a2fa1b5860e934307cf7 # v5.0.0
        with:
          token: ${{ secrets.RELEASE_PLEASE_TOKEN }}

  # Whether this commit is a release with something left to publish, from facts that are still true when a job
  # is run again (spec PUB-2a).
  release-facts:
    name: Is there something to publish
    if: github.event_name == 'push'
    needs: release-please
    runs-on: ubuntu-24.04
    outputs:
      tagged: ${{ steps.facts.outputs.tagged }}
      publish: ${{ steps.facts.outputs.publish }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
      - id: facts
        run: node scripts/release-facts.mjs

  # Publishes the very files the target jobs packed and tested. It waits for the owner's approval, in an
  # environment only main can use (spec PUB-2, PROT-2). It hands npm both ways to publish: its right to an
  # identity token, and the token's secret when the environment has one (spec PUB-13).
  publish:
    name: Publish to npm
    if: needs.release-facts.outputs.publish == 'true'
    needs: [release-facts, pack-main, target, rehearse]
    runs-on: ubuntu-24.04
    environment: npm
    permissions:
      contents: read
      id-token: write
    outputs:
      tag: ${{ steps.publish.outputs.tag }}
      line: ${{ steps.publish.outputs.line }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
          registry-url: https://registry.npmjs.org
          scope: '@dabay6'
      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
        with:
          pattern: pkg-*
          path: dist
          merge-multiple: true
      - id: publish
        run: node scripts/dry-run-publish.mjs --all --publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
          YAK_ANALYZER_PUBLISHING_JOB: 'true'

  # Adds one line to the release's notes: the packages that run published, and whether a token was stored for
  # it (spec PUB-13).
  release-notes:
    name: Say how it was published
    if: needs.publish.outputs.line != ''
    needs: publish
    runs-on: ubuntu-24.04
    permissions:
      contents: write
    steps:
      - run: |
          gh release view "$TAG" --json body --jq .body > notes.md
          printf '\n%s\n' "$LINE" >> notes.md
          gh release edit "$TAG" --notes-file notes.md
        env:
          GH_TOKEN: ${{ github.token }}
          GH_REPO: ${{ github.repository }}
          TAG: ${{ needs.publish.outputs.tag }}
          LINE: ${{ needs.publish.outputs.line }}

  # Installs the published packages by name from npmjs.com on every target, as a user's npm will, and reads
  # back their provenance (spec PUB-9, PUB-15). It runs whenever the tag is on this commit, so running it again
  # checks again.
  check-published:
    name: Published ${{ matrix.target }}
    if: always() && needs.release-facts.outputs.tagged == 'true' && (needs.publish.result == 'success' || needs.publish.result == 'skipped')
    needs: [release-facts, publish]
    strategy:
      fail-fast: false
      matrix:
        include:
          - { target: win32-x64, runner: windows-2025 }
          - { target: darwin-arm64, runner: macos-15 }
          - { target: darwin-x64, runner: macos-15-intel }
          - { target: linux-x64, runner: ubuntu-24.04 }
          - { target: linux-arm64, runner: ubuntu-24.04-arm }
    runs-on: ${{ matrix.runner }}
    defaults:
      run:
        shell: bash
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: 22.18.0
          package-manager-cache: false
      - run: node scripts/check-published.mjs
```

```json path=.github/rulesets/main.json
{
  "name": "required checks on main",
  "target": "branch",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "include": ["~DEFAULT_BRANCH"],
      "exclude": []
    }
  },
  "rules": [
    {
      "type": "required_status_checks",
      "parameters": {
        "strict_required_status_checks_policy": false,
        "do_not_enforce_on_create": false,
        "required_status_checks": [
          { "context": "Pack the main package", "integration_id": 15368 },
          { "context": "win32-x64", "integration_id": 15368 },
          { "context": "darwin-arm64", "integration_id": 15368 },
          { "context": "darwin-x64", "integration_id": 15368 },
          { "context": "linux-x64", "integration_id": 15368 },
          { "context": "linux-arm64", "integration_id": 15368 },
          { "context": "Rehearse the publishing", "integration_id": 15368 }
        ]
      }
    }
  ]
}
```

```json path=.github/rulesets/release-tags.json
{
  "name": "release tags",
  "target": "tag",
  "enforcement": "active",
  "bypass_actors": [
    { "actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "always" }
  ],
  "conditions": {
    "ref_name": {
      "include": ["refs/tags/v*"],
      "exclude": []
    }
  },
  "rules": [
    { "type": "creation" },
    { "type": "update" },
    { "type": "deletion" },
    { "type": "non_fast_forward" }
  ]
}
```

- [ ] **Step 4: Run the test, commit, push, and watch the pull request's run**

```bash
node --test scripts/workflow.test.mjs
git add .github scripts && git commit -m "ci: add the release jobs, the rehearsal and the checks of what was published"
git push
gh pr checks --watch
```

Expected: 8 tests pass. On the pull request seven checks pass: `Pack the main package`, `win32-x64`, `darwin-arm64`, `darwin-x64`, `linux-x64`,
`linux-arm64` and `Rehearse the publishing`. The rehearsal's log shows the dry run passing for the five platform
packages and then for the main package. The jobs that run only on `main` are skipped. A failure is a fault to find
and fix on the branch, recorded as an amendment with its reason, and the run is watched again.

### Task 5: the settings on GitHub

**Implements:** PROT-1, PROT-1a, PROT-2, PROT-3, PUB-1a

**Check:** on GitHub, the repository has the environments `npm` (the owner's approval, `main` only) and `release`
(`main` only, holding `RELEASE_PLEASE_TOKEN`), and two rulesets of its own: the seven required checks on `main`,
and the rule on tags named `v*`. Each step below reads its settings back.

**Files:** none. The two ruleset bodies were committed in Task 4.

**Interfaces:**
- Consumes: the check names of Task 4's run; `.github/rulesets/main.json` and `.github/rulesets/release-tags.json`.
- Produces: the environments `npm` and `release`; two rulesets; the secret `RELEASE_PLEASE_TOKEN`.

This task has no test command: each setting is read back from GitHub in the step that makes it (PROT-3). The
session makes the environments and the rulesets with the owner's rights; the owner makes the token.

- [ ] **Step 1: Make the two environments, and read them back**

The publishing environment asks for the owner's approval and can be used only by `main`. The release environment
can be used only by `main` and asks for no approval.

```bash
REVIEWER=$(gh api user --jq .id)
gh api -X PUT repos/dabay6/yak-analyzer/environments/npm --input - <<JSON
{
  "prevent_self_review": false,
  "reviewers": [{ "type": "User", "id": $REVIEWER }],
  "deployment_branch_policy": { "protected_branches": false, "custom_branch_policies": true }
}
JSON
gh api -X POST repos/dabay6/yak-analyzer/environments/npm/deployment-branch-policies -f name=main -f type=branch
gh api -X PUT repos/dabay6/yak-analyzer/environments/release --input - <<JSON
{
  "deployment_branch_policy": { "protected_branches": false, "custom_branch_policies": true }
}
JSON
gh api -X POST repos/dabay6/yak-analyzer/environments/release/deployment-branch-policies -f name=main -f type=branch
gh api repos/dabay6/yak-analyzer/environments/npm --jq '{name, can_admins_bypass, protection_rules, deployment_branch_policy}'
gh api repos/dabay6/yak-analyzer/environments/npm/deployment-branch-policies --jq '.branch_policies[].name'
gh api repos/dabay6/yak-analyzer/environments/release --jq '{name, protection_rules, deployment_branch_policy}'
gh api repos/dabay6/yak-analyzer/environments/release/deployment-branch-policies --jq '.branch_policies[].name'
```

Expected: `npm` has a `required_reviewers` rule naming the owner and a `branch_policy` rule, and its one branch
is `main`; `release` has a `branch_policy` rule only, and its one branch is `main`. If GitHub says
administrators can bypass the `npm` environment's rules, that is written in a ruling: the owner is the
administrator and the reviewer, so it changes nothing today.

- [ ] **Step 2: Compare the required checks' names with the run, send the two rulesets, and read them back**

```bash
gh pr checks --json name --jq '.[].name' | sort
node -p "JSON.parse(require('node:fs').readFileSync('.github/rulesets/main.json', 'utf8')).rules[0].parameters.required_status_checks.map((check) => check.context).sort().join('\n')"
gh api -X POST repos/dabay6/yak-analyzer/rulesets --input .github/rulesets/main.json --jq '{id, name, enforcement}'
gh api -X POST repos/dabay6/yak-analyzer/rulesets --input .github/rulesets/release-tags.json --jq '{id, name, enforcement}'
gh api repos/dabay6/yak-analyzer/rules/branches/main --jq '.[] | "\(.type) from \(.ruleset_source_type)"'
gh api repos/dabay6/yak-analyzer/rulesets --jq '.[] | "\(.name): \(.target), \(.enforcement)"'
```

Expected: the first two lists are the same seven names. Both rulesets come back `active`. The rules on `main` are
`deletion`, `non_fast_forward` and `pull_request` from the organization and `required_status_checks` from the
repository. If the two lists differ, the ruleset is not sent: the file is corrected first, as an amendment.

- [ ] **Step 3: The owner makes the token that opens the release pull request**

The owner makes a fine-grained personal access token on GitHub: resource owner `dabay6`, this repository only,
repository permissions "Contents: Read and write" and "Pull requests: Read and write", and an expiry the owner
chooses. The owner stores it, typing it into the prompt this command gives:

```bash
gh secret set RELEASE_PLEASE_TOKEN --env release --repo dabay6/yak-analyzer
gh secret list --env release --repo dabay6/yak-analyzer
```

Expected: the list shows `RELEASE_PLEASE_TOKEN`. The token is never pasted into a session. It expires on the day
the owner chose; a release after that day needs a new one, and the release-please job says so by failing.

### Task 6: how to release, in the contributing guide

**Implements:** DOC-1, PUB-3, PUB-5, PUB-12, PUB-14

**Test:** `node --test scripts/contributing.test.mjs`

**Files:**
- Modify: `CONTRIBUTING.md`
- Test: `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: the jobs of Task 4 and the settings of Task 5, by name.
- Produces: the steps for a release, and for a release that fails.

- [ ] **Step 1: Write the failing test**

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

test("the gates it names run the scripts' tests as well as the launcher's (spec GATE-2)", () => {
  assert.ok(guide.includes('node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"'));
});
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test scripts/contributing.test.mjs
```

Expected: both tests fail, because the guide has no section on releasing yet.

- [ ] **Step 3: Write the section**

```markdown edit path=CONTRIBUTING.md
<<<<<<< find
node --test "npm/**/*.test.mjs"
=======
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
>>>>>>> replace
<<<<<<< find
## Commits and pull requests
=======
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
>>>>>>> replace
```

- [ ] **Step 4: Run the test, commit and push**

```bash
node --test scripts/contributing.test.mjs
git add CONTRIBUTING.md scripts && git commit -m "docs: say how a release is made, and what to do when one fails"
git push
gh pr checks --watch
```

Expected: 2 tests pass, the commit is made and pushed, and the seven checks pass again on it.
