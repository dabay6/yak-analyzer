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
  as one JSON document: the packages, the files, each file's imports and where each leads, the
  entry points with what made each one, the files they reach, the uses of each export, and the
  notices. `yak-analyzer graph --hand-off <file>` reads the project the hand-off file names, with
  its declarations. A run that cannot go on prints one line on standard error and exits 1.
- `yak-analyzer audit` reads the project as `graph` does and prints one SARIF 2.1.0 report of seven
  kinds, the five dead-code kinds, complexity and duplication, with the notices as the run's notifications. `yak-analyzer audit
  --hand-off <file> --base <commit>` reads with the hand-off file's declarations, reads the base
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.
- `yak-analyzer complexity` reads the project as `audit` does and prints one JSON document: the
  limits the declarations give `cyclomaticLimit` and `cognitiveLimit`, each function over a limit
  with its scores, the measures it is over and the fingerprint of its finding, and the notices.
  `yak-analyzer complexity --hand-off <file> --all` reads with the hand-off file's declarations
  and lists every function. A command line it refuses exits 2. yak never runs it: it is for a
  person or an agent, and its name, options and document are a draft until the analyzer's stage 5
  releases it as an MCP tool.
- `yak-analyzer duplication` reads the project as `audit` does and prints one JSON document: the
  settings the declarations give duplication, each group of copies with its size, its copies and the
  fingerprint of its finding, and the notices.
  `yak-analyzer duplication --hand-off <file> --all` reads with the hand-off file's declarations and
  lists the groups a person accepted too. A command line it refuses exits 2. yak never runs it: it is
  for a person or an agent, and its name, options and document are a draft until the analyzer's
  stage 5.

Interface version 1 covers what yak relies on, fixed when the analyzer's stage 2 ended on
2026-11-17: the hand-off format the analyzer reads; the declaration keys and their shapes,
`entries`, `data`, `generated`, `internal`, `cyclomaticLimit`, `cognitiveLimit`,
`duplicationMinimum`, `configEntries`, `scriptCommands` and `commandFiles`; the rule names `audit`
reports, `unused-file`, `unused-export`, `unused-type`, `unresolved-import`,
`circular-dependency` and `complexity`; the fingerprint scheme it marks each finding with,
`yakAnalyzer/v1`; `audit`'s options; and what `interface` prints. A change that breaks any of them
raises the interface version, which `interface` prints and yak checks before it runs the analyzer.
A change that only adds, such as a new rule name, does not raise it.

A `complexity` finding carries its function's scores in its property bag, `complexity`:
`function`, `cyclomatic`, `cognitive`, `cyclomaticLimit`, `cognitiveLimit` and `over`, and with a
base `base`. Interface version 1 covers those names and meanings, since the engine will read them,
and the rules that name a function in the finding's fingerprint, its name path, as part of the
scheme. Both are a draft until the swap, the pack release that replaces fallow with the analyzer in
yak: nothing yak runs reads them before then, so changing them raises nothing. From the swap they
are fixed, and a change that breaks them raises the interface version.

Duplication adds to interface version 1 and breaks nothing in it: the rule name `duplication`; the
unit of `duplicationMinimum`, tokens, which no pack declared before; the keys
`duplicationMinimumLines`, `duplicationCopies`, `duplicationIgnore`, `duplicationTestCallees` and
`duplicationAccepted`; and a `duplication` finding's property bag, `duplication`: `tokens`, `lines`
and `copies`, each copy's `path`, `startLine` and `endLine`, in the order of the finding's locations.
The property bag's names, and the form of a run that a group's fingerprint is made from, which is part
of the fingerprint scheme, are a draft until the swap: from then, a change that breaks either raises
the interface version, and a change to the form stops every accepted group and recorded reason from
matching.

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
