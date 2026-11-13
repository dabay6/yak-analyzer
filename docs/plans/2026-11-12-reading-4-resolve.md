# Reading a project, part 4: where each import leads: implementation plan

**Goal:** `graph` prints, for every import it found, the one outcome RES-1 gives it, with the files it
leads to, the place outside it leads to, or the ignored path, and the step of RES-2 that decided:
through relative paths, packages of the project by name, `#` names, and tsconfig files' `paths` and
`baseUrl`, with build output mapped back to its source.

**Architecture:** `src/resolve.rs` takes each import of `scripts::Scripts` through RES-2's order
against the file list, never the disk, and asks git once, for every path that led to no file, which of
them it ignores. `src/packages.rs` keeps the `package.json` fields an import by name reads.
`src/tsconfig.rs` reads every tsconfig and hands `oxc_resolver` a file system of its own that serves
only the configs it read, so the library finds the config that governs a file; the analyzer matches
`paths` itself and maps build output back with each config's `outDir` and `rootDir`.

**Spec:** docs/specs/2026-11-11-reading-a-project-design.md

**Implements:** RES-1 to RES-6, RES-8 to RES-10, MAP-1 to MAP-4, CFG-1 to CFG-4, PACK-3, PACK-5, PACK-6, LIST-3, LIST-4, LIST-5, LIST-8, LIST-9, RUN-4, RUN-5, DEP-1, DEP-3, FIX-2, FIX-3, FIX-4, FIX-6

This is the fourth of the item's plans. It is worked on its own branch, `feat/reading-4-resolve`, made
from the third plan's branch, and its pull request is a draft stacked on the third plan's. HTML pages
(RES-7), Vue components and MDX documents come in the fifth plan.

## Global constraints

- Whether a path exists is asked of the file list, never of the file system; the one file read
  outside it is a tsconfig above the root that one under it extends, after git says it is a file of
  the work tree that git does not ignore (LIST-4, LIST-9).
- A path matches only when its letters' case matches, on every platform (RES-10).
- Nothing above the root is part of the project: every upward search ends at the root, and an import
  that leads above it is outside (LIST-8).
- The output is the same with and without `node_modules` and ignored build output on disk (design
  AN-5, FIX-3).
- RUN-4's order holds: every `package.json` is read (8), then the workspace's list (9), then the
  tsconfig files a file depends on (10), then every field's form, the first by path across
  `package.json` files, tsconfig files and the workspace's list (11), then the scripts (12).
- Every crate added is pinned with `=` (DEP-1); the oxc crates that are released together still move
  together (DEP-3).
- The analyzer runs no project code, writes nothing under the root and uses no network (RUN-6).

## The decisions this plan makes

1. **Finding a file for a path stays in the analyzer's own code** (RES-9). The first plan's trial
   found that the library gives neither RES-3's pair of a script and its declaration file nor a
   folder's `types` and `module`, and RES-4 needs every path tried for an import that leads to no
   file, which the library does not return. RES-9 lets a rule the library does not give as written
   move to the analyzer's own code; the rule and its fixtures are unchanged. The library keeps the
   other half RES-9 gives it: finding the tsconfig that governs a file, through `references` and
   `extends`, with its `paths` and `baseUrl`.
2. **Two crates are added in Task 3:** `oxc_resolver` `=11.24.3`, the version the trial used, for the
   tsconfig that governs a file; and `json-strip-comments` `=3.1.2`, the version `oxc_resolver` itself
   uses, to read a tsconfig that holds comments and trailing commas (CFG-1). `oxc_resolver` is
   released on its own, with versions of its own, and needs no other oxc crate, so it is left out of
   Dependabot's `oxc` group and moves alone; the test of the pins leaves it out of the set that shares
   one version.
3. **The library is given a file system that serves plain JSON:** for each config a file depends on,
   only `extends`, `files`, `include`, `exclude`, `references` and `compilerOptions.paths` and
   `baseUrl`, with every target of `extends` and `references` already found by the analyzer and
   written as a path from the config's folder. So the library reads no file outside the list, looks
   in no `node_modules`, and meets no field whose form the analyzer has not checked. A
   `jsconfig.json` is served as the `tsconfig.json` of a folder that has none (CFG-2). A config the
   library finds above the root is not used: the search ends at the root.
4. **The modules built into Node** are the 54 names `require('module').builtinModules` lists in Node
   22.19.0, those beginning with `_` left out, and any specifier that begins with `node:`.
5. **What `graph` prints for an import:** `rule`, one of `relative`, `paths`, `imports`, `base-url`,
   `package` and `outside`; `outcome`, one of `files`, `outside`, `ignored` and `unresolved`; with
   `files`, the paths; with `outside`, the place, one of `package`, `builtin`, `above-root`,
   `other-repository`, `link` and `elsewhere`, and for a package `package` and `listed`; with
   `ignored`, the path, and `from`, the target of a `package.json` it came from, where it came from
   one (MAP-4). The printed document's keys are sorted, since `oxc_resolver` turns on `serde_json`'s
   `preserve_order`.
6. **What git is asked about an import that leads to no file:** the path as written first, then each
   source form RES-3 and MAP-1 tried (the paths ending in `.ts`, `.tsx`, `.mts`, `.cts` or a type
   declaration's ending), in one start of `git check-ignore` for all the imports of a run. The first
   path git ignores is the one printed.
7. **Absolute paths and URLs:** a specifier that begins with `/` or `\`, or with a drive letter and a
   colon, is an absolute path; one that begins with a scheme of two characters or more and a colon is
   a URL. Both are outside, as loaded from elsewhere, and neither is mapped by `paths` or `baseUrl`.
8. **Targets of `exports` and `imports`.** Keys are matched as Node matches them, a key with no `*`
   first, then the pattern with the longest part before its `*`, the first written where two are as
   long. Every condition and every entry of a list is followed, in the order written, and the import
   leads to every file found. A target of `null` leads nowhere. A target in `imports` that does not
   begin with `./` is a package's name and is resolved by RES-2's steps 5 and 6; a target in `exports`
   that does not begin with `./` leads nowhere, as Node refuses one.
9. **Which package a name leads to** (RES-2, step 5): the importing file's own package by its own
   name; else the first workspace package by path with the name; else, where the nearest
   `package.json` at or above the importing file that lists the name gives it a version beginning with
   `workspace:`, the first package under the root by path with the name, and with `file:`, `link:` or
   `portal:`, the package in the folder it names. A version with any of them that leads to no package
   of the project leaves the import outside, as a listed package.
10. **MAP-1's second step** (the first folder replaced by `src`) is taken for a target of a
    `package.json` and a subpath of a package with no `exports`, and not for a relative specifier or a
    `paths` target: for those, only a config's `outDir` says that a folder is a build folder, which is
    what MAP-1 asks of "a relative specifier that reaches into a build folder". MAP-1's first step
    looks in the folder of the nearest package at or above the path, and in the root's when there is
    none.
11. **A folder that holds a `package.json`, reached by a path,** leads to every file its `types`,
    `module` and `main` lead to; when they lead to none, its `index` file is looked for as usual, and
    git is asked about the paths of both.
12. **`extends` that is not a relative path** names a config in a workspace package by the package's
    name and a path inside it, `tsconfig.json` where it names none, with `.json` added where the path
    lacks it; the package's `exports` are not read for it. Any other is left out, with a notice: a
    package outside the project, a `#` name, an absolute path, a target above the top of the work
    tree, a file declared as data, and one git ignores.
13. **A config no file depends on** (CFG-4) is read for MAP-1, and when it cannot be read, or its
    `extends` names a file that is not there, it is left out with a notice instead of stopping the
    run. Every config that was read has its fields' forms checked (RUN-4, step 11).
14. **RUN-4's steps 10 and 11** come in order because `packages::read` becomes two halves:
    `packages::load` reads the files, and `Loaded::checked` checks the fields after the tsconfig files
    are read. A fault of step 11 carries its file, so the first by path is reported across
    `package.json` files, tsconfig files and the workspace's list.
15. **The files the analyzer reads,** for CFG-4, are its scripts and type declaration files that are
    not data; the fifth plan adds Vue components, MDX documents and HTML pages as it reads them.
16. **A `paths` target above the root** is outside, as above the root, when it is reached in the order
    of the targets before any target leads to a file.
17. **The notice kinds this plan adds:** `tsconfig-skipped`, `tsconfig-extends-left-out` and
    `tsconfig-reference-missing`.

## Review focus

- **A circle of `extends`** reaches the library, which refuses it; the run stops with the library's
  words in the line. No fixture makes one. A reviewer reads `Configs::governing`.
- **A `${configDir}` in `outDir` or `rootDir`** is not filled in by the analyzer's own reading for
  MAP-1, though the library fills it in for `paths` and `baseUrl`. No fixture uses one.
- **A root given as a Windows verbatim path** (`\\?\C:\…`) is compared with the library's paths
  component by component; the fixtures give ordinary paths only.
- **TypeScript's reading of `include` and `exclude`** is the library's; the fixtures check the cases
  the spec names, and the trial checked the rest.
- **`extends` through a package's `exports`** is not followed (decision 12); the live check in the
  sixth plan shows whether a named repository needs it.
- **The time of a run:** yakshaver's checkout took about a second with every import resolved. LIVE-2
  weighs it on the named repositories.

### Task 1: relative imports, and every place outside

**Implements:** RES-1, RES-2, RES-3, RES-4, RES-8, RES-10, LIST-3, LIST-5, LIST-8, PACK-6, FIX-2, FIX-3, FIX-4, FIX-6

**Test:** `cargo test --release --locked --test resolve --test imports`

**Files:**
- Create: `src/resolve.rs`
- Modify: `src/files.rs`, `src/git.rs`, `src/packages.rs`, `src/graph.rs`, `src/main.rs`, `Cargo.toml`
- Test: `tests/resolve.test.rs`, `tests/imports.test.rs`

**Interfaces:**
- Consumes: the third plan's `scripts::Scripts`, `imports::Import`, `files::list` and
  `tests/support/project.rs`.
- Produces: `files::Listing` (the files, the folders of nested repositories, other worktrees and
  submodules, and the links), `git::ignored`, `packages::Package::dependencies`, and
  `resolve::Resolver`, `Resolution`, `Outcome`, `Place` and `Rule`, which Task 2 extends.

- [ ] **Step 1: Write the failing tests**

The imports tests now compare each import's own fields only, since where it leads is the resolve
tests' to check.

```rust path=tests/resolve.test.rs
//! Runs the built binary on made-up projects and checks where relative imports lead, and every kind of
//! outside (spec RES-1 to RES-4, RES-8, RES-10, LIST-5, LIST-8, PACK-6, FIX-2, FIX-3, FIX-4).

#[path = "support/project.rs"]
mod project;

use std::fs;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The import of a file, by its specifier, as `graph` prints it.
fn import<'a>(document: &'a Value, path: &str, specifier: &str) -> &'a Value {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .and_then(|file| file["imports"].as_array())
        .and_then(|imports| {
            imports
                .iter()
                .find(|import| import["specifier"] == json!(specifier))
        })
        .unwrap_or_else(|| panic!("{path} imports {specifier}"))
}

/// What an import leads to: its outcome, and the files, the place or the ignored path that go with it.
fn led(document: &Value, path: &str, specifier: &str) -> Value {
    let import = import(document, path, specifier);
    let mut led = json!({ "outcome": import["outcome"], "rule": import["rule"] });

    for key in ["files", "outside", "package", "listed", "ignored"] {
        if let Some(value) = import.get(key) {
            led[key] = value.clone();
        }
    }

    led
}

/// An import that leads to the files given, by a relative specifier.
fn files(found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": "relative", "files": found })
}

#[test]
fn finds_a_file_for_each_way_a_relative_path_is_written() {
    let project = Project::new("resolve-relative");

    project
        .write(
            "src/main.ts",
            "import './a.js';\nimport './b.ts';\nimport './types';\nimport './dir';\nimport './c.mjs';\n\
             import '../lib/d';\nimport './styles.css';\nimport './notes.md?raw';\nimport '.';\n",
        )
        .write("src/a.ts", "")
        .write("src/b.ts", "")
        .write("src/types.d.ts", "")
        .write("src/dir/index.tsx", "")
        .write("src/c.mts", "")
        .write("lib/d.js", "")
        .write("src/styles.css", "")
        .write("src/notes.md", "")
        .write("src/index.ts", "")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(main("./a.js"), files(&["src/a.ts"]));
    assert_eq!(main("./b.ts"), files(&["src/b.ts"]));
    assert_eq!(main("./types"), files(&["src/types.d.ts"]));
    assert_eq!(main("./dir"), files(&["src/dir/index.tsx"]));
    assert_eq!(main("./c.mjs"), files(&["src/c.mts"]));
    assert_eq!(main("../lib/d"), files(&["lib/d.js"]));
    assert_eq!(main("./styles.css"), files(&["src/styles.css"]));
    assert_eq!(main("./notes.md?raw"), files(&["src/notes.md"]));
    assert_eq!(main("."), files(&["src/index.ts"]));
}

#[test]
fn leads_to_both_a_script_and_the_declaration_file_beside_it() {
    let project = Project::new("resolve-pair");

    project
        .write(
            "src/main.ts",
            "import './plain.js';\nimport './plain';\nimport './typed.d.ts';\n",
        )
        .write("src/plain.js", "")
        .write("src/plain.d.ts", "")
        .write("src/typed.mjs", "")
        .write("src/typed.d.mts", "")
        .write("src/typed.d.ts", "")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(
        main("./plain.js"),
        files(&["src/plain.d.ts", "src/plain.js"])
    );
    assert_eq!(main("./plain"), files(&["src/plain.d.ts", "src/plain.js"]));
    assert_eq!(main("./typed.d.ts"), files(&["src/typed.d.ts"]));
}

#[test]
fn tells_an_ignored_path_from_a_missing_one() {
    let project = Project::new("resolve-ignored");

    project
        .write(".gitignore", "src/generated/\n*.js\n")
        .write("src/types/.gitignore", "*.d.ts\n")
        .write(
            "src/main.ts",
            "import './generated/api';\nimport './fooo';\nimport './fooo.js';\nimport './types/built';\n\
             import './missing';\nimport './Main';\n",
        )
        .commit();

    let run = |label: &str| {
        let document = project.graph();
        let main = |specifier: &str| led(&document, "src/main.ts", specifier);

        assert_eq!(
            main("./generated/api"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/generated/api" }),
            "{label}"
        );
        // A mistyped import with no extension stays unresolved though every `.js` file is ignored.
        assert_eq!(
            main("./fooo"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
        // Written with `.js`, the path itself is ignored, so the import is (spec RES-4).
        assert_eq!(
            main("./fooo.js"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/fooo.js" }),
            "{label}"
        );
        // An import with no extension of a file git ignores by its ending.
        assert_eq!(
            main("./types/built"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/types/built.d.ts" }),
            "{label}"
        );
        assert_eq!(
            main("./missing"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
        // Letter case counts on every platform (spec RES-10).
        assert_eq!(
            main("./Main"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
    };

    // The same with and without the ignored files on disk (spec AN-5, FIX-3).
    run("without the ignored files");
    project
        .write("src/generated/api.ts", "")
        .write("src/fooo.js", "")
        .write("src/types/built.d.ts", "");
    run("with the ignored files");
}

#[test]
fn names_each_place_outside_the_project() {
    let other = Project::new("resolve-outside-other");

    other.write("other.ts", "").commit();

    let project = Project::new("resolve-outside");

    project
        .write("package.json", r#"{ "dependencies": { "react": "^19.0.0" } }"#)
        .write(
            "src/main.ts",
            "import 'node:fs';\nimport 'fs/promises';\nimport 'react';\nimport 'react-dom/client';\n\
             import '@scope/pkg/deep';\nimport '/abs/file.js';\nimport 'https://cdn.example/x.js';\n\
             import 'C:/drive/file.js';\nimport '../nested/inner';\nimport '../sub/other';\nimport './link-to-main';\n\
             import './linked-folder/inner';\n",
        )
        .commit()
        .write("nested/inner.ts", "");
    project.git(&["-C", "nested", "init", "--quiet"]);
    project.git(&[
        "submodule",
        "--quiet",
        "add",
        other.root.to_str().expect("valid UTF-8"),
        "sub",
    ]);
    project.git(&["commit", "--quiet", "--message", "a submodule"]);

    let source = project.home.join("link-content");

    fs::write(&source, "src/main.ts").expect("the content can be written");

    let blob = project.git(&["hash-object", "-w", source.to_str().expect("valid UTF-8")]);

    fs::write(&source, "../lib").expect("the content can be written");

    let folder = project.git(&["hash-object", "-w", source.to_str().expect("valid UTF-8")]);

    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("120000,{},src/link-to-main", blob.trim()),
    ]);
    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("120000,{},src/linked-folder", folder.trim()),
    ]);
    project.git(&["commit", "--quiet", "--message", "two links"]);
    project.git(&["checkout", "--", "src/link-to-main", "src/linked-folder"]);

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);
    let place =
        |rule: &str, place: &str| json!({ "outcome": "outside", "rule": rule, "outside": place });
    let package = |name: &str, listed: bool| json!({ "outcome": "outside", "rule": "outside", "outside": "package", "package": name, "listed": listed });

    assert_eq!(main("node:fs"), place("outside", "builtin"));
    assert_eq!(main("fs/promises"), place("outside", "builtin"));
    assert_eq!(main("react"), package("react", true));
    assert_eq!(main("react-dom/client"), package("react-dom", false));
    assert_eq!(main("@scope/pkg/deep"), package("@scope/pkg", false));
    assert_eq!(main("/abs/file.js"), place("outside", "elsewhere"));
    assert_eq!(
        main("https://cdn.example/x.js"),
        place("outside", "elsewhere")
    );
    assert_eq!(main("C:/drive/file.js"), place("outside", "elsewhere"));
    assert_eq!(
        main("../nested/inner"),
        place("relative", "other-repository")
    );
    assert_eq!(main("../sub/other"), place("relative", "other-repository"));
    assert_eq!(main("./link-to-main"), place("relative", "link"));
    // A path through a linked folder is through a link (spec LIST-3).
    assert_eq!(main("./linked-folder/inner"), place("relative", "link"));
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
}

#[test]
fn leads_a_path_above_a_sub_folder_root_outside() {
    let project = Project::new("resolve-above");

    project
        .write("shared/util.ts", "")
        .write(
            "apps/web/src/main.ts",
            "import '../../../shared/util';\nimport '../lib';\n",
        )
        .write("apps/web/lib/index.ts", "")
        .commit();

    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(paths(&document), ["lib/index.ts", "src/main.ts"]);
    assert_eq!(
        led(&document, "src/main.ts", "../../../shared/util"),
        json!({ "outcome": "outside", "rule": "relative", "outside": "above-root" })
    );
    assert_eq!(
        led(&document, "src/main.ts", "../lib"),
        files(&["lib/index.ts"])
    );
}

#[test]
fn stops_on_a_dependency_list_of_a_form_it_does_not_have() {
    let project = Project::new("resolve-dependencies-form");

    project
        .write("package.json", r#"{ "devDependencies": ["typescript"] }"#)
        .commit();

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `devDependencies` in package.json holds a value of a form it does not have; \
         it must be an object that maps each package's name to the version it asks for; fix it"
    );
}
```

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

/// The imports of a file as `graph` prints them, with only each import's own fields: where an import
/// leads is the resolve tests' to check.
fn own(imports: &Value) -> Value {
    imports
        .as_array()
        .expect("a list of imports")
        .iter()
        .map(|import| {
            json!({
                "form": import["form"],
                "line": import["line"],
                "query": import["query"],
                "specifier": import["specifier"],
                "typeOnly": import["typeOnly"],
            })
        })
        .collect()
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
        own(&main["imports"]),
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
        own(&file(&document, "legacy/old.js")["imports"]),
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

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test resolve --test imports
```

Expected: cargo refuses, because the manifest names no test called `resolve` yet.

- [ ] **Step 3: Name the test**

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

[[test]]
name = "resolve"
path = "tests/resolve.test.rs"

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

- [ ] **Step 4: Write the file list's other places, git's question, the dependency lists and the resolver**

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

/// The file list, and the paths it leaves out that an import can still lead into.
#[derive(Debug, Default)]
pub struct Listing {
    /// The files, sorted by path, byte by byte.
    pub files: Vec<File>,
    /// The folders of nested repositories, other worktrees and submodules, which are never entered
    /// (spec LIST-3).
    pub elsewhere: BTreeSet<String>,
    /// The paths of symbolic links, which are never followed (spec LIST-3).
    pub links: BTreeSet<String>,
}

/// Asks git for the file list of the root. What was left out on the way is added to the notices.
pub fn list(root: &Path, notices: &mut Vec<Notice>) -> Result<Listing, Stop> {
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
    let mut elsewhere: BTreeSet<String> = BTreeSet::new();
    let mut link_paths: BTreeSet<String> = BTreeSet::new();
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
            match name.strip_suffix('/') {
                Some(folder) => {
                    elsewhere.insert(folder.to_owned());
                }
                None => {
                    untracked.insert(name.to_owned());
                }
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
            link_paths.insert(name.to_owned());
            tracked.insert(name.to_owned(), Entry::Link);
        } else {
            if mode == SUBMODULE_MODE {
                elsewhere.insert(name.to_owned());
            }

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

        if link {
            link_paths.insert(name.clone());
        }

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

    if !link_paths.is_empty() {
        let links = link_paths.len();

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

    Ok(Listing {
        files,
        elsewhere,
        links: link_paths,
    })
}
```

```rust path=src/git.rs
//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

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
    run(root, command, None)
}

/// Asks git which of the paths given, relative to the root, it ignores (spec LIST-5). It answers from
/// its patterns, whether or not a path is on disk. A path inside a submodule must not be given: git
/// stops on one.
pub fn ignored(root: &Path, paths: &[&str]) -> Result<Vec<String>, Stop> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let input: Vec<u8> = paths
        .iter()
        .flat_map(|path| path.bytes().chain([0]))
        .collect();
    let answer = run(
        root,
        &[
            "check-ignore",
            "-z",
            "--stdin",
            "--verbose",
            "--non-matching",
        ],
        Some(&input),
    )?;
    let fields: Vec<&[u8]> = answer.output.split(|byte| *byte == 0).collect();

    // Each path comes back as four fields: the file of the pattern that decided, its line, the pattern,
    // and the path. A path no pattern matched has the first three empty; a pattern that begins with `!`
    // says the path is not ignored.
    Ok(fields
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|answer| !answer[0].is_empty() && !answer[2].starts_with(b"!"))
        .map(|answer| String::from_utf8_lossy(answer[3]).into_owned())
        .collect())
}

/// Runs one git command in the root, with what it reads on standard input where there is any. With
/// input, exit code 1 is an answer too: `check-ignore` gives it when no path is ignored.
fn run(root: &Path, command: &[&str], input: Option<&[u8]>) -> Result<Answer, Stop> {
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
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    let (oldest_major, oldest_minor) = OLDEST;
    let started = |error: std::io::Error| {
        Stop::new(format!(
            "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
             {oldest_minor} or newer, which must be on the PATH"
        ))
    };
    let mut child = git.spawn().map_err(started)?;

    // The input is written from a thread of its own, so that git never waits on a full output pipe
    // while the analyzer waits to finish writing.
    let writer = match (input, child.stdin.take()) {
        (Some(input), Some(mut stdin)) => {
            let input = input.to_vec();

            Some(thread::spawn(move || stdin.write_all(&input)))
        }
        _ => None,
    };
    let ran = child.wait_with_output().map_err(started)?;

    // git stops reading when it fails, which breaks the pipe; its exit code says what happened.
    if let Some(writer) = writer {
        let _ = writer.join();
    }

    let answered = ran.status.success() || (input.is_some() && ran.status.code() == Some(1));
    let said = String::from_utf8_lossy(&ran.stderr);
    let first = said.lines().find(|line| !line.trim().is_empty());

    if answered {
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
    /// The packages its four lists of dependencies name, each with the version it asks for.
    pub dependencies: BTreeMap<String, String>,
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
    let mut names: BTreeMap<&str, (Option<String>, BTreeMap<String, String>)> = BTreeMap::new();

    for (folder, fields) in &read {
        let file = manifest_path(folder);

        names.insert(folder, (name(fields, &file)?, dependencies(fields, &file)?));
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
        .map(|(folder, (name, dependencies))| {
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
                dependencies,
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

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();

    for list in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let form = "an object that maps each package's name to the version it asks for";
        let entries = match fields.get(list) {
            None => continue,
            Some(Value::Object(entries)) => entries,
            Some(_) => return Err(misshapen(list, file, form)),
        };

        for (package, version) in entries {
            let version = version
                .as_str()
                .ok_or_else(|| misshapen(list, file, form))?;

            named
                .entry(package.clone())
                .or_insert_with(|| version.to_owned());
        }
    }

    Ok(named)
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

```rust path=src/resolve.rs
//! Where each import leads (spec RES-1 to RES-4, RES-8, RES-10, LIST-5).
//!
//! A path is looked for in the file list, never on disk, with exact letter case. What leads to no file
//! is asked about once, with one start of git for all of them, to tell an ignored path from a missing
//! one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::files::Listing;
use crate::git;
use crate::imports::Import;
use crate::packages::Packages;
use crate::scripts::Scripts;
use crate::stop::Stop;

/// The modules built into Node, as `require('module').builtinModules` lists them in Node 22.19.0, those
/// whose names begin with `_` left out. Any specifier that begins with `node:` is built in too.
const BUILTINS: [&str; 54] = [
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// The endings a script's path may be given, in the order they are tried: TypeScript before JavaScript,
/// then a type declaration file (spec RES-3).
const ENDINGS: [&str; 9] = [
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".d.ts",
];

/// A script's output ending, and the source forms it is looked for under first (spec RES-3).
const SOURCE_FORMS: [(&str, &[&str]); 4] = [
    (".js", &[".ts", ".tsx", ".d.ts"]),
    (".jsx", &[".tsx"]),
    (".mjs", &[".mts", ".d.mts"]),
    (".cjs", &[".cts", ".d.cts"]),
];

/// A script and the type declaration file that describes it, by their endings (spec RES-3).
const PAIRS: [(&str, &str); 3] = [(".js", ".d.ts"), (".mjs", ".d.mts"), (".cjs", ".d.cts")];

/// Where outside the project an import leads (spec RES-1, RES-8).
#[derive(Debug, PartialEq, Eq)]
pub enum Place {
    /// A package that is not in the project, with whether a `package.json` at or above the importing
    /// file lists it.
    Package { name: String, listed: bool },
    /// A module built into Node.
    Builtin,
    /// A path above the root.
    AboveRoot,
    /// A path inside a nested repository, a submodule or another worktree.
    OtherRepository,
    /// A symbolic link, or a path through one.
    Link,
    /// Something loaded from elsewhere: an absolute path or a URL.
    Elsewhere,
}

impl Place {
    /// The place's name in what `graph` prints.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Package { .. } => "package",
            Self::Builtin => "builtin",
            Self::AboveRoot => "above-root",
            Self::OtherRepository => "other-repository",
            Self::Link => "link",
            Self::Elsewhere => "elsewhere",
        }
    }
}

/// The one outcome of an import (spec RES-1).
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The files it leads to, by path, sorted.
    Files(Vec<String>),
    /// Outside the project.
    Outside(Place),
    /// A path git ignores, which is never a fault.
    Ignored(String),
    /// Nothing.
    Unresolved,
}

/// Which rule decided an import's outcome: a step of RES-2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Step 1: a relative specifier.
    Relative,
    /// Step 3: a name in the package's `imports`.
    Imports,
    /// Step 6: anything else.
    Outside,
}

impl Rule {
    /// The rule's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Relative => "relative",
            Self::Imports => "imports",
            Self::Outside => "outside",
        }
    }
}

/// Where one import leads, and which rule decided it.
#[derive(Debug)]
pub struct Resolution {
    /// The outcome.
    pub outcome: Outcome,
    /// The rule that decided it.
    pub rule: Rule,
}

/// What looking for a path found.
enum Found {
    /// An outcome that needs nothing more.
    Done(Outcome),
    /// No file: the paths to ask git about, the path as written first (spec RES-4).
    Missing(Vec<String>),
}

/// The project as the resolver needs it.
pub struct Resolver<'p> {
    /// The paths of the file list.
    files: BTreeSet<&'p str>,
    /// What the file list leaves out that an import can still lead into.
    listing: &'p Listing,
    /// The packages, for the dependencies they list.
    packages: &'p Packages,
}

impl<'p> Resolver<'p> {
    /// A resolver over the file list and the packages.
    pub fn new(listing: &'p Listing, packages: &'p Packages) -> Self {
        Self {
            files: listing
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect(),
            listing,
            packages,
        }
    }

    /// Where every import of every script read leads, by the script's path, in the order of its imports.
    pub fn resolve(
        &self,
        root: &Path,
        scripts: &Scripts,
    ) -> Result<BTreeMap<String, Vec<Resolution>>, Stop> {
        let mut found: Vec<(&str, Vec<(Found, Rule)>)> = Vec::with_capacity(scripts.read.len());

        for (path, read) in &scripts.read {
            found.push((
                path,
                read.imports
                    .iter()
                    .map(|import| self.one(path, import))
                    .collect(),
            ));
        }

        // One start of git answers for every path that led to no file.
        let asked: BTreeSet<&str> = found
            .iter()
            .flat_map(|(_, imports)| imports)
            .filter_map(|(found, _)| match found {
                Found::Missing(paths) => Some(paths.iter().map(String::as_str)),
                Found::Done(_) => None,
            })
            .flatten()
            .collect();
        let ignored: BTreeSet<String> = git::ignored(root, &asked.into_iter().collect::<Vec<_>>())?
            .into_iter()
            .collect();

        Ok(found
            .into_iter()
            .map(|(path, imports)| {
                let resolutions = imports
                    .into_iter()
                    .map(|(found, rule)| {
                        let outcome = match found {
                            Found::Done(outcome) => outcome,
                            Found::Missing(paths) => paths
                                .into_iter()
                                .find(|path| ignored.contains(path))
                                .map_or(Outcome::Unresolved, Outcome::Ignored),
                        };

                        Resolution { outcome, rule }
                    })
                    .collect();

                (path.to_owned(), resolutions)
            })
            .collect())
    }

    /// Where one import leads, before git is asked about what led to no file (spec RES-2).
    fn one(&self, importing: &str, import: &Import) -> (Found, Rule) {
        let specifier = written_path(&import.specifier);

        if is_relative(specifier) {
            let folder = importing.rsplit_once('/').map_or("", |(folder, _)| folder);

            return match joined(folder, specifier) {
                Some(path) => (self.locate(&path), Rule::Relative),
                None => (
                    Found::Done(Outcome::Outside(Place::AboveRoot)),
                    Rule::Relative,
                ),
            };
        }

        if specifier.starts_with('#') {
            return (Found::Done(Outcome::Unresolved), Rule::Imports);
        }

        let place = if specifier.starts_with("node:") || BUILTINS.contains(&specifier) {
            Place::Builtin
        } else if is_absolute(specifier) || is_url(specifier) {
            Place::Elsewhere
        } else {
            let name = package_name(specifier);

            Place::Package {
                listed: self.listed(importing, &name),
                name,
            }
        };

        (Found::Done(Outcome::Outside(place)), Rule::Outside)
    }

    /// Whether a `package.json` at or above the importing file lists the package (spec RES-8).
    fn listed(&self, importing: &str, name: &str) -> bool {
        self.packages.list.iter().any(|package| {
            let above = package.folder == "."
                || importing
                    .strip_prefix(package.folder.as_str())
                    .is_some_and(|rest| rest.starts_with('/'));

            above && package.dependencies.contains_key(name)
        })
    }

    /// Looks for a path, relative to the root, as TypeScript finds one, in the file list (spec RES-3).
    fn locate(&self, path: &str) -> Found {
        if under(path, &self.listing.elsewhere) {
            return Found::Done(Outcome::Outside(Place::OtherRepository));
        }

        if under(path, &self.listing.links) {
            return Found::Done(Outcome::Outside(Place::Link));
        }

        // Each candidate with whether it is the path as written or a source form, the ones git is asked
        // about when nothing is found.
        let mut candidates: Vec<(String, bool)> = Vec::new();

        if let Some((stem, forms)) = SOURCE_FORMS
            .iter()
            .find_map(|(output, forms)| path.strip_suffix(output).map(|stem| (stem, *forms)))
        {
            candidates.extend(forms.iter().map(|form| (format!("{stem}{form}"), true)));
        }

        if !path.is_empty() {
            candidates.push((path.to_owned(), true));
            candidates.extend(
                ENDINGS
                    .iter()
                    .map(|ending| (format!("{path}{ending}"), is_source(ending))),
            );
        }

        let index = if path.is_empty() {
            "index".to_owned()
        } else {
            format!("{path}/index")
        };

        candidates.extend(
            ENDINGS
                .iter()
                .map(|ending| (format!("{index}{ending}"), is_source(ending))),
        );

        match candidates
            .iter()
            .find(|(candidate, _)| self.files.contains(candidate.as_str()))
        {
            Some((found, _)) => Found::Done(Outcome::Files(self.with_twin(found))),
            // The path as written first, so that it is the one named when git ignores it.
            None => Found::Missing(
                std::iter::once(path.to_owned())
                    .chain(
                        candidates
                            .into_iter()
                            .filter(|(candidate, asked)| *asked && candidate != path)
                            .map(|(candidate, _)| candidate),
                    )
                    .filter(|candidate| !candidate.is_empty())
                    .collect(),
            ),
        }
    }

    /// A file found, with the other of its pair where the list holds it: a script and the type
    /// declaration file of the same name (spec RES-3).
    fn with_twin(&self, found: &str) -> Vec<String> {
        let twin = PAIRS.iter().find_map(|(script, declaration)| {
            if let Some(stem) = found.strip_suffix(declaration) {
                Some(format!("{stem}{script}"))
            } else {
                found
                    .strip_suffix(script)
                    .map(|stem| format!("{stem}{declaration}"))
            }
        });
        let mut files = vec![found.to_owned()];

        if let Some(twin) = twin.filter(|twin| self.files.contains(twin.as_str())) {
            files.push(twin);
        }

        files.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        files
    }
}

/// Whether an ending is a source form, which git is asked about when nothing is found: a TypeScript
/// ending or a type declaration's (spec RES-4).
fn is_source(ending: &str) -> bool {
    [".ts", ".tsx", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts"].contains(&ending)
}

/// The specifier without a query or a fragment (spec IMP-4).
fn written_path(specifier: &str) -> &str {
    let query = specifier.find('?').unwrap_or(specifier.len());
    let fragment = specifier
        .get(1..)
        .and_then(|rest| rest.find('#'))
        .map_or(specifier.len(), |at| at + 1);

    &specifier[..query.min(fragment)]
}

/// Whether a specifier is relative: `.`, `..`, or one that begins with `./` or `../` (spec RES-2).
fn is_relative(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}

/// Whether a specifier is an absolute path: from the top of a drive or a file system.
fn is_absolute(specifier: &str) -> bool {
    let bytes = specifier.as_bytes();
    let drive = bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');

    specifier.starts_with('/') || specifier.starts_with('\\') || drive
}

/// Whether a specifier is a URL: a scheme of two characters or more, then a colon.
fn is_url(specifier: &str) -> bool {
    specifier.split_once(':').is_some_and(|(scheme, _)| {
        scheme.len() > 1
            && scheme.starts_with(|first: char| first.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character))
    })
}

/// The name of the package a bare specifier names: its first segment, or its first two when it begins
/// with `@` (spec RES-2).
fn package_name(specifier: &str) -> String {
    let segments = if specifier.starts_with('@') { 2 } else { 1 };

    specifier
        .split('/')
        .take(segments)
        .collect::<Vec<_>>()
        .join("/")
}

/// A folder and a relative path joined, with `.` and `..` worked out; nothing when it leads above the
/// root (spec LIST-8). The root itself is the empty path.
fn joined(folder: &str, relative: &str) -> Option<String> {
    let mut segments: Vec<&str> = folder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    for segment in relative.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => segments.push(other),
        }
    }

    Some(segments.join("/"))
}

/// Whether a path is one of the paths given, or lies under one of them.
fn under(path: &str, places: &BTreeSet<String>) -> bool {
    places.iter().any(|place| {
        path.strip_prefix(place.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the imports of every script, and where each import leads.

use std::collections::BTreeMap;
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
use crate::resolve::{Outcome, Place, Resolution, Resolver};
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
    let listing = files::list(&root, &mut notices)?;
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed.as_ref(), &mut notices)?;
    let packages = packages::read(
        &root,
        files,
        &marks.data,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    )?;
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolved = Resolver::new(&listing, &packages).resolve(&root, &scripts)?;
    let document = json!({
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    Ok(format!("{document:#}"))
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, and the
/// count of those that cannot be followed.
fn file(
    file: &File,
    marks: &Marks,
    packages: &Packages,
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
) -> Value {
    let mut printed = json!({
        "data": marks.data.contains(&file.path),
        "generated": marks.generated.contains(&file.path),
        "kind": file.kind.name(),
        "package": packages.of_file.get(&file.path),
        "path": file.path,
        "tracked": file.tracked,
    });

    if let (Some(read), Some(resolutions)) =
        (scripts.read.get(&file.path), resolved.get(&file.path))
    {
        printed["computedImports"] = json!(read.computed);
        printed["imports"] = read
            .imports
            .iter()
            .zip(resolutions)
            .map(|(each, resolution)| import(each, resolution))
            .collect();
    }

    printed
}

/// One import as it is printed, with where it leads and the rule that decided it (spec OUT-1).
fn import(import: &Import, resolution: &Resolution) -> Value {
    let mut printed = json!({
        "form": import.form.name(),
        "line": import.line,
        "query": import.has_query(),
        "rule": resolution.rule.name(),
        "specifier": import.specifier,
        "typeOnly": import.type_only,
    });

    match &resolution.outcome {
        Outcome::Files(files) => {
            printed["outcome"] = json!("files");
            printed["files"] = json!(files);
        }
        Outcome::Outside(place) => {
            printed["outcome"] = json!("outside");
            printed["outside"] = json!(place.name());

            if let Place::Package { name, listed } = place {
                printed["package"] = json!(name);
                printed["listed"] = json!(listed);
            }
        }
        Outcome::Ignored(path) => {
            printed["outcome"] = json!("ignored");
            printed["ignored"] = json!(path);
        }
        Outcome::Unresolved => printed["outcome"] = json!("unresolved"),
    }

    printed
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
//! files, its packages, its scripts' imports and where they lead. It reports no findings yet.

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
mod resolve;
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
```

Expected: each exits 0. The tests report 7 passed in the binary's own tests, 6 for `cli`, 5 for
`declarations`, 16 for `file-list`, 10 for `hand-off`, 2 for `hand-off-schema`, 11 for `imports`,
9 for `packages` and 6 for `resolve`.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src tests
git commit -m "feat(resolve): lead relative imports to files and name every place outside" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: one commit on `feat/reading-4-resolve`.

### Task 2: packages by name, `#` names, and build output back to source

**Implements:** RES-2, RES-4, RES-5, RES-6, MAP-1, MAP-2, MAP-3, MAP-4, PACK-3, PACK-5, PACK-6, FIX-2, FIX-3, FIX-4

**Test:** `cargo test --release --locked --test resolve`

**Files:**
- Modify: `src/packages.rs`, `src/resolve.rs`, `src/graph.rs`
- Test: `tests/resolve.test.rs`

**Interfaces:**
- Consumes: Task 1's `resolve::Resolver`, `Outcome`, `Rule` and `packages::Package`.
- Produces: `packages::Package::exports`, `imports` and `entries`; `resolve::Outcome::Ignored` with
  `path` and `from`; `resolve::Rule::Package`; and the resolver's `target`, `relative` and `sources`
  steps that Task 3 adds MAP-1's first step to.

- [ ] **Step 1: Write the failing tests**

```rust path=tests/resolve.test.rs
//! Runs the built binary on made-up projects and checks where imports lead: relative ones, packages of
//! the project by name, `#` names, and every kind of outside (spec RES-1 to RES-6, RES-8, RES-10, MAP-1
//! to MAP-4, LIST-5, LIST-8, PACK-3, PACK-5, PACK-6, FIX-2, FIX-3, FIX-4).

#[path = "support/project.rs"]
mod project;

use std::fs;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// The import of a file, by its specifier, as `graph` prints it.
fn import<'a>(document: &'a Value, path: &str, specifier: &str) -> &'a Value {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .and_then(|file| file["imports"].as_array())
        .and_then(|imports| {
            imports
                .iter()
                .find(|import| import["specifier"] == json!(specifier))
        })
        .unwrap_or_else(|| panic!("{path} imports {specifier}"))
}

/// What an import leads to: its outcome, and the files, the place or the ignored path that go with it.
fn led(document: &Value, path: &str, specifier: &str) -> Value {
    let import = import(document, path, specifier);
    let mut led = json!({ "outcome": import["outcome"], "rule": import["rule"] });

    for key in ["files", "outside", "package", "listed", "ignored", "from"] {
        if let Some(value) = import.get(key) {
            led[key] = value.clone();
        }
    }

    led
}

/// An import that leads to the files given, by a relative specifier.
fn files(found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": "relative", "files": found })
}

#[test]
fn finds_a_file_for_each_way_a_relative_path_is_written() {
    let project = Project::new("resolve-relative");

    project
        .write(
            "src/main.ts",
            "import './a.js';\nimport './b.ts';\nimport './types';\nimport './dir';\nimport './c.mjs';\n\
             import '../lib/d';\nimport './styles.css';\nimport './notes.md?raw';\nimport '.';\n",
        )
        .write("src/a.ts", "")
        .write("src/b.ts", "")
        .write("src/types.d.ts", "")
        .write("src/dir/index.tsx", "")
        .write("src/c.mts", "")
        .write("lib/d.js", "")
        .write("src/styles.css", "")
        .write("src/notes.md", "")
        .write("src/index.ts", "")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(main("./a.js"), files(&["src/a.ts"]));
    assert_eq!(main("./b.ts"), files(&["src/b.ts"]));
    assert_eq!(main("./types"), files(&["src/types.d.ts"]));
    assert_eq!(main("./dir"), files(&["src/dir/index.tsx"]));
    assert_eq!(main("./c.mjs"), files(&["src/c.mts"]));
    assert_eq!(main("../lib/d"), files(&["lib/d.js"]));
    assert_eq!(main("./styles.css"), files(&["src/styles.css"]));
    assert_eq!(main("./notes.md?raw"), files(&["src/notes.md"]));
    assert_eq!(main("."), files(&["src/index.ts"]));
}

#[test]
fn leads_to_both_a_script_and_the_declaration_file_beside_it() {
    let project = Project::new("resolve-pair");

    project
        .write(
            "src/main.ts",
            "import './plain.js';\nimport './plain';\nimport './typed.d.ts';\n",
        )
        .write("src/plain.js", "")
        .write("src/plain.d.ts", "")
        .write("src/typed.mjs", "")
        .write("src/typed.d.mts", "")
        .write("src/typed.d.ts", "")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(
        main("./plain.js"),
        files(&["src/plain.d.ts", "src/plain.js"])
    );
    assert_eq!(main("./plain"), files(&["src/plain.d.ts", "src/plain.js"]));
    assert_eq!(main("./typed.d.ts"), files(&["src/typed.d.ts"]));
}

#[test]
fn tells_an_ignored_path_from_a_missing_one() {
    let project = Project::new("resolve-ignored");

    project
        .write(".gitignore", "src/generated/\n*.js\n")
        .write("src/types/.gitignore", "*.d.ts\n")
        .write(
            "src/main.ts",
            "import './generated/api';\nimport './fooo';\nimport './fooo.js';\nimport './types/built';\n\
             import './missing';\nimport './Main';\n",
        )
        .commit();

    let run = |label: &str| {
        let document = project.graph();
        let main = |specifier: &str| led(&document, "src/main.ts", specifier);

        assert_eq!(
            main("./generated/api"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/generated/api" }),
            "{label}"
        );
        // A mistyped import with no extension stays unresolved though every `.js` file is ignored.
        assert_eq!(
            main("./fooo"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
        // Written with `.js`, the path itself is ignored, so the import is (spec RES-4).
        assert_eq!(
            main("./fooo.js"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/fooo.js" }),
            "{label}"
        );
        // An import with no extension of a file git ignores by its ending.
        assert_eq!(
            main("./types/built"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/types/built.d.ts" }),
            "{label}"
        );
        assert_eq!(
            main("./missing"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
        // Letter case counts on every platform (spec RES-10).
        assert_eq!(
            main("./Main"),
            json!({ "outcome": "unresolved", "rule": "relative" }),
            "{label}"
        );
    };

    // The same with and without the ignored files on disk (spec AN-5, FIX-3).
    run("without the ignored files");
    project
        .write("src/generated/api.ts", "")
        .write("src/fooo.js", "")
        .write("src/types/built.d.ts", "");
    run("with the ignored files");
}

#[test]
fn names_each_place_outside_the_project() {
    let other = Project::new("resolve-outside-other");

    other.write("other.ts", "").commit();

    let project = Project::new("resolve-outside");

    project
        .write("package.json", r#"{ "dependencies": { "react": "^19.0.0" } }"#)
        .write(
            "src/main.ts",
            "import 'node:fs';\nimport 'fs/promises';\nimport 'react';\nimport 'react-dom/client';\n\
             import '@scope/pkg/deep';\nimport '/abs/file.js';\nimport 'https://cdn.example/x.js';\n\
             import 'C:/drive/file.js';\nimport '../nested/inner';\nimport '../sub/other';\nimport './link-to-main';\n\
             import './linked-folder/inner';\n",
        )
        .commit()
        .write("nested/inner.ts", "");
    project.git(&["-C", "nested", "init", "--quiet"]);
    project.git(&[
        "submodule",
        "--quiet",
        "add",
        other.root.to_str().expect("valid UTF-8"),
        "sub",
    ]);
    project.git(&["commit", "--quiet", "--message", "a submodule"]);

    let source = project.home.join("link-content");

    fs::write(&source, "src/main.ts").expect("the content can be written");

    let blob = project.git(&["hash-object", "-w", source.to_str().expect("valid UTF-8")]);

    fs::write(&source, "../lib").expect("the content can be written");

    let folder = project.git(&["hash-object", "-w", source.to_str().expect("valid UTF-8")]);

    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("120000,{},src/link-to-main", blob.trim()),
    ]);
    project.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("120000,{},src/linked-folder", folder.trim()),
    ]);
    project.git(&["commit", "--quiet", "--message", "two links"]);
    project.git(&["checkout", "--", "src/link-to-main", "src/linked-folder"]);

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);
    let place =
        |rule: &str, place: &str| json!({ "outcome": "outside", "rule": rule, "outside": place });
    let package = |name: &str, listed: bool| json!({ "outcome": "outside", "rule": "outside", "outside": "package", "package": name, "listed": listed });

    assert_eq!(main("node:fs"), place("outside", "builtin"));
    assert_eq!(main("fs/promises"), place("outside", "builtin"));
    assert_eq!(main("react"), package("react", true));
    assert_eq!(main("react-dom/client"), package("react-dom", false));
    assert_eq!(main("@scope/pkg/deep"), package("@scope/pkg", false));
    assert_eq!(main("/abs/file.js"), place("outside", "elsewhere"));
    assert_eq!(
        main("https://cdn.example/x.js"),
        place("outside", "elsewhere")
    );
    assert_eq!(main("C:/drive/file.js"), place("outside", "elsewhere"));
    assert_eq!(
        main("../nested/inner"),
        place("relative", "other-repository")
    );
    assert_eq!(main("../sub/other"), place("relative", "other-repository"));
    assert_eq!(main("./link-to-main"), place("relative", "link"));
    // A path through a linked folder is through a link (spec LIST-3).
    assert_eq!(main("./linked-folder/inner"), place("relative", "link"));
    assert_eq!(notices(&document), ["links-left-out", "no-hand-off-file"]);
}

#[test]
fn leads_a_path_above_a_sub_folder_root_outside() {
    let project = Project::new("resolve-above");

    project
        .write("shared/util.ts", "")
        .write(
            "apps/web/src/main.ts",
            "import '../../../shared/util';\nimport '../lib';\n",
        )
        .write("apps/web/lib/index.ts", "")
        .commit();

    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(paths(&document), ["lib/index.ts", "src/main.ts"]);
    assert_eq!(
        led(&document, "src/main.ts", "../../../shared/util"),
        json!({ "outcome": "outside", "rule": "relative", "outside": "above-root" })
    );
    assert_eq!(
        led(&document, "src/main.ts", "../lib"),
        files(&["lib/index.ts"])
    );
}

#[test]
fn stops_on_a_dependency_list_of_a_form_it_does_not_have() {
    let project = Project::new("resolve-dependencies-form");

    project
        .write("package.json", r#"{ "devDependencies": ["typescript"] }"#)
        .commit();

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `devDependencies` in package.json holds a value of a form it does not have; \
         it must be an object that maps each package's name to the version it asks for; fix it"
    );
}

/// An import that leads to the files given, by the name of a package of the project.
fn by_name(found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": "package", "files": found })
}

/// A workspace whose one package offers its build output through `exports`, which the worktree does
/// not have, and an app that imports it by name (spec FIX-2, pattern 10).
fn workspace_with_build_output(name: &str) -> Project {
    let project = Project::new(name);

    project
        .write(".gitignore", "node_modules/\ndist/\n")
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*", "apps/*"] }"#,
        )
        .write(
            "packages/lib/package.json",
            r#"{
                "name": "@demo/lib",
                "exports": {
                    ".": { "types": "./dist/index.d.ts", "import": "./dist/index.js" },
                    "./util": "./dist/util.js",
                    "./icons/*": "./dist/icons/*/*.js",
                    "./list": ["./dist/list.js", "./dist/fallback.js"],
                    "./platform": { "browser": "./dist/browser.js", "node": "./dist/node.js" },
                    "./typed": { "types": "./dist/typed.d.ts", "default": "./dist/main.js" },
                    "./generated": "./dist/generated.js",
                    "./hidden": null
                }
            }"#,
        )
        .write("packages/lib/src/index.ts", "")
        .write("packages/lib/src/util.ts", "")
        .write("packages/lib/src/icons/star/star.ts", "")
        .write("packages/lib/src/list.ts", "")
        .write("packages/lib/src/browser.ts", "")
        .write("packages/lib/src/node.ts", "")
        .write("packages/lib/src/main.ts", "")
        .write(
            "apps/web/package.json",
            r#"{ "name": "web", "dependencies": { "@demo/lib": "workspace:*" } }"#,
        )
        .write(
            "apps/web/src/main.ts",
            "import '@demo/lib';\nimport '@demo/lib/util';\nimport '@demo/lib/icons/star';\n\
             import '@demo/lib/list';\nimport '@demo/lib/platform';\nimport '@demo/lib/typed';\n\
             import '@demo/lib/generated';\nimport '@demo/lib/hidden';\nimport '@demo/lib/missing';\n",
        )
        .commit();

    project
}

#[test]
fn leads_an_import_by_a_workspace_packages_name_through_its_exports_to_source() {
    let project = workspace_with_build_output("resolve-exports");

    let run = |label: &str| {
        let document = project.graph();
        let web = |specifier: &str| led(&document, "apps/web/src/main.ts", specifier);

        // Two conditions that name build output of one source lead to that one file (spec RES-5).
        assert_eq!(
            web("@demo/lib"),
            by_name(&["packages/lib/src/index.ts"]),
            "{label}"
        );
        assert_eq!(
            web("@demo/lib/util"),
            by_name(&["packages/lib/src/util.ts"]),
            "{label}"
        );
        // A `*` written twice in a target is filled in at both places (spec RES-6).
        assert_eq!(
            web("@demo/lib/icons/star"),
            by_name(&["packages/lib/src/icons/star/star.ts"]),
            "{label}"
        );
        // A list is followed entry by entry; the entry with no source adds nothing (spec PACK-6).
        assert_eq!(
            web("@demo/lib/list"),
            by_name(&["packages/lib/src/list.ts"]),
            "{label}"
        );
        // Conditions that name two different sources lead to both.
        assert_eq!(
            web("@demo/lib/platform"),
            by_name(&["packages/lib/src/browser.ts", "packages/lib/src/node.ts"]),
            "{label}"
        );
        // Conditions whose first target has no source lead to what the others find.
        assert_eq!(
            web("@demo/lib/typed"),
            by_name(&["packages/lib/src/main.ts"]),
            "{label}"
        );
        // A target in an ignored build folder that maps to no source is ignored, with the target it
        // came from (spec MAP-4).
        assert_eq!(
            web("@demo/lib/generated"),
            json!({
                "outcome": "ignored",
                "rule": "package",
                "ignored": "packages/lib/dist/generated.js",
                "from": "packages/lib/dist/generated.js",
            }),
            "{label}"
        );
        // A subpath `exports` holds back, or does not offer, leads nowhere (spec RES-6).
        assert_eq!(
            web("@demo/lib/hidden"),
            json!({ "outcome": "unresolved", "rule": "package" }),
            "{label}"
        );
        assert_eq!(
            web("@demo/lib/missing"),
            json!({ "outcome": "unresolved", "rule": "package" }),
            "{label}"
        );
    };

    // The same with and without `node_modules` and the build output on disk (spec AN-5, FIX-3).
    run("without node_modules and build output");
    project
        .write(
            "node_modules/@demo/lib/package.json",
            r#"{ "name": "@demo/lib", "main": "./elsewhere.js" }"#,
        )
        .write("node_modules/@demo/lib/elsewhere.js", "")
        .write("packages/lib/dist/index.js", "")
        .write("packages/lib/dist/index.d.ts", "")
        .write("packages/lib/dist/generated.js", "");
    run("with node_modules and build output");
}

#[test]
fn looks_up_a_hash_name_in_the_imports_of_the_files_package() {
    let project = Project::new("resolve-hash");

    project
        .write(".gitignore", "dist/\nsrc/generated/\n")
        .write(
            "package.json",
            r##"{
                "name": "app",
                "dependencies": { "react": "^19.0.0" },
                "imports": {
                    "#lib/*": { "types": "./dist/lib/*.d.ts", "default": "./dist/lib/*.js" },
                    "#lib/special": "./dist/special.js",
                    "#config": "./config.js",
                    "#react": "react",
                    "#generated": "./src/generated/api.ts"
                }
            }"##,
        )
        .write("config.js", "")
        .write("src/lib/a.ts", "")
        .write("src/special.ts", "")
        .write(
            "src/main.ts",
            "import '#lib/a';\nimport '#lib/special';\nimport '#config';\nimport '#react';\n\
             import '#generated';\nimport '#missing';\n",
        )
        .write("tools/package.json", r#"{ "name": "tools" }"#)
        .write("tools/run.ts", "import '#config';\n")
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);
    let imports = |found: &[&str]| json!({ "outcome": "files", "rule": "imports", "files": found });

    assert_eq!(main("#lib/a"), imports(&["src/lib/a.ts"]));
    // A key with no `*` wins over a pattern that also matches (spec RES-5).
    assert_eq!(main("#lib/special"), imports(&["src/special.ts"]));
    // A committed JavaScript file is its own source (spec MAP-2).
    assert_eq!(main("#config"), imports(&["config.js"]));
    // A target that is a package's name leads where an import of that name does.
    assert_eq!(
        main("#react"),
        json!({ "outcome": "outside", "rule": "imports", "outside": "package", "package": "react", "listed": true })
    );
    assert_eq!(
        main("#generated"),
        json!({
            "outcome": "ignored",
            "rule": "imports",
            "ignored": "src/generated/api.ts",
            "from": "src/generated/api.ts",
        })
    );
    assert_eq!(
        main("#missing"),
        json!({ "outcome": "unresolved", "rule": "imports" })
    );
    // A `#` name is looked up in the importing file's own package only.
    assert_eq!(
        led(&document, "tools/run.ts", "#config"),
        json!({ "outcome": "unresolved", "rule": "imports" })
    );
}

#[test]
fn leads_a_package_with_no_exports_through_its_entries_its_index_or_a_path_inside_it() {
    let project = Project::new("resolve-entries");

    project
        .write(".gitignore", "dist/\n")
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )
        .write(
            "packages/typed/package.json",
            r#"{ "name": "typed", "typings": "./dist/index.d.ts", "main": "dist/index.cjs" }"#,
        )
        .write("packages/typed/src/index.ts", "")
        .write("packages/typed/src/helper.ts", "")
        .write("packages/plain/package.json", r#"{ "name": "plain" }"#)
        .write("packages/plain/index.js", "")
        .write(
            "packages/both/package.json",
            r#"{ "name": "both", "module": "./esm/index.js", "main": "./cjs/index.js" }"#,
        )
        .write("packages/both/esm/index.js", "")
        .write("packages/both/cjs/index.js", "")
        .write(
            "src/main.ts",
            "import 'typed';\nimport 'typed/src/helper';\nimport 'typed/dist/helper.js';\nimport 'plain';\n\
             import 'both';\nimport '../packages/typed';\n",
        )
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    // `typings` is the older name of `types`; both entries map to one source (spec PACK-6, MAP-1).
    assert_eq!(main("typed"), by_name(&["packages/typed/src/index.ts"]));
    assert_eq!(
        main("typed/src/helper"),
        by_name(&["packages/typed/src/helper.ts"])
    );
    // A subpath into build output maps to its source (spec MAP-1).
    assert_eq!(
        main("typed/dist/helper.js"),
        by_name(&["packages/typed/src/helper.ts"])
    );
    // With no entries, the bare name leads to the index file of the package's folder.
    assert_eq!(main("plain"), by_name(&["packages/plain/index.js"]));
    // Every file the entries lead to.
    assert_eq!(
        main("both"),
        by_name(&["packages/both/cjs/index.js", "packages/both/esm/index.js"])
    );
    // A folder that holds a `package.json` is tried through its entries (spec RES-3).
    assert_eq!(
        main("../packages/typed"),
        files(&["packages/typed/src/index.ts"])
    );
}

#[test]
fn leads_a_name_to_a_package_outside_the_workspace_only_by_itself_or_a_local_protocol() {
    let project = Project::new("resolve-stray");

    project
        .write(
            "package.json",
            r#"{
                "name": "root",
                "workspaces": ["packages/*"],
                "dependencies": { "react": "^19.0.0", "local-tool": "file:./tools/local" },
                "devDependencies": { "shared": "workspace:^" }
            }"#,
        )
        .write("examples/react/package.json", r#"{ "name": "react" }"#)
        .write("examples/react/index.ts", "import 'react';\n")
        .write(
            "tools/local/package.json",
            r#"{ "name": "local-tool", "main": "run.js" }"#,
        )
        .write("tools/local/run.js", "")
        .write("vendor/shared/package.json", r#"{ "name": "shared" }"#)
        .write("vendor/shared/index.ts", "")
        .write(
            "src/main.ts",
            "import 'react';\nimport 'local-tool';\nimport 'shared';\n",
        )
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    // A stray folder with an installed dependency's name does not take its imports (spec PACK-3).
    assert_eq!(
        main("react"),
        json!({ "outcome": "outside", "rule": "outside", "outside": "package", "package": "react", "listed": true })
    );
    // A package may import itself by its own name (spec RES-2, step 5).
    assert_eq!(
        led(&document, "examples/react/index.ts", "react"),
        by_name(&["examples/react/index.ts"])
    );
    assert_eq!(main("local-tool"), by_name(&["tools/local/run.js"]));
    assert_eq!(main("shared"), by_name(&["vendor/shared/index.ts"]));
}

#[test]
fn leads_a_name_two_workspace_packages_share_to_the_first_by_path() {
    let project = Project::new("resolve-repeated");

    project
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "same" }"#)
        .write("packages/a/index.ts", "")
        .write("packages/b/package.json", r#"{ "name": "same" }"#)
        .write("packages/b/index.ts", "")
        .write("src/main.ts", "import 'same';\n")
        .commit();

    let document = project.graph();

    assert_eq!(
        led(&document, "src/main.ts", "same"),
        by_name(&["packages/a/index.ts"])
    );
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "package-name-repeated"]
    );
}

#[test]
fn stops_on_a_package_field_of_a_form_it_does_not_have() {
    let cases = [
        (
            "exports",
            r#"{ "exports": { ".": "./a.js", "import": "./b.js" } }"#,
            "a path, a list, or an object whose keys are all subpaths beginning with `.` or all conditions",
        ),
        (
            "exports",
            r#"{ "exports": 1 }"#,
            "a path, a list, or an object whose keys are all subpaths beginning with `.` or all conditions",
        ),
        (
            "imports",
            r#"{ "imports": { "lib": "./lib.js" } }"#,
            "an object whose keys begin with `#` and whose values are paths, lists or conditions",
        ),
        ("types", r#"{ "types": ["./a.d.ts"] }"#, "a path"),
        ("typings", r#"{ "typings": true }"#, "a path"),
        ("module", r#"{ "module": {} }"#, "a path"),
        ("main", r#"{ "main": null }"#, "a path"),
    ];

    for (field, manifest, form) in cases {
        let project = Project::new("resolve-fields-form");

        project.write("package.json", manifest).commit();

        assert_eq!(
            stopped(&project.run(|_| {})),
            format!(
                "yak-analyzer: the field `{field}` in package.json holds a value of a form it does not have; \
                 it must be {form}; fix it"
            ),
            "{manifest}"
        );
    }
}
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test resolve
```

Expected: the six new tests fail: an import by a package's name is still outside, a `#` name is
unresolved, and a misshapen `exports` stops nothing; Task 1's six tests pass.

- [ ] **Step 3: Keep the fields an import by name reads, and resolve through them**

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
    /// The packages its four lists of dependencies name, each with the version it asks for.
    pub dependencies: BTreeMap<String, String>,
    /// Its `exports`, which say what an import of the package by name may reach (spec RES-6).
    pub exports: Option<Value>,
    /// Its `imports`, the names that begin with `#` (spec RES-5).
    pub imports: Option<Value>,
    /// The targets of its `types` (or `typings`), `module` and `main`, in that order, those it states.
    pub entries: Vec<String>,
}

/// What a `package.json` states that a package keeps.
struct Fields {
    name: Option<String>,
    dependencies: BTreeMap<String, String>,
    exports: Option<Value>,
    imports: Option<Value>,
    entries: Vec<String>,
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
    let mut names: BTreeMap<&str, Fields> = BTreeMap::new();

    for (folder, fields) in &read {
        let file = manifest_path(folder);

        names.insert(
            folder,
            Fields {
                name: name(fields, &file)?,
                dependencies: dependencies(fields, &file)?,
                exports: targets(fields, "exports", &file)?,
                imports: imports(fields, &file)?,
                entries: entries(fields, &file)?,
            },
        );
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
        .map(|(folder, fields)| {
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
                name: fields.name,
                workspace,
                detected,
                dependencies: fields.dependencies,
                exports: fields.exports,
                imports: fields.imports,
                entries: fields.entries,
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

/// The form a target of `exports` or `imports` may take: a path, null, a list of targets, or an object
/// of conditions whose values are targets.
fn is_target(value: &Value) -> bool {
    match value {
        Value::String(_) | Value::Null => true,
        Value::Array(targets) => targets.iter().all(is_target),
        Value::Object(conditions) => conditions.values().all(is_target),
        _ => false,
    }
}

/// A `package.json`'s `exports`: one target, or an object whose keys are all subpaths, which begin with
/// `.`, or all conditions, none of which does.
fn targets(fields: &Map<String, Value>, field: &str, file: &str) -> Result<Option<Value>, Stop> {
    let form = "a path, a list, or an object whose keys are all subpaths beginning with `.` or all \
                conditions";
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    let keys_agree = value.as_object().is_none_or(|keys| {
        let subpaths = keys.keys().filter(|key| key.starts_with('.')).count();

        subpaths == 0 || subpaths == keys.len()
    });

    if is_target(value) && keys_agree {
        Ok(Some(value.clone()))
    } else {
        Err(misshapen(field, file, form))
    }
}

/// A `package.json`'s `imports`: an object whose keys begin with `#` and whose values are targets.
fn imports(fields: &Map<String, Value>, file: &str) -> Result<Option<Value>, Stop> {
    let form =
        "an object whose keys begin with `#` and whose values are paths, lists or conditions";

    match fields.get("imports") {
        None => Ok(None),
        Some(Value::Object(map))
            if map
                .iter()
                .all(|(key, value)| key.starts_with('#') && is_target(value)) =>
        {
            Ok(fields.get("imports").cloned())
        }
        Some(_) => Err(misshapen("imports", file, form)),
    }
}

/// The targets of a `package.json`'s `types` (or its older name `typings`), `module` and `main`, in
/// that order, those it states.
fn entries(fields: &Map<String, Value>, file: &str) -> Result<Vec<String>, Stop> {
    let types = if fields.contains_key("types") {
        "types"
    } else {
        "typings"
    };
    let mut entries = Vec::new();

    for field in [types, "module", "main"] {
        match fields.get(field) {
            None => {}
            Some(Value::String(target)) => entries.push(target.clone()),
            Some(_) => return Err(misshapen(field, file, "a path")),
        }
    }

    Ok(entries)
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();

    for list in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let form = "an object that maps each package's name to the version it asks for";
        let entries = match fields.get(list) {
            None => continue,
            Some(Value::Object(entries)) => entries,
            Some(_) => return Err(misshapen(list, file, form)),
        };

        for (package, version) in entries {
            let version = version
                .as_str()
                .ok_or_else(|| misshapen(list, file, form))?;

            named
                .entry(package.clone())
                .or_insert_with(|| version.to_owned());
        }
    }

    Ok(named)
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

```rust path=src/resolve.rs
//! Where each import leads (spec RES-1 to RES-6, RES-8, RES-10, MAP-1 to MAP-4, LIST-5).
//!
//! A path is looked for in the file list, never on disk, with exact letter case. What leads to no file
//! is asked about once, with one start of git for all of them, to tell an ignored path from a missing
//! one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Map, Value};

use crate::files::Listing;
use crate::git;
use crate::imports::Import;
use crate::packages::{Package, Packages};
use crate::scripts::Scripts;
use crate::stop::Stop;

/// The modules built into Node, as `require('module').builtinModules` lists them in Node 22.19.0, those
/// whose names begin with `_` left out. Any specifier that begins with `node:` is built in too.
const BUILTINS: [&str; 54] = [
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// The endings a script's path may be given, in the order they are tried: TypeScript before JavaScript,
/// then a type declaration file (spec RES-3).
const ENDINGS: [&str; 9] = [
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".d.ts",
];

/// A script's output ending, and the source forms it is looked for under first (spec RES-3).
const SOURCE_FORMS: [(&str, &[&str]); 4] = [
    (".js", &[".ts", ".tsx", ".d.ts"]),
    (".jsx", &[".tsx"]),
    (".mjs", &[".mts", ".d.mts"]),
    (".cjs", &[".cts", ".d.cts"]),
];

/// A script and the type declaration file that describes it, by their endings (spec RES-3).
const PAIRS: [(&str, &str); 3] = [(".js", ".d.ts"), (".mjs", ".d.mts"), (".cjs", ".d.cts")];

/// The endings of build output that MAP-1 replaces: a type declaration's first, since `.d.ts` also
/// ends in `.ts`, then a script's output forms.
const OUTPUT_ENDINGS: [&str; 6] = [".d.ts", ".d.mts", ".d.cts", ".js", ".mjs", ".cjs"];

/// The source extensions MAP-1 puts in place of an output ending, in the order they are tried.
const SOURCE_ENDINGS: [&str; 8] = [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"];

/// The protocols of a dependency's version that name a package in the project (spec RES-2, step 5).
const LOCAL_PROTOCOLS: [&str; 3] = ["file:", "link:", "portal:"];

/// Where outside the project an import leads (spec RES-1, RES-8).
#[derive(Debug, PartialEq, Eq)]
pub enum Place {
    /// A package that is not in the project, with whether a `package.json` at or above the importing
    /// file lists it.
    Package { name: String, listed: bool },
    /// A module built into Node.
    Builtin,
    /// A path above the root.
    AboveRoot,
    /// A path inside a nested repository, a submodule or another worktree.
    OtherRepository,
    /// A symbolic link, or a path through one.
    Link,
    /// Something loaded from elsewhere: an absolute path or a URL.
    Elsewhere,
}

impl Place {
    /// The place's name in what `graph` prints.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Package { .. } => "package",
            Self::Builtin => "builtin",
            Self::AboveRoot => "above-root",
            Self::OtherRepository => "other-repository",
            Self::Link => "link",
            Self::Elsewhere => "elsewhere",
        }
    }
}

/// The one outcome of an import (spec RES-1).
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The files it leads to, by path, sorted.
    Files(Vec<String>),
    /// Outside the project.
    Outside(Place),
    /// A path git ignores, which is never a fault, and the target of a `package.json` it came from,
    /// where it came from one (spec MAP-4).
    Ignored { path: String, from: Option<String> },
    /// Nothing.
    Unresolved,
}

/// Which rule decided an import's outcome: a step of RES-2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Step 1: a relative specifier.
    Relative,
    /// Step 3: a name in the package's `imports`.
    Imports,
    /// Step 5: the name of a package in the project.
    Package,
    /// Step 6: anything else.
    Outside,
}

impl Rule {
    /// The rule's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Relative => "relative",
            Self::Imports => "imports",
            Self::Package => "package",
            Self::Outside => "outside",
        }
    }
}

/// Where one import leads, and which rule decided it.
#[derive(Debug)]
pub struct Resolution {
    /// The outcome.
    pub outcome: Outcome,
    /// The rule that decided it.
    pub rule: Rule,
}

/// A path git is asked about when nothing is found, and the target it came from, if any (spec MAP-4).
#[derive(Debug)]
struct Asked {
    path: String,
    from: Option<String>,
}

/// What looking for a path found.
#[derive(Debug)]
enum Found {
    /// An outcome that needs nothing more.
    Done(Outcome),
    /// No file: the paths to ask git about, the path as written first (spec RES-4).
    Missing(Vec<Asked>),
}

/// The project as the resolver needs it.
pub struct Resolver<'p> {
    /// The paths of the file list.
    files: BTreeSet<&'p str>,
    /// What the file list leaves out that an import can still lead into.
    listing: &'p Listing,
    /// The packages.
    packages: &'p Packages,
    /// The packages by folder, the root's as the empty path.
    by_folder: BTreeMap<&'p str, &'p Package>,
}

impl<'p> Resolver<'p> {
    /// A resolver over the file list and the packages.
    pub fn new(listing: &'p Listing, packages: &'p Packages) -> Self {
        Self {
            files: listing
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect(),
            listing,
            packages,
            by_folder: packages
                .list
                .iter()
                .map(|package| (folder_path(&package.folder), package))
                .collect(),
        }
    }

    /// Where every import of every script read leads, by the script's path, in the order of its imports.
    pub fn resolve(
        &self,
        root: &Path,
        scripts: &Scripts,
    ) -> Result<BTreeMap<String, Vec<Resolution>>, Stop> {
        let mut found: Vec<(&str, Vec<(Found, Rule)>)> = Vec::with_capacity(scripts.read.len());

        for (path, read) in &scripts.read {
            found.push((
                path,
                read.imports
                    .iter()
                    .map(|import| self.one(path, import))
                    .collect(),
            ));
        }

        // One start of git answers for every path that led to no file.
        let asked: BTreeSet<&str> = found
            .iter()
            .flat_map(|(_, imports)| imports)
            .filter_map(|(found, _)| match found {
                Found::Missing(paths) => Some(paths.iter().map(|asked| asked.path.as_str())),
                Found::Done(_) => None,
            })
            .flatten()
            .collect();
        let ignored: BTreeSet<String> = git::ignored(root, &asked.into_iter().collect::<Vec<_>>())?
            .into_iter()
            .collect();

        Ok(found
            .into_iter()
            .map(|(path, imports)| {
                let resolutions = imports
                    .into_iter()
                    .map(|(found, rule)| {
                        let outcome = match found {
                            Found::Done(outcome) => outcome,
                            Found::Missing(paths) => paths
                                .into_iter()
                                .find(|asked| ignored.contains(&asked.path))
                                .map_or(Outcome::Unresolved, |asked| Outcome::Ignored {
                                    path: asked.path,
                                    from: asked.from,
                                }),
                        };

                        Resolution { outcome, rule }
                    })
                    .collect();

                (path.to_owned(), resolutions)
            })
            .collect())
    }

    /// Where one import leads, before git is asked about what led to no file (spec RES-2).
    fn one(&self, importing: &str, import: &Import) -> (Found, Rule) {
        let specifier = written_path(&import.specifier);

        if is_relative(specifier) {
            let folder = importing.rsplit_once('/').map_or("", |(folder, _)| folder);

            return match joined(folder, specifier) {
                Some(path) => (self.locate(&path, true), Rule::Relative),
                None => (
                    Found::Done(Outcome::Outside(Place::AboveRoot)),
                    Rule::Relative,
                ),
            };
        }

        if specifier.starts_with('#') {
            return (self.hash(importing, specifier), Rule::Imports);
        }

        self.bare(importing, specifier)
    }

    /// A specifier that is neither relative nor a `#` name: a package of the project by its name, or
    /// somewhere outside (spec RES-2, steps 5 and 6).
    fn bare(&self, importing: &str, specifier: &str) -> (Found, Rule) {
        let outside = |place| (Found::Done(Outcome::Outside(place)), Rule::Outside);

        // Before a URL, which `node:` would read as.
        if specifier.starts_with("node:") {
            return outside(Place::Builtin);
        }

        if is_absolute(specifier) || is_url(specifier) {
            return outside(Place::Elsewhere);
        }

        let name = package_name(specifier);

        if let Some(package) = self.named(importing, &name) {
            return (
                self.through(importing, package, &specifier[name.len()..]),
                Rule::Package,
            );
        }

        if BUILTINS.contains(&specifier) {
            return outside(Place::Builtin);
        }

        outside(Place::Package {
            listed: self.listing_of(importing, &name).is_some(),
            name,
        })
    }

    /// The package of the project that an import by a name leads to, where one does (spec RES-2,
    /// step 5, PACK-5): the importing file's own package by its own name, the first workspace package
    /// by path with the name, or the package a dependency at or above the importing file names with a
    /// local protocol.
    fn named(&self, importing: &str, name: &str) -> Option<&'p Package> {
        let named = |package: &&Package| package.name.as_deref() == Some(name);

        if let Some(own) = self.own_package(importing).filter(|package| named(package)) {
            return Some(own);
        }

        if let Some(workspace) = self
            .packages
            .list
            .iter()
            .find(|package| package.workspace && named(package))
        {
            return Some(workspace);
        }

        let (listing, version) = self.listing_of(importing, name)?;

        if version.starts_with("workspace:") {
            return self.packages.list.iter().find(named);
        }

        let path = LOCAL_PROTOCOLS
            .iter()
            .find_map(|protocol| version.strip_prefix(protocol))?;

        joined(folder_path(&listing.folder), path)
            .and_then(|folder| self.by_folder.get(folder.as_str()).copied())
    }

    /// The nearest package at or above the importing file whose `package.json` lists a package as a
    /// dependency of any kind, with the version it asks for (spec RES-2, RES-8).
    fn listing_of(&self, importing: &str, name: &str) -> Option<(&'p Package, &'p str)> {
        let mut folder = importing;

        loop {
            folder = folder.rsplit_once('/').map_or("", |(above, _)| above);

            if let Some(package) = self.by_folder.get(folder)
                && let Some(version) = package.dependencies.get(name)
            {
                return Some((package, version));
            }

            if folder.is_empty() {
                return None;
            }
        }
    }

    /// The package the importing file belongs to (spec PACK-1).
    fn own_package(&self, importing: &str) -> Option<&'p Package> {
        self.packages
            .of_file
            .get(importing)
            .and_then(|folder| self.by_folder.get(folder_path(folder)).copied())
    }

    /// A `#` specifier, looked up in the `imports` of the importing file's package (spec RES-5).
    fn hash(&self, importing: &str, specifier: &str) -> Found {
        let map = self
            .own_package(importing)
            .and_then(|package| Some((package, package.imports.as_ref()?.as_object()?)));

        match map.and_then(|(package, map)| Some((package, matched(map, specifier)?))) {
            Some((package, (value, star))) => self.targets(importing, package, value, star, true),
            None => Found::Done(Outcome::Unresolved),
        }
    }

    /// An import of a package by its name, the bare name or with a subpath, through its `package.json`
    /// (spec RES-6).
    fn through(&self, importing: &str, package: &'p Package, subpath: &str) -> Found {
        let folder = folder_path(&package.folder);

        if let Some(exports) = &package.exports {
            let wanted = format!(".{subpath}");
            let subpaths = exports
                .as_object()
                .filter(|keys| keys.keys().any(|key| key.starts_with('.')));

            return match subpaths {
                Some(map) => match matched(map, &wanted) {
                    Some((value, star)) => self.targets(importing, package, value, star, false),
                    None => Found::Done(Outcome::Unresolved),
                },
                None if subpath.is_empty() => {
                    self.targets(importing, package, exports, None, false)
                }
                None => Found::Done(Outcome::Unresolved),
            };
        }

        if subpath.is_empty() {
            return self.entries(package).unwrap_or_else(|| self.index(folder));
        }

        match joined(folder, &subpath[1..]) {
            Some(path) => self.target(folder, &path, None),
            None => Found::Done(Outcome::Outside(Place::AboveRoot)),
        }
    }

    /// Every file a package's `types`, `module` and `main` lead to; nothing when it states none of them
    /// (spec RES-3, RES-6).
    fn entries(&self, package: &Package) -> Option<Found> {
        let folder = folder_path(&package.folder);

        (!package.entries.is_empty()).then(|| {
            combined(
                package
                    .entries
                    .iter()
                    .map(|entry| match joined(folder, entry) {
                        Some(path) => self.target(folder, &path, Some(&path)),
                        None => Found::Done(Outcome::Outside(Place::AboveRoot)),
                    })
                    .collect(),
            )
        })
    }

    /// The targets of a matched key of `exports` or `imports`, every condition and every entry of a list
    /// followed, each `*` filled in from the specifier (spec RES-5, RES-6, PACK-6).
    fn targets(
        &self,
        importing: &str,
        package: &'p Package,
        value: &Value,
        star: Option<&str>,
        imports: bool,
    ) -> Found {
        match value {
            Value::String(target) => {
                let target = star.map_or_else(|| target.clone(), |star| target.replace('*', star));
                let folder = folder_path(&package.folder);

                if let Some(inside) = target.strip_prefix("./") {
                    match joined(folder, inside) {
                        Some(path) => self.target(folder, &path, Some(&path)),
                        None => Found::Done(Outcome::Outside(Place::AboveRoot)),
                    }
                } else if imports && !target.starts_with("../") && !target.starts_with('/') {
                    // A name in `imports` may stand for a package, imported by its name.
                    self.bare(importing, &target).0
                } else {
                    Found::Done(Outcome::Unresolved)
                }
            }
            Value::Array(list) => combined(
                list.iter()
                    .map(|each| self.targets(importing, package, each, star, imports))
                    .collect(),
            ),
            Value::Object(conditions) => combined(
                conditions
                    .values()
                    .map(|each| self.targets(importing, package, each, star, imports))
                    .collect(),
            ),
            _ => Found::Done(Outcome::Unresolved),
        }
    }

    /// One target of a `package.json`, or a path inside a package, in the package's folder: the file it
    /// names as RES-3 finds one, or else the source it is built from (spec MAP-1, MAP-2).
    fn target(&self, folder: &str, path: &str, from: Option<&str>) -> Found {
        let mut asked = match self.locate(path, false) {
            Found::Done(outcome) => return Found::Done(outcome),
            Found::Missing(asked) => asked,
        };

        for candidate in sources(folder, path) {
            if self.files.contains(candidate.as_str()) {
                return Found::Done(Outcome::Files(self.with_twin(&candidate)));
            }

            if is_source(&candidate) {
                asked.push(Asked {
                    path: candidate,
                    from: None,
                });
            }
        }

        for each in &mut asked {
            each.from = from.map(str::to_owned);
        }

        Found::Missing(asked)
    }

    /// Looks for a path, relative to the root, as TypeScript finds one, in the file list (spec RES-3).
    /// A folder that holds a `package.json` is tried through its `types`, `module` and `main` first,
    /// where `packaged` allows it.
    fn locate(&self, path: &str, packaged: bool) -> Found {
        if under(path, &self.listing.elsewhere) {
            return Found::Done(Outcome::Outside(Place::OtherRepository));
        }

        if under(path, &self.listing.links) {
            return Found::Done(Outcome::Outside(Place::Link));
        }

        let mut asked = Vec::new();

        if let Some(found) = self
            .by_folder
            .get(path)
            .filter(|_| packaged)
            .and_then(|package| self.entries(package))
        {
            match found {
                Found::Missing(missing) => asked = missing,
                found => return found,
            }
        }

        // Each candidate with whether it is the path as written or a source form, the ones git is asked
        // about when nothing is found.
        let mut candidates: Vec<(String, bool)> = Vec::new();

        if let Some((stem, forms)) = SOURCE_FORMS
            .iter()
            .find_map(|(output, forms)| path.strip_suffix(output).map(|stem| (stem, *forms)))
        {
            candidates.extend(forms.iter().map(|form| (format!("{stem}{form}"), true)));
        }

        if !path.is_empty() {
            candidates.push((path.to_owned(), true));
            candidates.extend(
                ENDINGS
                    .iter()
                    .map(|ending| (format!("{path}{ending}"), is_source(ending))),
            );
        }

        candidates.extend(index_candidates(path));

        match candidates
            .iter()
            .find(|(candidate, _)| self.files.contains(candidate.as_str()))
        {
            Some((found, _)) => Found::Done(Outcome::Files(self.with_twin(found))),
            // The path as written first, so that it is the one named when git ignores it.
            None => Found::Missing(
                asked
                    .into_iter()
                    .chain(
                        std::iter::once(path.to_owned())
                            .chain(
                                candidates
                                    .into_iter()
                                    .filter(|(candidate, asked)| *asked && candidate != path)
                                    .map(|(candidate, _)| candidate),
                            )
                            .filter(|candidate| !candidate.is_empty())
                            .map(|path| Asked { path, from: None }),
                    )
                    .collect(),
            ),
        }
    }

    /// The `index` file of a package's folder, for a package that states no `types`, `module` or
    /// `main` (spec RES-6).
    fn index(&self, folder: &str) -> Found {
        let candidates = index_candidates(folder);

        match candidates
            .iter()
            .find(|(candidate, _)| self.files.contains(candidate.as_str()))
        {
            Some((found, _)) => Found::Done(Outcome::Files(self.with_twin(found))),
            None => Found::Missing(
                candidates
                    .into_iter()
                    .filter(|(_, asked)| *asked)
                    .map(|(path, _)| Asked { path, from: None })
                    .collect(),
            ),
        }
    }

    /// A file found, with the other of its pair where the list holds it: a script and the type
    /// declaration file of the same name (spec RES-3).
    fn with_twin(&self, found: &str) -> Vec<String> {
        let twin = PAIRS.iter().find_map(|(script, declaration)| {
            if let Some(stem) = found.strip_suffix(declaration) {
                Some(format!("{stem}{script}"))
            } else {
                found
                    .strip_suffix(script)
                    .map(|stem| format!("{stem}{declaration}"))
            }
        });
        let mut files = vec![found.to_owned()];

        if let Some(twin) = twin.filter(|twin| self.files.contains(twin.as_str())) {
            files.push(twin);
        }

        files.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        files
    }
}

/// What several targets found together: every file any of them leads to; else every path to ask git
/// about; else the first place outside; else nothing (spec RES-5).
fn combined(each: Vec<Found>) -> Found {
    let mut files: Vec<String> = Vec::new();
    let mut asked = Vec::new();
    let mut outside = None;

    for found in each {
        match found {
            Found::Done(Outcome::Files(found)) => files.extend(found),
            Found::Missing(missing) => asked.extend(missing),
            Found::Done(Outcome::Outside(place)) => {
                outside.get_or_insert(place);
            }
            Found::Done(_) => {}
        }
    }

    if !files.is_empty() {
        files.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        files.dedup();

        Found::Done(Outcome::Files(files))
    } else if !asked.is_empty() {
        Found::Missing(asked)
    } else {
        Found::Done(outside.map_or(Outcome::Unresolved, Outcome::Outside))
    }
}

/// The key of an `exports` or `imports` map that a subpath matches, as Node matches one: the key
/// itself when it has no `*`, or else the pattern with the longest part before its one `*`, the
/// longer key where two are as long; with the text the `*` stands for (spec RES-5, RES-6).
fn matched<'v, 'w>(
    map: &'v Map<String, Value>,
    wanted: &'w str,
) -> Option<(&'v Value, Option<&'w str>)> {
    if let Some(value) = map.get(wanted).filter(|_| !wanted.contains('*')) {
        return Some((value, None));
    }

    let mut best: Option<(&str, &Value, &str)> = None;

    for (key, value) in map {
        let Some((before, after)) = key.split_once('*') else {
            continue;
        };

        if after.contains('*')
            || wanted.len() < key.len()
            || !wanted.starts_with(before)
            || !wanted.ends_with(after)
        {
            continue;
        }

        let better = best.is_none_or(|(chosen, _, _)| {
            let chosen_before = chosen.find('*').map_or(chosen.len(), |at| at + 1);

            before.len() + 1 > chosen_before
                || (before.len() + 1 == chosen_before && key.len() > chosen.len())
        });

        if better {
            best = Some((
                key,
                value,
                &wanted[before.len()..wanted.len() - after.len()],
            ));
        }
    }

    best.map(|(_, value, star)| (value, Some(star)))
}

/// The places MAP-1 looks for the source of build output at a path inside a package's folder, in order:
/// the path with its first folder under the package replaced by `src`, with each source extension in
/// place of its output ending.
fn sources(folder: &str, path: &str) -> Vec<String> {
    let inside = if folder.is_empty() {
        path
    } else {
        path.strip_prefix(folder)
            .and_then(|rest| rest.strip_prefix('/'))
            .unwrap_or(path)
    };
    let Some((_, below)) = inside.split_once('/') else {
        return Vec::new();
    };
    let moved = if folder.is_empty() {
        format!("src/{below}")
    } else {
        format!("{folder}/src/{below}")
    };

    match OUTPUT_ENDINGS
        .iter()
        .find_map(|ending| moved.strip_suffix(ending))
    {
        Some(stem) => SOURCE_ENDINGS
            .iter()
            .map(|ending| format!("{stem}{ending}"))
            .collect(),
        None => vec![moved],
    }
}

/// The `index` files of a folder, in the order they are tried, each with whether git is asked about it.
fn index_candidates(folder: &str) -> Vec<(String, bool)> {
    let index = if folder.is_empty() {
        "index".to_owned()
    } else {
        format!("{folder}/index")
    };

    ENDINGS
        .iter()
        .map(|ending| (format!("{index}{ending}"), is_source(ending)))
        .collect()
}

/// Whether a path, or an ending, is a source form, which git is asked about when nothing is found: a
/// TypeScript ending or a type declaration's (spec RES-4).
fn is_source(path: &str) -> bool {
    [".ts", ".tsx", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending))
}

/// A package's folder as a path from the root: the root's own is the empty path.
fn folder_path(folder: &str) -> &str {
    if folder == "." { "" } else { folder }
}

/// The specifier without a query or a fragment (spec IMP-4).
fn written_path(specifier: &str) -> &str {
    let query = specifier.find('?').unwrap_or(specifier.len());
    let fragment = specifier
        .get(1..)
        .and_then(|rest| rest.find('#'))
        .map_or(specifier.len(), |at| at + 1);

    &specifier[..query.min(fragment)]
}

/// Whether a specifier is relative: `.`, `..`, or one that begins with `./` or `../` (spec RES-2).
fn is_relative(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}

/// Whether a specifier is an absolute path: from the top of a drive or a file system.
fn is_absolute(specifier: &str) -> bool {
    let bytes = specifier.as_bytes();
    let drive = bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');

    specifier.starts_with('/') || specifier.starts_with('\\') || drive
}

/// Whether a specifier is a URL: a scheme of two characters or more, then a colon.
fn is_url(specifier: &str) -> bool {
    specifier.split_once(':').is_some_and(|(scheme, _)| {
        scheme.len() > 1
            && scheme.starts_with(|first: char| first.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character))
    })
}

/// The name of the package a bare specifier names: its first segment, or its first two when it begins
/// with `@` (spec RES-2).
fn package_name(specifier: &str) -> String {
    let segments = if specifier.starts_with('@') { 2 } else { 1 };

    specifier
        .split('/')
        .take(segments)
        .collect::<Vec<_>>()
        .join("/")
}

/// A folder and a relative path joined, with `.` and `..` worked out; nothing when it leads above the
/// root (spec LIST-8). The root itself is the empty path.
fn joined(folder: &str, relative: &str) -> Option<String> {
    let mut segments: Vec<&str> = folder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    for segment in relative.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => segments.push(other),
        }
    }

    Some(segments.join("/"))
}

/// Whether a path is one of the paths given, or lies under one of them.
fn under(path: &str, places: &BTreeSet<String>) -> bool {
    places.iter().any(|place| {
        path.strip_prefix(place.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the imports of every script, and where each import leads.

use std::collections::BTreeMap;
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
use crate::resolve::{Outcome, Place, Resolution, Resolver};
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
    let listing = files::list(&root, &mut notices)?;
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed.as_ref(), &mut notices)?;
    let packages = packages::read(
        &root,
        files,
        &marks.data,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    )?;
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolved = Resolver::new(&listing, &packages).resolve(&root, &scripts)?;
    let document = json!({
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    Ok(format!("{document:#}"))
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, and the
/// count of those that cannot be followed.
fn file(
    file: &File,
    marks: &Marks,
    packages: &Packages,
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
) -> Value {
    let mut printed = json!({
        "data": marks.data.contains(&file.path),
        "generated": marks.generated.contains(&file.path),
        "kind": file.kind.name(),
        "package": packages.of_file.get(&file.path),
        "path": file.path,
        "tracked": file.tracked,
    });

    if let (Some(read), Some(resolutions)) =
        (scripts.read.get(&file.path), resolved.get(&file.path))
    {
        printed["computedImports"] = json!(read.computed);
        printed["imports"] = read
            .imports
            .iter()
            .zip(resolutions)
            .map(|(each, resolution)| import(each, resolution))
            .collect();
    }

    printed
}

/// One import as it is printed, with where it leads and the rule that decided it (spec OUT-1).
fn import(import: &Import, resolution: &Resolution) -> Value {
    let mut printed = json!({
        "form": import.form.name(),
        "line": import.line,
        "query": import.has_query(),
        "rule": resolution.rule.name(),
        "specifier": import.specifier,
        "typeOnly": import.type_only,
    });

    match &resolution.outcome {
        Outcome::Files(files) => {
            printed["outcome"] = json!("files");
            printed["files"] = json!(files);
        }
        Outcome::Outside(place) => {
            printed["outcome"] = json!("outside");
            printed["outside"] = json!(place.name());

            if let Place::Package { name, listed } = place {
                printed["package"] = json!(name);
                printed["listed"] = json!(listed);
            }
        }
        Outcome::Ignored { path, from } => {
            printed["outcome"] = json!("ignored");
            printed["ignored"] = json!(path);

            if let Some(from) = from {
                printed["from"] = json!(from);
            }
        }
        Outcome::Unresolved => printed["outcome"] = json!("unresolved"),
    }

    printed
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

- [ ] **Step 4: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
```

Expected: each exits 0, and the tests report 12 passed for `resolve`, the other counts as in Task 1.

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat(resolve): lead imports by a package's name and by # names to source" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: one commit on `feat/reading-4-resolve`.

### Task 3: tsconfig files

**Implements:** CFG-1, CFG-2, CFG-3, CFG-4, RES-2, RES-4, RES-9, MAP-1, MAP-3, LIST-4, LIST-9, RUN-4, RUN-5, DEP-1, DEP-3, FIX-2, FIX-3, FIX-4

**Test:** `cargo test --release --locked --test tsconfig && node --test scripts/dependabot.test.mjs`

**Files:**
- Create: `src/tsconfig.rs`
- Modify: `src/git.rs`, `src/stop.rs`, `src/packages.rs`, `src/resolve.rs`, `src/graph.rs`,
  `src/main.rs`, `Cargo.toml`, `Cargo.lock`, `.github/dependabot.yml`
- Test: `tests/tsconfig.test.rs`, `scripts/dependabot.test.mjs`

**Interfaces:**
- Consumes: Task 2's resolver, `packages::Packages` and `packages::misshapen`; `git::read` and
  `git::ignored`.
- Produces: `tsconfig::load`, `tsconfig::Loaded::checked`, `tsconfig::Configs` (`governing`, `paths`,
  `base_url`, `builds`), `tsconfig::Build` and `tsconfig::Mapped`; `packages::load` and
  `packages::Loaded::checked`; `stop::Fault`; `git::above` and `git::Above`; `resolve::Rule::Paths`
  and `Rule::BaseUrl`. The fifth plan adds the files it reads to CFG-4's list in `tsconfig::load`.

- [ ] **Step 1: Write the failing tests**

```rust path=tests/tsconfig.test.rs
//! Runs the built binary on made-up projects and checks how tsconfig files are read and what they do to
//! where imports lead: the config that governs a file, `paths`, `baseUrl`, `extends` above the root and
//! outside the project, and how a run stops on one it cannot read (spec CFG-1 to CFG-4, RES-2, RES-4,
//! MAP-1, MAP-3, LIST-4, LIST-8, RUN-4, FIX-2, FIX-3, FIX-4).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// What an import leads to: its outcome and rule, and the files, the place or the ignored path that go
/// with it.
fn led(document: &Value, path: &str, specifier: &str) -> Value {
    let import = document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .find(|file| file["path"] == json!(path))
        .and_then(|file| file["imports"].as_array())
        .and_then(|imports| {
            imports
                .iter()
                .find(|import| import["specifier"] == json!(specifier))
        })
        .unwrap_or_else(|| panic!("{path} imports {specifier}"));
    let mut led = json!({ "outcome": import["outcome"], "rule": import["rule"] });

    for key in ["files", "outside", "package", "listed", "ignored", "from"] {
        if let Some(value) = import.get(key) {
            led[key] = value.clone();
        }
    }

    led
}

/// An import that leads to the files given, by the rule given.
fn files(rule: &str, found: &[&str]) -> Value {
    json!({ "outcome": "files", "rule": rule, "files": found })
}

/// An import by the rule given that leads to a package outside the project.
fn package(rule: &str, name: &str, listed: bool) -> Value {
    json!({ "outcome": "outside", "rule": rule, "outside": "package", "package": name, "listed": listed })
}

#[test]
fn governs_a_file_by_the_nearest_config_that_covers_it_or_one_it_references() {
    let project = Project::new("tsconfig-governs");

    project
        .write(
            "tsconfig.json",
            r#"{
                "files": [],
                "references": [
                    { "path": "./tsconfig.app.json" },
                    { "path": "./tsconfig.node.json" },
                    { "path": "./tsconfig.missing.json" },
                ],
            }"#,
        )
        .write(
            "tsconfig.app.json",
            "// The app's own config.\n{ \"include\": [\"src\"], /* aliases */ \"compilerOptions\": { \"paths\": { \"@/*\": [\"./src/*\"] } } }\n",
        )
        .write(
            "tsconfig.node.json",
            r#"{ "include": ["scripts"], "compilerOptions": { "baseUrl": "." } }"#,
        )
        .write("src/lib/util.ts", "")
        .write("src/main.ts", "import '@/lib/util';\n")
        .write("src/legacy.js", "import '@/lib/util';\n")
        .write("scripts/build.ts", "import 'src/lib/util';\nimport 'src/lib/missing';\n")
        .write("other/x.ts", "import '@/lib/util';\n")
        .write("web/jsconfig.json", r#"{ "compilerOptions": { "paths": { "~/*": ["./*"] } } }"#)
        .write("web/util.js", "")
        .write("web/app.js", "import '~/util';\n")
        .commit();

    let document = project.graph();

    // A config that `references` names governs the files it covers (spec CFG-2).
    assert_eq!(
        led(&document, "src/main.ts", "@/lib/util"),
        files("paths", &["src/lib/util.ts"])
    );
    // A file that is not TypeScript is covered as a `.ts` file of its name would be.
    assert_eq!(
        led(&document, "src/legacy.js", "@/lib/util"),
        files("paths", &["src/lib/util.ts"])
    );
    // A path from `baseUrl` that leads to no file goes on down the order (spec RES-2, step 4).
    assert_eq!(
        led(&document, "scripts/build.ts", "src/lib/util"),
        files("base-url", &["src/lib/util.ts"])
    );
    assert_eq!(
        led(&document, "scripts/build.ts", "src/lib/missing"),
        package("outside", "src", false)
    );
    // A file no config covers has no `paths`.
    assert_eq!(
        led(&document, "other/x.ts", "@/lib/util"),
        package("outside", "@/lib", false)
    );
    // A `jsconfig.json` governs as a `tsconfig.json` does.
    assert_eq!(
        led(&document, "web/app.js", "~/util"),
        files("paths", &["web/util.js"])
    );
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "tsconfig-reference-missing"]
    );
}

#[test]
fn sends_an_import_a_paths_pattern_cannot_settle_on_down_the_order() {
    let project = Project::new("tsconfig-paths-miss");

    project
        .write(".gitignore", "src/generated/\n")
        .write(
            "package.json",
            r#"{ "name": "root", "workspaces": ["packages/*"], "dependencies": { "@scope/installed": "^1.0.0" } }"#,
        )
        .write("packages/ui/package.json", r#"{ "name": "@scope/ui" }"#)
        .write("packages/ui/index.ts", "")
        .write(
            "tsconfig.json",
            r#"{
                "compilerOptions": {
                    "paths": {
                        "@app/*": ["./src/app/*"],
                        "@gen/*": ["./src/generated/*"],
                        "@scope/*": ["./src/scoped/*"],
                        "*": ["./types/*"]
                    }
                }
            }"#,
        )
        .write("src/app/found.ts", "")
        .write("types/lodash.d.ts", "")
        .write(
            "src/main.ts",
            "import '@app/found';\nimport '@app/missing';\nimport '@gen/api';\nimport '@scope/missing';\n\
             import '@scope/ui';\nimport '@scope/installed';\nimport 'lodash';\nimport 'react';\n",
        )
        .commit();

    let document = project.graph();
    let main = |specifier: &str| led(&document, "src/main.ts", specifier);

    assert_eq!(main("@app/found"), files("paths", &["src/app/found.ts"]));
    // A pattern that matches and leads to no file is unresolved, or ignored (spec RES-4).
    assert_eq!(
        main("@app/missing"),
        json!({ "outcome": "unresolved", "rule": "paths" })
    );
    assert_eq!(
        main("@gen/api"),
        json!({ "outcome": "ignored", "rule": "paths", "ignored": "src/generated/api" })
    );
    assert_eq!(
        main("@scope/missing"),
        json!({ "outcome": "unresolved", "rule": "paths" })
    );
    // A workspace package's name, and a dependency's, go on down the order.
    assert_eq!(
        main("@scope/ui"),
        files("package", &["packages/ui/index.ts"])
    );
    assert_eq!(
        main("@scope/installed"),
        package("outside", "@scope/installed", true)
    );
    // So does whatever the bare `*` cannot settle.
    assert_eq!(main("lodash"), files("paths", &["types/lodash.d.ts"]));
    assert_eq!(main("react"), package("outside", "react", false));
}

#[test]
fn maps_one_alias_to_source_by_paths_and_build_output_back_by_out_dir() {
    let project = Project::new("tsconfig-alias");

    project
        .write(".gitignore", "dist/\nout/\n")
        .write(
            "package.json",
            r##"{ "name": "app", "workspaces": ["packages/*"], "imports": { "#lib/*": "./dist/lib/*.js" } }"##,
        )
        .write(
            "tsconfig.json",
            r##"{ "include": ["src"], "compilerOptions": { "outDir": "dist", "paths": { "#lib/*": ["./src/lib/*"] } } }"##,
        )
        .write("src/lib/a.ts", "")
        .write("src/lib/b.ts", "")
        .write(
            "src/main.ts",
            "import '#lib/a';\nimport '../dist/lib/b.js';\nimport 'legacy';\n",
        )
        .write("scripts/run.mjs", "import '#lib/a';\n")
        .write(
            "packages/legacy/package.json",
            r#"{ "name": "legacy", "main": "./out/index.js" }"#,
        )
        .write(
            "packages/legacy/tsconfig.json",
            r#"{ "extends": "./tsconfig.base.json" }"#,
        )
        .write(
            "packages/legacy/tsconfig.base.json",
            r#"{ "compilerOptions": { "outDir": "out", "rootDir": "source" } }"#,
        )
        .write("packages/legacy/source/index.ts", "")
        .commit();

    let document = project.graph();

    // One alias: source by the tsconfig's `paths`, which comes first (spec FIX-2, pattern 9).
    assert_eq!(
        led(&document, "src/main.ts", "#lib/a"),
        files("paths", &["src/lib/a.ts"])
    );
    // Build output by `imports`, for a file the tsconfig does not cover, mapped back by its `outDir`.
    assert_eq!(
        led(&document, "scripts/run.mjs", "#lib/a"),
        files("imports", &["src/lib/a.ts"])
    );
    // A relative import into the build folder (spec MAP-1, step 1).
    assert_eq!(
        led(&document, "src/main.ts", "../dist/lib/b.js"),
        files("relative", &["src/lib/b.ts"])
    );
    // An `outDir` and a `rootDir` taken through `extends` (spec MAP-3).
    assert_eq!(
        led(&document, "src/main.ts", "legacy"),
        files("package", &["packages/legacy/source/index.ts"])
    );
}

#[test]
fn reads_a_config_above_a_sub_folder_root_that_one_under_it_extends() {
    let project = Project::new("tsconfig-above");

    project
        .write(
            "tsconfig.base.json",
            r#"{ "compilerOptions": { "paths": { "@shared/*": ["./shared/*"] } } }"#,
        )
        .write(
            "apps/web/tsconfig.json",
            r#"{ "extends": "../../tsconfig.base.json", "include": ["src"] }"#,
        )
        .write(
            "apps/web/src/main.ts",
            "import '@shared/util';\nimport '../../../shared/util';\n",
        )
        .write("shared/util.ts", "")
        .commit();

    // From the repository's root, both lead to the shared file (spec FIX-2, pattern 15).
    let document = project.graph();

    assert_eq!(
        led(&document, "apps/web/src/main.ts", "@shared/util"),
        files("paths", &["shared/util.ts"])
    );
    assert_eq!(
        led(&document, "apps/web/src/main.ts", "../../../shared/util"),
        files("relative", &["shared/util.ts"])
    );

    // From the sub-folder, the config above it is read, and both lead above the root (spec LIST-8).
    let root = project.root.join("apps").join("web");
    let file = project.hand_off(&root, &["."], json!([]));
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));
    let above = |rule: &str| json!({ "outcome": "outside", "rule": rule, "outside": "above-root" });

    // Nothing above the root is listed, the config read there included.
    assert_eq!(paths(&document), ["src/main.ts", "tsconfig.json"]);

    assert_eq!(
        led(&document, "src/main.ts", "@shared/util"),
        above("paths")
    );
    assert_eq!(
        led(&document, "src/main.ts", "../../../shared/util"),
        above("relative")
    );
    assert_eq!(notices(&document), Vec::<&str>::new());
}

#[test]
fn uses_a_config_without_the_config_of_a_dependency_it_extends() {
    let project = Project::new("tsconfig-dependency");

    project
        .write(".gitignore", "node_modules/\nbuild/\n")
        .write(
            "tsconfig.json",
            r#"{ "extends": "@tsconfig/node22/tsconfig.json", "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#,
        )
        .write("src/a.ts", "")
        .write("src/main.ts", "import '@/a';\nimport './build/out.js';\n")
        .commit();

    let run = |label: &str| {
        let document = project.graph();

        assert_eq!(
            led(&document, "src/main.ts", "@/a"),
            files("paths", &["src/a.ts"]),
            "{label}"
        );
        assert_eq!(
            led(&document, "src/main.ts", "./build/out.js"),
            json!({ "outcome": "ignored", "rule": "relative", "ignored": "src/build/out.js" }),
            "{label}"
        );
        assert_eq!(
            document["notices"][1],
            json!({
                "file": "tsconfig.json",
                "kind": "tsconfig-extends-left-out",
                "sentence": "The tsconfig extends `@tsconfig/node22/tsconfig.json`, which is not in the project, so it is \
                             used without it.",
            }),
            "{label}"
        );
    };

    // The same with and without the dependency's config and the build output on disk (spec FIX-3).
    run("without node_modules");
    project
        .write(
            "node_modules/@tsconfig/node22/tsconfig.json",
            r#"{ "compilerOptions": { "paths": { "@/*": ["./elsewhere/*"] } } }"#,
        )
        .write("src/build/out.js", "");
    run("with node_modules");
}

#[test]
fn reads_no_tsconfig_declared_as_data() {
    let project = Project::new("tsconfig-data");

    project
        .write("fixtures/broken/tsconfig.json", "{ not json")
        .write("fixtures/broken/case.ts", "import '@/x';\n")
        .write(
            "fixtures/working/tsconfig.json",
            r#"{ "compilerOptions": { "paths": { "@/*": ["./*"] } } }"#,
        )
        .write("fixtures/working/case.ts", "")
        .write("src/main.ts", "import '../fixtures/working/case';\n")
        .commit();

    let declared = json!([{
        "name": "typescript",
        "interfaceVersion": 1,
        "pack": {},
        "project": { "data": ["fixtures/**"] },
    }]);
    let file = project.hand_off(&project.root, &["."], declared);
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // Data is never read, so a broken config among it stops nothing (spec FIX-2, pattern 5, KEY-4).
    assert_eq!(
        led(&document, "src/main.ts", "../fixtures/working/case"),
        files("relative", &["fixtures/working/case.ts"])
    );
    assert_eq!(notices(&document), Vec::<&str>::new());
}

#[test]
fn stops_on_a_config_a_file_depends_on_that_cannot_be_read_and_skips_any_other() {
    let project = Project::new("tsconfig-unreadable");

    project
        .write("tsconfig.json", "{ \"include\": [\"src\"] }")
        .write("tsconfig.lint.json", "{ broken")
        .write("src/a.ts", "")
        .commit();

    // A config no file depends on is left out, with a notice (spec CFG-4).
    let document = project.graph();

    assert_eq!(
        document["notices"][1],
        json!({
            "file": "tsconfig.lint.json",
            "kind": "tsconfig-skipped",
            "sentence": "The tsconfig cannot be read (line 1, column 3), and no file the analyzer reads depends on it, \
                         so it is left out.",
        })
    );

    // The config in a folder above a file the analyzer reads.
    project.write("tsconfig.json", "{ \"include\": [\"src\"], }}");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig tsconfig.json cannot be read (line 1, column 24); fix it, or declare it as data if \
         it is broken on purpose"
    );

    // A config it extends.
    project
        .write("tsconfig.json", r#"{ "extends": "./tsconfig.lint.json" }"#)
        .write("tsconfig.lint.json", "{ broken");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig tsconfig.lint.json cannot be read (line 1, column 3); fix it, or declare it as \
         data if it is broken on purpose"
    );

    // A file it extends that is not there.
    project.write("tsconfig.json", r#"{ "extends": "./base" }"#);

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig tsconfig.json extends `./base`, which should be the file base and is not there; \
         fix its `extends`, or add the file"
    );
}

#[test]
fn stops_on_a_config_field_of_a_form_it_does_not_have_after_any_config_it_cannot_read() {
    let project = Project::new("tsconfig-fields");

    project
        .write("package.json", r#"{ "main": 1 }"#)
        .write("a/tsconfig.json", r#"{ "include": "src" }"#)
        .write("a/src/x.ts", "")
        .commit();

    // Of two fields of a form they do not have, the first by path: `a/tsconfig.json` before
    // `package.json` (spec RUN-4).
    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `include` in a/tsconfig.json holds a value of a form it does not have; it must be a \
         list of paths; fix it"
    );

    // A config that cannot be read is reported before any field (spec RUN-4, step 10).
    project.write("b/tsconfig.json", "{").write("b/y.ts", "");

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the tsconfig b/tsconfig.json cannot be read (line 1, column 1); fix it, or declare it as data \
         if it is broken on purpose"
    );

    let cases = [
        (
            r#"{ "extends": 1 }"#,
            "extends",
            "a path, or a list of paths",
        ),
        (r#"{ "files": [1] }"#, "files", "a list of paths"),
        (r#"{ "exclude": {} }"#, "exclude", "a list of paths"),
        (
            r#"{ "references": ["./a"] }"#,
            "references",
            "a list of objects, each with a `path`",
        ),
        (
            r#"{ "compilerOptions": [] }"#,
            "compilerOptions",
            "an object",
        ),
        (
            r#"{ "compilerOptions": { "paths": { "@/*": "./src/*" } } }"#,
            "compilerOptions.paths",
            "an object that maps each pattern to a list of paths",
        ),
        (
            r#"{ "compilerOptions": { "baseUrl": true } }"#,
            "compilerOptions.baseUrl",
            "a path",
        ),
        (
            r#"{ "compilerOptions": { "outDir": 1 } }"#,
            "compilerOptions.outDir",
            "a path",
        ),
        (
            r#"{ "compilerOptions": { "rootDir": null } }"#,
            "compilerOptions.rootDir",
            "a path",
        ),
    ];

    for (config, field, form) in cases {
        let project = Project::new("tsconfig-field-form");

        project.write("tsconfig.json", config).commit();

        assert_eq!(
            stopped(&project.run(|_| {})),
            format!(
                "yak-analyzer: the field `{field}` in tsconfig.json holds a value of a form it does not have; it must \
                 be {form}; fix it"
            ),
            "{config}"
        );
    }
}
```

```js path=scripts/dependabot.test.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT } from './lib/run.mjs';

/** A text file of the repository, with line feeds whatever the checkout wrote. */
const read = (name) => readFileSync(join(ROOT, name), 'utf8').replaceAll('\r\n', '\n');

/** The resolver is released on its own, versioned apart from the other oxc crates, and needs none of them. */
const ALONE = 'oxc_resolver';

test('the oxc crates are pinned to one version, since they are released together (spec AN-2, DEP-3)', () => {
  const pins = [...read('Cargo.toml').matchAll(/^(?<crate>oxc_[a-z_]+) = "=(?<version>[^"]+)"$/gmu)].filter(
    (pin) => pin.groups.crate !== ALONE,
  );

  assert.ok(pins.length > 0);
  assert.equal(new Set(pins.map((pin) => pin.groups.version)).size, 1);
});

test('Dependabot moves the oxc crates together, in one pull request, and the resolver alone (spec DEP-3)', () => {
  const dependabot = read('.github/dependabot.yml');

  assert.match(
    dependabot,
    /- package-ecosystem: cargo\n(?: {4}.*\n)* {4}groups:\n {6}oxc:\n {8}patterns:\n {10}- "oxc_\*"\n {8}exclude-patterns:\n {10}- "oxc_resolver"\n/u,
  );
});
```

- [ ] **Step 2: Run them and see them fail**

```bash
cargo test --release --locked --test tsconfig && node --test scripts/dependabot.test.mjs
```

Expected: cargo refuses, because the manifest names no test called `tsconfig` yet, so the Node test
does not run. Run alone, its test of the pins passes, since `oxc_resolver` is not pinned yet, and its
test of the group fails, because the group names no exclusion.

- [ ] **Step 3: Name the test, add the two crates, and move the resolver alone**

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

[[test]]
name = "resolve"
path = "tests/resolve.test.rs"

[[test]]
name = "tsconfig"
path = "tests/tsconfig.test.rs"

[dependencies]
# Parse scripts into syntax trees and walk them; released together, so pinned together (spec AN-2, DEP-3).
oxc_allocator = "=0.156.0"
oxc_ast = "=0.156.0"
oxc_ast_visit = "=0.156.0"
oxc_parser = "=0.156.0"
oxc_span = "=0.156.0"
# Finds the tsconfig that governs a file, through its `references` and `extends`, with its `paths` and
# `baseUrl`, from a file system of the analyzer's own (spec RES-9).
oxc_resolver = "=11.24.3"
# Reads a tsconfig, which may hold comments and trailing commas (spec CFG-1); the version oxc_resolver
# itself uses.
json-strip-comments = "=3.1.2"
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
name = "arrayvec"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3fb67a6e08acf24fdeccbac2cb6ac4305825bd1f117462e0e6f2f193345ad56"

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
name = "crossbeam-utils"
version = "0.8.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a31eee39dddec8330830986fcd7625edb5a24ec90ea038215273bbc3adb08ac6"

[[package]]
name = "dashmap"
version = "6.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6361d5c062261c78a176addb82d4c821ae42bed6089de0e12603cd25de2059c"
dependencies = [
 "cfg-if",
 "crossbeam-utils",
 "hashbrown 0.14.5",
 "lock_api",
 "once_cell",
 "parking_lot_core",
]

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
name = "errno"
version = "0.3.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e35d6e78f4cebb90e51d543db8fe41528851d6deeef6ef21ebd4bfdc1e784776"
dependencies = [
 "libc",
 "windows-sys",
]

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
name = "fast-glob"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2a044ed96eb57253e806d08b74260638af189687e583f04947a5e11e75e0b0d1"
dependencies = [
 "arrayvec",
]

[[package]]
name = "fastrand"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da7c62ceae207dd37ea5b845da6a0696c799f85e97da1ab5b7910be3c1c80223"

[[package]]
name = "float-cmp"
version = "0.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b09cf3155332e944990140d967ff5eceb70df778b34f77d8075db46e4704e6d8"
dependencies = [
 "num-traits",
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
name = "halfbrown"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c7ed2f2edad8a14c8186b847909a41fbb9c3eafa44f88bd891114ed5019da09"
dependencies = [
 "hashbrown 0.16.1",
]

[[package]]
name = "hashbrown"
version = "0.14.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5274423e17b7c9fc20b6e7e208532f9b19825d82dfd615708b70edd83df41f1"

[[package]]
name = "hashbrown"
version = "0.16.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "841d1cc9bed7f9236f321df977030373f4a4163ae1a7dbfe1a51a2c1a51d9100"
dependencies = [
 "allocator-api2",
 "equivalent",
 "foldhash",
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
 "hashbrown 0.17.1",
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
name = "indexmap"
version = "2.14.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc4e190f5d26ca7051642629da2c52fc03bde85a03197c99408dcd291734c855"
dependencies = [
 "equivalent",
 "hashbrown 0.17.1",
 "serde",
 "serde_core",
]

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
name = "json-strip-comments"
version = "3.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "01ed659c5f428031b9c1fb7de6729af2eb491654d4bcd416a5b5b704986b9658"
dependencies = [
 "memchr",
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
name = "linux-raw-sys"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"

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
name = "nodejs-built-in-modules"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a5eb86a92577833b75522336f210c49d9ebd7dd55a44d80a92e68c668a75f27c"

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
 "hashbrown 0.17.1",
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
name = "oxc_resolver"
version = "11.24.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c7fd0839a5ecee17c0d2c88c8006e8ab4dff25d4a79e3a0bf7a9f0cce17f4e3"
dependencies = [
 "compact_str",
 "dashmap",
 "fast-glob",
 "indexmap",
 "json-strip-comments",
 "memchr",
 "nodejs-built-in-modules",
 "once_cell",
 "percent-encoding",
 "rustc-hash",
 "rustix",
 "self_cell",
 "serde",
 "serde_json",
 "simd-json",
 "simdutf8",
 "thiserror",
 "tracing",
 "windows",
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
 "hashbrown 0.17.1",
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
name = "pin-project-lite"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd"

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
 "hashbrown 0.17.1",
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
name = "rustix"
version = "1.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5e76742e5294153d4dde1207dbec5bdd81eef6fabdc0d2118babadabfd858aa"
dependencies = [
 "bitflags",
 "errno",
 "libc",
 "linux-raw-sys",
 "windows-sys",
]

[[package]]
name = "rustversion"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"

[[package]]
name = "ryu"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9774ba4a74de5f7b1c1451ed6cd5285a32eddb5cccb8cc655a4e50009e06477f"

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
name = "self_cell"
version = "1.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ab42ca02749e120097e328d91d415325bdf43b1c72c4c8badf37375fe40a813"

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
 "foldhash",
 "indexmap",
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "simd-json"
version = "0.18.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ebe141eb4a23ecc4d5de93fa5b139ac65dc07c38037a2fe9a8fb7c9d36bd832"
dependencies = [
 "halfbrown",
 "ref-cast",
 "simdutf8",
 "value-trait",
]

[[package]]
name = "simdutf8"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3a9fe34e3e7a50316060351f37187a3f546bce95496156754b601a5fa71b76e"

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
name = "thiserror"
version = "2.0.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09e52cb86a36cede5cb101bf8908837b3e4c6e5e59fe7fd85c23fb56200d189e"
dependencies = [
 "thiserror-impl",
]

[[package]]
name = "thiserror-impl"
version = "2.0.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fe5197923287db20a58125f0bc85c062f7f2c892de97b18c356f9efb14b28524"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.10",
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
name = "tracing"
version = "0.1.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100"
dependencies = [
 "pin-project-lite",
 "tracing-attributes",
 "tracing-core",
]

[[package]]
name = "tracing-attributes"
version = "0.1.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7490cfa5ec963746568740651ac6781f701c9c5ea257c58e057f3ba8cf69e8da"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "tracing-core"
version = "0.1.36"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "db97caf9d906fbde555dd62fa95ddba9eecfd14cb388e4f491a66d74cd5fb79a"
dependencies = [
 "once_cell",
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
name = "value-trait"
version = "0.12.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "545a3d7089f11f1741d12963c057adf9443fc24325110a6a6daf582e825fd51b"
dependencies = [
 "float-cmp",
 "halfbrown",
 "itoa",
 "ryu",
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
name = "windows"
version = "0.62.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "527fadee13e0c05939a6a05d5bd6eec6cd2e3dbd648b9f8e447c6518133d8580"
dependencies = [
 "windows-collections",
 "windows-core",
 "windows-future",
 "windows-numerics",
]

[[package]]
name = "windows-collections"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b2d95af1a8a14a3c7367e1ed4fc9c20e0a26e79551b1454d72583c97cc6610"
dependencies = [
 "windows-core",
]

[[package]]
name = "windows-core"
version = "0.62.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb"
dependencies = [
 "windows-implement",
 "windows-interface",
 "windows-link",
 "windows-result",
 "windows-strings",
]

[[package]]
name = "windows-future"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e1d6f90251fe18a279739e78025bd6ddc52a7e22f921070ccdc67dde84c605cb"
dependencies = [
 "windows-core",
 "windows-link",
 "windows-threading",
]

[[package]]
name = "windows-implement"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "windows-interface"
version = "0.59.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "windows-numerics"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e2e40844ac143cdb44aead537bbf727de9b044e107a0f1220392177d15b0f26"
dependencies = [
 "windows-core",
 "windows-link",
]

[[package]]
name = "windows-result"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-strings"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-sys"
version = "0.61.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-threading"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3949bd5b99cafdf1c7ca86b43ca564028dfe27d66958f2470940f73d86d75b37"
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
 "json-strip-comments",
 "jsonschema",
 "oxc_allocator",
 "oxc_ast",
 "oxc_ast_visit",
 "oxc_parser",
 "oxc_resolver",
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
    # DEP-3). oxc_resolver is released on its own and depends on none of them, so it moves alone.
    groups:
      oxc:
        patterns:
          - "oxc_*"
        exclude-patterns:
          - "oxc_resolver"
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: rust-toolchain
    directory: /
    schedule:
      interval: weekly
```

- [ ] **Step 4: Read the tsconfig files, and resolve through them**

```rust path=src/stop.rs
//! How a run ends when it cannot go on: one line on standard error and an exit code (spec RUN-2, RUN-3).

use std::fmt::Write as _;

/// The exit code of a run that could not go on (spec RUN-3).
pub const STOPPED: u8 = 1;

/// The exit code for a command or an option the binary does not have (spec RUN-2).
pub const REFUSED: u8 = 2;

/// The exit code of a fault in the analyzer itself (spec RUN-3).
pub const FAULT: u8 = 3;

/// The name every line on standard error begins with.
const NAME: &str = "yak-analyzer";

/// A run that cannot go on, as the one line that says what failed and what fixes it.
#[derive(Debug)]
pub struct Stop(String);

impl Stop {
    /// A stop whose line is the text given, without the program's name.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The line for standard error.
    pub fn line(&self) -> String {
        line(&self.0)
    }
}

/// One line for standard error: the program's name, then the text with every line break and control
/// character escaped, so that one line stays one line whatever a file is named.
pub fn line(text: &str) -> String {
    let mut escaped = String::with_capacity(NAME.len() + 2 + text.len());

    escaped.push_str(NAME);
    escaped.push_str(": ");

    for character in text.chars() {
        if character.is_control() {
            // Writing into a String cannot fail.
            let _ = write!(escaped, "{}", character.escape_default());
        } else {
            escaped.push(character);
        }
    }

    escaped
}

/// A stop of RUN-4's eleventh step, a field of a form it does not have, with the file it is about: where
/// several files fail that step, the first by path is the one reported (spec RUN-4).
#[derive(Debug)]
pub struct Fault {
    /// The file, by its path from the root.
    file: String,
    /// The stop.
    stop: Stop,
}

impl Fault {
    /// A fault about the file given.
    pub fn new(file: impl Into<String>, stop: Stop) -> Self {
        Self {
            file: file.into(),
            stop,
        }
    }

    /// The first of the faults by path, compared byte by byte; of one file's, the first found.
    pub fn first(faults: impl IntoIterator<Item = Self>) -> Option<Self> {
        faults.into_iter().reduce(Self::earlier)
    }

    /// The earlier of two faults by path; the first given when both are about one file.
    pub fn earlier(self, other: Self) -> Self {
        if other.file < self.file { other } else { self }
    }

    /// The stop.
    pub fn stop(self) -> Stop {
        self.stop
    }
}
```

```rust path=src/git.rs
//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

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
    run(root, command, None)
}

/// Asks git which of the paths given, relative to the root, it ignores (spec LIST-5). It answers from
/// its patterns, whether or not a path is on disk. A path inside a submodule must not be given: git
/// stops on one.
pub fn ignored(root: &Path, paths: &[&str]) -> Result<Vec<String>, Stop> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let input: Vec<u8> = paths
        .iter()
        .flat_map(|path| path.bytes().chain([0]))
        .collect();
    let answer = run(
        root,
        &[
            "check-ignore",
            "-z",
            "--stdin",
            "--verbose",
            "--non-matching",
        ],
        Some(&input),
    )?;
    let fields: Vec<&[u8]> = answer.output.split(|byte| *byte == 0).collect();

    // Each path comes back as four fields: the file of the pattern that decided, its line, the pattern,
    // and the path. A path no pattern matched has the first three empty; a pattern that begins with `!`
    // says the path is not ignored.
    Ok(fields
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|answer| !answer[0].is_empty() && !answer[2].starts_with(b"!"))
        .map(|answer| String::from_utf8_lossy(answer[3]).into_owned())
        .collect())
}

/// What git says of a path above the root, given from the root with `..` (spec LIST-4, CFG-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Above {
    /// A file of the work tree that git does not ignore: tracked, or untracked and not ignored.
    File,
    /// A path git ignores.
    Ignored,
    /// A path of the work tree that is neither: nothing is there.
    Missing,
    /// A path above the top of the work tree.
    OutsideWorkTree,
}

/// Asks git about one path above the root, as the file list asks about every path under it (spec
/// LIST-4).
pub fn above(root: &Path, path: &str) -> Result<Above, Stop> {
    let prefix = read(root, &["rev-parse", "--show-prefix"])?;
    let depth = String::from_utf8_lossy(&prefix.output)
        .trim()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .count();
    let up = path
        .split('/')
        .take_while(|segment| *segment == "..")
        .count();

    if up > depth {
        return Ok(Above::OutsideWorkTree);
    }

    let pathspec = format!(":(literal){path}");
    let listed = read(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            &pathspec,
        ],
    )?;

    if !listed.output.is_empty() {
        Ok(Above::File)
    } else if ignored(root, &[path])?.is_empty() {
        Ok(Above::Missing)
    } else {
        Ok(Above::Ignored)
    }
}

/// Runs one git command in the root, with what it reads on standard input where there is any. With
/// input, exit code 1 is an answer too: `check-ignore` gives it when no path is ignored.
fn run(root: &Path, command: &[&str], input: Option<&[u8]>) -> Result<Answer, Stop> {
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
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    let (oldest_major, oldest_minor) = OLDEST;
    let started = |error: std::io::Error| {
        Stop::new(format!(
            "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
             {oldest_minor} or newer, which must be on the PATH"
        ))
    };
    let mut child = git.spawn().map_err(started)?;

    // The input is written from a thread of its own, so that git never waits on a full output pipe
    // while the analyzer waits to finish writing.
    let writer = match (input, child.stdin.take()) {
        (Some(input), Some(mut stdin)) => {
            let input = input.to_vec();

            Some(thread::spawn(move || stdin.write_all(&input)))
        }
        _ => None,
    };
    let ran = child.wait_with_output().map_err(started)?;

    // git stops reading when it fails, which breaks the pipe; its exit code says what happened.
    if let Some(writer) = writer {
        let _ = writer.join();
    }

    let answered = ran.status.success() || (input.is_some() && ran.status.code() == Some(1));
    let said = String::from_utf8_lossy(&ran.stderr);
    let first = said.lines().find(|line| !line.trim().is_empty());

    if answered {
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
use crate::stop::{Fault, Stop};

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
    /// The packages its four lists of dependencies name, each with the version it asks for.
    pub dependencies: BTreeMap<String, String>,
    /// Its `exports`, which say what an import of the package by name may reach (spec RES-6).
    pub exports: Option<Value>,
    /// Its `imports`, the names that begin with `#` (spec RES-5).
    pub imports: Option<Value>,
    /// The targets of its `types` (or `typings`), `module` and `main`, in that order, those it states.
    pub entries: Vec<String>,
}

/// What a `package.json` states that a package keeps.
struct Fields {
    name: Option<String>,
    dependencies: BTreeMap<String, String>,
    exports: Option<Value>,
    imports: Option<Value>,
    entries: Vec<String>,
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

/// The `package.json` files that make packages and the workspace's list, read but not yet checked field
/// by field, which RUN-4 asks for later, after the tsconfig files are read (spec RUN-4, steps 8 to 11).
pub struct Loaded {
    /// Each `package.json` by the folder it lies in, in the order of the list.
    read: Vec<(String, Map<String, Value>)>,
    /// The workspace's list that pnpm reads, where the list holds one.
    pnpm: Option<Yaml>,
}

/// Reads every `package.json` that makes a package, then the workspace's list. `data` is the files
/// declared as data, whose `package.json` makes no package (spec KEY-4).
pub fn load(root: &Path, files: &[File], data: &BTreeSet<String>) -> Result<Loaded, Stop> {
    let manifests: Vec<&File> = files
        .iter()
        .filter(|file| file.kind == Kind::Manifest && !data.contains(&file.path))
        .collect();

    // Each `package.json` is read first, then the workspace's list, so that two faults always give the
    // same line (spec RUN-4).
    let mut read: Vec<(String, Map<String, Value>)> = Vec::with_capacity(manifests.len());

    for manifest in &manifests {
        read.push((folder_of(&manifest.path), object(root, &manifest.path)?));
    }

    let pnpm = if files.iter().any(|file| file.path == PNPM_LIST) && !data.contains(PNPM_LIST) {
        Some(yaml(root)?)
    } else {
        None
    };

    Ok(Loaded { read, pnpm })
}

impl Loaded {
    /// Checks the fields the analyzer reads and makes the packages. `detection` is the folders yak's
    /// detection looked in, with a hand-off file. A field of a form it does not have is a fault of the
    /// file that holds it (spec PACK-6).
    pub fn checked(
        self,
        files: &[File],
        detection: Option<&[String]>,
        notices: &mut Vec<Notice>,
    ) -> Result<Packages, Fault> {
        let mut faults = Vec::new();
        let mut names: BTreeMap<&str, Fields> = BTreeMap::new();

        for (folder, fields) in &self.read {
            let file = manifest_path(folder);

            match self::fields(fields, &file) {
                Ok(read) => {
                    names.insert(folder, read);
                }
                Err(stop) => faults.push(Fault::new(file, stop)),
            }
        }

        // Each source of the workspace's globs, with the file that states them.
        let mut sources = Vec::new();

        if let Some((_, fields)) = self.read.iter().find(|(folder, _)| folder == ".") {
            match workspaces(fields) {
                Ok(patterns) => sources.push(("package.json", patterns)),
                Err(stop) => faults.push(Fault::new("package.json", stop)),
            }
        }

        if let Some(pnpm) = &self.pnpm {
            match pnpm_packages(pnpm) {
                Ok(patterns) => sources.push((PNPM_LIST, patterns)),
                Err(stop) => faults.push(Fault::new(PNPM_LIST, stop)),
            }
        }

        let mut included = Vec::new();
        let mut excluded = Vec::new();

        for (file, patterns) in &sources {
            match compile(patterns) {
                Ok((include, exclude)) => {
                    included.extend(include);
                    excluded.extend(exclude);
                }
                Err(stop) => faults.push(Fault::new(*file, stop)),
            }
        }

        if let Some(fault) = Fault::first(faults) {
            return Err(fault);
        }

        let detection: Option<BTreeSet<&str>> =
            detection.map(|folders| folders.iter().map(String::as_str).collect());
        let mut list: Vec<Package> = names
            .into_iter()
            .map(|(folder, fields)| {
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
                    name: fields.name,
                    workspace,
                    detected,
                    dependencies: fields.dependencies,
                    exports: fields.exports,
                    imports: fields.imports,
                    entries: fields.entries,
                }
            })
            .collect();

        list.sort_by(|a, b| a.folder.as_bytes().cmp(b.folder.as_bytes()));
        repeated_names(&list, notices);

        let folders: BTreeSet<&str> = list.iter().map(|package| package.folder.as_str()).collect();
        let of_file = files
            .iter()
            .filter_map(|file| {
                nearest(&file.path, &folders).map(|folder| (file.path.clone(), folder))
            })
            .collect();

        Ok(Packages { list, of_file })
    }
}

/// The fields of one `package.json` that a package keeps, each checked for its form (spec PACK-6).
fn fields(fields: &Map<String, Value>, file: &str) -> Result<Fields, Stop> {
    Ok(Fields {
        name: name(fields, file)?,
        dependencies: dependencies(fields, file)?,
        exports: targets(fields, "exports", file)?,
        imports: imports(fields, file)?,
        entries: entries(fields, file)?,
    })
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
pub fn misshapen(field: &str, file: &str, form: &str) -> Stop {
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

/// The form a target of `exports` or `imports` may take: a path, null, a list of targets, or an object
/// of conditions whose values are targets.
fn is_target(value: &Value) -> bool {
    match value {
        Value::String(_) | Value::Null => true,
        Value::Array(targets) => targets.iter().all(is_target),
        Value::Object(conditions) => conditions.values().all(is_target),
        _ => false,
    }
}

/// A `package.json`'s `exports`: one target, or an object whose keys are all subpaths, which begin with
/// `.`, or all conditions, none of which does.
fn targets(fields: &Map<String, Value>, field: &str, file: &str) -> Result<Option<Value>, Stop> {
    let form = "a path, a list, or an object whose keys are all subpaths beginning with `.` or all \
                conditions";
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    let keys_agree = value.as_object().is_none_or(|keys| {
        let subpaths = keys.keys().filter(|key| key.starts_with('.')).count();

        subpaths == 0 || subpaths == keys.len()
    });

    if is_target(value) && keys_agree {
        Ok(Some(value.clone()))
    } else {
        Err(misshapen(field, file, form))
    }
}

/// A `package.json`'s `imports`: an object whose keys begin with `#` and whose values are targets.
fn imports(fields: &Map<String, Value>, file: &str) -> Result<Option<Value>, Stop> {
    let form =
        "an object whose keys begin with `#` and whose values are paths, lists or conditions";

    match fields.get("imports") {
        None => Ok(None),
        Some(Value::Object(map))
            if map
                .iter()
                .all(|(key, value)| key.starts_with('#') && is_target(value)) =>
        {
            Ok(fields.get("imports").cloned())
        }
        Some(_) => Err(misshapen("imports", file, form)),
    }
}

/// The targets of a `package.json`'s `types` (or its older name `typings`), `module` and `main`, in
/// that order, those it states.
fn entries(fields: &Map<String, Value>, file: &str) -> Result<Vec<String>, Stop> {
    let types = if fields.contains_key("types") {
        "types"
    } else {
        "typings"
    };
    let mut entries = Vec::new();

    for field in [types, "module", "main"] {
        match fields.get(field) {
            None => {}
            Some(Value::String(target)) => entries.push(target.clone()),
            Some(_) => return Err(misshapen(field, file, "a path")),
        }
    }

    Ok(entries)
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();

    for list in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let form = "an object that maps each package's name to the version it asks for";
        let entries = match fields.get(list) {
            None => continue,
            Some(Value::Object(entries)) => entries,
            Some(_) => return Err(misshapen(list, file, form)),
        };

        for (package, version) in entries {
            let version = version
                .as_str()
                .ok_or_else(|| misshapen(list, file, form))?;

            named
                .entry(package.clone())
                .or_insert_with(|| version.to_owned());
        }
    }

    Ok(named)
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

```rust path=src/tsconfig.rs
//! tsconfig files: each one read, as JSON that may hold comments; which ones a file the analyzer reads
//! depends on; the one that governs a file, with its `paths` and `baseUrl`; and the `outDir` and
//! `rootDir` that MAP-1 maps build output back with (spec CFG-1 to CFG-4, RES-9, MAP-1, MAP-3, LIST-4,
//! LIST-8, LIST-9).
//!
//! `oxc_resolver` finds the config that governs a file, through `references` and `extends`, from a file
//! system of the analyzer's own. That file system answers only for the configs the analyzer read, each
//! served as plain JSON that holds the fields this item uses, with every target of `extends` and
//! `references` already found by the analyzer. So the library reads no file outside the list, looks in
//! no `node_modules`, and meets no field the analyzer has not checked.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use oxc_resolver::{
    FileMetadata, FileSystem, ResolveError, ResolveOptions, ResolverGeneric, TsConfig,
    TsconfigDiscovery,
};
use serde_json::{Map, Value, json};

use crate::files::{File, Kind};
use crate::git::{self, Above};
use crate::notice::Notice;
use crate::packages::{Packages, misshapen};
use crate::stop::{Fault, Stop};

/// The names of the configs that govern the files of their folder (spec CFG-2).
const GOVERNING: [&str; 2] = ["tsconfig.json", "jsconfig.json"];

/// What a target of `extends` or `references` names, as the analyzer finds it (spec CFG-3, CFG-4).
#[derive(Debug)]
enum Target {
    /// A config that is read, by its path from the root.
    Read(String),
    /// Something the config is used without, and why, as the end of a sentence.
    Without(&'static str),
    /// A file that should be read and is not there, by its path from the root.
    Missing(String),
}

/// What a config's `extends` and `references` name, each with the text it wrote.
#[derive(Debug, Default)]
struct Targets {
    extends: Vec<(String, Target)>,
    references: Vec<(String, Target)>,
}

/// One config that was read.
#[derive(Debug)]
struct Config {
    /// Its fields, as read.
    fields: Map<String, Value>,
    /// The configs its `extends` names that are read, in its order.
    extends: Vec<String>,
    /// The configs its `references` names that are read.
    references: Vec<String>,
}

/// The configs, read but not yet checked field by field, which RUN-4 asks for after every `package.json`
/// is read (spec RUN-4, steps 10 and 11).
pub struct Loaded {
    /// The root, with `.` and `..` worked out.
    root: PathBuf,
    /// Each config that was read, by its path from the root; one above the root begins with `..`.
    configs: BTreeMap<String, Config>,
    /// The configs a file the analyzer reads depends on (spec CFG-4).
    named: BTreeSet<String>,
}

/// One build a config states, for MAP-1: the folder its output goes to, and the folders its sources are
/// looked for in, in order.
#[derive(Debug)]
pub struct Build {
    /// Its `outDir`, from the root.
    pub out_dir: String,
    /// Its `rootDir`; or, where it states none, `src` in the config's folder, then the folder itself.
    pub sources: Vec<String>,
}

/// The `paths` pattern a specifier matches (spec RES-2, step 2).
pub struct Mapped {
    /// Its targets, with the `*` filled in, from the root; one above the root begins with `..`.
    pub targets: Vec<String>,
    /// Whether the pattern is the bare `*`, which matches everything (spec RES-4).
    pub everything: bool,
}

/// The configs, checked, and the library that finds the one governing a file.
pub struct Configs {
    /// The root, with `.` and `..` worked out.
    root: PathBuf,
    /// The library, over the analyzer's own file system.
    resolver: ResolverGeneric<Served>,
    /// The builds of the configs in each folder, by the folder, the root's as the empty path: those of
    /// `tsconfig.json` first, then the others by path.
    builds: BTreeMap<String, Vec<Build>>,
}

/// Reads every tsconfig in the list that is not data, and the configs above the root they extend. A
/// config that a file the analyzer reads depends on stops the run when it cannot be read, or when it
/// extends a file that should be read and is not there; any other is left out with a notice (spec
/// CFG-1, CFG-3, CFG-4). `packages` is there when the `package.json` files' fields hold their forms,
/// for an `extends` that names a workspace package.
pub fn load(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    packages: Option<&Packages>,
    notices: &mut Vec<Notice>,
) -> Result<Loaded, Stop> {
    let listed: BTreeSet<&str> = files
        .iter()
        .filter(|file| file.kind == Kind::Tsconfig && !data.contains(&file.path))
        .map(|file| file.path.as_str())
        .collect();
    let mut parsed: BTreeMap<String, Result<Map<String, Value>, String>> = listed
        .iter()
        .map(|path| ((*path).to_owned(), parse(root, path)))
        .collect();
    let finder = Finder {
        root,
        listed: &listed,
        data,
        packages,
    };

    // The targets of each config read, with the text it wrote for each. A config above the root that one
    // extends is read when it is first named.
    let mut targets: BTreeMap<String, Targets> = BTreeMap::new();
    let mut waiting: Vec<String> = parsed.keys().cloned().collect();

    while let Some(path) = waiting.pop() {
        let Some(Ok(fields)) = parsed.get(&path) else {
            continue;
        };
        let (extends, references) = (texts(fields.get("extends")), referenced(fields));
        let mut found = Vec::with_capacity(extends.len());

        for text in extends {
            let target = finder.extended(&path, &text)?;

            if let Target::Read(above) = &target
                && !parsed.contains_key(above)
            {
                parsed.insert(above.clone(), parse(root, above));
                waiting.push(above.clone());
            }

            found.push((text, target));
        }

        let references = references
            .into_iter()
            .map(|text| {
                let target = finder.reference(&path, &text);

                (text, target)
            })
            .collect();

        targets.insert(
            path,
            Targets {
                extends: found,
                references,
            },
        );
    }

    let named = depended_on(files, data, &listed, &targets);

    // An unreadable config, or a missing target of `extends`, that a file depends on stops the run, the
    // first by path (spec RUN-4, step 10).
    let mut faults = Vec::new();

    for path in &named {
        match parsed.get(path) {
            Some(Err(why)) => faults.push(Fault::new(path.clone(), unreadable(path, why))),
            _ => {
                for (text, target) in targets.get(path).map_or(&[][..], |found| &found.extends) {
                    if let Target::Missing(missing) = target {
                        faults.push(Fault::new(
                            path.clone(),
                            Stop::new(format!(
                                "the tsconfig {path} extends `{text}`, which should be the file {missing} and is not \
                                 there; fix its `extends`, or add the file"
                            )),
                        ));
                    }
                }
            }
        }
    }

    if let Some(fault) = Fault::first(faults) {
        return Err(fault.stop());
    }

    let mut configs = BTreeMap::new();

    for (path, read) in parsed {
        let fields = match read {
            Ok(fields) => fields,
            Err(why) => {
                notices.push(Notice::new(
                    "tsconfig-skipped",
                    Some(path),
                    format!(
                        "The tsconfig cannot be read ({why}), and no file the analyzer reads depends on it, so it \
                         is left out."
                    ),
                ));

                continue;
            }
        };
        let Targets {
            extends,
            references,
        } = targets.remove(&path).unwrap_or_default();

        for (text, target) in &extends {
            let why = match target {
                Target::Read(_) => continue,
                Target::Without(why) => *why,
                Target::Missing(_) => "which is not there",
            };

            notices.push(Notice::new(
                "tsconfig-extends-left-out",
                Some(path.clone()),
                format!("The tsconfig extends `{text}`, {why}, so it is used without it."),
            ));
        }

        for (text, target) in &references {
            if !matches!(target, Target::Read(_)) {
                notices.push(Notice::new(
                    "tsconfig-reference-missing",
                    Some(path.clone()),
                    format!(
                        "The tsconfig's `references` names `{text}`, which is no tsconfig in the file list, so it is \
                         passed over."
                    ),
                ));
            }
        }

        configs.insert(
            path,
            Config {
                fields,
                extends: read_targets(extends),
                references: read_targets(references),
            },
        );
    }

    Ok(Loaded {
        root: clean(root),
        configs,
        named,
    })
}

impl Loaded {
    /// Checks the fields the analyzer reads in every config it read, and gives the library the configs a
    /// file depends on (spec CFG-1, RES-9, PACK-6).
    pub fn checked(self) -> Result<Configs, Fault> {
        let faults = self.configs.iter().filter_map(|(path, config)| {
            forms(&config.fields, path)
                .err()
                .map(|stop| Fault::new(path.clone(), stop))
        });

        if let Some(fault) = Fault::first(faults) {
            return Err(fault);
        }

        let mut served = BTreeMap::new();

        for path in &self.named {
            if let Some(config) = self.configs.get(path) {
                served.insert(
                    absolute(&self.root, path),
                    self.served(path, config).to_string(),
                );
            }
        }

        // A `jsconfig.json` governs as a `tsconfig.json` does, in a folder that has none (spec CFG-2).
        for path in &self.named {
            if file_name(path) == "jsconfig.json" {
                let stand_in = absolute(&self.root, &within(parent(path), "tsconfig.json"));

                if !served.contains_key(&stand_in) {
                    let content = served[&absolute(&self.root, path)].clone();

                    served.insert(stand_in, content);
                }
            }
        }

        let builds = self.builds();
        let options = ResolveOptions {
            tsconfig: Some(TsconfigDiscovery::Auto),
            ..ResolveOptions::default()
        };

        Ok(Configs {
            resolver: ResolverGeneric::new_with_file_system(Served(Arc::new(served)), options),
            root: self.root,
            builds,
        })
    }

    /// A config as the library is given it: only the fields it uses, with each target of `extends` and
    /// `references` the analyzer found, written from the config's folder.
    fn served(&self, path: &str, config: &Config) -> Value {
        let folder = parent(path);
        let from_here = |target: &String| relative(&self.root, folder, target);
        let mut served = Map::new();

        if !config.extends.is_empty() {
            served.insert(
                "extends".to_owned(),
                json!(config.extends.iter().map(from_here).collect::<Vec<_>>()),
            );
        }

        for field in ["files", "include", "exclude"] {
            if let Some(value) = config.fields.get(field) {
                served.insert(field.to_owned(), value.clone());
            }
        }

        if !config.references.is_empty() {
            served.insert(
                "references".to_owned(),
                config
                    .references
                    .iter()
                    .map(|target| json!({ "path": from_here(target) }))
                    .collect(),
            );
        }

        if let Some(Value::Object(options)) = config.fields.get("compilerOptions") {
            let kept: Map<String, Value> = ["paths", "baseUrl"]
                .into_iter()
                .filter_map(|field| Some((field.to_owned(), options.get(field)?.clone())))
                .collect();

            if !kept.is_empty() {
                served.insert("compilerOptions".to_owned(), Value::Object(kept));
            }
        }

        Value::Object(served)
    }

    /// The builds of the configs under the root, by folder (spec MAP-1, MAP-3).
    fn builds(&self) -> BTreeMap<String, Vec<Build>> {
        let mut builds: BTreeMap<String, Vec<(bool, Build)>> = BTreeMap::new();

        for path in self.configs.keys().filter(|path| !is_above(path)) {
            let folder = parent(path);
            let Some(out_dir) = self
                .stated(path, "outDir", &mut BTreeSet::new())
                .filter(|out_dir| !is_above(out_dir))
            else {
                continue;
            };
            let sources = match self.stated(path, "rootDir", &mut BTreeSet::new()) {
                Some(root_dir) => vec![root_dir],
                None => vec![within(folder, "src"), folder.to_owned()],
            };

            builds.entry(folder.to_owned()).or_default().push((
                file_name(path) != "tsconfig.json",
                Build { out_dir, sources },
            ));
        }

        builds
            .into_iter()
            .map(|(folder, mut found)| {
                // Stable, so the others stay in order by path.
                found.sort_by_key(|(other, _)| *other);

                (folder, found.into_iter().map(|(_, build)| build).collect())
            })
            .collect()
    }

    /// A folder a config states in `compilerOptions`, its own or one it takes through `extends`, the later
    /// of two it extends winning, from the root (spec MAP-3).
    fn stated(&self, path: &str, field: &str, seen: &mut BTreeSet<String>) -> Option<String> {
        let config = self.configs.get(path)?;

        if let Some(Value::String(folder)) = config
            .fields
            .get("compilerOptions")
            .and_then(|options| options.get(field))
        {
            return (!folder.starts_with('/') && !folder.contains(':'))
                .then(|| normalized(parent(path), folder));
        }

        config.extends.iter().rev().find_map(|target| {
            seen.insert(target.clone())
                .then(|| self.stated(target, field, seen))
                .flatten()
        })
    }
}

impl Configs {
    /// The config that governs a file, where one under the root does: the search ends at the root (spec
    /// CFG-2, LIST-8). A file that is not TypeScript is asked about as a `.ts` file of the same name in
    /// the same folder.
    pub fn governing(&self, file: &str) -> Result<Option<Arc<TsConfig>>, Stop> {
        let asked = absolute(&self.root, &as_typescript(file));

        match self.resolver.find_tsconfig(&asked) {
            Ok(Some(config)) if config.path().starts_with(&self.root) => Ok(Some(config)),
            Ok(_) => Ok(None),
            Err(error) => Err(Stop::new(format!(
                "the tsconfig that governs {file} cannot be read ({error}); fix it"
            ))),
        }
    }

    /// The `paths` pattern of a config that a specifier matches, as TypeScript matches one: the key
    /// itself when it has no `*`, or else the pattern with the longest part before its one `*`, the first
    /// of two as long (spec RES-2, step 2).
    pub fn paths(&self, config: &TsConfig, specifier: &str) -> Option<Mapped> {
        let paths = config.compiler_options.paths.as_ref()?;
        let (key, star) = match paths
            .get_key_value(specifier)
            .filter(|(key, _)| !key.contains('*'))
        {
            Some((key, _)) => (key.as_str(), None),
            None => {
                let mut best: Option<(&str, &str)> = None;

                for key in paths.keys() {
                    let Some((before, after)) = key.split_once('*') else {
                        continue;
                    };
                    let fits = !after.contains('*')
                        && specifier.len() >= before.len() + after.len()
                        && specifier.starts_with(before)
                        && specifier.ends_with(after);

                    if fits && best.is_none_or(|(_, chosen)| before.len() > chosen.len()) {
                        best = Some((key, before));
                    }
                }

                let (key, before) = best?;
                let after = &key[before.len() + 1..];

                (
                    key,
                    Some(&specifier[before.len()..specifier.len() - after.len()]),
                )
            }
        };

        Some(Mapped {
            targets: paths[key]
                .iter()
                .map(|target| {
                    let target = self.rooted(target);

                    star.map_or_else(|| target.clone(), |star| target.replacen('*', star, 1))
                })
                .collect(),
            everything: key == "*",
        })
    }

    /// The folder a config's `baseUrl` names, from the root, where it states one (spec RES-2, step 4).
    pub fn base_url(&self, config: &TsConfig) -> Option<String> {
        config
            .compiler_options
            .base_url
            .as_ref()
            .map(|folder| self.rooted(folder))
    }

    /// The builds of the configs that lie directly in a folder, the root's as the empty path (spec
    /// MAP-1).
    pub fn builds(&self, folder: &str) -> &[Build] {
        self.builds.get(folder).map_or(&[], Vec::as_slice)
    }

    /// A path the library gives, from the root; one above the root begins with `..`.
    fn rooted(&self, path: &Path) -> String {
        steps(&self.root, &clean(path)).join("/")
    }
}

/// Finds the targets of `extends` and `references`.
struct Finder<'a> {
    root: &'a Path,
    listed: &'a BTreeSet<&'a str>,
    data: &'a BTreeSet<String>,
    packages: Option<&'a Packages>,
}

impl Finder<'_> {
    /// What a config's `extends` names (spec CFG-3). A relative target is read when it is in the list,
    /// or, above the root, when it is a file of the work tree that git does not ignore; one that git
    /// ignores, or that is data, is left out, and one that is not there should be. A target in a
    /// workspace package is read; one in a package outside the project is left out.
    fn extended(&self, config: &str, text: &str) -> Result<Target, Stop> {
        if !(text.starts_with("./") || text.starts_with("../")) {
            return Ok(self.in_package(text));
        }

        let path = normalized(parent(config), text);
        // TypeScript adds `.json` to a target that is not there without it.
        let tried: Vec<String> = if path.ends_with(".json") {
            vec![path]
        } else {
            vec![path.clone(), format!("{path}.json")]
        };

        if is_above(&tried[0]) {
            for candidate in &tried {
                match git::above(self.root, candidate)? {
                    Above::File => return Ok(Target::Read(candidate.clone())),
                    Above::Ignored => return Ok(Target::Without("a file git ignores")),
                    Above::OutsideWorkTree => {
                        return Ok(Target::Without("a file outside the work tree"));
                    }
                    Above::Missing => {}
                }
            }

            return Ok(Target::Missing(tried[0].clone()));
        }

        if let Some(found) = tried
            .iter()
            .find(|candidate| self.listed.contains(candidate.as_str()))
        {
            return Ok(Target::Read(found.clone()));
        }

        if tried.iter().any(|candidate| self.data.contains(candidate)) {
            return Ok(Target::Without("a file declared as data"));
        }

        let asked: Vec<&str> = tried.iter().map(String::as_str).collect();

        Ok(if git::ignored(self.root, &asked)?.is_empty() {
            Target::Missing(tried[0].clone())
        } else {
            Target::Without("a file git ignores")
        })
    }

    /// What an `extends` that is not a relative path names: a config in a workspace package, by the
    /// package's name and a path inside it, `tsconfig.json` where it names none (spec CFG-3).
    fn in_package(&self, text: &str) -> Target {
        let outside = Target::Without("which is not in the project");
        let segments = if text.starts_with('@') { 2 } else { 1 };
        let name = text.split('/').take(segments).collect::<Vec<_>>().join("/");
        let Some(package) = self.packages.and_then(|packages| {
            packages
                .list
                .iter()
                .find(|package| package.workspace && package.name.as_deref() == Some(name.as_str()))
        }) else {
            return outside;
        };
        let folder = if package.folder == "." {
            ""
        } else {
            package.folder.as_str()
        };
        let inside = text[name.len()..].trim_start_matches('/');
        let path = if inside.is_empty() {
            within(folder, "tsconfig.json")
        } else {
            normalized(folder, inside)
        };

        [path.clone(), format!("{path}.json")]
            .into_iter()
            .find(|candidate| self.listed.contains(candidate.as_str()))
            .map_or(Target::Missing(path), Target::Read)
    }

    /// What a config's `references` names: a tsconfig in the list, the file itself or the
    /// `tsconfig.json` in the folder it names (spec CFG-4).
    fn reference(&self, config: &str, text: &str) -> Target {
        let path = normalized(parent(config), text);

        [path.clone(), within(&path, "tsconfig.json")]
            .into_iter()
            .find(|candidate| !is_above(candidate) && self.listed.contains(candidate.as_str()))
            .map_or(Target::Missing(path), Target::Read)
    }
}

/// The configs a file the analyzer reads depends on: each `tsconfig.json` and `jsconfig.json` at or
/// above such a file, up to the root, and every config one of those names in `extends` or `references`,
/// in turn (spec CFG-4).
fn depended_on(
    files: &[File],
    data: &BTreeSet<String>,
    listed: &BTreeSet<&str>,
    targets: &BTreeMap<String, Targets>,
) -> BTreeSet<String> {
    let mut folders: BTreeSet<&str> = BTreeSet::new();

    for file in files.iter().filter(|file| {
        matches!(file.kind, Kind::Script | Kind::Declaration) && !data.contains(&file.path)
    }) {
        let mut folder = file.path.as_str();

        loop {
            folder = parent(folder);

            if !folders.insert(folder) || folder.is_empty() {
                break;
            }
        }
    }

    let mut named: BTreeSet<String> = folders
        .iter()
        .flat_map(|folder| GOVERNING.iter().map(|name| within(folder, name)))
        .filter(|path| listed.contains(path.as_str()))
        .collect();
    let mut waiting: Vec<String> = named.iter().cloned().collect();

    while let Some(path) = waiting.pop() {
        let Some(found) = targets.get(&path) else {
            continue;
        };

        for (_, target) in found.extends.iter().chain(&found.references) {
            if let Target::Read(found) = target
                && named.insert(found.clone())
            {
                waiting.push(found.clone());
            }
        }
    }

    named
}

/// The targets that are read, by path.
fn read_targets(targets: Vec<(String, Target)>) -> Vec<String> {
    targets
        .into_iter()
        .filter_map(|(_, target)| match target {
            Target::Read(path) => Some(path),
            _ => None,
        })
        .collect()
}

/// A config's content as JSON that may hold comments and trailing commas, with a byte-order mark at its
/// start passed over (spec CFG-1, PARSE-1); or why it cannot be read.
fn parse(root: &Path, path: &str) -> Result<Map<String, Value>, String> {
    let bytes = fs::read(root.join(path)).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let mut text =
        String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())?;

    json_strip_comments::strip(&mut text).map_err(|error| error.to_string())?;

    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(fields)) => Ok(fields),
        Ok(_) => Err("it holds something other than an object".to_owned()),
        Err(error) => Err(format!("line {}, column {}", error.line(), error.column())),
    }
}

/// The stop for a config that cannot be read and that a file depends on (spec CFG-4).
fn unreadable(path: &str, why: &str) -> Stop {
    Stop::new(format!(
        "the tsconfig {path} cannot be read ({why}); fix it, or declare it as data if it is broken on purpose"
    ))
}

/// Checks the form of each field of a config that the analyzer reads (spec CFG-1, PACK-6).
fn forms(fields: &Map<String, Value>, file: &str) -> Result<(), Stop> {
    let paths = |value: &Value| {
        value
            .as_array()
            .is_some_and(|list| list.iter().all(Value::is_string))
    };

    if let Some(value) = fields.get("extends")
        && !(value.is_string() || paths(value))
    {
        return Err(misshapen("extends", file, "a path, or a list of paths"));
    }

    for field in ["files", "include", "exclude"] {
        if fields.get(field).is_some_and(|value| !paths(value)) {
            return Err(misshapen(field, file, "a list of paths"));
        }
    }

    if fields.get("references").is_some_and(|value| {
        !value.as_array().is_some_and(|list| {
            list.iter()
                .all(|reference| reference.get("path").is_some_and(Value::is_string))
        })
    }) {
        return Err(misshapen(
            "references",
            file,
            "a list of objects, each with a `path`",
        ));
    }

    let options = match fields.get("compilerOptions") {
        None => return Ok(()),
        Some(Value::Object(options)) => options,
        Some(_) => return Err(misshapen("compilerOptions", file, "an object")),
    };

    if options.get("paths").is_some_and(|value| {
        !value
            .as_object()
            .is_some_and(|patterns| patterns.values().all(paths))
    }) {
        return Err(misshapen(
            "compilerOptions.paths",
            file,
            "an object that maps each pattern to a list of paths",
        ));
    }

    for field in ["baseUrl", "outDir", "rootDir"] {
        if options.get(field).is_some_and(|value| !value.is_string()) {
            return Err(misshapen(
                &format!("compilerOptions.{field}"),
                file,
                "a path",
            ));
        }
    }

    Ok(())
}

/// The texts of a config's `extends`, one or a list, those that are text; the form is checked later.
fn texts(extends: Option<&Value>) -> Vec<String> {
    match extends {
        Some(Value::String(text)) => vec![text.clone()],
        Some(Value::Array(list)) => list
            .iter()
            .filter_map(|text| text.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// The paths of a config's `references`, those that are text; the form is checked later.
fn referenced(fields: &Map<String, Value>) -> Vec<String> {
    fields
        .get("references")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|reference| reference.get("path")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// A file that is not TypeScript as TypeScript asks about it: a `.ts` file of the same name in the same
/// folder (spec CFG-2).
fn as_typescript(file: &str) -> String {
    if [".ts", ".tsx", ".mts", ".cts"]
        .iter()
        .any(|ending| file.ends_with(ending))
    {
        return file.to_owned();
    }

    let name = file_name(file);
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);

    within(parent(file), &format!("{stem}.ts"))
}

/// The folder of a path from the root, the root's as the empty path.
fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(folder, _)| folder)
}

/// The last segment of a path.
fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// A name in a folder, the root's as the empty path.
fn within(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_owned()
    } else {
        format!("{folder}/{name}")
    }
}

/// Whether a path from the root leads above it.
fn is_above(path: &str) -> bool {
    path == ".." || path.starts_with("../")
}

/// A folder and a relative path joined, `.` and `..` worked out, from the root; a path above the root
/// keeps its leading `..`.
fn normalized(folder: &str, relative: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();

    for segment in folder.split('/').chain(relative.split('/')) {
        match segment {
            "" | "." => {}
            ".." if segments.last().is_some_and(|last| *last != "..") => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    segments.join("/")
}

/// A path from the root as an absolute path, with `.` and `..` worked out.
fn absolute(root: &Path, path: &str) -> PathBuf {
    clean(&root.join(path))
}

/// A path with `.` and `..` worked out, without asking the file system.
fn clean(path: &Path) -> PathBuf {
    let mut cleaned = PathBuf::new();

    for part in path.components() {
        match part {
            Component::ParentDir => {
                cleaned.pop();
            }
            Component::CurDir => {}
            other => cleaned.push(other.as_os_str()),
        }
    }

    cleaned
}

/// The segments that lead from one absolute path to another, `..` first where it goes up.
fn steps(from: &Path, to: &Path) -> Vec<String> {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();

    std::iter::repeat_n("..".to_owned(), from.len() - common)
        .chain(
            to[common..]
                .iter()
                .map(|part| part.as_os_str().to_string_lossy().into_owned()),
        )
        .collect()
}

/// A path from the root as a config writes a target: from its folder, beginning with `.`.
fn relative(root: &Path, folder: &str, path: &str) -> String {
    let steps = steps(&absolute(root, folder), &absolute(root, path)).join("/");

    if steps.starts_with("..") {
        steps
    } else {
        format!("./{steps}")
    }
}

/// The file system the library is given: the configs the analyzer read, served as plain JSON, and
/// nothing else (spec RES-9, LIST-9).
#[derive(Clone, Default)]
struct Served(Arc<BTreeMap<PathBuf, String>>);

impl Served {
    /// The answer for a path it does not hold.
    fn missing() -> io::Error {
        io::Error::new(io::ErrorKind::NotFound, "not a tsconfig the analyzer read")
    }
}

impl FileSystem for Served {
    fn new() -> Self {
        Self::default()
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.read_to_string(path).map(String::into_bytes)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.0.get(&clean(path)).cloned().ok_or_else(Self::missing)
    }

    fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        if self.0.contains_key(&clean(path)) {
            Ok(FileMetadata::new(true, false, false))
        } else {
            Err(Self::missing())
        }
    }

    fn symlink_metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        self.metadata(path)
    }

    fn read_link(&self, path: &Path) -> Result<PathBuf, ResolveError> {
        Err(ResolveError::from(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a link", path.display()),
        )))
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(clean(path))
    }
}
```

```rust path=src/resolve.rs
//! Where each import leads (spec RES-1 to RES-6, RES-8, RES-10, MAP-1 to MAP-4, CFG-2, CFG-3, LIST-5).
//!
//! A path is looked for in the file list, never on disk, with exact letter case. What leads to no file
//! is asked about once, with one start of git for all of them, to tell an ignored path from a missing
//! one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Map, Value};

use crate::files::Listing;
use crate::git;
use crate::imports::Import;
use crate::packages::{Package, Packages};
use crate::scripts::Scripts;
use crate::stop::Stop;
use crate::tsconfig::Configs;

/// The modules built into Node, as `require('module').builtinModules` lists them in Node 22.19.0, those
/// whose names begin with `_` left out. Any specifier that begins with `node:` is built in too.
const BUILTINS: [&str; 54] = [
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// The endings a script's path may be given, in the order they are tried: TypeScript before JavaScript,
/// then a type declaration file (spec RES-3).
const ENDINGS: [&str; 9] = [
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".d.ts",
];

/// A script's output ending, and the source forms it is looked for under first (spec RES-3).
const SOURCE_FORMS: [(&str, &[&str]); 4] = [
    (".js", &[".ts", ".tsx", ".d.ts"]),
    (".jsx", &[".tsx"]),
    (".mjs", &[".mts", ".d.mts"]),
    (".cjs", &[".cts", ".d.cts"]),
];

/// A script and the type declaration file that describes it, by their endings (spec RES-3).
const PAIRS: [(&str, &str); 3] = [(".js", ".d.ts"), (".mjs", ".d.mts"), (".cjs", ".d.cts")];

/// The endings of build output that MAP-1 replaces: a type declaration's first, since `.d.ts` also
/// ends in `.ts`, then a script's output forms.
const OUTPUT_ENDINGS: [&str; 6] = [".d.ts", ".d.mts", ".d.cts", ".js", ".mjs", ".cjs"];

/// The source extensions MAP-1 puts in place of an output ending, in the order they are tried.
const SOURCE_ENDINGS: [&str; 8] = [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"];

/// The protocols of a dependency's version that name a package in the project (spec RES-2, step 5).
const LOCAL_PROTOCOLS: [&str; 3] = ["file:", "link:", "portal:"];

/// Where outside the project an import leads (spec RES-1, RES-8).
#[derive(Debug, PartialEq, Eq)]
pub enum Place {
    /// A package that is not in the project, with whether a `package.json` at or above the importing
    /// file lists it.
    Package { name: String, listed: bool },
    /// A module built into Node.
    Builtin,
    /// A path above the root.
    AboveRoot,
    /// A path inside a nested repository, a submodule or another worktree.
    OtherRepository,
    /// A symbolic link, or a path through one.
    Link,
    /// Something loaded from elsewhere: an absolute path or a URL.
    Elsewhere,
}

impl Place {
    /// The place's name in what `graph` prints.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Package { .. } => "package",
            Self::Builtin => "builtin",
            Self::AboveRoot => "above-root",
            Self::OtherRepository => "other-repository",
            Self::Link => "link",
            Self::Elsewhere => "elsewhere",
        }
    }
}

/// The one outcome of an import (spec RES-1).
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The files it leads to, by path, sorted.
    Files(Vec<String>),
    /// Outside the project.
    Outside(Place),
    /// A path git ignores, which is never a fault, and the target of a `package.json` it came from,
    /// where it came from one (spec MAP-4).
    Ignored { path: String, from: Option<String> },
    /// Nothing.
    Unresolved,
}

/// Which rule decided an import's outcome: a step of RES-2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Step 1: a relative specifier.
    Relative,
    /// Step 2: a pattern in the `paths` of the tsconfig that governs the importing file.
    Paths,
    /// Step 3: a name in the package's `imports`.
    Imports,
    /// Step 4: a path from the `baseUrl` of the tsconfig that governs the importing file.
    BaseUrl,
    /// Step 5: the name of a package in the project.
    Package,
    /// Step 6: anything else.
    Outside,
}

impl Rule {
    /// The rule's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Relative => "relative",
            Self::Paths => "paths",
            Self::Imports => "imports",
            Self::BaseUrl => "base-url",
            Self::Package => "package",
            Self::Outside => "outside",
        }
    }
}

/// Where one import leads, and which rule decided it.
#[derive(Debug)]
pub struct Resolution {
    /// The outcome.
    pub outcome: Outcome,
    /// The rule that decided it.
    pub rule: Rule,
}

/// A path git is asked about when nothing is found, and the target it came from, if any (spec MAP-4).
#[derive(Debug)]
struct Asked {
    path: String,
    from: Option<String>,
}

/// What looking for a path found.
#[derive(Debug)]
enum Found {
    /// An outcome that needs nothing more.
    Done(Outcome),
    /// No file: the paths to ask git about, the path as written first (spec RES-4).
    Missing(Vec<Asked>),
}

/// The project as the resolver needs it.
pub struct Resolver<'p> {
    /// The paths of the file list.
    files: BTreeSet<&'p str>,
    /// What the file list leaves out that an import can still lead into.
    listing: &'p Listing,
    /// The packages.
    packages: &'p Packages,
    /// The packages by folder, the root's as the empty path.
    by_folder: BTreeMap<&'p str, &'p Package>,
    /// The tsconfig files.
    configs: &'p Configs,
}

impl<'p> Resolver<'p> {
    /// A resolver over the file list, the packages and the tsconfig files.
    pub fn new(listing: &'p Listing, packages: &'p Packages, configs: &'p Configs) -> Self {
        Self {
            files: listing
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect(),
            listing,
            packages,
            by_folder: packages
                .list
                .iter()
                .map(|package| (folder_path(&package.folder), package))
                .collect(),
            configs,
        }
    }

    /// Where every import of every script read leads, by the script's path, in the order of its imports.
    pub fn resolve(
        &self,
        root: &Path,
        scripts: &Scripts,
    ) -> Result<BTreeMap<String, Vec<Resolution>>, Stop> {
        let mut found: Vec<(&str, Vec<(Found, Rule)>)> = Vec::with_capacity(scripts.read.len());

        for (path, read) in &scripts.read {
            found.push((
                path,
                read.imports
                    .iter()
                    .map(|import| self.one(path, import))
                    .collect::<Result<_, Stop>>()?,
            ));
        }

        // One start of git answers for every path that led to no file.
        let asked: BTreeSet<&str> = found
            .iter()
            .flat_map(|(_, imports)| imports)
            .filter_map(|(found, _)| match found {
                Found::Missing(paths) => Some(paths.iter().map(|asked| asked.path.as_str())),
                Found::Done(_) => None,
            })
            .flatten()
            .collect();
        let ignored: BTreeSet<String> = git::ignored(root, &asked.into_iter().collect::<Vec<_>>())?
            .into_iter()
            .collect();

        Ok(found
            .into_iter()
            .map(|(path, imports)| {
                let resolutions = imports
                    .into_iter()
                    .map(|(found, rule)| {
                        let outcome = match found {
                            Found::Done(outcome) => outcome,
                            Found::Missing(paths) => paths
                                .into_iter()
                                .find(|asked| ignored.contains(&asked.path))
                                .map_or(Outcome::Unresolved, |asked| Outcome::Ignored {
                                    path: asked.path,
                                    from: asked.from,
                                }),
                        };

                        Resolution { outcome, rule }
                    })
                    .collect();

                (path.to_owned(), resolutions)
            })
            .collect())
    }

    /// Where one import leads, before git is asked about what led to no file (spec RES-2).
    fn one(&self, importing: &str, import: &Import) -> Result<(Found, Rule), Stop> {
        let specifier = written_path(&import.specifier);

        if is_relative(specifier) {
            let folder = importing.rsplit_once('/').map_or("", |(folder, _)| folder);

            return Ok(match joined(folder, specifier) {
                Some(path) => (self.relative(&path), Rule::Relative),
                None => (
                    Found::Done(Outcome::Outside(Place::AboveRoot)),
                    Rule::Relative,
                ),
            });
        }

        // An absolute path is no name a tsconfig maps.
        let governing = if is_absolute(specifier) {
            None
        } else {
            self.configs.governing(importing)?
        };

        if let Some(mapped) = governing
            .as_ref()
            .and_then(|config| self.configs.paths(config, specifier))
        {
            let found = self.mapped(&mapped.targets);
            // A pattern that leads to no file sends on what it cannot settle (spec RES-4).
            let goes_on = matches!(found, Found::Missing(_))
                && (mapped.everything || self.names_package(importing, specifier));

            if !goes_on {
                return Ok((found, Rule::Paths));
            }
        }

        if specifier.starts_with('#') {
            return Ok((self.hash(importing, specifier), Rule::Imports));
        }

        if let Some(path) = governing
            .as_ref()
            .and_then(|config| self.configs.base_url(config))
            .filter(|folder| !is_above(folder))
            .and_then(|folder| joined(&folder, specifier))
        {
            let found = self.relative(&path);

            if matches!(found, Found::Done(Outcome::Files(_))) {
                return Ok((found, Rule::BaseUrl));
            }
        }

        Ok(self.bare(importing, specifier))
    }

    /// The targets of a `paths` pattern, tried in order: the first that leads to a file, or outside,
    /// decides; one that leads above the root is outside (spec RES-2, step 2, CFG-3).
    fn mapped(&self, targets: &[String]) -> Found {
        let mut asked = Vec::new();

        for target in targets {
            if is_above(target) {
                return Found::Done(Outcome::Outside(Place::AboveRoot));
            }

            match self.relative(target) {
                Found::Missing(missing) => asked.extend(missing),
                found => return found,
            }
        }

        if asked.is_empty() {
            Found::Done(Outcome::Unresolved)
        } else {
            Found::Missing(asked)
        }
    }

    /// Whether a specifier names a workspace package, or a package that a `package.json` at or above the
    /// importing file lists as a dependency of any kind (spec RES-4).
    fn names_package(&self, importing: &str, specifier: &str) -> bool {
        let name = package_name(specifier);

        self.packages
            .list
            .iter()
            .any(|package| package.workspace && package.name.as_deref() == Some(name.as_str()))
            || self.listing_of(importing, &name).is_some()
    }

    /// A path an import names from a folder, a relative specifier's or a tsconfig's: the file it names
    /// as RES-3 finds one, or else the source it is built from, where a tsconfig's `outDir` holds it
    /// (spec MAP-1, step 1).
    fn relative(&self, path: &str) -> Found {
        let mut asked = match self.locate(path, true) {
            Found::Missing(asked) => asked,
            found => return found,
        };

        for candidate in self.sources(self.package_folder(path), path, false) {
            if self.files.contains(candidate.as_str()) {
                return Found::Done(Outcome::Files(self.with_twin(&candidate)));
            }

            if is_source(&candidate) {
                asked.push(Asked {
                    path: candidate,
                    from: None,
                });
            }
        }

        Found::Missing(asked)
    }

    /// The folder of the package a path lies in, or the root's, the empty path, where it lies in none.
    fn package_folder<'a>(&self, path: &'a str) -> &'a str {
        let mut folder = path;

        while let Some((above, _)) = folder.rsplit_once('/') {
            if self.by_folder.contains_key(above) {
                return above;
            }

            folder = above;
        }

        ""
    }

    /// The places MAP-1 looks for the source of build output at a path, with the folder of the package it
    /// lies in, in order (spec MAP-1). First, for each tsconfig that lies directly in that folder and
    /// states an `outDir` the path lies in, the same path under its sources. Then, where `moved` allows,
    /// the path with its first folder under the package replaced by `src`. In each place an output ending
    /// is replaced by each source extension in turn.
    fn sources(&self, folder: &str, path: &str, moved: bool) -> Vec<String> {
        let mut places = Vec::new();

        for build in self.configs.builds(folder) {
            let inside = path
                .strip_prefix(build.out_dir.as_str())
                .and_then(|rest| rest.strip_prefix('/'));

            if let Some(inside) = inside {
                places.extend(build.sources.iter().map(|source| within(source, inside)));
            }
        }

        let inside = if folder.is_empty() {
            Some(path)
        } else {
            path.strip_prefix(folder)
                .and_then(|rest| rest.strip_prefix('/'))
        };

        if moved && let Some((_, below)) = inside.and_then(|inside| inside.split_once('/')) {
            places.push(within(&within(folder, "src"), below));
        }

        places
            .into_iter()
            .flat_map(|place| {
                match OUTPUT_ENDINGS
                    .iter()
                    .find_map(|ending| place.strip_suffix(ending))
                {
                    Some(stem) => SOURCE_ENDINGS
                        .iter()
                        .map(|ending| format!("{stem}{ending}"))
                        .collect(),
                    None => vec![place],
                }
            })
            .collect()
    }

    /// A specifier that is neither relative nor a `#` name: a package of the project by its name, or
    /// somewhere outside (spec RES-2, steps 5 and 6).
    fn bare(&self, importing: &str, specifier: &str) -> (Found, Rule) {
        let outside = |place| (Found::Done(Outcome::Outside(place)), Rule::Outside);

        // Before a URL, which `node:` would read as.
        if specifier.starts_with("node:") {
            return outside(Place::Builtin);
        }

        if is_absolute(specifier) || is_url(specifier) {
            return outside(Place::Elsewhere);
        }

        let name = package_name(specifier);

        if let Some(package) = self.named(importing, &name) {
            return (
                self.through(importing, package, &specifier[name.len()..]),
                Rule::Package,
            );
        }

        if BUILTINS.contains(&specifier) {
            return outside(Place::Builtin);
        }

        outside(Place::Package {
            listed: self.listing_of(importing, &name).is_some(),
            name,
        })
    }

    /// The package of the project that an import by a name leads to, where one does (spec RES-2,
    /// step 5, PACK-5): the importing file's own package by its own name, the first workspace package
    /// by path with the name, or the package a dependency at or above the importing file names with a
    /// local protocol.
    fn named(&self, importing: &str, name: &str) -> Option<&'p Package> {
        let named = |package: &&Package| package.name.as_deref() == Some(name);

        if let Some(own) = self.own_package(importing).filter(|package| named(package)) {
            return Some(own);
        }

        if let Some(workspace) = self
            .packages
            .list
            .iter()
            .find(|package| package.workspace && named(package))
        {
            return Some(workspace);
        }

        let (listing, version) = self.listing_of(importing, name)?;

        if version.starts_with("workspace:") {
            return self.packages.list.iter().find(named);
        }

        let path = LOCAL_PROTOCOLS
            .iter()
            .find_map(|protocol| version.strip_prefix(protocol))?;

        joined(folder_path(&listing.folder), path)
            .and_then(|folder| self.by_folder.get(folder.as_str()).copied())
    }

    /// The nearest package at or above the importing file whose `package.json` lists a package as a
    /// dependency of any kind, with the version it asks for (spec RES-2, RES-8).
    fn listing_of(&self, importing: &str, name: &str) -> Option<(&'p Package, &'p str)> {
        let mut folder = importing;

        loop {
            folder = folder.rsplit_once('/').map_or("", |(above, _)| above);

            if let Some(package) = self.by_folder.get(folder)
                && let Some(version) = package.dependencies.get(name)
            {
                return Some((package, version));
            }

            if folder.is_empty() {
                return None;
            }
        }
    }

    /// The package the importing file belongs to (spec PACK-1).
    fn own_package(&self, importing: &str) -> Option<&'p Package> {
        self.packages
            .of_file
            .get(importing)
            .and_then(|folder| self.by_folder.get(folder_path(folder)).copied())
    }

    /// A `#` specifier, looked up in the `imports` of the importing file's package (spec RES-5).
    fn hash(&self, importing: &str, specifier: &str) -> Found {
        let map = self
            .own_package(importing)
            .and_then(|package| Some((package, package.imports.as_ref()?.as_object()?)));

        match map.and_then(|(package, map)| Some((package, matched(map, specifier)?))) {
            Some((package, (value, star))) => self.targets(importing, package, value, star, true),
            None => Found::Done(Outcome::Unresolved),
        }
    }

    /// An import of a package by its name, the bare name or with a subpath, through its `package.json`
    /// (spec RES-6).
    fn through(&self, importing: &str, package: &'p Package, subpath: &str) -> Found {
        let folder = folder_path(&package.folder);

        if let Some(exports) = &package.exports {
            let wanted = format!(".{subpath}");
            let subpaths = exports
                .as_object()
                .filter(|keys| keys.keys().any(|key| key.starts_with('.')));

            return match subpaths {
                Some(map) => match matched(map, &wanted) {
                    Some((value, star)) => self.targets(importing, package, value, star, false),
                    None => Found::Done(Outcome::Unresolved),
                },
                None if subpath.is_empty() => {
                    self.targets(importing, package, exports, None, false)
                }
                None => Found::Done(Outcome::Unresolved),
            };
        }

        if subpath.is_empty() {
            return self.entries(package).unwrap_or_else(|| self.index(folder));
        }

        match joined(folder, &subpath[1..]) {
            Some(path) => self.target(folder, &path, None),
            None => Found::Done(Outcome::Outside(Place::AboveRoot)),
        }
    }

    /// Every file a package's `types`, `module` and `main` lead to; nothing when it states none of them
    /// (spec RES-3, RES-6).
    fn entries(&self, package: &Package) -> Option<Found> {
        let folder = folder_path(&package.folder);

        (!package.entries.is_empty()).then(|| {
            combined(
                package
                    .entries
                    .iter()
                    .map(|entry| match joined(folder, entry) {
                        Some(path) => self.target(folder, &path, Some(&path)),
                        None => Found::Done(Outcome::Outside(Place::AboveRoot)),
                    })
                    .collect(),
            )
        })
    }

    /// The targets of a matched key of `exports` or `imports`, every condition and every entry of a list
    /// followed, each `*` filled in from the specifier (spec RES-5, RES-6, PACK-6).
    fn targets(
        &self,
        importing: &str,
        package: &'p Package,
        value: &Value,
        star: Option<&str>,
        imports: bool,
    ) -> Found {
        match value {
            Value::String(target) => {
                let target = star.map_or_else(|| target.clone(), |star| target.replace('*', star));
                let folder = folder_path(&package.folder);

                if let Some(inside) = target.strip_prefix("./") {
                    match joined(folder, inside) {
                        Some(path) => self.target(folder, &path, Some(&path)),
                        None => Found::Done(Outcome::Outside(Place::AboveRoot)),
                    }
                } else if imports && !target.starts_with("../") && !target.starts_with('/') {
                    // A name in `imports` may stand for a package, imported by its name.
                    self.bare(importing, &target).0
                } else {
                    Found::Done(Outcome::Unresolved)
                }
            }
            Value::Array(list) => combined(
                list.iter()
                    .map(|each| self.targets(importing, package, each, star, imports))
                    .collect(),
            ),
            Value::Object(conditions) => combined(
                conditions
                    .values()
                    .map(|each| self.targets(importing, package, each, star, imports))
                    .collect(),
            ),
            _ => Found::Done(Outcome::Unresolved),
        }
    }

    /// One target of a `package.json`, or a path inside a package, in the package's folder: the file it
    /// names as RES-3 finds one, or else the source it is built from (spec MAP-1, MAP-2).
    fn target(&self, folder: &str, path: &str, from: Option<&str>) -> Found {
        let mut asked = match self.locate(path, false) {
            Found::Done(outcome) => return Found::Done(outcome),
            Found::Missing(asked) => asked,
        };

        for candidate in self.sources(folder, path, true) {
            if self.files.contains(candidate.as_str()) {
                return Found::Done(Outcome::Files(self.with_twin(&candidate)));
            }

            if is_source(&candidate) {
                asked.push(Asked {
                    path: candidate,
                    from: None,
                });
            }
        }

        for each in &mut asked {
            each.from = from.map(str::to_owned);
        }

        Found::Missing(asked)
    }

    /// Looks for a path, relative to the root, as TypeScript finds one, in the file list (spec RES-3).
    /// A folder that holds a `package.json` is tried through its `types`, `module` and `main` first,
    /// where `packaged` allows it.
    fn locate(&self, path: &str, packaged: bool) -> Found {
        if under(path, &self.listing.elsewhere) {
            return Found::Done(Outcome::Outside(Place::OtherRepository));
        }

        if under(path, &self.listing.links) {
            return Found::Done(Outcome::Outside(Place::Link));
        }

        let mut asked = Vec::new();

        if let Some(found) = self
            .by_folder
            .get(path)
            .filter(|_| packaged)
            .and_then(|package| self.entries(package))
        {
            match found {
                Found::Missing(missing) => asked = missing,
                found => return found,
            }
        }

        // Each candidate with whether it is the path as written or a source form, the ones git is asked
        // about when nothing is found.
        let mut candidates: Vec<(String, bool)> = Vec::new();

        if let Some((stem, forms)) = SOURCE_FORMS
            .iter()
            .find_map(|(output, forms)| path.strip_suffix(output).map(|stem| (stem, *forms)))
        {
            candidates.extend(forms.iter().map(|form| (format!("{stem}{form}"), true)));
        }

        if !path.is_empty() {
            candidates.push((path.to_owned(), true));
            candidates.extend(
                ENDINGS
                    .iter()
                    .map(|ending| (format!("{path}{ending}"), is_source(ending))),
            );
        }

        candidates.extend(index_candidates(path));

        match candidates
            .iter()
            .find(|(candidate, _)| self.files.contains(candidate.as_str()))
        {
            Some((found, _)) => Found::Done(Outcome::Files(self.with_twin(found))),
            // The path as written first, so that it is the one named when git ignores it.
            None => Found::Missing(
                asked
                    .into_iter()
                    .chain(
                        std::iter::once(path.to_owned())
                            .chain(
                                candidates
                                    .into_iter()
                                    .filter(|(candidate, asked)| *asked && candidate != path)
                                    .map(|(candidate, _)| candidate),
                            )
                            .filter(|candidate| !candidate.is_empty())
                            .map(|path| Asked { path, from: None }),
                    )
                    .collect(),
            ),
        }
    }

    /// The `index` file of a package's folder, for a package that states no `types`, `module` or
    /// `main` (spec RES-6).
    fn index(&self, folder: &str) -> Found {
        let candidates = index_candidates(folder);

        match candidates
            .iter()
            .find(|(candidate, _)| self.files.contains(candidate.as_str()))
        {
            Some((found, _)) => Found::Done(Outcome::Files(self.with_twin(found))),
            None => Found::Missing(
                candidates
                    .into_iter()
                    .filter(|(_, asked)| *asked)
                    .map(|(path, _)| Asked { path, from: None })
                    .collect(),
            ),
        }
    }

    /// A file found, with the other of its pair where the list holds it: a script and the type
    /// declaration file of the same name (spec RES-3).
    fn with_twin(&self, found: &str) -> Vec<String> {
        let twin = PAIRS.iter().find_map(|(script, declaration)| {
            if let Some(stem) = found.strip_suffix(declaration) {
                Some(format!("{stem}{script}"))
            } else {
                found
                    .strip_suffix(script)
                    .map(|stem| format!("{stem}{declaration}"))
            }
        });
        let mut files = vec![found.to_owned()];

        if let Some(twin) = twin.filter(|twin| self.files.contains(twin.as_str())) {
            files.push(twin);
        }

        files.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        files
    }
}

/// What several targets found together: every file any of them leads to; else every path to ask git
/// about; else the first place outside; else nothing (spec RES-5).
fn combined(each: Vec<Found>) -> Found {
    let mut files: Vec<String> = Vec::new();
    let mut asked = Vec::new();
    let mut outside = None;

    for found in each {
        match found {
            Found::Done(Outcome::Files(found)) => files.extend(found),
            Found::Missing(missing) => asked.extend(missing),
            Found::Done(Outcome::Outside(place)) => {
                outside.get_or_insert(place);
            }
            Found::Done(_) => {}
        }
    }

    if !files.is_empty() {
        files.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        files.dedup();

        Found::Done(Outcome::Files(files))
    } else if !asked.is_empty() {
        Found::Missing(asked)
    } else {
        Found::Done(outside.map_or(Outcome::Unresolved, Outcome::Outside))
    }
}

/// The key of an `exports` or `imports` map that a subpath matches, as Node matches one: the key
/// itself when it has no `*`, or else the pattern with the longest part before its one `*`, the
/// longer key where two are as long; with the text the `*` stands for (spec RES-5, RES-6).
fn matched<'v, 'w>(
    map: &'v Map<String, Value>,
    wanted: &'w str,
) -> Option<(&'v Value, Option<&'w str>)> {
    if let Some(value) = map.get(wanted).filter(|_| !wanted.contains('*')) {
        return Some((value, None));
    }

    let mut best: Option<(&str, &Value, &str)> = None;

    for (key, value) in map {
        let Some((before, after)) = key.split_once('*') else {
            continue;
        };

        if after.contains('*')
            || wanted.len() < key.len()
            || !wanted.starts_with(before)
            || !wanted.ends_with(after)
        {
            continue;
        }

        let better = best.is_none_or(|(chosen, _, _)| {
            let chosen_before = chosen.find('*').map_or(chosen.len(), |at| at + 1);

            before.len() + 1 > chosen_before
                || (before.len() + 1 == chosen_before && key.len() > chosen.len())
        });

        if better {
            best = Some((
                key,
                value,
                &wanted[before.len()..wanted.len() - after.len()],
            ));
        }
    }

    best.map(|(_, value, star)| (value, Some(star)))
}

/// The `index` files of a folder, in the order they are tried, each with whether git is asked about it.
fn index_candidates(folder: &str) -> Vec<(String, bool)> {
    let index = if folder.is_empty() {
        "index".to_owned()
    } else {
        format!("{folder}/index")
    };

    ENDINGS
        .iter()
        .map(|ending| (format!("{index}{ending}"), is_source(ending)))
        .collect()
}

/// Whether a path, or an ending, is a source form, which git is asked about when nothing is found: a
/// TypeScript ending or a type declaration's (spec RES-4).
fn is_source(path: &str) -> bool {
    [".ts", ".tsx", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending))
}

/// A name in a folder, the root's as the empty path.
fn within(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_owned()
    } else {
        format!("{folder}/{name}")
    }
}

/// Whether a path from the root leads above it.
fn is_above(path: &str) -> bool {
    path == ".." || path.starts_with("../")
}

/// A package's folder as a path from the root: the root's own is the empty path.
fn folder_path(folder: &str) -> &str {
    if folder == "." { "" } else { folder }
}

/// The specifier without a query or a fragment (spec IMP-4).
fn written_path(specifier: &str) -> &str {
    let query = specifier.find('?').unwrap_or(specifier.len());
    let fragment = specifier
        .get(1..)
        .and_then(|rest| rest.find('#'))
        .map_or(specifier.len(), |at| at + 1);

    &specifier[..query.min(fragment)]
}

/// Whether a specifier is relative: `.`, `..`, or one that begins with `./` or `../` (spec RES-2).
fn is_relative(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}

/// Whether a specifier is an absolute path: from the top of a drive or a file system.
fn is_absolute(specifier: &str) -> bool {
    let bytes = specifier.as_bytes();
    let drive = bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');

    specifier.starts_with('/') || specifier.starts_with('\\') || drive
}

/// Whether a specifier is a URL: a scheme of two characters or more, then a colon.
fn is_url(specifier: &str) -> bool {
    specifier.split_once(':').is_some_and(|(scheme, _)| {
        scheme.len() > 1
            && scheme.starts_with(|first: char| first.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "+-.".contains(character))
    })
}

/// The name of the package a bare specifier names: its first segment, or its first two when it begins
/// with `@` (spec RES-2).
fn package_name(specifier: &str) -> String {
    let segments = if specifier.starts_with('@') { 2 } else { 1 };

    specifier
        .split('/')
        .take(segments)
        .collect::<Vec<_>>()
        .join("/")
}

/// A folder and a relative path joined, with `.` and `..` worked out; nothing when it leads above the
/// root (spec LIST-8). The root itself is the empty path.
fn joined(folder: &str, relative: &str) -> Option<String> {
    let mut segments: Vec<&str> = folder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    for segment in relative.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => segments.push(other),
        }
    }

    Some(segments.join("/"))
}

/// Whether a path is one of the paths given, or lies under one of them.
fn under(path: &str, places: &BTreeSet<String>) -> bool {
    places.iter().any(|place| {
        path.strip_prefix(place.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}
```

```rust path=src/graph.rs
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports of every script, and where each import leads.

use std::collections::BTreeMap;
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
use crate::resolve::{Outcome, Place, Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;

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
    let listing = files::list(&root, &mut notices)?;
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed.as_ref(), &mut notices)?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = packages::load(&root, files, &marks.data)?.checked(
        files,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    );
    let configs = tsconfig::load(
        &root,
        files,
        &marks.data,
        packages.as_ref().ok(),
        &mut notices,
    )?
    .checked();
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolved = Resolver::new(&listing, &packages, &configs).resolve(&root, &scripts)?;
    let mut document = json!({
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, and the
/// count of those that cannot be followed.
fn file(
    file: &File,
    marks: &Marks,
    packages: &Packages,
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
) -> Value {
    let mut printed = json!({
        "data": marks.data.contains(&file.path),
        "generated": marks.generated.contains(&file.path),
        "kind": file.kind.name(),
        "package": packages.of_file.get(&file.path),
        "path": file.path,
        "tracked": file.tracked,
    });

    if let (Some(read), Some(resolutions)) =
        (scripts.read.get(&file.path), resolved.get(&file.path))
    {
        printed["computedImports"] = json!(read.computed);
        printed["imports"] = read
            .imports
            .iter()
            .zip(resolutions)
            .map(|(each, resolution)| import(each, resolution))
            .collect();
    }

    printed
}

/// One import as it is printed, with where it leads and the rule that decided it (spec OUT-1).
fn import(import: &Import, resolution: &Resolution) -> Value {
    let mut printed = json!({
        "form": import.form.name(),
        "line": import.line,
        "query": import.has_query(),
        "rule": resolution.rule.name(),
        "specifier": import.specifier,
        "typeOnly": import.type_only,
    });

    match &resolution.outcome {
        Outcome::Files(files) => {
            printed["outcome"] = json!("files");
            printed["files"] = json!(files);
        }
        Outcome::Outside(place) => {
            printed["outcome"] = json!("outside");
            printed["outside"] = json!(place.name());

            if let Place::Package { name, listed } = place {
                printed["package"] = json!(name);
                printed["listed"] = json!(listed);
            }
        }
        Outcome::Ignored { path, from } => {
            printed["outcome"] = json!("ignored");
            printed["ignored"] = json!(path);

            if let Some(from) = from {
                printed["from"] = json!(from);
            }
        }
        Outcome::Unresolved => printed["outcome"] = json!("unresolved"),
    }

    printed
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
//! files, its packages, its scripts' imports and where they lead. It reports no findings yet.

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
mod resolve;
mod scripts;
mod stop;
mod tsconfig;

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
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
cargo build --release --locked && node scripts/check-floor.mjs
```

Expected: each exits 0. The Node tests report 47 passed; the Rust tests report 8 passed for
`tsconfig`, 12 for `resolve`, the others as in Task 1; the last line says what the binary asks of
the system, which is no more than the floor allows.

- [ ] **Step 6: Commit, push, and open the pull request as a draft on the third plan's**

```bash
git add Cargo.toml Cargo.lock .github/dependabot.yml scripts/dependabot.test.mjs src tests
git commit -m "feat(tsconfig): read tsconfig files for paths, baseUrl and outDir" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -u origin feat/reading-4-resolve
gh pr create --draft --base feat/reading-3-imports --head feat/reading-4-resolve --title "feat: where each import leads" --body "A draft, opened so that CI runs on all five targets. Not for merging until the owner has validated the plan."
gh pr checks --watch
```

Expected: the pull request's seven checks pass. Its description is written out in full when the plan
is handed over for validation.
