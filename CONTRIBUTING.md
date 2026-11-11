# Contributing

There is no code here yet, so there is no build or test command to give. This file holds the
rules that are already decided; the toolchain and the commands are added with the first stage of
the work.

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
- **No private names.** Nothing committed here names a private repository or copies its code,
  including the repositories the analyzer is checked against.
- **It only reads.** A change that makes the analyzer run a project's code, write inside a
  checkout, or use the network is not accepted.
- **Dependencies are pinned to exact versions.** oxc changes in ordinary weekly releases.

## Commits and pull requests

- Commits follow [Conventional Commits](https://www.conventionalcommits.org/):
  `type(scope): description`, in the imperative, with no trailing period.
- Work reaches `main` through a pull request, from a topic branch.
- Pull requests are merged with a merge commit or a rebase, never a squash, so each commit's type
  stays visible to the release tooling.

## License

By contributing you agree that your contribution is licensed under the [MIT license](LICENSE).
