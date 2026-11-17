# Stage 2, part 2: a configuration file a script names by an option: implementation plan

**Goal:** a `configEntries` declaration may name the options a program reads its configuration file
from, so a script such as `vitest --config vitest.e2e.config.ts` makes that file an entry point and
reads its value as a matched file's is, and the test files it includes are no longer reported unused
(CONF-1 to CONF-3).

**Architecture:** `src/declarations.rs` accepts one more optional field of a `configEntries` object,
`options`, and stops the run on any other shape of it (ENTRY-6a). `src/entries.rs` collects, for each
declaration, every configuration file it reads with each folder its value is relative to: the files
its `file` glob matches, as today, and the files the scripts of every `package.json` name after one
of the options. Each is an entry point, and its value is read from each of its folders. A word that
names no listed file gives one notice. `src/notice.rs` lets two notices be compared, so a glob read
from two folders is said once.

**Spec:** docs/specs/2026-11-15-dead-code-comparison-design.md

**Implements:** CONF-1, CONF-2, CONF-3

The second of the item's four plans, on its own branch, `feat/comparison-2-config-option`, made from
`main` after plan 1 merged as #28. Plan 3 is the comparison on the pinned commits; plan 4 the owner's
decision.

## Global constraints

- Which option a tool reads its configuration file from is a fact of the tool, so only a pack states
  `options`, as only a pack states `configEntries` (CONF-1, design ENT-5).
- A `configEntries` object of another shape, `options` included, stops the run, naming the stack,
  the key and the field (stage-1 ENTRY-6a).
- `options.relativeTo` is `"config"`, `"detection"` or `"script"`; absent, the declaration's own
  `relativeTo` holds. The declaration's own stays `"config"` or `"detection"` (CONF-1).
- The program is the first word of a command whose last path segment, with a `.cmd`, `.exe` or `.ps1`
  ending dropped, is `program`; a wrapper before it needs no declaration (CONF-2).
- Every declaration with `options` applies to the scripts of every `package.json` under the root, and
  a named file is found from the folder of the `package.json` whose script names it (CONF-2).
- A named file is an entry point whether or not its program is in a `scriptCommands` declaration;
  one both matched and named is one entry point, read from each folder its value is relative to
  (CONF-3).
- A named file that does not parse is an entry point, and the declaration's `default` stands (CONF-2,
  stage-1 ENTRY-6).
- Fixtures are made up and copy no code from the named repositories (CURE-3, CONTRIBUTING.md).

## The decisions this plan makes

1. **Where a named word leads.** The word is joined to the folder of its `package.json` as a glob of a
   configuration file's value is joined to its folder (`resolved`: `.` dropped, `..` climbing, nothing
   past the root), and must be a listed file. It is not mapped back from build output: a
   configuration file is run from source.
2. **The notice for a word that names no listed file.** Kind `script-config-not-listed`, about the
   `package.json`, with the sentence "The script `<command>` in `<package.json>` names `<word>` after
   `<option>` as a configuration file of `<program>`, and no listed file is there, so it adds no entry
   point." The spec leaves the kind and wording to the builder. The same word under two declarations
   of one program gives one notice.
3. **An option with no word after it** names nothing and says nothing; `--config=` names the empty
   word, which leads to no file and gives the notice.
4. **A glob passed over** from two folders of one file gives its `config-glob-not-read` notice once.
5. **No `audit` fixture.** The live check's shape is a configuration file a script names that includes
   test files; the `entries` fixture holds it, and an entry point's reach is `audit`'s, unchanged.
6. **No README or CONTRIBUTING change.** Neither describes the declaration keys' shapes today; the
   draft keys are written down when FREEZE-1 fixes them (plan 4).

## Review focus

- **A program named inside another word**, such as `vitest-runner --config x.ts`, is not `vitest`:
  the comparison is of the whole last segment. No fixture holds it.
- **A backslash path** before the program (`.\node_modules\.bin\vitest.cmd`) is split at `\` as well
  as `/`. The fixture holds the `/` form.
- **The same configuration file named by two packages' scripts** is read from each script's folder
  when `options.relativeTo` is `"script"`. The fixture holds a file both matched and named from
  another package.
- **A named file outside its package** (`vitest --config ../shared/vitest.config.ts`) is found by
  `..`. No fixture holds it.

### Task 1: read a configuration file a script names by an option

**Implements:** CONF-1, CONF-2, CONF-3

**Test:** `cargo test --release --locked --test entries --test declarations`

**Files:**
- Modify: `src/declarations.rs`, `src/entries.rs`, `src/notice.rs`
- Test: `tests/entries.test.rs`, `tests/declarations.test.rs`

**Interfaces:**
- Consumes: `tools::commands`, which cuts a script into commands and words (stage-1 ENTRY-7);
  `Declared::globs`, which reads a configuration file's value from a folder; the existing `resolved`,
  `deepest`, `inside`, `folder_path`, `manifest_path` and `mapped_globs` helpers of `src/entries.rs`;
  `Package::scripts`, the commands of a `package.json`'s scripts.
- Produces: `Entries::points` holding each named configuration file with the `configEntries` source
  and each file its value names with that file as `file`, as for a matched one; a
  `script-config-not-listed` notice for each word that names no listed file. `graph` and `audit`
  read them unchanged; plan 3's comparison runs `audit` with them.

- [ ] **Step 1: Write the failing tests**

The `entries` test: a script naming a second configuration file by a separate word and by `=`, behind
`npx` and `cross-env`, a file both matched and named from another package, and a word naming no file.

```rust edit path=tests/entries.test.rs
<<<<<<< find
//! exports, and the files the entry points reach (spec ENTRY-1 to ENTRY-8, USE-3, USE-6, CMD-4, FIX-1,
//! FIX-2).
=======
//! exports, and the files the entry points reach (spec ENTRY-1 to ENTRY-8, USE-3, USE-6, CMD-4, FIX-1,
//! FIX-2; comparison spec CONF-1 to CONF-3).
>>>>>>> replace
<<<<<<< find
#[test]
fn a_root_script_that_runs_another_packages_build_output_names_that_packages_source() {
=======
#[test]
fn a_configuration_file_a_script_names_by_an_option_is_read_as_a_matched_one_is() {
    let project = Project::new("entries-configured-option");

    project
        .write(
            "package.json",
            r#"{
                "name": "root",
                "private": true,
                "workspaces": ["packages/*"],
                "scripts": { "unit": "vitest --config packages/app/vitest.config.ts" }
            }"#,
        )
        .write(
            "packages/app/package.json",
            r#"{
                "name": "app",
                "scripts": {
                    "test": "vitest run",
                    "e2e": "npx vitest --config vitest.e2e.config.ts",
                    "int": "cross-env CI=1 ./node_modules/.bin/vitest.cmd -c=config/int.config.mjs",
                    "gone": "vitest --config missing.config.ts"
                }
            }"#,
        )
        .write(
            "packages/app/vitest.config.ts",
            "export default { test: { include: ['test/**/*.unit.ts'] } };\n",
        )
        .write(
            "packages/app/vitest.e2e.config.ts",
            "export default { test: { include: ['e2e/**/*.e2e.ts'] } };\n",
        )
        .write(
            "packages/app/config/int.config.mjs",
            "export default { test: { include: ['int/**/*.int.ts'] } };\n",
        )
        .write("packages/app/test/a.unit.ts", "")
        .write("packages/app/e2e/b.e2e.ts", "")
        .write("packages/app/int/c.int.ts", "")
        .write("packages/app/config/int/d.int.ts", "")
        .write("test/e.unit.ts", "")
        .commit();

    let declarations = json!({
        "configEntries": [{
            "file": "vitest.config.{ts,mts}",
            "property": ["test", "include"],
            "value": "globs",
            "relativeTo": "config",
            "default": ["**/*.test.ts"],
            "options": { "program": "vitest", "names": ["--config", "-c"], "relativeTo": "script" }
        }]
    });
    let file = project.hand_off(
        &project.root,
        &[".", "packages/app"],
        stack(declarations, json!({})),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // A file a script names after `--config` or `-c=`, behind `npx` or `cross-env`, is an entry point,
    // and its value is read from the folder of the script's `package.json`, not the file's own. A file
    // both matched and named is read from its own folder and from the naming script's (comparison spec
    // CONF-1 to CONF-3).
    assert_eq!(
        entries(&document),
        [
            ("packages/app/config/int.config.mjs", vec![configured(None)]),
            (
                "packages/app/e2e/b.e2e.ts",
                vec![configured(Some("packages/app/vitest.e2e.config.ts"))]
            ),
            (
                "packages/app/int/c.int.ts",
                vec![configured(Some("packages/app/config/int.config.mjs"))]
            ),
            (
                "packages/app/test/a.unit.ts",
                vec![configured(Some("packages/app/vitest.config.ts"))]
            ),
            ("packages/app/vitest.config.ts", vec![configured(None)]),
            ("packages/app/vitest.e2e.config.ts", vec![configured(None)]),
            (
                "test/e.unit.ts",
                vec![configured(Some("packages/app/vitest.config.ts"))]
            ),
        ]
    );

    // A word that names no listed file adds nothing, with a notice (comparison spec CONF-2).
    assert_eq!(notices(&document), ["script-config-not-listed"]);
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The script `vitest --config missing.config.ts` in packages/app/package.json names \
             `missing.config.ts` after `--config` as a configuration file of `vitest`, and no listed file is \
             there, so it adds no entry point."
        )
    );
}

#[test]
fn a_root_script_that_runs_another_packages_build_output_names_that_packages_source() {
>>>>>>> replace
```

The `declarations` tests: `options` in its usable form, and each shape it may not take.

```rust edit path=tests/declarations.test.rs
<<<<<<< find
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4, ENTRY-6a).
=======
//! Hands the built binary declarations and checks which it can use and how it refuses the rest (spec
//! KEY-1, KEY-2, KEY-5, RUN-4, ENTRY-6a; comparison spec CONF-1).
>>>>>>> replace
<<<<<<< find
            {
                "file": "runner.config.*",
                "property": ["test", "include"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["**/*.test.ts"]
            },
=======
            {
                "file": "runner.config.*",
                "property": ["test", "include"],
                "value": "globs",
                "relativeTo": "config",
                "default": ["**/*.test.ts"],
                "options": { "program": "runner", "names": ["--config", "-c"], "relativeTo": "script" }
            },
>>>>>>> replace
<<<<<<< find
    let cases: [(Value, &str); 14] = [
=======
    let cases: [(Value, &str); 20] = [
>>>>>>> replace
<<<<<<< find
        (
            json!({ "scriptCommands": "node" }),
=======
        (
            changed(&|entry| entry["options"] = json!(3)),
            ": its entry 1's field `options` must be an object; ",
        ),
        (
            changed(&|entry| {
                entry["options"] = json!({ "program": "runner", "names": ["-c"], "extra": 1 });
            }),
            ": its entry 1's field `options` holds `extra`, which it may not; ",
        ),
        (
            changed(&|entry| entry["options"] = json!({ "names": ["-c"] })),
            ": its entry 1's field `options` must name its `program`, one word with no white space; ",
        ),
        (
            changed(&|entry| entry["options"] = json!({ "program": "runner", "names": [] })),
            ": its entry 1's field `options` must list its option `names`, one or more; ",
        ),
        (
            changed(&|entry| {
                entry["options"] = json!({ "program": "runner", "names": ["--config x"] });
            }),
            ": its entry 1's field `options` names the option `--config x`, which is empty or holds white \
             space or `=`; ",
        ),
        (
            changed(&|entry| {
                entry["options"] =
                    json!({ "program": "runner", "names": ["-c"], "relativeTo": "package" });
            }),
            ": its entry 1's field `options` has a `relativeTo` that is not `config`, `detection` or \
             `script`; ",
        ),
        (
            json!({ "scriptCommands": "node" }),
>>>>>>> replace
```

- [ ] **Step 2: Run the tests to see them fail**

```
cargo test --release --locked --test entries --test declarations
```

Expected: FAIL. The new `entries` test and `uses_every_key_it_knows_in_every_form_a_glob_may_take`
stop with "its entry 1's field `options` is not one it may hold", and the six new shape cases of
`refuses_a_configuration_or_script_declaration_of_another_shape` get that line in place of theirs.
Every other test passes.

- [ ] **Step 3: Accept `options` and read the files scripts name by it**

```rust edit path=src/notice.rs
<<<<<<< find
#[derive(Debug)]
pub struct Notice {
=======
#[derive(Debug, PartialEq, Eq)]
pub struct Notice {
>>>>>>> replace
```

```rust edit path=src/declarations.rs
<<<<<<< find
/// The fields an object of `configEntries` may hold, each required but `pattern` (spec ENTRY-6).
const CONFIG_FIELDS: [&str; 6] = [
    "file",
    "property",
    "value",
    "pattern",
    "relativeTo",
    "default",
];
=======
/// The fields an object of `configEntries` may hold, each required but `pattern` and `options` (spec
/// ENTRY-6, CONF-1).
const CONFIG_FIELDS: [&str; 7] = [
    "file",
    "property",
    "value",
    "pattern",
    "relativeTo",
    "default",
    "options",
];

/// The fields the `options` of a `configEntries` object may hold, each required but `relativeTo`
/// (spec CONF-1).
const OPTIONS_FIELDS: [&str; 3] = ["program", "names", "relativeTo"];
>>>>>>> replace
<<<<<<< find
        if let Some(missing) = CONFIG_FIELDS
            .iter()
            .find(|name| **name != "pattern" && !fields.contains_key(**name))
        {
=======
        if let Some(missing) = CONFIG_FIELDS
            .iter()
            .find(|name| !matches!(**name, "pattern" | "options") && !fields.contains_key(**name))
        {
>>>>>>> replace
<<<<<<< find
        match texts(&fields["default"]) {
            Some(globs) => globs
=======
        if let Some(why) = fields.get("options").and_then(options_problem) {
            return field("options", &why);
        }

        match texts(&fields["default"]) {
            Some(globs) => globs
>>>>>>> replace
<<<<<<< find
/// The entries of a list of text, or nothing when the value is not one.
=======
/// Why the `options` of a `configEntries` object cannot be used, or nothing when they can: an object
/// with the program's name, a list of one or more option names, and, optionally, what a named file's
/// value is relative to (spec CONF-1).
fn options_problem(options: &Value) -> Option<String> {
    let Some(fields) = options.as_object() else {
        return Some("must be an object".to_owned());
    };

    if let Some(unknown) = fields
        .keys()
        .find(|key| !OPTIONS_FIELDS.contains(&key.as_str()))
    {
        return Some(format!("holds `{unknown}`, which it may not"));
    }

    if !fields
        .get("program")
        .and_then(Value::as_str)
        .is_some_and(is_word)
    {
        return Some("must name its `program`, one word with no white space".to_owned());
    }

    let Some(names) = fields
        .get("names")
        .and_then(texts)
        .filter(|names| !names.is_empty())
    else {
        return Some("must list its option `names`, one or more".to_owned());
    };

    if let Some(name) = names
        .iter()
        .find(|name| !is_word(name) || name.contains('='))
    {
        return Some(format!(
            "names the option `{name}`, which is empty or holds white space or `=`"
        ));
    }

    match fields.get("relativeTo").map(Value::as_str) {
        None | Some(Some("config" | "detection" | "script")) => None,
        Some(_) => {
            Some("has a `relativeTo` that is not `config`, `detection` or `script`".to_owned())
        }
    }
}

/// Whether a text is one word: not empty, with no white space.
fn is_word(text: &str) -> bool {
    !text.is_empty() && !text.contains(char::is_whitespace)
}

/// The entries of a list of text, or nothing when the value is not one.
>>>>>>> replace
```

```rust edit path=src/entries.rs
<<<<<<< find
//! They come from the project's standard files, the scripts of each `package.json`, the `entries` and
//! `internal` declarations, and the configuration files `configEntries` names (spec ENTRY-6, ENTRY-7).
=======
//! They come from the project's standard files, the scripts of each `package.json`, the `entries` and
//! `internal` declarations, and the configuration files `configEntries` names, by their names or by the
//! option a script gives them after (spec ENTRY-6, ENTRY-7; comparison spec CONF-1 to CONF-3).
>>>>>>> replace
<<<<<<< find
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

                // A glob that matches no listed file is mapped back from build output, and one that
                // matches nothing either way adds nothing, quietly (spec ENTRY-6c).
                for written in declaration.globs(project.tree, &config.path, folder, notices) {
                    let Ok(glob) = Glob::new(&written) else {
                        continue;
                    };
                    let mut found: Vec<&str> = project
                        .files
                        .iter()
                        .map(|file| file.path.as_str())
                        .filter(|path| glob.matches(path))
                        .collect();

                    if found.is_empty() {
                        let mapped: Vec<Glob> = mapped_globs(&written, "", project)
                            .iter()
                            .filter_map(|mapped| Glob::new(mapped).ok())
                            .collect();

                        found = project
                            .files
                            .iter()
                            .map(|file| file.path.as_str())
                            .filter(|path| mapped.iter().any(|glob| glob.matches(path)))
                            .collect();
                    }

                    for path in found {
                        entries.add(path.to_owned(), named.clone());
                    }
                }
            }
        }
    }
}
=======
/// The entry points the `configEntries` declarations of the packs name: each configuration file a
/// declaration's `file` matches, once, from the deepest folder yak's detection looked in that holds it,
/// and each one a script names after an option the declaration's `options` lists; and the files each
/// one's value names, read as a literal where it is one and the declaration's `default` where it is
/// not. A file both matched and named is read from each folder its value is relative to. A glob of the
/// file's that the analyzer does not read is passed over with a notice, and so is a word of a script
/// that names no listed file (spec ENTRY-6, CONF-1 to CONF-3).
fn configured(project: &Project<'_>, entries: &mut Entries, notices: &mut Vec<Notice>) {
    let Some(handed) = project.handed else {
        return;
    };
    let listed: BTreeSet<&str> = project
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    let mut missing = BTreeSet::new();

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
            // Each configuration file the declaration reads, with each folder its value is read from.
            let mut readings: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

            for config in project.files {
                let Some(base) = deepest(&handed.folders, &config.path).filter(|base| {
                    inside(base, &config.path).is_some_and(|rest| declaration.file.matches(rest))
                }) else {
                    continue;
                };
                let folder = relative(declaration.relative_to, &config.path, base, base).to_owned();

                readings
                    .entry(config.path.clone())
                    .or_default()
                    .insert(folder);
            }

            if let Some(options) = &declaration.options {
                let relative_to = options.relative_to.unwrap_or(declaration.relative_to);

                for (script, path) in named_configs(project, options, &listed, &mut missing) {
                    let detection = deepest(&handed.folders, &path).unwrap_or("");
                    let folder = relative(relative_to, &path, detection, script).to_owned();

                    readings.entry(path).or_default().insert(folder);
                }
            }

            for (config, folders) in &readings {
                let named = Source {
                    file: Some(config.clone()),
                    ..source.clone()
                };
                let mut said: Vec<Notice> = Vec::new();

                entries.add(config.clone(), source.clone());

                for folder in folders {
                    let mut read = Vec::new();

                    for written in declaration.globs(project.tree, config, folder, &mut read) {
                        for path in matching(project, &written) {
                            entries.add(path.to_owned(), named.clone());
                        }
                    }

                    // A glob passed over from two folders is said once.
                    for notice in read {
                        if !said.contains(&notice) {
                            said.push(notice);
                        }
                    }
                }

                notices.extend(said);
            }
        }
    }

    notices.extend(missing.into_iter().map(|(manifest, sentence)| {
        Notice::new("script-config-not-listed", Some(manifest), sentence)
    }));
}

/// The listed files a glob of a configuration file's value matches, from the root: as written, or,
/// where it matches none, mapped back from build output; nothing where it matches neither, quietly
/// (spec ENTRY-6c).
fn matching<'p>(project: &Project<'p>, written: &str) -> Vec<&'p str> {
    let Ok(glob) = Glob::new(written) else {
        return Vec::new();
    };
    let found: Vec<&str> = project
        .files
        .iter()
        .map(|file| file.path.as_str())
        .filter(|path| glob.matches(path))
        .collect();

    if !found.is_empty() {
        return found;
    }

    let mapped: Vec<Glob> = mapped_globs(written, "", project)
        .iter()
        .filter_map(|mapped| Glob::new(mapped).ok())
        .collect();

    project
        .files
        .iter()
        .map(|file| file.path.as_str())
        .filter(|path| mapped.iter().any(|glob| glob.matches(path)))
        .collect()
}

/// The folder, from the root, a configuration file's value is relative to: the file's own for
/// `config`, that of the `package.json` whose script named the file for `script`, and the detection
/// folder given for `detection` (spec ENTRY-6, CONF-1).
fn relative<'f>(
    relative_to: &str,
    config: &'f str,
    detection: &'f str,
    script: &'f str,
) -> &'f str {
    match relative_to {
        "config" => config.rsplit_once('/').map_or("", |(folder, _)| folder),
        "script" => script,
        _ => detection,
    }
}

/// The configuration files the scripts of every `package.json` name after one of a declaration's
/// options, each with the folder, from the root, of the `package.json` whose script names it, which the
/// word is found from. A word that names no listed file is kept in `missing`, by its `package.json`,
/// with the sentence of its notice (spec CONF-2).
fn named_configs<'p>(
    project: &Project<'p>,
    options: &Options<'_>,
    listed: &BTreeSet<&str>,
    missing: &mut BTreeSet<(String, String)>,
) -> Vec<(&'p str, String)> {
    let mut named = Vec::new();

    for package in &project.packages.list {
        let folder = folder_path(&package.folder);
        let manifest = manifest_path(&package.folder);

        for script in &package.scripts {
            for command in tools::commands(script) {
                for (option, value) in options.values(&command) {
                    match resolved(folder, value).filter(|path| listed.contains(path.as_str())) {
                        Some(path) => named.push((folder, path)),
                        None => {
                            missing.insert((
                                manifest.clone(),
                                format!(
                                    "The script `{script}` in {manifest} names `{value}` after `{option}` as \
                                     a configuration file of `{}`, and no listed file is there, so it adds no \
                                     entry point.",
                                    options.program
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    named
}

/// A word of a command as a program's name: its last path segment, with a `.cmd`, `.exe` or `.ps1`
/// ending dropped (spec CONF-2).
fn program_name(word: &str) -> &str {
    let last = word.rsplit(['/', '\\']).next().unwrap_or(word);

    [".cmd", ".exe", ".ps1"]
        .iter()
        .find_map(|ending| last.strip_suffix(ending))
        .unwrap_or(last)
}
>>>>>>> replace
<<<<<<< find
    /// The globs used where the file states no value the analyzer reads.
    default: Vec<&'d str>,
}
=======
    /// The globs used where the file states no value the analyzer reads.
    default: Vec<&'d str>,
    /// The options a script names the configuration file after, where the declaration states them
    /// (spec CONF-1).
    options: Option<Options<'d>>,
}

/// The `options` of a `configEntries` declaration: the program whose scripts name a configuration file,
/// the options that name it, and what its value is relative to where that differs from the
/// declaration's own (spec CONF-1).
struct Options<'d> {
    /// The program, by its name.
    program: &'d str,
    /// The option names, each given before its value or as `<name>=value`.
    names: Vec<&'d str>,
    /// What a named file's value is relative to, `config`, `detection` or `script`; none for the
    /// declaration's own.
    relative_to: Option<&'d str>,
}

impl<'d> Options<'d> {
    /// The options, where they have their shape.
    fn of(options: &'d Value) -> Option<Self> {
        Some(Self {
            program: options.get("program")?.as_str()?,
            names: texts(options.get("names")),
            relative_to: options.get("relativeTo").and_then(Value::as_str),
        })
    }

    /// Whether a word is one of the option names.
    fn names(&self, word: &str) -> bool {
        self.names.contains(&word)
    }

    /// The values a command gives the options, each with the option that gave it: among the words after
    /// the command's first word that is the program, the word after one of the option names, and the
    /// value of a word `<name>=value`, each time an option is given (spec CONF-2).
    fn values<'c>(&self, command: &'c [String]) -> Vec<(&'c str, &'c str)> {
        let Some(start) = command
            .iter()
            .position(|word| program_name(word) == self.program)
        else {
            return Vec::new();
        };
        let mut found = Vec::new();
        let mut words = command[start + 1..].iter();

        while let Some(word) = words.next() {
            if self.names(word) {
                if let Some(value) = words.next() {
                    found.push((word.as_str(), value.as_str()));
                }
            } else if let Some((name, value)) = word.split_once('=')
                && self.names(name)
            {
                found.push((name, value));
            }
        }

        found
    }
}
>>>>>>> replace
<<<<<<< find
            relative_to: declaration.get("relativeTo")?.as_str()?,
            default: texts(declaration.get("default")),
        })
=======
            relative_to: declaration.get("relativeTo")?.as_str()?,
            default: texts(declaration.get("default")),
            options: declaration.get("options").and_then(Options::of),
        })
>>>>>>> replace
```

- [ ] **Step 4: Run the tests to see them pass**

```
cargo test --release --locked --test entries --test declarations
```

Expected: PASS, every test in the two files.

- [ ] **Step 5: Run the gates**

```
cargo fmt --check
cargo clippy --release --all-targets --locked -- -D warnings
cargo test --release --locked
node --test "npm/**/*.test.mjs" "scripts/**/*.test.mjs"
```

Expected: each exits 0.

- [ ] **Step 6: Commit**

```
yak stack format
git add src/declarations.rs src/entries.rs src/notice.rs tests/entries.test.rs tests/declarations.test.rs
git commit -m "feat(entries): read a configuration file a script names by an option"
```

Expected: one commit on `feat/comparison-2-config-option`, with the trailer lines the session's
attribution asks for.
