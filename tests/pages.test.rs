//! Runs the built binary on made-up projects and checks the imports it finds in Vue components, MDX
//! documents and HTML pages, where a page's scripts lead, and the bindings a component's template and a
//! document's text use (spec PARSE-2 to PARSE-5, IMP-1, RES-7, CFG-2, FIX-2, FIX-4; dead-code spec
//! USE-4, USE-5, FIX-1; comparison spec DYN-1).

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

/// The uses of a file's exports a document holds, in its order: each with the export's name, `*` for
/// every export, and the file and line it counts from.
fn uses<'a>(document: &'a Value, path: &str) -> Vec<(&'a str, &'a str, u64)> {
    document["uses"]
        .as_array()
        .expect("a list of uses")
        .iter()
        .filter(|used| used["file"] == json!(path))
        .map(|used| {
            (
                used["name"].as_str().expect("a name"),
                used["by"].as_str().expect("a file"),
                used["line"].as_u64().expect("a line"),
            )
        })
        .collect()
}

/// The names of the exports `graph` prints for a file.
fn exported<'a>(document: &'a Value, path: &str) -> Vec<&'a str> {
    file(document, path)["exports"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} was read"))
        .iter()
        .map(|export| export["name"].as_str().expect("a name"))
        .collect()
}

#[test]
fn a_vue_template_uses_the_bindings_it_names() {
    let project = Project::new("pages-vue-uses");

    project
        .write(
            "src/App.vue",
            "<template>\n\
             \x20 <Child :value=\"format(count)\" />\n\
             \x20 <other-child class=\"plain\" />\n\
             \x20 <p>{{ label }} {{ texts.title }}</p>\n\
             </template>\n\
             \n\
             <script setup lang=\"ts\">\n\
             import Child from './Child';\n\
             import OtherChild from './OtherChild';\n\
             import Unused from './Unused';\n\
             import { format, spare } from './format';\n\
             import * as texts from './texts';\n\
             const count = 1;\n\
             </script>\n\
             \n\
             <script lang=\"ts\">\n\
             import plain from './plain';\n\
             export const label = 'x';\n\
             export const quiet = 'y';\n\
             export default { name: 'App' };\n\
             </script>\n",
        )
        .write("src/Child.ts", "export default {};\n")
        .write("src/OtherChild.ts", "export default {};\n")
        .write("src/Unused.ts", "export default {};\n")
        .write(
            "src/format.ts",
            "export const format = String;\nexport const spare = 1;\n",
        )
        .write("src/texts.ts", "export const title = 't';\n")
        .write("src/plain.ts", "export default 1;\n")
        .commit();

    let document = project.graph();

    // A binding a `<script setup>` block imports is used where the template names it: as a tag, in
    // PascalCase or in kebab-case, or as a word anywhere in its text. One it does not name is not used
    // (spec USE-4, FIX-1 pattern 16).
    assert_eq!(
        uses(&document, "src/Child.ts"),
        [("default", "src/App.vue", 2)]
    );
    assert_eq!(
        uses(&document, "src/OtherChild.ts"),
        [("default", "src/App.vue", 3)]
    );
    assert_eq!(uses(&document, "src/Unused.ts"), []);
    assert_eq!(
        uses(&document, "src/format.ts"),
        [("format", "src/App.vue", 2)]
    );
    assert_eq!(uses(&document, "src/texts.ts"), [("*", "src/App.vue", 4)]);
    // A plain block's import is not the template's to use, though the template holds the word.
    assert_eq!(uses(&document, "src/plain.ts"), []);
    // An export of a block is used when the template names it, and the component's default export is
    // the component, never among its exports.
    assert_eq!(exported(&document, "src/App.vue"), ["label", "quiet"]);
    assert_eq!(
        uses(&document, "src/App.vue"),
        [("label", "src/App.vue", 4)]
    );
}

#[test]
fn a_vue_template_that_names_a_binding_of_an_import_call_uses_every_export() {
    let project = Project::new("pages-vue-dynamic");

    project
        .write(
            "src/App.vue",
            "<template>\n\
             \x20 <p>{{ texts.title }}</p>\n\
             </template>\n\
             <script setup lang=\"ts\">\n\
             const texts = await import('./texts');\n\
             const quiet = await import('./quiet');\n\
             console.log(quiet.used);\n\
             </script>\n",
        )
        .write(
            "src/texts.ts",
            "export const title = 't';\nexport const spare = 1;\n",
        )
        .write(
            "src/quiet.ts",
            "export const used = 1;\nexport const spare = 2;\n",
        )
        .commit();

    let document = project.graph();

    // A binding that holds what an `import()` loads passes the module on where the template names it,
    // so every export is used, from the template's line; one the template doesn't name uses only what
    // its references read (comparison spec DYN-1, dead-code spec USE-4).
    assert_eq!(uses(&document, "src/texts.ts"), [("*", "src/App.vue", 2)]);
    assert_eq!(
        uses(&document, "src/quiet.ts"),
        [("used", "src/App.vue", 7)]
    );
}

#[test]
fn an_mdx_document_uses_the_bindings_its_text_names() {
    let project = Project::new("pages-mdx-uses");

    project
        .write(
            "docs/guide.mdx",
            "import { Chart, Table, Hidden } from './widgets'\n\
             export { extra } from './extra'\n\
             \n\
             # Guide\n\
             \n\
             <Chart />\n\
             \n\
             ```jsx\n\
             <Table />\n\
             ```\n",
        )
        .write(
            "docs/widgets.ts",
            "export const Chart = 1;\nexport const Table = 2;\nexport const Hidden = 3;\n",
        )
        .write("docs/extra.ts", "export const extra = 1;\n")
        .write(
            "src/main.ts",
            "import { extra } from '../docs/guide.mdx';\nconsole.log(extra);\n",
        )
        .commit();

    let document = project.graph();

    // A binding the document's text names is used, from the line that names it; one named only in its
    // statements or in a fenced code block is not (spec USE-5, FIX-1 pattern 16).
    assert_eq!(
        uses(&document, "docs/widgets.ts"),
        [("Chart", "docs/guide.mdx", 6)]
    );
    // Its `export … from` passes a use on as a script's does, and it has no export of its own.
    assert_eq!(
        uses(&document, "docs/extra.ts"),
        [("extra", "src/main.ts", 2)]
    );
    assert_eq!(
        uses(&document, "docs/guide.mdx"),
        [("extra", "src/main.ts", 2)]
    );
    assert_eq!(exported(&document, "docs/guide.mdx"), Vec::<&str>::new());
}
