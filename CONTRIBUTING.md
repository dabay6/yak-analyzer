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

- Commits follow [Conventional Commits](https://www.conventionalcommits.org/):
  `type(scope): description`, in the imperative, with no trailing period.
- Work reaches `main` through a pull request, from a topic branch.
- Pull requests are merged with a merge commit or a rebase, never a squash, so each commit's type
  stays visible to the release tooling.

## License

By contributing you agree that your contribution is licensed under the [MIT license](LICENSE).
