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
