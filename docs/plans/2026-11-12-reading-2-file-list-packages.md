# Reading a project, part 2: the file list, the data and generated declarations, and packages: implementation plan

**Goal:** `graph` prints a project's real file list, as git knows it, with each file's kind, whether it
is data or generated, and the package it belongs to, and prints the project's packages, with which of
them the workspace names; and it stops, with one line, on what it must not guess past.

**Architecture:** Two starts of git at most give the whole file list (`src/git.rs`, `src/files.rs`),
read-only and with the settings the first plan's trial chose. The hand-off file's `data` and
`generated` globs are matched against that list with a glob matcher of the analyzer's own
(`src/glob.rs`, `src/marks.rs`), and every `package.json` that is not data is read for its name and,
at the root, for the workspace's list (`src/packages.rs`). `graph` puts it all in its one document;
imports come in later plans.

**Spec:** docs/specs/2026-11-11-reading-a-project-design.md

**Implements:** LIST-1 to LIST-4, LIST-6 to LIST-8, LIST-10, LIST-11, RUN-4, RUN-5, RUN-6, RUN-6a, RUN-8, RUN-8a, KEY-3, KEY-4, KEY-5, PACK-1 to PACK-6, FIX-1, FIX-6, FIX-7, DEP-1, DEP-2, DEP-4

This is the second of the item's plans. It is worked on its own branch, `feat/reading-2-file-list`,
made from the first plan's branch, and its pull request is a draft stacked on the first plan's.

## Global constraints

- Every start of git is read-only: `--no-optional-locks --no-pager -c core.fsmonitor=false -c
  core.untrackedCache=false -c core.ignorecase=false`, with the variables `git rev-parse
  --local-env-vars` prints removed from its environment (RUN-6a, from the first plan's trial).
- One start of git lists the whole file list; a root that is a sub-folder costs two more (LIST-11).
- Whether a path exists is asked of the file list; the file system is asked only for the content of
  a listed file and, by LIST-10 and LIST-11, for the identity of paths git listed (LIST-4).
- The stops this plan adds keep RUN-4's order: git cannot be started or is too old (6), git fails
  (7), the root is spelt otherwise than git spells it (7a), a `package.json` cannot be read (8), the
  workspace's list cannot be read (9), a field of a form it does not have (11).
- The output is the same bytes on every platform, except where RUN-8a says (RUN-8, RUN-8a).
- Every crate is pinned with `=` and listed here with its reason (DEP-1).
- The tests make each project a git repository of their own, with the machine's and the user's git
  settings shut out (FIX-1).

## The decisions this plan makes

1. **LIST-5 and LIST-9 move to the fourth plan,** where resolving imports first asks git whether a
   path is ignored and first reads a tsconfig above the root. The first plan named them for this
   one; building them here would leave code no part of the analyzer calls, which the lint refuses.
2. **Three crates are added, each pinned:** `same-file` `=1.0.6`, to compare two paths' identities
   (LIST-10, LIST-11; DEP-4); `regex` `=1.13.1`, which the glob matcher turns each glob into, since
   the spec's glob dialect has nested braces that no glob crate reads; and `yaml-rust2` `=0.13.0`
   with its default features off, to read `pnpm-workspace.yaml`. Each was the newest version at
   least two weeks old on 2026-11-12, and each is MIT or Apache licensed. `regex` was already in the
   lock file for the tests' validator and brings nothing new.
3. **What `graph` prints for each file and package:** a file has `data`, `generated`, `kind`,
   `package` (its package's folder, or null), `path` and `tracked`; a package has `detected`,
   `folder` (`.` for the root), `name` (or null) and `workspace`. The kinds are `script`,
   `declaration`, `vue`, `mdx`, `page`, `manifest`, `tsconfig` and `other`.
4. **The notice kinds this plan adds:** `git-warning`, `links-left-out`, `names-left-out`,
   `paths-differ-in-case`, `glob-matches-nothing` and `package-name-repeated`.
5. **PACK-6 here reads three fields:** `name` in every `package.json`, `workspaces` in the root's,
   and `packages` in `pnpm-workspace.yaml`. The fields that resolving imports reads (`exports`,
   `imports`, `types`, `typings`, `module`, `main`, the dependency lists) are read, and their forms
   checked, in the fourth plan.
6. **A workspace entry's leading `./` and trailing `/` are passed over,** since npm accepts both, and
   an entry that begins with `!` takes folders out again (PACK-3).
7. **A glob is turned into a regular expression once,** with `*` and `?` kept inside one segment, a
   whole-segment `**` spanning segments, `[...]` intersected with every character but `/`, and
   braces as alternatives. `folder/**` matches files under the folder and not the folder itself.
8. **The oldest git, 2.15, is asked about only after git fails,** since an older git fails on
   `--no-optional-locks` itself; a working git costs no start for its version.
9. **The README's line about git waits for the sixth plan,** which writes the README's status for
   what the analyzer reads (DOC-1). DEP-2's other part, a git at least that new in the Linux build
   images, holds: the first plan's pull request run printed `git version 2.56.0` in the
   `manylinux_2_28` image.
10. **The tests share one helper,** `tests/support/project.rs`, which every test file that uses it
    uses whole, so that no part of it is dead in any of them; the file-list tests keep their own
    helpers for links, recorded paths and nested repositories.

## Review focus

- **The two outputs of FIX-7 are tested on the volume each runner has.** Windows and the macOS
  runners ignore letter case by default, and Linux keeps it apart, so CI takes both branches; this
  machine takes only the first. The tests detect the volume, they do not assume it.
- **Tests that a system does not allow run where it does:** an untracked symbolic link and a stand-in
  for an old git on Linux and macOS only; a name that is not valid UTF-8 on Linux only.
- **A git that succeeds and warns** gives a `git-warning` notice. No test can make git warn on
  purpose, so a reviewer reads it in `src/files.rs`.
- **`same-file` on folders.** LIST-11 compares two folders' identities. It is tested where case is
  ignored (the stop) and where it is kept apart (the read); the Windows half rests on this machine's
  run and the CI Windows runner.
- **A `package.json` among data that does not parse** makes no package and stops nothing; the test
  that marks data holds one.

### Task 1: the file list

**Implements:** LIST-1 to LIST-4, LIST-6 to LIST-8, LIST-10, LIST-11, RUN-4, RUN-5, RUN-6, RUN-6a, RUN-8, RUN-8a, FIX-1, FIX-6, FIX-7, DEP-1, DEP-2, DEP-4

**Test:** `cargo test --release --locked --test file-list --test cli`

**Files:**
- Create: `src/git.rs`, `src/files.rs`, `src/notice.rs`, `tests/support/project.rs`
- Modify: `src/graph.rs`, `src/hand_off.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`, `tests/support/mod.rs`
- Test: `tests/file-list.test.rs`, `tests/cli.test.rs`

**Interfaces:**
- Consumes: the first plan's `hand_off::read`, `declarations::check`, `stop::Stop` and `graph::read`.
- Produces: `git::read`; `files::list`, `files::File` and `files::Kind`; `notice::Notice` and
  `notice::printed`; `hand_off::HandOff::root`; `tests/support/project.rs` with `Project`,
  `printed`, `stopped`, `paths` and `notices`; `tests/support/mod.rs`'s `folder` making an empty
  repository, `checkout`, which `document` names as the root.

- [ ] **Step 1: Write the failing tests**

The test of `graph` without a hand-off file now runs in an empty repository of its own, since the
root is read through git.

```rust path=tests/file-list.test.rs
//! Runs the built binary on made-up projects and checks the file list it prints, how it stops, and
//! what it leaves untouched (spec LIST-1 to LIST-4, LIST-6 to LIST-8, LIST-10, RUN-4, RUN-6, RUN-6a,
//! FIX-1, FIX-6, FIX-7).

#[path = "support/project.rs"]
mod project;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// Records a path in git's index with the mode and content given, commits it and checks it out, as a
/// checkout of a repository that already held it would.
fn record(project: &Project, mode: &str, path: &str, content: &str) {
    let source = project.home.join("recorded-content");

    fs::write(&source, content).expect("the content can be written");

    let blob = project.git(&[
        "hash-object",
        "-w",
        source.to_str().expect("the test's folder is valid UTF-8"),
    ]);

    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("{mode},{},{path}", blob.trim()),
    ]);
    project.git(&["commit", "--quiet", "--message", "a recorded path"]);
    project.git(&["checkout", "--", path]);
}

/// Records a symbolic link to the target given, as git records one, whatever the system checks out for
/// a link.
fn link(project: &Project, path: &str, target: &str) {
    record(project, "120000", path, target);
}

/// Makes the folder given a git repository of its own, inside the project, with one commit.
fn nest(project: &Project, folder: &str) {
    project.git(&["-C", folder, "init", "--quiet"]);
    project.git(&["-C", folder, "add", "--all"]);
    project.git(&["-C", folder, "commit", "--quiet", "--message", "nested"]);
}

/// The content of every file under a folder, `.git` included, by its path.
fn snapshot(folder: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![folder.to_path_buf()];

    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).expect("a folder can be listed") {
            let path = entry.expect("an entry can be read").path();

            if path.is_dir() {
                waiting.push(path);
            } else {
                found.insert(path.clone(), fs::read(&path).expect("a file can be read"));
            }
        }
    }

    found
}

#[test]
fn lists_tracked_files_and_untracked_files_that_nothing_ignores_each_with_its_kind() {
    let project = Project::new("list-kinds");

    project
        .write(".gitignore", "dist/\n*.log\n")
        .write("package.json", "{}")
        .write("tsconfig.json", "{}")
        .write("tsconfig.build.json", "{}")
        .write("tools/jsconfig.json", "{}")
        .write("src/main.ts", "")
        .write("src/view.tsx", "")
        .write("src/types.d.ts", "")
        .write("src/App.vue", "")
        .write("src/styles.css", "")
        .write("docs/intro.mdx", "")
        .write("public/index.html", "")
        .write("lib/old.cjs", "")
        .write("Zeta.mts", "")
        .commit()
        .write("scratch.ts", "")
        .write("dist/out.js", "")
        .write("debug.log", "")
        .write("local/only.ts", "")
        .write(".git/info/exclude", "local/\n");

    let document = project.graph();
    let file = |path: &str, kind: &str, tracked: bool| json!({ "kind": kind, "path": path, "tracked": tracked });

    // By path, byte by byte: a dot and capital letters come before small letters (spec RUN-8).
    assert_eq!(
        document["files"],
        json!([
            file(".gitignore", "other", true),
            file("Zeta.mts", "script", true),
            file("docs/intro.mdx", "mdx", true),
            file("lib/old.cjs", "script", true),
            file("package.json", "manifest", true),
            file("public/index.html", "page", true),
            file("scratch.ts", "script", false),
            file("src/App.vue", "vue", true),
            file("src/main.ts", "script", true),
            file("src/styles.css", "other", true),
            file("src/types.d.ts", "declaration", true),
            file("src/view.tsx", "script", true),
            file("tools/jsconfig.json", "tsconfig", true),
            file("tsconfig.build.json", "tsconfig", true),
            file("tsconfig.json", "tsconfig", true),
        ])
    );
    assert_eq!(document["handOff"], json!(false));
    assert_eq!(notices(&document), ["no-hand-off-file"]);
    assert_eq!(document["packages"], json!([]));
}

#[test]
fn never_reaches_into_a_nested_repository_a_submodule_or_another_worktree() {
    let other = Project::new("list-nested-other");

    other.write("other.ts", "").commit();

    let project = Project::new("list-nested");

    project.write("src/main.ts", "").commit();
    project.git(&[
        "submodule",
        "--quiet",
        "add",
        other.root.to_str().expect("valid UTF-8"),
        "sub",
    ]);
    project.git(&["commit", "--quiet", "--message", "a submodule"]);
    project.git(&[
        "worktree",
        "add",
        "--quiet",
        ".worktrees/second",
        "-b",
        "second",
    ]);
    project
        .write(".git/info/exclude", ".worktrees/\n")
        .write("nested/inner.ts", "");
    nest(&project, "nested");

    assert_eq!(paths(&project.graph()), [".gitmodules", "src/main.ts"]);
}

#[test]
fn leaves_a_symbolic_link_out_and_says_how_many() {
    let project = Project::new("list-links");

    project.write("src/main.ts", "").commit();
    link(&project, "link-to-main", "src/main.ts");

    let document = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
    assert!(
        document["notices"][0]["sentence"]
            .as_str()
            .is_some_and(|sentence| sentence.starts_with("1 symbolic link(s) were left out"))
    );
}

/// An untracked link can be made by a test only where the system lets any user make one.
#[cfg(unix)]
#[test]
fn leaves_an_untracked_symbolic_link_out_too() {
    let project = Project::new("list-links-untracked");

    project.write("src/main.ts", "").commit();
    std::os::unix::fs::symlink("src/main.ts", project.root.join("also-main.ts"))
        .expect("a link can be made");

    let document = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
}

#[test]
fn leaves_out_a_tracked_file_that_is_missing_from_the_working_tree() {
    let project = Project::new("list-missing");

    project
        .write("src/main.ts", "")
        .write("src/gone.ts", "")
        .commit();
    fs::remove_file(project.root.join("src/gone.ts")).expect("the file can be removed");

    assert_eq!(paths(&project.graph()), ["src/main.ts"]);
}

#[test]
fn counts_letter_case_in_an_ignore_pattern_on_every_platform() {
    let project = Project::new("list-ignore-case");

    project
        .write(".gitignore", "Build/\n")
        .write("src/main.ts", "")
        .commit()
        .write("build/out.js", "")
        .write("Build/other.js", "");

    // Where the file system ignores case the two folders are one, named as it was first written.
    let listed = project.graph();
    let listed = paths(&listed);

    assert!(listed.contains(&"build/out.js"), "{listed:?}");
    assert!(
        !listed.iter().any(|path| path.starts_with("Build/")),
        "{listed:?}"
    );
}

#[test]
fn leaves_out_an_untracked_path_that_differs_from_a_tracked_file_only_in_letter_case() {
    let project = Project::new("list-case-pair");

    project
        .write("docs/Guide.md", "tracked")
        .write("Assets/logo.txt", "tracked")
        .write("src/main.ts", "")
        .commit();

    // Where the file system ignores case, a pair is one file on disk under another spelling, made by
    // renaming it; where it keeps case apart, a pair is two files.
    let ignores_case = project.root.join("docs/guide.md").exists();

    if ignores_case {
        fs::rename(
            project.root.join("docs/Guide.md"),
            project.root.join("docs/guide.tmp"),
        )
        .expect("a rename");
        fs::rename(
            project.root.join("docs/guide.tmp"),
            project.root.join("docs/guide.md"),
        )
        .expect("a rename");
        fs::rename(project.root.join("Assets"), project.root.join("assets.tmp")).expect("a rename");
        fs::rename(project.root.join("assets.tmp"), project.root.join("assets")).expect("a rename");
    } else {
        project
            .write("docs/guide.md", "untracked")
            .write("assets/logo.txt", "untracked");
    }

    let document = project.graph();

    if !ignores_case {
        // Two real files under each pair of spellings: both are read, and nothing is left out.
        assert_eq!(
            paths(&document),
            [
                "Assets/logo.txt",
                "assets/logo.txt",
                "docs/Guide.md",
                "docs/guide.md",
                "src/main.ts"
            ]
        );
        assert_eq!(notices(&document), ["no-hand-off-file"]);

        return;
    }

    assert_eq!(
        paths(&document),
        ["Assets/logo.txt", "docs/Guide.md", "src/main.ts"]
    );
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "paths-differ-in-case"]
    );

    let notice = &document["notices"][1];
    let sentence = notice["sentence"].as_str().expect("a sentence");

    // One notice for both pairs, naming the first.
    assert_eq!(notice["file"], json!("Assets/logo.txt"));
    assert!(
        sentence.starts_with("2 untracked path(s) were left out"),
        "{sentence}"
    );
    assert!(
        sentence.contains("the first is `assets/logo.txt`, which git holds as `Assets/logo.txt`"),
        "{sentence}"
    );
}

#[test]
fn takes_an_index_that_spells_one_folder_two_ways_as_the_files_git_holds() {
    let project = Project::new("list-case-index");

    // The index holds `Dir/a.ts` and `dir/b.ts`, as a repository made on a system that keeps case
    // apart can; a checkout where case is ignored writes both into one folder.
    project.write("Dir/a.ts", "").commit();
    record(&project, "100644", "dir/b.ts", "");

    let ignores_case = project.root.join("dir/a.ts").exists();
    let document = project.graph();

    assert_eq!(paths(&document), ["Dir/a.ts", "dir/b.ts"]);

    if ignores_case {
        assert_eq!(
            notices(&document),
            ["no-hand-off-file", "paths-differ-in-case"]
        );
        assert!(
            document["notices"][1]["sentence"].as_str().is_some_and(
                |sentence| sentence.contains("`Dir/b.ts`, which git holds as `dir/b.ts`")
            )
        );
    } else {
        assert_eq!(notices(&document), ["no-hand-off-file"]);
    }
}

#[test]
fn lists_a_new_file_in_a_folder_whose_case_differs_under_the_disks_spelling() {
    let project = Project::new("list-case-folder-new");

    project.write("Assets/logo.txt", "tracked").commit();

    let ignores_case = project.root.join("assets/logo.txt").exists();

    if ignores_case {
        fs::rename(project.root.join("Assets"), project.root.join("assets.tmp")).expect("a rename");
        fs::rename(project.root.join("assets.tmp"), project.root.join("assets")).expect("a rename");
    }

    project.write("assets/new.ts", "");

    let document = project.graph();

    // The new file has no tracked path to pair with, so it is listed as the disk spells it.
    assert_eq!(paths(&document), ["Assets/logo.txt", "assets/new.ts"]);

    if ignores_case {
        // The one notice is about the tracked file renamed with its folder, not about the new file.
        assert_eq!(
            notices(&document),
            ["no-hand-off-file", "paths-differ-in-case"]
        );
        assert!(
            document["notices"][1]["sentence"]
                .as_str()
                .is_some_and(|sentence| sentence.starts_with("1 untracked path(s)"))
        );
    } else {
        assert_eq!(notices(&document), ["no-hand-off-file"]);
    }
}

#[test]
fn keeps_a_file_renamed_in_letter_case_alone_under_the_name_its_file_system_gives_it() {
    let project = Project::new("list-case-rename");

    project
        .write("src/Old.ts", "")
        .write("src/main.ts", "")
        .commit();

    let ignores_case = project.root.join("src/old.ts").exists();

    fs::rename(
        project.root.join("src/Old.ts"),
        project.root.join("src/old.tmp"),
    )
    .expect("a rename");
    fs::rename(
        project.root.join("src/old.tmp"),
        project.root.join("src/old.ts"),
    )
    .expect("a rename");

    let document = project.graph();

    if ignores_case {
        // git still finds the tracked name, so the file is listed under git's spelling.
        assert_eq!(paths(&document), ["src/Old.ts", "src/main.ts"]);
        assert_eq!(
            notices(&document),
            ["no-hand-off-file", "paths-differ-in-case"]
        );
    } else {
        // git calls the old name missing and the new one untracked, and the file is read as renamed.
        assert_eq!(paths(&document), ["src/main.ts", "src/old.ts"]);
        assert_eq!(notices(&document), ["no-hand-off-file"]);
    }
}

#[test]
fn reads_only_what_lies_under_a_root_that_is_a_sub_folder_of_the_work_tree() {
    let project = Project::new("list-sub-folder");

    project
        .write("README.md", "")
        .write("server/main.ts", "")
        .write("apps/web/package.json", "{}")
        .write("apps/web/src/main.ts", "")
        .commit()
        .write("apps/web/scratch.ts", "");

    let expected = ["package.json", "scratch.ts", "src/main.ts"];
    let web = project.root.join("apps/web");

    // Started in the sub-folder, with no hand-off file.
    assert_eq!(
        paths(&printed(&project.run(|binary| {
            binary.current_dir(&web);
        }))),
        expected
    );

    // Started elsewhere, with a hand-off file that names the sub-folder: the file decides the root.
    let file = project.hand_off(&web, &["."], json!([]));
    let handed = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(paths(&handed), expected);
    assert_eq!(handed["handOff"], json!(true));
    assert_eq!(notices(&handed), [] as [&str; 0]);
}

#[test]
fn stops_on_a_root_whose_folder_git_spells_otherwise_where_case_is_ignored() {
    let project = Project::new("list-case-root");

    project.write("Apps/web/a.ts", "").commit();

    let ignores_case = project.root.join("apps/web/a.ts").exists();

    if ignores_case {
        fs::rename(project.root.join("Apps"), project.root.join("apps.tmp")).expect("a rename");
        fs::rename(project.root.join("apps.tmp"), project.root.join("apps")).expect("a rename");
    } else {
        // Where case is kept apart, `apps/web` is a folder of its own beside git's `Apps/web`.
        project.write("apps/web/b.ts", "");
    }

    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let output = project.run(|binary| {
        binary.args(["--hand-off", &file]);
    });

    if ignores_case {
        assert_eq!(
            stopped(&output),
            format!(
                "yak-analyzer: the root {} is spelt `apps/web` on disk, and git holds the files under it as \
                 `Apps/web`; rename the folder on disk to git's spelling, or record the disk's with `git mv`, \
                 and run again",
                root.display()
            )
        );
    } else {
        assert_eq!(paths(&printed(&output)), ["b.ts"]);
    }
}

#[test]
fn stops_when_the_root_is_not_a_folder_or_is_not_inside_a_git_work_tree() {
    let project = Project::new("list-no-work-tree");
    let outside = project
        .root
        .parent()
        .expect("the project's home")
        .join("no-repository");
    let absent = project.root.join("not-there");

    fs::create_dir_all(&outside).expect("a folder can be made");

    // The test's own folders lie inside this repository's work tree, so git is told to look no higher.
    let line = stopped(&project.run(|binary| {
        binary.current_dir(&outside).env(
            "GIT_CEILING_DIRECTORIES",
            outside.parent().expect("a parent"),
        );
    }));

    assert!(
        line.starts_with(&format!(
            "yak-analyzer: git cannot read the root {}: ",
            outside.display()
        )),
        "{line}"
    );
    assert!(
        line.ends_with("; run the analyzer inside a git work tree"),
        "{line}"
    );

    let file = project.hand_off(&absent, &["."], json!([]));

    assert_eq!(
        stopped(&project.run(|binary| {
            binary.args(["--hand-off", &file]);
        })),
        format!(
            "yak-analyzer: the root {} is not a folder; run the analyzer in a checkout, or hand it a hand-off \
             file that names one",
            absent.display()
        )
    );
}

#[test]
fn stops_when_git_cannot_be_started() {
    let project = Project::new("list-no-git");

    project.write("src/main.ts", "").commit();

    let empty = project
        .root
        .parent()
        .expect("the project's home")
        .join("empty-path");

    fs::create_dir_all(&empty).expect("a folder can be made");

    let line = stopped(&project.run(|binary| {
        binary.env("PATH", &empty);
    }));

    assert!(
        line.starts_with("yak-analyzer: git cannot be started ("),
        "{line}"
    );
    assert!(
        line.ends_with(
            "); the analyzer reads a project through git 2.15 or newer, which must be on the PATH"
        ),
        "{line}"
    );
}

/// A stand-in for an old git can be put on the `PATH` by a test only where a script is a program.
#[cfg(unix)]
#[test]
fn stops_on_a_git_older_than_the_oldest_it_works_with() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new("list-old-git");

    project.write("src/main.ts", "").commit();

    let bin = project
        .root
        .parent()
        .expect("the project's home")
        .join("old-git");
    let git = bin.join("git");

    fs::create_dir_all(&bin).expect("a folder can be made");
    fs::write(
        &git,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'git version 2.14.6'; exit 0; fi\n\
         echo 'error: unknown option: --no-optional-locks' >&2\nexit 129\n",
    )
    .expect("the stand-in can be written");
    fs::set_permissions(&git, fs::Permissions::from_mode(0o755))
        .expect("the stand-in can be made a program");

    assert_eq!(
        stopped(&project.run(|binary| {
            binary.env("PATH", &bin);
        })),
        "yak-analyzer: git 2.14.6 is older than 2.15, the oldest the analyzer works with; update git"
    );
}

/// A name that is not valid UTF-8 can be given to a file only where the file system takes any bytes.
#[cfg(target_os = "linux")]
#[test]
fn leaves_out_a_file_whose_name_is_not_valid_utf8_and_says_so() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let project = Project::new("list-odd-name");

    project.write("src/main.ts", "").commit();
    fs::write(
        project.root.join(OsStr::from_bytes(b"odd-\xFF-name.ts")),
        "",
    )
    .expect("the file can be written");

    let document = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert_eq!(notices(&document), ["names-left-out", "no-hand-off-file"]);
}

#[test]
fn changes_nothing_under_the_root_the_git_folder_included() {
    let project = Project::new("git-untouched");

    project
        .write("src/main.ts", "")
        .write("src/other.ts", "")
        .commit()
        .write("scratch.ts", "");

    // A changed file makes git's index stale, which a plain `git status` would then rewrite.
    fs::write(
        project.root.join("src/other.ts"),
        "export const changed = 1;\n",
    )
    .expect("the file can be written");

    let before = snapshot(&project.root);

    project.graph();

    assert!(
        before == snapshot(&project.root),
        "a file under the root changed"
    );
}

#[test]
fn does_not_obey_a_callers_variables_that_point_git_at_another_repository() {
    let other = Project::new("git-variables-other");

    other.write("other.ts", "").commit();

    let project = Project::new("git-variables");

    project.write("src/main.ts", "").commit();

    let pointed = printed(&project.run(|binary| {
        binary
            .env("GIT_DIR", other.root.join(".git"))
            .env("GIT_WORK_TREE", &other.root)
            .env("GIT_INDEX_FILE", other.root.join(".git/index"));
    }));

    assert_eq!(pointed, project.graph());
    assert_eq!(paths(&pointed), ["src/main.ts"]);
}

#[test]
fn does_not_start_a_program_the_repositorys_own_settings_name() {
    let project = Project::new("git-program");

    project.write("src/main.ts", "").commit();

    let home = project.root.parent().expect("the project's home");
    let marker = home.join("the-program-ran");
    let program = home.join("monitor.sh");
    let unix = |path: &Path| path.to_str().expect("valid UTF-8").replace('\\', "/");

    fs::write(
        &program,
        format!("#!/bin/sh\ntouch \"{}\"\nexit 1\n", unix(&marker)),
    )
    .expect("the program is written");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&program, fs::Permissions::from_mode(0o755))
            .expect("the program can be started");
    }

    project.git(&["config", "core.fsmonitor", &unix(&program)]);

    // The setting works: a plain git command starts the program.
    project.git(&["status", "--short"]);
    assert!(
        marker.exists(),
        "git did not start the program, so the test proves nothing"
    );
    fs::remove_file(&marker).expect("the marker can be removed");

    let document: Value = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert!(!marker.exists(), "the analyzer's git started the program");
}
```

```rust path=tests/cli.test.rs
//! Starts the built binary and checks its commands and how it refuses (spec RUN-1, RUN-2, RUN-3, RUN-7).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

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
fn prints_its_interface_version_and_the_hand_off_formats_it_reads_as_one_line_of_json() {
    let output = run(&["interface"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"handOffFormats\":[1],\"interfaceVersion\":1}\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn graph_without_a_hand_off_file_prints_one_document_and_says_so_in_a_notice() {
    // Without a hand-off file the root is the folder the binary is started in: an empty repository here.
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-graph");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root).expect("the test's folder can be made");
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("git starts")
            .success()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .arg("graph")
        .current_dir(&root)
        .output()
        .expect("the binary starts");
    let document: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(document["handOff"], json!(false));
    assert_eq!(document["files"], json!([]));
    assert_eq!(document["packages"], json!([]));
    assert_eq!(document["notices"].as_array().map(Vec::len), Some(1));
    assert_eq!(document["notices"][0]["kind"], json!("no-hand-off-file"));
    assert_eq!(document["notices"][0]["file"], Value::Null);
}

#[test]
fn refuses_anything_else_with_one_line_on_standard_error() {
    let cases: [&[&str]; 6] = [
        &["analyze"],
        &["--version", "extra"],
        &["--help"],
        &["interface", "extra"],
        &["graph", "extra"],
        &["graph", "--base", "main"],
    ];

    for args in cases {
        let output = run(args);
        let error = String::from_utf8_lossy(&output.stderr);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(error.lines().count(), 1, "{args:?}");
        assert!(error.starts_with("yak-analyzer: cannot do `"), "{args:?}");
        assert!(error.contains(&args.join(" ")), "{args:?}");
        assert!(
            error.contains("--version, interface, and graph"),
            "{args:?}"
        );
    }
}

#[test]
fn refuses_no_argument_at_all_the_same_way() {
    let output = run(&[]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("--version, interface, and graph"));
}

#[test]
fn keeps_a_refusal_on_one_line_when_an_argument_holds_a_line_break() {
    let output = run(&["first\nsecond"]);
    let error = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(error.lines().count(), 1);
    assert!(error.contains("first\\nsecond"));
}
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test file-list --test cli
```

Expected: cargo refuses, because the manifest names no test called `file-list` yet.

- [ ] **Step 3: Name the test, add the crate, and write the tests' helpers**

```toml path=Cargo.toml
[package]
name = "yak-analyzer"
version = "0.1.0"
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

[[test]]
name = "hand-off"
path = "tests/hand-off.test.rs"

[[test]]
name = "declarations"
path = "tests/declarations.test.rs"

[[test]]
name = "hand-off-schema"
path = "tests/hand-off-schema.test.rs"

[[test]]
name = "file-list"
path = "tests/file-list.test.rs"

[dependencies]
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"
# Tells whether two paths are one file on disk, which Rust gives on Windows only in an unstable form.
same-file = "=1.0.6"

[dev-dependencies]
# Checks the tests' hand-off files against the copy of yakshaver's schema. Never part of the binary.
jsonschema = { version = "=0.60.2", default-features = false }
```

```toml path=Cargo.lock
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "ahash"
version = "0.8.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
dependencies = [
 "cfg-if",
 "getrandom",
 "once_cell",
 "serde",
 "version_check",
 "zerocopy",
]

[[package]]
name = "aho-corasick"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c982642fa9e8606056828ee9a8505737230110bb1099153c79efe865c59d12ba"
dependencies = [
 "memchr",
]

[[package]]
name = "allocator-api2"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923"

[[package]]
name = "autocfg"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"

[[package]]
name = "bit-set"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3"
dependencies = [
 "bit-vec",
]

[[package]]
name = "bit-vec"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7"

[[package]]
name = "bitflags"
version = "2.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"

[[package]]
name = "borrow-or-share"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc0b364ead1874514c8c2855ab558056ebfeb775653e7ae45ff72f28f8f3166c"

[[package]]
name = "bumpalo"
version = "3.20.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649"

[[package]]
name = "bytecount"
version = "0.6.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175812e0be2bccb6abe50bb8d566126198344f707e304f45c648fd8f2cc0365e"

[[package]]
name = "cfg-if"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"

[[package]]
name = "data-encoding"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "email_address"
version = "0.2.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e079f19b08ca6239f47f8ba8509c11cf3ea30095831f7fed61441475edd8c449"
dependencies = [
 "serde",
]

[[package]]
name = "equivalent"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00d174d5400e5e8fd687ad1049e2f578285fa914201b1af7e8b112a4546bd826"

[[package]]
name = "fancy-regex"
version = "0.19.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d301f5bf187b3c295fce6468d3875037a0bccc5f6b151c63cac2f85babf21912"
dependencies = [
 "bit-set",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "fluent-uri"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bc74ac4d8359ae70623506d512209619e5cf8f347124910440dbc221714b328e"
dependencies = [
 "borrow-or-share",
 "ref-cast",
 "serde",
]

[[package]]
name = "foldhash"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77ce24cb58228fbb8aa041425bb1050850ac19177686ea6e0f41a70416f56fdb"

[[package]]
name = "fraction"
version = "0.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e246562084dde8ebbcc943b261c406ce4f68e5032ec28029a251a47d6a295500"
dependencies = [
 "num",
 "num-bigint",
]

[[package]]
name = "getrandom"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "r-efi",
 "wasip2",
 "wasm-bindgen",
]

[[package]]
name = "hashbrown"
version = "0.17.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a"
dependencies = [
 "allocator-api2",
 "equivalent",
 "foldhash",
]

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "itoa"
version = "1.0.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3bd1574e617a9a6c1f8f8181bd7e841e1f20e4277607ef28945765188d6284d4"

[[package]]
name = "js-sys"
version = "0.3.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8dc09d2154098bbf4ef54eab6b117a3c24cbbffe94046b16ee6d6d4510584d3a"
dependencies = [
 "cfg-if",
 "wasm-bindgen",
]

[[package]]
name = "jsonschema"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a42a11b47492e6a2cc76e9c7bcc8077776cb31dac444e0795d7f61bcc6441c08"
dependencies = [
 "ahash",
 "bytecount",
 "data-encoding",
 "email_address",
 "fancy-regex",
 "fraction",
 "getrandom",
 "itoa",
 "jsonschema-regex",
 "jsonschema-value",
 "num-cmp",
 "num-traits",
 "percent-encoding",
 "referencing",
 "regex",
 "serde",
 "serde_json",
 "strum",
 "unicode-general-category",
 "uuid-simd",
]

[[package]]
name = "jsonschema-regex"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "107aeb6ff14b8deb5610096a8ce5c554ec89806b521147eaeb3cae05b07edb7a"
dependencies = [
 "regex-syntax",
]

[[package]]
name = "jsonschema-value"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e281d6b177bb4834f2ffd6f92aa0443cffe29c996b251c8d94eb6f456186feba"
dependencies = [
 "ahash",
 "bytecount",
 "fraction",
 "getrandom",
 "memchr",
 "num-cmp",
 "num-traits",
 "serde_json",
 "zmij",
]

[[package]]
name = "libc"
version = "0.2.191"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "65cd87c9a6d60536580b316ad985726104730a5151c99845f2490428ef91240a"

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "memchr"
version = "2.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"

[[package]]
name = "micromap"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a86d3146ed3995b5913c414f6664344b9617457320782e64f0bb44afd49d74"

[[package]]
name = "num"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "35bd024e8b2ff75562e5f34e7f4905839deb4b22955ef5e73d2fea1b9813cb23"
dependencies = [
 "num-bigint",
 "num-complex",
 "num-integer",
 "num-iter",
 "num-rational",
 "num-traits",
]

[[package]]
name = "num-bigint"
version = "0.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-cmp"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63335b2e2c34fae2fb0aa2cecfd9f0832a1e24b3b32ecec612c3426d46dc8aaa"

[[package]]
name = "num-complex"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-integer"
version = "0.1.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7ce2d95d4b3734dc35aa2f45e1aa22cd416814592a4f9d9205e11affd5b8e10b"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-iter"
version = "0.1.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c92800bd69a1eac91786bcfe9da64a897eb72911b8dc3095decbd07429e8048b"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-rational"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824"
dependencies = [
 "num-bigint",
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-traits"
version = "0.2.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
dependencies = [
 "autocfg",
]

[[package]]
name = "once_cell"
version = "1.21.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"

[[package]]
name = "outref"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a80800c0488c3a21695ea981a54918fbb37abf04f4d0720c453632255e2ff0e"

[[package]]
name = "parking_lot"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
dependencies = [
 "lock_api",
 "parking_lot_core",
]

[[package]]
name = "parking_lot_core"
version = "0.9.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
dependencies = [
 "cfg-if",
 "libc",
 "redox_syscall",
 "smallvec",
 "windows-link",
]

[[package]]
name = "percent-encoding"
version = "2.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"

[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "r-efi"
version = "5.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"

[[package]]
name = "redox_syscall"
version = "0.5.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
dependencies = [
 "bitflags",
]

[[package]]
name = "ref-cast"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e440fb4e4b4147295338efb76001ab9e4efc0e5839df2c47fc5ac2381d365c3"
dependencies = [
 "ref-cast-impl",
]

[[package]]
name = "ref-cast-impl"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92ecd8964f8453721699a1ed72037b0db49ce2f5a5138486ee89bed6f67cdf3a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "referencing"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f42818036d3b37cda6bd4c494543158a36e431cb717015ef7c93acc6ba214d3b"
dependencies = [
 "ahash",
 "fluent-uri",
 "getrandom",
 "hashbrown",
 "itoa",
 "micromap",
 "parking_lot",
 "percent-encoding",
 "serde_json",
]

[[package]]
name = "regex"
version = "1.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "regex-automata"
version = "0.4.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad8553b9b26413251cbf30e620595c7a41b3887f03da04579c0e6b0d6a06b4b2"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-syntax",
]

[[package]]
name = "regex-syntax"
version = "0.8.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4"

[[package]]
name = "rustversion"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"

[[package]]
name = "same-file"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502"
dependencies = [
 "winapi-util",
]

[[package]]
name = "scopeguard"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"

[[package]]
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
dependencies = [
 "serde_core",
 "serde_derive",
]

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "serde_json"
version = "1.0.154"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "smallvec"
version = "1.16.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "149309498a5f7334b12ddae0cad8aba81ce785fe91755068867e017cf98fee46"

[[package]]
name = "strum"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9628de9b8791db39ceda2b119bbe13134770b56c138ec1d3af810d045c04f9bd"
dependencies = [
 "strum_macros",
]

[[package]]
name = "strum_macros"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab85eea0270ee17587ed4156089e10b9e6880ee688791d45a905f5b1ca36f664"
dependencies = [
 "heck",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "syn"
version = "2.0.119"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "3.0.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ae9ccbea3a7b0b09443036490c8f34a8b1b760fd345e5c0dae0aac718faea79"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "unicode-general-category"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b993bddc193ae5bd0d623b49ec06ac3e9312875fdae725a975c51db1cc1677f"

[[package]]
name = "unicode-ident"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563"

[[package]]
name = "uuid-simd"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b082222b4f6619906941c17eb2297fff4c2fb96cb60164170522942a200bd8"
dependencies = [
 "outref",
 "vsimd",
]

[[package]]
name = "version_check"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"

[[package]]
name = "vsimd"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c3082ca00d5a5ef149bb8b555a72ae84c9c59f7250f013ac822ac2e49b19c64"

[[package]]
name = "wasip2"
version = "1.0.4+wasi-0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b67efb37e106e55ce722a510d6b5f9c17f083e5fc79afc2badeb12cc313d9487"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasm-bindgen"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0181c14ddd6c7b8b0b0132a5f1ea2f32be0d893cd3ea97b6e3b28886b8a3d1c"
dependencies = [
 "cfg-if",
 "rustversion",
 "wasm-bindgen-macro",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e931a02a39abdbf66966fc35902b5ed58344750a70885bed4ebb3d8eaf28ed9"
dependencies = [
 "quote",
 "wasm-bindgen-macro-support",
]

[[package]]
name = "wasm-bindgen-macro-support"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ca4b54280e455bd831e33a6b8e734baf432fe63fbc35c42ac14ce67a043a53a"
dependencies = [
 "bumpalo",
 "proc-macro2",
 "quote",
 "syn 3.0.10",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-shared"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b1f9c3798725016c6ee5f2f57869ec0831e8bbc0ab0310a86e787e97f41389e"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "winapi-util"
version = "0.1.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22"
dependencies = [
 "windows-sys",
]

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "windows-sys"
version = "0.61.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
dependencies = [
 "windows-link",
]

[[package]]
name = "wit-bindgen"
version = "0.57.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e"

[[package]]
name = "yak-analyzer"
version = "0.1.0"
dependencies = [
 "jsonschema",
 "same-file",
 "serde_json",
]

[[package]]
name = "zerocopy"
version = "0.8.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5fe1f8f1b06191a00962174c61aa5005e0bb391a6d80d07e24d115c01a92ed8"
dependencies = [
 "zerocopy-derive",
]

[[package]]
name = "zerocopy-derive"
version = "0.8.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "863ad3ac83293fb4d740aedbfdc9240dd8d1a50c1099acd76ce80ce7c7230c7f"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"
```

```rust path=tests/support/project.rs
//! A made-up project as a git repository of its own in a temporary folder, for the tests that run the
//! binary on one (spec FIX-1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Shuts the machine's and the user's own git settings out of a command, global excludes included, so
/// that a test's result does not depend on the machine.
fn alone(command: &mut Command) -> &mut Command {
    let none = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-git-settings");

    if !none.exists() {
        fs::write(&none, "").expect("an empty settings file can be written");
    }

    command
        .env("GIT_CONFIG_GLOBAL", &none)
        .env("GIT_CONFIG_SYSTEM", &none)
        .env("GIT_CONFIG_NOSYSTEM", "1")
}

/// One made-up project.
pub struct Project {
    /// The folder that holds the checkout and, beside it, the project's hand-off files and whatever
    /// else a test makes outside the checkout.
    pub home: PathBuf,
    /// The checkout's root.
    pub root: PathBuf,
}

impl Project {
    /// A new project of the name given, with an empty git repository as its checkout.
    pub fn new(name: &str) -> Self {
        let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);

        // The folder is left over from an earlier run, or is not there: either way it is made new.
        let _ = fs::remove_dir_all(&home);

        let project = Self {
            root: home.join("checkout"),
            home,
        };

        fs::create_dir_all(&project.root).expect("the project's folder can be made");
        project.git(&["init", "--quiet", "--initial-branch=main"]);

        project
    }

    /// Writes a file of the project, with the folders above it.
    pub fn write(&self, path: &str, content: &str) -> &Self {
        let file = self.root.join(path);

        fs::create_dir_all(file.parent().expect("a file has a folder"))
            .expect("the folder can be made");
        fs::write(file, content).expect("the file can be written");

        self
    }

    /// Runs git in the checkout and returns what it printed. The command must succeed.
    pub fn git(&self, args: &[&str]) -> String {
        let mut git = Command::new("git");

        git.args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(["-c", "protocol.file.allow=always"])
        .args(args)
        .current_dir(&self.root);

        let ran = alone(&mut git).output().expect("git starts");

        assert!(
            ran.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&ran.stderr)
        );

        String::from_utf8_lossy(&ran.stdout).into_owned()
    }

    /// Tracks everything that is not ignored, in one commit.
    pub fn commit(&self) -> &Self {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--quiet", "--message", "the project"]);

        self
    }

    /// Runs `graph` in the checkout, after the change the test asks for, and returns how it ended.
    pub fn run(&self, change: impl FnOnce(&mut Command)) -> Output {
        let mut binary = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"));

        binary.arg("graph").current_dir(&self.root);
        alone(&mut binary);
        change(&mut binary);

        binary.output().expect("the binary starts")
    }

    /// The document `graph` prints for the checkout. The run must succeed and write no line.
    pub fn graph(&self) -> Value {
        printed(&self.run(|_| {}))
    }

    /// Writes a hand-off file that names the folder given as the root, with the folders yak's detection
    /// looked in and the stacks given, and returns the file's path.
    pub fn hand_off(&self, root: &Path, folders: &[&str], stacks: Value) -> String {
        let folder = self.home.join("hand-off");
        let file = folder.join("hand-off.json");
        let text = |path: &Path| {
            path.to_str()
                .expect("the test's folder is valid UTF-8")
                .to_owned()
        };
        let document = json!({
            "formatVersion": 1,
            "root": text(root),
            "detection": { "folders": folders, "truncated": false },
            "stacks": stacks,
            "siblings": text(&folder),
        });

        fs::create_dir_all(&folder).expect("the folder can be made");
        fs::write(&file, document.to_string()).expect("the hand-off file can be written");

        text(&file)
    }
}

/// The document a successful run printed.
pub fn printed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).expect("graph prints JSON")
}

/// The one line a stopped run wrote, after checking that it stopped as a stop does.
pub fn stopped(output: &Output) -> String {
    let error = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(output.stdout.is_empty(), "{error}");
    assert_eq!(error.lines().count(), 1, "{error}");

    error.trim_end().to_owned()
}

/// The paths of the files a document lists, in its order.
pub fn paths(document: &Value) -> Vec<&str> {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .map(|file| file["path"].as_str().expect("a path"))
        .collect()
}

/// The kinds of the notices a document holds, in its order.
pub fn notices(document: &Value) -> Vec<&str> {
    document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .map(|notice| notice["kind"].as_str().expect("a kind"))
        .collect()
}
```

```rust path=tests/support/mod.rs
//! What the tests that hand the binary a hand-off file share.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Runs the built binary with the arguments given.
pub fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yak-analyzer"))
        .args(args)
        .output()
        .expect("the binary starts")
}

/// A new, empty folder of one test's own, under cargo's folder for the files tests make. It holds an
/// empty git repository, `checkout`, which the test's hand-off files name as their root, since the
/// analyzer reads its root through git.
pub fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let checkout = folder.join("checkout");

    // The folder is left over from an earlier run, or is not there: either way it is made new.
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&checkout).expect("the test's folder can be made");

    let made = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&checkout)
        .output()
        .expect("git starts");

    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );

    folder
}

/// A hand-off file of format 1 that the analyzer can use, for the checkout in the folder given.
pub fn document(folder: &Path) -> Value {
    let text = |path: &Path| {
        path.to_str()
            .expect("the test's folder is valid UTF-8")
            .to_owned()
    };

    json!({
        "formatVersion": 1,
        "root": text(&folder.join("checkout")),
        "detection": { "folders": [".", "packages/app"], "truncated": false },
        "stacks": [
            {
                "name": "typescript",
                "interfaceVersion": 1,
                "pack": {
                    "data": ["**/fixtures/**"],
                    "cyclomaticLimit": 20,
                    "configEntries": [{ "file": "runner.config.*", "property": "include" }]
                },
                "project": { "generated": ["src/generated/**"], "internal": ["packages/core"] }
            },
            { "name": "bare", "pack": {}, "project": {} }
        ],
        "siblings": text(folder)
    })
}

/// Writes a document as the hand-off file in the folder, beside the checkout, and returns the file's
/// path as text.
pub fn written(folder: &Path, document: &Value) -> String {
    let file = folder.join("hand-off.json");

    fs::write(&file, document.to_string()).expect("the hand-off file can be written");

    file.to_str()
        .expect("the test's folder is valid UTF-8")
        .to_owned()
}
```

- [ ] **Step 4: Write git's part, the file list, and the notices**

```rust path=src/git.rs
//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::stop::Stop;

/// The oldest git these commands work with: the release that brought `--no-optional-locks`.
const OLDEST: (u32, u32) = (2, 15);

/// What every start of git is given before its command (spec RUN-6a): no lock is taken, so nothing is
/// written inside `.git`; no pager and no file-system monitor is started; and letter case counts in an
/// ignore pattern on every platform.
const SETTINGS: [&str; 8] = [
    "--no-optional-locks",
    "--no-pager",
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.untrackedCache=false",
    "-c",
    "core.ignorecase=false",
];

/// The variables git itself calls local to a repository, which `git rev-parse --local-env-vars` prints.
/// Left in place, they would make git read another repository than the root's.
const REPOSITORY_VARIABLES: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// What git printed, and the first line of a warning it gave while succeeding.
pub struct Answer {
    /// What git wrote on standard output.
    pub output: Vec<u8>,
    /// The first line git wrote on standard error, where it wrote one.
    pub warning: Option<String>,
}

/// Runs one git command in the root and returns what it printed.
pub fn read(root: &Path, command: &[&str]) -> Result<Answer, Stop> {
    let shown = root.display();

    if !root.is_dir() {
        return Err(Stop::new(format!(
            "the root {shown} is not a folder; run the analyzer in a checkout, or hand it a hand-off file \
             that names one"
        )));
    }

    let mut git = Command::new("git");

    git.args(SETTINGS)
        .args(command)
        .current_dir(root)
        .stdin(Stdio::null());

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    let (oldest_major, oldest_minor) = OLDEST;
    let ran = git.output().map_err(|error| {
        Stop::new(format!(
            "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
             {oldest_minor} or newer, which must be on the PATH"
        ))
    })?;
    let said = String::from_utf8_lossy(&ran.stderr);
    let first = said.lines().find(|line| !line.trim().is_empty());

    if ran.status.success() {
        return Ok(Answer {
            output: ran.stdout,
            warning: first.map(str::to_owned),
        });
    }

    // git failed. An old git fails on the settings themselves, so the version is asked only now.
    if let Some(version) = version().filter(|version| version.0 < OLDEST) {
        return Err(Stop::new(format!(
            "git {} is older than {oldest_major}.{oldest_minor}, the oldest the analyzer works with; update git",
            version.1
        )));
    }

    Err(Stop::new(format!(
        "git cannot read the root {shown}: {}; run the analyzer inside a git work tree",
        first.unwrap_or("it gave no reason")
    )))
}

/// The version of the git on the `PATH`: its first two numbers, and the version as git writes it.
fn version() -> Option<((u32, u32), String)> {
    let answer = Command::new("git")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&answer.stdout);
    // git prints `git version 2.56.0`, and some builds add more after the numbers.
    let written = text
        .trim()
        .strip_prefix("git version ")?
        .split(' ')
        .next()?;
    let mut numbers = written.split('.').map(str::parse::<u32>);
    let major = numbers.next()?.ok()?;
    let minor = numbers.next()?.ok()?;

    Some(((major, minor), written.to_owned()))
}
```

```rust path=src/files.rs
//! The file list: every file under the root that git tracks, and every file there that is untracked and
//! that git does not ignore (spec LIST-1 to LIST-4, LIST-6, LIST-7, LIST-10, LIST-11).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::git;
use crate::notice::Notice;
use crate::stop::Stop;

/// The command that lists everything in one start of git: tracked files with their modes, untracked
/// files that nothing ignores, and tracked files missing from the working tree, each name ended by a
/// zero byte.
const LIST: [&str; 8] = [
    "ls-files",
    "-z",
    "-t",
    "--stage",
    "--cached",
    "--others",
    "--exclude-standard",
    "--deleted",
];

/// The mode git records for a symbolic link.
const LINK_MODE: &str = "120000";

/// The mode git records for a submodule.
const SUBMODULE_MODE: &str = "160000";

/// The endings of a type declaration file. They are asked before a script's, which they end in.
const DECLARATION_ENDINGS: [&str; 3] = [".d.ts", ".d.mts", ".d.cts"];

/// The endings of a script.
const SCRIPT_ENDINGS: [&str; 8] = [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"];

/// What kind of file a listed file is, from its name (spec LIST-6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A script in TypeScript or JavaScript.
    Script,
    /// A type declaration file.
    Declaration,
    /// A Vue component.
    Vue,
    /// An MDX document.
    Mdx,
    /// An HTML page.
    Page,
    /// A `package.json`.
    Manifest,
    /// A tsconfig or a `jsconfig.json`.
    Tsconfig,
    /// Any other file, which is never read.
    Other,
}

impl Kind {
    /// The kind of the file with the path given.
    fn of(path: &str) -> Self {
        let name = path.rsplit('/').next().unwrap_or(path);

        if DECLARATION_ENDINGS
            .iter()
            .any(|ending| name.ends_with(ending))
        {
            Self::Declaration
        } else if SCRIPT_ENDINGS.iter().any(|ending| name.ends_with(ending)) {
            Self::Script
        } else if name.ends_with(".vue") {
            Self::Vue
        } else if name.ends_with(".mdx") {
            Self::Mdx
        } else if name.ends_with(".html") {
            Self::Page
        } else if name == "package.json" {
            Self::Manifest
        } else if name == "jsconfig.json"
            || (name.starts_with("tsconfig") && name.ends_with(".json"))
        {
            Self::Tsconfig
        } else {
            Self::Other
        }
    }

    /// The kind's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Declaration => "declaration",
            Self::Vue => "vue",
            Self::Mdx => "mdx",
            Self::Page => "page",
            Self::Manifest => "manifest",
            Self::Tsconfig => "tsconfig",
            Self::Other => "other",
        }
    }
}

/// One file of the list.
#[derive(Debug)]
pub struct File {
    /// Its path, relative to the root, with forward slashes.
    pub path: String,
    /// Its kind.
    pub kind: Kind,
    /// Whether git tracks it.
    pub tracked: bool,
}

/// Stops the run when the root is a sub-folder that git holds files under by another spelling of its
/// path, which would make git list every file under it as untracked (spec LIST-11).
fn root_spelling(root: &Path) -> Result<(), Stop> {
    // A root that holds a `.git` of its own is the top of its work tree: there is nothing to find.
    if fs::symlink_metadata(root.join(".git")).is_ok() {
        return Ok(());
    }

    let answer = git::read(root, &["rev-parse", "--show-toplevel", "--show-prefix"])?;
    let answer = String::from_utf8_lossy(&answer.output);
    let mut lines = answer.lines();
    let (Some(top), Some(prefix)) = (lines.next(), lines.next()) else {
        return Ok(());
    };
    let pathspec = format!(":(top,icase){prefix}");
    let held = git::read(
        root,
        &["ls-files", "-z", "--cached", "--full-name", "--", &pathspec],
    )?;
    let depth = prefix.trim_end_matches('/').split('/').count();
    let spellings: BTreeSet<String> = held
        .output
        .split(|byte| *byte == 0)
        .filter_map(|path| std::str::from_utf8(path).ok())
        .filter(|path| !path.starts_with(prefix))
        .map(|path| path.split('/').take(depth).collect::<Vec<_>>().join("/"))
        .filter(|spelling| lowered(&format!("{spelling}/")) == lowered(prefix))
        .collect();

    for spelling in spellings {
        if one_folder(root, &Path::new(top).join(&spelling)) {
            return Err(Stop::new(format!(
                "the root {} is spelt `{}` on disk, and git holds the files under it as `{spelling}`; rename the \
                 folder on disk to git's spelling, or record the disk's with `git mv`, and run again",
                root.display(),
                prefix.trim_end_matches('/')
            )));
        }
    }

    Ok(())
}

/// Whether two paths are one folder on disk, by their identities (spec LIST-11).
fn one_folder(first: &Path, second: &Path) -> bool {
    let folder =
        |path: &Path| fs::symlink_metadata(path).is_ok_and(|facts| facts.file_type().is_dir());

    folder(first) && folder(second) && same_file::is_same_file(first, second).unwrap_or(false)
}

/// A path with each character put in lower case by itself, so that no letter's mapping depends on its
/// neighbours (spec LIST-10).
fn lowered(path: &str) -> String {
    path.chars().flat_map(char::to_lowercase).collect()
}

/// Whether two paths are one file on disk (spec LIST-10). Both are looked at first without following a
/// link, and are compared only when both are ordinary files, so no link is followed and no special file
/// is opened. When either cannot be looked at or opened, they are not taken for one file.
fn one_file(first: &Path, second: &Path) -> bool {
    let ordinary =
        |path: &Path| fs::symlink_metadata(path).is_ok_and(|facts| facts.file_type().is_file());

    ordinary(first) && ordinary(second) && same_file::is_same_file(first, second).unwrap_or(false)
}

/// What git records a tracked path as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    /// A file, which is read.
    File,
    /// A symbolic link, which is never followed or read, but is on disk.
    Link,
    /// A submodule, which is never entered.
    Submodule,
}

/// Asks git for the file list of the root. The files come back sorted by path, byte by byte; what was
/// left out on the way is added to the notices.
pub fn list(root: &Path, notices: &mut Vec<Notice>) -> Result<Vec<File>, Stop> {
    root_spelling(root)?;

    let answer = git::read(root, &LIST)?;

    if let Some(warning) = answer.warning {
        notices.push(Notice::new(
            "git-warning",
            None,
            format!("git warned while it listed the files: {warning}"),
        ));
    }

    // The tracked names with what each is, the untracked names, and the tracked names that are missing
    // from the working tree.
    let mut tracked: BTreeMap<String, Entry> = BTreeMap::new();
    let mut untracked: BTreeSet<String> = BTreeSet::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let mut links = 0_usize;
    let mut unreadable_names = 0_usize;

    for record in answer
        .output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Ok(record) = std::str::from_utf8(record) else {
            unreadable_names += 1;

            continue;
        };

        if let Some(name) = record.strip_prefix("? ") {
            // A name that ends in a slash is a nested repository, which is never entered (spec LIST-3).
            if !name.ends_with('/') {
                untracked.insert(name.to_owned());
            }

            continue;
        }

        let entry = record.split_once('\t').and_then(|(facts, name)| {
            let mut facts = facts.split(' ');

            Some((facts.next()?, facts.next()?, name))
        });
        let Some((letter, mode, name)) = entry else {
            return Err(Stop::new(format!(
                "git listed an entry the analyzer cannot read, `{record}`; this is a fault in the analyzer, \
                 so report it with the git version"
            )));
        };

        // `R` marks a tracked file missing from the working tree, and `S` one a sparse checkout left out.
        if matches!(letter, "R" | "S") {
            missing.insert(name.to_owned());
        }

        if mode == LINK_MODE {
            links += usize::from(tracked.get(name) != Some(&Entry::Link));
            tracked.insert(name.to_owned(), Entry::Link);
        } else {
            tracked
                .entry(name.to_owned())
                .or_insert(if mode == SUBMODULE_MODE {
                    Entry::Submodule
                } else {
                    Entry::File
                });
        }
    }

    // A symbolic link is never followed and never read, tracked or not (spec LIST-3).
    untracked.retain(|name| {
        let link =
            fs::symlink_metadata(root.join(name)).is_ok_and(|facts| facts.file_type().is_symlink());

        links += usize::from(link);

        !link
    });

    // An untracked path that is the same file on disk as a tracked file in the list is that file under
    // the disk's spelling, and is left out (spec LIST-10). The tracked files whose paths differ from it
    // in letter case alone are the candidates; the files' identities decide.
    let mut candidates: BTreeMap<String, Vec<&String>> = BTreeMap::new();

    for (name, entry) in &tracked {
        if *entry == Entry::File && !missing.contains(name) {
            candidates.entry(lowered(name)).or_default().push(name);
        }
    }

    let mut pairs: Vec<(String, String)> = Vec::new();

    untracked.retain(|name| {
        let held = candidates.get(&lowered(name)).and_then(|held| {
            held.iter()
                .find(|held| one_file(&root.join(name), &root.join(held)))
        });

        match held {
            Some(held) => {
                pairs.push(((*held).clone(), name.clone()));

                false
            }
            None => true,
        }
    });

    if let Some((held, other)) = pairs.first() {
        notices.push(Notice::new(
            "paths-differ-in-case",
            Some(held.clone()),
            format!(
                "{} untracked path(s) were left out of the file list because each is a tracked file under \
                 another spelling; the first is `{other}`, which git holds as `{held}`. A rename git was not \
                 told of, or an index that spells a folder two ways, causes this; the spellings agree again \
                 once the file or folder is renamed on disk to git's spelling, or git is given the disk's \
                 with `git mv`.",
                pairs.len()
            ),
        ));
    }

    if links > 0 {
        notices.push(Notice::new(
            "links-left-out",
            None,
            format!("{links} symbolic link(s) were left out of the file list; a link is never followed or read."),
        ));
    }

    if unreadable_names > 0 {
        notices.push(Notice::new(
            "names-left-out",
            None,
            format!("{unreadable_names} file(s) whose names are not valid UTF-8 were left out of the file list."),
        ));
    }

    let mut files: Vec<File> = tracked
        .into_iter()
        .filter(|(name, entry)| *entry == Entry::File && !missing.contains(name))
        .map(|(path, _)| (path, true))
        .chain(untracked.into_iter().map(|path| (path, false)))
        .map(|(path, tracked)| File {
            kind: Kind::of(&path),
            path,
            tracked,
        })
        .collect();

    files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));

    Ok(files)
}
```

```rust path=src/notice.rs
//! Notices: what a person should know about a run that went on (spec RUN-5).

use serde_json::{Value, json};

/// Something a person should know about a run that went on.
#[derive(Debug)]
pub struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
}

impl Notice {
    /// A notice of the kind given, about a file or about none.
    pub const fn new(kind: &'static str, file: Option<String>, sentence: String) -> Self {
        Self {
            kind,
            file,
            sentence,
        }
    }

    /// The notice as it is printed.
    fn printed(&self) -> Value {
        json!({ "file": self.file, "kind": self.kind, "sentence": self.sentence })
    }
}

/// The notices as they are printed: those about no file first, then by file, by kind and by sentence
/// (spec RUN-8).
pub fn printed(mut notices: Vec<Notice>) -> Vec<Value> {
    notices.sort_by(|a, b| (&a.file, a.kind, &a.sentence).cmp(&(&b.file, b.kind, &b.sentence)));

    notices.iter().map(Notice::printed).collect()
}
```

```rust path=src/hand_off.rs
//! Reads the hand-off file yak writes for one checkout (spec HO-1 to HO-4).
//!
//! The file is checked in a fixed order, so that two faults in one file always give the same line: that
//! it can be read, its format version, the shape of each field, and then each stack's interface version
//! (spec RUN-4).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::stop::Stop;
use crate::{HAND_OFF_FORMATS, INTERFACE_VERSION};

/// The largest whole number the format allows, which is the largest JSON carries exactly.
const LARGEST_NUMBER: u64 = 9_007_199_254_740_991;

/// The lengths, in hexadecimal digits, a commit's full name can have.
const COMMIT_NAME_LENGTHS: [usize; 2] = [40, 64];

/// What the analyzer takes from a hand-off file of format 1. Every field of the format is checked when
/// the file is read; a field is kept here once a part of the analyzer uses it.
#[derive(Debug)]
pub struct HandOff {
    /// The checkout's root, which is the folder to read whatever folder the analyzer was started in.
    pub root: PathBuf,
    /// Whether yak's detection stopped at its limit, so the folders it lists are not the whole workspace.
    pub truncated: bool,
    /// The stacks that carry declarations, in the order yak applies them.
    pub stacks: Vec<Stack>,
}

/// One stack and the declarations it carries, the pack's kept apart from the project's.
#[derive(Debug)]
pub struct Stack {
    /// The stack's name.
    pub name: String,
    /// The interface version the stack's declarations were written for.
    pub interface_version: Option<u64>,
    /// The pack's declarations: each key with its value.
    pub pack: Map<String, Value>,
    /// The project's own declarations under this stack.
    pub project: Map<String, Value>,
}

/// Reads and checks the hand-off file at the path given.
pub fn read(path: &Path) -> Result<HandOff, Stop> {
    let shown = path.display();
    let bytes = fs::read(path).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} cannot be read ({error}); pass the path of the file yak wrote for this \
             checkout"
        ))
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let document: Value = serde_json::from_slice(text).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} is not JSON (line {}, column {}); yak writes this file, so write it again \
             with yak",
            error.line(),
            error.column()
        ))
    })?;
    let shape = |problem: String| {
        Stop::new(format!(
            "the hand-off file {shown} is not format 1: {problem}; yak writes this file, so write it again with \
             yak"
        ))
    };
    let fields = document
        .as_object()
        .ok_or_else(|| shape("the file must hold one object".to_owned()))?;

    format_version(fields, &shown.to_string()).map_err(|stopped| match stopped {
        Fault::Stop(stop) => stop,
        Fault::Shape(problem) => shape(problem),
    })?;

    // The fields are checked in the order the format's table lists them (spec HO-2).
    let root = absolute_path(fields, "root").map_err(shape)?;

    folders(fields).map_err(shape)?;

    let truncated = truncated(fields).map_err(shape)?;
    let stacks = stacks(fields).map_err(shape)?;

    absolute_path(fields, "siblings").map_err(shape)?;
    base(fields).map_err(shape)?;

    let handed = HandOff {
        root,
        truncated,
        stacks,
    };

    for stack in &handed.stacks {
        interface_version(stack, &shown.to_string())?;
    }

    Ok(handed)
}

/// What can be wrong with the format version: a format the analyzer does not read, or a shape.
enum Fault {
    /// The file states a format the analyzer does not read.
    Stop(Stop),
    /// The field is missing or is not a whole number.
    Shape(String),
}

/// Checks the format version, before any other field is looked at (spec HO-3).
fn format_version(fields: &Map<String, Value>, shown: &str) -> Result<(), Fault> {
    let stated = fields
        .get("formatVersion")
        .ok_or_else(|| Fault::Shape("`formatVersion` is missing".to_owned()))?
        .as_f64()
        .filter(|number| number.fract() == 0.0)
        .ok_or_else(|| Fault::Shape("`formatVersion` must be a whole number".to_owned()))?;
    let reads = HAND_OFF_FORMATS.map(|format| format.to_string()).join(", ");
    let oldest = f64::from(HAND_OFF_FORMATS[0]);
    let newest = f64::from(HAND_OFF_FORMATS[HAND_OFF_FORMATS.len() - 1]);

    if stated < oldest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, older than any this analyzer reads ({reads}); update \
             yak"
        ))));
    }

    if stated > newest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, newer than any this analyzer reads ({reads}); update \
             the analyzer's version pinned in the pack"
        ))));
    }

    Ok(())
}

/// A field that must be text holding an absolute path.
fn absolute_path(fields: &Map<String, Value>, name: &str) -> Result<PathBuf, String> {
    let text = fields
        .get(name)
        .ok_or_else(|| format!("`{name}` is missing"))?
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("`{name}` must be text that is not empty"))?;

    let path = PathBuf::from(text);

    if path.is_absolute() {
        Ok(path)
    } else {
        Err(format!("`{name}` must be an absolute path"))
    }
}

/// The object under `detection`, which allows two fields and no other.
fn detection(fields: &Map<String, Value>) -> Result<&Map<String, Value>, String> {
    let detection = fields
        .get("detection")
        .ok_or_else(|| "`detection` is missing".to_owned())?
        .as_object()
        .ok_or_else(|| "`detection` must be an object".to_owned())?;

    match detection
        .keys()
        .find(|key| !matches!(key.as_str(), "folders" | "truncated"))
    {
        Some(other) => Err(format!("`detection.{other}` is not a field of the format")),
        None => Ok(detection),
    }
}

/// Checks the folders yak's detection looked in: at least one, the first `.`, each inside the root.
fn folders(fields: &Map<String, Value>) -> Result<(), String> {
    let listed = detection(fields)?
        .get("folders")
        .ok_or_else(|| "`detection.folders` is missing".to_owned())?
        .as_array()
        .filter(|listed| !listed.is_empty())
        .ok_or_else(|| "`detection.folders` must be a list of at least one path".to_owned())?;

    for (index, folder) in listed.iter().enumerate() {
        let folder = folder
            .as_str()
            .filter(|folder| !folder.is_empty())
            .ok_or_else(|| {
                format!("`detection.folders[{index}]` must be text that is not empty")
            })?;

        if leaves_its_folder(folder) || folder.contains('\\') {
            return Err(format!(
                "`detection.folders[{index}]` must be a path inside the root, written with forward slashes"
            ));
        }
    }

    if listed[0] == "." {
        Ok(())
    } else {
        Err("`detection.folders` must begin with `.`, the root itself".to_owned())
    }
}

/// Whether yak's detection stopped at its limit.
fn truncated(fields: &Map<String, Value>) -> Result<bool, String> {
    detection(fields)?
        .get("truncated")
        .ok_or_else(|| "`detection.truncated` is missing".to_owned())?
        .as_bool()
        .ok_or_else(|| "`detection.truncated` must be true or false".to_owned())
}

/// The stacks, each of which allows four fields and no other.
fn stacks(fields: &Map<String, Value>) -> Result<Vec<Stack>, String> {
    let listed = fields
        .get("stacks")
        .ok_or_else(|| "`stacks` is missing".to_owned())?
        .as_array()
        .ok_or_else(|| "`stacks` must be a list".to_owned())?;
    let mut stacks = Vec::with_capacity(listed.len());

    for (index, stack) in listed.iter().enumerate() {
        let stack = stack
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}]` must be an object"))?;
        let name = stack
            .get("name")
            .ok_or_else(|| format!("`stacks[{index}].name` is missing"))?
            .as_str()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| format!("`stacks[{index}].name` must be text that is not empty"))?;
        let interface_version = match stack.get("interfaceVersion") {
            Some(version) => Some(
                version
                    .as_u64()
                    .filter(|version| (1..=LARGEST_NUMBER).contains(version))
                    .ok_or_else(|| {
                        format!(
                            "`stacks[{index}].interfaceVersion` must be a positive whole number"
                        )
                    })?,
            ),
            None => None,
        };
        let pack = stack
            .get("pack")
            .ok_or_else(|| format!("`stacks[{index}].pack` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].pack` must be an object"))?;
        let project = stack
            .get("project")
            .ok_or_else(|| format!("`stacks[{index}].project` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].project` must be an object"))?;

        if let Some((key, _)) = project.iter().find(|(_, value)| !is_project_value(value)) {
            return Err(format!(
                "`stacks[{index}].project.{key}` must be a list of text or a positive whole number"
            ));
        }

        if let Some(other) = stack.keys().find(|key| {
            !matches!(
                key.as_str(),
                "name" | "interfaceVersion" | "pack" | "project"
            )
        }) {
            return Err(format!(
                "`stacks[{index}].{other}` is not a field of the format"
            ));
        }

        stacks.push(Stack {
            name: name.to_owned(),
            interface_version,
            pack: pack.clone(),
            project: project.clone(),
        });
    }

    Ok(stacks)
}

/// Whether a value is one the format lets a project state: a list of text, or a positive whole number.
fn is_project_value(value: &Value) -> bool {
    match value {
        Value::Array(entries) => entries.iter().all(Value::is_string),
        other => other
            .as_u64()
            .is_some_and(|number| (1..=LARGEST_NUMBER).contains(&number)),
    }
}

/// Checks the base commit, where the file states one: a commit's full name in lower-case hexadecimal
/// digits.
fn base(fields: &Map<String, Value>) -> Result<(), String> {
    let Some(base) = fields.get("base") else {
        return Ok(());
    };

    base.as_str()
        .filter(|name| {
            COMMIT_NAME_LENGTHS.contains(&name.len())
                && name
                    .bytes()
                    .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
        })
        .map(|_| ())
        .ok_or_else(|| {
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits".to_owned()
        })
}

/// Checks that a stack's declarations were written for the interface version this analyzer speaks, and
/// that a stack which states no version holds no declarations (spec HO-4).
fn interface_version(stack: &Stack, shown: &str) -> Result<(), Stop> {
    let name = &stack.name;

    match stack.interface_version {
        Some(INTERFACE_VERSION) => Ok(()),
        Some(stated) => {
            let update = if stated > INTERFACE_VERSION {
                "the analyzer's version pinned in the pack"
            } else {
                "the stack's declarations"
            };

            Err(Stop::new(format!(
                "the stack `{name}` in {shown} states declarations for interface version {stated}, and this \
                 analyzer speaks version {INTERFACE_VERSION}; update {update}"
            )))
        }
        None if stack.pack.is_empty() && stack.project.is_empty() => Ok(()),
        None => Err(Stop::new(format!(
            "the stack `{name}` in {shown} holds declarations and states no interface version; state the \
             version they were written for"
        ))),
    }
}

/// Whether a path written with forward slashes is absolute or steps out of its folder with `..`.
pub fn leaves_its_folder(path: &str) -> bool {
    let has_drive = path.as_bytes().get(1) == Some(&b':');

    path.starts_with('/') || has_drive || path.split('/').any(|segment| segment == "..")
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file and the file list: the list of packages is empty until the part
//! that fills it is built.

use std::env;
use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::files::{self, File};
use crate::hand_off;
use crate::notice::{self, Notice};
use crate::stop::Stop;

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
    let mut notices = Vec::new();

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice::new(
                    "detection-cut-short",
                    None,
                    "yak's detection stopped at its limit of folders, so the folders the hand-off file lists \
                     are not the whole workspace."
                        .to_owned(),
                ));
            }

            handed.root
        }
        None => {
            notices.push(Notice::new(
                "no-hand-off-file",
                None,
                "No hand-off file was given, so the project is read from its own standard files, with no \
                 declarations."
                    .to_owned(),
            ));

            env::current_dir().map_err(|error| {
                Stop::new(format!(
                    "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
                ))
            })?
        }
    };
    let files = files::list(&root, &mut notices)?;
    let document = json!({
        "files": files.iter().map(file).collect::<Vec<Value>>(),
        "handOff": hand_off_path.is_some(),
        "notices": notice::printed(notices),
        "packages": [],
    });

    Ok(format!("{document:#}"))
}

/// One file as it is printed.
fn file(file: &File) -> Value {
    json!({ "kind": file.kind.name(), "path": file.path, "tracked": file.tracked })
}
```

```rust path=src/main.rs
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: so far its hand-off file and
//! its list of files. It reports no findings yet.

mod cli;
mod declarations;
mod files;
mod git;
mod graph;
mod hand_off;
mod notice;
mod stop;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;

/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
        eprintln!(
            "{}",
            stop::line(&format!(
                "a fault in the analyzer itself, not in the project: {fault}"
            ))
        );
    }));

    panic::catch_unwind(run).unwrap_or_else(|_| ExitCode::from(stop::FAULT))
}

/// Does what the command line asks.
fn run() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match cli::parse(&args) {
        Ok(Command::Version) => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        Ok(Command::Interface) => {
            println!(
                "{}",
                serde_json::json!({ "handOffFormats": HAND_OFF_FORMATS, "interfaceVersion": INTERFACE_VERSION })
            );

            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => match graph::read(hand_off.as_deref()) {
            Ok(document) => {
                println!("{document}");

                ExitCode::SUCCESS
            }
            Err(stopped) => {
                eprintln!("{}", stopped.line());

                ExitCode::from(stop::STOPPED)
            }
        },
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

            ExitCode::from(stop::REFUSED)
        }
    }
}
```

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0. The tests report 6 passed for `cli`, 5 for `declarations`, 16 for
`file-list`, 10 for `hand-off` and 2 for `hand-off-schema`.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src tests
git commit -m "feat(files): read the file list through git" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: one commit on `feat/reading-2-file-list`.

### Task 2: the data and generated declarations, and packages

**Implements:** KEY-3, KEY-4, KEY-5, PACK-1 to PACK-6, RUN-4, DEP-1

**Test:** `cargo test --release --locked --test packages --test file-list --test hand-off --bins`

**Files:**
- Create: `src/glob.rs`, `src/marks.rs`, `src/packages.rs`
- Modify: `src/declarations.rs`, `src/hand_off.rs`, `src/graph.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`
- Test: `tests/packages.test.rs`, `tests/file-list.test.rs`, `tests/hand-off.test.rs`

**Interfaces:**
- Consumes: Task 1's `files::list`, `files::File`, `files::Kind`, `notice::Notice` and
  `tests/support/project.rs`; the first plan's `hand_off::HandOff` and its stacks.
- Produces: `glob::Glob`; `marks::apply` and `marks::Marks`; `packages::read`, `packages::Package`
  and `packages::Packages`; `hand_off::HandOff::folders`; the glob check of `declarations::check`
  made by `Glob::new`.

- [ ] **Step 1: Write the failing tests**

The tests of the first plan and of Task 1 that compare a whole document change with it: a file now
has its marks and its package, and a glob that matches nothing has a notice.

```rust path=tests/packages.test.rs
//! Runs the built binary on made-up projects and checks the `data` and `generated` declarations and the
//! packages it prints, and how it stops on a standard file it cannot read (spec KEY-3, KEY-4, KEY-5,
//! PACK-1 to PACK-6, RUN-4).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// One stack named `typescript` that holds the declarations given, the pack's and the project's.
fn stack(pack: Value, project: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": project }])
}

/// The files a document marks as the key given, `data` or `generated`.
fn marked<'a>(document: &'a Value, key: &str) -> Vec<&'a str> {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .filter(|file| file[key] == json!(true))
        .map(|file| file["path"].as_str().expect("a path"))
        .collect()
}

/// The package each file belongs to, by the file's path.
fn package_of<'a>(document: &'a Value, path: &str) -> &'a Value {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .map_or(&Value::Null, |file| &file["package"])
}

#[test]
fn marks_data_and_generated_files_a_packs_globs_from_each_folder_and_a_projects_from_the_root() {
    let project = Project::new("marks");

    project
        .write("fixtures/a.json", "")
        .write("fixtures/broken/package.json", "{ not json")
        .write("fixtures/broken/tsconfig.json", "{ not json")
        .write("packages/app/fixtures/b.ts", "")
        .write("packages/app/src/x.ts", "")
        .write("other/fixtures/c.ts", "")
        .write("src/gen/types.ts", "")
        .write("src/main.ts", "")
        .commit();

    let declared = stack(
        json!({ "data": ["fixtures/**"], "generated": ["**/*.never"] }),
        json!({ "generated": ["src/gen/**"] }),
    );
    let file = project.hand_off(&project.root, &[".", "packages/app"], declared);
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // The pack's glob applies under each folder detection looked in, and not under `other/`.
    assert_eq!(
        marked(&document, "data"),
        [
            "fixtures/a.json",
            "fixtures/broken/package.json",
            "fixtures/broken/tsconfig.json",
            "packages/app/fixtures/b.ts"
        ]
    );
    assert_eq!(marked(&document, "generated"), ["src/gen/types.ts"]);

    // A broken `package.json` among data makes no package and stops nothing (spec KEY-4).
    assert_eq!(document["packages"], json!([]));
    assert_eq!(notices(&document), ["glob-matches-nothing"]);
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The `generated` glob `**/*.never` that the pack states for the stack `typescript` matches no file."
        )
    );
}

#[test]
fn makes_a_package_of_every_package_json_and_names_the_workspace_ones() {
    let project = Project::new("packages-npm");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*", "./tools/", "!packages/excluded"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write("packages/a/src/x.ts", "")
        .write(
            "packages/excluded/package.json",
            r#"{ "name": "excluded" }"#,
        )
        .write("tools/package.json", "\u{feff}{}")
        .write("examples/demo/package.json", r#"{ "name": "a" }"#)
        .write("examples/demo/index.ts", "")
        .write("README.md", "")
        .commit();

    let document = project.graph();
    let package = |folder: &str, name: Value, workspace: bool, detected: bool| {
        json!({
            "detected": detected,
            "folder": folder,
            "name": name,
            "workspace": workspace,
        })
    };

    // Without a hand-off file the detected folders are the root and the workspace's (spec PACK-4).
    assert_eq!(
        document["packages"],
        json!([
            package(".", json!("root"), true, true),
            package("examples/demo", json!("a"), false, false),
            package("packages/a", json!("a"), true, true),
            package("packages/excluded", json!("excluded"), false, false),
            package("tools", Value::Null, true, true),
        ])
    );

    // A stray package of a workspace package's name is no workspace package, so it is no repeat.
    assert_eq!(notices(&document), ["no-hand-off-file"]);
    assert_eq!(
        package_of(&document, "packages/a/src/x.ts"),
        &json!("packages/a")
    );
    assert_eq!(
        package_of(&document, "examples/demo/index.ts"),
        &json!("examples/demo")
    );
    assert_eq!(package_of(&document, "README.md"), &json!("."));
}

#[test]
fn reads_the_workspace_from_an_object_and_from_pnpms_list() {
    let object = Project::new("packages-object");

    object
        .write(
            "package.json",
            r#"{ "workspaces": { "packages": ["libs/*"], "nohoist": ["**"] } }"#,
        )
        .write("libs/one/package.json", r#"{ "name": "one" }"#)
        .commit();

    let pnpm = Project::new("packages-pnpm");

    pnpm.write("package.json", "{}")
        .write(
            "pnpm-workspace.yaml",
            "# the workspace\npackages:\n  - 'apps/*'\n  - \"!apps/legacy\"\n",
        )
        .write("apps/web/package.json", r#"{ "name": "web" }"#)
        .write("apps/legacy/package.json", r#"{ "name": "legacy" }"#)
        .commit();

    let workspace = |document: &Value| -> Vec<String> {
        document["packages"]
            .as_array()
            .expect("a list of packages")
            .iter()
            .filter(|package| package["workspace"] == json!(true))
            .map(|package| package["folder"].as_str().expect("a folder").to_owned())
            .collect()
    };

    assert_eq!(workspace(&object.graph()), [".", "libs/one"]);
    assert_eq!(workspace(&pnpm.graph()), [".", "apps/web"]);
}

#[test]
fn takes_the_folders_a_hand_off_file_lists_as_detected_and_as_workspace_packages() {
    let project = Project::new("packages-detection");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write("apps/web/package.json", r#"{ "name": "web" }"#)
        .commit();

    let file = project.hand_off(&project.root, &[".", "apps/web"], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let facts: Vec<(&str, bool, bool)> = document["packages"]
        .as_array()
        .expect("a list of packages")
        .iter()
        .map(|package| {
            (
                package["folder"].as_str().expect("a folder"),
                package["workspace"] == json!(true),
                package["detected"] == json!(true),
            )
        })
        .collect();

    assert_eq!(
        facts,
        [
            (".", true, true),
            ("apps/web", true, true),
            ("packages/a", true, false)
        ]
    );
}

#[test]
fn says_when_two_workspace_packages_have_one_name() {
    let project = Project::new("packages-repeated");

    project
        .write("package.json", r#"{ "workspaces": ["packages/*"] }"#)
        .write("packages/first/package.json", r#"{ "name": "same" }"#)
        .write("packages/second/package.json", r#"{ "name": "same" }"#)
        .commit();

    let document = project.graph();

    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "package-name-repeated"]
    );
    assert_eq!(
        document["notices"][1],
        json!({
            "file": "packages/second/package.json",
            "kind": "package-name-repeated",
            "sentence": "The workspace packages in `packages/first` and `packages/second` are both named `same`; \
                         an import of `same` leads to the first, `packages/first`."
        })
    );
}

#[test]
fn leaves_a_file_with_no_package_above_it_in_no_package() {
    let project = Project::new("packages-none");

    project
        .write("web/package.json", "{}")
        .write("web/main.ts", "")
        .write("server/main.ts", "")
        .commit();

    let document = project.graph();

    assert_eq!(
        paths(&document),
        ["server/main.ts", "web/main.ts", "web/package.json"]
    );
    assert_eq!(package_of(&document, "server/main.ts"), &Value::Null);
    assert_eq!(package_of(&document, "web/main.ts"), &json!("web"));
}

#[test]
fn stops_on_a_package_json_or_a_workspace_list_it_cannot_read() {
    let cases: [(&str, &str, &str, &str); 4] = [
        (
            "packages-stop-json",
            "package.json",
            "{ \"name\": ",
            "yak-analyzer: the package.json package.json cannot be read as a JSON object (line 1, column 10); fix \
             it, or declare it as data if it is broken on purpose",
        ),
        (
            "packages-stop-array",
            "lib/package.json",
            "[]",
            "yak-analyzer: the package.json lib/package.json cannot be read as a JSON object (it holds something \
             other than an object); fix it, or declare it as data if it is broken on purpose",
        ),
        (
            "packages-stop-yaml",
            "pnpm-workspace.yaml",
            "packages: [\n",
            "yak-analyzer: the workspace's list pnpm-workspace.yaml cannot be read (",
        ),
        (
            "packages-stop-utf8",
            "package.json",
            "\u{0}",
            "yak-analyzer: the package.json package.json cannot be read as a JSON object (line 1, column 1)",
        ),
    ];

    for (name, path, content, expected) in cases {
        let project = Project::new(name);

        project.write(path, content).commit();

        let line = stopped(&project.run(|_| {}));

        assert!(line.starts_with(expected), "{name}: {line}");
    }
}

#[test]
fn stops_on_a_field_of_a_form_it_does_not_have() {
    let cases: [(&str, &str, &str, &str); 5] = [
        (
            "packages-field-name",
            "lib/package.json",
            r#"{ "name": 4 }"#,
            "yak-analyzer: the field `name` in lib/package.json holds a value of a form it does not have; it \
             must be text; fix it",
        ),
        (
            "packages-field-workspaces",
            "package.json",
            r#"{ "workspaces": 4 }"#,
            "yak-analyzer: the field `workspaces` in package.json holds a value of a form it does not have; it \
             must be a list of globs, or an object whose `packages` is one; fix it",
        ),
        (
            "packages-field-workspace-entry",
            "package.json",
            r#"{ "workspaces": ["packages/*", 4] }"#,
            "yak-analyzer: the field `workspaces` in package.json holds a value of a form it does not have; it \
             must be a list of globs, or an object whose `packages` is one; fix it",
        ),
        (
            "packages-field-pnpm",
            "pnpm-workspace.yaml",
            "packages: apps/*\n",
            "yak-analyzer: the field `packages` in pnpm-workspace.yaml holds a value of a form it does not \
             have; it must be a list of globs; fix it",
        ),
        (
            "packages-field-glob",
            "package.json",
            r#"{ "workspaces": ["packages/[a"] }"#,
            "yak-analyzer: a glob of the workspace's list cannot be read: the glob `packages/[a` has a bracket \
             left open; fix it in package.json or pnpm-workspace.yaml",
        ),
    ];

    for (name, path, content, expected) in cases {
        let project = Project::new(name);

        project.write(path, content).commit();
        assert_eq!(stopped(&project.run(|_| {})), expected, "{name}");
    }
}

#[test]
fn reports_an_unreadable_package_json_before_an_unreadable_workspace_list_and_either_before_a_field()
 {
    let project = Project::new("packages-order");

    project
        .write("package.json", r#"{ "name": 4 }"#)
        .write("pnpm-workspace.yaml", "packages: [\n")
        .write("z/package.json", "{")
        .commit();

    let line = stopped(&project.run(|_| {}));

    assert!(
        line.starts_with("yak-analyzer: the package.json z/package.json cannot be read"),
        "{line}"
    );
}
```

```rust path=tests/file-list.test.rs
//! Runs the built binary on made-up projects and checks the file list it prints, how it stops, and
//! what it leaves untouched (spec LIST-1 to LIST-4, LIST-6 to LIST-8, LIST-10, RUN-4, RUN-6, RUN-6a,
//! FIX-1, FIX-6, FIX-7).

#[path = "support/project.rs"]
mod project;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// Records a path in git's index with the mode and content given, commits it and checks it out, as a
/// checkout of a repository that already held it would.
fn record(project: &Project, mode: &str, path: &str, content: &str) {
    let source = project.home.join("recorded-content");

    fs::write(&source, content).expect("the content can be written");

    let blob = project.git(&[
        "hash-object",
        "-w",
        source.to_str().expect("the test's folder is valid UTF-8"),
    ]);

    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("{mode},{},{path}", blob.trim()),
    ]);
    project.git(&["commit", "--quiet", "--message", "a recorded path"]);
    project.git(&["checkout", "--", path]);
}

/// Records a symbolic link to the target given, as git records one, whatever the system checks out for
/// a link.
fn link(project: &Project, path: &str, target: &str) {
    record(project, "120000", path, target);
}

/// Makes the folder given a git repository of its own, inside the project, with one commit.
fn nest(project: &Project, folder: &str) {
    project.git(&["-C", folder, "init", "--quiet"]);
    project.git(&["-C", folder, "add", "--all"]);
    project.git(&["-C", folder, "commit", "--quiet", "--message", "nested"]);
}

/// The content of every file under a folder, `.git` included, by its path.
fn snapshot(folder: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![folder.to_path_buf()];

    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).expect("a folder can be listed") {
            let path = entry.expect("an entry can be read").path();

            if path.is_dir() {
                waiting.push(path);
            } else {
                found.insert(path.clone(), fs::read(&path).expect("a file can be read"));
            }
        }
    }

    found
}

#[test]
fn lists_tracked_files_and_untracked_files_that_nothing_ignores_each_with_its_kind() {
    let project = Project::new("list-kinds");

    project
        .write(".gitignore", "dist/\n*.log\n")
        .write("package.json", "{}")
        .write("tsconfig.json", "{}")
        .write("tsconfig.build.json", "{}")
        .write("tools/jsconfig.json", "{}")
        .write("src/main.ts", "")
        .write("src/view.tsx", "")
        .write("src/types.d.ts", "")
        .write("src/App.vue", "")
        .write("src/styles.css", "")
        .write("docs/intro.mdx", "")
        .write("public/index.html", "")
        .write("lib/old.cjs", "")
        .write("Zeta.mts", "")
        .commit()
        .write("scratch.ts", "")
        .write("dist/out.js", "")
        .write("debug.log", "")
        .write("local/only.ts", "")
        .write(".git/info/exclude", "local/\n");

    let document = project.graph();
    let file = |path: &str, kind: &str, tracked: bool| {
        json!({
            "data": false,
            "generated": false,
            "kind": kind,
            "package": ".",
            "path": path,
            "tracked": tracked,
        })
    };

    // By path, byte by byte: a dot and capital letters come before small letters (spec RUN-8).
    assert_eq!(
        document["files"],
        json!([
            file(".gitignore", "other", true),
            file("Zeta.mts", "script", true),
            file("docs/intro.mdx", "mdx", true),
            file("lib/old.cjs", "script", true),
            file("package.json", "manifest", true),
            file("public/index.html", "page", true),
            file("scratch.ts", "script", false),
            file("src/App.vue", "vue", true),
            file("src/main.ts", "script", true),
            file("src/styles.css", "other", true),
            file("src/types.d.ts", "declaration", true),
            file("src/view.tsx", "script", true),
            file("tools/jsconfig.json", "tsconfig", true),
            file("tsconfig.build.json", "tsconfig", true),
            file("tsconfig.json", "tsconfig", true),
        ])
    );
    assert_eq!(document["handOff"], json!(false));
    assert_eq!(notices(&document), ["no-hand-off-file"]);
    assert_eq!(
        document["packages"],
        json!([{ "detected": true, "folder": ".", "name": null, "workspace": true }])
    );
}

#[test]
fn never_reaches_into_a_nested_repository_a_submodule_or_another_worktree() {
    let other = Project::new("list-nested-other");

    other.write("other.ts", "").commit();

    let project = Project::new("list-nested");

    project.write("src/main.ts", "").commit();
    project.git(&[
        "submodule",
        "--quiet",
        "add",
        other.root.to_str().expect("valid UTF-8"),
        "sub",
    ]);
    project.git(&["commit", "--quiet", "--message", "a submodule"]);
    project.git(&[
        "worktree",
        "add",
        "--quiet",
        ".worktrees/second",
        "-b",
        "second",
    ]);
    project
        .write(".git/info/exclude", ".worktrees/\n")
        .write("nested/inner.ts", "");
    nest(&project, "nested");

    assert_eq!(paths(&project.graph()), [".gitmodules", "src/main.ts"]);
}

#[test]
fn leaves_a_symbolic_link_out_and_says_how_many() {
    let project = Project::new("list-links");

    project.write("src/main.ts", "").commit();
    link(&project, "link-to-main", "src/main.ts");

    let document = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
    assert!(
        document["notices"][0]["sentence"]
            .as_str()
            .is_some_and(|sentence| sentence.starts_with("1 symbolic link(s) were left out"))
    );
}

/// An untracked link can be made by a test only where the system lets any user make one.
#[cfg(unix)]
#[test]
fn leaves_an_untracked_symbolic_link_out_too() {
    let project = Project::new("list-links-untracked");

    project.write("src/main.ts", "").commit();
    std::os::unix::fs::symlink("src/main.ts", project.root.join("also-main.ts"))
        .expect("a link can be made");

    let document = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
}

#[test]
fn leaves_out_a_tracked_file_that_is_missing_from_the_working_tree() {
    let project = Project::new("list-missing");

    project
        .write("src/main.ts", "")
        .write("src/gone.ts", "")
        .commit();
    fs::remove_file(project.root.join("src/gone.ts")).expect("the file can be removed");

    assert_eq!(paths(&project.graph()), ["src/main.ts"]);
}

#[test]
fn counts_letter_case_in_an_ignore_pattern_on_every_platform() {
    let project = Project::new("list-ignore-case");

    project
        .write(".gitignore", "Build/\n")
        .write("src/main.ts", "")
        .commit()
        .write("build/out.js", "")
        .write("Build/other.js", "");

    // Where the file system ignores case the two folders are one, named as it was first written.
    let listed = project.graph();
    let listed = paths(&listed);

    assert!(listed.contains(&"build/out.js"), "{listed:?}");
    assert!(
        !listed.iter().any(|path| path.starts_with("Build/")),
        "{listed:?}"
    );
}

#[test]
fn leaves_out_an_untracked_path_that_differs_from_a_tracked_file_only_in_letter_case() {
    let project = Project::new("list-case-pair");

    project
        .write("docs/Guide.md", "tracked")
        .write("Assets/logo.txt", "tracked")
        .write("src/main.ts", "")
        .commit();

    // Where the file system ignores case, a pair is one file on disk under another spelling, made by
    // renaming it; where it keeps case apart, a pair is two files.
    let ignores_case = project.root.join("docs/guide.md").exists();

    if ignores_case {
        fs::rename(
            project.root.join("docs/Guide.md"),
            project.root.join("docs/guide.tmp"),
        )
        .expect("a rename");
        fs::rename(
            project.root.join("docs/guide.tmp"),
            project.root.join("docs/guide.md"),
        )
        .expect("a rename");
        fs::rename(project.root.join("Assets"), project.root.join("assets.tmp")).expect("a rename");
        fs::rename(project.root.join("assets.tmp"), project.root.join("assets")).expect("a rename");
    } else {
        project
            .write("docs/guide.md", "untracked")
            .write("assets/logo.txt", "untracked");
    }

    let document = project.graph();

    if !ignores_case {
        // Two real files under each pair of spellings: both are read, and nothing is left out.
        assert_eq!(
            paths(&document),
            [
                "Assets/logo.txt",
                "assets/logo.txt",
                "docs/Guide.md",
                "docs/guide.md",
                "src/main.ts"
            ]
        );
        assert_eq!(notices(&document), ["no-hand-off-file"]);

        return;
    }

    assert_eq!(
        paths(&document),
        ["Assets/logo.txt", "docs/Guide.md", "src/main.ts"]
    );
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "paths-differ-in-case"]
    );

    let notice = &document["notices"][1];
    let sentence = notice["sentence"].as_str().expect("a sentence");

    // One notice for both pairs, naming the first.
    assert_eq!(notice["file"], json!("Assets/logo.txt"));
    assert!(
        sentence.starts_with("2 untracked path(s) were left out"),
        "{sentence}"
    );
    assert!(
        sentence.contains("the first is `assets/logo.txt`, which git holds as `Assets/logo.txt`"),
        "{sentence}"
    );
}

#[test]
fn takes_an_index_that_spells_one_folder_two_ways_as_the_files_git_holds() {
    let project = Project::new("list-case-index");

    // The index holds `Dir/a.ts` and `dir/b.ts`, as a repository made on a system that keeps case
    // apart can; a checkout where case is ignored writes both into one folder.
    project.write("Dir/a.ts", "").commit();
    record(&project, "100644", "dir/b.ts", "");

    let ignores_case = project.root.join("dir/a.ts").exists();
    let document = project.graph();

    assert_eq!(paths(&document), ["Dir/a.ts", "dir/b.ts"]);

    if ignores_case {
        assert_eq!(
            notices(&document),
            ["no-hand-off-file", "paths-differ-in-case"]
        );
        assert!(
            document["notices"][1]["sentence"].as_str().is_some_and(
                |sentence| sentence.contains("`Dir/b.ts`, which git holds as `dir/b.ts`")
            )
        );
    } else {
        assert_eq!(notices(&document), ["no-hand-off-file"]);
    }
}

#[test]
fn lists_a_new_file_in_a_folder_whose_case_differs_under_the_disks_spelling() {
    let project = Project::new("list-case-folder-new");

    project.write("Assets/logo.txt", "tracked").commit();

    let ignores_case = project.root.join("assets/logo.txt").exists();

    if ignores_case {
        fs::rename(project.root.join("Assets"), project.root.join("assets.tmp")).expect("a rename");
        fs::rename(project.root.join("assets.tmp"), project.root.join("assets")).expect("a rename");
    }

    project.write("assets/new.ts", "");

    let document = project.graph();

    // The new file has no tracked path to pair with, so it is listed as the disk spells it.
    assert_eq!(paths(&document), ["Assets/logo.txt", "assets/new.ts"]);

    if ignores_case {
        // The one notice is about the tracked file renamed with its folder, not about the new file.
        assert_eq!(
            notices(&document),
            ["no-hand-off-file", "paths-differ-in-case"]
        );
        assert!(
            document["notices"][1]["sentence"]
                .as_str()
                .is_some_and(|sentence| sentence.starts_with("1 untracked path(s)"))
        );
    } else {
        assert_eq!(notices(&document), ["no-hand-off-file"]);
    }
}

#[test]
fn keeps_a_file_renamed_in_letter_case_alone_under_the_name_its_file_system_gives_it() {
    let project = Project::new("list-case-rename");

    project
        .write("src/Old.ts", "")
        .write("src/main.ts", "")
        .commit();

    let ignores_case = project.root.join("src/old.ts").exists();

    fs::rename(
        project.root.join("src/Old.ts"),
        project.root.join("src/old.tmp"),
    )
    .expect("a rename");
    fs::rename(
        project.root.join("src/old.tmp"),
        project.root.join("src/old.ts"),
    )
    .expect("a rename");

    let document = project.graph();

    if ignores_case {
        // git still finds the tracked name, so the file is listed under git's spelling.
        assert_eq!(paths(&document), ["src/Old.ts", "src/main.ts"]);
        assert_eq!(
            notices(&document),
            ["no-hand-off-file", "paths-differ-in-case"]
        );
    } else {
        // git calls the old name missing and the new one untracked, and the file is read as renamed.
        assert_eq!(paths(&document), ["src/main.ts", "src/old.ts"]);
        assert_eq!(notices(&document), ["no-hand-off-file"]);
    }
}

#[test]
fn reads_only_what_lies_under_a_root_that_is_a_sub_folder_of_the_work_tree() {
    let project = Project::new("list-sub-folder");

    project
        .write("README.md", "")
        .write("server/main.ts", "")
        .write("apps/web/package.json", "{}")
        .write("apps/web/src/main.ts", "")
        .commit()
        .write("apps/web/scratch.ts", "");

    let expected = ["package.json", "scratch.ts", "src/main.ts"];
    let web = project.root.join("apps/web");

    // Started in the sub-folder, with no hand-off file.
    assert_eq!(
        paths(&printed(&project.run(|binary| {
            binary.current_dir(&web);
        }))),
        expected
    );

    // Started elsewhere, with a hand-off file that names the sub-folder: the file decides the root.
    let file = project.hand_off(&web, &["."], json!([]));
    let handed = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(paths(&handed), expected);
    assert_eq!(handed["handOff"], json!(true));
    assert_eq!(notices(&handed), [] as [&str; 0]);
}

#[test]
fn stops_on_a_root_whose_folder_git_spells_otherwise_where_case_is_ignored() {
    let project = Project::new("list-case-root");

    project.write("Apps/web/a.ts", "").commit();

    let ignores_case = project.root.join("apps/web/a.ts").exists();

    if ignores_case {
        fs::rename(project.root.join("Apps"), project.root.join("apps.tmp")).expect("a rename");
        fs::rename(project.root.join("apps.tmp"), project.root.join("apps")).expect("a rename");
    } else {
        // Where case is kept apart, `apps/web` is a folder of its own beside git's `Apps/web`.
        project.write("apps/web/b.ts", "");
    }

    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let output = project.run(|binary| {
        binary.args(["--hand-off", &file]);
    });

    if ignores_case {
        assert_eq!(
            stopped(&output),
            format!(
                "yak-analyzer: the root {} is spelt `apps/web` on disk, and git holds the files under it as \
                 `Apps/web`; rename the folder on disk to git's spelling, or record the disk's with `git mv`, \
                 and run again",
                root.display()
            )
        );
    } else {
        assert_eq!(paths(&printed(&output)), ["b.ts"]);
    }
}

#[test]
fn stops_when_the_root_is_not_a_folder_or_is_not_inside_a_git_work_tree() {
    let project = Project::new("list-no-work-tree");
    let outside = project
        .root
        .parent()
        .expect("the project's home")
        .join("no-repository");
    let absent = project.root.join("not-there");

    fs::create_dir_all(&outside).expect("a folder can be made");

    // The test's own folders lie inside this repository's work tree, so git is told to look no higher.
    let line = stopped(&project.run(|binary| {
        binary.current_dir(&outside).env(
            "GIT_CEILING_DIRECTORIES",
            outside.parent().expect("a parent"),
        );
    }));

    assert!(
        line.starts_with(&format!(
            "yak-analyzer: git cannot read the root {}: ",
            outside.display()
        )),
        "{line}"
    );
    assert!(
        line.ends_with("; run the analyzer inside a git work tree"),
        "{line}"
    );

    let file = project.hand_off(&absent, &["."], json!([]));

    assert_eq!(
        stopped(&project.run(|binary| {
            binary.args(["--hand-off", &file]);
        })),
        format!(
            "yak-analyzer: the root {} is not a folder; run the analyzer in a checkout, or hand it a hand-off \
             file that names one",
            absent.display()
        )
    );
}

#[test]
fn stops_when_git_cannot_be_started() {
    let project = Project::new("list-no-git");

    project.write("src/main.ts", "").commit();

    let empty = project
        .root
        .parent()
        .expect("the project's home")
        .join("empty-path");

    fs::create_dir_all(&empty).expect("a folder can be made");

    let line = stopped(&project.run(|binary| {
        binary.env("PATH", &empty);
    }));

    assert!(
        line.starts_with("yak-analyzer: git cannot be started ("),
        "{line}"
    );
    assert!(
        line.ends_with(
            "); the analyzer reads a project through git 2.15 or newer, which must be on the PATH"
        ),
        "{line}"
    );
}

/// A stand-in for an old git can be put on the `PATH` by a test only where a script is a program.
#[cfg(unix)]
#[test]
fn stops_on_a_git_older_than_the_oldest_it_works_with() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new("list-old-git");

    project.write("src/main.ts", "").commit();

    let bin = project
        .root
        .parent()
        .expect("the project's home")
        .join("old-git");
    let git = bin.join("git");

    fs::create_dir_all(&bin).expect("a folder can be made");
    fs::write(
        &git,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'git version 2.14.6'; exit 0; fi\n\
         echo 'error: unknown option: --no-optional-locks' >&2\nexit 129\n",
    )
    .expect("the stand-in can be written");
    fs::set_permissions(&git, fs::Permissions::from_mode(0o755))
        .expect("the stand-in can be made a program");

    assert_eq!(
        stopped(&project.run(|binary| {
            binary.env("PATH", &bin);
        })),
        "yak-analyzer: git 2.14.6 is older than 2.15, the oldest the analyzer works with; update git"
    );
}

/// A name that is not valid UTF-8 can be given to a file only where the file system takes any bytes.
#[cfg(target_os = "linux")]
#[test]
fn leaves_out_a_file_whose_name_is_not_valid_utf8_and_says_so() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let project = Project::new("list-odd-name");

    project.write("src/main.ts", "").commit();
    fs::write(
        project.root.join(OsStr::from_bytes(b"odd-\xFF-name.ts")),
        "",
    )
    .expect("the file can be written");

    let document = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert_eq!(notices(&document), ["names-left-out", "no-hand-off-file"]);
}

#[test]
fn changes_nothing_under_the_root_the_git_folder_included() {
    let project = Project::new("git-untouched");

    project
        .write("src/main.ts", "")
        .write("src/other.ts", "")
        .commit()
        .write("scratch.ts", "");

    // A changed file makes git's index stale, which a plain `git status` would then rewrite.
    fs::write(
        project.root.join("src/other.ts"),
        "export const changed = 1;\n",
    )
    .expect("the file can be written");

    let before = snapshot(&project.root);

    project.graph();

    assert!(
        before == snapshot(&project.root),
        "a file under the root changed"
    );
}

#[test]
fn does_not_obey_a_callers_variables_that_point_git_at_another_repository() {
    let other = Project::new("git-variables-other");

    other.write("other.ts", "").commit();

    let project = Project::new("git-variables");

    project.write("src/main.ts", "").commit();

    let pointed = printed(&project.run(|binary| {
        binary
            .env("GIT_DIR", other.root.join(".git"))
            .env("GIT_WORK_TREE", &other.root)
            .env("GIT_INDEX_FILE", other.root.join(".git/index"));
    }));

    assert_eq!(pointed, project.graph());
    assert_eq!(paths(&pointed), ["src/main.ts"]);
}

#[test]
fn does_not_start_a_program_the_repositorys_own_settings_name() {
    let project = Project::new("git-program");

    project.write("src/main.ts", "").commit();

    let home = project.root.parent().expect("the project's home");
    let marker = home.join("the-program-ran");
    let program = home.join("monitor.sh");
    let unix = |path: &Path| path.to_str().expect("valid UTF-8").replace('\\', "/");

    fs::write(
        &program,
        format!("#!/bin/sh\ntouch \"{}\"\nexit 1\n", unix(&marker)),
    )
    .expect("the program is written");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&program, fs::Permissions::from_mode(0o755))
            .expect("the program can be started");
    }

    project.git(&["config", "core.fsmonitor", &unix(&program)]);

    // The setting works: a plain git command starts the program.
    project.git(&["status", "--short"]);
    assert!(
        marker.exists(),
        "git did not start the program, so the test proves nothing"
    );
    fs::remove_file(&marker).expect("the marker can be removed");

    let document: Value = project.graph();

    assert_eq!(paths(&document), ["src/main.ts"]);
    assert!(!marker.exists(), "the analyzer's git started the program");
}
```

```rust path=tests/hand-off.test.rs
//! Hands the built binary a hand-off file and checks what it reads and what it refuses (spec HO-2 to
//! HO-5, RUN-3, RUN-4).

mod support;

use std::fs;

use serde_json::{Value, json};

use support::{document, folder, run, written};

/// The one line a stopped run wrote, after checking that it stopped as a stop does.
fn stopped(args: &[&str]) -> String {
    let output = run(args);
    let error = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(output.stdout.is_empty(), "{error}");
    assert_eq!(error.lines().count(), 1, "{error}");
    assert!(error.starts_with("yak-analyzer: "), "{error}");

    error.trim_end().to_owned()
}

#[test]
fn reads_a_hand_off_file_it_can_use() {
    let place = folder("hand-off-usable");
    let file = written(&place, &document(&place));
    let output = run(&["graph", "--hand-off", &file]);
    let printed: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    // The checkout is empty, so the two globs the hand-off file declares match nothing (spec KEY-5).
    let nothing = |key: &str, glob: &str, source: &str| {
        json!({
            "file": null,
            "kind": "glob-matches-nothing",
            "sentence": format!(
                "The `{key}` glob `{glob}` that the {source} states for the stack `typescript` matches no file."
            ),
        })
    };

    assert_eq!(
        printed,
        json!({
            "files": [],
            "handOff": true,
            "notices": [
                nothing("data", "**/fixtures/**", "pack"),
                nothing("generated", "src/generated/**", "project"),
            ],
            "packages": [],
        })
    );
}

#[test]
fn passes_over_a_byte_order_mark_and_a_top_level_field_it_does_not_know() {
    let place = folder("hand-off-mark");
    let mut handed = document(&place);

    handed["cache"] = json!({ "folder": "somewhere" });

    let file = written(&place, &handed);
    let mut bytes = b"\xEF\xBB\xBF".to_vec();

    bytes.extend(fs::read(&file).expect("the file was written"));
    fs::write(&file, bytes).expect("the file can be written again");

    assert!(run(&["graph", "--hand-off", &file]).status.success());
}

#[test]
fn says_in_a_notice_that_the_detection_list_was_cut_short() {
    let place = folder("hand-off-cut-short");
    let mut handed = document(&place);

    handed["detection"]["truncated"] = json!(true);

    let output = run(&["graph", "--hand-off", &written(&place, &handed)]);
    let printed: Value = serde_json::from_slice(&output.stdout).expect("graph prints JSON");

    assert!(output.status.success());
    assert!(
        printed["notices"]
            .as_array()
            .expect("a list of notices")
            .iter()
            .any(|notice| notice["kind"] == json!("detection-cut-short"))
    );
}

#[test]
fn refuses_the_option_with_no_file_after_it() {
    let output = run(&["graph", "--hand-off"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stderr).lines().count(), 1);
}

#[test]
fn stops_on_a_file_that_is_not_there_or_is_not_json() {
    let place = folder("hand-off-unreadable");
    let missing = place.join("none.json");
    let missing = missing.to_str().expect("the test's folder is valid UTF-8");
    let broken = place.join("broken.json");

    fs::write(&broken, "{ \"formatVersion\": 1,\n").expect("the file can be written");

    let broken = broken.to_str().expect("the test's folder is valid UTF-8");
    let unread = stopped(&["graph", "--hand-off", missing]);

    assert!(
        unread.starts_with(&format!(
            "yak-analyzer: the hand-off file {missing} cannot be read ("
        )),
        "{unread}"
    );
    assert!(
        unread.ends_with("); pass the path of the file yak wrote for this checkout"),
        "{unread}"
    );
    assert_eq!(
        stopped(&["graph", "--hand-off", broken]),
        format!(
            "yak-analyzer: the hand-off file {broken} is not JSON (line 2, column 0); yak writes this file, so \
             write it again with yak"
        )
    );
}

#[test]
fn stops_on_a_format_it_does_not_read_and_says_which_side_to_update() {
    let place = folder("hand-off-format");
    let mut newer = document(&place);
    let mut older = document(&place);

    // The format is asked before any other field, so the missing root is not what is reported.
    newer["formatVersion"] = json!(2);
    newer.as_object_mut().expect("an object").remove("root");
    older["formatVersion"] = json!(0);

    let file = written(&place, &newer);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the hand-off file {file} is format 2, newer than any this analyzer reads (1); update \
             the analyzer's version pinned in the pack"
        )
    );

    let file = written(&place, &older);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the hand-off file {file} is format 0, older than any this analyzer reads (1); update \
             yak"
        )
    );
}

#[test]
fn stops_on_a_field_that_is_missing_misshapen_or_not_allowed_and_names_it() {
    let place = folder("hand-off-shape");
    let relative = json!("checkout");
    let cases: [(&str, Value, &str); 22] = [
        (
            "/formatVersion",
            Value::Null,
            "`formatVersion` must be a whole number",
        ),
        (
            "/formatVersion",
            json!(1.5),
            "`formatVersion` must be a whole number",
        ),
        (
            "/root",
            Value::Null,
            "`root` must be text that is not empty",
        ),
        ("/root", relative.clone(), "`root` must be an absolute path"),
        ("/detection", json!([]), "`detection` must be an object"),
        (
            "/detection/extra",
            json!(1),
            "`detection.extra` is not a field of the format",
        ),
        (
            "/detection/folders",
            json!([]),
            "`detection.folders` must be a list of at least one path",
        ),
        (
            "/detection/folders",
            json!(["src"]),
            "`detection.folders` must begin with `.`, the root itself",
        ),
        (
            "/detection/folders/1",
            json!("../other"),
            "`detection.folders[1]` must be a path inside the root, written with forward slashes",
        ),
        (
            "/detection/folders/1",
            json!("packages\\app"),
            "`detection.folders[1]` must be a path inside the root, written with forward slashes",
        ),
        (
            "/detection/truncated",
            json!("no"),
            "`detection.truncated` must be true or false",
        ),
        ("/stacks", json!({}), "`stacks` must be a list"),
        (
            "/stacks/0",
            json!("typescript"),
            "`stacks[0]` must be an object",
        ),
        (
            "/stacks/0/name",
            json!(""),
            "`stacks[0].name` must be text that is not empty",
        ),
        (
            "/stacks/0/interfaceVersion",
            json!(0),
            "`stacks[0].interfaceVersion` must be a positive whole number",
        ),
        (
            "/stacks/0/pack",
            json!([]),
            "`stacks[0].pack` must be an object",
        ),
        (
            "/stacks/1/project",
            json!(null),
            "`stacks[1].project` must be an object",
        ),
        (
            "/stacks/0/project/data",
            json!({ "globs": [] }),
            "`stacks[0].project.data` must be a list of text or a positive whole number",
        ),
        (
            "/stacks/1/tool",
            json!("analyzer"),
            "`stacks[1].tool` is not a field of the format",
        ),
        ("/siblings", relative, "`siblings` must be an absolute path"),
        (
            "/base",
            json!("abc123"),
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits",
        ),
        (
            "/base",
            json!("ABCDEF0123456789ABCDEF0123456789ABCDEF01"),
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits",
        ),
    ];

    for (pointer, value, problem) in cases {
        let mut handed = document(&place);

        set(&mut handed, pointer, value);

        let file = written(&place, &handed);

        assert_eq!(
            stopped(&["graph", "--hand-off", &file]),
            format!(
                "yak-analyzer: the hand-off file {file} is not format 1: {problem}; yak writes this file, so write \
                 it again with yak"
            ),
            "{pointer}"
        );
    }

    for field in ["formatVersion", "root", "detection", "stacks", "siblings"] {
        let mut handed = document(&place);

        handed.as_object_mut().expect("an object").remove(field);

        let line = stopped(&["graph", "--hand-off", &written(&place, &handed)]);

        assert!(
            line.contains(&format!("is not format 1: `{field}` is missing; ")),
            "{line}"
        );
    }
}

#[test]
fn takes_a_base_and_a_stack_with_no_declarations_and_no_interface_version() {
    let place = folder("hand-off-base");
    let mut handed = document(&place);

    handed["base"] = json!("0123456789abcdef0123456789abcdef01234567");

    assert!(
        run(&["graph", "--hand-off", &written(&place, &handed)])
            .status
            .success()
    );
}

#[test]
fn stops_on_declarations_written_for_another_interface_version_or_for_none() {
    let place = folder("hand-off-interface");
    let mut newer = document(&place);
    let mut unstated = document(&place);

    // The interface version is asked before the declarations, so the unknown key is not what is reported.
    newer["stacks"][0]["interfaceVersion"] = json!(2);
    newer["stacks"][0]["pack"]["unknownKey"] = json!([]);
    unstated["stacks"][1]["project"] = json!({ "data": ["fixtures/**"] });

    let file = written(&place, &newer);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the stack `typescript` in {file} states declarations for interface version 2, and this \
             analyzer speaks version 1; update the analyzer's version pinned in the pack"
        )
    );

    let file = written(&place, &unstated);

    assert_eq!(
        stopped(&["graph", "--hand-off", &file]),
        format!(
            "yak-analyzer: the stack `bare` in {file} holds declarations and states no interface version; state \
             the version they were written for"
        )
    );
}

#[test]
fn reports_the_shape_before_the_interface_version_when_both_are_wrong() {
    let place = folder("hand-off-order");
    let mut handed = document(&place);

    handed["stacks"][0]["interfaceVersion"] = json!(2);
    handed["siblings"] = json!("relative");

    let line = stopped(&["graph", "--hand-off", &written(&place, &handed)]);

    assert!(
        line.contains("is not format 1: `siblings` must be an absolute path; "),
        "{line}"
    );
}

/// Puts a value at a JSON pointer, making the last step when it is not there yet.
fn set(document: &mut Value, pointer: &str, value: Value) {
    let (parent, last) = pointer.rsplit_once('/').expect("a pointer with a step");
    let parent = if parent.is_empty() {
        document
    } else {
        document
            .pointer_mut(parent)
            .expect("the pointer's parent is there")
    };

    match parent {
        Value::Object(fields) => {
            fields.insert(last.to_owned(), value);
        }
        Value::Array(entries) => entries[last.parse::<usize>().expect("an index")] = value,
        _ => panic!("the pointer's parent holds nothing"),
    }
}
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test packages --test file-list --test hand-off --bins
```

Expected: cargo refuses, because the manifest names no test called `packages` yet.

- [ ] **Step 3: Name the test and add the two crates**

```toml path=Cargo.toml
[package]
name = "yak-analyzer"
version = "0.1.0"
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

[[test]]
name = "hand-off"
path = "tests/hand-off.test.rs"

[[test]]
name = "declarations"
path = "tests/declarations.test.rs"

[[test]]
name = "hand-off-schema"
path = "tests/hand-off-schema.test.rs"

[[test]]
name = "file-list"
path = "tests/file-list.test.rs"

[[test]]
name = "packages"
path = "tests/packages.test.rs"

[dependencies]
# Reads the hand-off file and writes what `graph` prints.
serde_json = "=1.0.154"
# Matches the globs of declarations and of the workspace's list (spec KEY-5).
regex = "=1.13.1"
# Tells whether two paths are one file on disk, which Rust gives on Windows only in an unstable form.
same-file = "=1.0.6"
# Reads the workspace's list that pnpm keeps, which is YAML (spec PACK-3).
yaml-rust2 = { version = "=0.13.0", default-features = false }

[dev-dependencies]
# Checks the tests' hand-off files against the copy of yakshaver's schema. Never part of the binary.
jsonschema = { version = "=0.60.2", default-features = false }
```

```toml path=Cargo.lock
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "ahash"
version = "0.8.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
dependencies = [
 "cfg-if",
 "getrandom",
 "once_cell",
 "serde",
 "version_check",
 "zerocopy",
]

[[package]]
name = "aho-corasick"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c982642fa9e8606056828ee9a8505737230110bb1099153c79efe865c59d12ba"
dependencies = [
 "memchr",
]

[[package]]
name = "allocator-api2"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923"

[[package]]
name = "arraydeque"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7d902e3d592a523def97af8f317b08ce16b7ab854c1985a0c671e6f15cebc236"

[[package]]
name = "autocfg"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"

[[package]]
name = "bit-set"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3"
dependencies = [
 "bit-vec",
]

[[package]]
name = "bit-vec"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7"

[[package]]
name = "bitflags"
version = "2.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"

[[package]]
name = "borrow-or-share"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc0b364ead1874514c8c2855ab558056ebfeb775653e7ae45ff72f28f8f3166c"

[[package]]
name = "bumpalo"
version = "3.20.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649"

[[package]]
name = "bytecount"
version = "0.6.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175812e0be2bccb6abe50bb8d566126198344f707e304f45c648fd8f2cc0365e"

[[package]]
name = "cfg-if"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"

[[package]]
name = "data-encoding"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "email_address"
version = "0.2.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e079f19b08ca6239f47f8ba8509c11cf3ea30095831f7fed61441475edd8c449"
dependencies = [
 "serde",
]

[[package]]
name = "equivalent"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00d174d5400e5e8fd687ad1049e2f578285fa914201b1af7e8b112a4546bd826"

[[package]]
name = "fancy-regex"
version = "0.19.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d301f5bf187b3c295fce6468d3875037a0bccc5f6b151c63cac2f85babf21912"
dependencies = [
 "bit-set",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "fluent-uri"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bc74ac4d8359ae70623506d512209619e5cf8f347124910440dbc221714b328e"
dependencies = [
 "borrow-or-share",
 "ref-cast",
 "serde",
]

[[package]]
name = "foldhash"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77ce24cb58228fbb8aa041425bb1050850ac19177686ea6e0f41a70416f56fdb"

[[package]]
name = "fraction"
version = "0.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e246562084dde8ebbcc943b261c406ce4f68e5032ec28029a251a47d6a295500"
dependencies = [
 "num",
 "num-bigint",
]

[[package]]
name = "getrandom"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "r-efi",
 "wasip2",
 "wasm-bindgen",
]

[[package]]
name = "hashbrown"
version = "0.17.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a"
dependencies = [
 "allocator-api2",
 "equivalent",
 "foldhash",
]

[[package]]
name = "hashlink"
version = "0.12.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a596f1b20ed2cc5ecac41a164aaebc7258057060f06c0cf7a2ba3991ee7990fb"
dependencies = [
 "hashbrown",
]

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "itoa"
version = "1.0.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3bd1574e617a9a6c1f8f8181bd7e841e1f20e4277607ef28945765188d6284d4"

[[package]]
name = "js-sys"
version = "0.3.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8dc09d2154098bbf4ef54eab6b117a3c24cbbffe94046b16ee6d6d4510584d3a"
dependencies = [
 "cfg-if",
 "wasm-bindgen",
]

[[package]]
name = "jsonschema"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a42a11b47492e6a2cc76e9c7bcc8077776cb31dac444e0795d7f61bcc6441c08"
dependencies = [
 "ahash",
 "bytecount",
 "data-encoding",
 "email_address",
 "fancy-regex",
 "fraction",
 "getrandom",
 "itoa",
 "jsonschema-regex",
 "jsonschema-value",
 "num-cmp",
 "num-traits",
 "percent-encoding",
 "referencing",
 "regex",
 "serde",
 "serde_json",
 "strum",
 "unicode-general-category",
 "uuid-simd",
]

[[package]]
name = "jsonschema-regex"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "107aeb6ff14b8deb5610096a8ce5c554ec89806b521147eaeb3cae05b07edb7a"
dependencies = [
 "regex-syntax",
]

[[package]]
name = "jsonschema-value"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e281d6b177bb4834f2ffd6f92aa0443cffe29c996b251c8d94eb6f456186feba"
dependencies = [
 "ahash",
 "bytecount",
 "fraction",
 "getrandom",
 "memchr",
 "num-cmp",
 "num-traits",
 "serde_json",
 "zmij",
]

[[package]]
name = "libc"
version = "0.2.191"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "65cd87c9a6d60536580b316ad985726104730a5151c99845f2490428ef91240a"

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "memchr"
version = "2.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"

[[package]]
name = "micromap"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a86d3146ed3995b5913c414f6664344b9617457320782e64f0bb44afd49d74"

[[package]]
name = "num"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "35bd024e8b2ff75562e5f34e7f4905839deb4b22955ef5e73d2fea1b9813cb23"
dependencies = [
 "num-bigint",
 "num-complex",
 "num-integer",
 "num-iter",
 "num-rational",
 "num-traits",
]

[[package]]
name = "num-bigint"
version = "0.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-cmp"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63335b2e2c34fae2fb0aa2cecfd9f0832a1e24b3b32ecec612c3426d46dc8aaa"

[[package]]
name = "num-complex"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-integer"
version = "0.1.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7ce2d95d4b3734dc35aa2f45e1aa22cd416814592a4f9d9205e11affd5b8e10b"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-iter"
version = "0.1.46"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c92800bd69a1eac91786bcfe9da64a897eb72911b8dc3095decbd07429e8048b"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-rational"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824"
dependencies = [
 "num-bigint",
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-traits"
version = "0.2.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
dependencies = [
 "autocfg",
]

[[package]]
name = "once_cell"
version = "1.21.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"

[[package]]
name = "outref"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a80800c0488c3a21695ea981a54918fbb37abf04f4d0720c453632255e2ff0e"

[[package]]
name = "parking_lot"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
dependencies = [
 "lock_api",
 "parking_lot_core",
]

[[package]]
name = "parking_lot_core"
version = "0.9.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
dependencies = [
 "cfg-if",
 "libc",
 "redox_syscall",
 "smallvec",
 "windows-link",
]

[[package]]
name = "percent-encoding"
version = "2.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"

[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "r-efi"
version = "5.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"

[[package]]
name = "redox_syscall"
version = "0.5.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
dependencies = [
 "bitflags",
]

[[package]]
name = "ref-cast"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e440fb4e4b4147295338efb76001ab9e4efc0e5839df2c47fc5ac2381d365c3"
dependencies = [
 "ref-cast-impl",
]

[[package]]
name = "ref-cast-impl"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92ecd8964f8453721699a1ed72037b0db49ce2f5a5138486ee89bed6f67cdf3a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "referencing"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f42818036d3b37cda6bd4c494543158a36e431cb717015ef7c93acc6ba214d3b"
dependencies = [
 "ahash",
 "fluent-uri",
 "getrandom",
 "hashbrown",
 "itoa",
 "micromap",
 "parking_lot",
 "percent-encoding",
 "serde_json",
]

[[package]]
name = "regex"
version = "1.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "regex-automata"
version = "0.4.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad8553b9b26413251cbf30e620595c7a41b3887f03da04579c0e6b0d6a06b4b2"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-syntax",
]

[[package]]
name = "regex-syntax"
version = "0.8.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4"

[[package]]
name = "rustversion"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"

[[package]]
name = "same-file"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502"
dependencies = [
 "winapi-util",
]

[[package]]
name = "scopeguard"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"

[[package]]
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
dependencies = [
 "serde_core",
 "serde_derive",
]

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "serde_json"
version = "1.0.154"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "smallvec"
version = "1.16.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "149309498a5f7334b12ddae0cad8aba81ce785fe91755068867e017cf98fee46"

[[package]]
name = "strum"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9628de9b8791db39ceda2b119bbe13134770b56c138ec1d3af810d045c04f9bd"
dependencies = [
 "strum_macros",
]

[[package]]
name = "strum_macros"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab85eea0270ee17587ed4156089e10b9e6880ee688791d45a905f5b1ca36f664"
dependencies = [
 "heck",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "syn"
version = "2.0.119"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "3.0.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ae9ccbea3a7b0b09443036490c8f34a8b1b760fd345e5c0dae0aac718faea79"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "unicode-general-category"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b993bddc193ae5bd0d623b49ec06ac3e9312875fdae725a975c51db1cc1677f"

[[package]]
name = "unicode-ident"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563"

[[package]]
name = "uuid-simd"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b082222b4f6619906941c17eb2297fff4c2fb96cb60164170522942a200bd8"
dependencies = [
 "outref",
 "vsimd",
]

[[package]]
name = "version_check"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"

[[package]]
name = "vsimd"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c3082ca00d5a5ef149bb8b555a72ae84c9c59f7250f013ac822ac2e49b19c64"

[[package]]
name = "wasip2"
version = "1.0.4+wasi-0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b67efb37e106e55ce722a510d6b5f9c17f083e5fc79afc2badeb12cc313d9487"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasm-bindgen"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0181c14ddd6c7b8b0b0132a5f1ea2f32be0d893cd3ea97b6e3b28886b8a3d1c"
dependencies = [
 "cfg-if",
 "rustversion",
 "wasm-bindgen-macro",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e931a02a39abdbf66966fc35902b5ed58344750a70885bed4ebb3d8eaf28ed9"
dependencies = [
 "quote",
 "wasm-bindgen-macro-support",
]

[[package]]
name = "wasm-bindgen-macro-support"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ca4b54280e455bd831e33a6b8e734baf432fe63fbc35c42ac14ce67a043a53a"
dependencies = [
 "bumpalo",
 "proc-macro2",
 "quote",
 "syn 3.0.10",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-shared"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b1f9c3798725016c6ee5f2f57869ec0831e8bbc0ab0310a86e787e97f41389e"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "winapi-util"
version = "0.1.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22"
dependencies = [
 "windows-sys",
]

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "windows-sys"
version = "0.61.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
dependencies = [
 "windows-link",
]

[[package]]
name = "wit-bindgen"
version = "0.57.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e"

[[package]]
name = "yak-analyzer"
version = "0.1.0"
dependencies = [
 "jsonschema",
 "regex",
 "same-file",
 "serde_json",
 "yaml-rust2",
]

[[package]]
name = "yaml-rust2"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "57e5b818a27a4cd30884ea380857a5e56f7ec3ba24a3990a3cc0b95af3238e18"
dependencies = [
 "arraydeque",
 "hashlink",
]

[[package]]
name = "zerocopy"
version = "0.8.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5fe1f8f1b06191a00962174c61aa5005e0bb391a6d80d07e24d115c01a92ed8"
dependencies = [
 "zerocopy-derive",
]

[[package]]
name = "zerocopy-derive"
version = "0.8.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "863ad3ac83293fb4d740aedbfdc9240dd8d1a50c1099acd76ce80ce7c7230c7f"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"
```

- [ ] **Step 4: Write the globs, the marks and the packages**

```rust path=src/glob.rs
//! Globs: patterns for paths, in the dialect the spec defines (spec KEY-5).
//!
//! A glob is turned into a regular expression once, and matched against a path relative to the glob's
//! folder, written with forward slashes, with exact letter case.

use regex::Regex;

/// A glob, ready to match.
#[derive(Debug)]
pub struct Glob {
    /// The regular expression it stands for.
    pattern: Regex,
}

impl Glob {
    /// Reads a glob. The error says why it cannot be read.
    pub fn new(written: &str) -> Result<Self, String> {
        let expression = translate(written)?;
        let pattern = Regex::new(&format!("^{expression}$"))
            .map_err(|_| format!("the glob `{written}` cannot be read"))?;

        Ok(Self { pattern })
    }

    /// Whether the glob matches a path, relative to the glob's folder.
    pub fn matches(&self, path: &str) -> bool {
        self.pattern.is_match(path)
    }
}

/// The regular expression a glob stands for.
fn translate(glob: &str) -> Result<String, String> {
    let characters: Vec<char> = glob.chars().collect();
    let mut expression = String::with_capacity(glob.len() * 2);
    let mut braces = 0_usize;
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        match character {
            '*' if characters.get(index + 1) == Some(&'*') && whole_segment(&characters, index) => {
                let at_start = index == 0;
                let at_end = index + 2 == characters.len();

                expression.push_str(match (at_start, at_end) {
                    // `**` alone matches every path.
                    (true, true) => ".*",
                    // `**/` at the start matches any number of whole folders, none included.
                    (true, false) => "(?:[^/]+/)*",
                    // `/**` at the end matches everything under the folder before it.
                    (false, true) => ".+",
                    // `/**/` in the middle matches any number of whole folders, none included.
                    (false, false) => "(?:[^/]+/)*",
                });

                // The slash after a `**` that is not at the end is part of what was written above.
                index += if at_end { 2 } else { 3 };

                continue;
            }
            '*' => expression.push_str("[^/]*"),
            '?' => expression.push_str("[^/]"),
            '[' => {
                let (class, next) = set(&characters, index, glob)?;

                expression.push_str(&class);
                index = next;

                continue;
            }
            '{' => {
                braces += 1;
                expression.push_str("(?:");
            }
            ',' if braces > 0 => expression.push('|'),
            '}' if braces > 0 => {
                braces -= 1;
                expression.push(')');
            }
            '\\' => {
                let escaped = characters.get(index + 1).ok_or_else(|| {
                    format!("the glob `{glob}` ends in a backslash that escapes nothing")
                })?;

                expression.push_str(&regex::escape(&escaped.to_string()));
                index += 2;

                continue;
            }
            other => expression.push_str(&regex::escape(&other.to_string())),
        }

        index += 1;
    }

    if braces > 0 {
        return Err(format!("the glob `{glob}` has a brace left open"));
    }

    Ok(expression)
}

/// Whether the `**` at the index is a whole path segment: at the start or after a slash, and at the
/// end or before a slash. Anywhere else it is two `*`.
fn whole_segment(characters: &[char], index: usize) -> bool {
    let before = index == 0 || characters[index - 1] == '/';
    let after = index + 2 == characters.len() || characters[index + 2] == '/';

    before && after
}

/// The character class a `[` at the index begins, and the index after its `]`. A `!` first means any
/// character not in the set, a `]` first is a character of the set, and a class never matches `/`.
fn set(characters: &[char], start: usize, glob: &str) -> Result<(String, usize), String> {
    let mut index = start + 1;
    let mut class = String::from("[[");

    if characters.get(index) == Some(&'!') {
        class.push('^');
        index += 1;
    }

    let first = index;

    loop {
        let character = *characters
            .get(index)
            .ok_or_else(|| format!("the glob `{glob}` has a bracket left open"))?;

        if character == ']' && index > first {
            break;
        }

        match character {
            '\\' => {
                let escaped = *characters
                    .get(index + 1)
                    .ok_or_else(|| format!("the glob `{glob}` has a bracket left open"))?;

                if escaped.is_ascii_punctuation() {
                    class.push('\\');
                }

                class.push(escaped);
                index += 2;

                continue;
            }
            // A dash between two characters is a range; anywhere else it is itself.
            '-' if index > first && characters.get(index + 1).is_some_and(|next| *next != ']') => {
                class.push('-');
            }
            other if other.is_ascii_punctuation() => {
                class.push('\\');
                class.push(other);
            }
            other => class.push(other),
        }

        index += 1;
    }

    // The set, negated or not, intersected with every character but `/`, so that a class stays inside
    // one path segment.
    class.push_str("]&&[^/]]");

    Ok((class, index + 1))
}

#[cfg(test)]
mod tests {
    use super::Glob;

    /// Whether the glob matches each path given, as the spec's dialect says it must.
    fn check(written: &str, matching: &[&str], not_matching: &[&str]) {
        let glob = Glob::new(written).expect("the glob can be read");

        for path in matching {
            assert!(glob.matches(path), "`{written}` should match `{path}`");
        }

        for path in not_matching {
            assert!(!glob.matches(path), "`{written}` should not match `{path}`");
        }
    }

    #[test]
    fn a_star_stays_inside_one_segment() {
        check(
            "src/*.ts",
            &["src/a.ts", "src/.ts"],
            &["src/a/b.ts", "a.ts", "src/a.tsx"],
        );
    }

    #[test]
    fn a_double_star_spans_whole_segments() {
        check(
            "**/fixtures/**",
            &["fixtures/a.json", "a/b/fixtures/c/d.ts"],
            &["fixtures", "a/myfixtures/b.ts"],
        );
        check(
            "src/**/x.ts",
            &["src/x.ts", "src/a/b/x.ts"],
            &["srcx.ts", "src/ax.ts"],
        );
        check("**", &["a", "a/b/c.ts"], &[]);
        check(
            "folder/**",
            &["folder/a.ts", "folder/a/b.ts"],
            &["folder", "other/a.ts"],
        );
        check("a**b.ts", &["ab.ts", "axxb.ts"], &["a/b.ts"]);
    }

    #[test]
    fn a_question_mark_is_one_character_and_never_a_slash() {
        check("?.ts", &["a.ts", "é.ts"], &["ab.ts", "/.ts"]);
    }

    #[test]
    fn a_set_may_hold_a_range_be_negated_and_begin_with_a_bracket() {
        check("[a-c].ts", &["a.ts", "c.ts"], &["d.ts", "A.ts"]);
        check("[!_]*.mjs", &["run.mjs"], &["_private.mjs"]);
        check("[]x].ts", &["].ts", "x.ts"], &["y.ts"]);
        check("[-a].ts", &["-.ts", "a.ts"], &["b.ts"]);
        check("x[!a]y", &["xby"], &["xay", "x/y"]);
    }

    #[test]
    fn braces_hold_alternatives_and_may_nest() {
        check(
            "**/*.{test,spec}.{ts,tsx}",
            &["a.test.ts", "b/c.spec.tsx"],
            &["a.test.js", "a.tests.ts"],
        );
        check(
            "docs/{a,{b,c}}/*.mdx",
            &["docs/a/x.mdx", "docs/c/y.mdx"],
            &["docs/d/x.mdx"],
        );
    }

    #[test]
    fn a_backslash_makes_a_character_stand_for_itself_and_the_rest_stand_for_themselves() {
        check("odd\\[name\\]/?.ts", &["odd[name]/a.ts"], &["oddn/a.ts"]);
        check("a+b(c).ts", &["a+b(c).ts"], &["aab(c).ts"]);
        check("Src/A.ts", &["Src/A.ts"], &["src/a.ts"]);
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        for (glob, why) in [
            ("src/[a-z.ts", "has a bracket left open"),
            ("src/{a,b.ts", "has a brace left open"),
            ("src/a\\", "ends in a backslash that escapes nothing"),
            ("[]", "has a bracket left open"),
        ] {
            let refused = Glob::new(glob).expect_err(glob);

            assert!(refused.ends_with(why), "{refused}");
        }
    }
}
```

```rust path=src/marks.rs
//! The `data` and `generated` declarations, applied to the file list (spec KEY-3, KEY-4, KEY-5).

use std::collections::BTreeSet;

use serde_json::Value;

use crate::files::File;
use crate::glob::Glob;
use crate::hand_off::HandOff;
use crate::notice::Notice;
use crate::stop::Stop;

/// The keys whose globs mark files, and what they mark them as.
const KEYS: [&str; 2] = ["data", "generated"];

/// Which files the declarations mark, by path.
#[derive(Debug, Default)]
pub struct Marks {
    /// Files that are not the project's code, which are never parsed or read (spec KEY-4).
    pub data: BTreeSet<String>,
    /// Committed files a generator writes, which are read like any other.
    pub generated: BTreeSet<String>,
}

/// Marks the files the declarations of the hand-off file name. Without a hand-off file there are no
/// declarations, and nothing is marked.
pub fn apply(
    files: &[File],
    handed: Option<&HandOff>,
    notices: &mut Vec<Notice>,
) -> Result<Marks, Stop> {
    let mut marks = Marks::default();
    let Some(handed) = handed else {
        return Ok(marks);
    };

    for stack in &handed.stacks {
        for key in KEYS {
            // A pack's globs are relative to each folder yak's detection looked in, and a project's to
            // the root (spec KEY-3).
            let sources = [
                ("pack", &stack.pack, handed.folders.as_slice()),
                ("project", &stack.project, &[".".to_owned()][..]),
            ];

            for (source, declarations, folders) in sources {
                for written in globs(declarations.get(key)) {
                    let glob = Glob::new(written).map_err(Stop::new)?;
                    let matched: Vec<&File> = files
                        .iter()
                        .filter(|file| {
                            folders
                                .iter()
                                .any(|folder| under(folder, &file.path, &glob))
                        })
                        .collect();

                    if matched.is_empty() {
                        notices.push(Notice::new(
                            "glob-matches-nothing",
                            None,
                            format!(
                                "The `{key}` glob `{written}` that the {source} states for the stack `{}` \
                                 matches no file.",
                                stack.name
                            ),
                        ));
                    }

                    let marked = if key == "data" {
                        &mut marks.data
                    } else {
                        &mut marks.generated
                    };

                    marked.extend(matched.into_iter().map(|file| file.path.clone()));
                }
            }
        }
    }

    Ok(marks)
}

/// The globs a declaration holds. The declaration was checked when the hand-off file was read, so
/// anything else holds none.
fn globs(declaration: Option<&Value>) -> Vec<&str> {
    declaration
        .and_then(Value::as_array)
        .map(|globs| globs.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// Whether a path lies under the folder and the glob matches it relative to that folder.
fn under(folder: &str, path: &str, glob: &Glob) -> bool {
    if folder == "." {
        return glob.matches(path);
    }

    path.strip_prefix(folder)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|rest| glob.matches(rest))
}
```

```rust path=src/packages.rs
//! Packages: the folders that hold a `package.json`, which of them the workspace names, and which
//! package each file belongs to (spec PACK-1 to PACK-6).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};
use yaml_rust2::{Yaml, YamlLoader};

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::notice::Notice;
use crate::stop::Stop;

/// The workspace's list that pnpm reads, at the root.
const PNPM_LIST: &str = "pnpm-workspace.yaml";

/// One package.
#[derive(Debug)]
pub struct Package {
    /// Its folder, relative to the root; `.` for the root itself.
    pub folder: String,
    /// The `name` its `package.json` states, where it states one.
    pub name: Option<String>,
    /// Whether the workspace names it, so an import by its name leads to it (spec PACK-3).
    pub workspace: bool,
    /// Whether its folder is one the design spec reports the entry-point kinds in (spec PACK-4).
    pub detected: bool,
}

/// The packages of a project, and which package each file belongs to.
#[derive(Debug)]
pub struct Packages {
    /// The packages, by folder, byte by byte.
    pub list: Vec<Package>,
    /// The folder of the package each file belongs to, by the file's path; a file with no package
    /// above it is not here.
    pub of_file: BTreeMap<String, String>,
}

/// Reads the packages of the root. `data` is the files declared as data, whose `package.json` makes
/// no package (spec KEY-4); `detection` is the folders yak's detection looked in, with a hand-off file.
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    detection: Option<&[String]>,
    notices: &mut Vec<Notice>,
) -> Result<Packages, Stop> {
    let manifests: Vec<&File> = files
        .iter()
        .filter(|file| file.kind == Kind::Manifest && !data.contains(&file.path))
        .collect();

    // Each `package.json` is read first, then the workspace's list, then the fields, so that two faults
    // always give the same line (spec RUN-4).
    let mut read: Vec<(String, Map<String, Value>)> = Vec::with_capacity(manifests.len());

    for manifest in &manifests {
        read.push((folder_of(&manifest.path), object(root, &manifest.path)?));
    }

    let pnpm = if files.iter().any(|file| file.path == PNPM_LIST) && !data.contains(PNPM_LIST) {
        Some(yaml(root)?)
    } else {
        None
    };
    let mut names: BTreeMap<&str, Option<String>> = BTreeMap::new();

    for (folder, fields) in &read {
        names.insert(folder, name(fields, &manifest_path(folder))?);
    }

    let mut patterns = Vec::new();

    if let Some((_, fields)) = read.iter().find(|(folder, _)| folder == ".") {
        patterns.extend(workspaces(fields)?);
    }

    if let Some(pnpm) = &pnpm {
        patterns.extend(pnpm_packages(pnpm)?);
    }

    let (included, excluded) = compile(&patterns)?;
    let detection: Option<BTreeSet<&str>> =
        detection.map(|folders| folders.iter().map(String::as_str).collect());
    let mut list: Vec<Package> = names
        .into_iter()
        .map(|(folder, name)| {
            let listed = folder != "."
                && included.iter().any(|glob| glob.matches(folder))
                && !excluded.iter().any(|glob| glob.matches(folder));
            let workspace = folder == "."
                || listed
                || detection
                    .as_ref()
                    .is_some_and(|found| found.contains(folder));
            let detected = match &detection {
                Some(found) => found.contains(folder),
                None => folder == "." || listed,
            };

            Package {
                folder: folder.to_owned(),
                name,
                workspace,
                detected,
            }
        })
        .collect();

    list.sort_by(|a, b| a.folder.as_bytes().cmp(b.folder.as_bytes()));
    repeated_names(&list, notices);

    let folders: BTreeSet<&str> = list.iter().map(|package| package.folder.as_str()).collect();
    let of_file = files
        .iter()
        .filter_map(|file| nearest(&file.path, &folders).map(|folder| (file.path.clone(), folder)))
        .collect();

    Ok(Packages { list, of_file })
}

/// The folder a `package.json` lies in, `.` for the root.
fn folder_of(path: &str) -> String {
    path.rsplit_once('/')
        .map_or_else(|| ".".to_owned(), |(folder, _)| folder.to_owned())
}

/// The path of the `package.json` in a folder.
fn manifest_path(folder: &str) -> String {
    if folder == "." {
        "package.json".to_owned()
    } else {
        format!("{folder}/package.json")
    }
}

/// The content of a file, with a byte-order mark at its start passed over.
fn text(root: &Path, path: &str) -> Result<String, String> {
    let bytes = fs::read(root.join(path)).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);

    String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())
}

/// A `package.json` read as a JSON object (spec PACK-2).
fn object(root: &Path, path: &str) -> Result<Map<String, Value>, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the package.json {path} cannot be read as a JSON object ({why}); fix it, or declare it as data \
             if it is broken on purpose"
        ))
    };
    let content = text(root, path).map_err(refused)?;

    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(fields)) => Ok(fields),
        Ok(_) => Err(refused(
            "it holds something other than an object".to_owned(),
        )),
        Err(error) => Err(refused(format!(
            "line {}, column {}",
            error.line(),
            error.column()
        ))),
    }
}

/// The workspace's list that pnpm reads, as YAML (spec PACK-3).
fn yaml(root: &Path) -> Result<Yaml, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the workspace's list {PNPM_LIST} cannot be read ({why}); fix it, or declare it as data"
        ))
    };
    let content = text(root, PNPM_LIST).map_err(refused)?;
    let documents =
        YamlLoader::load_from_str(&content).map_err(|error| refused(error.to_string()))?;

    Ok(documents.into_iter().next().unwrap_or(Yaml::Null))
}

/// The stop for a field that holds a value of a form it does not have (spec PACK-6).
fn misshapen(field: &str, file: &str, form: &str) -> Stop {
    Stop::new(format!(
        "the field `{field}` in {file} holds a value of a form it does not have; it must be {form}; fix it"
    ))
}

/// The `name` a `package.json` states.
fn name(fields: &Map<String, Value>, file: &str) -> Result<Option<String>, Stop> {
    match fields.get("name") {
        None => Ok(None),
        Some(Value::String(name)) => Ok(Some(name.clone())),
        Some(_) => Err(misshapen("name", file, "text")),
    }
}

/// The globs of the root `package.json`'s `workspaces`: a list, or an object with `packages`.
fn workspaces(fields: &Map<String, Value>) -> Result<Vec<String>, Stop> {
    let form = "a list of globs, or an object whose `packages` is one";
    let list = match fields.get("workspaces") {
        None => return Ok(Vec::new()),
        Some(Value::Array(list)) => list,
        Some(Value::Object(object)) => match object.get("packages") {
            None => return Ok(Vec::new()),
            Some(Value::Array(list)) => list,
            Some(_) => return Err(misshapen("workspaces", "package.json", form)),
        },
        Some(_) => return Err(misshapen("workspaces", "package.json", form)),
    };

    list.iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| misshapen("workspaces", "package.json", form))
        })
        .collect()
}

/// The globs of pnpm's `packages`.
fn pnpm_packages(document: &Yaml) -> Result<Vec<String>, Stop> {
    let form = "a list of globs";
    let packages = match document {
        Yaml::Null => return Ok(Vec::new()),
        Yaml::Hash(_) => &document["packages"],
        _ => {
            return Err(misshapen(
                "packages",
                PNPM_LIST,
                "a mapping that holds `packages`",
            ));
        }
    };

    match packages {
        Yaml::BadValue | Yaml::Null => Ok(Vec::new()),
        Yaml::Array(list) => list
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| misshapen("packages", PNPM_LIST, form))
            })
            .collect(),
        _ => Err(misshapen("packages", PNPM_LIST, form)),
    }
}

/// The workspace's globs, read: those that include folders, and those, written with `!`, that take
/// folders out again. A leading `./` and a trailing `/` say nothing and are passed over.
fn compile(patterns: &[String]) -> Result<(Vec<Glob>, Vec<Glob>), Stop> {
    let mut included = Vec::new();
    let mut excluded = Vec::new();

    for pattern in patterns {
        let (negated, written) = pattern
            .strip_prefix('!')
            .map_or((false, pattern.as_str()), |rest| (true, rest));
        let written = written
            .strip_prefix("./")
            .unwrap_or(written)
            .trim_end_matches('/');
        let glob = Glob::new(written).map_err(|why| {
            Stop::new(format!(
                "a glob of the workspace's list cannot be read: {why}; fix it in package.json or {PNPM_LIST}"
            ))
        })?;

        if negated {
            excluded.push(glob);
        } else {
            included.push(glob);
        }
    }

    Ok((included, excluded))
}

/// A notice for each workspace package whose name an earlier one already has (spec PACK-5).
fn repeated_names(list: &[Package], notices: &mut Vec<Notice>) {
    let mut first: BTreeMap<&str, &str> = BTreeMap::new();

    for package in list.iter().filter(|package| package.workspace) {
        let Some(name) = &package.name else {
            continue;
        };

        match first.get(name.as_str()) {
            Some(earlier) => notices.push(Notice::new(
                "package-name-repeated",
                Some(manifest_path(&package.folder)),
                format!(
                    "The workspace packages in `{earlier}` and `{}` are both named `{name}`; an import of `{name}` \
                     leads to the first, `{earlier}`.",
                    package.folder
                ),
            )),
            None => {
                first.insert(name, &package.folder);
            }
        }
    }
}

/// The folder of the nearest package at or above a file, searching no higher than the root (spec
/// PACK-1, LIST-8).
fn nearest(path: &str, folders: &BTreeSet<&str>) -> Option<String> {
    let mut folder = path;

    while let Some((above, _)) = folder.rsplit_once('/') {
        if folders.contains(above) {
            return Some(above.to_owned());
        }

        folder = above;
    }

    folders.contains(".").then(|| ".".to_owned())
}
```

```rust path=src/declarations.rs
//! The declaration keys the analyzer knows, and the check that it can use every declaration a hand-off
//! file holds (spec KEY-1, KEY-2, KEY-5).
//!
//! The keys are drafts until the design spec's stage 2 ends. This part checks them; the parts that read a
//! project use them.

use serde_json::Value;

use crate::glob::Glob;
use crate::hand_off::{Stack, leaves_its_folder};
use crate::stop::Stop;

/// What kind of value a key takes.
#[derive(Clone, Copy)]
enum Kind {
    /// A list of globs.
    Globs,
    /// A list of paths.
    Paths,
    /// A positive whole number.
    Number,
    /// Any JSON, which only a pack may state.
    Opaque,
}

/// Every key the analyzer knows, with the kind of its value (spec KEY-1).
const KEYS: [(&str, Kind); 9] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("internal", Kind::Paths),
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("configEntries", Kind::Opaque),
    ("scriptCommands", Kind::Opaque),
];

/// Who states a declaration: the hand-off file keeps the two apart.
#[derive(Clone, Copy)]
enum Source {
    /// The pack that carries the stack.
    Pack,
    /// The project itself.
    Project,
}

impl Source {
    /// The word for the line.
    const fn word(self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::Project => "project",
        }
    }

    /// Where a person changes the declaration. The line names no `yak` command: those names are yak's.
    const fn fix(self) -> &'static str {
        match self {
            Self::Pack => "change it in the pack",
            Self::Project => "change it in the project's own declarations",
        }
    }
}

/// Checks every declaration of every stack, the stacks in the file's order and the pack's before the
/// project's. The first that cannot be used stops the run (spec KEY-2).
pub fn check(stacks: &[Stack]) -> Result<(), Stop> {
    for stack in stacks {
        for (source, declarations) in [
            (Source::Pack, &stack.pack),
            (Source::Project, &stack.project),
        ] {
            for (key, value) in declarations {
                if let Some(why) = problem(key, value, source) {
                    return Err(Stop::new(format!(
                        "cannot use the declaration `{key}` that the {} states for the stack `{}`: {why}; {}",
                        source.word(),
                        stack.name,
                        source.fix()
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Why a declaration cannot be used, or nothing when it can.
fn problem(key: &str, value: &Value, source: Source) -> Option<String> {
    let Some((_, kind)) = KEYS.iter().find(|(known, _)| *known == key) else {
        let known = KEYS.map(|(known, _)| known).join(", ");

        return Some(format!("the analyzer knows no such key, only {known}"));
    };

    match kind {
        Kind::Globs => match texts(value) {
            Some(globs) => globs.iter().find_map(|glob| glob_problem(glob)),
            None => Some("its value must be a list of globs".to_owned()),
        },
        Kind::Paths => match texts(value) {
            Some(paths) => paths.iter().find_map(|path| path_problem(path)),
            None => Some("its value must be a list of paths".to_owned()),
        },
        Kind::Number => match value.as_u64() {
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
        Kind::Opaque => match source {
            Source::Pack => None,
            Source::Project => Some("only a pack may state this key".to_owned()),
        },
    }
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
}

/// Why a path cannot be used, or nothing when it can.
fn path_problem(path: &str) -> Option<String> {
    if path.is_empty() {
        Some("a path in it is empty".to_owned())
    } else if leaves_its_folder(path) {
        Some(format!(
            "the path `{path}` is absolute or steps out of its folder with `..`"
        ))
    } else {
        None
    }
}

/// Why a glob cannot be used, or nothing when it can (spec KEY-5).
fn glob_problem(glob: &str) -> Option<String> {
    if glob.is_empty() {
        return Some("a glob in it is empty".to_owned());
    }

    if leaves_its_folder(glob) {
        return Some(format!(
            "the glob `{glob}` is absolute or steps out of its folder with `..`"
        ));
    }

    Glob::new(glob).err()
}
```

```rust path=src/hand_off.rs
//! Reads the hand-off file yak writes for one checkout (spec HO-1 to HO-4).
//!
//! The file is checked in a fixed order, so that two faults in one file always give the same line: that
//! it can be read, its format version, the shape of each field, and then each stack's interface version
//! (spec RUN-4).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::stop::Stop;
use crate::{HAND_OFF_FORMATS, INTERFACE_VERSION};

/// The largest whole number the format allows, which is the largest JSON carries exactly.
const LARGEST_NUMBER: u64 = 9_007_199_254_740_991;

/// The lengths, in hexadecimal digits, a commit's full name can have.
const COMMIT_NAME_LENGTHS: [usize; 2] = [40, 64];

/// What the analyzer takes from a hand-off file of format 1. Every field of the format is checked when
/// the file is read; a field is kept here once a part of the analyzer uses it.
#[derive(Debug)]
pub struct HandOff {
    /// The checkout's root, which is the folder to read whatever folder the analyzer was started in.
    pub root: PathBuf,
    /// The folders yak's detection looked in, each relative to the root; the first is `.`.
    pub folders: Vec<String>,
    /// Whether yak's detection stopped at its limit, so the folders it lists are not the whole workspace.
    pub truncated: bool,
    /// The stacks that carry declarations, in the order yak applies them.
    pub stacks: Vec<Stack>,
}

/// One stack and the declarations it carries, the pack's kept apart from the project's.
#[derive(Debug)]
pub struct Stack {
    /// The stack's name.
    pub name: String,
    /// The interface version the stack's declarations were written for.
    pub interface_version: Option<u64>,
    /// The pack's declarations: each key with its value.
    pub pack: Map<String, Value>,
    /// The project's own declarations under this stack.
    pub project: Map<String, Value>,
}

/// Reads and checks the hand-off file at the path given.
pub fn read(path: &Path) -> Result<HandOff, Stop> {
    let shown = path.display();
    let bytes = fs::read(path).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} cannot be read ({error}); pass the path of the file yak wrote for this \
             checkout"
        ))
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let document: Value = serde_json::from_slice(text).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} is not JSON (line {}, column {}); yak writes this file, so write it again \
             with yak",
            error.line(),
            error.column()
        ))
    })?;
    let shape = |problem: String| {
        Stop::new(format!(
            "the hand-off file {shown} is not format 1: {problem}; yak writes this file, so write it again with \
             yak"
        ))
    };
    let fields = document
        .as_object()
        .ok_or_else(|| shape("the file must hold one object".to_owned()))?;

    format_version(fields, &shown.to_string()).map_err(|stopped| match stopped {
        Fault::Stop(stop) => stop,
        Fault::Shape(problem) => shape(problem),
    })?;

    // The fields are checked in the order the format's table lists them (spec HO-2).
    let root = absolute_path(fields, "root").map_err(shape)?;

    let folders = folders(fields).map_err(shape)?;

    let truncated = truncated(fields).map_err(shape)?;
    let stacks = stacks(fields).map_err(shape)?;

    absolute_path(fields, "siblings").map_err(shape)?;
    base(fields).map_err(shape)?;

    let handed = HandOff {
        root,
        folders,
        truncated,
        stacks,
    };

    for stack in &handed.stacks {
        interface_version(stack, &shown.to_string())?;
    }

    Ok(handed)
}

/// What can be wrong with the format version: a format the analyzer does not read, or a shape.
enum Fault {
    /// The file states a format the analyzer does not read.
    Stop(Stop),
    /// The field is missing or is not a whole number.
    Shape(String),
}

/// Checks the format version, before any other field is looked at (spec HO-3).
fn format_version(fields: &Map<String, Value>, shown: &str) -> Result<(), Fault> {
    let stated = fields
        .get("formatVersion")
        .ok_or_else(|| Fault::Shape("`formatVersion` is missing".to_owned()))?
        .as_f64()
        .filter(|number| number.fract() == 0.0)
        .ok_or_else(|| Fault::Shape("`formatVersion` must be a whole number".to_owned()))?;
    let reads = HAND_OFF_FORMATS.map(|format| format.to_string()).join(", ");
    let oldest = f64::from(HAND_OFF_FORMATS[0]);
    let newest = f64::from(HAND_OFF_FORMATS[HAND_OFF_FORMATS.len() - 1]);

    if stated < oldest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, older than any this analyzer reads ({reads}); update \
             yak"
        ))));
    }

    if stated > newest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, newer than any this analyzer reads ({reads}); update \
             the analyzer's version pinned in the pack"
        ))));
    }

    Ok(())
}

/// A field that must be text holding an absolute path.
fn absolute_path(fields: &Map<String, Value>, name: &str) -> Result<PathBuf, String> {
    let text = fields
        .get(name)
        .ok_or_else(|| format!("`{name}` is missing"))?
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("`{name}` must be text that is not empty"))?;

    let path = PathBuf::from(text);

    if path.is_absolute() {
        Ok(path)
    } else {
        Err(format!("`{name}` must be an absolute path"))
    }
}

/// The object under `detection`, which allows two fields and no other.
fn detection(fields: &Map<String, Value>) -> Result<&Map<String, Value>, String> {
    let detection = fields
        .get("detection")
        .ok_or_else(|| "`detection` is missing".to_owned())?
        .as_object()
        .ok_or_else(|| "`detection` must be an object".to_owned())?;

    match detection
        .keys()
        .find(|key| !matches!(key.as_str(), "folders" | "truncated"))
    {
        Some(other) => Err(format!("`detection.{other}` is not a field of the format")),
        None => Ok(detection),
    }
}

/// The folders yak's detection looked in: at least one, the first `.`, each inside the root.
fn folders(fields: &Map<String, Value>) -> Result<Vec<String>, String> {
    let listed = detection(fields)?
        .get("folders")
        .ok_or_else(|| "`detection.folders` is missing".to_owned())?
        .as_array()
        .filter(|listed| !listed.is_empty())
        .ok_or_else(|| "`detection.folders` must be a list of at least one path".to_owned())?;

    for (index, folder) in listed.iter().enumerate() {
        let folder = folder
            .as_str()
            .filter(|folder| !folder.is_empty())
            .ok_or_else(|| {
                format!("`detection.folders[{index}]` must be text that is not empty")
            })?;

        if leaves_its_folder(folder) || folder.contains('\\') {
            return Err(format!(
                "`detection.folders[{index}]` must be a path inside the root, written with forward slashes"
            ));
        }
    }

    if listed[0] == "." {
        Ok(listed
            .iter()
            .filter_map(Value::as_str)
            .map(|folder| folder.trim_end_matches('/').to_owned())
            .collect())
    } else {
        Err("`detection.folders` must begin with `.`, the root itself".to_owned())
    }
}

/// Whether yak's detection stopped at its limit.
fn truncated(fields: &Map<String, Value>) -> Result<bool, String> {
    detection(fields)?
        .get("truncated")
        .ok_or_else(|| "`detection.truncated` is missing".to_owned())?
        .as_bool()
        .ok_or_else(|| "`detection.truncated` must be true or false".to_owned())
}

/// The stacks, each of which allows four fields and no other.
fn stacks(fields: &Map<String, Value>) -> Result<Vec<Stack>, String> {
    let listed = fields
        .get("stacks")
        .ok_or_else(|| "`stacks` is missing".to_owned())?
        .as_array()
        .ok_or_else(|| "`stacks` must be a list".to_owned())?;
    let mut stacks = Vec::with_capacity(listed.len());

    for (index, stack) in listed.iter().enumerate() {
        let stack = stack
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}]` must be an object"))?;
        let name = stack
            .get("name")
            .ok_or_else(|| format!("`stacks[{index}].name` is missing"))?
            .as_str()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| format!("`stacks[{index}].name` must be text that is not empty"))?;
        let interface_version = match stack.get("interfaceVersion") {
            Some(version) => Some(
                version
                    .as_u64()
                    .filter(|version| (1..=LARGEST_NUMBER).contains(version))
                    .ok_or_else(|| {
                        format!(
                            "`stacks[{index}].interfaceVersion` must be a positive whole number"
                        )
                    })?,
            ),
            None => None,
        };
        let pack = stack
            .get("pack")
            .ok_or_else(|| format!("`stacks[{index}].pack` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].pack` must be an object"))?;
        let project = stack
            .get("project")
            .ok_or_else(|| format!("`stacks[{index}].project` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].project` must be an object"))?;

        if let Some((key, _)) = project.iter().find(|(_, value)| !is_project_value(value)) {
            return Err(format!(
                "`stacks[{index}].project.{key}` must be a list of text or a positive whole number"
            ));
        }

        if let Some(other) = stack.keys().find(|key| {
            !matches!(
                key.as_str(),
                "name" | "interfaceVersion" | "pack" | "project"
            )
        }) {
            return Err(format!(
                "`stacks[{index}].{other}` is not a field of the format"
            ));
        }

        stacks.push(Stack {
            name: name.to_owned(),
            interface_version,
            pack: pack.clone(),
            project: project.clone(),
        });
    }

    Ok(stacks)
}

/// Whether a value is one the format lets a project state: a list of text, or a positive whole number.
fn is_project_value(value: &Value) -> bool {
    match value {
        Value::Array(entries) => entries.iter().all(Value::is_string),
        other => other
            .as_u64()
            .is_some_and(|number| (1..=LARGEST_NUMBER).contains(&number)),
    }
}

/// Checks the base commit, where the file states one: a commit's full name in lower-case hexadecimal
/// digits.
fn base(fields: &Map<String, Value>) -> Result<(), String> {
    let Some(base) = fields.get("base") else {
        return Ok(());
    };

    base.as_str()
        .filter(|name| {
            COMMIT_NAME_LENGTHS.contains(&name.len())
                && name
                    .bytes()
                    .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
        })
        .map(|_| ())
        .ok_or_else(|| {
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits".to_owned()
        })
}

/// Checks that a stack's declarations were written for the interface version this analyzer speaks, and
/// that a stack which states no version holds no declarations (spec HO-4).
fn interface_version(stack: &Stack, shown: &str) -> Result<(), Stop> {
    let name = &stack.name;

    match stack.interface_version {
        Some(INTERFACE_VERSION) => Ok(()),
        Some(stated) => {
            let update = if stated > INTERFACE_VERSION {
                "the analyzer's version pinned in the pack"
            } else {
                "the stack's declarations"
            };

            Err(Stop::new(format!(
                "the stack `{name}` in {shown} states declarations for interface version {stated}, and this \
                 analyzer speaks version {INTERFACE_VERSION}; update {update}"
            )))
        }
        None if stack.pack.is_empty() && stack.project.is_empty() => Ok(()),
        None => Err(Stop::new(format!(
            "the stack `{name}` in {shown} holds declarations and states no interface version; state the \
             version they were written for"
        ))),
    }
}

/// Whether a path written with forward slashes is absolute or steps out of its folder with `..`.
pub fn leaves_its_folder(path: &str) -> bool {
    let has_drive = path.as_bytes().get(1) == Some(&b':');

    path.starts_with('/') || has_drive || path.split('/').any(|segment| segment == "..")
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations and the
//! packages. Imports come with the parts that read them.

use std::env;
use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::files::{self, File};
use crate::hand_off;
use crate::marks::{self, Marks};
use crate::notice::{self, Notice};
use crate::packages::{self, Package, Packages};
use crate::stop::Stop;

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
    let mut notices = Vec::new();
    let handed = match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice::new(
                    "detection-cut-short",
                    None,
                    "yak's detection stopped at its limit of folders, so the folders the hand-off file lists \
                     are not the whole workspace."
                        .to_owned(),
                ));
            }

            Some(handed)
        }
        None => {
            notices.push(Notice::new(
                "no-hand-off-file",
                None,
                "No hand-off file was given, so the project is read from its own standard files, with no \
                 declarations."
                    .to_owned(),
            ));

            None
        }
    };

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match &handed {
        Some(handed) => handed.root.clone(),
        None => env::current_dir().map_err(|error| {
            Stop::new(format!(
                "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
            ))
        })?,
    };
    let files = files::list(&root, &mut notices)?;
    let marks = marks::apply(&files, handed.as_ref(), &mut notices)?;
    let packages = packages::read(
        &root,
        &files,
        &marks.data,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    )?;
    let document = json!({
        "files": files.iter().map(|each| file(each, &marks, &packages)).collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    Ok(format!("{document:#}"))
}

/// One file as it is printed.
fn file(file: &File, marks: &Marks, packages: &Packages) -> Value {
    json!({
        "data": marks.data.contains(&file.path),
        "generated": marks.generated.contains(&file.path),
        "kind": file.kind.name(),
        "package": packages.of_file.get(&file.path),
        "path": file.path,
        "tracked": file.tracked,
    })
}

/// One package as it is printed.
fn package(package: &Package) -> Value {
    json!({
        "detected": package.detected,
        "folder": package.folder,
        "name": package.name,
        "workspace": package.workspace,
    })
}
```

```rust path=src/main.rs
//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: so far its hand-off file, its
//! files and its packages. It reports no findings yet.

mod cli;
mod declarations;
mod files;
mod git;
mod glob;
mod graph;
mod hand_off;
mod marks;
mod notice;
mod packages;
mod stop;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;

/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
        eprintln!(
            "{}",
            stop::line(&format!(
                "a fault in the analyzer itself, not in the project: {fault}"
            ))
        );
    }));

    panic::catch_unwind(run).unwrap_or_else(|_| ExitCode::from(stop::FAULT))
}

/// Does what the command line asks.
fn run() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match cli::parse(&args) {
        Ok(Command::Version) => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        Ok(Command::Interface) => {
            println!(
                "{}",
                serde_json::json!({ "handOffFormats": HAND_OFF_FORMATS, "interfaceVersion": INTERFACE_VERSION })
            );

            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => match graph::read(hand_off.as_deref()) {
            Ok(document) => {
                println!("{document}");

                ExitCode::SUCCESS
            }
            Err(stopped) => {
                eprintln!("{}", stopped.line());

                ExitCode::from(stop::STOPPED)
            }
        },
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

            ExitCode::from(stop::REFUSED)
        }
    }
}
```

- [ ] **Step 5: Run every gate**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo build --release --locked && node scripts/check-floor.mjs
```

Expected: each exits 0. The Rust tests report 7 passed in the binary's own tests (the glob
matcher's), 6 for `cli`, 5 for `declarations`, 16 for `file-list`, 10 for `hand-off`, 2 for
`hand-off-schema` and 9 for `packages`; the Node tests 45 passed; and the last line says what the
binary asks of the system, which is no more than the floor allows.

- [ ] **Step 6: Commit, push, and open the pull request as a draft on the first plan's**

```bash
git add Cargo.toml Cargo.lock src tests
git commit -m "feat(packages): read the declarations' marks and the packages" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -u origin feat/reading-2-file-list
gh pr create --draft --base feat/reading-a-project --head feat/reading-2-file-list --title "feat: the file list, the declarations' marks and the packages" --body "A draft, opened so that CI runs on all five targets. Not for merging until the owner has validated the plan."
gh pr checks --watch
```

Expected: the pull request's seven checks pass. Its description is written out in full when the plan
is handed over for validation.
