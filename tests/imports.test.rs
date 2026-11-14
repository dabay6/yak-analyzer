//! Runs the built binary on made-up projects and checks the imports it finds in each script, the names
//! each brings in, what a script exports and what it uses of the files it imports, and how it treats a
//! script it cannot parse (spec PARSE-1, PARSE-2, PARSE-6, IMP-1 to IMP-4, RUN-4; dead-code spec USE-1,
//! USE-2, USE-2a, FIX-1, FIX-2).

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

/// The exports of a file as `graph` prints them: each with its name, its kind, the line and the column
/// of its name, and whether its own file refers to it.
fn exports<'a>(document: &'a Value, path: &str) -> Vec<(&'a str, &'a str, u64, u64, bool)> {
    file(document, path)["exports"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} was read"))
        .iter()
        .map(|export| {
            (
                export["name"].as_str().expect("a name"),
                export["kind"].as_str().expect("a kind"),
                export["line"].as_u64().expect("a line"),
                export["column"].as_u64().expect("a column"),
                export["usedInFile"].as_bool().expect("a mark"),
            )
        })
        .collect()
}

/// One name an import brings in, as `graph` prints it: the target's name for it, its local binding,
/// and the names the importing file passes it on under.
fn name(name: &str, local: Option<&str>, exported: &[&str]) -> Value {
    json!({ "exported": exported, "local": local, "name": name })
}

/// The uses of a file's exports that count from a reference, in the document's order: each with the
/// export's name, `*` for every export, and the file and line it counts from.
fn uses<'a>(document: &'a Value, path: &str) -> Vec<(&'a str, &'a str, u64)> {
    document["uses"]
        .as_array()
        .expect("a list of uses")
        .iter()
        .filter(|used| used["file"] == json!(path) && used["entry"].is_null())
        .map(|used| {
            (
                used["name"].as_str().expect("a name"),
                used["by"].as_str().expect("a file"),
                used["line"].as_u64().expect("a line"),
            )
        })
        .collect()
}

#[test]
fn reads_what_a_script_exports_each_once_as_a_value_or_a_type() {
    let project = Project::new("imports-exports");

    project
        .write(
            "src/names.ts",
            "function helper() {\n\
             \x20 return 1;\n\
             }\n\
             export function used() {\n\
             \x20 return helper();\n\
             }\n\
             export const alone = used();\n\
             export interface Shape {}\n\
             export type Alias = Shape;\n\
             export enum Color {\n\
             \x20 Red,\n\
             }\n\
             export namespace OnlyTypes {\n\
             \x20 export type Inner = string;\n\
             }\n\
             export namespace WithValues {\n\
             \x20 export const inner = 1;\n\
             }\n\
             export class Widget {}\n\
             export const { first, rest: [second] } = { first: 1, rest: [2] };\n\
             const hidden = 1;\n\
             export { hidden as shown };\n\
             export function over(value: string): void;\n\
             export function over(value: number): void;\n\
             export function over(value: unknown): void {\n\
             \x20 void value;\n\
             }\n\
             export interface Merged {}\n\
             export const Merged = 1;\n\
             /* \u{1F600} */ export const [\u{3C0}, after] = [1, 2];\n\
             export default class {}\n",
        )
        .write(
            "src/common.cjs",
            "module.exports = { a: 1 };\nexports.b = 2;\n",
        )
        .commit();

    let document = project.graph();

    // A type is an interface, a type alias or a namespace of types alone; anything else is a value. A
    // name offered more than once, as overloads and merged declarations are, is one export, and a value
    // when any of them is. Each is where its name is written, the column in UTF-16 code units (spec
    // USE-1).
    assert_eq!(
        exports(&document, "src/names.ts"),
        [
            ("used", "value", 4, 17, true),
            ("alone", "value", 7, 14, false),
            ("Shape", "type", 8, 18, true),
            ("Alias", "type", 9, 13, false),
            ("Color", "value", 10, 13, false),
            ("OnlyTypes", "type", 13, 18, false),
            ("WithValues", "value", 16, 18, false),
            ("Widget", "value", 19, 14, false),
            ("first", "value", 20, 16, false),
            ("second", "value", 20, 30, false),
            ("shown", "value", 22, 20, false),
            ("over", "value", 23, 17, false),
            ("Merged", "value", 28, 18, false),
            ("\u{3C0}", "value", 30, 24, false),
            ("after", "value", 30, 27, false),
            ("default", "value", 31, 16, false),
        ]
    );
    // An export is a name an `export` statement offers: CommonJS offers none.
    assert_eq!(exports(&document, "src/common.cjs"), []);
}

#[test]
fn reads_the_names_each_import_brings_in_and_the_names_it_passes_on() {
    let project = Project::new("imports-names");

    project
        .write(
            "src/main.ts",
            "import fallback, { a, b as bee, type T } from './lib';\n\
             import * as all from './lib';\n\
             import './lib';\n\
             export { c, d as dee } from './lib';\n\
             export * from './lib';\n\
             export * as grouped from './lib';\n\
             import old = require('./lib');\n\
             type Later = import('./lib').Deep.Deeper;\n\
             const lazy = import('./lib');\n\
             const now = require('./lib');\n\
             import { again, twice } from './lib';\n\
             export { again as onceMore, twice, twice as thrice };\n\
             export default fallback;\n\
             export { all };\n",
        )
        .write("src/lib.ts", "")
        .commit();

    let document = project.graph();
    let names: Vec<Value> = file(&document, "src/main.ts")["imports"]
        .as_array()
        .expect("a list of imports")
        .iter()
        .map(|import| import["names"].clone())
        .collect();

    // A default, a named and a namespace import each make a binding; `export … from` makes none and
    // offers the name again; an imported binding exported again is passed on under each name it is
    // exported by (spec USE-1, USE-2a).
    assert_eq!(
        names,
        [
            json!([
                name("default", Some("fallback"), &["default"]),
                name("a", Some("a"), &[]),
                name("b", Some("bee"), &[]),
                name("T", Some("T"), &[]),
            ]),
            json!([name("*", Some("all"), &["all"])]),
            json!([]),
            json!([name("c", None, &["c"]), name("d", None, &["dee"])]),
            json!([name("*", None, &[])]),
            json!([name("*", None, &["grouped"])]),
            json!([name("*", Some("old"), &[])]),
            json!([name("Deep", None, &[])]),
            json!([]),
            json!([]),
            json!([
                name("again", Some("again"), &["onceMore"]),
                name("twice", Some("twice"), &["twice", "thrice"]),
            ]),
        ]
    );
    // A name a file only passes on is not its own export (spec USE-2a).
    assert_eq!(exports(&document, "src/main.ts"), []);
}

#[test]
fn counts_a_use_where_a_binding_is_referred_to_and_not_where_it_is_only_imported() {
    let project = Project::new("imports-uses");

    project
        .write(
            "src/lib.ts",
            "export const used = 1;\n\
             export const unreferenced = 2;\n\
             export const inFileOnly = 3;\n\
             export const nowhere = inFileOnly;\n\
             export const shadowed = 5;\n\
             export interface Shape {}\n\
             export const one = 1;\n\
             export const two = 2;\n\
             export const three = 3;\n\
             export default function main() {}\n",
        )
        .write(
            "src/main.ts",
            "import main, { used, unreferenced, shadowed, type Shape } from './lib';\n\
             import * as lib from './lib';\n\
             \n\
             function inner(shadowed: number): Shape | number {\n\
             \x20 return shadowed + used;\n\
             }\n\
             \n\
             main();\n\
             inner(lib.one + lib['two']);\n",
        )
        // The same names as locals of another file are no use of the exports.
        .write(
            "src/other.ts",
            "const nowhere = 1;\nconst lib = { three: 3 };\nconsole.log(nowhere, lib.three);\n",
        )
        .write("src/bag.ts", "export const x = 1;\nexport const y = 2;\n")
        .write(
            "src/whole.ts",
            "import * as bag from './bag';\nconsole.log(bag);\n",
        )
        .write("src/thing.js", "export const twin = 1;\n")
        .write("src/thing.d.ts", "export declare const twin: number;\n")
        .write(
            "src/twins.ts",
            "import { twin } from './thing';\nconsole.log(twin);\n",
        )
        .commit();

    let document = project.graph();

    // A use is a reference, in code or in a type, from the line of the first one: an import whose
    // binding nothing refers to, or that a parameter of the same name hides, is none. A namespace's
    // member read by a name the script writes uses that export (spec USE-2, FIX-1 pattern 17).
    assert_eq!(
        uses(&document, "src/lib.ts"),
        [
            ("Shape", "src/main.ts", 4),
            ("default", "src/main.ts", 8),
            ("one", "src/main.ts", 9),
            ("two", "src/main.ts", 9),
            ("used", "src/main.ts", 5),
        ]
    );
    // An export its own file refers to is marked, and is not used by another file.
    assert_eq!(
        exports(&document, "src/lib.ts")[2..4],
        [
            ("inFileOnly", "value", 3, 14, true),
            ("nowhere", "value", 4, 14, false),
        ]
    );
    // A namespace referred to in any other way uses every export.
    assert_eq!(uses(&document, "src/bag.ts"), [("*", "src/whole.ts", 2)]);
    // An import that leads to several files uses the export in each.
    assert_eq!(
        uses(&document, "src/thing.d.ts"),
        [("twin", "src/twins.ts", 2)]
    );
    assert_eq!(
        uses(&document, "src/thing.js"),
        [("twin", "src/twins.ts", 2)]
    );
}

#[test]
fn passes_a_use_on_through_re_exports_to_the_file_that_declares_the_name() {
    let project = Project::new("imports-chains");

    project
        .write(
            "src/index.ts",
            "export { leaf as renamed } from './middle';\n\
             export * as group from './group';\n\
             import { kept } from './middle';\n\
             export { kept };\n",
        )
        .write(
            "src/middle.ts",
            "export * from './leaf';\nexport * from './loop';\n",
        )
        // Two files that pass each other's names on end the walk all the same.
        .write(
            "src/loop.ts",
            "export * from './middle';\nexport const looped = 1;\n",
        )
        .write(
            "src/leaf.ts",
            "export const leaf = 1;\nexport const kept = 2;\nexport const spare = 3;\nexport default 4;\n",
        )
        .write(
            "src/group.ts",
            "export const g1 = 1;\nexport const g2 = 2;\n",
        )
        .write(
            "src/main.ts",
            "import { renamed, group, kept, missing } from './index';\n\
             import fallback from './middle';\n\
             console.log(renamed, group, kept, missing, fallback);\n",
        )
        .write(
            "src/all.ts",
            "import * as middle from './middle';\nconsole.log(middle);\n",
        )
        .commit();

    let document = project.graph();

    // A use of a name a file passes on is a use of the name it comes from, through `export … from`, an
    // imported binding exported again, and `export *`, to the file that declares it. `export * as`
    // passes a use on to every export, and `export *` never passes `default` (spec USE-2, FIX-2).
    assert_eq!(
        uses(&document, "src/index.ts"),
        [
            ("group", "src/main.ts", 3),
            ("kept", "src/main.ts", 3),
            ("renamed", "src/main.ts", 3),
        ]
    );
    assert_eq!(uses(&document, "src/middle.ts"), [("*", "src/all.ts", 2)]);
    assert_eq!(
        uses(&document, "src/leaf.ts"),
        [
            ("kept", "src/all.ts", 2),
            ("kept", "src/main.ts", 3),
            ("leaf", "src/all.ts", 2),
            ("leaf", "src/main.ts", 3),
            ("spare", "src/all.ts", 2),
        ]
    );
    assert_eq!(
        uses(&document, "src/loop.ts"),
        [("looped", "src/all.ts", 2)]
    );
    assert_eq!(uses(&document, "src/group.ts"), [("*", "src/main.ts", 3)]);
    // Neither file that only passes names on has an export of its own (spec USE-2a).
    assert_eq!(exports(&document, "src/index.ts"), []);
    assert_eq!(exports(&document, "src/middle.ts"), []);
}

#[test]
fn uses_every_export_of_a_file_a_call_loads_and_the_name_a_type_import_reads() {
    let project = Project::new("imports-loads");

    project
        .write(
            "src/main.ts",
            "import old = require('./old');\n\
             const now = require('./now');\n\
             const later = import('./later');\n\
             type Picked = import('./types').Picked;\n\
             type Whole = typeof import('./shapes');\n\
             console.log(old, now, later);\n",
        )
        .write("src/old.ts", "export const a = 1;\n")
        .write("src/now.ts", "export const b = 1;\n")
        .write("src/later.ts", "export const c = 1;\n")
        .write(
            "src/types.ts",
            "export type Picked = string;\nexport type Left = number;\n",
        )
        .write("src/shapes.ts", "export const d = 1;\n")
        .commit();

    let document = project.graph();

    // What is read off the result of `require`, `import()` and `import … = require` is not followed, so
    // each uses every export; a type import uses the name it reads (spec USE-2).
    assert_eq!(uses(&document, "src/old.ts"), [("*", "src/main.ts", 1)]);
    assert_eq!(uses(&document, "src/now.ts"), [("*", "src/main.ts", 2)]);
    assert_eq!(uses(&document, "src/later.ts"), [("*", "src/main.ts", 3)]);
    assert_eq!(
        uses(&document, "src/types.ts"),
        [("Picked", "src/main.ts", 4)]
    );
    assert_eq!(uses(&document, "src/shapes.ts"), [("*", "src/main.ts", 5)]);
}
