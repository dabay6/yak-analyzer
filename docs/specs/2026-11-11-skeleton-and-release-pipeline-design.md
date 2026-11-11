# A walking skeleton: a binary that answers its version, built on three platforms and released through npm

**Status:** draft.

**Work item:** `2026-11-11-skeleton-and-release-pipeline`, the first item of the analyzer's stage 1.

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. Its IDs are cited as "design AN-1". This spec changes nothing in it. It covers
design AN-1, AN-2, the version and interface answers of IF-1, REL-1 to REL-4 and REL-6, for a
program that does not analyze anything yet.

"Session's call" after a requirement means the session that wrote this spec decided it and the
owner has not ruled on it. The owner asked for this spec to be written from what the design spec
already decides, so every choice here that the design spec does not make is the session's call,
whether or not it is marked. A requirement marked "design" restates the design spec.

## Intent

The analyzer is a Rust program that yakshaver installs from npm, as it installs any tool server:
one main package whose `bin` is a small JavaScript launcher, and one package per platform holding
the binary. Nothing of that exists. This repository holds a license and a README.

Two things about that path are unknown, and neither depends on any analysis being written:

- Whether five native binaries can be built, tested on their own operating system and
  architecture, and published together from GitHub's runners.
- Whether the package's name works. The owner's machines and repositories map the `@dabay6` npm
  scope to GitHub Packages, and the analyzer is to be public on npmjs.com under that scope. If the
  mapping sends an install to the wrong registry, the name has to change, and that is cheapest to
  learn before anything depends on it.

So the first item is a walking skeleton: a binary that only answers its version and its interface
version, taken the whole way through the gates, the build, the packaging, a real release and one
install through yak. Every later item of stage 1 ships through what this one builds.

Terms:

- **The binary** is the Rust program, `yak-analyzer`. **The launcher** is the JavaScript file the
  main npm package names as its `bin`.
- **The main package** is the one yak pins and installs. **A platform package** holds the binary
  for one operating system and architecture.
- **A target** is one operating system and architecture a binary is built for.
- **The probe** is the command yak runs after an install to read the installed version; yak
  accepts the install when the output contains the pinned version.
- **The interface version** is the one whole number that covers the analyzer's commands,
  declaration keys, rule names, fingerprints and MCP tools (design IF-6).
- **A hand-off format** is a version of the file yak hands the analyzer on each run (design
  HAND-6).
- **Provenance** is npm's signed statement of which repository, commit and workflow built a
  published package.

## Goals, non-goals and anti-goals

**Goals:**

- A release of this repository puts a working `yak-analyzer` on npmjs.com for five targets, and
  yak installs it and reads its version.
- The package name is settled, by a real install on a machine with the `@dabay6` mapping.
- Every binary released has run its tests on its own operating system and architecture.
- The owner's part in a release is one merge.
- The gates, the build and the release are in place, so that a later item adds analysis and
  nothing else.

**Non-goals:**

- Any analysis: no parsing, no oxc dependency, no findings, no SARIF, no hand-off file, no MCP
  server. Each comes in a later item.
- Fixing the split of the rest of stage 1.
- musl Linux and Windows on arm64 (design REL-3).
- The pull request that moves the analyzer's pin in yakshaver's content pack (design REL-5). The
  pack declares no analyzer until the swap.
- Publishing the crate on crates.io, or release files for download from GitHub (design,
  alternatives).
- A cache, speed targets, or measuring anything beyond what the launcher adds (PUB-9).

**Anti-goals,** failures even if it technically works:

- A package with an install script, or a launcher that downloads anything.
- A main package published without every platform package of the same version.
- A released binary whose printed version differs from its package's version.
- A long-lived npm token in the repository's secrets where npm offers a way without one.
- A workflow that gives a pull request from a fork a token that can write or publish.
- Committed text that names a private repository or a machine.

## Requirements

### The binary

- **BIN-1** The binary is named `yak-analyzer`. Given its version option, it prints one line,
  `yak-analyzer <version>`, on standard output and exits 0. This is the probe (design IF-1).
- **BIN-2** Given its interface command, it prints one JSON object on standard output and exits 0.
  The object holds the interface version and the list of hand-off formats the binary reads
  (design IF-1). In this item the interface version is 0 and the list is empty: the binary reads
  no hand-off file and has no interface yet. 1 is the first interface version a declarations file
  can be written for, and belongs to the item that first reads declarations. Session's call.
- **BIN-3** Given anything else, or nothing, it writes one line on standard error that says what
  it was given and names the two things it answers, prints nothing on standard output, and exits
  non-zero (design IF-3, for the shape).
- **BIN-4** The version it prints is the crate's version, read at build time, never written twice.
- **BIN-5** It reads no file, starts no program and uses no network (design AN-6).

### Toolchain and gates

- **GATE-1** The repository pins one exact stable Rust version in a toolchain file, so every
  machine and every runner builds with the same compiler. `Cargo.lock` is committed, and every
  dependency in `Cargo.toml` is pinned to an exact version (design AN-2, for oxc; here for all).
- **GATE-2** Three gates pass before a change is reviewable: formatting is checked, the linter
  runs with warnings as errors, and the tests pass. Each is one command, written in
  `CONTRIBUTING.md`.
- **GATE-3** The tests start the built binary and check BIN-1 to BIN-3 by their output and exit
  code. The launcher has its own tests (PKG-6).
- **GATE-4** The skeleton keeps its dependencies to the fewest that do the job. A dependency added
  only to parse two commands needs a reason in the plan.

### Continuous integration

- **CI-1** Every pull request and every push to `main` runs the gates on Windows, macOS and Linux,
  on GitHub's own runners, which are free for a public repository (design, alternatives).
- **CI-2** A workflow triggered by a pull request has a read-only token and no secret. No workflow
  uses a trigger that runs a fork's code with the repository's rights.
- **CI-3** Each workflow asks for the least permission it needs, stated in the workflow.

### The npm packages

- **PKG-1** The main package is `@dabay6/yak-analyzer` unless CHK-3 changes the name. Its `bin`
  is the launcher, under the name `yak-analyzer` (design REL-1).
- **PKG-2** There is one platform package for each target: Windows x64, macOS arm64, macOS x64,
  Linux x64 and Linux arm64, the Linux ones for glibc (design REL-3). Each states its operating
  system and processor in its `package.json`, so npm installs only the one that matches.
- **PKG-3** The main package lists every platform package as an optional dependency at the one
  exact version released with it (design REL-1).
- **PKG-4** No package has an install script of any kind (design REL-1).
- **PKG-5** The launcher finds the platform package's binary, runs it with the arguments,
  standard input, standard output and standard error it was given, and exits with the binary's
  exit code. It prints nothing of its own when the binary runs. It answers no command itself, so
  the version the probe reads is printed by the binary (design REL-1).
- **PKG-6** When no platform package is installed, the launcher writes one line on standard error
  and exits non-zero. On a platform with no package, the line names the platforms that have one
  (design REL-3). On a platform that has one, it says the package is missing and that optional
  dependencies must not be left out of the install. The launcher's tests cover both, and the
  ordinary run.
- **PKG-7** Every package carries a description, keywords, the repository, a home page, where to
  report a bug, and the license. So does `Cargo.toml`.
- **PKG-8** The crate and every package have one version number, written in one place and copied
  by the release, never typed by hand in six files.

### Releasing

- **PUB-1** A release is cut by merging a release pull request that the release tooling opens and
  keeps up to date from the Conventional Commits on `main`. That merge is the owner's whole part
  (the owner's other repositories release this way). The first release is 0.1.0.
- **PUB-2** A release builds the binary for all five targets and runs GATE-3's tests on a runner
  of each target's own operating system and architecture (design REL-4). The one exception is a
  target GitHub offers no free runner for: it is built and released without that run, and the
  release notes name it.
- **PUB-3** Before anything is published, the release checks that each built binary prints the
  version being released (BIN-1). A mismatch stops the release with nothing published.
- **PUB-4** The packages are published to npmjs.com, public, with provenance (design REL-2). The
  platform packages go first and the main package last, so a main package never exists without
  its platform packages.
- **PUB-5** A release that fails part of the way can be run again: a package version already
  published is skipped, and the rest are published. Nothing is ever unpublished or overwritten.
- **PUB-6** Publishing uses npm's trusted publishing from this repository's release workflow
  wherever npm allows it, so the repository holds no long-lived npm token. Where a package's
  first publish cannot be done that way, the plan says what the owner does once, and the token
  for it is removed after.
- **PUB-7** The release notes say which targets were tested on their own runner and which were
  not (PUB-2).
- **PUB-8** A scheduled job opens pull requests that move the pinned versions: the Cargo
  dependencies, the workflow actions and the npm development dependencies. CI decides whether one
  can merge; none merges by itself. This is where oxc's pins will move once a later item adds
  them (design REL-6).
- **PUB-9** After the first release, the time the launcher adds over running the binary directly
  is measured once on the owner's machine and written on the work item's record, as a first
  number for the design spec's speed check (design SPEED-3). Nothing is tuned here.

### The install check

- **CHK-1** After the first release, yak installs the released main package as a tool server and
  runs its probe, on a machine whose npm configuration maps the `@dabay6` scope to GitHub
  Packages (design REL-2). The tool server is declared by a pack made for the check and removed
  after it. yakshaver's generic pack is not changed.
- **CHK-2** The check passes when the install comes from npmjs.com, the platform package for that
  machine is installed with it, and the probe prints the released version. The builder first
  reads how yak's installer runs npm, to know whether it would leave optional dependencies out.
- **CHK-3** When the mapping sends the install to the wrong registry, or the install fails for
  the name's sake, the owner chooses another name that the mapping does not catch. The packages
  are released again under it, and the old ones are marked deprecated on npmjs.com with the new
  name.
- **CHK-4** The result, with the registry each package came from, is written on the work item's
  record. No later item of stage 1 publishes before it is.

### The repository's own text

- **DOC-1** `CONTRIBUTING.md` gains the toolchain, how to install it, and the gate commands.
  `README.md` gains how to install and run the released binary, and its status section says what
  exists.
- **DOC-2** A `CLAUDE.md` tells an agent session the rules that shape every change here: those
  of `CONTRIBUTING.md`, above all that committed text never names a private repository or a
  machine.

## Alternatives considered

- **A release tool that writes the npm installer for a Rust binary.** The common ones install by
  downloading the binary in an install script, which the design spec rules out (design REL-1):
  yak asks a person's yes for an install, and a script that fetches code after it is a second,
  unseen step.
- **Cross-compiling every target on one runner.** One job instead of five, and no dependence on
  which runners GitHub offers. The Windows and macOS binaries would ship without ever running
  (design REL-4).
- **One package holding all five binaries.** Simpler to publish, and no optional dependencies to
  go missing. Every install would carry four binaries it cannot run, and the design spec chose
  the layout that fallow, esbuild and Biome use.
- **Publishing by a tag pushed by hand.** No release tooling to configure. The owner would have
  to remember the version and the tag, where a release pull request shows what is going out
  before the merge.
- **Starting with the analysis and adding the release later.** The first useful code would come
  sooner. The package name and the five-target build would then be tested for the first time
  with real work depending on them.
- **An unscoped package name from the start,** which no scope mapping can catch. The design spec
  places the packages under `@dabay6`; CHK-3 falls back to another name only if the check fails.

## Risks

- **The `@dabay6` scope on npmjs.com may not be the owner's to publish under.** The owner's
  packages are on GitHub Packages today. Handled by asking the owner before the plan is written;
  without the scope, nothing can be published under it and CHK-3's fallback applies from the
  start.
- **A release fails part of the way.** Handled by the publishing order (PUB-4) and by a rerun
  that skips what is published (PUB-5).
- **GitHub retires a runner a target needs.** The macOS x64 runners are the likely case. Handled
  by PUB-2's exception: the target is still built, and the release notes say it was not tested.
- **The platform package is left out of an install,** by an npm setting that omits optional
  dependencies. Handled by the launcher's line (PKG-6) and by reading how yak installs (CHK-2).
- **A fork's pull request runs code in CI.** The repository is public. Handled by CI-2 and CI-3.
- **A compromised action or dependency reaches a release.** Handled in part: exact pins, a
  committed lock file, pull requests for every pin change (PUB-8), least permission (CI-3), no
  install scripts, and provenance. Nothing in yak checks provenance (design REL-2).
- **The binary and the package disagree on the version.** Handled by one source for the version
  (BIN-4, PKG-8) and by the check before publishing (PUB-3).
- **Interface version 0 is taken for a real interface.** Handled by BIN-2: the list of hand-off
  formats is empty, and no declarations file can state 0.
- **The first release publishes a name that then changes.** Handled by CHK-3: the old packages
  are deprecated with the new name. They cannot be removed, which is why the check comes before
  any later item publishes (CHK-4).

## Left to the builder

- The names of the version option and the interface command, and the keys of the JSON object.
- The exact Rust version, the crate layout, and whether arguments are parsed by hand.
- The names of the platform packages, and how the Linux ones state glibc.
- Which runner builds and tests each target, checked against what GitHub offers when the plan is
  written.
- The release tooling and its configuration, and how one version reaches the crate and the six
  packages.
- How trusted publishing is set up for six new packages, and what the owner does once.
- Which tool opens the pin pull requests.
- The pack made for the install check, and how the scope mapping is set for it.
- How the launcher's added time is measured.
