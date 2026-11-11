# The skeleton, part 1: the binary, the npm packages and CI on five targets: implementation plan

**Goal:** A Rust binary that answers its version, a JavaScript launcher and six npm packages around it, and a CI
run that builds, tests, packs and installs them on all five targets, with nothing published.

**Architecture:** The crate at the repository's root builds `yak-analyzer`. `npm/yak-analyzer` is the main
package, whose launcher finds the platform package's binary and runs it; `npm/platforms/<target>` holds one
manifest per target, and the binary is copied in when the package is packed. Three small Node scripts in
`scripts/` pack the packages, install the packed files offline and probe them, and check what each binary needs
from the system; CI runs the same scripts a person runs locally.

**Spec:** docs/specs/2026-11-11-skeleton-and-release-pipeline-design.md

**Implements:** NAME-1, BIN-1 to BIN-4, GATE-1 to GATE-4, CI-1, CI-1a, CI-2 to CI-4, PKG-1, PKG-2, PKG-2a, PKG-3 to PKG-10, PUB-10, DOC-1, DOC-2

This is the first of two plans. It covers what ORDER-1 says can be shown before the item's pull request merges.
The second plan covers the release: release-please, the publishing job, the job that checks the published
packages, the rules on `main` and on the tags, the publishing environment, the pin pull requests, the first
release and the install check (PUB-1 to PUB-9, PROT-1 to PROT-3, NAME-2, NAME-2a, CHK-1 to CHK-4, and the parts of
CI-1, CI-1a and DOC-1 that belong to a release).

## Global constraints

- The packages are `@dabay6/yak-analyzer` and `@dabay6/yak-analyzer-` followed by `win32-x64`, `darwin-arm64`,
  `darwin-x64`, `linux-x64` and `linux-arm64`. Nothing is done before the owner has confirmed holding the
  `@dabay6` scope on npmjs.com (NAME-1).
- The binary prints `yak-analyzer <version>` for its version option and exits 0; given anything else, or nothing,
  it writes one line on standard error, nothing on standard output, and exits non-zero (BIN-1, BIN-2).
- One exact Rust version in `rust-toolchain.toml`, `Cargo.lock` committed, every dependency pinned with `=`
  (GATE-1). The skeleton has no dependency at all (GATE-4).
- The gates are `cargo fmt --check`, `cargo clippy` with warnings as errors, `cargo test` on a release build, and
  the launcher's tests on Node's own test runner (GATE-2).
- No package has an install script. The launcher is one plain script that runs on Node 22.18 and answers no
  command itself (PKG-4, PKG-5).
- Every action is pinned to a full commit hash with its version beside it; a pull request's jobs have a read-only
  token and no secret; no `pull_request_target` (CI-2, CI-3, CI-4).
- The packed install fetches nothing and runs no install script (PKG-9).
- A binary asks for nothing newer than the Node yak needs: glibc 2.28 on Linux, macOS 11.0, and no separately
  installed C runtime on Windows; every target's job checks it (PKG-2a).
- Committed text names no machine, and no private repository other than yakshaver.

## The decisions this plan makes

1. **The version option is `--version`.** The binary parses its arguments by hand; two answers do not need a
   library (GATE-4).
2. **The exit code for a refusal is 2,** the usual code for wrong usage. The launcher's own failures exit 1.
3. **The Rust version is 1.99.1,** the stable release when this was written, with edition 2024.
4. **The integration test is `tests/cli.test.rs`,** declared by name in `Cargo.toml`, because yak's plan format
   takes a task's test files from names like `*.test.*` and Cargo cannot derive a test's name from a file name
   with a second dot.
5. **The tree's version is 0.0.0** until release-please's first release pull request writes 0.1.0 (the second
   plan). The version test of GATE-3 and the launcher's tests are JavaScript, since they read JSON.
6. **The launcher is two files:** `bin/yak-analyzer.js` wires the real machine in, and `lib/launch.js` holds the
   logic as functions the tests call with a made-up platform. The package reaches its own `lib` through the
   subpath import `#yak-analyzer/*`, never `../`.
7. **A missing binary's two cases, and a third,** are told apart as PKG-6 says. The C library is read from
   Node's own report, without starting a process.
8. **Linux builds run inside the `manylinux_2_28` images** (AlmaLinux 8, glibc 2.28) on GitHub's own x64 and
   arm64 runners. That is what holds the build to the floor; the check reads the highest glibc version the built
   binary asks for. The images are named by tag, not by digest: nothing in this plan moves a digest.
9. **macOS builds set `MACOSX_DEPLOYMENT_TARGET` to 11.0,** and Windows builds link the C runtime statically
   through `.cargo/config.toml`.
10. **The runners** are `windows-2025`, `macos-15`, `macos-15-intel`, `ubuntu-24.04` and `ubuntu-24.04-arm`.
    `ubuntu-latest` is moving to a new image this month, so it is not used. GitHub has said `macos-15-intel`
    stays until August 2027.
11. **The dry run uses npm 11.21.2,** fetched by `npx` at that exact version. Trusted publishing needs npm 11.5.1
    or later, and npm 12 does not run on Node 22.18. The script names npmjs.com as the registry for `@dabay6` on
    its command, so it gives the same answer on a machine whose own settings map the scope elsewhere.
12. **Each package gets a copy of `LICENSE` when it is packed.** The copies and the copied binaries are ignored
    by git.
13. **`Cargo.toml` has no field for where to report a bug;** it carries the description, keywords, repository,
    home page and license (PKG-10).
14. **CI is checked by running it.** Task 5 pushes the branch and opens a draft pull request, because nothing
    else can show that the workflow works. The pull request stays a draft until the owner has validated the plan.

## Review focus

- **`container: ${{ matrix.image }}` with an empty image** for the three targets that use no container. GitHub
  is expected to run those jobs on the runner itself; this was not run before the plan was written. Task 5's
  run is the test, and the fallback is two job definitions.
- **The arm64 `manylinux_2_28` image on `ubuntu-24.04-arm`,** and `actions/setup-node` inside both images. Read
  from their documentation, not run. Task 5's run is the test.
- **`otool -l` on both macOS targets** giving `minos 11.0` under `LC_BUILD_VERSION`. Not run before the plan was
  written. Task 5's run is the test.
- **An argument that holds a line break** makes the binary's refusal more than one line. No test covers it;
  nothing yak passes can hold one.
- **rustfmt's own layout.** The Rust in this plan was written to rustfmt's style by hand, with no toolchain on
  the machine it was written on. Task 2 runs `cargo fmt --check`; a difference is fixed there and recorded.
- **`Cargo.lock`'s format version.** Cargo writes the file; Task 2 builds without `--locked` first and commits
  what Cargo wrote.
- **A launcher started by yak** is run as `node <bin file>`, which is how the packed install runs it too.

### Task 1: the owner holds the scope, and the machine has the toolchain

**Implements:** NAME-1, GATE-1

**Files:** none.

**Interfaces:**
- Consumes: nothing.
- Produces: the owner's confirmation, on the work item's record; `cargo` on this machine.

- [ ] **Step 1: The owner confirms holding `@dabay6` on npmjs.com**

The owner runs these at a terminal and says what they print:

```bash
npm login --registry=https://registry.npmjs.org/
npm whoami --registry=https://registry.npmjs.org/
npm org ls dabay6 --registry=https://registry.npmjs.org/
```

Expected: `npm whoami` prints the owner's npm user. Either that user is `dabay6`, or `npm org ls dabay6` lists
that user as a member of the organization `dabay6`. If neither is so, stop: the work does not go on, and the
name comes back to the owner (NAME-1). The answer is recorded with a ruling.

- [ ] **Step 2: Install the Rust toolchain on this machine**

With the tool manager the machine already uses for Node:

```bash
mise use -g rust@1.99.1
cargo --version
```

Expected: `cargo 1.99.1` and its build date.

### Task 2: the binary

**Implements:** BIN-1, BIN-2, BIN-3, BIN-4, GATE-1, GATE-2, GATE-3, GATE-4, PKG-2a, PKG-10

**Test:** `cargo test --release --test cli`

**Files:**
- Create: `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, `src/main.rs`
- Test: `tests/cli.test.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: the binary `yak-analyzer` at `target/release/`; the version option `--version`; exit code 2 for a
  refusal.

- [ ] **Step 1: Write the failing test**

```rust path=tests/cli.test.rs
//! Starts the built binary and checks what it answers (spec BIN-1, BIN-2).

use std::process::{Command, Output};

/// Runs the built binary with the arguments given.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .output()
        .expect("the binary starts")
}

#[test]
fn prints_its_version_on_standard_output_and_exits_zero() {
    let output = run(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("yak-analyzer {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    let cases: [&[&str]; 3] = [&["analyze"], &["--version", "extra"], &["--help"]];

    for args in cases {
        let output = run(args);
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(error.lines().count(), 1, "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(error.contains("--version"), "{args:?}");
    }
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version"));
}
```

- [ ] **Step 2: Run it and see it fail**

```bash
cargo test --release --test cli
```

Expected: the run fails, because there is no `Cargo.toml` yet.

- [ ] **Step 3: Write the crate**

```toml path=rust-toolchain.toml
[toolchain]
channel = "1.99.1"
components = ["rustfmt", "clippy"]
```

```toml path=Cargo.toml
[package]
name = "yak-analyzer"
version = "0.0.0"
edition = "2024"
description = "A static analyzer for TypeScript and JavaScript, built for yakshaver"
license = "MIT"
repository = "https://github.com/dabay6/yak-analyzer"
homepage = "https://github.com/dabay6/yak-analyzer#readme"
keywords = ["static-analysis", "dead-code", "typescript", "javascript", "sarif"]
categories = ["command-line-utilities", "development-tools"]
publish = false
autotests = false

[[bin]]
name = "yak-analyzer"
path = "src/main.rs"

[[test]]
name = "cli"
path = "tests/cli.test.rs"

[dependencies]
```

```toml path=Cargo.lock
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "yak-analyzer"
version = "0.0.0"
```

```toml path=.cargo/config.toml
# The Windows binary carries its C runtime inside it, so no separate runtime has to be installed (spec PKG-2a).
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

```rust path=src/main.rs
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! This is the walking skeleton. It answers its version and analyzes nothing yet.

use std::env;
use std::process::ExitCode;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The exit code for a command the binary does not have.
const REFUSED: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match args.as_slice() {
        [only] if only == VERSION_OPTION => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        [] => {
            eprintln!("yak-analyzer: nothing to do; it answers only {VERSION_OPTION}");

            ExitCode::from(REFUSED)
        }
        _ => {
            let given = args.join(" ");

            eprintln!("yak-analyzer: cannot do `{given}`; it answers only {VERSION_OPTION}");

            ExitCode::from(REFUSED)
        }
    }
}
```

- [ ] **Step 4: Build once without the lock check, so Cargo writes its own lock file**

```bash
cargo build --release
git diff --stat -- Cargo.lock
```

Expected: the build succeeds. If Cargo rewrote `Cargo.lock`, the difference is its format version only; what
Cargo wrote is what gets committed.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: `cargo fmt --check` prints nothing; clippy finishes with no warning; 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add rust-toolchain.toml Cargo.toml Cargo.lock .cargo src tests && git commit -m "feat: add a binary that answers its version"
```

Expected: the commit is made.

### Task 3: the launcher and the six packages

**Implements:** NAME-1, PKG-1, PKG-2, PKG-3, PKG-4, PKG-5, PKG-6, PKG-7, PKG-8, PKG-10, GATE-2, GATE-3

**Test:** `node --test "npm/**/*.test.mjs"`

**Files:**
- Create: `npm/yak-analyzer/package.json`, `npm/yak-analyzer/bin/yak-analyzer.js`,
  `npm/yak-analyzer/lib/launch.js`
- Create: `npm/platforms/win32-x64/package.json`, `npm/platforms/darwin-arm64/package.json`,
  `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-x64/package.json`,
  `npm/platforms/linux-arm64/package.json`
- Test: `npm/yak-analyzer/test/launcher.test.mjs`, `npm/yak-analyzer/test/manifests.test.mjs`

**Interfaces:**
- Consumes: Task 2's `Cargo.toml` and `Cargo.lock` (their version), and the binary's file name.
- Produces: `launch(options)`, `linuxLibc()` and `TARGETS` from `#yak-analyzer/launch.js`; the package names; the
  main package's `bin` entry `yak-analyzer`.

- [ ] **Step 1: Write the failing tests**

```js path=npm/yak-analyzer/test/launcher.test.mjs
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { test } from 'node:test';

import { launch, linuxLibc, TARGETS } from '#yak-analyzer/launch.js';

/** A folder that stands for an installed platform package. */
const INSTALLED = join('made-up', 'node_modules', '@dabay6', 'yak-analyzer-linux-x64');

/**
 * Runs the launcher on a made-up machine, keeping what it writes.
 *
 * @param {object} machine - What to change from a Linux x64 machine with glibc and the package installed.
 * @returns {{ code: number, lines: string[], ran: unknown[] }} The exit code, the lines written, and each run.
 */
const onMachine = (machine = {}) => {
  const lines = [];
  const ran = [];
  const code = launch({
    arch: 'x64',
    args: ['--version'],
    libc: 'glibc',
    platform: 'linux',
    resolve: () => join(INSTALLED, 'package.json'),
    run: (file, args) => {
      ran.push({ args, file });

      return { signal: null, status: 0 };
    },
    write: (line) => lines.push(line),
    ...machine,
  });

  return { code, lines, ran };
};

test('the targets are the five the packages exist for', () => {
  assert.deepEqual(TARGETS, ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64']);
});

test('runs the platform package binary with the arguments it was given, and says nothing itself', () => {
  const { code, lines, ran } = onMachine();

  assert.equal(code, 0);
  assert.deepEqual(lines, []);
  assert.deepEqual(ran, [{ args: ['--version'], file: join(INSTALLED, 'yak-analyzer') }]);
});

test('the binary has the Windows name on Windows', () => {
  const { ran } = onMachine({ platform: 'win32' });

  assert.equal(ran[0].file, join(INSTALLED, 'yak-analyzer.exe'));
});

test('exits with the exit code of a real program that stands in for the binary', () => {
  const script = 'process.stdout.write(process.argv.slice(1).join(","));process.exit(7)';
  let output = '';
  const { code, lines } = onMachine({
    args: ['one', 'two'],
    run: (_file, args) => {
      const child = spawnSync(process.execPath, ['-e', script, '--', ...args], { encoding: 'utf8' });

      output = child.stdout;

      return child;
    },
  });

  assert.equal(code, 7);
  assert.equal(output, 'one,two');
  assert.deepEqual(lines, []);
});

test('names the targets that have a package where the machine has none', () => {
  const { code, lines, ran } = onMachine({ arch: 'x64', platform: 'freebsd' });

  assert.equal(code, 1);
  assert.deepEqual(ran, []);
  assert.equal(lines.length, 1);
  assert.match(lines[0], /^yak-analyzer: there is no package for freebsd-x64; /u);

  for (const target of TARGETS) {
    assert.ok(lines[0].includes(target), target);
  }
});

test('treats Linux with another C library as a machine with no package, and says which library', () => {
  const { code, lines, ran } = onMachine({ libc: 'musl' });

  assert.equal(code, 1);
  assert.deepEqual(ran, []);
  assert.match(lines[0], /^yak-analyzer: there is no package for linux-x64 \(musl\); /u);
  assert.ok(lines[0].includes('linux-x64 (glibc)'));
});

test('says the package is missing, and why it may be, where one exists and is not installed', () => {
  const { code, lines, ran } = onMachine({
    resolve: () => {
      throw new Error('Cannot find module');
    },
  });

  assert.equal(code, 1);
  assert.deepEqual(ran, []);
  assert.equal(lines.length, 1);
  assert.ok(lines[0].includes('@dabay6/yak-analyzer-linux-x64 is not installed'));
  assert.ok(lines[0].includes('optional dependencies'));
});

test('names the file and the reason when the binary is there and cannot be started', () => {
  const { code, lines } = onMachine({ run: (file, args) => spawnSync(file, args) });

  assert.equal(code, 1);
  assert.equal(lines.length, 1);
  assert.ok(lines[0].startsWith(`yak-analyzer: cannot start ${join(INSTALLED, 'yak-analyzer')}: `));
});

test('names the signal when the binary was ended by one', () => {
  const { code, lines } = onMachine({ run: () => ({ signal: 'SIGKILL', status: null }) });

  assert.equal(code, 1);
  assert.deepEqual(lines, [`yak-analyzer: ${join(INSTALLED, 'yak-analyzer')} was ended by SIGKILL\n`]);
});

test('every line the launcher writes is one line', () => {
  for (const machine of [{ platform: 'freebsd' }, { libc: 'musl' }, { run: () => ({ signal: 'SIGTERM' }) }]) {
    const { lines } = onMachine(machine);

    assert.equal(lines.join('').trimEnd().split('\n').length, 1);
  }
});

test('reads the C library only on Linux', () => {
  if (process.platform === 'linux') {
    assert.ok(['glibc', 'musl'].includes(linuxLibc()));
  } else {
    assert.equal(linuxLibc(), undefined);
  }
});
```

```js path=npm/yak-analyzer/test/manifests.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { TARGETS } from '#yak-analyzer/launch.js';

/** The repository's root. */
const ROOT = join(import.meta.dirname, '..', '..', '..');

/**
 * Reads a JSON file under the repository's root.
 *
 * @param {...string} path - The file's path, in parts.
 * @returns {Record<string, any>} The parsed file.
 */
const json = (...path) => JSON.parse(readFileSync(join(ROOT, ...path), 'utf8'));

/**
 * The version a Cargo file gives the crate.
 *
 * @param {string} file - `Cargo.toml` or `Cargo.lock`.
 * @returns {string | undefined} The version on the line after the crate's name.
 */
const cargoVersion = (file) =>
  /name = "yak-analyzer"\r?\nversion = "([^"]+)"/u.exec(readFileSync(join(ROOT, file), 'utf8'))?.[1];

const main = json('npm', 'yak-analyzer', 'package.json');
const platforms = TARGETS.map((target) => ({ manifest: json('npm', 'platforms', target, 'package.json'), target }));
const packages = [main, ...platforms.map(({ manifest }) => manifest)];

test('every place the version is written agrees (spec PKG-8)', () => {
  const version = cargoVersion('Cargo.toml');

  assert.match(version, /^\d+\.\d+\.\d+$/u);
  assert.equal(cargoVersion('Cargo.lock'), version);

  for (const manifest of packages) {
    assert.equal(manifest.version, version, manifest.name);
  }

  for (const [name, range] of Object.entries(main.optionalDependencies)) {
    assert.equal(range, version, name);
  }
});

test("the main package's optional dependencies are exactly the five platform packages (spec PKG-3)", () => {
  assert.deepEqual(
    Object.keys(main.optionalDependencies).sort(),
    TARGETS.map((target) => `@dabay6/yak-analyzer-${target}`).sort()
  );
});

test('each platform package is named for its target and states its system, processor and C library', () => {
  for (const { manifest, target } of platforms) {
    const [os, cpu] = target.split('-');

    assert.equal(manifest.name, `@dabay6/yak-analyzer-${target}`);
    assert.deepEqual(manifest.os, [os], target);
    assert.deepEqual(manifest.cpu, [cpu], target);
    assert.deepEqual(manifest.libc, os === 'linux' ? ['glibc'] : undefined, target);
    assert.deepEqual(manifest.files, [os === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer'], target);
  }
});

test('the main package names the launcher as its bin, and the oldest Node it runs on (spec PKG-1, PKG-5)', () => {
  assert.equal(main.name, '@dabay6/yak-analyzer');
  assert.deepEqual(main.bin, { 'yak-analyzer': 'bin/yak-analyzer.js' });
  assert.deepEqual(main.engines, { node: '>=22.18.0' });
});

test('no package has a script of any kind (spec PKG-4)', () => {
  for (const manifest of packages) {
    assert.equal(manifest.scripts, undefined, manifest.name);
  }
});

test('every package carries its metadata (spec PKG-10)', () => {
  for (const manifest of packages) {
    assert.ok(manifest.description.length > 0, manifest.name);
    assert.ok(manifest.keywords.length > 0, manifest.name);
    assert.equal(manifest.license, 'MIT', manifest.name);
    assert.equal(manifest.homepage, 'https://github.com/dabay6/yak-analyzer#readme', manifest.name);
    assert.deepEqual(manifest.bugs, { url: 'https://github.com/dabay6/yak-analyzer/issues' }, manifest.name);
    assert.equal(manifest.repository.url, 'git+https://github.com/dabay6/yak-analyzer.git', manifest.name);
    assert.deepEqual(manifest.publishConfig, { access: 'public' }, manifest.name);
  }
});
```

- [ ] **Step 2: Run them and see them fail**

```bash
node --test "npm/**/*.test.mjs"
```

Expected: both files fail, because `#yak-analyzer/launch.js` cannot be found.

- [ ] **Step 3: Write the launcher and the manifests**

```json path=npm/yak-analyzer/package.json
{
  "name": "@dabay6/yak-analyzer",
  "version": "0.0.0",
  "description": "A static analyzer for TypeScript and JavaScript, built for yakshaver",
  "keywords": ["static-analysis", "dead-code", "typescript", "javascript", "sarif", "yakshaver"],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": { "url": "https://github.com/dabay6/yak-analyzer/issues" },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/yak-analyzer"
  },
  "license": "MIT",
  "type": "module",
  "imports": { "#yak-analyzer/*": "./lib/*" },
  "bin": { "yak-analyzer": "bin/yak-analyzer.js" },
  "files": ["bin", "lib"],
  "engines": { "node": ">=22.18.0" },
  "optionalDependencies": {
    "@dabay6/yak-analyzer-darwin-arm64": "0.0.0",
    "@dabay6/yak-analyzer-darwin-x64": "0.0.0",
    "@dabay6/yak-analyzer-linux-arm64": "0.0.0",
    "@dabay6/yak-analyzer-linux-x64": "0.0.0",
    "@dabay6/yak-analyzer-win32-x64": "0.0.0"
  },
  "publishConfig": { "access": "public" }
}
```

```js path=npm/yak-analyzer/lib/launch.js
import { dirname, join } from 'node:path';

/** The targets a platform package exists for, as Node names the operating system and the processor. */
export const TARGETS = ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64'];

/** The exit code when the launcher itself cannot run the binary. */
const FAILED = 1;

/**
 * The targets as a person reads them: the Linux ones are for glibc.
 *
 * @returns {string} The five targets, in one phrase.
 */
const targetsInWords = () => {
  return TARGETS.map((target) => {
    return target.startsWith('linux') ? `${target} (glibc)` : target;
  }).join(', ');
};

/**
 * The C library of a Linux machine, read from Node's own report without starting a process. Node's report names
 * the glibc version it runs on, and names none on a machine with another C library.
 *
 * @returns {'glibc' | 'musl' | undefined} The library; undefined off Linux.
 */
export function linuxLibc() {
  if (process.platform !== 'linux') {
    return undefined;
  }

  const excluded = process.report.excludeNetwork;

  // The report's list of network interfaces is slow to build and not needed.
  process.report.excludeNetwork = true;

  const { header } = process.report.getReport();

  process.report.excludeNetwork = excluded;

  return header.glibcVersionRuntime === undefined ? 'musl' : 'glibc';
}

/**
 * Finds the platform package's binary and runs it (spec PKG-5, PKG-6). The launcher answers no command itself: it
 * hands every argument on, and writes a line of its own only when it cannot run the binary.
 *
 * @param {object} options - The machine, and how to reach it.
 * @param {string} options.arch - The processor, as Node names it.
 * @param {readonly string[]} options.args - The arguments to hand on.
 * @param {string | undefined} options.libc - The C library on Linux.
 * @param {string} options.platform - The operating system, as Node names it.
 * @param {(id: string) => string} options.resolve - Resolves a module id to its file, and throws when it cannot.
 * @param {(file: string, args: readonly string[]) => { error?: Error, signal?: string | null, status?: number | null }} options.run -
 *   Runs the binary and waits for it.
 * @param {(line: string) => void} options.write - Writes a line to standard error.
 * @returns {number} The exit code: the binary's own, or 1 when it did not run to an end of its own.
 */
export function launch({ arch, args, libc, platform, resolve, run, write }) {
  const target = `${platform}-${arch}`;
  const otherLibc = platform === 'linux' && libc !== 'glibc';

  if (!TARGETS.includes(target) || otherLibc) {
    const machine = otherLibc ? `${target} (${libc ?? 'an unknown C library'})` : target;

    write(`yak-analyzer: there is no package for ${machine}; packages exist for ${targetsInWords()}\n`);

    return FAILED;
  }

  const name = `@dabay6/yak-analyzer-${target}`;
  let manifest;

  try {
    manifest = resolve(`${name}/package.json`);
  } catch {
    write(`yak-analyzer: the package ${name} is not installed; install again without leaving out optional dependencies\n`);

    return FAILED;
  }

  const file = join(dirname(manifest), platform === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer');
  const ended = run(file, args);

  if (ended.error !== undefined) {
    write(`yak-analyzer: cannot start ${file}: ${ended.error.message}\n`);

    return FAILED;
  }

  if (typeof ended.signal === 'string') {
    write(`yak-analyzer: ${file} was ended by ${ended.signal}\n`);

    return FAILED;
  }

  return ended.status ?? FAILED;
}
```

```js path=npm/yak-analyzer/bin/yak-analyzer.js
#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

import { launch, linuxLibc } from '#yak-analyzer/launch.js';

/** Resolves a package from where this package is installed, which is where its platform package is too. */
const require = createRequire(import.meta.url);

process.exitCode = launch({
  arch: process.arch,
  args: process.argv.slice(2),
  libc: linuxLibc(),
  platform: process.platform,
  resolve: (id) => {
    return require.resolve(id);
  },
  run: (file, args) => {
    return spawnSync(file, args, { stdio: 'inherit', windowsHide: true });
  },
  write: (line) => {
    process.stderr.write(line);
  },
});
```

```json path=npm/platforms/win32-x64/package.json
{
  "name": "@dabay6/yak-analyzer-win32-x64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for Windows on x64",
  "keywords": ["yak-analyzer", "static-analysis"],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": { "url": "https://github.com/dabay6/yak-analyzer/issues" },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/win32-x64"
  },
  "license": "MIT",
  "os": ["win32"],
  "cpu": ["x64"],
  "files": ["yak-analyzer.exe"],
  "publishConfig": { "access": "public" }
}
```

```json path=npm/platforms/darwin-arm64/package.json
{
  "name": "@dabay6/yak-analyzer-darwin-arm64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for macOS on arm64",
  "keywords": ["yak-analyzer", "static-analysis"],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": { "url": "https://github.com/dabay6/yak-analyzer/issues" },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/darwin-arm64"
  },
  "license": "MIT",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "files": ["yak-analyzer"],
  "publishConfig": { "access": "public" }
}
```

```json path=npm/platforms/darwin-x64/package.json
{
  "name": "@dabay6/yak-analyzer-darwin-x64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for macOS on x64",
  "keywords": ["yak-analyzer", "static-analysis"],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": { "url": "https://github.com/dabay6/yak-analyzer/issues" },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/darwin-x64"
  },
  "license": "MIT",
  "os": ["darwin"],
  "cpu": ["x64"],
  "files": ["yak-analyzer"],
  "publishConfig": { "access": "public" }
}
```

```json path=npm/platforms/linux-x64/package.json
{
  "name": "@dabay6/yak-analyzer-linux-x64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for Linux on x64, for glibc",
  "keywords": ["yak-analyzer", "static-analysis"],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": { "url": "https://github.com/dabay6/yak-analyzer/issues" },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/linux-x64"
  },
  "license": "MIT",
  "os": ["linux"],
  "cpu": ["x64"],
  "libc": ["glibc"],
  "files": ["yak-analyzer"],
  "publishConfig": { "access": "public" }
}
```

```json path=npm/platforms/linux-arm64/package.json
{
  "name": "@dabay6/yak-analyzer-linux-arm64",
  "version": "0.0.0",
  "description": "The yak-analyzer binary for Linux on arm64, for glibc",
  "keywords": ["yak-analyzer", "static-analysis"],
  "homepage": "https://github.com/dabay6/yak-analyzer#readme",
  "bugs": { "url": "https://github.com/dabay6/yak-analyzer/issues" },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/dabay6/yak-analyzer.git",
    "directory": "npm/platforms/linux-arm64"
  },
  "license": "MIT",
  "os": ["linux"],
  "cpu": ["arm64"],
  "libc": ["glibc"],
  "files": ["yak-analyzer"],
  "publishConfig": { "access": "public" }
}
```

- [ ] **Step 4: Run them and see them pass**

```bash
node --test "npm/**/*.test.mjs"
```

Expected: 17 tests pass, none fail.

- [ ] **Step 5: Commit**

```bash
git add npm && git commit -m "feat: add the launcher and the six npm packages"
```

Expected: the commit is made.

### Task 4: pack, install the packed files, and check what the binary needs

**Implements:** PKG-9, PKG-2a, PUB-10

**Test:** `cargo build --release && node scripts/pack.mjs main && node scripts/pack.mjs platform && node scripts/packed-install.mjs && node scripts/check-floor.mjs && node scripts/dry-run-publish.mjs`

**Files:**
- Create: `scripts/lib/run.mjs`, `scripts/pack.mjs`, `scripts/packed-install.mjs`, `scripts/check-floor.mjs`,
  `scripts/dry-run-publish.mjs`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: Task 2's release build at `target/release/`; Task 3's packages and the main package's `bin` entry.
- Produces: `dist/*.tgz`, the files npm would publish; four commands CI runs (Task 5).

This task has no test file of its own: its test is running the scripts, on this machine here and on every
target in CI.

- [ ] **Step 1: Write the scripts**

```js path=scripts/lib/run.mjs
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/** The repository's root. */
export const ROOT = join(import.meta.dirname, '..', '..');

/** Where the packed files go. */
export const DIST = join(ROOT, 'dist');

/** This machine's target, as Node names the operating system and the processor. */
export const TARGET = `${process.platform}-${process.arch}`;

/** The main package's folder. */
export const MAIN_DIR = join(ROOT, 'npm', 'yak-analyzer');

/** The folder of this machine's platform package. */
export const PLATFORM_DIR = join(ROOT, 'npm', 'platforms', TARGET);

/** The binary's file name on this machine. */
export const BINARY = process.platform === 'win32' ? 'yak-analyzer.exe' : 'yak-analyzer';

/** The release build of the binary. */
export const BUILT = join(ROOT, 'target', 'release', BINARY);

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
```

```js path=scripts/pack.mjs
/**
 * Packs one package into `dist/`, as the file npm would publish (spec PKG-9).
 *
 * `node scripts/pack.mjs main` packs the main package. `node scripts/pack.mjs platform` copies the release build
 * of the binary into this machine's platform package, and packs that.
 */
import { chmodSync, copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

import { BINARY, BUILT, DIST, fail, MAIN_DIR, npm, PLATFORM_DIR, ROOT, TARGET } from './lib/run.mjs';

/** Which package to pack. */
const which = process.argv[2];

if (which !== 'main' && which !== 'platform') {
  fail('usage: node scripts/pack.mjs main|platform');
}

/** The folder of the package to pack. */
const dir = which === 'main' ? MAIN_DIR : PLATFORM_DIR;

if (!existsSync(dir)) {
  fail(`there is no package for ${TARGET}: ${dir} does not exist`);
}

if (which === 'platform') {
  if (!existsSync(BUILT)) {
    fail(`${BUILT} is not built; run \`cargo build --release\` first`);
  }

  copyFileSync(BUILT, join(dir, BINARY));
  chmodSync(join(dir, BINARY), 0o755);
}

copyFileSync(join(ROOT, 'LICENSE'), join(dir, 'LICENSE'));
mkdirSync(DIST, { recursive: true });

/** How `npm pack` ended. */
const packed = npm('npm', ['pack', '--pack-destination', DIST], { cwd: dir, stdio: 'inherit' });

if (packed.status !== 0) {
  fail(`npm pack failed in ${dir}`);
}
```

```js path=scripts/packed-install.mjs
/**
 * Installs the packed main package and this machine's packed platform package into a new folder, the way npm and
 * yak will, and runs the launcher's version answer (spec PKG-9). The install fetches nothing and runs no install
 * script: the four other platform packages are simply absent, as they are on a user's machine.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { fail, MAIN_DIR, manifest, npm, packedFile, PLATFORM_DIR } from './lib/run.mjs';

/** The two packed files. */
const files = [packedFile(MAIN_DIR), packedFile(PLATFORM_DIR)];

for (const file of files) {
  if (!existsSync(file)) {
    fail(`${file} is not packed; run scripts/pack.mjs first`);
  }
}

/** The main package's name, version and `bin` map. */
const { bin, name, version } = manifest(MAIN_DIR);

/** The line the binary prints for its version (spec BIN-1). */
const expected = `yak-analyzer ${version}\n`;

/** A new folder to install into. */
const folder = mkdtempSync(join(tmpdir(), 'yak-analyzer-install-'));

/** What went wrong, if anything did. */
let problem;

try {
  // The folder needs a manifest of its own, or npm installs into the nearest folder above that has one.
  writeFileSync(join(folder, 'package.json'), '{"private":true}\n');

  const installed = npm(
    'npm',
    ['install', '--offline', '--ignore-scripts', '--no-audit', '--no-fund', '--no-update-notifier', ...files],
    { cwd: folder, stdio: 'inherit' }
  );

  if (installed.status === 0) {
    // yak starts a tool by running its `bin` file with Node, through no wrapper script.
    const answered = spawnSync(process.execPath, [join(folder, 'node_modules', name, bin['yak-analyzer']), '--version'], {
      encoding: 'utf8',
    });

    if (answered.status !== 0 || answered.stdout !== expected) {
      problem = `the installed launcher answered ${JSON.stringify(answered.stdout)} and exited ${answered.status} (${answered.stderr.trim()}); expected ${JSON.stringify(expected)}`;
    }
  } else {
    problem = 'npm could not install the packed files';
  }
} finally {
  rmSync(folder, { force: true, recursive: true });
}

if (problem !== undefined) {
  fail(problem);
}

console.log(`the packed install answers ${expected.trim()}`);
```

```js path=scripts/check-floor.mjs
/**
 * Checks that the release build asks for nothing newer than the Node yak needs (spec PKG-2a): glibc 2.28 on
 * Linux, macOS 11.0, and no C runtime library on Windows. A build takes the libraries of the machine it is made
 * on as its floor unless told otherwise, and that machine's own tests cannot notice.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

import { BUILT, fail } from './lib/run.mjs';

/** The newest glibc the Linux binaries may ask for. */
const GLIBC_FLOOR = '2.28';

/** The newest macOS the macOS binaries may ask for. */
const MACOS_FLOOR = '11.0';

/** The Windows libraries that are a C runtime. */
const C_RUNTIME = /^(?:vcruntime|msvcp|ucrtbase|api-ms-win-crt-)/iu;

/**
 * Runs a program and gives what it printed.
 *
 * @param {string} program - The program.
 * @param {readonly string[]} args - Its arguments.
 * @returns {string} Its standard output.
 */
const printed = (program, args) => {
  const ended = spawnSync(program, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });

  if (ended.status !== 0) {
    fail(`${program} failed: ${ended.error?.message ?? ended.stderr}`);
  }

  return ended.stdout;
};

/**
 * Whether one dotted version is newer than another.
 *
 * @param {string} version - The version to judge.
 * @param {string} floor - The version to judge it against.
 * @returns {boolean} True when `version` is newer than `floor`.
 */
const newerThan = (version, floor) => {
  const left = version.split('.').map(Number);
  const right = floor.split('.').map(Number);

  for (let i = 0; i < Math.max(left.length, right.length); i += 1) {
    const difference = (left[i] ?? 0) - (right[i] ?? 0);

    if (difference !== 0) {
      return difference > 0;
    }
  }

  return false;
};

/**
 * The newest of some dotted versions.
 *
 * @param {readonly string[]} versions - The versions.
 * @returns {string} The newest.
 */
const newest = (versions) => {
  return versions.reduce((best, version) => {
    return newerThan(version, best) ? version : best;
  });
};

if (!existsSync(BUILT)) {
  fail(`${BUILT} is not built; run \`cargo build --release\` first`);
}

if (process.platform === 'linux') {
  const versions = [...printed('objdump', ['-T', BUILT]).matchAll(/GLIBC_(\d+(?:\.\d+)+)/gu)].map(([, version]) => {
    return version;
  });

  if (versions.length === 0) {
    fail(`objdump shows no glibc version in ${BUILT}`);
  }

  if (newerThan(newest(versions), GLIBC_FLOOR)) {
    fail(`${BUILT} asks for glibc ${newest(versions)}, newer than ${GLIBC_FLOOR}`);
  }

  console.log(`the binary asks for glibc ${newest(versions)} at most`);
} else if (process.platform === 'darwin') {
  const minimum = /LC_BUILD_VERSION[\s\S]*?minos (\S+)/u.exec(printed('otool', ['-l', BUILT]))?.[1];

  if (minimum === undefined) {
    fail(`otool shows no minimum macOS version in ${BUILT}`);
  }

  if (newerThan(minimum, MACOS_FLOOR)) {
    fail(`${BUILT} asks for macOS ${minimum}, newer than ${MACOS_FLOOR}`);
  }

  console.log(`the binary asks for macOS ${minimum} at least`);
} else if (process.platform === 'win32') {
  const vswhere = join(process.env['ProgramFiles(x86)'] ?? '', 'Microsoft Visual Studio', 'Installer', 'vswhere.exe');
  const installation = printed(vswhere, [
    '-latest',
    '-products',
    '*',
    '-requires',
    'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
    '-property',
    'installationPath',
  ]).trim();
  const tools = join(installation, 'VC', 'Tools', 'MSVC');
  const dumpbin = join(tools, readdirSync(tools).sort().at(-1) ?? '', 'bin', 'Hostx64', 'x64', 'dumpbin.exe');
  const libraries = printed(dumpbin, ['/NOLOGO', '/DEPENDENTS', BUILT])
    .split(/\r?\n/u)
    .map((line) => {
      return line.trim();
    })
    .filter((line) => {
      return /^[\w.-]+\.dll$/iu.test(line);
    });
  const runtime = libraries.filter((library) => {
    return C_RUNTIME.test(library);
  });

  if (runtime.length > 0) {
    fail(`${BUILT} needs a C runtime to be installed: ${runtime.join(', ')}`);
  }

  console.log(`the binary carries its C runtime; it imports ${libraries.join(', ')}`);
} else {
  fail(`no floor is known for ${process.platform}`);
}
```

```js path=scripts/dry-run-publish.mjs
/**
 * Tries the publishing on every packed file in `dist/`, without publishing (spec PUB-10), with the npm that will
 * publish. Between releases the version in the tree is one already on npmjs.com, and npm's dry run refuses a
 * version that exists; that one refusal counts as a pass.
 */
import { existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

import { DIST, fail, npm } from './lib/run.mjs';

/** The npm that publishes. Trusted publishing needs 11.5.1 or later, and npm 12 does not run on Node 22.18. */
const PUBLISHER = 'npm@11.21.2';

/** Where the packages are published, said on the command so that a machine's own settings cannot change it. */
const REGISTRY = '--@dabay6:registry=https://registry.npmjs.org/';

/** What npm says when the version is already published. */
const ALREADY_PUBLISHED = 'cannot publish over the previously published versions';

/** The packed files. */
const files = existsSync(DIST)
  ? readdirSync(DIST)
      .filter((file) => {
        return file.endsWith('.tgz');
      })
      .sort()
  : [];

if (files.length === 0) {
  fail(`${DIST} holds no packed file; run scripts/pack.mjs first`);
}

for (const file of files) {
  const tried = npm('npx', ['--yes', PUBLISHER, 'publish', '--dry-run', '--access', 'public', REGISTRY, join(DIST, file)]);
  const said = `${tried.stdout ?? ''}${tried.stderr ?? ''}`;

  if (tried.status === 0) {
    console.log(`the dry run passed for ${file}`);
  } else if (said.includes(ALREADY_PUBLISHED)) {
    console.log(`the dry run passed for ${file}; its version is already published`);
  } else {
    console.error(said);
    fail(`npm's dry run refused ${file}`);
  }
}
```

```text path=.gitignore
# Rust build output
/target/

# npm packaging
node_modules/
/dist/

# Copied into each package when it is packed
/npm/yak-analyzer/LICENSE
/npm/platforms/*/LICENSE
/npm/platforms/*/yak-analyzer
/npm/platforms/*/yak-analyzer.exe
```

- [ ] **Step 2: Run them on this machine**

```bash
cargo build --release && node scripts/pack.mjs main && node scripts/pack.mjs platform && node scripts/packed-install.mjs && node scripts/check-floor.mjs && node scripts/dry-run-publish.mjs
```

Expected: `npm pack` lists each package's files twice over, then
`the packed install answers yak-analyzer 0.0.0`, then one line saying what the binary asks of the system, then
`the dry run passed for` each of the two packed files. `git status --short` shows nothing new: the packed files
and the copies are ignored.

- [ ] **Step 3: Commit**

```bash
git add scripts .gitignore && git commit -m "build: pack the packages, install them offline and check the binary's floor"
```

Expected: the commit is made.

### Task 5: CI on all five targets

**Implements:** CI-1, CI-1a, CI-2, CI-3, CI-4, PKG-2a, PKG-9, PUB-10

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: the gates of Task 2, the tests of Task 3, the scripts of Task 4.
- Produces: the workflow `ci.yml` with the jobs `pack-main` and `target`; the artifacts `pkg-main` and
  `pkg-<target>`, which the second plan's publishing job takes.

This task has no test command: a workflow is tested by running it, which Step 2 does.

- [ ] **Step 1: Write the workflow**

```yaml path=.github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

# A pull request's jobs can read the repository and nothing more (spec CI-2, CI-3).
permissions:
  contents: read

concurrency:
  group: ci-${{ github.ref }}
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
      - name: The launcher's tests
        run: node --test "npm/**/*.test.mjs"
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
```

- [ ] **Step 2: Commit, push the branch and open a draft pull request, so the workflow runs**

```bash
git add .github && git commit -m "ci: build, test, pack and install on five targets"
git push -u origin feat/skeleton-and-release-pipeline
gh pr create --draft --base main --head feat/skeleton-and-release-pipeline --title "feat: a walking skeleton that answers its version" --body "A draft, opened so that CI runs. Not for merging until the owner has validated the plan."
gh pr checks --watch
```

Expected: the job `Pack the main package` and the five jobs `win32-x64`, `darwin-arm64`, `darwin-x64`,
`linux-x64` and `linux-arm64` all pass. Each target's log shows the gates, one line saying what the binary asks
of the system (glibc 2.28 at most on Linux, macOS 11.0 on macOS, no C runtime on Windows),
`the packed install answers yak-analyzer 0.0.0`, and a passed dry run. A failure is a fault to find and fix on
the branch, recorded as an amendment with its reason, and the run is watched again.

### Task 6: the repository's own text

**Implements:** DOC-1, DOC-2, GATE-2

**Files:**
- Create: `CLAUDE.md`
- Modify: `CONTRIBUTING.md`, `README.md`

**Interfaces:**
- Consumes: the gate commands of Task 2 and Task 3, and the scripts of Task 4, by name.
- Produces: the repository's text, true after this plan.

- [ ] **Step 1: Write the text**

```markdown path=CLAUDE.md
# CLAUDE.md

yak-analyzer is a static analyzer for TypeScript and JavaScript, built for yakshaver. Read
`CONTRIBUTING.md` before changing anything: it holds the rules that shape every change here, the
toolchain and the gates, and none of them is repeated in this file.

What a session needs beyond them:

- **The design spec** is `docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver
  repository. Each spec in this repository's own `docs/specs/` builds one part of it.
- **A work item here names the design spec's requirements it covers,** by their IDs, in its own
  spec.
```

````markdown edit path=CONTRIBUTING.md
<<<<<<< find
There is no code here yet, so there is no build or test command to give. This file holds the
rules that are already decided; the toolchain and the commands are added with the first stage of
the work.
=======
This file holds the rules that shape every change here, the toolchain, and the gates a change
passes before it is reviewable.
>>>>>>> replace
<<<<<<< find
- **No private names.** Nothing committed here names a private repository or copies its code,
  including the repositories the analyzer is checked against.
=======
- **No private names.** Nothing committed here names a machine, or a private repository other
  than yakshaver, which this program is built for; and nothing copies a private repository's
  code. That includes the repositories the analyzer is checked against.
>>>>>>> replace
<<<<<<< find
## Commits and pull requests
=======
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
node --test "npm/**/*.test.mjs"
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

## Commits and pull requests
>>>>>>> replace
````

```markdown edit path=README.md
<<<<<<< find
Nothing is built yet. This repository was created on 2026-11-11 to hold the work. The design is
written and approved; the first stage has not started, and there is no release.
=======
The walking skeleton is in: a binary that answers its version, built and tested on five targets,
with its npm packages put together and installed in CI. It analyzes nothing yet, and there is no
release.
>>>>>>> replace
```

- [ ] **Step 2: Commit and push**

```bash
git add CLAUDE.md CONTRIBUTING.md README.md && git commit -m "docs: say what exists, and how to build and check it"
git push
gh pr checks --watch
```

Expected: the commit is made and pushed, and the six jobs pass again on it.
