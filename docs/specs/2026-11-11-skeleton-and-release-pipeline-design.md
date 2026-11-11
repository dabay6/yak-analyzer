# A walking skeleton: a binary that answers its version, built on three platforms and released through npm

**Status:** draft.

**Work item:** `2026-11-11-skeleton-and-release-pipeline`, the first item of the analyzer's stage 1.

**Builds from:** the analyzer's design spec,
`docs/specs/2026-11-10-in-house-code-analyzer-design.md` in the yakshaver repository, called **the
design spec** below. That repository is not public yet, so this spec says in its own words
everything a builder needs from it. This spec changes none of the design spec's requirements; the
one place it cannot keep the design spec's order is said in the Intent. From it, this item builds
REL-1 to REL-3 (the npm packages, where they are published, and the targets) and the first of the
answers IF-1 lists, the version. It builds AN-2 (exact pins), REL-4 (every released binary tested on
its own platform) and REL-6 (pull requests that move the pins) as far as a program with no analysis
can. AN-1, a separate program in a public repository of its own under the MIT license, was met when
this repository was created.

How to read the marks: "(design REL-1)" after a sentence names the design spec's requirement that
sentence follows from, and "(owner)" marks what the owner decided while this spec was written.
Everything else is the session's call: the session that wrote this spec decided it, and the owner
has not ruled on it. The owner asked for this spec to be written from what the design spec already
decides.

## Intent

The analyzer is a Rust program that yak installs from npm: one main package whose `bin` is a small
JavaScript launcher, and one package per platform holding the binary. Nothing of that exists. This
repository holds a license and a README.

Two things about that path do not depend on any analysis being written:

- Whether five native binaries can be built, tested on their own operating system and architecture,
  and published together from GitHub's runners. This is unknown.
- Whether the package's name works. The design spec places the packages on npmjs.com under the
  `@dabay6` scope, and has yak's install of the package checked before the name is fixed (design
  REL-2). No install could be tried before this spec: an install needs a published package, and
  publishing is what fixes a name. So the question was answered on 2026-11-11 from two things that
  were read: yak's own code for installing a tool, and npm's settings on the owner's machine. yak
  installs a tool by running `npm install` from its own tools folder, outside any project, so the
  npm settings that apply are the user's and the machine's own: their configuration files, and any
  npm setting in the environment yak was started with. On the owner's machine, one line of the
  user's npm configuration file maps `@dabay6` to GitHub Packages, where the owner's private
  packages are. npm sends every request for a scoped name to the registry its scope is mapped to and
  tries no other, so yak there could not install a `@dabay6` package from npmjs.com. The design
  spec's answer to that was a name the mapping does not catch. The owner decided otherwise on
  2026-11-11: the packages keep `@dabay6`, and the mapping leaves the user's settings (NAME-1,
  NAME-2). Each project that needs `@dabay6` from GitHub Packages states the mapping in its own
  `.npmrc`, which yak's install never reads. The install check (CHK) confirms all of this after the
  first release.

So the first item is a walking skeleton: a binary that only answers its version, taken the whole way
through the gates, the build, the packaging, a real release and one install through yak. Every later
item ships through what this one builds.

Terms:

- **The owner** is the person who owns this repository and approves its specs, plans and releases.
  **A session** is an AI agent's working session on the owner's behalf; one wrote this spec, and one
  will usually be the builder.
- **yak** is the command of yakshaver, the developer tool the analyzer is built for. It runs on
  Node.
- **A pack** is a folder of content yak loads. Among other things a pack can declare a **tool
  server**: a program yak installs at one exact pinned version, from npm, into **yak's tools
  folder** under the user's home, each version in a folder of its own. A pack's declaration gives
  the package, the version, the probe, and the command that starts the tool's server; CHK-2 shows
  one. `yak tools install` does the install and asks the person at the terminal to confirm it first.
- **The probe** is the command yak runs after an install to read the installed version; yak accepts
  the install when the output contains the pinned version.
- **A host** is an agent program, such as Claude Code, that yak sets up to use a project's tool
  servers.
- **A work item** is yak's record of one piece of work: its spec, its plans and a log of decisions,
  **the work item's record**. **The builder** is whoever plans and builds from this spec. **The
  plan** is the builder's written plan, which the owner approves before it is carried out.
- **Stage 1** is the design spec's first stage: the analyzer's repository, its release, and its
  first findings. This item is the first part of it. **The swap** is the design spec's last stage,
  when yakshaver's own pack first declares the analyzer in place of the tool it uses today. Nothing
  installs the analyzer from npm for real use before the swap.
- **The binary** is the Rust program, `yak-analyzer`. **The crate** is its Rust package, whose
  manifest is `Cargo.toml` and whose lock file is `Cargo.lock`; "the lock file" below always means
  `Cargo.lock`. **The launcher** is the JavaScript file the main npm package names as its `bin`.
- **The main package** is the one yak pins and installs. **A platform package** holds the binary for
  one target.
- **A target** is what a binary is built for: an operating system and a processor, and on Linux the
  C library, glibc.
- **Provenance** is npm's signed statement of which repository, commit and workflow built a
  published package.
- **Trusted publishing** is npm's way of letting one named workflow of one repository publish a
  package with no stored token.
- **release-please** is the release tool the owner's other repositories use. It runs as one job of
  this repository's workflow (CI-1), on every push to `main`. It keeps one open pull request, **the
  release pull request**, that holds the next version and its notes, worked out from the
  Conventional Commits on `main`. When that pull request is merged, release-please's run on the
  merge creates the version's tag and its GitHub release.

## Goals, non-goals and anti-goals

**Goals:**

- A release of this repository puts a working `yak-analyzer` on npmjs.com for five targets, and yak
  installs it and reads its version.
- yak installs the packages from npmjs.com on the owner's machines, while the owner's projects go on
  taking their private `@dabay6` packages from GitHub Packages.
- Every binary released has run its tests on its own operating system and processor.
- The owner's part in a release is one merge and one approval of the publishing (PROT-2), after a
  setup done once for the first release (PUB-6).
- The gates, the build and the release are in place, so that a later item adds analysis and nothing
  else.

**Non-goals:**

- Any analysis: no parsing, no findings, no report, no server. Each comes in a later item.
- The interface version, the second answer design IF-1 lists: a number later items use to tell
  whether yak and the analyzer agree. It comes with the item that first needs it.
- What happens to the binary when the launcher is stopped. The skeleton's binary ends at once; the
  item that adds the first long-running command states and tests it.
- Fixing how the rest of stage 1 is split into items.
- Linux with the musl C library, and Windows on arm64 (design REL-3).
- Moving the analyzer's pinned version in yakshaver's pack (design REL-5). The pack declares no
  analyzer until the swap.
- Publishing the crate on crates.io, or release files for download from GitHub.
- A formatter or linter for the launcher's JavaScript. It is one small file with its own tests.
- A cache, speed targets, or any measurement of speed.

**Anti-goals,** failures even if it technically works:

- A package with an install script, or a launcher that downloads anything.
- A main package published without every platform package of the same version.
- A released binary whose printed version differs from its package's version.
- A long-lived npm token in the repository's secrets.
- A workflow that gives a pull request from a fork a token that can write or publish.
- Committed text that names a machine, or any private repository other than yakshaver itself, which
  this program is built for and may name (owner: yakshaver is to be public).

## Requirements

### The name

- **NAME-1** The npm packages are under the `@dabay6` scope on npmjs.com, as the design spec places
  them (owner; design REL-2). The main package is `@dabay6/yak-analyzer`. Each platform package is
  `@dabay6/yak-analyzer-` followed by the platform: `win32-x64`, `darwin-arm64`, `darwin-x64`,
  `linux-x64` and `linux-arm64`, the names Node gives the operating system and processor. This rests
  on the owner holding the `@dabay6` scope on npmjs.com. The plan's first step has the owner confirm
  that by signing in to npm, before anything else is done; if the scope is not the owner's, the work
  stops and the name comes back to the owner. A scope the owner holds lets nobody else publish under
  it, so the six names, and those of the platforms left for later (design REL-3), are safe from the
  start. The binary, the repository and the command are named `yak-analyzer`.
- **NAME-2** The line that maps `@dabay6` to GitHub Packages is removed from the user's npm
  configuration on each of the owner's machines where yak installs the analyzer (owner). It is
  removed with npm's own command for deleting a setting, never by editing the file by hand. This
  replaces the design spec's fallback of a name the mapping does not catch. A project's `.npmrc`
  does not reach yak's install, which runs outside every project. The owner does the removal, or a
  session with the owner's yes on that machine.
- **NAME-2a** This item removes the line on one machine, the one the install check runs on (CHK-1).
  The owner's other machines are done the same way by the owner when the analyzer is first installed
  there; the README says how (DOC-1). Before the line goes, the builder finds what on that machine
  leans on it and tells the owner, who decides whether to go ahead:
  - Projects. The ones looked at are the checkouts yak has registered on the machine and any folder
    the owner names. One that takes a private `@dabay6` package and has no mapping of its own needs
    one. That is a change in that project's own repository, made its own way; this item does not
    make it and does not wait for it to merge, only for the owner's yes to remove the line.
  - Installs outside any project: packages installed globally with npm, and anything yak itself
    installs from npm into its own folders. Once the line is gone, a private `@dabay6` package can
    be installed outside a project only by naming the registry on the command. On 2026-11-11 nothing
    on the owner's machine was installed that way.

### The binary

- **BIN-1** The binary is named `yak-analyzer`. Given its version option, it prints one line,
  `yak-analyzer <version>`, on standard output and exits 0. This is the probe (design IF-1).
- **BIN-2** Given anything else, or nothing, it writes one line on standard error that says what it
  was given and names the version option, prints nothing on standard output, and exits non-zero.
- **BIN-3** The version it prints is the crate's version, read at build time, never written twice.
- **BIN-4** It reads no file, starts no program and uses no network.

### Toolchain and gates

- **GATE-1** The repository pins one exact stable Rust version in `rust-toolchain.toml`, so every
  machine and every runner builds with the same compiler. `Cargo.lock` is committed, and every
  dependency in `Cargo.toml` is pinned to one exact version, written with `=` (design AN-2 asks this
  of the parser libraries a later item adds; here it holds for all).
- **GATE-2** Three gates pass before a change is reviewable, each one command written in
  `CONTRIBUTING.md`: `cargo fmt --check` for formatting, `cargo clippy` with warnings as errors for
  lint, and `cargo test` on a release build, the build that is packed. The launcher's tests are a
  fourth command, run with Node's own test runner.
- **GATE-3** The Rust tests start the built binary and check BIN-1 and BIN-2 by their output and
  exit code. One test checks that every place the version is written agrees (PKG-8), and another
  that the main package's optional dependencies are exactly the five platform packages, by name
  (PKG-3). So a mismatch or a missing platform fails a pull request, not a release.
- **GATE-4** The skeleton keeps its dependencies to the fewest that do the job. A dependency added
  only to parse one option needs a reason in the plan. The launcher's tests need no package beyond
  Node itself. No test can show this or BIN-4; the plan lists each dependency with its reason, and a
  reviewer reads the binary's code for a file, a process or a network call.

### Continuous integration

- **CI-1** One workflow runs on every pull request and every push to `main`. Its jobs are these. One
  job packs the main package, once (PKG-9). The per-target job runs once for each of the five
  targets (PKG-2), on a runner of that target's own operating system and processor: it checks
  formatting and lint, builds the binary once as a release build, runs the tests on that build, runs
  the launcher's tests and the packed install (PKG-9), tries the publishing without publishing
  (PUB-10), and keeps its packed files with the run, for PUB-2. The file the tests start is the file
  that is packed. Those two jobs run on pull requests and on pushes. The release-please job, the
  publishing job and the job that checks the published packages (PUB-9) run only on a push to
  `main`. The runners are GitHub's own, which are free for a public repository.
- **CI-1a** CI runs the launcher's tests and the packed install on Node 22.18, the oldest Node yak
  supports (PKG-5), written in the workflow. It is a floor kept on purpose, so no pin pull request
  moves it (PUB-7). The publishing job runs on a newer Node and npm, since trusted publishing needs
  a newer npm than Node 22.18 carries; that version is written in the workflow too, and is moved by
  hand when npm's publishing rules ask for it.
- **CI-2** On a pull request only the per-target job runs, with a read-only token and no secret. No
  workflow uses `pull_request_target`, or any other trigger that runs a fork's code with the
  repository's rights.
- **CI-3** Each job asks for the least permission it needs, stated in the workflow.
- **CI-4** Every action a workflow uses is pinned to a full commit hash, with its version in a
  comment beside it. A tag can be moved to other code after the fact; a hash cannot.

### The npm packages

- **PKG-1** The main package, named by NAME-1, has the launcher as its `bin`, under the name
  `yak-analyzer` (design REL-1).
- **PKG-2** There is one platform package for each target: Windows x64, macOS arm64, macOS x64,
  Linux x64 and Linux arm64, the two Linux ones for glibc (design REL-3). Each states its operating
  system, its processor and, on Linux, its C library in its `package.json`, so npm installs only the
  one that matches.
- **PKG-2a** A binary runs on any machine that can run the Node yak needs, and asks for nothing
  more. On Linux that means a glibc no newer than the oldest one Node 22.18's own builds run on,
  2.28 when this was written; the builder confirms the number. On macOS it means the oldest version
  that Node supports. On Windows the binary carries its C runtime inside it, so no separate runtime
  has to be installed. A build made on a runner takes that runner's own libraries as its floor
  unless told otherwise, and the runner's own test cannot notice, so each per-target job reads what
  the built binary requires and fails when it is above the floor.
- **PKG-3** The main package lists every platform package as an optional dependency at the one exact
  version released with it (design REL-1).
- **PKG-4** No package has an install script of any kind (design REL-1).
- **PKG-5** The launcher finds the platform package's binary, runs it with the arguments, standard
  input, standard output and standard error it was given, and exits with the binary's exit code.
  When the binary is ended by a signal and so has no exit code, the launcher writes one line naming
  the signal and exits non-zero. It prints nothing of its own otherwise. It answers no command
  itself, so the version the probe reads is printed by the binary (design REL-1). yak starts a tool
  by running its `bin` file with the Node that runs yak, not through the small wrapper script npm
  writes for a `bin` at install. So the launcher is one plain script that runs on the oldest Node
  yak supports, 22.18, and the main package says so in its `engines` field.
- **PKG-6** When the launcher cannot run a binary, it writes one line on standard error and exits
  non-zero. There are three cases. Where no package exists for the machine, the line names the
  targets that have one (design REL-3): that is an operating system or processor outside the five,
  and Linux whose C library is not glibc, which the launcher must tell apart from Linux with glibc.
  Where a package exists but is not installed, the line says the package is missing and that
  optional dependencies must not be left out of the install. Where the binary is there and cannot be
  started, the line names the file and gives the system's reason.
- **PKG-7** The launcher's tests cover the three cases of PKG-6 and the ordinary run. They give the
  launcher a made-up platform and a stand-in program in place of the binary, so they run the same on
  every runner.
- **PKG-8** The crate and every package have one version number. It is written in the crate's
  manifest, the lock file, the six package manifests, and the main package's list of optional
  dependencies. The release pull request (PUB-1) changes all of them together, so the version is in
  the commit that is tested, and nobody types it by hand.
- **PKG-9** The packages are put together and run the way npm and yak will, before anything is
  published. The main package is packed once, in a job of its own, into the file npm would publish;
  it holds no binary, and every target then tests that very file. On each target's runner, CI packs
  that target's platform package the same way, installs the two files with npm into an empty folder,
  and runs the main package's `bin` file with Node and the version option, as yak does. It passes
  when the line that comes back is BIN-1's, with the version the packages carry. The install fetches
  nothing it was not handed: it runs with install scripts off and with no registry to reach, so the
  four other platform packages are simply absent, as they are on a user's machine. Their names may
  not be owned by anyone yet, and a stranger's package must never be installed, let alone run, in
  the job whose files are published. This shows that npm accepts each platform package on its own
  platform, that the launcher finds the binary inside a real install, and that the binary still runs
  after packing. It does not show that the main package's optional dependencies bring the platform
  package in; GATE-3 checks the list, and PUB-9 the real thing.
- **PKG-10** Every package carries a description, keywords, the repository, a home page, where to
  report a bug, and the license. So does `Cargo.toml`.

### Releasing

- **PUB-1** A release is cut by merging the release pull request. That merge, and the approval
  PROT-2 asks for, are the owner's whole part, once PUB-6's setup is done. The first version is
  0.1.0; if its release fails for a fault (PUB-3), the first version published is the next one.
- **PUB-1a** The release pull request gets a real CI run, so it can be merged with no step beyond
  the merge. It needs one: the version is compiled into the binary (BIN-3) and the lock file changes
  with it, so this pull request changes what is built. A pull request a workflow opens with its own
  token starts no other workflow, so the plan says how this one's checks run. If that takes a
  credential beyond the workflow's own token, the plan names it and the owner creates it. Updating
  the release pull request means pushing to its branch, so the credential can write this
  repository's contents and pull requests; it has no right over workflows or settings, and reaches
  no other repository. PROT-1 is what keeps it from `main`.
- **PUB-2** A release is one run of CI-1's workflow, on the push to `main` that the merged release
  pull request makes. These things happen in that run. release-please creates the tag and the GitHub
  release. The per-target job runs for all five targets, as on any push. Then the publishing job
  runs, after all five target jobs passed. It takes the packed files those jobs kept (CI-1) and
  publishes them; nothing is built again. The main package it publishes is the one file every target
  tested (PKG-9). So every binary released has run its tests on a runner of its own operating system
  and processor (design REL-4). The design spec allows a target with no free runner to be released
  untested; nothing is built for that here, since all five have a runner, and a release that cannot
  test a target fails.
- **PUB-2a** The publishing job decides whether to publish from two facts that are still true when a
  job is run again: a tag for the version in the tree points at the very commit this run built, and
  that version is not yet whole on npmjs.com. It runs on every push to `main`, after that run's
  release-please job has finished, and does nothing when either is false. It never asks
  release-please whether a release was just made, since that answer is given once and lost on a
  rerun. And it never publishes files built from a commit other than the tag's, so the tag, the
  provenance and the binary always name one commit.
- **PUB-3** When a target's job fails on the release run, nothing is published, and a person decides
  what it was: the owner, or a session with the owner's yes. They run the failed job again from
  GitHub. If it passes, the failure was the runner's, and the publishing job follows with nothing
  more to do. If it fails again, the fault is in the code: the tag and the GitHub release stay, that
  person edits the GitHub release to say the version was not published to npm, and the fix goes out
  as the next version, by the next release pull request. No workflow writes that line. The same job
  passed on the release pull request before the merge, so this is rare. When the release-please job
  itself fails on that run, it is run again; if another push reached `main` first and its run made
  the tag, the publishing job of the run on the tagged commit is run again and publishes (PUB-2a).
- **PUB-4** The packages are published to npmjs.com, public, with provenance (design REL-2). The
  platform packages go first and the main package last, so a main package never exists without its
  platform packages.
- **PUB-5** When the publishing job itself fails part of the way, it is run again from GitHub, on
  the same run. It skips each package version already on npmjs.com and publishes the rest. Nothing
  is ever unpublished or overwritten. A rerun cures a failure that passes with time. A fault in the
  job's own steps cannot be fixed on the tagged commit: the fix is a new commit and goes out as the
  next version. The version left behind may then have platform packages and no main package. That is
  harmless, since nothing installs a platform package except through a main package that names it,
  and the GitHub release is marked as PUB-3 says. For the first release this means PUB-6's token
  stays until all six packages exist, and is made again if it has expired.
- **PUB-6** Releases publish by trusted publishing, so the repository holds no npm token. The first
  release cannot, because npm sets trusted publishing up on a package that already exists. For that
  one release the owner creates an npm access token that expires within days and can publish under
  the `@dabay6` scope and nothing else, where npm can limit it so; where it cannot, the plan says
  what else the token could publish. The owner stores it as a secret of the publishing environment
  (PROT-2), never of the repository at large; the release publishes with it, still with provenance.
  After the install check passes, the owner sets up trusted publishing on each of the six packages
  and deletes the secret. The plan lists these steps for the owner. The builder checks npm's rules
  when the plan is written; if npm then lets a new package start with trusted publishing, the token
  is never made. This item ends with trusted publishing set up and read back, not used: PUB-8 asks
  for no second release to prove it. Its first use is the next release the owner chooses. A failure
  then is a setting to put right and the publishing job to run again (PUB-2a), and nothing is lost
  by it.
- **PUB-7** Pull requests that move each pin are opened without anyone asking: the Cargo
  dependencies, the workflow actions' hashes (CI-4) and the Rust version of GATE-1. Dependabot,
  GitHub's own dependency-update service, does this from a configuration file, and its pull requests
  start CI with a read-only token, which fits CI-2. The builder checks that it can move the Rust
  version, and says who does where it cannot. CI decides whether one can merge; none merges by
  itself. This is where the parser libraries' pins will move once a later item adds them (design
  REL-6).
- **PUB-8** The first release is the only one this item needs. Nothing installs the analyzer from
  npm before the swap, and every published version is public for good, so each later release is the
  owner's choice when its release pull request is open.
- **PUB-9** After publishing, each target's runner installs the main package by its name and version
  from npmjs.com into an empty folder and runs the probe through the launcher. It needs no secret.
  This is the one place the published packages are installed as a user's npm will install them, on
  all five targets. A failure here cannot stop a publish that is done; it fails the run, the owner
  marks that version deprecated on npmjs.com with a line saying why, and the fix goes out as the
  next version. The job runs on Node 22.18, waits and tries again for a few minutes while npmjs.com
  catches up, and runs whenever the publishing job found the tag on this run's commit, so running it
  again checks again.
- **PUB-10** The publishing steps are tried on every pull request and push, without publishing: each
  per-target job runs npm's publish in its dry-run form on its packed files, and the main package's
  job does the same. They use the Node and npm the publishing job uses (CI-1a), since that is the
  npm that will publish. Between releases the version in the tree is one already on npmjs.com, so
  the dry run must pass there too: if npm's dry run refuses a version that exists, the step counts
  that one refusal as a pass and fails on anything else. So the first real release is not the first
  time those steps run. What a dry run cannot show, the rights and the registry's answer, is what
  PUB-6 and PUB-9 are for.

### Who can change `main`, and who can publish

- **PROT-1** A rule on `main` lets it change only by a pull request whose five per-target checks
  have passed, and nobody pushes to it directly. The rule also names who may merge: the owner, and
  no credential a workflow holds. The organization this repository belongs to already requires pull
  requests on default branches; this adds the required checks and the limit on who merges. Without
  it, "CI decides" and "the owner merges" are habits and not facts.
- **PROT-1a** A rule on the release tags lets only the release-please job create one, and lets
  nobody move or delete one, since the publishing job trusts the tag (PUB-2a).
- **PROT-2** The publishing job runs in a GitHub environment that only `main` can use and that asks
  the owner to approve each run before it starts (owner). The first release's token is that
  environment's secret, and npm's trusted publishing is set up to trust that workflow in that
  environment. So a pull request, from a fork or from a branch of this repository, cannot reach
  either, whatever its copy of the workflow says; and even a merge the owner did not make publishes
  nothing until the owner approves. Any credential PUB-1a needs is kept the same way, in an
  environment only `main` can use.
- **PROT-3** PROT-1, PROT-1a, PROT-2 and PUB-6 are settings on GitHub and npmjs.com, not files. The
  plan lists each as a step, says who does it, and has it read back once set.

### The install check

- **CHK-1** After the first release, yak installs the released main package as a tool server and
  runs its probe, on the owner's machine, once NAME-2's line is gone from it (design REL-2). NAME-1
  settled the name and PKG-9 the packaging; this confirms what neither can: the registry, and yak's
  own install.
- **CHK-2** The builder makes two folders for the check, outside every repository, and removes both
  after, with the install the check leaves in yak's tools folder. yak skips an install it has
  already made, so a check that is repeated starts with that install folder removed. One is a pack:
  a `pack.json` that names it, and one file, `tools/yak-analyzer.md`, that declares the released
  version as a tool server:

  ```markdown
  ---
  summary: The released yak-analyzer, for the install check
  install:
    kind: npm
    package: <the main package's name>
    version: <the released version>
  probe: [yak-analyzer, --version]
  serve: [yak-analyzer]
  ---
  Made for one install check and removed after it.
  ```

  The `probe` line uses whatever the version option is named. A pack must give a tool server a
  `serve` command, the one that starts its server; the skeleton has no server, so that line is a
  stand-in. The other folder is a new project that holds one file, an `.npmrc` mapping `@dabay6` to
  GitHub Packages as the owner's projects do, to show that a project's mapping does not reach yak's
  install. The pack is added to it with `yak pack add --project path:<the pack's folder>`, run
  inside it. No agent program is pointed at that project, so nothing ever starts the stand-in. The
  builder checks the pack's exact shape against the yak version in use when the plan is written. The
  owner runs `yak tools install` in that project at a terminal, since yak asks a person to confirm
  every install. yakshaver's own pack is not changed.
- **CHK-3** The check passes when the install succeeds, which means the probe printed the released
  version; the platform package for that machine is in the install folder; and every installed
  package came from npmjs.com. That is read from `node_modules/.package-lock.json` in the install
  folder, `tools/yak-analyzer/<version>/` under yak's home, where npm records the address each
  package was fetched from. yak's own npm command leaves no optional dependency out and turns no
  install script off; only a machine's npm settings could, and the launcher says so when they do
  (PKG-6).
- **CHK-4** The result, with the registry each package came from, is written on the work item's
  record. No later release is made before it is. A failed check stops the work until the owner has
  decided what to change.

### What is shown before the merge, and what after

- **ORDER-1** The release tooling works on `main`, so this item's work is shown in two parts. Before
  its pull request merges, the owner can check the binary, the gates, CI on all five targets and the
  packed install (BIN, GATE, CI, PKG). The release pull request, the publishing and the install
  check (PUB, CHK) can only run after that merge. The item stays open until CHK-4 is on the record,
  and the plan says how the work after the merge is carried and checked.

### The repository's own text

- **DOC-1** `CONTRIBUTING.md` gains the toolchain, how to install it, and the gate commands.
  `README.md` gains how to install and run the released binary, and its status section says what
  exists: a binary that answers its version and analyzes nothing yet. `CONTRIBUTING.md`'s rule on
  private names is reworded to match the anti-goal above: it reads today as if it barred naming
  yakshaver, which that file itself names. The README's install section says that a user-level npm
  setting mapping `@dabay6` to another registry must be removed first, and how.
- **DOC-2** A `CLAUDE.md` points an agent session at `CONTRIBUTING.md` for the rules, repeating none
  of them, and holds only what a session needs beyond them: where the design spec is, and that a
  work item here names the design spec's requirements it covers.

## Alternatives considered

- **A name the mapping does not catch:** a scope of the analyzer's own, or no scope at all. This is
  the design spec's own fallback, and no machine's settings would have to change. The owner chose to
  keep one scope for every package: the mapping is one line, the projects that need it carry their
  own, and it goes away altogether when the owner's private packages are public.
- **Publishing the analyzer to GitHub Packages as well.** yak would find it where the mapping
  points. GitHub Packages asks for a token on every read, and the design spec wants an install that
  needs none.
- **Reserving the six names with placeholder versions published by hand,** so that even the first
  release could use trusted publishing. Six versions that do nothing would be public for good, with
  no provenance. A token that lives for days does the same job and leaves nothing behind.
- **A release tool that writes the npm installer for a Rust binary.** The common ones install by
  downloading the binary in an install script, which the design spec rules out (design REL-1): yak
  asks a person's yes for an install, and a script that fetches code after it is a second, unseen
  step.
- **Cross-compiling every target on one runner.** One job instead of five, and no dependence on
  which runners GitHub offers. The Windows and macOS binaries would ship without ever running
  (design REL-4).
- **Testing three targets on pull requests and all five only at release.** Fewer runner minutes on
  each pull request. Two targets would first run after the owner's merge, when the version's tag
  already exists, and there would be two build setups to keep in step.
- **Building the design spec's allowance for an untested target now.** Every target has a runner
  today, so it would be a second path through the release that never runs. The change that first
  needs it adds it.
- **One package holding all five binaries.** Simpler to publish, and no optional dependencies to go
  missing. Every install would carry four binaries it cannot run, and the design spec chose the
  layout other tools shipped this way use.
- **Publishing by a tag pushed by hand.** No release tooling to configure. The owner would have to
  remember the version and the tag, where a release pull request shows what is going out before the
  merge.
- **Starting with the analysis and adding the release later.** The first useful code would come
  sooner. The five-target build and the publishing would then be tested for the first time with real
  work depending on them.
- **Publishing as soon as the release pull request is merged, with no approval.** One step fewer for
  the owner. Whatever can merge could then publish, and the credential that keeps the release pull
  request up to date is one such thing.
- **Answering the interface version now.** The design spec lists it among stage 1's answers. Nothing
  reads it until a later item, and the number would mean nothing yet.

## Risks

- **A name once published cannot be taken back.** npm lets a package be deprecated, not removed.
  Handled by the owner's choice of the name before the plan (NAME-1), and by the plan's first step,
  which confirms the scope is the owner's before anything is published.
- **A token that can publish exists for the first release.** Handled by PUB-6 and PROT-2: it expires
  within days, it is limited to the scope where npm allows, only the publishing environment on
  `main` can read it, and it is deleted once the install check passes.
- **A platform package is not yet published when CI installs.** Handled by PKG-9's install fetching
  nothing and running no script, and by the scope: nobody but the owner can publish under `@dabay6`.
- **The published files are built from a commit other than the tagged one,** or a rerun skips
  publishing for good. Handled by PUB-2a.
- **The binary needs newer system libraries than a user's machine has.** Handled by PKG-2a's floor
  and its check in every per-target job.
- **A credential or a pull request changes `main` or publishes without the owner.** Handled by
  PROT-1, PROT-1a and PROT-2: a rule on who merges, a rule on the tags, and the owner's approval of
  every publishing run.
- **The packaging is wrong in a way only an install shows.** Handled by PKG-9, which installs the
  packed files on every target before any publish, and by PUB-9, which installs the published
  packages by name on every target.
- **A release fails after its tag exists.** Handled by running the same job on the release pull
  request before the merge (PUB-1a), and by PUB-3 saying what is left behind and who acts.
- **A release fails part of the way through publishing.** Handled by the publishing order (PUB-4)
  and by a rerun that skips what is published (PUB-5). The steps are tried without publishing on
  every pull request (PUB-10), and PUB-5 says what a fault in them leaves.
- **GitHub retires a runner a target needs.** The macOS x64 runners are the likely case. Releases
  then fail (PUB-2) until a change either drops the target or builds the design spec's allowance for
  an untested one.
- **The platform package is left out of an install,** by an npm setting that omits optional
  dependencies. Handled by the launcher's line (PKG-6).
- **A fork's pull request runs code in CI.** The repository is public. Handled by CI-2 and CI-3.
- **A compromised action or dependency reaches a release.** Handled in part: exact pins, a committed
  lock file, pull requests for every pin change (PUB-7), least permission (CI-3), no install
  scripts, and provenance. Nothing in yak checks provenance.
- **The binary and the package disagree on the version.** Handled by one change that writes the
  version everywhere (PKG-8), a test that they agree (GATE-3), and the packed install comparing the
  printed version with the packages' (PKG-9).
- **A machine still maps `@dabay6` in the user's settings.** yak's install of the analyzer fails
  there, with npm's not-found answer from the other registry. Handled for the owner's machines by
  NAME-2 and the install check; the README says what to remove (DOC-1).
- **A project that maps `@dabay6` to GitHub Packages cannot take the analyzer as a dependency of its
  own.** Not handled, and not needed: yak installs it outside every project.

## Left to the builder

- The name of the version option.
- The exact Rust version, the crate layout, and whether the option is parsed by hand.
- How the launcher tells glibc from another C library.
- How each build holds to PKG-2a's floor, and how a job reads what the built binary requires.
- How PKG-9's install is kept from reaching a registry.
- Which runner builds and tests each target, checked against what GitHub offers when the plan is
  written.
- release-please's configuration, how it writes one version into every place of PKG-8, and how the
  release pull request gets its CI run.
- How the work after the merge is carried: a second plan, or steps the first plan leaves for after
  it.
- npm's current rules for trusted publishing and a new package (PUB-6).
- How GitHub's rules express PROT-1 and PROT-1a. Where they cannot do all of it, the plan says what
  stays a habit.
- The pack and the project made for the install check.

## Amendments

### 2026-11-11: Trusted publishing waits for the next release, and the publishing job's Node

PUB-6 asked the builder to check npm's rules when the plan was written. They were read on
2026-11-11, from npm's own documentation on docs.npmjs.com: the pages on trusted publishers and on
staged publishing, and the page of the `npm trust` command. The first is as PUB-6 expected; the
other three change what this spec says:

- Trusted publishing can be set up only on a package that already exists.
- A new setup must complete its first publish within two days, or it has to be deleted and made
  again. PUB-6 ends this item with trusted publishing "set up and read back, not used", which npm
  would void two days later.
- A setup offers two kinds of publishing. **Direct publishing** is what this spec means by
  publishing: the workflow's `npm publish` puts the version on npmjs.com. **Staged publishing**
  holds the version back until a person approves it on npmjs.com. A setup made today allows only
  staged publishing unless direct publishing is also chosen when it is made.
- Trusted publishing needs npm 11.5.1 or later, on Node 22.14 or later. The newest npm, 12, does not
  run on Node 22.18.

Three terms used below. **The token** is the npm access token of PUB-6: made on npmjs.com, and
stored as a secret of the publishing environment on GitHub. **Deleting the token** means both: the
secret is removed from the environment, and the token is revoked on npmjs.com. **The next release**
is the first release after this item has ended. It is not "the next version" of PUB-3 and PUB-5,
which is made inside this item.

The owner decided on 2026-11-11 to leave trusted publishing for the next release, and not to publish
a second version of a binary that does nothing only to prove it.

- **PUB-11** This item sets up no trusted publishing (owner). Every release made inside this item
  publishes with the token: the first, and any made to put a failed one right, whatever failed
  (PUB-3, PUB-5, PUB-9, CHK-4). The owner deletes the token once the install check has passed, and
  the item then ends with no stored right to publish at all. This replaces the part of PUB-6 from
  "After the install check passes" to its end, and PROT-2's words about trusted publishing being set
  up in the publishing environment; the rest of both stands. Other places read differently for this
  amendment:
  - The goal's "setup done once for the first release" is the token alone, and PUB-1's "once PUB-6's
    setup is done" is the same.
  - The goal that a later item "adds analysis and nothing else" still holds for the code; the next
    release also has the owner's steps of PUB-12.
  - PROT-3's list keeps PROT-1, PROT-1a and PROT-2. Of PUB-6 it now holds the token's settings only,
    with no trusted publishing among them.
  - PUB-5 is read together with PUB-14.
  - PUB-10's "(CI-1a)" is now CI-5.
  - DOC-1's list of what `CONTRIBUTING.md` gains has one more entry, the section on releasing
    (PUB-12, PUB-14).
  - The risk "A token that can publish exists for the first release" holds for a rescue too
    (PUB-14), with the same handling.
  - The builder's open question on npm's rules for a new package is answered above. The open
    question of how the work after the merge is carried (ORDER-1) is settled too: by a second plan,
    which builds the release.
- **PUB-12** The owner sets up trusted publishing for the next release while its publishing job
  waits for the owner's approval (PROT-2): the release pull request is merged, the five target jobs
  have passed, and nothing is published yet. So npm's two days start with the publish only an
  approval away, and a setup is never made for a release whose tests then fail. Before that release
  pull request is merged, the owner, or a session for the owner, reads npm's rules again, since they
  have changed twice in a year; if a rule has changed so that these steps no longer work, the
  release waits and the change comes back to this spec as an amendment. The setup is made on each of
  the six packages. It names this repository, the workflow's file and the publishing environment. It
  chooses direct publishing, since staged publishing would add a second approval to every release,
  where PROT-2 already asks for one. And it leaves each package accepting a token as well: npm lets
  a package refuse tokens once trusted publishing is set up, and that choice is not made, so
  PUB-14's rescue stays open. Each setting is read back before the owner approves. Until a release
  has been published by trusted publishing, these steps are done for every release: a setup that
  never published is voided by npm, and the voided one is deleted before a new one is made. After
  one release has published that way, the setup stands. `CONTRIBUTING.md` gains these steps and
  PUB-14's, under a heading of their own for releasing.
- **PUB-13** The publishing job needs no change for that release. When the publishing environment
  has the token's secret, the job publishes with it; when the environment has none, the job
  publishes by trusted publishing. Its log says which. The job cannot tell a token that has expired
  from one that works: an expired one fails the publish with npm's own message, and the owner
  deletes it or replaces it. The second plan says how the job chooses. The trusted-publishing way
  asks one thing of the workflow that the token's way does not: the publishing job's right to ask
  GitHub for an identity token, which the job is given from the start. That way cannot run in this
  item, since it needs packages that exist and a release to publish. So the builder checks it by
  reading the workflow against npm's page: that the job has that right, and that the workflow's file
  and the environment are named as PUB-12 will give them to npm. The next release is its first real
  use. The owner accepted this over a second release now.
- **PUB-14** The token's way stays in the job after this item, for one purpose: to save a release
  that the trusted-publishing way fails. A wrong setting on npmjs.com is put right and the job run
  again (PUB-2a). When that does not cure it, the owner makes a new token as PUB-6 describes one,
  stores it in the publishing environment, and runs the job again on the same tagged commit. The job
  then takes the token's way, which the first release proved, so the version is published; the owner
  then deletes the token. So a fault that lies only in the trusted-publishing way costs a rerun and
  not a version. A fault in steps both ways share is PUB-5's case, and the fix still goes out as the
  next version. A release saved this way was not published by trusted publishing, so PUB-12's steps
  are done again for the one after it. A token in the environment wins over trusted publishing for
  as long as it is there, which is why it lives for days at most and is deleted after its one use;
  the log's line (PUB-13) is how a forgotten one shows.
- **CI-5** The publishing job runs on Node 22.18, as every other job does, with npm fetched by `npx`
  at one exact version of 11.5.1 or later that runs on that Node. It is the npm the dry run uses
  (PUB-10): the publish and the dry run are one script run two ways, today's
  `scripts/dry-run-publish.mjs` made to do both, so the npm's version, the registry and the answer
  for a version already published are written once, in that script. `npx` is the program, shipped
  with npm, that fetches a package at a named version and runs it. The npm's version is moved by
  hand, by whoever makes a release whose publishing needs a newer npm; no pin pull request moves it.
  If npm's rules come to need an npm that no longer runs on Node 22.18, the publishing job and the
  dry run move to a newer Node together, in a change of its own. This replaces CI-1a's sentence that
  the publishing job runs on a newer Node and that its version is written in the workflow; CI-1a's
  floor of Node 22.18 for the tests and the packed install stands.

One risk is added by this amendment. **The trusted-publishing way is first used by a real release.**
The dry run does not cover it: it uses no credential and writes no provenance, so it shows only what
the two ways share (PUB-10). Handled by PUB-12, which has each setting read back before the
approval, and by PUB-14, which saves the release with a token when that way fails.
