# yak-analyzer

A static analyzer for TypeScript and JavaScript, built for
[yakshaver](https://github.com/dabay6/yakshaver). It is being written to take the place of the
third-party analyzer yakshaver wraps today, so that yakshaver depends on an interface it designs.

## Status

Nothing is built yet. This repository was created on 2026-11-11 to hold the work. The design is
written and approved; the first stage has not started, and there is no release.

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
