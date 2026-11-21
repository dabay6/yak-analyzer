# Dead-code findings, part 6: what a tool's configuration names, the repository's own text, and the check on the named repositories: implementation plan

**Goal:** a glob a tool's configuration file states, and a word of a script, name the files the tool
really loads; the README and the contributing guide say what the analyzer now reports; and the check
on the named repositories shows no finding known to be false.

**Architecture:** Task 1 changes how `src/entries.rs` turns a `configEntries` value into files: each
glob is joined to its folder and resolved from the root as the tool reads it, and one that matches
nothing is mapped back from build output with the code ENTRY-3 already uses; a script's word is
mapped back by the package it names (`Resolver::script_target`). Task 2 changes `README.md`,
`CONTRIBUTING.md` and the two tests that hold them. Task 3 is the owner's check of the live run,
whose results stay in the item's report folder.

**Spec:** docs/specs/2026-11-13-dead-code-findings-design.md

**Implements:** ENTRY-6b, ENTRY-6c, ENTRY-6d, ENTRY-7a, DOC-1, LIVE-1 to LIVE-8

The last of the item's six plans, listed in plan 1. Plan 5 has merged, so this plan's branch,
`feat/dead-code-6-live-check`, is made from `main`, and its pull request goes against `main`. The
four requirements ENTRY-6b to ENTRY-7a are the spec's amendment of 2026-11-15, `amendment:1` on the
record, written from what the check found.

## Global constraints

- Nothing committed names a machine, or a private repository other than yakshaver (the
  repository's rule): the named repositories, their files, the hand-off files and the judged lists
  stay in the report folder, `.git/yak-reports/2026-11-14-dead-code-findings/live/`, which is
  committed nowhere (LIVE-7).
- The live check runs on clean worktrees with nothing installed and nothing built, made with git in
  a temporary folder outside every repository, and removed when their runs are judged (LIVE-1).
- A false finding becomes a made-up fixture of its shape before it is fixed; fixtures copy no code
  from the repositories (LIVE-6).
- A glob read from a configuration file never stops a run; one passed over gives the notice
  `config-glob-not-read` (ENTRY-6, ENTRY-6b).
- A pack's own globs in `configEntries` (`file`, `pattern`, `default`) still may not step out
  (ENTRY-6b, reading KEY-2).
- A speed miss is reported to the owner, not fixed here (LIVE-5).

## The decisions this plan makes

1. **The check was run while this plan was written,** on 2026-11-15, first with plan 5's build
   (`e3788ac`) and then with this plan's Task 1 built, and every finding was judged. Task 3 runs it
   again with the branch's own build before handing over, and compares.
2. **The repositories and roots** are those the reading item's check covered, from the design item's
   record: four repositories, each at its local default branch in a detached worktree in one
   temporary folder, and the two web apps in sub-folders run a second time with the sub-folder as the
   root: six roots.
3. **The hand-off files** hold two stacks: `dead-code-tools`, with the pack's declarations only
   (`configEntries`, `scriptCommands`, and `entries` for a tool's fixed file names), and
   `typescript`, with the project's only (`data`, `generated`, `entries`, `internal`), as LIVE-2
   keeps them apart. Each declaration's reason and match count are in the report.
4. **The branch audits** (LIVE-3) use, per repository, the latest merged pull request that changes
   code, named with its commits in the report: three kept their last commit as a merge's second
   parent, and one was squashed.
5. **Resolving a configuration glob** (ENTRY-6b, ENTRY-6d): the glob is joined to its folder and
   resolved from the root segment by segment, `.` dropped and `..` taking off the segment before it,
   until the first segment that holds a wildcard; a `..` after that, an absolute glob, and one that
   would climb above the root are passed over. The glob is then matched from the root, so a folder
   value and a globs value are resolved alike.
6. **Mapping a configuration glob back** (ENTRY-6c) reuses `mapped_globs`, the function that maps an
   `entries` glob (ENTRY-3), on the resolved glob with the root as its base, so the two follow the
   same rules.
7. **A script's word** is mapped back by a new `Resolver::script_target`, which takes the folder of
   the package the joined path lies in. `package_target` stays as it is for `main`, `bin` and
   `exports`, which name their own package's files.
8. **The fixtures** join `tests/entries.test.rs`, beside the configuration and script fixtures they
   extend.
9. **The time** (LIVE-5) is the median of three warm runs after one more, in each branch worktree,
   one root at a time with nothing else running: `graph` for the working tree's reading, `audit`
   without and with the base. The rest is the difference. The check's first runs, made four at a
   time and cold, took up to 43 s, and are not the measure.
10. **The counts and the times go on the record** as a ruling on the plan, since the engine refuses a
    ruling on a live-check task, as the reading item found.
11. **The worktrees are removed once the owner has validated the plan,** since a declaration the
    owner rejects means a run again (LIVE-1).
12. **The README** also says that an audit with a base in a partial clone needs git 2.44, which plan
    5's stop names (COMMIT-2).
13. **Left as they are:** the ENTRY-8 hint on a package whose `bin` or `exports` target maps to no
    source, which on one repository pointed away from the real cause until Task 1 cured it, and the
    `glob-matches-nothing` notice a pack's `entries` glob gives in a detection folder without the
    tool. Both are the spec's wording, reported for stage 2.

## Review focus

- **A configuration value written with backslashes** (`.\\setup.ts`) is not a glob KEY-5 reads as a
  path, and is passed over with a notice; no repository of the check has one.
- **A folder value that steps out** (`value: "folder"` with `../shared`) is resolved by the same
  function as a globs value; Task 1's fixture covers globs only. The function has one path for both.
- **A default that names build output** is mapped back too (ENTRY-6c says so); the fixture's default
  matches nothing either way and checks that it says nothing.

### Task 1: a tool's configuration and a script's word, read as the tool reads them

**Implements:** ENTRY-6b, ENTRY-6c, ENTRY-6d, ENTRY-7a, LIVE-6

**Test:** `cargo test --release --locked --test entries`

**Files:**
- Modify: `src/entries.rs`, `src/resolve.rs`
- Test: `tests/entries.test.rs`

**Interfaces:**
- Consumes: `Declared` and `mapped_globs` in `src/entries.rs`; `Resolver::target` and
  `Resolver::package_folder` in `src/resolve.rs`.
- Produces: `Declared::globs(tree, path, folder, notices)`, which returns globs from the root;
  `entries::resolved(folder, glob)`; `Resolver::script_target(package, word)`.

- [ ] **Step 1: Write the failing tests**

```rust edit path=tests/entries.test.rs
<<<<<<< find
    );
}

/// The uses a document holds that count from an entry point: each with the file whose exports are
/// used, the export's name, `*` for every export, and the field or declaration that made the entry
/// point one.
=======
    );
}

#[test]
fn a_configuration_files_globs_are_relative_as_the_tool_reads_them_dot_and_dot_dot_included() {
    let project = Project::new("entries-configured-relative");

    project
        .write(
            "apps/book/.storybook/main.ts",
            "export default {\n\
               stories: ['../src/**/*.stories.ts', '../../../../far/*.ts', 'docs/**/../*.mdx'],\n\
             };\n",
        )
        .write("apps/book/src/a.stories.ts", "")
        .write("apps/book/src/deep/b.stories.ts", "")
        .write(
            "packages/a/vitest.config.ts",
            "export default {\n\
               test: { setupFiles: ['./setup.ts'], globalSetup: ['../../shared.mjs'] },\n\
             };\n",
        )
        .write("packages/a/setup.ts", "")
        .write("shared.mjs", "")
        .commit();

    let declarations = json!({
        "configEntries": [
            {
                "file": ".storybook/main.{ts,js}",
                "property": ["stories"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            },
            {
                "file": "vitest.config.{ts,js}",
                "property": ["test", "setupFiles"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            },
            {
                "file": "vitest.config.{ts,js}",
                "property": ["test", "globalSetup"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            }
        ]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "apps/book", "packages/a"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // `./` names the folder the value is relative to, and a `..` climbs from it while it stays inside
    // the root (spec ENTRY-6b, ENTRY-6d).
    assert_eq!(
        entries(&document),
        [
            ("apps/book/.storybook/main.ts", vec![configured(None)]),
            (
                "apps/book/src/a.stories.ts",
                vec![configured(Some("apps/book/.storybook/main.ts"))]
            ),
            (
                "apps/book/src/deep/b.stories.ts",
                vec![configured(Some("apps/book/.storybook/main.ts"))]
            ),
            (
                "packages/a/setup.ts",
                vec![configured(Some("packages/a/vitest.config.ts"))]
            ),
            ("packages/a/vitest.config.ts", vec![configured(None)]),
            (
                "shared.mjs",
                vec![configured(Some("packages/a/vitest.config.ts"))]
            ),
        ]
    );

    // A glob that would leave the root, and one with a `..` after a wildcard, are passed over.
    assert_eq!(
        notices(&document),
        ["config-glob-not-read", "config-glob-not-read"]
    );
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The glob `../../../../far/*.ts` in apps/book/.storybook/main.ts is not one the analyzer reads, so \
             it adds no entry point."
        )
    );
}

#[test]
fn a_configuration_files_glob_of_build_output_names_the_source_it_is_built_from() {
    let project = Project::new("entries-configured-built");

    project
        .write("packages/ui/package.json", r#"{ "name": "ui" }"#)
        .write(
            "packages/ui/tsconfig.json",
            r#"{ "compilerOptions": { "rootDir": "src", "outDir": "out" } }"#,
        )
        .write(
            "packages/ui/runner.config.mjs",
            "export default { files: 'out/**/*.test.js' };\n",
        )
        .write("packages/ui/src/a.test.ts", "")
        .write("packages/ui/src/deep/b.test.ts", "")
        .write("packages/ui/src/c.ts", "")
        .commit();

    let declarations = json!({
        "configEntries": [
            {
                "file": "runner.config.{js,mjs}",
                "property": ["files"],
                "value": "globs",
                "relativeTo": "config",
                "default": []
            },
            {
                "file": "runner.config.{js,mjs}",
                "property": ["unstated"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["checks/**/*.check.js"]
            }
        ]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/ui"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // The tests the runner runs once built are their sources, mapped back through the package's
    // `outDir`; a default that matches nothing adds nothing and says nothing (spec ENTRY-6c).
    assert_eq!(
        entries(&document),
        [
            ("packages/ui/runner.config.mjs", vec![configured(None)]),
            (
                "packages/ui/src/a.test.ts",
                vec![configured(Some("packages/ui/runner.config.mjs"))]
            ),
            (
                "packages/ui/src/deep/b.test.ts",
                vec![configured(Some("packages/ui/runner.config.mjs"))]
            ),
        ]
    );
    assert_eq!(notices(&document), Vec::<&str>::new());
}

#[test]
fn a_root_script_that_runs_another_packages_build_output_names_that_packages_source() {
    let project = Project::new("entries-scripts-elsewhere");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "scripts": { "docs": "node packages/a/out/run.js" } }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write(
            "packages/a/tsconfig.json",
            r#"{ "compilerOptions": { "rootDir": "lib", "outDir": "out" } }"#,
        )
        .write("packages/a/lib/run.ts", "")
        .commit();

    // The word is mapped back by the `outDir` of the package it names, not by the root's, which has
    // none (spec ENTRY-7a).
    assert_eq!(
        entries(&project.graph()),
        [(
            "packages/a/lib/run.ts",
            vec![named("scripts", "package.json")]
        )]
    );
}

/// The uses a document holds that count from an entry point: each with the file whose exports are
/// used, the export's name, `*` for every export, and the field or declaration that made the entry
/// point one.
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test entries
```

Expected: FAIL: the three new tests fail, since a `./` or `..` value and a glob of build output name
nothing, and a root script's word is mapped back by the root; the other 11 pass.

- [ ] **Step 3: Write the code**

```rust edit path=src/entries.rs
<<<<<<< find

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::hand_off::{HandOff, leaves_its_folder};
use crate::notice::Notice;
use crate::packages::{Package, Packages};
use crate::resolve::{self, Outcome, Resolution, Resolver};
=======

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::hand_off::HandOff;
use crate::notice::Notice;
use crate::packages::{Package, Packages};
use crate::resolve::{self, Outcome, Resolution, Resolver};
>>>>>>> replace
<<<<<<< find

                    for path in project
                        .resolver
                        .package_target(package, named)
                        .into_iter()
                        .flatten()
                    {
=======

                    for path in project
                        .resolver
                        .script_target(package, named)
                        .into_iter()
                        .flatten()
                    {
>>>>>>> replace
<<<<<<< find
                    ..source.clone()
                };

                for glob in declaration
                    .globs(project.tree, &config.path, notices)
                    .iter()
                    .filter_map(|written| Glob::new(written).ok())
                {
                    for found in project.files.iter().filter(|file| {
                        inside(folder, &file.path).is_some_and(|rest| glob.matches(rest))
                    }) {
                        entries.add(found.path.clone(), named.clone());
                    }
                }
            }
=======
                    ..source.clone()
                };

                // A glob that matches no listed file is mapped back from build output, and one that
                // matches nothing either way adds nothing, quietly (spec ENTRY-6c).
                for written in declaration.globs(project.tree, &config.path, folder, notices) {
                    let Ok(glob) = Glob::new(&written) else {
                        continue;
                    };
                    let mut found: Vec<&str> = project
                        .files
                        .iter()
                        .map(|file| file.path.as_str())
                        .filter(|path| glob.matches(path))
                        .collect();

                    if found.is_empty() {
                        let mapped: Vec<Glob> = mapped_globs(&written, "", project)
                            .iter()
                            .filter_map(|mapped| Glob::new(mapped).ok())
                            .collect();

                        found = project
                            .files
                            .iter()
                            .map(|file| file.path.as_str())
                            .filter(|path| mapped.iter().any(|glob| glob.matches(path)))
                            .collect();
                    }

                    for path in found {
                        entries.add(path.to_owned(), named.clone());
                    }
                }
            }
>>>>>>> replace
<<<<<<< find
        }
    }

    /// The globs a configuration file's value gives, from the folder the value is relative to: the
    /// value where it is a literal and leaves a glob the analyzer reads, and the declaration's
    /// `default` where it does not.
    fn globs(&self, tree: &Tree, path: &str, notices: &mut Vec<Notice>) -> Vec<String> {
        let stated = tree
            .read(path)
            .ok()
=======
        }
    }

    /// The globs a configuration file's value gives, each joined to the folder the value is relative
    /// to and so from the root: the value where it is a literal and leaves a glob the analyzer reads,
    /// and the declaration's `default` where it does not.
    fn globs(
        &self,
        tree: &Tree,
        path: &str,
        folder: &str,
        notices: &mut Vec<Notice>,
    ) -> Vec<String> {
        let stated = tree
            .read(path)
            .ok()
>>>>>>> replace
<<<<<<< find
        for value in &stated {
            let glob = self.joined(value);

            if readable(&glob) {
                read.push(glob);
            } else {
                notices.push(Notice::new(
                    "config-glob-not-read",
=======
        for value in &stated {
            let glob = self.joined(value);

            if let Some(resolved) = readable(&glob).then(|| resolved(folder, &glob)).flatten() {
                read.push(resolved);
            } else {
                notices.push(Notice::new(
                    "config-glob-not-read",
>>>>>>> replace
<<<<<<< find
        if read.is_empty() {
            self.default
                .iter()
                .map(|value| self.joined(value))
                .collect()
        } else {
            read
=======
        if read.is_empty() {
            self.default
                .iter()
                .filter_map(|value| resolved(folder, &self.joined(value)))
                .collect()
        } else {
            read
>>>>>>> replace
<<<<<<< find
    }
}

/// Whether the analyzer reads a glob a tool's configuration file states: one KEY-5 reads, that is not
/// absolute, does not step out with `..`, does not begin with `!`, and holds no extended pattern such as
/// `?(…)` (spec ENTRY-6).
fn readable(glob: &str) -> bool {
    let extended = ["?(", "*(", "+(", "@(", "!("]
        .iter()
        .any(|opening| glob.contains(opening));

    !glob.is_empty()
        && !extended
        && !glob.starts_with('!')
        && !leaves_its_folder(glob)
        && Glob::new(glob).is_ok()
}

/// The deepest of the folders yak's detection looked in that holds a path, from the root.
=======
    }
}

/// Whether the analyzer reads a glob a tool's configuration file states: one KEY-5 reads, that does not
/// begin with `!`, and holds no extended pattern such as `?(…)` (spec ENTRY-6). Where it may lead is
/// `resolved`'s to say.
fn readable(glob: &str) -> bool {
    let extended = ["?(", "*(", "+(", "@(", "!("]
        .iter()
        .any(|opening| glob.contains(opening));

    !glob.is_empty() && !extended && !glob.starts_with('!') && Glob::new(glob).is_ok()
}

/// A glob a configuration file states, joined to the folder it is relative to and so from the root, as
/// the tool reads it: each `.` dropped and each `..` taking off the segment before it (spec ENTRY-6b,
/// ENTRY-6d). Nothing when the glob is absolute, would leave the root, or holds a `..` after a
/// wildcard, whose folder cannot be known.
fn resolved(folder: &str, glob: &str) -> Option<String> {
    let has_drive = glob.as_bytes().get(1) == Some(&b':');

    if glob.starts_with('/') || has_drive {
        return None;
    }

    let mut segments: Vec<&str> = folder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let mut wild = false;

    for segment in glob.split('/') {
        match segment {
            "" | "." => {}
            ".." if wild => return None,
            ".." => {
                segments.pop()?;
            }
            _ => {
                wild |= segment.contains(WILDCARDS);
                segments.push(segment);
            }
        }
    }

    Some(segments.join("/"))
}

/// The deepest of the folders yak's detection looked in that holds a path, from the root.
>>>>>>> replace
```

```rust edit path=src/resolve.rs
<<<<<<< find
        }
    }

    /// The paths, from the root, where the source of an `exports` target with a `*` may lie: the target
    /// in its source forms, the target itself, then the places MAP-1 maps build output to, each still
    /// holding its `*` (spec ENTRY-2, MAP-3).
=======
        }
    }

    /// The files a word of a package's script names, from the package's folder: the path itself, or the
    /// source of build output, mapped back by the package the path lies in, which may be another
    /// (spec ENTRY-7, ENTRY-7a).
    pub fn script_target(&self, package: &Package, word: &str) -> Option<Vec<String>> {
        let path = joined(folder_path(&package.folder), word)?;

        match self.target(self.package_folder(&path), &path, None) {
            Found::Done(Outcome::Files(files)) => Some(files),
            _ => None,
        }
    }

    /// The paths, from the root, where the source of an `exports` target with a `*` may lie: the target
    /// in its source forms, the target itself, then the places MAP-1 maps build output to, each still
    /// holding its `*` (spec ENTRY-2, MAP-3).
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test entries
```

Expected: PASS: 14 tests.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two, 53 of them in the node
tests.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add src/entries.rs src/resolve.rs tests/entries.test.rs
git commit -m "fix(entries): read what a tool's configuration names as the tool reads it"
```

Expected: one commit on `feat/dead-code-6-live-check`.

### Task 2: the README and the contributing guide

**Implements:** DOC-1

**Test:** `node --test scripts/readme.test.mjs scripts/contributing.test.mjs`

**Files:**
- Modify: `README.md`, `CONTRIBUTING.md`
- Test: `scripts/readme.test.mjs`, `scripts/contributing.test.mjs`

**Interfaces:**
- Consumes: the commands `audit` and `graph` as plans 4 and 5 left them, and plan 5's git 2.44 for a
  base in a partial clone.
- Produces: the README's status that says the analyzer reports the five dead-code kinds, new or
  unchanged against a base, and the guide's `audit` command and what `graph` now prints.

- [ ] **Step 1: Write the failing tests**

```js edit path=scripts/readme.test.mjs
<<<<<<< find
  assert.ok(readme.includes('npx yak-analyzer --version\n'));
});

test('its status says what the analyzer reads, and that it reports no findings yet (spec DOC-1)', () => {
  assert.ok(readme.includes('reports\nno findings yet'));
  assert.ok(readme.includes('`yak-analyzer graph` prints what it read'));
  assert.ok(!readme.includes('analyzes nothing yet'));
});

test('it names the oldest git the analyzer reads a project with (spec DEP-2)', () => {
  assert.ok(readme.includes('`git` 2.15 or later on the `PATH`'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
=======
  assert.ok(readme.includes('npx yak-analyzer --version\n'));
});

test('its status says the analyzer reports the five dead-code kinds, new or inherited (dead-code spec DOC-1)', () => {
  assert.ok(readme.includes('`yak-analyzer audit` reports the five dead-code kinds'));
  assert.ok(readme.includes('marks each finding new or unchanged'));
  assert.ok(readme.includes('`yak-analyzer graph` prints what it read'));
  assert.ok(!readme.includes('no findings yet'));
});

test('it names the oldest git the analyzer reads a project with, and a base in a partial clone (spec DEP-2, dead-code COMMIT-2)', () => {
  assert.ok(readme.includes('`git` 2.15 or later on the `PATH`'));
  assert.ok(readme.includes('a partial clone needs git 2.44 or later'));
});

test('it says what to remove on a machine that maps the scope to another registry (spec NAME-2a)', () => {
>>>>>>> replace
```

```js edit path=scripts/contributing.test.mjs
<<<<<<< find
    '## The commands',
    '`yak-analyzer interface`',
    '`yak-analyzer graph --hand-off <file>`',
    'not part of the interface\nversion',
    '## Fixtures',
    '`tests/support/project.rs`',
=======
    '## The commands',
    '`yak-analyzer interface`',
    '`yak-analyzer graph --hand-off <file>`',
    'entry points with what made each one',
    '`yak-analyzer audit` reads the project as `graph` does',
    '--hand-off <file> --base <commit>`',
    'not part of the interface\nversion',
    '## Fixtures',
    '`tests/support/project.rs`',
>>>>>>> replace
```

- [ ] **Step 2: Run them and see them fail**

```bash
node --test scripts/readme.test.mjs scripts/contributing.test.mjs
```

Expected: FAIL: the README's status and git tests and the guide's commands test fail, since neither
says those things yet; the other four pass.

- [ ] **Step 3: Write the README and the guide**

```markdown edit path=README.md
<<<<<<< find
The first release, 0.1.0, is a walking skeleton: a binary that answers its version, built and
tested on five targets and published to npm with provenance.

The code since then reads a project. It lists the files git lists, reads the packages and the
tsconfig files, finds every import of the scripts, Vue components, MDX documents and HTML pages,
and works out where each import leads, with build output mapped back to its source. It reports
no findings yet: `yak-analyzer graph` prints what it read, as JSON, and `yak-analyzer interface`
prints the interface version and the hand-off formats it reads.

## Install

=======
The first release, 0.1.0, is a walking skeleton: a binary that answers its version, built and
tested on five targets and published to npm with provenance.

The code since then reads a project and reports what it finds. It lists the files git lists,
reads the packages and the tsconfig files, finds every import of the scripts, Vue components, MDX
documents and HTML pages, and works out where each import leads, with build output mapped back to
its source. `yak-analyzer audit` reports the five dead-code kinds, from the entry points the
project's files and the hand-off file name: unused files, unused exports and unused types,
unresolved imports, and circular dependencies, as one SARIF 2.1.0 report. With `--base <commit>`
it reads that commit from git as well, and marks each finding new or unchanged.
`yak-analyzer graph` prints what it read, as JSON, and `yak-analyzer interface` prints the
interface version and the hand-off formats it reads.

## Install

>>>>>>> replace
<<<<<<< find

To read a project it needs `git` 2.15 or later on the `PATH`: it asks git for the files and for
what git ignores, and stops with one line naming both versions when the git it finds is older.

If your own npm settings map the `@dabay6` scope to another registry, npm looks for the package
there and does not find it. Remove that setting first:
=======

To read a project it needs `git` 2.15 or later on the `PATH`: it asks git for the files and for
what git ignores, and stops with one line naming both versions when the git it finds is older.
An audit with `--base` in a partial clone needs git 2.44 or later, the first that reads a
commit's files without fetching what the clone left out.

If your own npm settings map the `@dabay6` scope to another registry, npm looks for the package
there and does not find it. Remove that setting first:
>>>>>>> replace
```

```markdown edit path=CONTRIBUTING.md
<<<<<<< find
- `yak-analyzer interface` prints one line of JSON: the interface version, and the hand-off
  formats the analyzer reads.
- `yak-analyzer graph` reads the project in the folder it is started in and prints what it read,
  as one JSON document: the packages, the files, each file's imports and where each leads, and the
  notices. `yak-analyzer graph --hand-off <file>` reads the project the hand-off file names, with
  its declarations. A run that cannot go on prints one line on standard error and exits 1.

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
version; its form may change. When it does, the fixtures' written-down output changes in the same
=======
- `yak-analyzer interface` prints one line of JSON: the interface version, and the hand-off
  formats the analyzer reads.
- `yak-analyzer graph` reads the project in the folder it is started in and prints what it read,
  as one JSON document: the packages, the files, each file's imports and where each leads, the
  entry points with what made each one, the files they reach, the uses of each export, and the
  notices. `yak-analyzer graph --hand-off <file>` reads the project the hand-off file names, with
  its declarations. A run that cannot go on prints one line on standard error and exits 1.
- `yak-analyzer audit` reads the project as `graph` does and prints one SARIF 2.1.0 report of the
  five dead-code kinds, with the notices as the run's notifications. `yak-analyzer audit
  --hand-off <file> --base <commit>` reads with the hand-off file's declarations, reads the base
  commit from git as well, and marks each finding new or unchanged. A command line it refuses
  exits 2.

What `graph` prints is the builder's and the tests' view of a reading, not part of the interface
version; its form may change. When it does, the fixtures' written-down output changes in the same
>>>>>>> replace
```

- [ ] **Step 4: Run every gate**

```bash
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0, and the node tests report 53 passed.

- [ ] **Step 5: Commit**

```bash
yak stack format
git add README.md CONTRIBUTING.md scripts/readme.test.mjs scripts/contributing.test.mjs
git commit -m "docs: say that the analyzer reports the five dead-code kinds"
```

Expected: one commit on `feat/dead-code-6-live-check`.

### Task 3: the check on the named repositories

**Implements:** LIVE-1, LIVE-2, LIVE-3, LIVE-4, LIVE-5, LIVE-7, LIVE-8

**Check:** the owner reads the report folder's `live/report.md` and each repository's `judged.md`:
that each declaration is one a pack or the repository would truly state, with its reason and its
count; that every finding of each run with no base is judged, true or false with why, and that every
false one is cured by Task 1 or written down with the stage it waits for; that each branch audit's
new findings, and its unchanged findings in files the branch changed, are judged; fallow's counts
beside the analyzer's; and the times beside the bars. A declaration the owner rejects is removed and
the runs judged again.

**Files:**
- None in this repository: the report folder, `.git/yak-reports/2026-11-14-dead-code-findings/live/`,
  is committed nowhere (LIVE-7).

**Interfaces:**
- Consumes: the binary built from this branch; the hand-off files and the worktrees of decisions 2 to 4.
- Produces: the report, and the counts, times and outcome on the record.

- [ ] **Step 1: Run every root again with this branch's build**

On each root, `audit --hand-off <file>`, and in each branch worktree `audit --hand-off <file> --base
<base>`, saving what each prints beside the earlier runs, and comparing with the runs of decision 1
made with Task 1 built.

Expected: each run exits 0 and reports what the run of decision 1 reported.

- [ ] **Step 2: Record the counts, the times and the outcome**

Record a ruling on the plan that holds, for each root, the counts by kind with and without the
hand-off file, the branch audit's new and unchanged counts, fallow's counts, the times against the
bars, and the outcome of LIVE-8.

Expected: the ruling is on the record.

- [ ] **Step 3: Hand the report to the owner, and remove the worktrees after validation**

```bash
git -C <repository> worktree remove <worktree>
```

Expected: after the owner validates the plan, each worktree is gone, and each repository's
`git worktree list` no longer names it.
