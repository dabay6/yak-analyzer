# Reading a project, part 5: Vue components, MDX documents and HTML pages: implementation plan

**Goal:** `graph` reads the imports of every Vue component, MDX document and HTML page that is not
data, as it reads a script's, and leads a page's scripts where RES-7 says.

**Architecture:** `src/pages.rs` holds three small readers of the analyzer's own: the script blocks of
a Vue component, the `import` and `export … from` statements of an MDX document, and the `src` of an
HTML page's script elements. What they find is parsed by `src/imports.rs` as any script is.
`src/scripts.rs` reads the three kinds beside scripts, and `src/resolve.rs` follows a page's `src` by
RES-7 rather than RES-2.

**Spec:** docs/specs/2026-11-11-reading-a-project-design.md

**Implements:** PARSE-2, PARSE-3, PARSE-4, PARSE-5, PARSE-6, IMP-1, RES-7, CFG-2, CFG-4, RUN-5, FIX-2, FIX-4

This is the fifth of the item's plans. It is worked on its own branch, `feat/reading-5-pages`, made
from the fourth plan's branch, and its pull request is a draft stacked on the fourth plan's.

## Global constraints

- No crate is added: the spec rejects an existing parser for these formats, so the readers are the
  analyzer's own (spec, "Alternatives considered").
- A Vue block that does not parse is treated as PARSE-2 treats its file; an MDX document and an HTML
  page fail only by not being readable or not being valid UTF-8, and nothing in their text stops a
  run (PARSE-2, PARSE-4, PARSE-5).
- A data file is never read (PARSE-6).
- An HTML page's `src` never ends unresolved or ignored: it leads to a file or is outside, as loaded
  from elsewhere (RES-7).

## The decisions this plan makes

1. **The blocks of a Vue component** are its top-level elements, read in order: a comment is passed
   over, a `<template>` ends at the end tag that matches it, counting the `<template>` elements inside
   it, and any other block ends at the first end tag of its name, as Vue's own reader ends one. Names
   are read with their letter case, as Vue reads them.
2. **A Vue script block** is read in the language its `lang` names: `ts`, `tsx`, `js` or `jsx`, and
   JavaScript with JSX allowed when it names none; any other `lang`, an empty one included, is not
   read, with the notice `vue-block-not-read`. Its `src` comes first among its imports, with the form
   `src` and the line of the block's start tag, and is resolved by RES-2 like any import. Its content
   is parsed as a module and then as a script, and each import's line is its line in the file. A
   block that does not parse makes the component not parse, at the block's line in the file.
3. **The statements of an MDX document:** front matter, a block that opens with `---` on the first
   line and closes with `---` on a later one, is passed over, and so is every fenced code block, of
   three or more backticks or tildes indented by up to three spaces. A statement begins at the very
   start of a line with the word `import` or `export` and runs to the next blank line, as MDX's own
   reader takes one. It is parsed as a JavaScript module with JSX allowed, and only its `import` and
   `export … from` statements are kept. One that does not parse is text, with the notice
   `mdx-line-not-read` naming its first line. Nothing else in the document is read, an `import(…)` in
   its JSX included, so its `computedImports` is always 0.
4. **An HTML page is read leniently, as a browser reads markup:** a comment, a `<!…>` or `<?…>`
   declaration and an end tag are passed over; a tag's name and its attributes' names are read in any
   letter case; a value may be quoted or not. The content of `script`, `style`, `textarea`, `title`,
   `xmp`, `iframe`, `noembed`, `noframes` and `noscript` is text up to its end tag, as a browser with
   scripting reads it, and a `<script … />` still opens one. A script's `src` is trimmed, and `&amp;`
   in it read as `&`; no other character reference is decoded.
5. **RES-7 is its own rule,** printed as `page`. The query and fragment of a `src` are dropped (IMP-4).
   One that begins with `//`, or is a URL, names another host. The places are tried in order with a
   place tried once only, a place above the root is passed over, and a place where the path leads into
   another repository or through a link gives that outcome; a path that leads to no file anywhere is
   outside, as loaded from elsewhere.
6. **The files the analyzer reads,** for CFG-4, now include Vue components, MDX documents and HTML
   pages, as the fourth plan's decision 15 said; `scripts::reads` is the one list of them.
7. **The notice kinds this plan adds:** `vue-block-not-read` and `mdx-line-not-read`.

## Review focus

- **A `</script>` inside a string of a Vue script block** ends the block there, as it does for Vue
  itself; the rest then fails to parse. No fixture holds one, since Vue refuses the component too.
- **An MDX statement with a blank line inside it** is cut at the blank line and read as text, with a
  notice; MDX's own reader refuses such a document. No fixture holds one.
- **Character references other than `&amp;`** in a page's `src` stay as written, so such a `src`
  leads to no file and is outside. No fixture holds one.
- **A tsconfig that only a Vue component, an MDX document or an HTML page depends on** stops the run
  when it cannot be read, by decision 6; the fourth plan's fixtures check the rule for scripts.
- **The readers are hand-rolled**, which the spec names as a risk: a missed form would leave a file
  looking unused later. The fixtures hold the forms the design spec names; the sixth plan's check on
  the named repositories is the wider test.

### Task 1: the three readers, and where a page's scripts lead

**Implements:** PARSE-2, PARSE-3, PARSE-4, PARSE-5, PARSE-6, IMP-1, RES-7, CFG-2, CFG-4, RUN-5, FIX-2, FIX-4

**Test:** `cargo test --release --locked --test pages --test imports --test file-list`

**Files:**
- Create: `src/pages.rs`
- Modify: `src/imports.rs`, `src/scripts.rs`, `src/resolve.rs`, `src/tsconfig.rs`, `src/main.rs`,
  `Cargo.toml`
- Test: `tests/pages.test.rs`, `tests/imports.test.rs`, `tests/file-list.test.rs`

**Interfaces:**
- Consumes: the third plan's `imports::read`, `Read`, `Import` and `Broken`, and `scripts::read`;
  the fourth plan's `resolve::Resolver` (`locate`, `own_package`) and `tsconfig::load`.
- Produces: `pages::vue`, `pages::mdx`, `pages::html` and `pages::Skipped`; `imports::read_block`,
  `imports::read_statements` and `imports::Form::Src`; `scripts::reads`; `resolve::Rule::Page`. The
  sixth plan's check reads what they print.

- [ ] **Step 1: Write the failing tests**

The second and third plans' tests that pinned Vue components, MDX documents and HTML pages as unread
change with this task.

```rust path=tests/pages.test.rs
//! Runs the built binary on made-up projects and checks the imports it finds in Vue components, MDX
//! documents and HTML pages, and where a page's scripts lead (spec PARSE-2 to PARSE-5, IMP-1, RES-7,
//! CFG-2, FIX-2, FIX-4).

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

/// The imports of a file, each with its specifier, form, line and outcome, and the files it leads to or
/// the place outside.
fn imports(document: &Value, path: &str) -> Vec<Value> {
    file(document, path)["imports"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} was read"))
        .iter()
        .map(|import| {
            let mut found = json!({
                "specifier": import["specifier"],
                "form": import["form"],
                "line": import["line"],
                "rule": import["rule"],
                "outcome": import["outcome"],
            });

            for key in ["files", "outside", "typeOnly", "query"] {
                if import[key] == json!(true)
                    || (key != "typeOnly" && key != "query" && !import[key].is_null())
                {
                    found[key] = import[key].clone();
                }
            }

            found
        })
        .collect()
}

/// An import that leads to the files given.
fn to_files(specifier: &str, form: &str, line: usize, rule: &str, found: &[&str]) -> Value {
    json!({ "specifier": specifier, "form": form, "line": line, "rule": rule, "outcome": "files", "files": found })
}

#[test]
fn reads_the_script_blocks_of_a_vue_component_in_their_languages() {
    let project = Project::new("pages-vue");

    project
        .write(
            "tsconfig.json",
            r#"{ "include": ["src"], "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#,
        )
        .write(
            "src/App.vue",
            "<template>\n  <div>\n    <template v-if=\"ok\"><span>import nothing from here</span></template>\n  </div>\n\
             </template>\n\n<script setup lang=\"ts\">\nimport Child from './Child.vue';\nimport type { Props } from './types';\n\
             import util from '@/util';\n</script>\n\n<script>\nexport default { components: { Other: () => import('./Other.vue') } };\n\
             </script>\n\n<style>\n.a { background: url('./not-an-import.png'); }\n</style>\n",
        )
        .write("src/Child.vue", "<template><p /></template>\n")
        .write("src/Other.vue", "<script>import('./' + name);</script>\n")
        .write("src/types.ts", "")
        .write("src/util.ts", "")
        .write(
            "src/Plain.vue",
            "<script src=\"./plain-logic.js\"></script>\n<template><p /></template>\n",
        )
        .write("src/plain-logic.js", "")
        .write(
            "src/Coffee.vue",
            "<template><p /></template>\n<script lang=\"coffee\">\nimport x from './x'\n</script>\n",
        )
        .commit();

    let document = project.graph();

    // Both blocks, `<script setup>` and a plain one, in their own languages; nothing in the template or
    // the style (spec FIX-2, pattern 16, PARSE-3).
    assert_eq!(
        imports(&document, "src/App.vue"),
        [
            to_files("./Child.vue", "import", 8, "relative", &["src/Child.vue"]),
            json!({
                "specifier": "./types", "form": "import", "line": 9, "rule": "relative", "outcome": "files",
                "files": ["src/types.ts"], "typeOnly": true,
            }),
            // A component is governed as a `.ts` file of its name would be (spec CFG-2).
            to_files("@/util", "import", 10, "paths", &["src/util.ts"]),
            to_files("./Other.vue", "dynamic", 14, "relative", &["src/Other.vue"]),
        ]
    );
    assert_eq!(
        file(&document, "src/Other.vue")["computedImports"],
        json!(1)
    );
    // A block's `src` is an import (spec IMP-1).
    assert_eq!(
        imports(&document, "src/Plain.vue"),
        [to_files(
            "./plain-logic.js",
            "src",
            1,
            "relative",
            &["src/plain-logic.js"]
        )]
    );
    // A block in another language is not read, with a notice.
    assert_eq!(imports(&document, "src/Coffee.vue"), Vec::<Value>::new());
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "vue-block-not-read"]
    );
    assert_eq!(
        document["notices"][1],
        json!({
            "file": "src/Coffee.vue",
            "kind": "vue-block-not-read",
            "sentence": "The script block on line 2 is in the language `coffee`, which the analyzer does not read, so its \
                         imports are not read.",
        })
    );
}

#[test]
fn stops_on_a_tracked_vue_block_that_does_not_parse_at_its_line_in_the_file() {
    let project = Project::new("pages-vue-broken");

    project
        .write("src/Fine.vue", "<script>\nexport default {};\n</script>\n")
        .commit()
        .write("src/Scratch.vue", "<script>\nconst = ;\n</script>\n");

    // An untracked component that does not parse is skipped with a notice (spec PARSE-2).
    let document = project.graph();

    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "untracked-file-skipped"]
    );
    assert_eq!(file(&document, "src/Scratch.vue").get("imports"), None);

    project
        .write(
            "src/Broken.vue",
            "<template>\n  <p />\n</template>\n<script>\nconst = ;\n</script>\n",
        )
        .commit();

    let line = stopped(&project.run(|_| {}));

    assert!(
        line.starts_with("yak-analyzer: the file src/Broken.vue does not parse (line 5: "),
        "{line}"
    );
    assert!(
        line.ends_with("); fix it, or declare it as data if it is broken on purpose"),
        "{line}"
    );
}

#[test]
fn reads_only_the_import_and_export_statements_of_an_mdx_document() {
    let project = Project::new("pages-mdx");
    let fence = "`".repeat(3);

    project
        .write(
            "docs/intro.mdx",
            &format!(
                "---\ntitle: Intro\nimport: not-this\n---\nimport {{ Chart }} from '../src/Chart.tsx'\n\
                 export {{ meta }} from './meta.js'\nexport const shown = true\n\nSome prose.\n\n\
                 import the right tools before you start.\n\n{fence}js\nimport fake from './fake'\n{fence}\n\n\
                 ~~~\nimport tilde from './tilde'\n~~~\n\nexport * from './all'\n\n<Chart load={{() => import('./lazy')}} />\n"
            ),
        )
        .write("src/Chart.tsx", "")
        .write("docs/meta.ts", "")
        .write("docs/all.ts", "")
        .commit();

    let document = project.graph();

    // The file only the document imports is led to (spec FIX-2, pattern 16).
    assert_eq!(
        imports(&document, "docs/intro.mdx"),
        [
            to_files(
                "../src/Chart.tsx",
                "import",
                5,
                "relative",
                &["src/Chart.tsx"]
            ),
            to_files("./meta.js", "export", 6, "relative", &["docs/meta.ts"]),
            to_files("./all", "export-all", 21, "relative", &["docs/all.ts"]),
        ]
    );
    assert_eq!(
        file(&document, "docs/intro.mdx")["computedImports"],
        json!(0)
    );
    // Prose that begins a line with "import" is text, with a notice (spec FIX-4, PARSE-4).
    assert_eq!(
        notices(&document),
        ["no-hand-off-file", "mdx-line-not-read"]
    );
    assert_eq!(
        document["notices"][1]["sentence"],
        json!(
            "Line 11 begins like an `import` or `export` statement and does not parse as one, so it is read as text."
        )
    );
}

#[test]
fn reads_the_script_sources_of_an_html_page_and_where_each_leads() {
    let project = Project::new("pages-html");

    project
        .write("web/package.json", r#"{ "name": "web" }"#)
        .write(
            "web/pages/about.html",
            "<!doctype html>\n<html>\n<head>\n  <!-- <script src=\"./commented.js\"></script> -->\n\
             <SCRIPT SRC=\"./about.js\" type=\"module\"></SCRIPT>\n  <script src=/main.js></script>\n  <script>\n\
             \x20   document.write('<script src=\"./written.js\"></script>');\n  </script>\n\
             \x20 <script src=\"https://cdn.example/lib.js\"></script>\n  <script src=\"//cdn.example/other.js\"></script>\n\
             \x20 <script src=\"./missing.js\"></script>\n  <script src='/shared.js?v=2&amp;x=1'></script>\n\
             \x20 <style>.x::before { content: '<script src=\"./styled.js\">'; }</style>\n</head>\n</html>\n",
        )
        .write("web/pages/about.ts", "")
        .write("web/main.ts", "")
        .write("shared.js", "")
        .commit();

    let document = project.graph();
    let elsewhere = |specifier: &str, line: usize| {
        json!({
            "specifier": specifier, "form": "src", "line": line, "rule": "page", "outcome": "outside",
            "outside": "elsewhere",
        })
    };

    // A page that is not `index.html`, by a path from its folder and one from the server's root (spec
    // FIX-2, pattern 14); nothing in a comment, a script's content or a style (spec PARSE-5).
    assert_eq!(
        imports(&document, "web/pages/about.html"),
        [
            to_files("./about.js", "src", 5, "page", &["web/pages/about.ts"]),
            // From the package's folder, skipping the page's (spec RES-7).
            to_files("/main.js", "src", 6, "page", &["web/main.ts"]),
            elsewhere("https://cdn.example/lib.js", 10),
            elsewhere("//cdn.example/other.js", 11),
            elsewhere("./missing.js", 12),
            // From the root, where the package's folder has none.
            json!({
                "specifier": "/shared.js?v=2&x=1", "form": "src", "line": 13, "rule": "page", "outcome": "files",
                "files": ["shared.js"], "query": true,
            }),
        ]
    );
    assert_eq!(notices(&document), ["no-hand-off-file"]);
}

#[test]
fn never_reads_a_component_a_document_or_a_page_declared_as_data() {
    let project = Project::new("pages-data");

    project
        .write("fixtures/Broken.vue", "<script>\nconst = ;\n</script>\n")
        .write("fixtures/notes.mdx", "import a from './a'\n")
        .write("fixtures/page.html", "<script src=\"./b.js\"></script>\n")
        .write("src/main.ts", "")
        .commit();

    let declared = json!([{
        "name": "typescript",
        "interfaceVersion": 1,
        "pack": {},
        "project": { "data": ["fixtures/**"] },
    }]);
    let file_path = project.hand_off(&project.root, &["."], declared);
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file_path]);
    }));

    // Listed, and never read, so a broken component among data stops nothing (spec PARSE-6, KEY-4).
    assert_eq!(
        paths(&document),
        [
            "fixtures/Broken.vue",
            "fixtures/notes.mdx",
            "fixtures/page.html",
            "src/main.ts"
        ]
    );

    for path in [
        "fixtures/Broken.vue",
        "fixtures/notes.mdx",
        "fixtures/page.html",
    ] {
        assert_eq!(file(&document, path).get("imports"), None, "{path}");
    }
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
fn reads_no_file_of_a_kind_that_holds_no_imports() {
    let project = Project::new("imports-not-scripts");

    // Vue components, MDX documents and HTML pages are the pages tests' to check.
    project
        .write("README.md", "import a from './a';\n")
        .write("src/styles.css", "@import './b.css';\n")
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
    // A script, a component, a document and a page are read, so each has its imports, which in these
    // empty files are none.
    let file = |path: &str, kind: &str, tracked: bool| {
        let mut expected = json!({
            "data": false,
            "generated": false,
            "kind": kind,
            "package": ".",
            "path": path,
            "tracked": tracked,
        });

        if ["script", "declaration", "vue", "mdx", "page"].contains(&kind) {
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
cargo test --release --locked --test pages --test imports --test file-list
```

Expected: cargo refuses, because the manifest names no test called `pages` yet.

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

[[test]]
name = "tsconfig"
path = "tests/tsconfig.test.rs"

[[test]]
name = "pages"
path = "tests/pages.test.rs"

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

- [ ] **Step 4: Write the readers, the reading of the three kinds, and RES-7**

```rust path=src/pages.rs
//! Reading the files that hold scripts inside another format: a Vue component's script blocks, an MDX
//! document's `import` and `export … from` statements, and an HTML page's script elements (spec PARSE-3
//! to PARSE-5, IMP-1).
//!
//! Each reader is the analyzer's own and small. It finds only what the analyzer needs from its format,
//! and the scripts it finds are parsed as any script is.

use std::ops::Range;

use crate::imports::{self, Broken, Form, Import, Read};

/// The elements of an HTML page whose content is text a browser does not read as markup.
const RAW_TEXT: [&str; 9] = [
    "script", "style", "textarea", "title", "xmp", "iframe", "noembed", "noframes", "noscript",
];

/// What a reader passed over that a notice is due for.
#[derive(Debug, PartialEq, Eq)]
pub enum Skipped {
    /// A Vue script block in a language the analyzer does not read, with its line and its `lang`.
    Language { line: usize, language: String },
    /// A line of an MDX document that begins like an `import` or `export` statement and does not parse
    /// as one.
    Statement { line: usize },
}

/// The script blocks of a Vue component: each block's `src` and the imports of its content, in the
/// language its `lang` names; a block in another language is passed over. A block that does not parse
/// makes the component not parse, at the block's line in the file (spec PARSE-3).
pub fn vue(text: &str) -> Result<(Read, Vec<Skipped>), Broken> {
    let mut read = Read {
        imports: Vec::new(),
        computed: 0,
    };
    let mut skipped = Vec::new();

    for block in blocks(text)
        .into_iter()
        .filter(|block| block.tag.name == "script")
    {
        let line = line_at(text, block.start);

        if let Some(src) = block.tag.attribute("src") {
            read.imports.push(source(src, line));
        }

        // The lines of the file above the block's content.
        let above = line_at(text, block.content.start) - 1;
        let lang = block.tag.attribute("lang");

        match imports::read_block(&text[block.content.clone()], lang) {
            None => skipped.push(Skipped::Language {
                line,
                language: lang.unwrap_or_default().to_owned(),
            }),
            Some(Ok(found)) => {
                read.imports
                    .extend(found.imports.into_iter().map(|import| Import {
                        line: import.line + above,
                        ..import
                    }));
                read.computed += found.computed;
            }
            Some(Err(broken)) => {
                return Err(Broken {
                    line: broken.line.map(|line| line + above),
                    message: broken.message,
                });
            }
        }
    }

    Ok((read, skipped))
}

/// The `import` and `export … from` statements of an MDX document, and nothing else (spec PARSE-4). A
/// statement begins at the start of a line with `import` or `export`, outside the front matter and
/// outside a fenced code block, and runs to the next blank line. One that does not parse is text.
pub fn mdx(text: &str) -> (Read, Vec<Skipped>) {
    let lines: Vec<(usize, &str)> = text
        .split_inclusive('\n')
        .scan(0, |start, line| {
            let at = *start;

            *start += line.len();

            Some((at, line.trim_end_matches(['\n', '\r'])))
        })
        .collect();
    let mut imports = Vec::new();
    let mut skipped = Vec::new();
    let mut index = front_matter(&lines);
    let mut fence: Option<(char, usize)> = None;

    while let Some((start, line)) = lines.get(index).copied() {
        if let Some((mark, size)) = fence {
            if closes(line, mark, size) {
                fence = None;
            }

            index += 1;

            continue;
        }

        if let Some(opened) = opens(line) {
            fence = Some(opened);
            index += 1;

            continue;
        }

        if !begins_statement(line) {
            index += 1;

            continue;
        }

        let end = (index + 1..lines.len())
            .find(|after| lines[*after].1.trim().is_empty())
            .unwrap_or(lines.len());
        let stop = lines.get(end).map_or(text.len(), |(at, _)| *at);

        match imports::read_statements(&text[start..stop]) {
            Ok(found) => imports.extend(found.into_iter().map(|import| Import {
                line: import.line + index,
                ..import
            })),
            Err(_) => skipped.push(Skipped::Statement { line: index + 1 }),
        }

        index = end;
    }

    (
        Read {
            imports,
            computed: 0,
        },
        skipped,
    )
}

/// The `src` of every script element of an HTML page, read leniently, as a browser reads markup; nothing
/// else, and not the content of a script element (spec PARSE-5).
pub fn html(text: &str) -> Read {
    let lowered = text.to_ascii_lowercase();
    let mut imports = Vec::new();
    let mut at = 0;

    while let Some(found) = text[at..].find('<') {
        let start = at + found;
        let rest = &text[start..];

        if rest.starts_with("<!--") {
            at = past(text, start, "-->");
        } else if rest.starts_with("<!") || rest.starts_with("<?") || rest.starts_with("</") {
            at = past(text, start, ">");
        } else if let Some(tag) = open_tag(text, start, true) {
            if tag.name == "script"
                && let Some(src) = tag.attribute("src").map(str::trim)
                && !src.is_empty()
            {
                imports.push(source(&src.replace("&amp;", "&"), line_at(text, start)));
            }

            // A browser reads the content of these as text up to their end tag, whatever it holds.
            at = if RAW_TEXT.contains(&tag.name.as_str()) {
                let end = format!("</{}", tag.name);

                lowered[tag.end..]
                    .find(&end)
                    .map_or(text.len(), |found| past(text, tag.end + found, ">"))
            } else {
                tag.end
            };
        } else {
            at = start + 1;
        }
    }

    Read {
        imports,
        computed: 0,
    }
}

/// The import a `src` attribute makes.
fn source(src: &str, line: usize) -> Import {
    Import {
        specifier: src.to_owned(),
        form: Form::Src,
        type_only: false,
        line,
    }
}

/// The line an offset of a text lies on, counted from one.
fn line_at(text: &str, offset: usize) -> usize {
    text[..offset].matches('\n').count() + 1
}

/// The offset just past the first `end` at or after `from`, or the end of the text.
fn past(text: &str, from: usize, end: &str) -> usize {
    text[from..]
        .find(end)
        .map_or(text.len(), |found| from + found + end.len())
}

/// One start tag.
#[derive(Debug)]
struct Tag {
    /// Its name.
    name: String,
    /// Its attributes, each with its value where it has one.
    attributes: Vec<(String, Option<String>)>,
    /// The offset just past its `>`.
    end: usize,
    /// Whether it ends with `/>`.
    closed: bool,
}

impl Tag {
    /// The value of an attribute, the first of the name; an attribute with no value is empty.
    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(each, _)| each == name)
            .map(|(_, value)| value.as_deref().unwrap_or_default())
    }
}

/// The start tag at an offset that holds `<`, where one begins there: a name that begins with a letter,
/// then attributes, to `>`. With `lower`, the tag's name and its attributes' names are put in lower
/// case, as HTML reads them.
fn open_tag(text: &str, start: usize, lower: bool) -> Option<Tag> {
    let bytes = text.as_bytes();
    let named = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'-';
    let mut at = start + 1;

    if !bytes.get(at)?.is_ascii_alphabetic() {
        return None;
    }

    while bytes.get(at).copied().is_some_and(named) {
        at += 1;
    }

    let fold = |name: &str| {
        if lower {
            name.to_ascii_lowercase()
        } else {
            name.to_owned()
        }
    };
    let name = fold(&text[start + 1..at]);
    let mut attributes = Vec::new();

    loop {
        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }

        match bytes[at] {
            b'>' => {
                return Some(Tag {
                    name,
                    attributes,
                    end: at + 1,
                    closed: false,
                });
            }
            b'/' if bytes.get(at + 1) == Some(&b'>') => {
                return Some(Tag {
                    name,
                    attributes,
                    end: at + 2,
                    closed: true,
                });
            }
            b'/' | b'=' => {
                at += 1;

                continue;
            }
            _ => {}
        }

        let begins = at;

        while bytes
            .get(at)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !b"=>/".contains(byte))
        {
            at += 1;
        }

        let attribute = fold(&text[begins..at]);

        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }

        if bytes[at] != b'=' {
            attributes.push((attribute, None));

            continue;
        }

        at += 1;

        while bytes.get(at)?.is_ascii_whitespace() {
            at += 1;
        }

        let value = match bytes[at] {
            quote @ (b'"' | b'\'') => {
                let close = text[at + 1..].find(char::from(quote))? + at + 1;
                let value = &text[at + 1..close];

                at = close + 1;
                value
            }
            _ => {
                let begins = at;

                while bytes
                    .get(at)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
                {
                    at += 1;
                }

                &text[begins..at]
            }
        };

        attributes.push((attribute, Some(value.to_owned())));
    }
}

/// One top-level block of a Vue component.
#[derive(Debug)]
struct Block {
    /// Its start tag.
    tag: Tag,
    /// The offset of its `<`.
    start: usize,
    /// Its content, between its start tag and its end tag.
    content: Range<usize>,
}

/// The top-level blocks of a Vue component, in order. A `<template>` may hold `<template>` elements;
/// every other block ends at the first end tag of its name, as Vue's own reader ends one.
fn blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut at = 0;

    while let Some(found) = text[at..].find('<') {
        let start = at + found;

        if text[start..].starts_with("<!--") {
            at = past(text, start, "-->");

            continue;
        }

        let Some(tag) = open_tag(text, start, false) else {
            at = start + 1;

            continue;
        };

        let (content, after) = if tag.closed {
            (tag.end..tag.end, tag.end)
        } else {
            closing(text, &tag.name, tag.end)
        };

        at = after;
        blocks.push(Block {
            tag,
            start,
            content,
        });
    }

    blocks
}

/// Where the content of a block that begins at `from` ends, and the offset past its end tag; the end
/// of the text when it has none. Nested blocks of the same name are counted for a `<template>` only.
fn closing(text: &str, name: &str, from: usize) -> (Range<usize>, usize) {
    let opening = format!("<{name}");
    let ending = format!("</{name}");
    let bounded = |at: usize, prefix: &str| {
        text[at..].starts_with(prefix)
            && text[at + prefix.len()..]
                .bytes()
                .next()
                .is_none_or(|byte| byte.is_ascii_whitespace() || byte == b'>' || byte == b'/')
    };
    let mut depth = 0_usize;
    let mut at = from;

    while let Some(found) = text[at..].find('<') {
        let start = at + found;

        if bounded(start, &ending) {
            if depth == 0 {
                return (from..start, past(text, start, ">"));
            }

            depth -= 1;
        } else if name == "template"
            && bounded(start, &opening)
            && open_tag(text, start, false).is_some_and(|tag| !tag.closed)
        {
            depth += 1;
        }

        at = start + 1;
    }

    (from..text.len(), text.len())
}

/// The first line after an MDX document's front matter: a block that opens with `---` on the first
/// line and closes with `---` on a later one. Without one, the first line.
fn front_matter(lines: &[(usize, &str)]) -> usize {
    if lines
        .first()
        .is_none_or(|(_, line)| line.trim_end() != "---")
    {
        return 0;
    }

    lines
        .iter()
        .skip(1)
        .position(|(_, line)| line.trim_end() == "---")
        .map_or(0, |closing| closing + 2)
}

/// The mark and length of the fence a line opens: up to three spaces, then three or more backticks or
/// tildes.
fn opens(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start_matches(' ');

    if line.len() - trimmed.len() > 3 {
        return None;
    }

    let mark = trimmed
        .chars()
        .next()
        .filter(|mark| ['`', '~'].contains(mark))?;
    let size = trimmed.chars().take_while(|each| *each == mark).count();

    (size >= 3).then_some((mark, size))
}

/// Whether a line closes a fence of the mark and length given: up to three spaces, at least as many of
/// the mark, and nothing after but spaces.
fn closes(line: &str, mark: char, size: usize) -> bool {
    let trimmed = line.trim_start_matches(' ');
    let run = trimmed.chars().take_while(|each| *each == mark).count();

    line.len() - trimmed.len() <= 3 && run >= size && trimmed[run..].trim().is_empty()
}

/// Whether a line begins like an `import` or `export` statement: the word at its very start, followed
/// by anything that cannot continue a name.
fn begins_statement(line: &str) -> bool {
    ["import", "export"].iter().any(|word| {
        line.strip_prefix(word).is_some_and(|rest| {
            rest.chars()
                .next()
                .is_none_or(|next| !(next.is_alphanumeric() || next == '_' || next == '$'))
        })
    })
}
```

```rust path=src/imports.rs
//! What an import is: parsing a script and finding every import in it (spec PARSE-1 to PARSE-4, IMP-1
//! to IMP-5).

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
    /// The `src` of a Vue component's script block, or of an HTML page's script element.
    Src,
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
            Self::Src => "src",
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
    read_as(text, source_type(path))
}

/// Parses a Vue component's script block in the language its `lang` names: `ts`, `tsx`, `js` or
/// `jsx`, and JavaScript when it names none; nothing for a block in any other language (spec PARSE-3).
pub fn read_block(text: &str, lang: Option<&str>) -> Option<Result<Read, Broken>> {
    let source_type = match lang {
        None | Some("js" | "jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some(_) => return None,
    };

    Some(read_as(text, source_type))
}

/// Parses statements of an MDX document as a JavaScript module and keeps the `import` and `export …
/// from` statements among them, and nothing else (spec PARSE-4).
pub fn read_statements(text: &str) -> Result<Vec<Import>, Broken> {
    parse(text, SourceType::jsx().with_module(true)).map(|read| {
        read.imports
            .into_iter()
            .filter(|import| matches!(import.form, Form::Import | Form::Export | Form::ExportAll))
            .collect()
    })
}

/// Parses as a module first and then as a script.
fn read_as(text: &str, source_type: SourceType) -> Result<Read, Broken> {
    let module = source_type.with_module(true);

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
//! Reading every file of the list that holds imports: its content from disk, parsed, and its imports
//! found. Scripts and type declaration files are parsed whole; Vue components, MDX documents and HTML
//! pages by their own readers (spec PARSE-1 to PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::files::{File, Kind};
use crate::imports::{self, Broken, Read};
use crate::notice::Notice;
use crate::pages::{self, Skipped};
use crate::stop::Stop;

/// What reading the scripts found.
#[derive(Debug, Default)]
pub struct Scripts {
    /// What each file that was read holds, by its path.
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

/// Reads every script, type declaration file, Vue component, MDX document and HTML page in the list
/// that is not data, in the order of the list. A tracked file that cannot be read stops the run; an
/// untracked one is skipped with a notice (spec PARSE-2).
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
) -> Result<Scripts, Stop> {
    let mut scripts = Scripts::default();

    for file in files {
        if !reads(file.kind) || data.contains(&file.path) {
            continue;
        }

        match one(root, file) {
            Ok((read, skipped)) => {
                notices.extend(
                    skipped
                        .into_iter()
                        .map(|each| passed_over(&file.path, each)),
                );
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

/// Whether the analyzer reads the imports of a file of the kind given (spec PARSE-1, PARSE-3 to
/// PARSE-5).
pub const fn reads(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Script | Kind::Declaration | Kind::Vue | Kind::Mdx | Kind::Page
    )
}

/// The notice for what a reader passed over (spec PARSE-3, PARSE-4).
fn passed_over(path: &str, skipped: Skipped) -> Notice {
    match skipped {
        Skipped::Language { line, language } => Notice::new(
            "vue-block-not-read",
            Some(path.to_owned()),
            format!(
                "The script block on line {line} is in the language `{language}`, which the analyzer does not read, \
                 so its imports are not read."
            ),
        ),
        Skipped::Statement { line } => Notice::new(
            "mdx-line-not-read",
            Some(path.to_owned()),
            format!(
                "Line {line} begins like an `import` or `export` statement and does not parse as one, so it is \
                 read as text."
            ),
        ),
    }
}

/// Reads one file and finds its imports, with what its reader passed over.
fn one(root: &Path, file: &File) -> Result<(Read, Vec<Skipped>), Unread> {
    let path = file.path.as_str();
    let bytes = fs::read(root.join(path)).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Unread::Gone,
        _ => Unread::Broken(format!("cannot be read ({error})")),
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let text =
        std::str::from_utf8(bytes).map_err(|_| Unread::Broken("is not valid UTF-8".to_owned()))?;

    let broken = |Broken { line, message }| {
        Unread::Broken(match line {
            Some(line) => format!("does not parse (line {line}: {message})"),
            None => format!("does not parse ({message})"),
        })
    };

    // An MDX document and an HTML page hold nothing that fails to parse (spec PARSE-2).
    match file.kind {
        Kind::Vue => pages::vue(text).map_err(broken),
        Kind::Mdx => Ok(pages::mdx(text)),
        Kind::Page => Ok((pages::html(text), Vec::new())),
        _ => imports::read(path, text)
            .map(|read| (read, Vec::new()))
            .map_err(broken),
    }
}
```

```rust path=src/resolve.rs
//! Where each import leads (spec RES-1 to RES-10, MAP-1 to MAP-4, CFG-2, CFG-3, LIST-5).
//!
//! A path is looked for in the file list, never on disk, with exact letter case. What leads to no file
//! is asked about once, with one start of git for all of them, to tell an ignored path from a missing
//! one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Map, Value};

use crate::files::Listing;
use crate::git;
use crate::imports::{Form, Import};
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
    /// RES-7: the `src` of an HTML page's script element.
    Page,
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
            Self::Page => "page",
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

        if import.form == Form::Src && importing.ends_with(".html") {
            return Ok((self.page(importing, specifier), Rule::Page));
        }

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

    /// The `src` of an HTML page's script element: a path tried from the page's folder, then from the
    /// folder of the page's package, then from the root, one that begins with `/` skipping the first,
    /// each as RES-3 finds a file. One that names another host, or leads to no file, is outside, as
    /// loaded from elsewhere (spec RES-7).
    fn page(&self, importing: &str, specifier: &str) -> Found {
        let elsewhere = Found::Done(Outcome::Outside(Place::Elsewhere));

        if specifier.starts_with("//") || is_url(specifier) {
            return elsewhere;
        }

        let page = importing.rsplit_once('/').map_or("", |(folder, _)| folder);
        let package = self
            .own_package(importing)
            .map_or("", |package| folder_path(&package.folder));
        let (path, places) = match specifier.strip_prefix('/') {
            Some(path) => (path, vec![package, ""]),
            None => (specifier, vec![page, package, ""]),
        };
        let mut tried = BTreeSet::new();

        for folder in places {
            if !tried.insert(folder) {
                continue;
            }

            if let Some(found) = joined(folder, path).map(|joined| self.locate(&joined, true))
                && !matches!(found, Found::Missing(_))
            {
                return found;
            }
        }

        elsewhere
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
use crate::scripts;
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

    for file in files
        .iter()
        .filter(|file| scripts::reads(file.kind) && !data.contains(&file.path))
    {
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
mod pages;
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

Expected: each exits 0. The Node tests report 47 passed; the Rust tests report 7 passed in the
binary's own tests, 6 for `cli`, 5 for `declarations`, 16 for `file-list`, 10 for `hand-off`, 2 for
`hand-off-schema`, 11 for `imports`, 9 for `packages`, 5 for `pages`, 12 for `resolve` and 8 for
`tsconfig`; the last line says what the binary asks of the system, which is no more than the floor
allows.

- [ ] **Step 6: Commit, push, and open the pull request as a draft on the fourth plan's**

```bash
git add Cargo.toml src tests
git commit -m "feat(pages): read Vue components, MDX documents and HTML pages" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git push -u origin feat/reading-5-pages
gh pr create --draft --base feat/reading-4-resolve --head feat/reading-5-pages --title "feat: read Vue components, MDX documents and HTML pages" --body "A draft, opened so that CI runs on all five targets. Not for merging until the owner has validated the plan."
gh pr checks --watch
```

Expected: one commit on `feat/reading-5-pages`, and the pull request's seven checks pass. Its
description is written out in full when the plan is handed over for validation.
