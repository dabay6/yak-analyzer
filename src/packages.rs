//! Packages: the folders that hold a `package.json`, which of them the workspace names, and which
//! package each file belongs to (spec PACK-1 to PACK-6).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};
use yaml_rust2::{Yaml, YamlLoader};

use crate::files::{File, Kind};
use crate::glob::Glob;
use crate::notice::Notice;
use crate::stop::Stop;

/// The workspace's list that pnpm reads, at the root.
const PNPM_LIST: &str = "pnpm-workspace.yaml";

/// One package.
#[derive(Debug)]
pub struct Package {
    /// Its folder, relative to the root; `.` for the root itself.
    pub folder: String,
    /// The `name` its `package.json` states, where it states one.
    pub name: Option<String>,
    /// Whether the workspace names it, so an import by its name leads to it (spec PACK-3).
    pub workspace: bool,
    /// Whether its folder is one the design spec reports the entry-point kinds in (spec PACK-4).
    pub detected: bool,
    /// The packages its four lists of dependencies name, each with the version it asks for.
    pub dependencies: BTreeMap<String, String>,
}

/// The packages of a project, and which package each file belongs to.
#[derive(Debug)]
pub struct Packages {
    /// The packages, by folder, byte by byte.
    pub list: Vec<Package>,
    /// The folder of the package each file belongs to, by the file's path; a file with no package
    /// above it is not here.
    pub of_file: BTreeMap<String, String>,
}

/// Reads the packages of the root. `data` is the files declared as data, whose `package.json` makes
/// no package (spec KEY-4); `detection` is the folders yak's detection looked in, with a hand-off file.
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    detection: Option<&[String]>,
    notices: &mut Vec<Notice>,
) -> Result<Packages, Stop> {
    let manifests: Vec<&File> = files
        .iter()
        .filter(|file| file.kind == Kind::Manifest && !data.contains(&file.path))
        .collect();

    // Each `package.json` is read first, then the workspace's list, then the fields, so that two faults
    // always give the same line (spec RUN-4).
    let mut read: Vec<(String, Map<String, Value>)> = Vec::with_capacity(manifests.len());

    for manifest in &manifests {
        read.push((folder_of(&manifest.path), object(root, &manifest.path)?));
    }

    let pnpm = if files.iter().any(|file| file.path == PNPM_LIST) && !data.contains(PNPM_LIST) {
        Some(yaml(root)?)
    } else {
        None
    };
    let mut names: BTreeMap<&str, (Option<String>, BTreeMap<String, String>)> = BTreeMap::new();

    for (folder, fields) in &read {
        let file = manifest_path(folder);

        names.insert(folder, (name(fields, &file)?, dependencies(fields, &file)?));
    }

    let mut patterns = Vec::new();

    if let Some((_, fields)) = read.iter().find(|(folder, _)| folder == ".") {
        patterns.extend(workspaces(fields)?);
    }

    if let Some(pnpm) = &pnpm {
        patterns.extend(pnpm_packages(pnpm)?);
    }

    let (included, excluded) = compile(&patterns)?;
    let detection: Option<BTreeSet<&str>> =
        detection.map(|folders| folders.iter().map(String::as_str).collect());
    let mut list: Vec<Package> = names
        .into_iter()
        .map(|(folder, (name, dependencies))| {
            let listed = folder != "."
                && included.iter().any(|glob| glob.matches(folder))
                && !excluded.iter().any(|glob| glob.matches(folder));
            let workspace = folder == "."
                || listed
                || detection
                    .as_ref()
                    .is_some_and(|found| found.contains(folder));
            let detected = match &detection {
                Some(found) => found.contains(folder),
                None => folder == "." || listed,
            };

            Package {
                folder: folder.to_owned(),
                name,
                workspace,
                detected,
                dependencies,
            }
        })
        .collect();

    list.sort_by(|a, b| a.folder.as_bytes().cmp(b.folder.as_bytes()));
    repeated_names(&list, notices);

    let folders: BTreeSet<&str> = list.iter().map(|package| package.folder.as_str()).collect();
    let of_file = files
        .iter()
        .filter_map(|file| nearest(&file.path, &folders).map(|folder| (file.path.clone(), folder)))
        .collect();

    Ok(Packages { list, of_file })
}

/// The folder a `package.json` lies in, `.` for the root.
fn folder_of(path: &str) -> String {
    path.rsplit_once('/')
        .map_or_else(|| ".".to_owned(), |(folder, _)| folder.to_owned())
}

/// The path of the `package.json` in a folder.
fn manifest_path(folder: &str) -> String {
    if folder == "." {
        "package.json".to_owned()
    } else {
        format!("{folder}/package.json")
    }
}

/// The content of a file, with a byte-order mark at its start passed over.
fn text(root: &Path, path: &str) -> Result<String, String> {
    let bytes = fs::read(root.join(path)).map_err(|error| error.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);

    String::from_utf8(bytes.to_vec()).map_err(|_| "it is not valid UTF-8".to_owned())
}

/// A `package.json` read as a JSON object (spec PACK-2).
fn object(root: &Path, path: &str) -> Result<Map<String, Value>, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the package.json {path} cannot be read as a JSON object ({why}); fix it, or declare it as data \
             if it is broken on purpose"
        ))
    };
    let content = text(root, path).map_err(refused)?;

    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(fields)) => Ok(fields),
        Ok(_) => Err(refused(
            "it holds something other than an object".to_owned(),
        )),
        Err(error) => Err(refused(format!(
            "line {}, column {}",
            error.line(),
            error.column()
        ))),
    }
}

/// The workspace's list that pnpm reads, as YAML (spec PACK-3).
fn yaml(root: &Path) -> Result<Yaml, Stop> {
    let refused = |why: String| {
        Stop::new(format!(
            "the workspace's list {PNPM_LIST} cannot be read ({why}); fix it, or declare it as data"
        ))
    };
    let content = text(root, PNPM_LIST).map_err(refused)?;
    let documents =
        YamlLoader::load_from_str(&content).map_err(|error| refused(error.to_string()))?;

    Ok(documents.into_iter().next().unwrap_or(Yaml::Null))
}

/// The stop for a field that holds a value of a form it does not have (spec PACK-6).
fn misshapen(field: &str, file: &str, form: &str) -> Stop {
    Stop::new(format!(
        "the field `{field}` in {file} holds a value of a form it does not have; it must be {form}; fix it"
    ))
}

/// The `name` a `package.json` states.
fn name(fields: &Map<String, Value>, file: &str) -> Result<Option<String>, Stop> {
    match fields.get("name") {
        None => Ok(None),
        Some(Value::String(name)) => Ok(Some(name.clone())),
        Some(_) => Err(misshapen("name", file, "text")),
    }
}

/// The packages a `package.json`'s four lists of dependencies name, each with the version it asks for.
fn dependencies(fields: &Map<String, Value>, file: &str) -> Result<BTreeMap<String, String>, Stop> {
    let mut named = BTreeMap::new();

    for list in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let form = "an object that maps each package's name to the version it asks for";
        let entries = match fields.get(list) {
            None => continue,
            Some(Value::Object(entries)) => entries,
            Some(_) => return Err(misshapen(list, file, form)),
        };

        for (package, version) in entries {
            let version = version
                .as_str()
                .ok_or_else(|| misshapen(list, file, form))?;

            named
                .entry(package.clone())
                .or_insert_with(|| version.to_owned());
        }
    }

    Ok(named)
}

/// The globs of the root `package.json`'s `workspaces`: a list, or an object with `packages`.
fn workspaces(fields: &Map<String, Value>) -> Result<Vec<String>, Stop> {
    let form = "a list of globs, or an object whose `packages` is one";
    let list = match fields.get("workspaces") {
        None => return Ok(Vec::new()),
        Some(Value::Array(list)) => list,
        Some(Value::Object(object)) => match object.get("packages") {
            None => return Ok(Vec::new()),
            Some(Value::Array(list)) => list,
            Some(_) => return Err(misshapen("workspaces", "package.json", form)),
        },
        Some(_) => return Err(misshapen("workspaces", "package.json", form)),
    };

    list.iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| misshapen("workspaces", "package.json", form))
        })
        .collect()
}

/// The globs of pnpm's `packages`.
fn pnpm_packages(document: &Yaml) -> Result<Vec<String>, Stop> {
    let form = "a list of globs";
    let packages = match document {
        Yaml::Null => return Ok(Vec::new()),
        Yaml::Hash(_) => &document["packages"],
        _ => {
            return Err(misshapen(
                "packages",
                PNPM_LIST,
                "a mapping that holds `packages`",
            ));
        }
    };

    match packages {
        Yaml::BadValue | Yaml::Null => Ok(Vec::new()),
        Yaml::Array(list) => list
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| misshapen("packages", PNPM_LIST, form))
            })
            .collect(),
        _ => Err(misshapen("packages", PNPM_LIST, form)),
    }
}

/// The workspace's globs, read: those that include folders, and those, written with `!`, that take
/// folders out again. A leading `./` and a trailing `/` say nothing and are passed over.
fn compile(patterns: &[String]) -> Result<(Vec<Glob>, Vec<Glob>), Stop> {
    let mut included = Vec::new();
    let mut excluded = Vec::new();

    for pattern in patterns {
        let (negated, written) = pattern
            .strip_prefix('!')
            .map_or((false, pattern.as_str()), |rest| (true, rest));
        let written = written
            .strip_prefix("./")
            .unwrap_or(written)
            .trim_end_matches('/');
        let glob = Glob::new(written).map_err(|why| {
            Stop::new(format!(
                "a glob of the workspace's list cannot be read: {why}; fix it in package.json or {PNPM_LIST}"
            ))
        })?;

        if negated {
            excluded.push(glob);
        } else {
            included.push(glob);
        }
    }

    Ok((included, excluded))
}

/// A notice for each workspace package whose name an earlier one already has (spec PACK-5).
fn repeated_names(list: &[Package], notices: &mut Vec<Notice>) {
    let mut first: BTreeMap<&str, &str> = BTreeMap::new();

    for package in list.iter().filter(|package| package.workspace) {
        let Some(name) = &package.name else {
            continue;
        };

        match first.get(name.as_str()) {
            Some(earlier) => notices.push(Notice::new(
                "package-name-repeated",
                Some(manifest_path(&package.folder)),
                format!(
                    "The workspace packages in `{earlier}` and `{}` are both named `{name}`; an import of `{name}` \
                     leads to the first, `{earlier}`.",
                    package.folder
                ),
            )),
            None => {
                first.insert(name, &package.folder);
            }
        }
    }
}

/// The folder of the nearest package at or above a file, searching no higher than the root (spec
/// PACK-1, LIST-8).
fn nearest(path: &str, folders: &BTreeSet<&str>) -> Option<String> {
    let mut folder = path;

    while let Some((above, _)) = folder.rsplit_once('/') {
        if folders.contains(above) {
            return Some(above.to_owned());
        }

        folder = above;
    }

    folders.contains(".").then(|| ".".to_owned())
}
