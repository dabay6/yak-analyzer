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
