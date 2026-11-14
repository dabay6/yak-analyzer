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
pub fn matched<'v, 'w>(
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
