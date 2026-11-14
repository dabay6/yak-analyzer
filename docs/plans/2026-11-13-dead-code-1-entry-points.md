# Dead-code findings, part 1: entry points from the standard files and the entries declarations: implementation plan

**Goal:** `graph` lists every entry point the project's standard files and its `entries` and
`internal` declarations name, each with every source that made it one, and the packages whose entry
points are likely incomplete.

**Architecture:** A new `src/entries.rs` reads the entry points after the imports are resolved. It
reuses the resolver to map a `package.json` target back from build output to its source (MAP-1),
gains the resolver's places for an `exports` pattern, and reads a tsconfig's `files` from what
`src/tsconfig.rs` already parsed. `src/packages.rs` reads `bin`. `graph` prints the result as two new
top-level lists, so nothing it printed before changes form.

**Spec:** docs/specs/2026-11-13-dead-code-findings-design.md

**Implements:** ENTRY-1, ENTRY-2, ENTRY-3, ENTRY-4, ENTRY-5, ENTRY-8, CMD-4, FIX-1

This is the first of the item's plans, worked on the item's branch, `feat/dead-code-findings`. The
spec leaves the split to the builder; the session's plans, in order:

1. this one: entry points from the standard files and the `entries` and `internal` declarations;
2. entry points from a `package.json`'s scripts and from a tool's configuration file (ENTRY-6,
   ENTRY-6a, ENTRY-7), and the declaration keys' shapes;
3. the names of each file, the uses, and the unused files, exports and types, Vue and MDX included
   (USE-*, KIND-1, KIND-2, ENTRY-9, ENTRY-10);
4. unresolved imports and cycles, the SARIF report and the `audit` command (KIND-3 to KIND-5,
   REPORT-*, CMD-1 to CMD-3, CMD-5);
5. reading a commit, new against inherited, and the life of a run (COMMIT-*, INH-*, LIFE-*);
6. the check on the named repositories (LIVE-*), with DEP-1 and DOC-1 closing it.

## Global constraints

- A tool's convention is a declaration, never code: nothing here names a tool (design ENT-5).
- Whether a path exists is asked of the file list, never of the file system, and a target that names
  build output maps back to its source by MAP-1, so the result is the same with and without build
  output and `node_modules` (reading LIST-4, design AN-5).
- `graph` stays outside the interface version, so its new lists may change form later; the facts the
  fixtures hold do not (CMD-4, reading OUT-2).
- Every crate stays pinned; this plan adds none.

## The decisions this plan makes

1. **What `graph` prints.** Two top-level lists, so the objects it printed before keep their form and
   no earlier fixture changes but the one that compares a whole document. `entries` lists each entry
   point by path, with `sources`, each `{ by, file, from, stack }`: `from` is the field of the
   `package.json` (`main`, `module`, `types`, `typings`, `bin`, `exports`), `files` for a tsconfig's
   list, `page` for a page's script, or `entries` for a declaration; `file` is the `package.json`, the
   tsconfig or the page; `stack` and `by` (`pack` or `project`) are a declaration's. `entryFaults` lists
   each package whose entry points are likely incomplete, `{ fault, package }`, the fault
   `unmapped-target` or `no-entry-point` (ENTRY-8).
2. **ENTRY-9's notices come with the findings, in plan 3.** They say where unused files, exports and
   types are not reported, which only means something once they are reported; `graph` already prints
   each package's PACK-4 mark as `detected`. Moving them there also keeps this plan from changing the
   notices every earlier fixture checks.
3. **Which targets mark a package (ENTRY-8).** A `bin` target, a plain `exports` target, and an
   `exports` pattern that fills from no listed file. A target that is not a path in the package, one
   that does not begin with `./`, is not read, as Node does not read it; `main`, `module` and `types`
   that map to nothing mark nothing, as ENTRY-8 names only `bin` and `exports`. A package with no
   entry point at all is marked when no fault of the first kind is.
4. **An `exports` pattern** is matched forward: the target's places, its source forms, the target
   itself, and MAP-1's places (`Resolver::pattern_places`), each still holding its `*`, are compared
   with every listed file of the package. The `*` is any text that is not empty, `/` included, as
   `resolve::matched` already reads a key; a target holding three or more `*` matches nothing. A
   filled subpath that a key with a `null` target matches, by Node's choice of the most specific key,
   is hidden.
5. **Which `entries` globs are mapped back (ENTRY-3).** A glob names build output only by its ending,
   so only a glob whose last segment ends in an output ending (`.js`, `.mjs`, `.cjs`, a type
   declaration's), alone or among braces, is mapped. Without that, a glob such as `e2e/**` that matches
   nothing would map to `src/**` and make every source an entry point.
6. **`internal` paths** are package folders: a pack's joined to each detection folder, a project's from
   the root, `.` and empty segments passed over, and `.` meaning the folder itself.
7. **The `bin` field** is read as a path or as an object of paths, its commands taken by name, which is
   the order `serde_json` keeps them in; any other form is PACK-6's stop, step 11.
8. **A tsconfig's `files`** are its own, not those of a config it extends, each path from the config's
   folder; one above the root is left out.

## Review focus

- **Two `*` in a pattern with a short repeated text** (`./*/*.js` against `a/a/a/a.ts`): the split is
  computed from the lengths, so only an even split is tried. No fixture holds one that has two
  splits; Node itself has none, since its `*` is filled once from the subpath.
- **An `exports` target that holds a `*` under a key that holds none**, and the reverse, are read as a
  plain target; such a target then maps to nothing and marks the package. No fixture holds one.
- **A pack's `entries` glob that matches in one detection folder and not in another** is mapped back
  only in the folders where it matched nothing, and gives no notice. The fourth test holds a glob that
  matches in `packages/b` and maps in `packages/a`.
- **A page that is data** is never read, so its scripts are not entry points; the reading spec's
  fixtures already hold a data page.

### Task 1: entry points from the standard files and the entries declarations, in graph

**Implements:** ENTRY-1, ENTRY-2, ENTRY-3, ENTRY-4, ENTRY-5, ENTRY-8, CMD-4, FIX-1

**Test:** `cargo test --release --locked --test entries --test hand-off`

**Files:**
- Create: `src/entries.rs`
- Modify: `src/packages.rs`, `src/resolve.rs`, `src/tsconfig.rs`, `src/graph.rs`, `src/main.rs`,
  `Cargo.toml`
- Test: `tests/entries.test.rs`, `tests/hand-off.test.rs`

**Interfaces:**
- Consumes: `packages::Package` and `Packages`; `resolve::Resolver` with its `target` and `sources`;
  `tsconfig::Configs` with `builds`; `hand_off::HandOff`; `glob::Glob`; `files::File`.
- Produces: `entries::find`, `entries::Project`, `entries::Entries` (`points`, `faults`),
  `entries::Source::printed` and `entries::Fault::name`; `Package::starts`;
  `Resolver::package_target` and `Resolver::pattern_places`; `resolve::matched` made public;
  `Configs::named_files`. Plan 3 reads `Entries` to find what is reachable, and plan 2 adds the
  scripts' and the configuration files' entry points to it.

- [ ] **Step 1: Write the failing tests**

The test of a whole document in `tests/hand-off.test.rs` gains the two new lists.

```rust path=tests/entries.test.rs
//! Runs the built binary on made-up projects and checks the entry points it finds, the source of each,
//! and the packages whose entry points are likely incomplete (spec ENTRY-1 to ENTRY-5, ENTRY-8, CMD-4,
//! FIX-1).

#[path = "support/project.rs"]
mod project;

use serde_json::{Value, json};

use project::{Project, notices, paths, printed, stopped};

/// A source that a file of the project names.
fn named(from: &str, file: &str) -> Value {
    json!({ "by": null, "file": file, "from": from, "stack": null })
}

/// A source that a declaration of the stack `typescript` names, the pack's or the project's.
fn declared(by: &str) -> Value {
    json!({ "by": by, "file": null, "from": "entries", "stack": "typescript" })
}

/// One stack named `typescript` that holds the declarations given, the pack's and the project's.
fn stack(pack: Value, project: Value) -> Value {
    json!([{ "name": "typescript", "interfaceVersion": 1, "pack": pack, "project": project }])
}

/// The entry points a document lists, each with its sources.
fn entries(document: &Value) -> Vec<(&str, Vec<Value>)> {
    document["entries"]
        .as_array()
        .expect("a list of entry points")
        .iter()
        .map(|entry| {
            (
                entry["path"].as_str().expect("a path"),
                entry["sources"]
                    .as_array()
                    .expect("a list of sources")
                    .clone(),
            )
        })
        .collect()
}

/// The packages a document marks as likely missing entry points, each with why.
fn faults(document: &Value) -> Vec<(&str, &str)> {
    document["entryFaults"]
        .as_array()
        .expect("a list of faults")
        .iter()
        .map(|fault| {
            (
                fault["package"].as_str().expect("a folder"),
                fault["fault"].as_str().expect("a fault"),
            )
        })
        .collect()
}

#[test]
fn a_package_jsons_main_module_types_and_bin_name_entry_points_mapped_back_to_their_source() {
    let project = Project::new("entries-package-fields");

    project
        .write(
            "package.json",
            r#"{
                "name": "lib",
                "main": "./dist/index.js",
                "module": "./dist/index.mjs",
                "types": "./dist/index.d.ts",
                "bin": { "lib": "./bin/cli.js", "lib-admin": "./dist/admin.js" }
            }"#,
        )
        .write(
            "tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "dist", "rootDir": "src" } }"#,
        )
        .write("src/index.ts", "")
        .write("src/admin.ts", "")
        .write("src/other.ts", "")
        .write("bin/cli.js", "")
        .commit();

    let document = project.graph();

    assert_eq!(
        entries(&document),
        [
            ("bin/cli.js", vec![named("bin", "package.json")]),
            ("src/admin.ts", vec![named("bin", "package.json")]),
            (
                "src/index.ts",
                vec![
                    named("main", "package.json"),
                    named("module", "package.json"),
                    named("types", "package.json"),
                ]
            ),
        ]
    );
    assert_eq!(faults(&document), []);
}

#[test]
fn exports_patterns_fill_one_star_twice_hide_what_a_null_key_matches_and_add_nothing_when_internal()
{
    let project = Project::new("entries-exports");

    project
        .write(
            "package.json",
            r#"{ "private": true, "workspaces": ["packages/*"] }"#,
        )
        .write(
            "packages/ui/package.json",
            r#"{
                "name": "ui",
                "exports": {
                    ".": { "types": "./dist/index.d.ts", "default": "./dist/index.js" },
                    "./*": "./dist/*/*.js",
                    "./secret": null
                }
            }"#,
        )
        .write(
            "packages/ui/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "dist", "rootDir": "src" } }"#,
        )
        .write("packages/ui/src/index.ts", "")
        .write("packages/ui/src/button/button.ts", "")
        .write("packages/ui/src/card/card.tsx", "")
        .write("packages/ui/src/secret/secret.ts", "")
        .write("packages/ui/src/button/helper.ts", "")
        .commit();

    let document = project.graph();
    let exported = || vec![named("exports", "packages/ui/package.json")];

    // `./*` reaches `button/button` and `card/card`, never `button/helper`; `./secret` is hidden.
    assert_eq!(
        entries(&document),
        [
            ("packages/ui/src/button/button.ts", exported()),
            ("packages/ui/src/card/card.tsx", exported()),
            ("packages/ui/src/index.ts", exported()),
        ]
    );
    assert_eq!(faults(&document), [(".", "no-entry-point")]);

    // Declared internal, the package's `exports` adds no entry points (spec ENTRY-5).
    let file = project.hand_off(
        &project.root,
        &[".", "packages/ui"],
        stack(json!({}), json!({ "internal": ["packages/ui"] })),
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    assert_eq!(entries(&document), []);
    assert_eq!(
        faults(&document),
        [(".", "no-entry-point"), ("packages/ui", "no-entry-point")]
    );
}

#[test]
fn a_bin_or_exports_target_with_no_source_marks_its_package() {
    let project = Project::new("entries-unmapped");

    project
        .write(
            "package.json",
            r#"{
                "name": "generated",
                "bin": "./bin/run.js",
                "exports": { "./schema": "./generated/schema.js" }
            }"#,
        )
        .write("lib/x.ts", "")
        .commit();

    let document = project.graph();

    assert_eq!(entries(&document), []);
    assert_eq!(faults(&document), [(".", "unmapped-target")]);
}

#[test]
fn a_tsconfigs_files_and_a_pages_scripts_are_entry_points() {
    let project = Project::new("entries-files-and-pages");

    project
        .write("package.json", r#"{ "name": "app" }"#)
        .write("tsconfig.json", r#"{ "files": ["src/setup.ts"] }"#)
        .write(
            "public/admin.html",
            r#"<script type="module" src="../src/main.ts"></script>"#,
        )
        .write("src/setup.ts", "")
        .write("src/main.ts", "")
        .write("src/other.ts", "")
        .commit();

    let document = project.graph();

    assert_eq!(
        entries(&document),
        [
            ("src/main.ts", vec![named("page", "public/admin.html")]),
            ("src/setup.ts", vec![named("files", "tsconfig.json")]),
        ]
    );
    assert_eq!(faults(&document), []);
}

#[test]
fn entries_globs_apply_a_packs_in_each_folder_and_a_projects_from_the_root_mapped_from_build_output()
 {
    let project = Project::new("entries-declared");

    project
        .write(
            "package.json",
            r#"{ "private": true, "workspaces": ["packages/*"] }"#,
        )
        .write("packages/a/package.json", r#"{ "name": "a" }"#)
        .write(
            "packages/a/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "build", "rootDir": "src" } }"#,
        )
        .write("packages/a/src/x.ts", "")
        .write("packages/a/src/x.test.ts", "")
        .write("packages/a/src/deep/y.test.tsx", "")
        .write("packages/b/package.json", r#"{ "name": "b" }"#)
        .write("packages/b/test/z.test.js", "")
        .write("scripts/release.mjs", "")
        .commit();

    let declarations = stack(
        json!({ "entries": ["build/**/*.test.js", "test/**/*.test.js", "nothing/**"] }),
        json!({ "entries": ["scripts/*.mjs"] }),
    );
    let file = project.hand_off(
        &project.root,
        &[".", "packages/a", "packages/b"],
        declarations,
    );
    let document = printed(&project.run(|binary| {
        binary.args(["--hand-off", &file]);
    }));

    // In `packages/a` the built tests map back to their sources; in `packages/b` they are committed
    // JavaScript and match as written (spec ENTRY-3, ENTRY-4).
    assert_eq!(
        entries(&document),
        [
            ("packages/a/src/deep/y.test.tsx", vec![declared("pack")]),
            ("packages/a/src/x.test.ts", vec![declared("pack")]),
            ("packages/b/test/z.test.js", vec![declared("pack")]),
            ("scripts/release.mjs", vec![declared("project")]),
        ]
    );
    assert_eq!(notices(&document), ["glob-matches-nothing"]);
    assert_eq!(
        document["notices"][0]["sentence"],
        json!(
            "The `entries` glob `nothing/**` that the pack states for the stack `typescript` matches no file, \
             nor any source mapped back from build output."
        )
    );
}

#[test]
fn a_bin_of_a_form_npm_does_not_give_it_stops_the_run() {
    let project = Project::new("entries-bin-form");

    project
        .write("package.json", r#"{ "name": "tool", "bin": ["./cli.js"] }"#)
        .write("cli.js", "")
        .commit();

    // The field is read in every form npm gives it, and a value of another form stops the run at
    // RUN-4's step 11 (spec ENTRY-1, PACK-6).
    assert_eq!(
        stopped(&project.run(|_| {})),
        "yak-analyzer: the field `bin` in package.json holds a value of a form it does not have; it must be a \
         path, or an object that maps each command to a path; fix it"
    );

    project
        .write("package.json", r#"{ "name": "tool", "bin": "./cli.js" }"#)
        .commit();

    let document = project.graph();

    assert_eq!(paths(&document), ["cli.js", "package.json"]);
    assert_eq!(
        entries(&document),
        [("cli.js", vec![named("bin", "package.json")])]
    );
}
```

```rust edit path=tests/hand-off.test.rs
<<<<<<< find
    assert_eq!(
        printed,
        json!({
            "files": [],
            "handOff": true,
            "notices": [
=======
    assert_eq!(
        printed,
        json!({
            "entries": [],
            "entryFaults": [],
            "files": [],
            "handOff": true,
            "notices": [
>>>>>>> replace
```

```toml edit path=Cargo.toml
<<<<<<< find
name = "pages"
path = "tests/pages.test.rs"

[dependencies]
# Parse scripts into syntax trees and walk them; released together, so pinned together (spec AN-2, DEP-3).
oxc_allocator = "=0.158.0"
=======
name = "pages"
path = "tests/pages.test.rs"

[[test]]
name = "entries"
path = "tests/entries.test.rs"

[dependencies]
# Parse scripts into syntax trees and walk them; released together, so pinned together (spec AN-2, DEP-3).
oxc_allocator = "=0.158.0"
>>>>>>> replace
```

- [ ] **Step 2: Run the tests and see them fail**

```bash
cargo test --release --locked --test entries --test hand-off
```

Expected: FAIL. `graph` prints no `entries` and no `entryFaults`, so every test of
`tests/entries.test.rs` but the one that stops on `bin` fails on a missing list, that one fails because
`bin` is not read yet, and `reads_a_hand_off_file_it_can_use` fails on the two lists it now expects.

- [ ] **Step 3: Write the code**

`src/packages.rs` reads `bin`, and keeps each field that makes an entry point with its target:

```rust edit path=src/packages.rs
<<<<<<< find
    pub imports: Option<Value>,
    /// The targets of its `types` (or `typings`), `module` and `main`, in that order, those it states.
    pub entries: Vec<String>,
}

/// What a `package.json` states that a package keeps.
=======
    pub imports: Option<Value>,
    /// The targets of its `types` (or `typings`), `module` and `main`, in that order, those it states.
    pub entries: Vec<String>,
    /// The fields that make files entry points, each with the target it states: `types` (or
    /// `typings`), `module`, `main`, then each of `bin` (spec ENTRY-1).
    pub starts: Vec<(&'static str, String)>,
}

/// What a `package.json` states that a package keeps.
>>>>>>> replace
<<<<<<< find
    dependencies: BTreeMap<String, String>,
    exports: Option<Value>,
    imports: Option<Value>,
    entries: Vec<String>,
}

/// The packages of a project, and which package each file belongs to.
=======
    dependencies: BTreeMap<String, String>,
    exports: Option<Value>,
    imports: Option<Value>,
    entries: Vec<(&'static str, String)>,
    bin: Vec<String>,
}

/// The packages of a project, and which package each file belongs to.
>>>>>>> replace
<<<<<<< find
                    None => folder == "." || listed,
                };

                Package {
                    folder: folder.to_owned(),
                    name: fields.name,
=======
                    None => folder == "." || listed,
                };

                let starts = fields
                    .entries
                    .iter()
                    .cloned()
                    .chain(fields.bin.iter().map(|target| ("bin", target.clone())))
                    .collect();

                Package {
                    folder: folder.to_owned(),
                    name: fields.name,
>>>>>>> replace
<<<<<<< find
                    dependencies: fields.dependencies,
                    exports: fields.exports,
                    imports: fields.imports,
                    entries: fields.entries,
                }
            })
            .collect();
=======
                    dependencies: fields.dependencies,
                    exports: fields.exports,
                    imports: fields.imports,
                    entries: fields
                        .entries
                        .into_iter()
                        .map(|(_, target)| target)
                        .collect(),
                    starts,
                }
            })
            .collect();
>>>>>>> replace
<<<<<<< find
        exports: targets(fields, "exports", file)?,
        imports: imports(fields, file)?,
        entries: entries(fields, file)?,
    })
}

=======
        exports: targets(fields, "exports", file)?,
        imports: imports(fields, file)?,
        entries: entries(fields, file)?,
        bin: bin(fields, file)?,
    })
}

>>>>>>> replace
<<<<<<< find
}

/// The targets of a `package.json`'s `types` (or its older name `typings`), `module` and `main`, in
/// that order, those it states.
fn entries(fields: &Map<String, Value>, file: &str) -> Result<Vec<String>, Stop> {
    let types = if fields.contains_key("types") {
        "types"
    } else {
=======
}

/// The targets of a `package.json`'s `types` (or its older name `typings`), `module` and `main`, in
/// that order, those it states, each with the field that states it.
fn entries(fields: &Map<String, Value>, file: &str) -> Result<Vec<(&'static str, String)>, Stop> {
    let types = if fields.contains_key("types") {
        "types"
    } else {
>>>>>>> replace
<<<<<<< find
    for field in [types, "module", "main"] {
        match fields.get(field) {
            None => {}
            Some(Value::String(target)) => entries.push(target.clone()),
            Some(_) => return Err(misshapen(field, file, "a path")),
        }
    }
=======
    for field in [types, "module", "main"] {
        match fields.get(field) {
            None => {}
            Some(Value::String(target)) => entries.push((field, target.clone())),
            Some(_) => return Err(misshapen(field, file, "a path")),
        }
    }
>>>>>>> replace
<<<<<<< find
    Ok(entries)
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();
=======
    Ok(entries)
}

/// The paths a `package.json`'s `bin` states: one path, or an object that maps each command to a path,
/// by the commands' names (spec ENTRY-1, PACK-6).
fn bin(fields: &Map<String, Value>, file: &str) -> Result<Vec<String>, Stop> {
    let form = "a path, or an object that maps each command to a path";

    match fields.get("bin") {
        None => Ok(Vec::new()),
        Some(Value::String(target)) => Ok(vec![target.clone()]),
        Some(Value::Object(commands)) => commands
            .values()
            .map(|target| {
                target
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| misshapen("bin", file, form))
            })
            .collect(),
        Some(_) => Err(misshapen("bin", file, form)),
    }
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();
>>>>>>> replace
```

The resolver maps a single target of a package back to its source, and gives the places an
`exports` pattern's source may lie in; `matched` is made public for the `null` keys:

```rust edit path=src/resolve.rs
<<<<<<< find
        }
    }

    /// Every file a package's `types`, `module` and `main` lead to; nothing when it states none of them
    /// (spec RES-3, RES-6).
    fn entries(&self, package: &Package) -> Option<Found> {
=======
        }
    }

    /// The files a target of a package's `package.json` leads to, a path from the package's folder,
    /// mapped back to the source it is built from where it names build output; nothing when it leads to
    /// no file (spec ENTRY-1, MAP-1).
    pub fn package_target(&self, package: &Package, target: &str) -> Option<Vec<String>> {
        let folder = folder_path(&package.folder);
        let path = joined(folder, target)?;

        match self.target(folder, &path, None) {
            Found::Done(Outcome::Files(files)) => Some(files),
            _ => None,
        }
    }

    /// The paths, from the root, where the source of an `exports` target with a `*` may lie: the target
    /// in its source forms, the target itself, then the places MAP-1 maps build output to, each still
    /// holding its `*` (spec ENTRY-2, MAP-3).
    pub fn pattern_places(&self, package: &Package, target: &str) -> Vec<String> {
        let folder = folder_path(&package.folder);
        let Some(path) = joined(folder, target) else {
            return Vec::new();
        };
        let mut places: Vec<String> = SOURCE_FORMS
            .iter()
            .find_map(|(output, forms)| {
                path.strip_suffix(output)
                    .map(|stem| forms.iter().map(|form| format!("{stem}{form}")).collect())
            })
            .unwrap_or_default();

        let mapped = self.sources(folder, &path, true);

        places.push(path);
        places.extend(mapped);
        places
    }

    /// Every file a package's `types`, `module` and `main` lead to; nothing when it states none of them
    /// (spec RES-3, RES-6).
    fn entries(&self, package: &Package) -> Option<Found> {
>>>>>>> replace
<<<<<<< find
/// The key of an `exports` or `imports` map that a subpath matches, as Node matches one: the key
/// itself when it has no `*`, or else the pattern with the longest part before its one `*`, the
/// longer key where two are as long; with the text the `*` stands for (spec RES-5, RES-6).
fn matched<'v, 'w>(
    map: &'v Map<String, Value>,
    wanted: &'w str,
) -> Option<(&'v Value, Option<&'w str>)> {
=======
/// The key of an `exports` or `imports` map that a subpath matches, as Node matches one: the key
/// itself when it has no `*`, or else the pattern with the longest part before its one `*`, the
/// longer key where two are as long; with the text the `*` stands for (spec RES-5, RES-6).
pub fn matched<'v, 'w>(
    map: &'v Map<String, Value>,
    wanted: &'w str,
) -> Option<(&'v Value, Option<&'w str>)> {
>>>>>>> replace
```

`src/tsconfig.rs` keeps the files each config names in its own `files`:

```rust edit path=src/tsconfig.rs
<<<<<<< find
    /// The builds of the configs in each folder, by the folder, the root's as the empty path: those of
    /// `tsconfig.json` first, then the others by path.
    builds: BTreeMap<String, Vec<Build>>,
}

/// Reads every tsconfig in the list that is not data, and the configs above the root they extend. A
=======
    /// The builds of the configs in each folder, by the folder, the root's as the empty path: those of
    /// `tsconfig.json` first, then the others by path.
    builds: BTreeMap<String, Vec<Build>>,
    /// The files each config under the root names in its own `files`, as pairs of the config's path and
    /// the file's, both from the root, in the order of the configs and of each list (spec ENTRY-1).
    pub named_files: Vec<(String, String)>,
}

/// Reads every tsconfig in the list that is not data, and the configs above the root they extend. A
>>>>>>> replace
<<<<<<< find
        }

        let builds = self.builds();
        let options = ResolveOptions {
            tsconfig: Some(TsconfigDiscovery::Auto),
            ..ResolveOptions::default()
=======
        }

        let builds = self.builds();
        let named_files = self.named_files();
        let options = ResolveOptions {
            tsconfig: Some(TsconfigDiscovery::Auto),
            ..ResolveOptions::default()
>>>>>>> replace
<<<<<<< find
            resolver: ResolverGeneric::new_with_file_system(Served(Arc::new(served)), options),
            root: self.root,
            builds,
        })
    }

    /// A config as the library is given it: only the fields it uses, with each target of `extends` and
    /// `references` the analyzer found, written from the config's folder.
    fn served(&self, path: &str, config: &Config) -> Value {
=======
            resolver: ResolverGeneric::new_with_file_system(Served(Arc::new(served)), options),
            root: self.root,
            builds,
            named_files,
        })
    }

    /// The files each config under the root names in its own `files`, each from the root; a path that
    /// leads above the root is left out (spec ENTRY-1).
    fn named_files(&self) -> Vec<(String, String)> {
        self.configs
            .iter()
            .filter(|(path, _)| !is_above(path))
            .flat_map(|(path, config)| {
                config
                    .fields
                    .get("files")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(|named| (path.clone(), normalized(parent(path), named)))
                    .filter(|(_, named)| !is_above(named))
            })
            .collect()
    }

    /// A config as the library is given it: only the fields it uses, with each target of `extends` and
    /// `references` the analyzer found, written from the config's folder.
    fn served(&self, path: &str, config: &Config) -> Value {
>>>>>>> replace
```

The entry points:

```rust path=src/entries.rs
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
/// also ends in `.ts` (spec ENTRY-3).
const OUTPUT_ENDINGS: [&str; 6] = [".d.ts", ".d.mts", ".d.cts", ".js", ".mjs", ".cjs"];

/// The same endings without their dot, as a glob's braces hold them.
const OUTPUT_ALTERNATIVES: [&str; 6] = ["d.ts", "d.mts", "d.cts", "js", "mjs", "cjs"];

/// The source endings a mapped glob puts in place of an output ending (spec ENTRY-3, MAP-1).
const SOURCE_ENDINGS: &str = ".{ts,tsx,mts,cts,js,jsx,mjs,cjs}";

/// The characters that make a segment of a glob more than a plain name (spec KEY-5).
const WILDCARDS: [char; 5] = ['*', '?', '[', '{', '\\'];

/// One source that made a file an entry point (spec ENTRY-1).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Source {
    /// What named it: a field of a `package.json` (`main`, `module`, `types`, `typings`, `bin` or
    /// `exports`), a tsconfig's `files`, a page's script (`page`), or the declaration `entries`.
    from: &'static str,
    /// The file that names it: the `package.json`, the tsconfig or the page; none for a declaration.
    file: Option<String>,
    /// For a declaration, the stack that holds it.
    stack: Option<String>,
    /// For a declaration, who states it: `pack` or `project`.
    by: Option<&'static str>,
}

impl Source {
    /// A source that a file of the project names.
    fn named(from: &'static str, file: &str) -> Self {
        Self {
            from,
            file: Some(file.to_owned()),
            stack: None,
            by: None,
        }
    }

    /// The source as `graph` prints it.
    pub fn printed(&self) -> Value {
        json!({ "by": self.by, "file": self.file, "from": self.from, "stack": self.stack })
    }
}

/// Why a package's entry points are likely incomplete, so that an unused file in it is more likely a
/// missing declaration than dead code (spec ENTRY-8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Nothing makes any of its files an entry point.
    NoEntryPoint,
    /// A target of its `bin` or `exports` maps to no source.
    UnmappedTarget,
}

impl Fault {
    /// The fault's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoEntryPoint => "no-entry-point",
            Self::UnmappedTarget => "unmapped-target",
        }
    }
}

/// The entry points of a project.
#[derive(Debug, Default)]
pub struct Entries {
    /// Each entry point, by its path, with every source that made it one.
    pub points: BTreeMap<String, BTreeSet<Source>>,
    /// The packages whose entry points are likely incomplete, by folder (spec ENTRY-8).
    pub faults: BTreeMap<String, Fault>,
}

impl Entries {
    /// Records that a source makes a file an entry point.
    fn add(&mut self, path: String, source: Source) {
        self.points.entry(path).or_default().insert(source);
    }
}

/// What the entry points are read from.
pub struct Project<'p> {
    /// The file list.
    pub files: &'p [File],
    /// The packages.
    pub packages: &'p Packages,
    /// The tsconfig files.
    pub configs: &'p Configs,
    /// The resolver, which maps a target of a `package.json` back to its source.
    pub resolver: &'p Resolver<'p>,
    /// Where every import leads, by the importing file's path.
    pub resolved: &'p BTreeMap<String, Vec<Resolution>>,
    /// The hand-off file, where one was given.
    pub handed: Option<&'p HandOff>,
}

/// Finds every entry point and the source of each (spec ENTRY-1). A glob was checked when the hand-off
/// file was read, so one that cannot be read stops the run as it would have there.
pub fn find(project: &Project<'_>, notices: &mut Vec<Notice>) -> Result<Entries, Stop> {
    let mut entries = Entries::default();
    let mut unmapped = BTreeSet::new();
    let internal = internal(project.handed);

    for package in &project.packages.list {
        let manifest = manifest_path(&package.folder);

        for (field, target) in &package.starts {
            match project.resolver.package_target(package, target) {
                Some(found) => {
                    for path in found {
                        entries.add(path, Source::named(field, &manifest));
                    }
                }
                None if *field == "bin" => {
                    unmapped.insert(package.folder.clone());
                }
                None => {}
            }
        }

        if let Some(exports) = package
            .exports
            .as_ref()
            .filter(|_| !internal.contains(&package.folder))
        {
            exported(project, package, exports, &mut entries, &mut unmapped);
        }
    }

    let listed: BTreeSet<&str> = project
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect();

    for (config, named) in &project.configs.named_files {
        if listed.contains(named.as_str()) {
            entries.add(named.clone(), Source::named("files", config));
        }
    }

    for page in project.files.iter().filter(|file| file.kind == Kind::Page) {
        for resolution in project.resolved.get(&page.path).into_iter().flatten() {
            if let Outcome::Files(found) = &resolution.outcome {
                for path in found {
                    entries.add(path.clone(), Source::named("page", &page.path));
                }
            }
        }
    }

    declared(project, &mut entries, notices)?;

    for package in &project.packages.list {
        let fault = if unmapped.contains(&package.folder) {
            Some(Fault::UnmappedTarget)
        } else if entries.points.keys().all(|path| {
            project.packages.of_file.get(path).map(String::as_str) != Some(package.folder.as_str())
        }) {
            Some(Fault::NoEntryPoint)
        } else {
            None
        };

        if let Some(fault) = fault {
            entries.faults.insert(package.folder.clone(), fault);
        }
    }

    Ok(entries)
}

/// The entry points a package's `exports` offers: every target of every subpath, through every
/// condition, mapped back to its source. A pattern's `*` stands for any text, `/` included, the same
/// text wherever it is written twice, and a subpath that a key whose target is `null` matches offers
/// nothing (spec ENTRY-1, ENTRY-2).
fn exported(
    project: &Project<'_>,
    package: &Package,
    exports: &Value,
    entries: &mut Entries,
    unmapped: &mut BTreeSet<String>,
) {
    let manifest = manifest_path(&package.folder);
    let map: Map<String, Value> = match exports
        .as_object()
        .filter(|keys| keys.keys().any(|key| key.starts_with('.')))
    {
        Some(map) => map.clone(),
        None => Map::from_iter([(".".to_owned(), exports.clone())]),
    };
    let hidden =
        |subpath: &str| resolve::matched(&map, subpath).is_some_and(|(value, _)| value.is_null());
    let in_package = |path: &str| inside(folder_path(&package.folder), path).is_some();

    for (key, value) in &map {
        for target in strings(value) {
            // Node reads only a target that begins with `./` as a path in the package.
            let Some(target) = target.strip_prefix("./") else {
                continue;
            };

            if key.contains('*') && target.contains('*') {
                let places = project.resolver.pattern_places(package, target);
                let mut found = false;

                for file in project.files.iter().filter(|file| in_package(&file.path)) {
                    let Some(star) = places.iter().find_map(|place| filled(place, &file.path))
                    else {
                        continue;
                    };

                    found = true;

                    if !hidden(&key.replacen('*', star, 1)) {
                        entries.add(file.path.clone(), Source::named("exports", &manifest));
                    }
                }

                if !found {
                    unmapped.insert(package.folder.clone());
                }
            } else {
                match project.resolver.package_target(package, target) {
                    Some(found) => {
                        for path in found {
                            entries.add(path, Source::named("exports", &manifest));
                        }
                    }
                    None => {
                        unmapped.insert(package.folder.clone());
                    }
                }
            }
        }
    }
}

/// Every path a target of `exports` holds, through every condition and every entry of a list.
fn strings(value: &Value) -> Vec<&str> {
    match value {
        Value::String(target) => vec![target.as_str()],
        Value::Array(list) => list.iter().flat_map(strings).collect(),
        Value::Object(conditions) => conditions.values().flat_map(strings).collect(),
        _ => Vec::new(),
    }
}

/// The text a pattern's `*` stands for in a path, where the pattern matches it: any text that is not
/// empty, `/` included, the same text where the pattern writes `*` twice (spec ENTRY-2).
fn filled<'p>(pattern: &str, path: &'p str) -> Option<&'p str> {
    let parts: Vec<&str> = pattern.split('*').collect();

    match parts.as_slice() {
        [before, after] => {
            let star = path.strip_prefix(before)?.strip_suffix(after)?;

            (!star.is_empty()).then_some(star)
        }
        [before, middle, after] => {
            let rest = path.strip_prefix(before)?.strip_suffix(after)?;
            let twice = rest.len().checked_sub(middle.len())?;

            if twice == 0 || twice % 2 != 0 {
                return None;
            }

            let (star, tail) = rest.split_at_checked(twice / 2)?;

            (tail.strip_prefix(middle) == Some(star)).then_some(star)
        }
        _ => None,
    }
}

/// The package folders the `internal` declarations name: a pack's from each folder yak's detection
/// looked in, a project's from the root (spec ENTRY-5).
fn internal(handed: Option<&HandOff>) -> BTreeSet<String> {
    let mut folders = BTreeSet::new();
    let Some(handed) = handed else {
        return folders;
    };

    for stack in &handed.stacks {
        for (declarations, bases) in [
            (&stack.pack, handed.folders.as_slice()),
            (&stack.project, &[".".to_owned()][..]),
        ] {
            for path in texts(declarations.get("internal")) {
                for base in bases {
                    folders.insert(folder_name(&join(&[base, path])));
                }
            }
        }
    }

    folders
}

/// The entry points the `entries` declarations name: a pack's globs from each folder yak's detection
/// looked in, a project's from the root. A glob that matches no file in a folder is mapped back from
/// build output there; one that matches nothing either way gives a notice (spec ENTRY-1, ENTRY-3,
/// ENTRY-4).
fn declared(
    project: &Project<'_>,
    entries: &mut Entries,
    notices: &mut Vec<Notice>,
) -> Result<(), Stop> {
    let Some(handed) = project.handed else {
        return Ok(());
    };

    for stack in &handed.stacks {
        for (by, declarations, bases) in [
            ("pack", &stack.pack, handed.folders.as_slice()),
            ("project", &stack.project, &[".".to_owned()][..]),
        ] {
            for written in texts(declarations.get("entries")) {
                let glob = Glob::new(written).map_err(Stop::new)?;
                let source = Source {
                    from: "entries",
                    file: None,
                    stack: Some(stack.name.clone()),
                    by: Some(by),
                };
                let mut matched = false;

                for base in bases {
                    let base = folder_path(base);
                    let mut found: Vec<&str> = project
                        .files
                        .iter()
                        .map(|file| file.path.as_str())
                        .filter(|path| inside(base, path).is_some_and(|rest| glob.matches(rest)))
                        .collect();

                    if found.is_empty() {
                        let mapped: Vec<Glob> = mapped_globs(written, base, project)
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

                    matched |= !found.is_empty();

                    for path in found {
                        entries.add(path.to_owned(), source.clone());
                    }
                }

                if !matched {
                    notices.push(Notice::new(
                        "glob-matches-nothing",
                        None,
                        format!(
                            "The `entries` glob `{written}` that the {by} states for the stack `{}` matches no \
                             file, nor any source mapped back from build output.",
                            stack.name
                        ),
                    ));
                }
            }
        }
    }

    Ok(())
}

/// An `entries` glob that names build output, mapped back to the globs of its source, each from the
/// root (spec ENTRY-3). The glob's folder, its leading segments that hold no wildcard, is rewritten as
/// MAP-1 rewrites a path: from each `outDir` of a tsconfig in the folder's package that holds it, to
/// that config's sources; then with its first folder under the package replaced by `src`. An output
/// ending of its last segment, alone or among braces, becomes the source endings. A glob names build
/// output only by such an ending, so one without it is not mapped, and neither is one whose first
/// segment holds a wildcard.
fn mapped_globs(written: &str, base: &str, project: &Project<'_>) -> Vec<String> {
    let segments: Vec<&str> = written.split('/').collect();
    let literal = segments
        .iter()
        .take(segments.len() - 1)
        .take_while(|segment| !segment.contains(WILDCARDS))
        .count();

    let Some(rest) = source_endings(&segments[literal..].join("/")).filter(|_| literal > 0) else {
        return Vec::new();
    };
    let folder = join(&[base, &segments[..literal].join("/")]);
    let package = package_folder(&folder, project.packages);
    let mut globs = Vec::new();

    for build in project.configs.builds(package) {
        if let Some(within) = inside(&build.out_dir, &folder) {
            globs.extend(
                build
                    .sources
                    .iter()
                    .map(|source| join(&[source, within, &rest])),
            );
        }
    }

    if let Some((_, below)) = inside(package, &folder).and_then(|under| {
        under
            .split_once('/')
            .or_else(|| (!under.is_empty()).then_some((under, "")))
    }) {
        globs.push(join(&[package, "src", below, &rest]));
    }

    globs
}

/// The end of a glob with its output ending made the source endings, where its last segment ends in
/// one, alone or among braces; nothing where it does not.
fn source_endings(rest: &str) -> Option<String> {
    if let Some(stem) = OUTPUT_ENDINGS
        .iter()
        .find_map(|ending| rest.strip_suffix(ending))
    {
        return Some(format!("{stem}{SOURCE_ENDINGS}"));
    }

    if let Some((stem, braces)) = rest.rsplit_once(".{")
        && let Some(alternatives) = braces.strip_suffix('}')
        && !stem.ends_with('/')
        && alternatives
            .split(',')
            .all(|alternative| OUTPUT_ALTERNATIVES.contains(&alternative))
    {
        return Some(format!("{stem}{SOURCE_ENDINGS}"));
    }

    None
}

/// The folder, from the root, of the package a folder lies in: the deepest package at or above it, or
/// the root's, the empty path, where it lies in none.
fn package_folder<'a>(folder: &str, packages: &'a Packages) -> &'a str {
    packages
        .list
        .iter()
        .map(|package| folder_path(&package.folder))
        .filter(|candidate| inside(candidate, folder).is_some())
        .max_by_key(|candidate| candidate.len())
        .unwrap_or("")
}

/// The part of a path below a folder, the empty text for the folder itself; nothing when the path does
/// not lie at or under it. The root is the empty path.
fn inside<'a>(folder: &str, path: &'a str) -> Option<&'a str> {
    if folder.is_empty() {
        Some(path)
    } else if path == folder {
        Some("")
    } else {
        path.strip_prefix(folder)?.strip_prefix('/')
    }
}

/// Parts of a path joined with `/`, the empty ones and `.` passed over.
fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .flat_map(|part| part.split('/'))
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

/// A folder from the root as a package names its folder: `.` for the root.
fn folder_name(path: &str) -> String {
    if path.is_empty() {
        ".".to_owned()
    } else {
        path.to_owned()
    }
}

/// A package's folder as a path from the root: the root's own is the empty path.
fn folder_path(folder: &str) -> &str {
    if folder == "." { "" } else { folder }
}

/// The path of the `package.json` in a package's folder.
fn manifest_path(folder: &str) -> String {
    join(&[folder, "package.json"])
}

/// The texts a declaration holds. The declaration was checked when the hand-off file was read, so
/// anything else holds none.
fn texts(declaration: Option<&Value>) -> Vec<&str> {
    declaration
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}
```

`graph` finds them after resolving, and prints them:

```rust edit path=src/graph.rs
<<<<<<< find
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports of every script, and where each import leads.

use std::collections::BTreeMap;
use std::env;
=======
//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports of every script, where each import leads, and the entry
//! points with the source of each (spec CMD-4).

use std::collections::BTreeMap;
use std::env;
>>>>>>> replace
<<<<<<< find
use serde_json::{Value, json};

use crate::declarations;
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::Import;
=======
use serde_json::{Value, json};

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::Import;
>>>>>>> replace
<<<<<<< find
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolved = Resolver::new(&listing, &packages, &configs).resolve(&root, &scripts)?;
    let mut document = json!({
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
=======
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed: handed.as_ref(),
        },
        &mut notices,
    )?;
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
            .faults
            .iter()
            .map(|(folder, fault)| json!({ "fault": fault.name(), "package": folder }))
            .collect::<Vec<Value>>(),
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
>>>>>>> replace
<<<<<<< find
    Ok(format!("{document:#}"))
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, and the
/// count of those that cannot be followed.
fn file(
=======
    Ok(format!("{document:#}"))
}

/// The entry points as they are printed, by path, each with every source that made it one (spec
/// CMD-4).
fn entry_points(entries: &Entries) -> Vec<Value> {
    entries
        .points
        .iter()
        .map(|(path, sources)| {
            json!({
                "path": path,
                "sources": sources.iter().map(entries::Source::printed).collect::<Vec<Value>>(),
            })
        })
        .collect()
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, and the
/// count of those that cannot be followed.
fn file(
>>>>>>> replace
```

```rust edit path=src/main.rs
<<<<<<< find

mod cli;
mod declarations;
mod files;
mod git;
mod glob;
=======

mod cli;
mod declarations;
mod entries;
mod files;
mod git;
mod glob;
>>>>>>> replace
```

- [ ] **Step 4: Run the tests and see them pass**

```bash
cargo test --release --locked --test entries --test hand-off
```

Expected: PASS, 6 tests in `entries` and 10 in `hand-off`.

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
git add src/entries.rs src/packages.rs src/resolve.rs src/tsconfig.rs src/graph.rs src/main.rs Cargo.toml tests/entries.test.rs tests/hand-off.test.rs
git commit -m "feat(entries): find entry points from standard files and declarations"
```

Expected: one commit on `feat/dead-code-findings`.
