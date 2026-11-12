# The skeleton, part 3: the first release and the install check: implementation plan

**Goal:** Version 0.1.0 is on npmjs.com for five targets, yak has installed it and read its version on the
owner's machine, nothing is left that can publish, and the README says how to install it.

**Architecture:** This plan is carried out after the pull request of the first two plans is merged, since the
release tooling works on `main` (ORDER-1). Four of its five tasks change no file: they are acts on npmjs.com, on
GitHub and on the owner's machine, each read back and recorded. The owner does what only a person can (a token,
two merges, an approval, the settings of the packages); the session watches the runs, reads the results and
records them. The last task is the README, on a branch and a pull request of its own.

**Spec:** docs/specs/2026-11-11-skeleton-and-release-pipeline-design.md

**Implements:** NAME-2, NAME-2a, PUB-6, PUB-8, PUB-11, CHK-1, CHK-2, CHK-3, CHK-4, ORDER-1, DOC-1

This is the third of three plans. It starts only when the first two are merged and the secret
`RELEASE_PLEASE_TOKEN` is stored (the second plan's Task 5). The work item closes when CHK-4's result is on its
record and Task 5's pull request is merged.

## Global constraints

- Every release made inside this item publishes with a token, made with write access to the `@dabay6` scope and
  with "Bypass 2FA" chosen, living for days, and kept as a secret of the `npm` environment (PUB-6, PUB-11).
- The first version is 0.1.0. If its release fails for a fault in the code, the first version published is the
  next one (PUB-1, PUB-3).
- The first release is the only one this item needs (PUB-8).
- The line that maps `@dabay6` to GitHub Packages is removed from the user's npm settings on the machine the
  install check runs on, with npm's own command, after the owner has been told what on that machine leans on it
  and has said to go ahead (NAME-2, NAME-2a).
- The install check passes when the install succeeds, the platform package for the machine is in the install
  folder, and every installed package came from npmjs.com (CHK-3). Its result is written on the work item's
  record, and no later release is made before it is (CHK-4).
- Once the install check has passed, the owner deletes the token, on GitHub and on npmjs.com, and sets each of
  the six packages to refuse access tokens. The item ends with nothing that can publish, and that is read back
  (PUB-11).
- The token is never pasted into a session. Committed text names no machine, and no private repository other
  than yakshaver.

## The decisions this plan makes

1. **The token lives for seven days.** That is long enough for a release that has to be made twice, and short
   enough to be gone soon if a step is forgotten.
2. **The mapping line comes out before the release, not after.** It is needed only for the install check, but
   taking it out first means a problem with it is found while nothing is published yet.
3. **The owner merges the release pull request and approves the publishing job; the session watches.** The
   session never merges and never approves.
4. **The pack for the install check is named `yak-analyzer-install-check`** and its tool server is named
   `yak-analyzer`, so that the install lands in `tools/yak-analyzer/0.1.0/` under yak's home, the folder CHK-3
   reads.
5. **The result of the install check is recorded with a ruling on this plan,** which is how yak keeps a decision
   on a work item's record, and the registry of each package is in its text.
6. **Refusing tokens is set on npmjs.com's website,** on each package's settings page, under "Publishing
   access": "Require two-factor authentication and disallow tokens". The owner does it; no command in this plan
   changes a package's settings.
7. **The README is changed last, on its own branch from `main`,** after the release exists, so that what it
   says is true when it is merged. Its commit is a `docs` commit, which release-please does not release.

## Review focus

- **This is the first real use of everything the second plan built on `main`:** release-please's settings, the
  release pull request's own CI run, the tag rule, the job that says there is something to publish, the publish
  with a token and provenance, the line in the release's notes, and the five jobs that install what was
  published. The second plan's review focus lists what was only rehearsed. Each has its failure named in
  `CONTRIBUTING.md` under "When a release fails"; a failure is met as that section says, and recorded.
- **npm falls back from trusted publishing to the token.** No package has a trusted publisher in this item, so
  every publish here takes the fallback. If npm does not fall back, nothing is published and the job fails with
  npm's message; that is a fault in the job's steps, and PUB-5 applies.
- **The release pull request is opened by the owner's token.** If its seven checks do not start, the token is
  missing, expired, or lacks a right; the release-please job's log says which.
- **A project on the machine that leans on the mapping line.** Task 1 looks for them before the line is removed.
- **The install check leaves an install in yak's tools folder.** Task 3 removes it, or a repeated check would
  skip the install.

### Task 1: take the mapping line out of the user's npm settings

**Implements:** NAME-2, NAME-2a

**Check:** on the machine the install check will run on, `npm config get @dabay6:registry`, asked from yak's
tools folder, no longer answers GitHub Packages, and the owner was told beforehand what on the machine leans on
the line.

**Files:** none.

**Interfaces:**
- Consumes: nothing.
- Produces: a machine on which yak's install of a `@dabay6` package asks npmjs.com.

- [ ] **Step 1: Find what leans on the line**

```bash
yak project list
npm ls --global --depth=0
ls "$HOME/.yak/tools"
```

For each checkout `yak project list` names, and any folder the owner adds: whether a `package.json` in it takes
a `@dabay6` package, and whether an `.npmrc` in it maps `@dabay6` itself.

Expected: a list for the owner of the projects that take a private `@dabay6` package and have no mapping of
their own, the globally installed packages that came from the registry under that scope, and the tools yak
installed from npm under it. On 2026-11-11 all three lists were empty on the owner's machine. A project that
needs a mapping gets it in its own repository, its own way; this plan does not make that change.

- [ ] **Step 2: With the owner's yes, remove the line and read it back**

```bash
npm config delete @dabay6:registry --location=user
cd "$HOME/.yak/tools" && npm config get @dabay6:registry
```

Expected: the second command prints `undefined`. The line that holds the GitHub Packages token is left alone;
projects still use it through their own mapping.

### Task 2: the first release

**Implements:** PUB-6, PUB-8, PUB-11, ORDER-1

**Check:** `@dabay6/yak-analyzer@0.1.0` and its five platform packages are on npmjs.com with provenance, the
release `v0.1.0` on GitHub has the line saying which packages were published and that a token was stored, and
the five jobs that install the published packages passed.

**Files:** none.

**Interfaces:**
- Consumes: the workflow, the settings and the secret `RELEASE_PLEASE_TOKEN` of the second plan; `main` with
  the first two plans merged.
- Produces: version 0.1.0 on npmjs.com; the tag and the release `v0.1.0`.

- [ ] **Step 1: The owner makes the token and stores it**

On npmjs.com the owner makes a granular access token: read and write for the scope `@dabay6`, "Bypass 2FA"
chosen, expiring in seven days. The owner stores it, typing it into the prompt this command gives:

```bash
gh secret set NPM_TOKEN --env npm --repo dabay6/yak-analyzer
gh secret list --env npm --repo dabay6/yak-analyzer
```

Expected: the list shows `NPM_TOKEN`.

- [ ] **Step 2: Read the release pull request**

release-please opened it on the push that merged the first two plans.

```bash
gh pr list --repo dabay6/yak-analyzer --label "autorelease: pending"
gh pr diff <its number> --repo dabay6/yak-analyzer --name-only
gh pr checks <its number> --repo dabay6/yak-analyzer
```

Expected: one pull request, titled `chore(main): release 0.1.0`. It changes `.release-please-manifest.json`,
`CHANGELOG.md`, `Cargo.toml`, `Cargo.lock` and the six `package.json` files, and nothing else; every version in
them, and the five optional dependencies, is 0.1.0. Its seven checks pass. If there is no such pull request, or
its checks did not start, the log of the job "The release pull request and the tag" on `main` says why.

- [ ] **Step 3: The owner merges the release pull request, then approves the publishing job**

After the merge, the session watches the run on `main`:

```bash
gh run list --repo dabay6/yak-analyzer --branch main --limit 1
gh run watch <its id> --repo dabay6/yak-analyzer
```

Expected, in order: "The release pull request and the tag" makes the tag `v0.1.0` and the release; the five
target jobs and the rehearsal pass; "Is there something to publish" says `v0.1.0 is on this commit; a package is
still to be published`; "Publish to npm" waits. The owner approves it. It then publishes the five platform
packages and the main package, in that order; "Say how it was published" adds its line to the release; the five
"Published" jobs each install `@dabay6/yak-analyzer@0.1.0` from npmjs.com, get `yak-analyzer 0.1.0` from it, and
find provenance on the main package and their own platform package.

- [ ] **Step 4: Read the release back**

```bash
npm view @dabay6/yak-analyzer@0.1.0 version dist.attestations.provenance.predicateType --@dabay6:registry=https://registry.npmjs.org/
gh release view v0.1.0 --repo dabay6/yak-analyzer --json body --jq .body
```

Expected: version `0.1.0` with a provenance type that begins `https://slsa.dev/provenance/`, and notes whose
last line begins `Published to npm by one run of the job:`, names the six packages, and says a token was stored.

### Task 3: the install check through yak

**Implements:** CHK-1, CHK-2, CHK-3, CHK-4

**Check:** yak installed `@dabay6/yak-analyzer@0.1.0` as a tool server on the owner's machine, its probe printed
the version, the platform package is in the install folder, every package there came from npmjs.com, and the
result is on the work item's record.

**Files:** none in this repository. Two folders are made outside every repository, and removed.

**Interfaces:**
- Consumes: version 0.1.0 on npmjs.com (Task 2); the machine of Task 1.
- Produces: the install check's result, on the record.

- [ ] **Step 1: Make the pack and the project**

A folder for the pack, holding two files. `pack.json`:

```json
{
  "name": "yak-analyzer-install-check",
  "sdk": 3,
  "version": "0.0.0",
  "description": "Made for one install check and removed after it."
}
```

`tools/yak-analyzer.md`:

```markdown
---
summary: The released yak-analyzer, for the install check
install:
  kind: npm
  package: "@dabay6/yak-analyzer"
  version: 0.1.0
probe: [yak-analyzer, --version]
serve: [yak-analyzer]
---
Made for one install check and removed after it.
```

A second folder for the project, holding one file, `.npmrc`, so that the check shows a project's mapping does
not reach yak's install:

```ini
@dabay6:registry=https://npm.pkg.github.com
```

The pack's shape is checked first against the yak in use (`sdk` is the pack format that yak reads). Then, in the
project's folder:

```bash
yak pack add --project path:<the pack's folder>
yak tools list
```

Expected: `yak tools list` shows `yak-analyzer` at 0.1.0, not installed. No agent program is pointed at the
project, so nothing starts the stand-in `serve` line.

- [ ] **Step 2: The owner runs the install**

In the project's folder, at a terminal:

```bash
yak tools install
```

Expected: yak asks the owner to confirm, installs, runs the probe, and says the tool is installed.

- [ ] **Step 3: Read the install back**

```bash
node -e "const lock = require(require('node:path').join(require('node:os').homedir(), '.yak', 'tools', 'yak-analyzer', '0.1.0', 'node_modules', '.package-lock.json')); for (const [name, entry] of Object.entries(lock.packages)) console.log(name, entry.resolved)"
```

Expected: two lines, the main package and this machine's platform package, each with an address that begins
`https://registry.npmjs.org/`. Anything else is a failed check: the work stops, the result is recorded as
failed, and the owner decides what to change (CHK-4).

- [ ] **Step 4: Record the result, and remove what the check made**

The result is recorded with a ruling on this plan: that the install succeeded and the probe printed 0.1.0, and
the address each of the two packages came from. Then the pack is taken out of the project, and the pack's
folder, the project's folder and the install folder `tools/yak-analyzer/0.1.0/` under yak's home are removed.

### Task 4: nothing is left that can publish

**Implements:** PUB-11

**Check:** the `npm` environment holds no npm token, npmjs.com lists no token that can write to `@dabay6`, and
each of the six packages refuses access tokens.

**Files:** none.

**Interfaces:**
- Consumes: Task 3's passed check.
- Produces: the item's end state.

- [ ] **Step 1: The owner deletes the token and closes the packages to tokens**

```bash
gh secret delete NPM_TOKEN --env npm --repo dabay6/yak-analyzer
```

On npmjs.com the owner revokes the token, and on each of the six packages' settings pages sets "Publishing
access" to "Require two-factor authentication and disallow tokens".

- [ ] **Step 2: Read it back**

```bash
gh secret list --env npm --repo dabay6/yak-analyzer
```

Expected: the list is empty. The owner reads on npmjs.com that no token is listed that can write to `@dabay6`
and that each of the six packages shows the setting, and says so; that is recorded with a ruling.

### Task 5: the README says how to install it

**Implements:** DOC-1, NAME-2a

**Test:** `node --test scripts/readme.test.mjs`

**Files:**
- Modify: `README.md`
- Test: `scripts/readme.test.mjs`

**Interfaces:**
- Consumes: the released main package's name; `scripts/lib/run.mjs` (`MAIN_DIR`, `manifest`, `ROOT`).
- Produces: the README, true after the release.

This task is worked on a branch of its own, made from `main` once Task 4 is done, and reaches `main` by a pull
request of its own.

- [ ] **Step 1: Write the failing test**

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

test('its status says what the release is: a binary that answers its version and analyzes nothing yet', () => {
  assert.ok(readme.includes('answers its version and analyzes nothing yet'));
  assert.ok(!readme.includes('there is no\nrelease'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
  assert.ok(readme.includes('npm config delete @dabay6:registry --location=user\n'));
});
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test scripts/readme.test.mjs
```

Expected: the three tests fail, because the README has no section on installing and still says there is no
release.

- [ ] **Step 3: Write the README's status and its section on installing**

````markdown edit path=README.md
<<<<<<< find
The walking skeleton is in: a binary that answers its version, built and tested on five targets,
with its npm packages put together and installed in CI. It analyzes nothing yet, and there is no
release.
=======
The first release is out. It is a walking skeleton: a binary that
answers its version and analyzes nothing yet, built and tested on five targets and published to
npm with provenance.

## Install

yakshaver will install the analyzer by itself, at one pinned version, once its content pack
declares it. To try the released binary by hand:

```bash
npm install @dabay6/yak-analyzer
npx yak-analyzer --version
```

It needs Node 22.18 or later, and there is a binary for Windows on x64, macOS on arm64 and x64,
and Linux with glibc on x64 and arm64. No package runs a script when it is installed.

If your own npm settings map the `@dabay6` scope to another registry, npm looks for the package
there and does not find it. Remove that setting first:

```bash
npm config delete @dabay6:registry --location=user
```
>>>>>>> replace
````

- [ ] **Step 4: Run the test, commit, push, and open the pull request**

```bash
node --test scripts/readme.test.mjs
git add README.md scripts && git commit -m "docs: say how to install the released binary"
git push -u origin docs/first-release
gh pr create --base main --head docs/first-release --title "docs: say how to install the released binary" --body "The README's status and install section, after the first release."
gh pr checks --watch
```

Expected: 3 tests pass, and the pull request's seven checks pass. The owner merges it. Its commit is a `docs`
commit, so release-please opens no release pull request for it.
