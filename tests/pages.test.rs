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
