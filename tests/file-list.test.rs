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
                let content = fs::read(&path)
                    .unwrap_or_else(|error| panic!("{} can be read: {error}", path.display()));

                found.insert(path, content);
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
