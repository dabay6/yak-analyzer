# A walking skeleton: a binary that answers its version, built on three platforms and released through npm

**Status:** draft.

**Work item:** `2026-11-11-skeleton-and-release-pipeline`, the first item of the analyzer's stage 1.

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. Its IDs are cited as "design AN-1". This spec changes nothing in it. It covers
design AN-1, AN-2, the version answer of IF-1, REL-1 to REL-4 and REL-6, for a program that does
not analyze anything yet.

"Session's call" after a requirement means the session that wrote this spec decided it and the
owner has not ruled on it. The owner asked for this spec to be written from what the design spec
already decides, so every choice here that the design spec does not make is the session's call,
whether or not it is marked. A requirement marked "design" restates the design spec.

## Intent

The analyzer is a Rust program that yakshaver installs from npm, as it installs any tool server:
one main package whose `bin` is a small JavaScript launcher, and one package per platform holding
the binary. Nothing of that exists. This repository holds a license and a README.

Two things about that path do not depend on any analysis being written:

- Whether five native binaries can be built, tested on their own operating system and
  architecture, and published together from GitHub's runners. This is unknown.
- Whether the package's name works. The design spec places the packages on npmjs.com under the
  `@dabay6` scope, and has the name checked before it is fixed (design REL-2). That check was made
  on 2026-11-11, before this spec: yak installs a tool by running npm in its own tools folder, so
  only the machine's own npm settings apply, and the owner's machine maps the `@dabay6` scope to
  GitHub Packages there. npm sends every request for a scoped name to the registry its scope is
  mapped to and tries no other. So yak on that machine cannot install any `@dabay6` package from
  npmjs.com, and the design spec's answer applies: the packages take a name the mapping does not
  catch (NAME-1).

So the first item is a walking skeleton: a binary that only answers its version, taken the whole
way through the gates, the build, the packaging, a real release and one install through yak.
Every later item of stage 1 ships through what this one builds.

Terms:

- **The binary** is the Rust program, `yak-analyzer`. **The launcher** is the JavaScript file the
  main npm package names as its `bin`.
- **The main package** is the one yak pins and installs. **A platform package** holds the binary
  for one operating system and architecture.
- **A target** is one operating system and architecture a binary is built for.
- **The probe** is the command yak runs after an install to read the installed version; yak
  accepts the install when the output contains the pinned version.
- **Provenance** is npm's signed statement of which repository, commit and workflow built a
  published package.
- **release-please** is the release tool the owner's other repositories use. It keeps one open
  pull request that holds the next version and its notes, worked out from the Conventional
  Commits on `main`; merging that pull request creates the version's tag and GitHub release.

## Goals, non-goals and anti-goals

**Goals:**

- A release of this repository puts a working `yak-analyzer` on npmjs.com for five targets, and
  yak installs it and reads its version.
- The packages have a name yak can install on a machine that maps `@dabay6` elsewhere.
- Every binary released has run its tests on its own operating system and architecture.
- The owner's part in a release is one merge.
- The gates, the build and the release are in place, so that a later item adds analysis and
  nothing else.

**Non-goals:**

- Any analysis: no parsing, no oxc dependency, no findings, no SARIF, no hand-off file, no MCP
  server. Each comes in a later item.
- The interface version's answer (design IF-1, IF-6). It comes with the item that first reads a
  hand-off file, where its values are real. Session's call.
- What happens to the binary when the launcher is stopped. The skeleton's binary ends at once;
  the item that adds the first long-running command states and tests it.
- Fixing the split of the rest of stage 1.
- musl Linux and Windows on arm64 (design REL-3).
- The pull request that moves the analyzer's pin in yakshaver's content pack (design REL-5). The
  pack declares no analyzer until the swap.
- Publishing the crate on crates.io, or release files for download from GitHub (design,
  alternatives).
- A cache, speed targets, or any measurement of speed.

**Anti-goals,** failures even if it technically works:

- A package with an install script, or a launcher that downloads anything.
- A main package published without every platform package of the same version.
- A released binary whose printed version differs from its package's version.
- A long-lived npm token in the repository's secrets where npm offers a way without one.
- A workflow that gives a pull request from a fork a token that can write or publish.
- Committed text that names a private repository or a machine.

## Requirements

### The name

- **NAME-1** The npm packages are not under the `@dabay6` scope, or under any scope the owner's
  machines map to another registry. The owner chooses the name before the plan is written. The
  session proposes no scope at all: `yak-analyzer` for the main package, and `yak-analyzer-`
  followed by the operating system and processor for each platform package. Those names were
  unused on npmjs.com on 2026-11-11. The binary, the repository and the command keep the name
  `yak-analyzer` whatever the packages are called.

### The binary

- **BIN-1** The binary is named `yak-analyzer`. Given its version option, it prints one line,
  `yak-analyzer <version>`, on standard output and exits 0. This is the probe (design IF-1).
- **BIN-2** Given anything else, or nothing, it writes one line on standard error that says what
  it was given and names the version option, prints nothing on standard output, and exits
  non-zero (design IF-3, for the shape).
- **BIN-3** The version it prints is the crate's version, read at build time, never written twice.
- **BIN-4** It reads no file, starts no program and uses no network (design AN-6).

### Toolchain and gates

- **GATE-1** The repository pins one exact stable Rust version in a toolchain file, so every
  machine and every runner builds with the same compiler. `Cargo.lock` is committed, and every
  dependency in `Cargo.toml` is pinned to an exact version (design AN-2, for oxc; here for all).
- **GATE-2** Three gates pass before a change is reviewable: formatting is checked, the linter
  runs with warnings as errors, and the tests pass. Each is one command, written in
  `CONTRIBUTING.md`.
- **GATE-3** The tests start the built binary and check BIN-1 and BIN-2 by their output and exit
  code. The launcher has its own tests (PKG-6), which need no package beyond Node itself.
- **GATE-4** The skeleton keeps its dependencies to the fewest that do the job. A dependency added
  only to parse one option needs a reason in the plan.

### Continuous integration

- **CI-1** One job builds the binary for a target and runs the gates and the launcher's tests on
  a runner of that target's own operating system and architecture. It runs for all five targets
  (PKG-2) on every pull request and every push to `main`, and the release runs the same job
  (PUB-2), so a release repeats what already passed. The runners are GitHub's own, which are free
  for a public repository (design, alternatives). The exception is PUB-2's.
- **CI-2** A workflow triggered by a pull request has a read-only token and no secret. No workflow
  uses a trigger that runs a fork's code with the repository's rights.
- **CI-3** Each workflow asks for the least permission it needs, stated in the workflow.

### The npm packages

- **PKG-1** The main package, named by NAME-1, has the launcher as its `bin`, under the name
  `yak-analyzer` (design REL-1).
- **PKG-2** There is one platform package for each target: Windows x64, macOS arm64, macOS x64,
  Linux x64 and Linux arm64, the Linux ones for glibc (design REL-3). Each states its operating
  system and processor in its `package.json`, so npm installs only the one that matches.
- **PKG-3** The main package lists every platform package as an optional dependency at the one
  exact version released with it (design REL-1).
- **PKG-4** No package has an install script of any kind (design REL-1).
- **PKG-5** The launcher finds the platform package's binary, runs it with the arguments,
  standard input, standard output and standard error it was given, and exits with the binary's
  exit code. It prints nothing of its own when the binary runs. It answers no command itself, so
  the version the probe reads is printed by the binary (design REL-1). yak runs a tool's `bin`
  file directly with the Node that runs yak, through no shim, so the launcher is one plain script
  that runs on the oldest Node yak supports, 22.18, and the main package states that.
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

- **PUB-1** A release is cut by merging release-please's pull request. That merge is the owner's
  whole part. The first release is 0.1.0.
- **PUB-1a** The release pull request gets its CI run like any other, so it can be merged with no
  step beyond the merge. A pull request a workflow opens with its own token starts no other
  workflow, so the plan says how this one's checks run.
- **PUB-2** A release builds the binary for all five targets and runs their tests on a runner of
  each target's own operating system and architecture, by CI-1's job (design REL-4). The one
  exception is a target GitHub offers no free runner for: it is built and released without ever
  being run, PUB-3 is not checked for it, and the release notes name it.
- **PUB-3** Before anything is published, the release checks that each built binary prints the
  version being released (BIN-1). A mismatch, or a failed test, stops the release with nothing
  published to npm. The version's tag and GitHub release exist by then, since the merge made
  them; they stay, with a note that the version was not published, and the fix goes out as the
  next version.
- **PUB-4** The packages are published to npmjs.com, public, with provenance (design REL-2). The
  platform packages go first and the main package last, so a main package never exists without
  its platform packages.
- **PUB-5** A release that fails part of the way through publishing can be run again: a package
  version already published is skipped, and the rest are published. Nothing is ever unpublished
  or overwritten.
- **PUB-6** Publishing uses npm's trusted publishing from this repository's release workflow
  wherever npm allows it, so the repository holds no long-lived npm token. Where a package's
  first publish cannot be done that way, the plan says what the owner does once, and the token
  for it is removed after.
- **PUB-7** The release notes say which targets were tested on their own runner and which were
  not (PUB-2).
- **PUB-8** A scheduled job opens pull requests that move the pinned versions: the Cargo
  dependencies and the workflow actions. CI decides whether one can merge; none merges by
  itself. This is where oxc's pins will move once a later item adds them (design REL-6).

### The install check

- **CHK-1** After the first release, yak installs the released main package as a tool server and
  runs its probe, on a machine whose own npm settings map the `@dabay6` scope to GitHub Packages
  (design REL-2). NAME-1 settled the name; this confirms the whole path.
- **CHK-2** The tool server is declared by a pack made for the check and removed after it, added
  to a scratch project that no host is set up from. A pack must give a tool server a command that
  starts an MCP server, and the skeleton has none, so that line is a stand-in and is never run:
  the check runs yak's install and nothing else. yakshaver's generic pack is not changed.
- **CHK-3** The check passes when the install comes from npmjs.com, the platform package for that
  machine is installed with it, and the probe prints the released version. yak's own npm command
  leaves no optional dependency out and turns no install script off; only a machine's npm
  settings could, and the launcher says so when they do (PKG-6).
- **CHK-4** The result, with the registry each package came from, is written on the work item's
  record. No later item of stage 1 publishes before it is. A failed check stops the work until
  the owner has decided what to change.

### The repository's own text

- **DOC-1** `CONTRIBUTING.md` gains the toolchain, how to install it, and the gate commands.
  `README.md` gains how to install and run the released binary, and its status section says what
  exists.
- **DOC-2** A `CLAUDE.md` tells an agent session the rules that shape every change here: those
  of `CONTRIBUTING.md`, above all that committed text never names a private repository or a
  machine.

## Alternatives considered

- **Keeping the `@dabay6` scope, and changing each machine's npm settings.** The owner's private
  packages are on GitHub Packages under that scope and need the mapping until they are public, and
  every machine that installs the analyzer would need the change. The design spec already says
  which way to go (design REL-2).
- **Publishing the analyzer to GitHub Packages as well.** yak would find it where the mapping
  points. GitHub Packages asks for a token on every read, and the design spec wants an install
  that needs none.
- **A release tool that writes the npm installer for a Rust binary.** The common ones install by
  downloading the binary in an install script, which the design spec rules out (design REL-1):
  yak asks a person's yes for an install, and a script that fetches code after it is a second,
  unseen step.
- **Cross-compiling every target on one runner.** One job instead of five, and no dependence on
  which runners GitHub offers. The Windows and macOS binaries would ship without ever running
  (design REL-4).
- **Testing three targets on pull requests and all five only at release.** Fewer runner minutes
  on each pull request. Two targets would first run after the owner's merge, when the version's
  tag already exists, and there would be two build setups to keep in step.
- **One package holding all five binaries.** Simpler to publish, and no optional dependencies to
  go missing. Every install would carry four binaries it cannot run, and the design spec chose
  the layout that fallow, esbuild and Biome use.
- **Publishing by a tag pushed by hand.** No release tooling to configure. The owner would have
  to remember the version and the tag, where a release pull request shows what is going out
  before the merge.
- **Starting with the analysis and adding the release later.** The first useful code would come
  sooner. The five-target build and the publishing would then be tested for the first time with
  real work depending on them.
- **Answering the interface version now.** The design spec lists it among stage 1's answers.
  Nothing reads it until declarations exist, and it would publish a shape with placeholder
  values.

## Risks

- **A name once published cannot be taken back.** npm lets a package be deprecated, not removed.
  Handled by settling the name before the plan (NAME-1), so the first publish is under the name
  that stays.
- **The first publish of six new packages cannot use trusted publishing.** Handled by PUB-6: the
  plan says what the owner does once.
- **A release fails after its tag exists.** Handled by running the same job on every pull request
  (CI-1), so a release rarely meets a new failure, and by PUB-3 saying what is left behind.
- **A release fails part of the way through publishing.** Handled by the publishing order
  (PUB-4) and by a rerun that skips what is published (PUB-5).
- **GitHub retires a runner a target needs.** The macOS x64 runners are the likely case. Handled
  by PUB-2's exception: the target is still built, and the release notes say it was never run.
- **The platform package is left out of an install,** by an npm setting that omits optional
  dependencies. Handled by the launcher's line (PKG-6).
- **A fork's pull request runs code in CI.** The repository is public. Handled by CI-2 and CI-3.
- **A compromised action or dependency reaches a release.** Handled in part: exact pins, a
  committed lock file, pull requests for every pin change (PUB-8), least permission (CI-3), no
  install scripts, and provenance. Nothing in yak checks provenance (design REL-2).
- **The binary and the package disagree on the version.** Handled by one source for the version
  (BIN-3, PKG-8) and by the check before publishing (PUB-3).
- **Another machine maps a different scope, or the default registry.** Not handled: an unscoped
  name follows the machine's default registry, and a machine that points that elsewhere cannot
  install most of npm either.

## Left to the builder

- The name of the version option.
- The exact Rust version, the crate layout, and whether the option is parsed by hand.
- How the Linux platform packages state glibc.
- Which runner builds and tests each target, checked against what GitHub offers when the plan is
  written.
- release-please's configuration, how one version reaches the crate and the six packages, and
  how the release pull request gets its CI run.
- How trusted publishing is set up for six new packages, and what the owner does once.
- Which tool opens the pin pull requests.
- The pack made for the install check.
