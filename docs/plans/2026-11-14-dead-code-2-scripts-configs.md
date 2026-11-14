# Dead-code findings, part 2: entry points from a package's scripts and a tool's configuration file: implementation plan

**Goal:** `graph` adds the entry points a `package.json`'s scripts start and those a tool's
configuration file names through a `configEntries` declaration, and the analyzer refuses a
`configEntries` or `scriptCommands` declaration of the wrong shape.

**Architecture:** A new `src/tools.rs` reads what a tool starts without running anything: it splits a
script into commands and words, and reads a literal value out of a configuration file's default export
with `oxc_parser`. `src/entries.rs`, from plan 1, gains two more sources that use it, `scripts` and
`configEntries`. `src/declarations.rs` gives the two keys that were opaque their shapes, and
`src/packages.rs` reads `scripts`.

**Spec:** docs/specs/2026-11-13-dead-code-findings-design.md

**Implements:** ENTRY-6, ENTRY-6a, ENTRY-7, ENTRY-1, FIX-1, FIX-2

The second of the item's six plans, listed in plan 1. Plan 1 has merged, so this plan's branch,
`feat/dead-code-2-scripts-configs`, is made from `main`, and its pull request goes against `main`.

## Global constraints

- The analyzer never runs a project's code, its configuration files included: a value is read only as
  a literal in the file's syntax tree (design AN-6, ENT-4).
- A tool's convention is a declaration: only `node` is read without one, as the design spec's ENT-2
  names it; every other program comes from `scriptCommands` (design ENT-5).
- `configEntries` and `scriptCommands` are a pack's alone; a project that states either is refused, as
  before (reading KEY-2).
- No crate is added: `oxc_parser` and `oxc_ast` are pinned already.

## The decisions this plan makes

1. **Each source's name in `graph`.** A script's entry point has the source `{ from: "scripts", file:
   <its package.json> }`. A configuration file a `configEntries` declaration matches has
   `{ from: "configEntries", stack, by: "pack" }` with no file, as a declaration's own entry does; a
   file its value names has the same with `file` set to the configuration file.
2. **Where a declared program counts.** In every package's scripts, not only the detection folders':
   more entry points is the safe direction, and a package outside them has none of the three kinds
   reported anyway.
3. **Which words are tried.** Every word after the program, but one that begins with `-` and holds no
   `=`, since an option's own name is never a file; `--option=value` is tried by its value, and an
   empty word or value is passed over. Each is looked for as `Resolver::package_target`, from plan 1,
   looks for a target: from the folder of the `package.json`, by RES-3 and MAP-1.
4. **Splitting a script.** Quotes are read first, from the left, so a quoted `&&` or space splits
   nothing; a pair of quotes alone makes an empty word. A backslash is an ordinary character. `&&`
   and `||` are matched before `&` and `|`.
5. **What a configuration file's default export is.** `export default …`, or `module.exports = …`, the
   last such statement in the file. Parentheses, `as` and `satisfies` around a value are passed through.
   A key is a plain property, not computed, written as a name or a string; the last property of a name
   wins, as it does in the object. A value is one string, one template with no substitution, or a list
   of nothing else. The file is parsed as TypeScript for a TypeScript ending and as JavaScript with JSX
   otherwise; one that does not parse gives its `default`, and the reading of scripts reports it as
   PARSE-2 says.
6. **The shapes' lines.** A problem names the entry by its place in the list, counted from one, and the
   field: `its entry 2's field `pattern` is missing, and a `folder` value needs it`. A program name may
   not be empty or hold white space.
7. **The new notice kind:** `config-glob-not-read`, about the configuration file.
8. **The fixtures' shapes are made valid.** The shared hand-off document in `tests/support/mod.rs` and
   the declarations test that uses every key held `configEntries` and `scriptCommands` values of no
   particular shape, which ENTRY-6a now refuses.

## Review focus

- **A configuration file whose default export is a variable** (`const config = {…}; export default
  config`) gives its `default`, as ENT-4 says. Real configuration files are often written so; the check
  on the named repositories (plan 6) shows how often the defaults then stand. No fixture holds that form
  but the variable `include` of the third case.
- **A script that starts a file through a program the pack does not declare** makes no entry point,
  and that file may be reported unused in plan 3. The fixture holds `eslint` for it.
- **A word that names a folder** is looked for as RES-3 looks for one, through its `index` file, so
  `node .` makes the package's `index` an entry point. That is the safe direction; no fixture holds it.
- **A shell variable, a substitution or a glob in a word** is read as written and names no file.

### Task 1: entry points from scripts and configuration files, and the shapes of their declarations

**Implements:** ENTRY-6, ENTRY-6a, ENTRY-7, ENTRY-1, FIX-1, FIX-2

**Test:** `cargo test --release --locked --test entries --test declarations --test hand-off --test hand-off-schema`

**Files:**
- Create: `src/tools.rs`
- Modify: `src/declarations.rs`, `src/packages.rs`, `src/entries.rs`, `src/graph.rs`, `src/main.rs`
- Test: `tests/entries.test.rs`, `tests/declarations.test.rs`, `tests/support/mod.rs`

**Interfaces:**
- Consumes: plan 1's `entries::find`, `entries::Project`, `Source` and `Resolver::package_target`;
  `hand_off::leaves_its_folder`; `glob::Glob`; `oxc_parser` and `oxc_ast`.
- Produces: `tools::commands`, `tools::is_assignment` and `tools::configured`; `Package::scripts`;
  `entries::Project::root`; the shape checks of `configEntries` and `scriptCommands`. Plan 3 reads the
  entry points as a whole and needs nothing more from this plan.

- [ ] **Step 1: Write the failing tests**

Three tests join `tests/entries.test.rs`, one joins `tests/declarations.test.rs`, and the two
declarations of the old free shape take the new one:

```rust edit path=tests/entries.test.rs
<<<<<<< find
//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! and the packages whose entry points are likely incomplete (spec ENTRY-1 to ENTRY-5, ENTRY-8, CMD-4,
//! FIX-1).

#[path = "support/project.rs"]
mod project;
=======
//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! and the packages whose entry points are likely incomplete (spec ENTRY-1 to ENTRY-8, CMD-4, FIX-1,
//! FIX-2).

#[path = "support/project.rs"]
mod project;
>>>>>>> replace
<<<<<<< find
        [("cli.js", vec![named("bin", "package.json")])]
    );
}
=======
        [("cli.js", vec![named("bin", "package.json")])]
    );
}

/// A source that a stack's `configEntries` declaration names: the configuration file itself, with no
/// file, or a file the configuration file's value names.
fn configured(file: Option<&str>) -> Value {
    json!({ "by": "pack", "file": file, "from": "configEntries", "stack": "typescript" })
}

#[test]
fn a_scripts_words_after_node_or_a_declared_program_are_entry_points() {
    let project = Project::new("entries-scripts");

    project
        .write(
            "package.json",
            r#"{
                "name": "app",
                "scripts": {
                    "test": "node scripts/run-tests.mjs",
                    "dev": "NODE_ENV=dev node --watch './src/server.ts' && echo done",
                    "bundle": "cross-env MODE=prod esbuild src/cli.ts --outfile=dist/cli.js",
                    "lint": "eslint src/other.ts",
                    "spaced": "node \"scripts/a b.mjs\"; node scripts/missing.mjs"
                }
            }"#,
        )
        .write("scripts/run-tests.mjs", "")
        .write("scripts/a b.mjs", "")
        .write("src/server.ts", "")
        .write("src/cli.ts", "")
        .write("src/other.ts", "")
        .commit();

    let scripts = || vec![named("scripts", "package.json")];

    // Without a hand-off file only `node` runs files: `cross-env` is no program the analyzer knows.
    assert_eq!(
        entries(&project.graph()),
        [
            ("scripts/a b.mjs", scripts()),
            ("scripts/run-tests.mjs", scripts()),
            ("src/server.ts", scripts()),
        ]
    );

    // Declared, every later word of `cross-env`'s command is tried, the option's value mapped back from
    // build output (spec ENTRY-7).
    let file = project.hand_off(
        &project.root,
        &["."],
        stack(json!({ "scriptCommands": ["cross-env"] }), json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(
        entries(&document),
        [
            ("scripts/a b.mjs", scripts()),
            ("scripts/run-tests.mjs", scripts()),
            ("src/cli.ts", scripts()),
            ("src/server.ts", scripts()),
        ]
    );
}

#[test]
fn scripts_of_a_form_npm_does_not_give_them_stop_the_run() {
    let project = Project::new("entries-scripts-form");

    project
        .write(
            "package.json",
            r#"{ "name": "app", "scripts": ["node x.js"] }"#,
        )
        .commit();

    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `scripts` in package.json holds a value of a form it does not have; it must \
         be an object that maps each script's name to its command; fix it"
    );
}

#[test]
fn a_configuration_file_names_entry_points_by_a_literal_a_folder_or_its_default() {
    let project = Project::new("entries-configured");

    project
        .write(
            "packages/app/vitest.config.ts",
            "import { defineConfig } from 'vitest/config';\n\
             import './test/helper';\n\
             export default defineConfig({\n\
               test: { include: ['test/**/*.spec.ts', '!skip/**', 'odd/**/*.?(c|m)ts'] },\n\
             });\n",
        )
        .write("packages/app/test/a.spec.ts", "")
        .write("packages/app/test/helper.ts", "")
        .write(
            "packages/app/tools/jest.config.js",
            "module.exports = { roots: [`tools/tests`] };\n",
        )
        .write("packages/app/tools/tests/x.test.js", "")
        .write(
            "runner.config.js",
            "const folders = ['x'];\nexport default { include: folders };\n",
        )
        .write("checks/a.check.js", "")
        .commit();

    let declarations = json!({
        "configEntries": [
            {
                "file": "vitest.config.{ts,mts,js,mjs}",
                "property": ["test", "include"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["**/*.test.ts"]
            },
            {
                "file": "**/jest.config.js",
                "property": ["roots"],
                "value": "folder",
                "pattern": "**/*.test.js",
                "relativeTo": "detection",
                "default": ["**/*.test.js"]
            },
            {
                "file": "runner.config.js",
                "property": ["include"],
                "value": "globs",
                "relativeTo": "detection",
                "default": ["checks/*.check.js"]
            }
        ]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/app"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // The literal inside `defineConfig`, from the configuration file's folder; the folder `roots` names,
    // from the detection folder, joined to its pattern; and the default where the value is a variable
    // (spec ENTRY-6). Each configuration file is itself an entry point, from the deepest detection folder.
    assert_eq!(
        entries(&document),
        [
            (
                "checks/a.check.js",
                vec![configured(Some("runner.config.js"))]
            ),
            (
                "packages/app/test/a.spec.ts",
                vec![configured(Some("packages/app/vitest.config.ts"))]
            ),
            ("packages/app/tools/jest.config.js", vec![configured(None)]),
            (
                "packages/app/tools/tests/x.test.js",
                vec![configured(Some("packages/app/tools/jest.config.js"))]
            ),
            ("packages/app/vitest.config.ts", vec![configured(None)]),
            ("runner.config.js", vec![configured(None)]),
        ]
    );
    assert_eq!(
        notices(&document),
        ["config-glob-not-read", "config-glob-not-read"]
    );
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The glob `!skip/**` in packages/app/vitest.config.ts is not one the analyzer reads, so it adds no \
             entry point."
        )
    );
}
>>>>>>> replace
```

```rust edit path=tests/declarations.test.rs
<<<<<<< find
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4).

mod support;

=======
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4, ENTRY-6a).

mod support;

>>>>>>> replace
<<<<<<< find
        "cyclomaticLimit": 20,
        "cognitiveLimit": 15,
        "duplicationMinimum": 50,
        "configEntries": [{ "file": "runner.config.*", "property": ["test", "include"] }],
        "scriptCommands": { "any": "JSON", "is": ["accepted", 1, null] }
    });
    handed["stacks"][0]["project"] = json!({
        "entries": ["tools/run.ts"],
=======
        "cyclomaticLimit": 20,
        "cognitiveLimit": 15,
        "duplicationMinimum": 50,
        "configEntries": [
            {
                "file": "runner.config.*",
                "property": ["test", "include"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["**/*.test.ts"]
            },
            {
                "file": "**/runner.config.{js,ts}",
                "property": ["roots"],
                "value": "folder",
                "pattern": "**/*.spec.{js,ts}",
                "relativeTo": "detection",
                "default": []
            }
        ],
        "scriptCommands": ["cross-env", "tsx"]
    });
    handed["stacks"][0]["project"] = json!({
        "entries": ["tools/run.ts"],
>>>>>>> replace
<<<<<<< find
    );
}

#[test]
fn refuses_a_glob_or_a_path_it_cannot_read_or_that_leaves_its_folder() {
    let cases: [(Value, &str); 10] = [
=======
    );
}

#[test]
fn refuses_a_configuration_or_script_declaration_of_another_shape() {
    let usable = || {
        json!({
            "file": "runner.config.js",
            "property": ["include"],
            "value": "globs",
            "relativeTo": "config",
            "default": ["**/*.test.js"]
        })
    };
    let changed = |change: &dyn Fn(&mut Value)| {
        let mut entry = usable();

        change(&mut entry);

        json!({ "configEntries": [entry] })
    };
    let cases: [(Value, &str); 14] = [
        (
            json!({ "configEntries": {} }),
            ": its value must be a list of objects; ",
        ),
        (
            json!({ "configEntries": [3] }),
            ": its entry 1 must be an object; ",
        ),
        (
            changed(&|entry| entry["extra"] = json!(1)),
            ": its entry 1's field `extra` is not one it may hold; ",
        ),
        (
            changed(&|entry| {
                entry.as_object_mut().expect("an object").remove("default");
            }),
            ": its entry 1's field `default` is missing; ",
        ),
        (
            changed(&|entry| entry["value"] = json!("folder")),
            ": its entry 1's field `pattern` is missing, and a `folder` value needs it; ",
        ),
        (
            changed(&|entry| entry["pattern"] = json!("**/*.js")),
            ": its entry 1's field `pattern` is only for a `folder` value; ",
        ),
        (
            changed(&|entry| entry["property"] = json!([])),
            ": its entry 1's field `property` must be a list of keys, one or more; ",
        ),
        (
            changed(&|entry| entry["value"] = json!("list")),
            ": its entry 1's field `value` must be `globs` or `folder`; ",
        ),
        (
            changed(&|entry| entry["relativeTo"] = json!("package")),
            ": its entry 1's field `relativeTo` must be `config` or `detection`; ",
        ),
        (
            changed(&|entry| entry["file"] = json!("src/[a")),
            ": its entry 1's field `file` cannot be used: the glob `src/[a` has a bracket left open; ",
        ),
        (
            changed(&|entry| entry["file"] = json!(4)),
            ": its entry 1's field `file` must be a glob; ",
        ),
        (
            changed(&|entry| entry["default"] = json!(["../x/*.js"])),
            ": its entry 1's field `default` cannot be used: the glob `../x/*.js` is absolute or steps out",
        ),
        (
            json!({ "scriptCommands": "node" }),
            ": its value must be a list of program names; ",
        ),
        (
            json!({ "scriptCommands": ["cross env"] }),
            ": the program name `cross env` is empty or holds white space; ",
        ),
    ];

    for (index, (pack, expected)) in cases.into_iter().enumerate() {
        let line = refused(&format!("declarations-shape-{index}"), pack, json!({}));

        assert!(line.contains(expected), "{line}");
    }
}

#[test]
fn refuses_a_glob_or_a_path_it_cannot_read_or_that_leaves_its_folder() {
    let cases: [(Value, &str); 10] = [
>>>>>>> replace
```

```rust edit path=tests/support/mod.rs
<<<<<<< find
                "pack": {
                    "data": ["**/fixtures/**"],
                    "cyclomaticLimit": 20,
                    "configEntries": [{ "file": "runner.config.*", "property": "include" }]
                },
                "project": { "generated": ["src/generated/**"], "internal": ["packages/core"] }
            },
=======
                "pack": {
                    "data": ["**/fixtures/**"],
                    "cyclomaticLimit": 20,
                    "configEntries": [{
                        "file": "runner.config.*",
                        "property": ["include"],
                        "value": "globs",
                        "relativeTo": "config",
                        "default": []
                    }]
                },
                "project": { "generated": ["src/generated/**"], "internal": ["packages/core"] }
            },
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test entries --test declarations --test hand-off --test hand-off-schema
```

Expected: FAIL. In `entries`, the three new tests fail: no script and no configuration file makes an
entry point yet, and a `scripts` of the wrong form does not stop the run. In `declarations`,
`refuses_a_configuration_or_script_declaration_of_another_shape` fails, since both keys still accept
any value. The rest pass.

- [ ] **Step 3: Write the code**

The readers of a script and of a configuration file:

```rust path=src/tools.rs
//! What a tool starts, read without running anything: the commands of a `package.json`'s scripts, split
//! into words as far as ENTRY-7 reads a shell, and a literal value of a tool's configuration file
//! (spec ENTRY-6, ENTRY-7, design AN-6).

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrayExpressionElement, AssignmentTarget, Expression, ObjectExpression, ObjectPropertyKind,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// The text between unquoted separators that ends one command and begins the next (spec ENTRY-7).
const SEPARATORS: [&str; 5] = ["&&", "||", ";", "|", "&"];

/// The commands of a script, each as its words, quotes removed. Quotes are found first, from the left,
/// and nothing quoted splits; then the script is cut into commands at unquoted `&&`, `||`, `;`, `|` and
/// `&`, and each command into words at unquoted white space (spec ENTRY-7).
pub fn commands(script: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    let mut rest = script;

    while let Some(character) = rest.chars().next() {
        if let Some(open) = quote {
            if character == open {
                quote = None;
            } else {
                word.push(character);
            }

            rest = &rest[character.len_utf8()..];
            continue;
        }

        if let Some(separator) = SEPARATORS
            .iter()
            .find(|separator| rest.starts_with(**separator))
        {
            end_word(&mut words, &mut word, &mut started);
            commands.push(std::mem::take(&mut words));
            rest = &rest[separator.len()..];
            continue;
        }

        if character.is_whitespace() {
            end_word(&mut words, &mut word, &mut started);
        } else if character == '\'' || character == '"' {
            quote = Some(character);
            started = true;
        } else {
            word.push(character);
            started = true;
        }

        rest = &rest[character.len_utf8()..];
    }

    end_word(&mut words, &mut word, &mut started);
    commands.push(words);
    commands.retain(|command| !command.is_empty());
    commands
}

/// Ends the word being read, when one was begun: a pair of quotes alone makes an empty word.
fn end_word(words: &mut Vec<String>, word: &mut String, started: &mut bool) {
    if *started {
        words.push(std::mem::take(word));
        *started = false;
    }
}

/// Whether a word of a command is an assignment that comes before its program, `NAME=value`.
pub fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        name.starts_with(|first: char| first.is_ascii_alphabetic() || first == '_')
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
    })
}

/// The value a configuration file states at a property path, where it is written as a string or a list
/// of strings in the object that is the file's default export, or in the one object passed to the call
/// that is its default export, reached through the path by object literals alone. Nothing where the file
/// does not parse, or the value is written any other way: the file is never run (spec ENTRY-6, design
/// AN-6).
pub fn configured(path: &str, text: &str, property: &[String]) -> Option<Vec<String>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type(path)).parse();

    if parsed.fatal_error || !parsed.diagnostics.is_empty() {
        return None;
    }

    let exported = parsed.program.body.iter().rev().find_map(default_export)?;
    let mut object = object_of(exported)?;
    let (last, path) = property.split_last()?;

    for key in path {
        object = match unwrapped(value_of(object, key)?) {
            Expression::ObjectExpression(inner) => inner,
            _ => return None,
        };
    }

    strings(unwrapped(value_of(object, last)?))
}

/// The language a configuration file is parsed in, from its name: TypeScript for a TypeScript ending,
/// and otherwise JavaScript, as a module.
fn source_type(path: &str) -> SourceType {
    let typescript = [".ts", ".mts", ".cts"]
        .iter()
        .any(|ending| path.ends_with(ending));

    if typescript {
        SourceType::ts()
    } else {
        SourceType::jsx()
    }
    .with_module(true)
}

/// The expression a statement exports as the default: `export default …`, or `module.exports = …`.
fn default_export<'s, 'a>(statement: &'s Statement<'a>) -> Option<&'s Expression<'a>> {
    match statement {
        Statement::ExportDefaultDeclaration(declaration) => declaration.declaration.as_expression(),
        Statement::ExpressionStatement(statement) => match &statement.expression {
            Expression::AssignmentExpression(assignment) => match &assignment.left {
                AssignmentTarget::StaticMemberExpression(member)
                    if member.property.name == "exports"
                        && matches!(&member.object, Expression::Identifier(module) if module.name == "module") =>
                {
                    Some(&assignment.right)
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// The object an exported expression is, or the one object passed to the call it is.
fn object_of<'s, 'a>(exported: &'s Expression<'a>) -> Option<&'s ObjectExpression<'a>> {
    match unwrapped(exported) {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) if call.arguments.len() == 1 => {
            match call.arguments[0].as_expression().map(unwrapped) {
                Some(Expression::ObjectExpression(object)) => Some(object),
                _ => None,
            }
        }
        _ => None,
    }
}

/// An expression without the parentheses, `as` and `satisfies` around it, which change no value.
fn unwrapped<'s, 'a>(expression: &'s Expression<'a>) -> &'s Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => unwrapped(&inner.expression),
        Expression::TSAsExpression(inner) => unwrapped(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrapped(&inner.expression),
        other => other,
    }
}

/// The value an object literal states for a key, written as a plain property; the last such one, as the
/// object's own value is the last one written.
fn value_of<'s, 'a>(object: &'s ObjectExpression<'a>, key: &str) -> Option<&'s Expression<'a>> {
    object
        .properties
        .iter()
        .rev()
        .find_map(|property| match property {
            ObjectPropertyKind::ObjectProperty(property)
                if !property.computed
                    && property
                        .key
                        .static_name()
                        .is_some_and(|name| name.as_str() == Some(key)) =>
            {
                Some(&property.value)
            }
            _ => None,
        })
}

/// The strings a value is written as: one string, or a list of strings and nothing else.
fn strings(value: &Expression<'_>) -> Option<Vec<String>> {
    match value {
        Expression::StringLiteral(literal) => Some(vec![literal.value.display().to_string()]),
        Expression::TemplateLiteral(literal) => {
            Some(vec![literal.single_quasi()?.display().to_string()])
        }
        Expression::ArrayExpression(list) => list
            .elements
            .iter()
            .map(|element| match element {
                ArrayExpressionElement::StringLiteral(literal) => {
                    Some(literal.value.display().to_string())
                }
                ArrayExpressionElement::TemplateLiteral(literal) => {
                    Some(literal.single_quasi()?.display().to_string())
                }
                _ => None,
            })
            .collect(),
        _ => None,
    }
}
```

The shapes of the two keys:

```rust edit path=src/declarations.rs
<<<<<<< find
//! The declaration keys the analyzer knows, and the check that it can use every declaration a hand-off
//! file holds (spec KEY-1, KEY-2, KEY-5).
//!
//! The keys are drafts until the design spec's stage 2 ends. This part checks them; the parts that read a
//! project use them.

use serde_json::Value;

=======
//! The declaration keys the analyzer knows, and the check that it can use every declaration a hand-off
//! file holds (spec KEY-1, KEY-2, KEY-5).
//!
//! The keys are drafts until the design spec's stage 2 ends. This part checks them, the shapes of the two
//! that describe what a tool does included (spec ENTRY-6a); the parts that read a project use them.

use serde_json::Value;

>>>>>>> replace
<<<<<<< find
    Paths,
    /// A positive whole number.
    Number,
    /// Any JSON, which only a pack may state.
    Opaque,
}

/// Every key the analyzer knows, with the kind of its value (spec KEY-1).
const KEYS: [(&str, Kind); 9] = [
    ("entries", Kind::Globs),
=======
    Paths,
    /// A positive whole number.
    Number,
    /// What a tool reads from its configuration file, which only a pack may state (spec ENTRY-6).
    ConfigEntries,
    /// The programs whose file arguments in scripts are entry points, which only a pack may state (spec
    /// ENTRY-7).
    ScriptCommands,
}

/// The fields an object of `configEntries` may hold, each required but `pattern` (spec ENTRY-6).
const CONFIG_FIELDS: [&str; 6] = [
    "file",
    "property",
    "value",
    "pattern",
    "relativeTo",
    "default",
];

/// Every key the analyzer knows, with the kind of its value (spec KEY-1).
const KEYS: [(&str, Kind); 9] = [
    ("entries", Kind::Globs),
>>>>>>> replace
<<<<<<< find
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("configEntries", Kind::Opaque),
    ("scriptCommands", Kind::Opaque),
];

/// Who states a declaration: the hand-off file keeps the two apart.
=======
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("configEntries", Kind::ConfigEntries),
    ("scriptCommands", Kind::ScriptCommands),
];

/// Who states a declaration: the hand-off file keeps the two apart.
>>>>>>> replace
<<<<<<< find
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
=======
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
        Kind::ConfigEntries | Kind::ScriptCommands if matches!(source, Source::Project) => {
            Some("only a pack may state this key".to_owned())
        }
        Kind::ConfigEntries => config_entries_problem(value),
        Kind::ScriptCommands => match texts(value) {
            Some(names) => names
                .iter()
                .find(|name| name.is_empty() || name.contains(char::is_whitespace))
                .map(|name| format!("the program name `{name}` is empty or holds white space")),
            None => Some("its value must be a list of program names".to_owned()),
        },
    }
}

/// Why a `configEntries` value cannot be used, or nothing when it can: a list of objects, each with the
/// fields ENTRY-6 gives and no other (spec ENTRY-6, ENTRY-6a).
fn config_entries_problem(value: &Value) -> Option<String> {
    let Some(list) = value.as_array() else {
        return Some("its value must be a list of objects".to_owned());
    };

    list.iter().enumerate().find_map(|(index, entry)| {
        let number = index + 1;
        let Some(fields) = entry.as_object() else {
            return Some(format!("its entry {number} must be an object"));
        };
        let field =
            |name: &str, why: &str| Some(format!("its entry {number}'s field `{name}` {why}"));

        if let Some(unknown) = fields
            .keys()
            .find(|key| !CONFIG_FIELDS.contains(&key.as_str()))
        {
            return field(unknown, "is not one it may hold");
        }

        let folder = fields.get("value") == Some(&Value::from("folder"));

        match (fields.get("pattern"), folder) {
            (None, true) => return field("pattern", "is missing, and a `folder` value needs it"),
            (Some(_), false) => return field("pattern", "is only for a `folder` value"),
            _ => {}
        }

        if let Some(missing) = CONFIG_FIELDS
            .iter()
            .find(|name| **name != "pattern" && !fields.contains_key(**name))
        {
            return field(missing, "is missing");
        }

        let property = fields["property"]
            .as_array()
            .filter(|keys| !keys.is_empty() && keys.iter().all(Value::is_string));

        if property.is_none() {
            return field("property", "must be a list of keys, one or more");
        }

        if !matches!(fields["value"].as_str(), Some("globs" | "folder")) {
            return field("value", "must be `globs` or `folder`");
        }

        if !matches!(fields["relativeTo"].as_str(), Some("config" | "detection")) {
            return field("relativeTo", "must be `config` or `detection`");
        }

        for name in ["file", "pattern"] {
            match fields.get(name).map(Value::as_str) {
                None => {}
                Some(Some(glob)) => {
                    if let Some(why) = glob_problem(glob) {
                        return field(name, &format!("cannot be used: {why}"));
                    }
                }
                Some(None) => return field(name, "must be a glob"),
            }
        }

        match texts(&fields["default"]) {
            Some(globs) => globs
                .iter()
                .find_map(|glob| glob_problem(glob))
                .and_then(|why| field("default", &format!("cannot be used: {why}"))),
            None => field("default", "must be a list of globs"),
        }
    })
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
>>>>>>> replace
```

`src/packages.rs` reads `scripts`:

```rust edit path=src/packages.rs
<<<<<<< find
    /// The fields that make files entry points, each with the target it states: `types` (or
    /// `typings`), `module`, `main`, then each of `bin` (spec ENTRY-1).
    pub starts: Vec<(&'static str, String)>,
}

/// What a `package.json` states that a package keeps.
=======
    /// The fields that make files entry points, each with the target it states: `types` (or
    /// `typings`), `module`, `main`, then each of `bin` (spec ENTRY-1).
    pub starts: Vec<(&'static str, String)>,
    /// The commands of its `scripts`, by the scripts' names (spec ENTRY-7).
    pub scripts: Vec<String>,
}

/// What a `package.json` states that a package keeps.
>>>>>>> replace
<<<<<<< find
    imports: Option<Value>,
    entries: Vec<(&'static str, String)>,
    bin: Vec<String>,
}

/// The packages of a project, and which package each file belongs to.
=======
    imports: Option<Value>,
    entries: Vec<(&'static str, String)>,
    bin: Vec<String>,
    scripts: Vec<String>,
}

/// The packages of a project, and which package each file belongs to.
>>>>>>> replace
<<<<<<< find
                        .map(|(_, target)| target)
                        .collect(),
                    starts,
                }
            })
            .collect();
=======
                        .map(|(_, target)| target)
                        .collect(),
                    starts,
                    scripts: fields.scripts,
                }
            })
            .collect();
>>>>>>> replace
<<<<<<< find
        imports: imports(fields, file)?,
        entries: entries(fields, file)?,
        bin: bin(fields, file)?,
    })
}

=======
        imports: imports(fields, file)?,
        entries: entries(fields, file)?,
        bin: bin(fields, file)?,
        scripts: scripts(fields, file)?,
    })
}

>>>>>>> replace
<<<<<<< find
    }
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();
=======
    }
}

/// The commands of a `package.json`'s `scripts`, an object that maps each script's name to its command,
/// by the scripts' names (spec ENTRY-7, PACK-6).
fn scripts(fields: &Map<String, Value>, file: &str) -> Result<Vec<String>, Stop> {
    let form = "an object that maps each script's name to its command";

    match fields.get("scripts") {
        None => Ok(Vec::new()),
        Some(Value::Object(scripts)) => scripts
            .values()
            .map(|command| {
                command
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| misshapen("scripts", file, form))
            })
            .collect(),
        Some(_) => Err(misshapen("scripts", file, form)),
    }
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();
>>>>>>> replace
```

The two new sources of entry points:

```rust edit path=src/entries.rs
<<<<<<< find
//! Entry points: the files something outside the project's imports starts or loads, each with every
//! source that made it one (spec ENTRY-1 to ENTRY-5, ENTRY-8).
//!
//! This part reads the project's standard files and the `entries` and `internal` declarations. The
//! scripts of a `package.json` and a tool's configuration file are read by a later part.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::hand_off::HandOff;
use crate::notice::Notice;
use crate::packages::{Package, Packages};
use crate::resolve::{self, Outcome, Resolution, Resolver};
use crate::stop::Stop;
use crate::tsconfig::Configs;

/// The endings of build output an `entries` glob may end in, a type declaration's first, since `.d.ts`
=======
//! Entry points: the files something outside the project's imports starts or loads, each with every
//! source that made it one (spec ENTRY-1 to ENTRY-5, ENTRY-8).
//!
//! They come from the project's standard files, the scripts of each `package.json`, the `entries` and
//! `internal` declarations, and the configuration files `configEntries` names (spec ENTRY-6, ENTRY-7).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::hand_off::{HandOff, leaves_its_folder};
use crate::notice::Notice;
use crate::packages::{Package, Packages};
use crate::resolve::{self, Outcome, Resolution, Resolver};
use crate::stop::Stop;
use crate::tools;
use crate::tsconfig::Configs;

/// The endings of build output an `entries` glob may end in, a type declaration's first, since `.d.ts`
>>>>>>> replace
<<<<<<< find

/// What the entry points are read from.
pub struct Project<'p> {
    /// The file list.
    pub files: &'p [File],
    /// The packages.
=======

/// What the entry points are read from.
pub struct Project<'p> {
    /// The root, from which a configuration file's content is read.
    pub root: &'p Path,
    /// The file list.
    pub files: &'p [File],
    /// The packages.
>>>>>>> replace
<<<<<<< find
        }
    }

    declared(project, &mut entries, notices)?;

    for package in &project.packages.list {
        let fault = if unmapped.contains(&package.folder) {
=======
        }
    }

    scripted(project, &mut entries);
    declared(project, &mut entries, notices)?;
    configured(project, &mut entries, notices);

    for package in &project.packages.list {
        let fault = if unmapped.contains(&package.folder) {
>>>>>>> replace
<<<<<<< find
    }
}

/// The package folders the `internal` declarations name: a pack's from each folder yak's detection
/// looked in, a project's from the root (spec ENTRY-5).
fn internal(handed: Option<&HandOff>) -> BTreeSet<String> {
=======
    }
}

/// The entry points the scripts of each `package.json` start: a file a word of a command names, from
/// the folder of that `package.json`, after `node` or a program a `scriptCommands` declaration names,
/// as RES-3 and MAP-1 find it. Words of the form `NAME=value` before the program are passed over, an
/// option's own name is never a file, and `--option=value` is tried by its value. A declared program
/// counts in every package's scripts (spec ENTRY-7).
fn scripted(project: &Project<'_>, entries: &mut Entries) {
    let mut programs: BTreeSet<&str> = BTreeSet::from(["node"]);

    if let Some(handed) = project.handed {
        for stack in &handed.stacks {
            programs.extend(texts(stack.pack.get("scriptCommands")));
        }
    }

    for package in &project.packages.list {
        let manifest = manifest_path(&package.folder);

        for script in &package.scripts {
            for command in tools::commands(script) {
                let mut words = command.iter().skip_while(|word| tools::is_assignment(word));

                if !words
                    .next()
                    .is_some_and(|program| programs.contains(program.as_str()))
                {
                    continue;
                }

                for word in words {
                    let named = match word.split_once('=') {
                        Some((option, value)) if option.starts_with("--") => value,
                        _ if word.starts_with('-') => continue,
                        _ => word.as_str(),
                    };

                    if named.is_empty() {
                        continue;
                    }

                    for path in project
                        .resolver
                        .package_target(package, named)
                        .into_iter()
                        .flatten()
                    {
                        entries.add(path, Source::named("scripts", &manifest));
                    }
                }
            }
        }
    }
}

/// The entry points the `configEntries` declarations of the packs name: each configuration file a
/// declaration's `file` matches, once, from the deepest folder yak's detection looked in that holds it,
/// and the files its value names, read as a literal where it is one and the declaration's `default`
/// where it is not. A glob of the file's that the analyzer does not read is passed over with a notice
/// (spec ENTRY-6).
fn configured(project: &Project<'_>, entries: &mut Entries, notices: &mut Vec<Notice>) {
    let Some(handed) = project.handed else {
        return;
    };

    for stack in &handed.stacks {
        let Some(declarations) = stack.pack.get("configEntries").and_then(Value::as_array) else {
            continue;
        };
        let source = Source {
            from: "configEntries",
            file: None,
            stack: Some(stack.name.clone()),
            by: Some("pack"),
        };

        for declaration in declarations.iter().filter_map(Declared::of) {
            for config in project.files {
                let Some(base) = deepest(&handed.folders, &config.path).filter(|base| {
                    inside(base, &config.path).is_some_and(|rest| declaration.file.matches(rest))
                }) else {
                    continue;
                };

                entries.add(config.path.clone(), source.clone());

                let folder = if declaration.relative_to == "config" {
                    config
                        .path
                        .rsplit_once('/')
                        .map_or("", |(folder, _)| folder)
                } else {
                    base
                };
                let named = Source {
                    file: Some(config.path.clone()),
                    ..source.clone()
                };

                for glob in declaration
                    .globs(project.root, &config.path, notices)
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
        }
    }
}

/// One `configEntries` declaration, read. Its shape was checked when the hand-off file was read (spec
/// ENTRY-6a), so a declaration of another shape is passed over.
struct Declared<'d> {
    /// The configuration file, from each folder yak's detection looked in.
    file: Glob,
    /// The path to the value in the object the file exports.
    property: Vec<String>,
    /// Whether the value is a folder that `pattern` is joined to, rather than globs.
    folder: bool,
    /// The glob joined to a folder value.
    pattern: Option<&'d str>,
    /// What the value is relative to: `config` or `detection`.
    relative_to: &'d str,
    /// The globs used where the file states no value the analyzer reads.
    default: Vec<&'d str>,
}

impl<'d> Declared<'d> {
    /// The declaration, where it has its shape.
    fn of(declaration: &'d Value) -> Option<Self> {
        Some(Self {
            file: Glob::new(declaration.get("file")?.as_str()?).ok()?,
            property: texts(declaration.get("property"))
                .into_iter()
                .map(str::to_owned)
                .collect(),
            folder: declaration.get("value")?.as_str()? == "folder",
            pattern: declaration.get("pattern").and_then(Value::as_str),
            relative_to: declaration.get("relativeTo")?.as_str()?,
            default: texts(declaration.get("default")),
        })
    }

    /// A value of the file's as a glob: a `folder` value joined to the declaration's `pattern`.
    fn joined(&self, value: &str) -> String {
        match (self.folder, self.pattern) {
            (true, Some(pattern)) => join(&[value, pattern]),
            _ => value.to_owned(),
        }
    }

    /// The globs a configuration file's value gives, from the folder the value is relative to: the
    /// value where it is a literal and leaves a glob the analyzer reads, and the declaration's
    /// `default` where it does not.
    fn globs(&self, root: &Path, path: &str, notices: &mut Vec<Notice>) -> Vec<String> {
        let stated = fs::read(root.join(path))
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|text| {
                let text = text.strip_prefix('\u{feff}').unwrap_or(&text);

                tools::configured(path, text, &self.property)
            })
            .unwrap_or_default();
        let mut read = Vec::new();

        for value in &stated {
            let glob = self.joined(value);

            if readable(&glob) {
                read.push(glob);
            } else {
                notices.push(Notice::new(
                    "config-glob-not-read",
                    Some(path.to_owned()),
                    format!(
                        "The glob `{glob}` in {path} is not one the analyzer reads, so it adds no entry \
                         point."
                    ),
                ));
            }
        }

        if read.is_empty() {
            self.default
                .iter()
                .map(|value| self.joined(value))
                .collect()
        } else {
            read
        }
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
fn deepest<'f>(folders: &'f [String], path: &str) -> Option<&'f str> {
    folders
        .iter()
        .map(|folder| folder_path(folder))
        .filter(|folder| inside(folder, path).is_some())
        .max_by_key(|folder| folder.len())
}

/// The package folders the `internal` declarations name: a pack's from each folder yak's detection
/// looked in, a project's from the root (spec ENTRY-5).
fn internal(handed: Option<&HandOff>) -> BTreeSet<String> {
>>>>>>> replace
```

```rust edit path=src/graph.rs
<<<<<<< find
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            files,
            packages: &packages,
            configs: &configs,
=======
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            root: &root,
            files,
            packages: &packages,
            configs: &configs,
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find
mod resolve;
mod scripts;
mod stop;
mod tsconfig;

use std::env;
=======
mod resolve;
mod scripts;
mod stop;
mod tools;
mod tsconfig;

use std::env;
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test entries --test declarations --test hand-off --test hand-off-schema
```

Expected: PASS: 9 tests in `entries`, 6 in `declarations`, 10 in `hand-off`, 2 in `hand-off-schema`.

- [ ] **Step 5: Run the gates**

```bash
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: no output from the first two; every test passes in the last two.

- [ ] **Step 6: Commit**

```bash
yak stack format
git add src/tools.rs src/declarations.rs src/packages.rs src/entries.rs src/graph.rs src/main.rs tests/entries.test.rs tests/declarations.test.rs tests/support/mod.rs
git commit -m "feat(entries): read entry points from scripts and tool configuration"
```

Expected: one commit on `feat/dead-code-2-scripts-configs`.
