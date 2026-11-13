# Reading a project, part 3: parsing scripts and finding their imports: implementation plan

**Goal:** `graph` parses every script and type declaration file that is not data, prints each one's
imports, with the form, the line and the marks the spec asks for, and the count of those it cannot
follow; and it stops, with one line, on a tracked script that does not parse.

**Architecture:** `src/imports.rs` parses one script with `oxc_parser`, as a module and then as a
script, and walks its syntax tree with oxc's visitor to collect the imports. `src/scripts.rs` reads
each script of the file list from disk, after the packages, and turns a failure into the stop or the
notice PARSE-2 gives. `graph` adds the imports to each script it read. Where each import leads comes
in the fourth plan.

**Spec:** docs/specs/2026-11-11-reading-a-project-design.md

**Implements:** PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-5, RUN-4, RUN-5, LIST-3, FIX-2, FIX-5, DEP-1, DEP-3

This is the third of the item's plans. It is worked on its own branch, `feat/reading-3-imports`, made
from the second plan's branch, and its pull request is a draft stacked on the second plan's.

## Global constraints

- Every oxc crate is pinned with `=` to one version, the same for all, and Dependabot moves them in
  one pull request (DEP-1, DEP-3).
- A script is parsed as a module first, and as a script when that fails; only then is it broken
  (PARSE-1).
- A tracked script that cannot be read, is not valid UTF-8, or does not parse stops the run with one
  line, the first by path; this is RUN-4's stop 12, checked after the packages' stops. An untracked
  one is skipped with a notice (PARSE-2).
- A data file is never parsed (PARSE-6).
- What is an import, and what is not, is IMP-1 to IMP-3's list and nothing else.

## The decisions this plan makes

1. **Five oxc crates are added, each at `=0.156.0`:** `oxc_allocator`, `oxc_ast`, `oxc_ast_visit`,
   `oxc_parser` and `oxc_span`. 0.156.0 was the newest version at least two weeks old on 2026-11-12;
   it asks for Rust 1.97, and the repository pins 1.99.1. `oxc_semantic` is not added: nothing in
   this item counts uses.
2. **The forms of an import, as `graph` prints them:** `import`, `export` (a re-export of names),
   `export-all`, `import-equals`, `dynamic`, `require` and `type-import`. Each import also has
   `specifier`, `line`, `typeOnly` (for `import type`, `export type` and a type import) and `query`
   (IMP-4). A script that was read has `imports`, in the order written, and `computedImports`; a file
   that was not read has neither.
3. **The language of a script comes from its ending:** TypeScript for `.ts`, `.mts` and `.cts`;
   TypeScript with JSX for `.tsx`; a type declaration file as TypeScript's definition; and JavaScript
   with JSX allowed for every JavaScript ending.
4. **A string's value is written as oxc displays it,** which keeps a lone surrogate visible instead
   of failing; a template with no substitution gives its cooked text.
5. **The line of a broken script** is the line of the first error oxc reports, and the line holds
   oxc's own message. oxc words its messages differently from one release to the next, so the tests
   check the line's start and end.
6. **`require` counts wherever it is the bare name,** a local of that name included, as IMP-1 says;
   `module.require(…)` and any other call do not.
7. **Scripts are parsed one after another.** On yakshaver's main checkout, 2,056 scripts and 10,365
   imports took 1.8 seconds in all on this machine, about what the design spec measured for the same
   work. Nothing in the spec asks for work in parallel, and LIVE-2 weighs the times.
8. **DEP-3 lands here, not in the sixth plan,** because the oxc crates arrive here.
9. **A file gone from disk between git's list and its reading** leaves the list with a `file-gone`
   notice (LIST-3); no test can make that race happen on purpose.
10. **The notice kinds this plan adds:** `untracked-file-skipped` and `file-gone`.

## Review focus

- **A file gone between the listing and the reading** is handled in `src/scripts.rs` and has no test,
  since a test cannot time that race. A reviewer reads it there.
- **The wording of oxc's messages** reaches the stop's line; the tests check only what the analyzer
  writes around it.
- **The time of a reading** on the largest named repository is weighed in the sixth plan's check
  (LIVE-2): two readings at this speed would pass the design spec's 3-second bar on neither
  repository of that size, and the owner is told then, as LIVE-2 says.
- **Vue components, MDX documents and HTML pages** are read in the fifth plan; until then they have no
  `imports`, which a test pins.

### Task 1: parsing scripts and finding their imports

**Implements:** PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-5, RUN-4, RUN-5, LIST-3, FIX-2, FIX-5, DEP-1

**Test:** `cargo test --release --locked --test imports --test file-list`

**Files:**
- Create: `src/imports.rs`, `src/scripts.rs`
- Modify: `src/graph.rs`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`
- Test: `tests/imports.test.rs`, `tests/file-list.test.rs`

**Interfaces:**
- Consumes: the second plan's `files::File` and `files::Kind`, `marks::Marks`, `notice::Notice` and
  `tests/support/project.rs`.
- Produces: `imports::read`, `imports::Import`, `imports::Form`, `imports::Read` and
  `imports::Broken`; `scripts::read` and `scripts::Scripts`, which the fourth plan resolves from.

- [ ] **Step 1: Write the failing tests**

The second plan's test that compares whole files changes with this task: a script now has its imports.

```rust path=tests/imports.test.rs
//! Runs the built binary on made-up projects and checks the imports it finds in each script, and how it
//! treats a script it cannot parse (spec PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-5, RUN-4).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The file a document lists under the path given.
fn file<'a>(document: &'a Value, path: &str) -> &'a Value {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .unwrap_or_else(|| panic!("{path} is listed"))
}

/// One import as `graph` prints it, with no query and not for types alone.
fn plain(specifier: &str, form: &str, line: usize) -> Value {
    json!({ "form": form, "line": line, "query": false, "specifier": specifier, "typeOnly": false })
}

/// The same import, for types alone.
fn types(specifier: &str, form: &str, line: usize) -> Value {
    json!({ "form": form, "line": line, "query": false, "specifier": specifier, "typeOnly": true })
}

#[test]
fn finds_every_form_of_import_with_its_line() {
    let project = Project::new("imports-forms");

    project
        .write(
            "src/main.ts",
            "import a from './a';\n\
             import type { B } from './b';\n\
             import './side-effect';\n\
             export { c } from './c';\n\
             export type { D } from './d';\n\
             export * from './e';\n\
             export * as f from './f';\n\
             import g = require('./g');\n\
             const h = await import('./h');\n\
             const i = import(`./i`);\n\
             const j = require('./j');\n\
             const k = require(`./k`);\n\
             type L = import('./l').L;\n\
             const m = import('./m', { with: { type: 'json' } });\n\
             function later() {\n\
             \x20 return import('./nested');\n\
             }\n",
        )
        .commit();

    let document = project.graph();
    let main = file(&document, "src/main.ts");

    assert_eq!(
        main["imports"],
        json!([
            plain("./a", "import", 1),
            types("./b", "import", 2),
            plain("./side-effect", "import", 3),
            plain("./c", "export", 4),
            types("./d", "export", 5),
            plain("./e", "export-all", 6),
            plain("./f", "export-all", 7),
            plain("./g", "import-equals", 8),
            plain("./h", "dynamic", 9),
            plain("./i", "dynamic", 10),
            plain("./j", "require", 11),
            plain("./k", "require", 12),
            types("./l", "type-import", 13),
            plain("./m", "dynamic", 14),
            plain("./nested", "dynamic", 16),
        ])
    );
    assert_eq!(main["computedImports"], json!(0));
}

#[test]
fn counts_the_imports_it_cannot_follow_and_reads_nothing_else_as_one() {
    let project = Project::new("imports-computed");

    project
        .write(
            "src/load.js",
            "/// <reference path=\"./ref.d.ts\" />\n\
             /** @type {import('./jsdoc').T} */\n\
             const name = 'x';\n\
             import(`./pages/${name}`);\n\
             import(name);\n\
             require(name + '.js');\n\
             new Worker('./worker.js');\n\
             module.require('./not-bare');\n\
             vi.mock('./mocked');\n",
        )
        .commit();

    let document = project.graph();
    let load = file(&document, "src/load.js");

    assert_eq!(load["imports"], json!([]));
    assert_eq!(load["computedImports"], json!(3));
}

#[test]
fn marks_an_import_that_carries_a_query_or_a_fragment_and_not_a_hash_name() {
    let project = Project::new("imports-query");

    project
        .write(
            "src/main.ts",
            "import text from './notes.md?raw';\n\
             import part from './page.html#top';\n\
             import alias from '#utils/text';\n",
        )
        .commit();

    let document = project.graph();
    let marks: Vec<(&str, bool)> = file(&document, "src/main.ts")["imports"]
        .as_array()
        .expect("a list of imports")
        .iter()
        .map(|import| {
            (
                import["specifier"].as_str().expect("a specifier"),
                import["query"] == json!(true),
            )
        })
        .collect();

    assert_eq!(
        marks,
        [
            ("./notes.md?raw", true),
            ("./page.html#top", true),
            ("#utils/text", false)
        ]
    );
}

#[test]
fn reads_each_kind_of_script_in_its_own_language() {
    let project = Project::new("imports-languages");

    project
        .write(
            "src/view.jsx",
            "import a from './a';\nexport const V = () => <div />;\n",
        )
        .write(
            "src/old.js",
            "import b from './b';\nexport const W = () => <span />;\n",
        )
        .write(
            "src/page.tsx",
            "import c from './c';\nexport const P = (): JSX.Element => <p />;\n",
        )
        .write(
            "src/server.cjs",
            "const d = require('./d');\nmodule.exports = d;\n",
        )
        .write(
            "src/tool.mts",
            "import e from './e';\nexport const t: number = e;\n",
        )
        .write(
            "types/env.d.ts",
            "declare module 'virtual' {\n  export * from './f';\n}\n",
        )
        .write("src/marked.ts", "\u{feff}import g from './g';\n")
        .commit();

    let document = project.graph();
    let first = |path: &str| file(&document, path)["imports"][0]["specifier"].clone();

    assert_eq!(first("src/view.jsx"), json!("./a"));
    assert_eq!(first("src/old.js"), json!("./b"));
    assert_eq!(first("src/page.tsx"), json!("./c"));
    assert_eq!(first("src/server.cjs"), json!("./d"));
    assert_eq!(first("src/tool.mts"), json!("./e"));
    assert_eq!(first("types/env.d.ts"), json!("./f"));
    assert_eq!(first("src/marked.ts"), json!("./g"));
}

#[test]
fn reads_a_script_that_parses_only_as_a_script() {
    let project = Project::new("imports-script");

    // `with` is not allowed in a module, which is strict, and is in a script.
    project
        .write(
            "legacy/old.js",
            "with (Math) {\n  require('./helper');\n}\n",
        )
        .commit();

    let document = project.graph();

    assert_eq!(
        file(&document, "legacy/old.js")["imports"],
        json!([plain("./helper", "require", 2)])
    );
}

#[test]
fn reads_no_file_that_is_not_a_script() {
    let project = Project::new("imports-not-scripts");

    project
        .write("src/App.vue", "<script>import a from './a';</script>\n")
        .write("docs/intro.mdx", "import b from './b';\n")
        .write("public/index.html", "<script src=\"./c.js\"></script>\n")
        .write("data.json", "{}")
        .commit();

    let document = project.graph();

    for path in paths(&document) {
        assert_eq!(file(&document, path).get("imports"), None, "{path}");
    }
}

#[test]
fn stops_on_a_tracked_script_that_does_not_parse_the_first_by_path() {
    let project = Project::new("imports-broken-tracked");

    project
        .write("b/bad.ts", "let = ;\n")
        .write("a/bad.ts", "export const ok = 1;\nconst = 2;\n")
        .commit();

    let line = stopped(&project.run(|_| {}));

    assert!(
        line.starts_with("yak-analyzer: the file a/bad.ts does not parse (line 2: "),
        "{line}"
    );
    assert!(
        line.ends_with("); fix it, or declare it as data if it is broken on purpose"),
        "{line}"
    );
}

#[test]
fn stops_on_a_tracked_script_that_is_not_valid_utf8() {
    let project = Project::new("imports-not-utf8");

    project
        .write("src/main.ts", "export const a = 1;\n")
        .commit();
    std::fs::write(
        project.root.join("src/main.ts"),
        b"export const a = '\xFF';\n",
    )
    .expect("the file can be written");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the file src/main.ts is not valid UTF-8; fix it, or declare it as data if it is broken \
         on purpose"
    );
}

#[test]
fn skips_an_untracked_script_that_does_not_parse_with_a_notice() {
    let project = Project::new("imports-broken-untracked");

    project
        .write("src/main.ts", "import './scratch';\n")
        .commit()
        .write("scratch.ts", "let half = ;\n");

    let document = project.graph();

    assert_eq!(file(&document, "scratch.ts").get("imports"), None);
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "untracked-file-skipped"]
    );
    assert!(
        document["notices"][1]["sentence"]
            .as_str()
            .is_some_and(|sentence| sentence
                .starts_with("The untracked file does not parse (line 1: ")
                && sentence.ends_with("), so its imports are not read."))
    );
}

#[test]
fn never_parses_a_data_file_so_a_broken_one_stops_nothing() {
    let project = Project::new("imports-data");

    project
        .write("fixtures/broken.ts", "let = ;\n")
        .write("src/main.ts", "import x from '../fixtures/broken';\n")
        .commit();

    let declared = json!([{
        "name": "typescript",
        "interfaceVersion": 1,
        "pack": {},
        "project": { "data": ["fixtures/**"] }
    }]);
    let handed = project.hand_off(&project.root, &["."], declared);
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &handed]);
    }));

    assert_eq!(file(&document, "fixtures/broken.ts")["data"], json!(true));
    assert_eq!(file(&document, "fixtures/broken.ts").get("imports"), None);
    assert_eq!(
        file(&document, "src/main.ts")["imports"][0]["specifier"],
        json!("../fixtures/broken")
    );
}

#[test]
fn reports_a_field_of_the_wrong_form_before_a_script_that_does_not_parse() {
    let project = Project::new("imports-order");

    project
        .write("package.json", r#"{ "name": 4 }"#)
        .write("src/bad.ts", "let = ;\n")
        .commit();

    let line = stopped(&project.run(|_| {}));

    assert!(
        line.starts_with("yak-analyzer: the field `name` in package.json"),
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
    // A script is read, so it has its imports, which in these empty files are none.
    let file = |path: &str, kind: &str, tracked: bool| {
        let mut expected = json!({
            "data": false,
            "generated": false,
            "kind": kind,
            "package": ".",
            "path": path,
            "tracked": tracked,
        });

        if kind == "script" || kind == "declaration" {
            expected["computedImports"] = json!(0);
            expected["imports"] = json!([]);
        }

        expected
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

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test imports --test file-list
```

Expected: cargo refuses, because the manifest names no test called `imports` yet.

- [ ] **Step 3: Name the test and add the oxc crates**

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

[[test]]
name = "imports"
path = "tests/imports.test.rs"

[dependencies]
# Parse scripts into syntax trees and walk them; released together, so pinned together (spec AN-2, DEP-3).
oxc_allocator = "=0.156.0"
oxc_ast = "=0.156.0"
oxc_ast_visit = "=0.156.0"
oxc_parser = "=0.156.0"
oxc_span = "=0.156.0"
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
name = "castaway"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dec551ab6e7578819132c713a93c022a05d60159dc86e7a7050223577484c55a"
dependencies = [
 "rustversion",
]

[[package]]
name = "cfg-if"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"

[[package]]
name = "compact_str"
version = "0.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "79fcda08c33bb58b97008b2cdada6622500e949e060f5913361763121abd2416"
dependencies = [
 "castaway",
 "cfg-if",
 "itoa",
 "static_assertions",
 "zmij",
]

[[package]]
name = "cow-utils"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "417bef24afe1460300965a25ff4a24b8b45ad011948302ec221e8a0a81eb2c79"

[[package]]
name = "data-encoding"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "displaydoc"
version = "0.2.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6232dd377dcc64799954cbd3a9bb882e9cdc1308ccd87b1c098f1fb2eaf82a8"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "dragonbox_ecma"
version = "0.1.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fd8e701084c37e7ef62d3f9e453b618130cbc0ef3573847785952a3ac3f746bf"

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
name = "fastrand"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da7c62ceae207dd37ea5b845da6a0696c799f85e97da1ab5b7910be3c1c80223"

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
 "num-bigint 0.4.8",
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
name = "icu_collections"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fe090a42a0d3140fed8c63263ed1fecc02422cb2f4bfe55cd8de2de1b8e105dd"
dependencies = [
 "displaydoc",
 "potential_utf",
 "utf8_iter",
 "yoke",
 "zerofrom",
 "zerovec",
]

[[package]]
name = "icu_locale_core"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "25d29b57c17c6528d0dc4c2141cfa2d08200465acb537a3909c8b9b8a4a984c0"
dependencies = [
 "displaydoc",
 "litemap",
 "serde",
 "tinystr",
 "writeable",
 "zerovec",
]

[[package]]
name = "icu_locale_fallback"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "406e9ae5b95d8aa37db6293405931b766a6172d1848daee22d70eb44aa78a43c"
dependencies = [
 "icu_locale_core",
 "icu_locale_fallback_data",
 "icu_provider",
 "potential_utf",
 "tinystr",
 "zerovec",
]

[[package]]
name = "icu_locale_fallback_data"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3632ebbd0c5c3637ba1a40bdf8288f2cb9d2268e73441a69d5fc73d7cfbaa87c"

[[package]]
name = "icu_provider"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "413099315636c588ee8f011190b329b971a09a8fae9d157e01dcae8379468514"
dependencies = [
 "displaydoc",
 "icu_locale_core",
 "serde",
 "stable_deref_trait",
 "writeable",
 "yoke",
 "zerofrom",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "icu_segmenter"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "61b195dc605f81aa181bec9b3bd14e26cc71b903e4644f6af578ca9772c7a710"
dependencies = [
 "icu_collections",
 "icu_locale_fallback",
 "icu_provider",
 "icu_segmenter_data",
 "potential_utf",
 "smallvec",
 "utf8_iter",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "icu_segmenter_data"
version = "2.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "913a7d294ad056474749152107309fdee3ec17d2482d1ab70fdc61a81c64b9a3"

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
name = "litemap"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47d9d19d1d6efa0109d2f65ff4c85cddd50bd572e5a00127ab10987290bcefae"

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
name = "nonmax"
version = "0.5.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "610a5acd306ec67f907abe5567859a3c693fb9886eb1f012ab8f2a47bef3db51"

[[package]]
name = "num"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "35bd024e8b2ff75562e5f34e7f4905839deb4b22955ef5e73d2fea1b9813cb23"
dependencies = [
 "num-bigint 0.4.8",
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
name = "num-bigint"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93e7820bc0a80a0238e650327316f929ba18d5be054b647490a3a6a339f3e7c0"
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
 "num-bigint 0.4.8",
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
name = "oxc_allocator"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11eaca3827c60c9da743111a49dff9a8e274e558e0e725e9a717f31ea173da0f"
dependencies = [
 "allocator-api2",
 "hashbrown",
 "oxc_data_structures",
 "rustc-hash",
]

[[package]]
name = "oxc_ast"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5f68bed0e97be9c9450a85f7fe0b1d62f85bf23f0a2d4f5c665bf4b848be1632"
dependencies = [
 "bitflags",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_data_structures",
 "oxc_diagnostics",
 "oxc_estree",
 "oxc_regular_expression",
 "oxc_span",
 "oxc_str",
 "oxc_syntax",
]

[[package]]
name = "oxc_ast_macros"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94f8bdc02323a201e5a3b29bb87bac855ea642ee26a630c5b05e5f463fa71b77"
dependencies = [
 "phf",
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "oxc_ast_visit"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "db583366340d5221fe2129e0240e0413cb58e6c6e926740a493d6e82f5c5e40f"
dependencies = [
 "oxc_allocator",
 "oxc_ast",
 "oxc_span",
 "oxc_syntax",
]

[[package]]
name = "oxc_data_structures"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d8e8e829b37d1654ed3fb4b4d554eac80da790a2dcea333b5f4ca3209bca795"

[[package]]
name = "oxc_diagnostics"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "433f690c5235d132aebde47e385d3e21ef244b28b1f0d1ad3d479b9679bc7be2"
dependencies = [
 "bytecount",
 "cow-utils",
 "itoa",
 "memchr",
 "oxc_span",
 "percent-encoding",
 "smallvec",
 "textwrap",
 "unicode-segmentation",
 "unicode-width",
]

[[package]]
name = "oxc_ecmascript"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a3e6e129f51055adb903e0fd5d0d412e1adf2244c7f592596850fc1a3ae33c2b"
dependencies = [
 "dragonbox_ecma",
 "itoa",
 "memchr",
 "num-bigint 0.5.1",
 "num-traits",
 "oxc_ast",
 "oxc_data_structures",
 "oxc_span",
 "oxc_str",
 "oxc_syntax",
]

[[package]]
name = "oxc_estree"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d20e8ab103436f676b4f2e67838de0f658fe06409804d8f5741ac6ec09b3d539"

[[package]]
name = "oxc_index"
version = "5.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "191884bee6c3744909a51acc7d78d4ae370d817b25875b10642f632327b6296e"
dependencies = [
 "nonmax",
 "serde",
]

[[package]]
name = "oxc_parser"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b33d6f23016689c207ecf7ccf62656bbb4b453808e3915dbe562b330f7bf53a0"
dependencies = [
 "bitflags",
 "cow-utils",
 "memchr",
 "num-bigint 0.5.1",
 "num-traits",
 "oxc_allocator",
 "oxc_ast",
 "oxc_data_structures",
 "oxc_diagnostics",
 "oxc_ecmascript",
 "oxc_regular_expression",
 "oxc_span",
 "oxc_str",
 "oxc_syntax",
 "rustc-hash",
 "seq-macro",
]

[[package]]
name = "oxc_regular_expression"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d105b439894681c19d35b9b42e4daac6425f0753ce1a289232502f2d55347468"
dependencies = [
 "bitflags",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_diagnostics",
 "oxc_span",
 "oxc_str",
 "phf",
 "rustc-hash",
 "unicode-id-start",
]

[[package]]
name = "oxc_span"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "982988d6278f598e8a38ea4ab03b4883b673ac2cb5d96f588a42d8e868eed002"
dependencies = [
 "compact_str",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_estree",
 "oxc_str",
]

[[package]]
name = "oxc_str"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3c0b6c6915f8fc8a395db7065b5b764067a9694c1e88320d095c9e88ce30d28f"
dependencies = [
 "compact_str",
 "hashbrown",
 "memchr",
 "oxc_allocator",
 "oxc_estree",
]

[[package]]
name = "oxc_syntax"
version = "0.156.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0e64bb2b43d48a43636ceaafaed251e98ee79daca32558bfe99664064ff1923"
dependencies = [
 "bitflags",
 "cow-utils",
 "dragonbox_ecma",
 "nonmax",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_estree",
 "oxc_index",
 "oxc_span",
 "oxc_str",
 "phf",
 "unicode-id-start",
]

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
name = "phf"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "010378780309880b08997fae13be7834dba947d36393bd372f2b1556deb2a2f6"
dependencies = [
 "phf_macros",
 "phf_shared",
 "serde",
]

[[package]]
name = "phf_generator"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aeb62e0959d5a1bebc965f4d15d9e2b7cea002b6b0f5ba8cde6cc26738467100"
dependencies = [
 "fastrand",
 "phf_shared",
]

[[package]]
name = "phf_macros"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5fa8d0ca26d424d27630da600c6624696e7dec8bf7b3b492b383c5dc49e5e085"
dependencies = [
 "phf_generator",
 "phf_shared",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "phf_shared"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6fd9027e2d9319be6349febd1db4e8d02aa544921200c9b777720ac34a3aa89"
dependencies = [
 "siphasher",
]

[[package]]
name = "potential_utf"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d83eb9bc6d8e5cf568e7a1101d60ee05e81ed50ea106026f3d18deeb046d7661"
dependencies = [
 "serde_core",
 "writeable",
 "zerovec",
]

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
name = "rustc-hash"
version = "2.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6b1e7f9a428571be2dc5bc0505c13fb6bf936822b894ec87abf8a08a4e51742d"

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
name = "seq-macro"
version = "0.3.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1bc711410fbe7399f390ca1c3b60ad0f53f80e95c5eb935e52268a0e2cd49acc"

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
name = "siphasher"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "33f4fe9184a62d842c9ef383018f3306d8ba224fd9d836f56d7288308847c256"

[[package]]
name = "smallvec"
version = "1.16.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "149309498a5f7334b12ddae0cad8aba81ce785fe91755068867e017cf98fee46"
dependencies = [
 "serde",
]

[[package]]
name = "smawk"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e8e2fb0f499abb4d162f2bedad68f5ef91a1682b5a03596ddb67efd37768d100"

[[package]]
name = "stable_deref_trait"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596"

[[package]]
name = "static_assertions"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2eb9349b6444b326872e140eb1cf5e7c522154d69e7a0ffb0fb81c06b37543f"

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
name = "synstructure"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "901704edd0dfe137f1987838ee4f259e4e063c31371bdb423f7ae38ec6f77f02"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "textwrap"
version = "0.16.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ecfad6c3abc80a577f2b91c1e412ee57e7a060d430b553c1b0c940974ebcd49"
dependencies = [
 "icu_segmenter",
 "smawk",
 "unicode-width",
]

[[package]]
name = "tinystr"
version = "0.8.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc840c238154d58f9e86b2792261edfde565a17696e1cda1a449de27979801a"
dependencies = [
 "displaydoc",
 "serde_core",
 "zerovec",
]

[[package]]
name = "unicode-general-category"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b993bddc193ae5bd0d623b49ec06ac3e9312875fdae725a975c51db1cc1677f"

[[package]]
name = "unicode-id-start"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0825d2e09dcae814a4273c1e1435979d414ede25b2ee235b1545da6682483765"

[[package]]
name = "unicode-ident"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563"

[[package]]
name = "unicode-segmentation"
version = "1.13.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6f5d3c3b1bf09027a88a6bc961fc00497d651009560b5463668dc81b0fa87a8"

[[package]]
name = "unicode-width"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ca718471876992059f251e33aafc334346ada08042b7661dd1fe313a9bced557"

[[package]]
name = "utf8_iter"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "af3bd9d3e571c26fcce9d033b87ea5ab061b8d2ffe9b7a5675d810d68f03edf0"
dependencies = [
 "utf_types",
]

[[package]]
name = "utf_types"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3a3b425286980a367278453f3b6d28cdcb134b574e5a35824a6ff5eebad50bb6"

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
name = "writeable"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ad82d2a33cdc9674dc7465672f271e096168fcdbe0f799d9e6db8c5892679dc"

[[package]]
name = "yak-analyzer"
version = "0.1.0"
dependencies = [
 "jsonschema",
 "oxc_allocator",
 "oxc_ast",
 "oxc_ast_visit",
 "oxc_parser",
 "oxc_span",
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
name = "yoke"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "417ef3fcf6c300b7dbd25c85531a746931c02ad536f3cf4786ed1e88223340b3"
dependencies = [
 "stable_deref_trait",
 "yoke-derive",
 "zerofrom",
]

[[package]]
name = "yoke-derive"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec8ebde2db3681e8c9980cc27822030e68752690ddfa9473e739aeb4dbde6d71"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
 "synstructure",
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
name = "zerofrom"
version = "0.1.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ec05a11813ea801ff6d75110ad09cd0824ddba17dfe17128ea0d5f68e6c5272"
dependencies = [
 "zerofrom-derive",
]

[[package]]
name = "zerofrom-derive"
version = "0.1.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f75b4683f6c7f45248d4d64056a24298c6281e0993356d7d1b4a1a962ef10d4a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
 "synstructure",
]

[[package]]
name = "zerotrie"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ea269c3bd32f0a32c321907a2ae912ba6f4649bb0fc764a15627e99a7095a3f"
dependencies = [
 "displaydoc",
 "yoke",
 "zerofrom",
 "zerovec",
]

[[package]]
name = "zerovec"
version = "0.11.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f58a2799cfe9a1670c7ea8edbd52ef8edb93cb37a7f83bf31ba9e35d9dae6a40"
dependencies = [
 "serde",
 "yoke",
 "zerofrom",
 "zerovec-derive",
]

[[package]]
name = "zerovec-derive"
version = "0.11.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c3743ddbf47738d929cd1ae2e199d40d0162979dfe31aef8ae646d2bf00bf97a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
]

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"
```

- [ ] **Step 4: Write the parser's part, the reading of scripts, and the output**

```rust path=src/imports.rs
//! What an import is: parsing a script and finding every import in it (spec PARSE-1, PARSE-2, IMP-1 to
//! IMP-5).

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ExportAllDeclaration, ExportFromDeclaration, Expression,
    ImportDeclaration, ImportExpression, StringLiteral, TSImportEqualsDeclaration, TSImportType,
    TSModuleReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

/// How an import is written (spec IMP-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// `import … from '…'`, or `import '…'`.
    Import,
    /// `export … from '…'`.
    Export,
    /// `export * from '…'`, and `export * as name from '…'`.
    ExportAll,
    /// `import name = require('…')`.
    ImportEquals,
    /// `import('…')` with a literal.
    Dynamic,
    /// `require('…')` with a literal, where `require` is the bare name.
    Require,
    /// `import('…')` written where a type is expected.
    TypeImport,
}

impl Form {
    /// The form's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Export => "export",
            Self::ExportAll => "export-all",
            Self::ImportEquals => "import-equals",
            Self::Dynamic => "dynamic",
            Self::Require => "require",
            Self::TypeImport => "type-import",
        }
    }
}

/// One import.
#[derive(Debug, PartialEq, Eq)]
pub struct Import {
    /// The specifier as it is written.
    pub specifier: String,
    /// How it is written.
    pub form: Form,
    /// Whether it brings in types alone: `import type`, `export type`, or a type import.
    pub type_only: bool,
    /// The line it begins on, counted from one.
    pub line: usize,
}

impl Import {
    /// Whether the specifier carries a query or a fragment: a `?`, or a `#` that is not its first
    /// character (spec IMP-4).
    pub fn has_query(&self) -> bool {
        self.specifier.contains('?')
            || self
                .specifier
                .get(1..)
                .is_some_and(|rest| rest.contains('#'))
    }
}

/// What reading a script found.
#[derive(Debug)]
pub struct Read {
    /// Its imports, in the order they are written.
    pub imports: Vec<Import>,
    /// How many calls of `import(…)` or `require(…)` have an argument that cannot be followed
    /// (spec IMP-2).
    pub computed: usize,
}

/// Why a script did not parse: the line of the first syntax error, where there is one, and its message.
#[derive(Debug)]
pub struct Broken {
    /// The line, counted from one.
    pub line: Option<usize>,
    /// What the parser said.
    pub message: String,
}

/// Parses a script, as a module first and then as a script, and finds its imports (spec PARSE-1).
pub fn read(path: &str, text: &str) -> Result<Read, Broken> {
    let module = source_type(path).with_module(true);

    match parse(text, module) {
        Ok(read) => Ok(read),
        Err(broken) => parse(text, module.with_script(true)).map_err(|_| broken),
    }
}

/// The language of a script, from its name. A type declaration file is TypeScript, and JavaScript allows
/// JSX whatever its ending.
fn source_type(path: &str) -> SourceType {
    let typescript = [".ts", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending));
    let declaration = [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|ending| path.ends_with(ending));

    if path.ends_with(".tsx") {
        SourceType::tsx()
    } else if typescript {
        SourceType::ts().with_typescript_definition(declaration)
    } else {
        SourceType::jsx()
    }
}

/// Parses once with the source type given.
fn parse(text: &str, source_type: SourceType) -> Result<Read, Broken> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
    let lines = Lines::of(text);

    if let Some(error) = parsed.diagnostics.iter().next() {
        let line = error.labels.first().map(|label| lines.at(label.offset()));

        return Err(Broken {
            line,
            message: error.message.to_string(),
        });
    }

    if parsed.fatal_error {
        return Err(Broken {
            line: None,
            message: "the parser stopped".to_owned(),
        });
    }

    let mut finder = Finder {
        lines: &lines,
        imports: Vec::new(),
        computed: 0,
    };

    finder.visit_program(&parsed.program);

    Ok(Read {
        imports: finder.imports,
        computed: finder.computed,
    })
}

/// Where each line of a text begins, to turn an offset into a line.
struct Lines(Vec<usize>);

impl Lines {
    /// The starts of the lines of a text.
    fn of(text: &str) -> Self {
        Self(
            std::iter::once(0)
                .chain(text.match_indices('\n').map(|(at, _)| at + 1))
                .collect(),
        )
    }

    /// The line an offset lies on, counted from one.
    fn at(&self, offset: u32) -> usize {
        let offset = usize::try_from(offset).unwrap_or(usize::MAX);

        self.0.partition_point(|start| *start <= offset)
    }
}

/// Walks a program and collects its imports.
struct Finder<'l> {
    lines: &'l Lines,
    imports: Vec<Import>,
    computed: usize,
}

impl Finder<'_> {
    /// Records one import.
    fn found(&mut self, specifier: String, form: Form, type_only: bool, span: Span) {
        self.imports.push(Import {
            specifier,
            form,
            type_only,
            line: self.lines.at(span.start),
        });
    }

    /// The text of a string literal, as JavaScript reads it.
    fn text(literal: &StringLiteral<'_>) -> String {
        literal.value.display().to_string()
    }

    /// The text of an argument that is a string or a template with no substitution, or nothing.
    fn literal(expression: &Expression<'_>) -> Option<String> {
        match expression {
            Expression::StringLiteral(literal) => Some(Self::text(literal)),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
                .quasis
                .first()
                .map(|quasi| match quasi.value.cooked {
                    Some(cooked) => cooked.display().to_string(),
                    None => quasi.value.raw.to_string(),
                }),
            _ => None,
        }
    }
}

impl<'a> Visit<'a> for Finder<'_> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        self.found(
            Self::text(&declaration.source),
            Form::Import,
            declaration.import_kind.is_type(),
            declaration.span,
        );
        walk::walk_import_declaration(self, declaration);
    }

    fn visit_export_from_declaration(&mut self, declaration: &ExportFromDeclaration<'a>) {
        self.found(
            Self::text(&declaration.source),
            Form::Export,
            declaration.export_kind.is_type(),
            declaration.span,
        );
        walk::walk_export_from_declaration(self, declaration);
    }

    fn visit_export_all_declaration(&mut self, declaration: &ExportAllDeclaration<'a>) {
        self.found(
            Self::text(&declaration.source),
            Form::ExportAll,
            declaration.export_kind.is_type(),
            declaration.span,
        );
        walk::walk_export_all_declaration(self, declaration);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        if let TSModuleReference::ExternalModuleReference(reference) = &declaration.module_reference
        {
            self.found(
                Self::text(&reference.expression),
                Form::ImportEquals,
                declaration.import_kind.is_type(),
                declaration.span,
            );
        }

        walk::walk_ts_import_equals_declaration(self, declaration);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        match Self::literal(&expression.source) {
            Some(specifier) => self.found(specifier, Form::Dynamic, false, expression.span),
            None => self.computed += 1,
        }

        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && callee.name == "require"
        {
            match call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(Self::literal)
            {
                Some(specifier) => self.found(specifier, Form::Require, false, call.span),
                None => self.computed += 1,
            }
        }

        walk::walk_call_expression(self, call);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        self.found(
            Self::text(&import.source),
            Form::TypeImport,
            true,
            import.span,
        );
        walk::walk_ts_import_type(self, import);
    }
}
```

```rust path=src/scripts.rs
//! Reading every script of the file list: its content from disk, parsed, and its imports found (spec
//! PARSE-1, PARSE-2, PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::files::{File, Kind};
use crate::imports::{self, Broken, Read};
use crate::notice::Notice;
use crate::stop::Stop;

/// What reading the scripts found.
#[derive(Debug, Default)]
pub struct Scripts {
    /// What each script that was read holds, by its path.
    pub read: BTreeMap<String, Read>,
    /// The listed files that were gone by the time they were read, which leave the list (spec LIST-3).
    pub gone: BTreeSet<String>,
}

/// Why a file was not read.
enum Unread {
    /// It is gone from disk since git listed it.
    Gone,
    /// It cannot be read, does not hold UTF-8, or does not parse; the text says which.
    Broken(String),
}

/// Reads every script and type declaration file in the list that is not data, in the order of the list.
/// A tracked file that cannot be read stops the run; an untracked one is skipped with a notice (spec
/// PARSE-2).
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
) -> Result<Scripts, Stop> {
    let mut scripts = Scripts::default();

    for file in files {
        let script = matches!(file.kind, Kind::Script | Kind::Declaration);

        if !script || data.contains(&file.path) {
            continue;
        }

        match one(root, &file.path) {
            Ok(read) => {
                scripts.read.insert(file.path.clone(), read);
            }
            Err(Unread::Gone) => {
                notices.push(Notice::new(
                    "file-gone",
                    Some(file.path.clone()),
                    "The file was gone by the time it was read, so it is left out of the file list.".to_owned(),
                ));
                scripts.gone.insert(file.path.clone());
            }
            Err(Unread::Broken(why)) if file.tracked => {
                return Err(Stop::new(format!(
                    "the file {} {why}; fix it, or declare it as data if it is broken on purpose",
                    file.path
                )));
            }
            Err(Unread::Broken(why)) => notices.push(Notice::new(
                "untracked-file-skipped",
                Some(file.path.clone()),
                format!("The untracked file {why}, so its imports are not read."),
            )),
        }
    }

    Ok(scripts)
}

/// Reads and parses one script.
fn one(root: &Path, path: &str) -> Result<Read, Unread> {
    let bytes = fs::read(root.join(path)).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Unread::Gone,
        _ => Unread::Broken(format!("cannot be read ({error})")),
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let text =
        std::str::from_utf8(bytes).map_err(|_| Unread::Broken("is not valid UTF-8".to_owned()))?;

    imports::read(path, text).map_err(|Broken { line, message }| {
        Unread::Broken(match line {
            Some(line) => format!("does not parse (line {line}: {message})"),
            None => format!("does not parse ({message})"),
        })
    })
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, and the imports of every script. Where each import leads comes with the part that resolves
//! them.

use std::env;
use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::Import;
use crate::marks::{self, Marks};
use crate::notice::{self, Notice};
use crate::packages::{self, Package, Packages};
use crate::scripts::{self, Scripts};
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
    let scripts = scripts::read(&root, &files, &marks.data, &mut notices)?;
    let document = json!({
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    Ok(format!("{document:#}"))
}

/// One file as it is printed. A file that was read has its imports and the count of those that cannot
/// be followed.
fn file(file: &File, marks: &Marks, packages: &Packages, scripts: &Scripts) -> Value {
    let mut printed = json!({
        "data": marks.data.contains(&file.path),
        "generated": marks.generated.contains(&file.path),
        "kind": file.kind.name(),
        "package": packages.of_file.get(&file.path),
        "path": file.path,
        "tracked": file.tracked,
    });

    if let Some(read) = scripts.read.get(&file.path) {
        printed["computedImports"] = json!(read.computed);
        printed["imports"] = read.imports.iter().map(import).collect();
    }

    printed
}

/// One import as it is printed (spec OUT-1).
fn import(import: &Import) -> Value {
    json!({
        "form": import.form.name(),
        "line": import.line,
        "query": import.has_query(),
        "specifier": import.specifier,
        "typeOnly": import.type_only,
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
//! files, its packages and its scripts' imports. It reports no findings yet.

mod cli;
mod declarations;
mod files;
mod git;
mod glob;
mod graph;
mod hand_off;
mod imports;
mod marks;
mod notice;
mod packages;
mod scripts;
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
cargo build --release --locked && node scripts/check-floor.mjs
```

Expected: each exits 0. The tests report 7 passed in the binary's own tests, 6 for `cli`, 5 for
`declarations`, 16 for `file-list`, 10 for `hand-off`, 2 for `hand-off-schema`, 11 for `imports`
and 9 for `packages`; the last line says what the binary asks of the system, which is no more than
the floor allows.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src tests
git commit -m "feat(imports): parse scripts and find their imports" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: one commit on `feat/reading-3-imports`.

### Task 2: the oxc crates move together

**Implements:** DEP-3

**Test:** `node --test scripts/dependabot.test.mjs`

**Files:**
- Modify: `.github/dependabot.yml`
- Test: `scripts/dependabot.test.mjs`

**Interfaces:**
- Consumes: Task 1's oxc pins in `Cargo.toml`; `ROOT` from `scripts/lib/run.mjs`.
- Produces: one Dependabot pull request for all the oxc crates.

- [ ] **Step 1: Write the failing test**

```js path=scripts/dependabot.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** A text file of the repository, with line feeds whatever the checkout wrote. */
const read = (name) => readFileSync(join(ROOT, name), 'utf8').replaceAll('\r\n', '\n');

test('the oxc crates are pinned to one version, since they are released together (spec AN-2, DEP-3)', () => {
  const pins = [...read('Cargo.toml').matchAll(/^(?<crate>oxc_[a-z_]+) = "=(?<version>[^"]+)"$/gmu)];

  assert.ok(pins.length > 0);
  assert.equal(new Set(pins.map((pin) => pin.groups.version)).size, 1);
});

test('Dependabot moves the oxc crates together, in one pull request (spec DEP-3)', () => {
  const dependabot = read('.github/dependabot.yml');

  assert.match(
    dependabot,
    /- package-ecosystem: cargo\n(?: {4}.*\n)* {4}groups:\n {6}oxc:\n {8}patterns:\n {10}- "oxc_\*"\n/u,
  );
});
```

- [ ] **Step 2: Run it and see it fail**

```bash
node --test scripts/dependabot.test.mjs
```

Expected: the test of the group fails, because the settings name no group; the test of the pins passes.

- [ ] **Step 3: Put the oxc crates in one group**

```yaml path=.github/dependabot.yml
# Pull requests that move each pin, opened without anyone asking (spec PUB-7). CI decides whether one can
# merge; none merges by itself. The npm that publishes is not here: it is moved by hand (spec CI-5).
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    # The oxc crates are released together, and one moved without the others does not build (spec
    # DEP-3).
    groups:
      oxc:
        patterns:
          - "oxc_*"
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: rust-toolchain
    directory: /
    schedule:
      interval: weekly
```

- [ ] **Step 4: Run every gate**

```bash
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0, and the Node tests report 47 passed.

- [ ] **Step 5: Commit, push, and open the pull request as a draft on the second plan's**

```bash
git add .github/dependabot.yml scripts/dependabot.test.mjs
git commit -m "ci: move the oxc crates together" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -u origin feat/reading-3-imports
gh pr create --draft --base feat/reading-2-file-list --head feat/reading-3-imports --title "feat: parse scripts and find their imports" --body "A draft, opened so that CI runs on all five targets. Not for merging until the owner has validated the plan."
gh pr checks --watch
```

Expected: the pull request's seven checks pass. Its description is written out in full when the plan
is handed over for validation.
