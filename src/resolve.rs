//! Where each import leads (spec RES-1 to RES-4, RES-8, RES-10, LIST-5).
//!
//! A path is looked for in the file list, never on disk, with exact letter case. What leads to no file
//! is asked about once, with one start of git for all of them, to tell an ignored path from a missing
//! one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::files::Listing;
use crate::git;
use crate::imports::Import;
use crate::packages::Packages;
use crate::scripts::Scripts;
use crate::stop::Stop;

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
    /// A path git ignores, which is never a fault.
    Ignored(String),
    /// Nothing.
    Unresolved,
}

/// Which rule decided an import's outcome: a step of RES-2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Step 1: a relative specifier.
    Relative,
    /// Step 3: a name in the package's `imports`.
    Imports,
    /// Step 6: anything else.
    Outside,
}

impl Rule {
    /// The rule's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Relative => "relative",
            Self::Imports => "imports",
            Self::Outside => "outside",
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

/// What looking for a path found.
enum Found {
    /// An outcome that needs nothing more.
    Done(Outcome),
    /// No file: the paths to ask git about, the path as written first (spec RES-4).
    Missing(Vec<String>),
}

/// The project as the resolver needs it.
pub struct Resolver<'p> {
    /// The paths of the file list.
    files: BTreeSet<&'p str>,
    /// What the file list leaves out that an import can still lead into.
    listing: &'p Listing,
    /// The packages, for the dependencies they list.
    packages: &'p Packages,
}

impl<'p> Resolver<'p> {
    /// A resolver over the file list and the packages.
    pub fn new(listing: &'p Listing, packages: &'p Packages) -> Self {
        Self {
            files: listing
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect(),
            listing,
            packages,
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
                    .collect(),
            ));
        }

        // One start of git answers for every path that led to no file.
        let asked: BTreeSet<&str> = found
            .iter()
            .flat_map(|(_, imports)| imports)
            .filter_map(|(found, _)| match found {
                Found::Missing(paths) => Some(paths.iter().map(String::as_str)),
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
                                .find(|path| ignored.contains(path))
                                .map_or(Outcome::Unresolved, Outcome::Ignored),
                        };

                        Resolution { outcome, rule }
                    })
                    .collect();

                (path.to_owned(), resolutions)
            })
            .collect())
    }

    /// Where one import leads, before git is asked about what led to no file (spec RES-2).
    fn one(&self, importing: &str, import: &Import) -> (Found, Rule) {
        let specifier = written_path(&import.specifier);

        if is_relative(specifier) {
            let folder = importing.rsplit_once('/').map_or("", |(folder, _)| folder);

            return match joined(folder, specifier) {
                Some(path) => (self.locate(&path), Rule::Relative),
                None => (
                    Found::Done(Outcome::Outside(Place::AboveRoot)),
                    Rule::Relative,
                ),
            };
        }

        if specifier.starts_with('#') {
            return (Found::Done(Outcome::Unresolved), Rule::Imports);
        }

        let place = if specifier.starts_with("node:") || BUILTINS.contains(&specifier) {
            Place::Builtin
        } else if is_absolute(specifier) || is_url(specifier) {
            Place::Elsewhere
        } else {
            let name = package_name(specifier);

            Place::Package {
                listed: self.listed(importing, &name),
                name,
            }
        };

        (Found::Done(Outcome::Outside(place)), Rule::Outside)
    }

    /// Whether a `package.json` at or above the importing file lists the package (spec RES-8).
    fn listed(&self, importing: &str, name: &str) -> bool {
        self.packages.list.iter().any(|package| {
            let above = package.folder == "."
                || importing
                    .strip_prefix(package.folder.as_str())
                    .is_some_and(|rest| rest.starts_with('/'));

            above && package.dependencies.contains_key(name)
        })
    }

    /// Looks for a path, relative to the root, as TypeScript finds one, in the file list (spec RES-3).
    fn locate(&self, path: &str) -> Found {
        if under(path, &self.listing.elsewhere) {
            return Found::Done(Outcome::Outside(Place::OtherRepository));
        }

        if under(path, &self.listing.links) {
            return Found::Done(Outcome::Outside(Place::Link));
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

        let index = if path.is_empty() {
            "index".to_owned()
        } else {
            format!("{path}/index")
        };

        candidates.extend(
            ENDINGS
                .iter()
                .map(|ending| (format!("{index}{ending}"), is_source(ending))),
        );

        match candidates
            .iter()
            .find(|(candidate, _)| self.files.contains(candidate.as_str()))
        {
            Some((found, _)) => Found::Done(Outcome::Files(self.with_twin(found))),
            // The path as written first, so that it is the one named when git ignores it.
            None => Found::Missing(
                std::iter::once(path.to_owned())
                    .chain(
                        candidates
                            .into_iter()
                            .filter(|(candidate, asked)| *asked && candidate != path)
                            .map(|(candidate, _)| candidate),
                    )
                    .filter(|candidate| !candidate.is_empty())
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

/// Whether an ending is a source form, which git is asked about when nothing is found: a TypeScript
/// ending or a type declaration's (spec RES-4).
fn is_source(ending: &str) -> bool {
    [".ts", ".tsx", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts"].contains(&ending)
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
