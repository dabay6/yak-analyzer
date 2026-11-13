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
