//! The file list: every file under the root that git tracks, and every file there that is untracked and
//! that git does not ignore (spec LIST-1 to LIST-4, LIST-6, LIST-7, LIST-10, LIST-11).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::git;
use crate::notice::Notice;
use crate::stop::Stop;

/// The command that lists everything in one start of git: tracked files with their modes, untracked
/// files that nothing ignores, and tracked files missing from the working tree, each name ended by a
/// zero byte.
const LIST: [&str; 8] = [
    "ls-files",
    "-z",
    "-t",
    "--stage",
    "--cached",
    "--others",
    "--exclude-standard",
    "--deleted",
];

/// The mode git records for a symbolic link.
const LINK_MODE: &str = "120000";

/// The mode git records for a submodule.
const SUBMODULE_MODE: &str = "160000";

/// The endings of a type declaration file. They are asked before a script's, which they end in.
const DECLARATION_ENDINGS: [&str; 3] = [".d.ts", ".d.mts", ".d.cts"];

/// The endings of a script.
const SCRIPT_ENDINGS: [&str; 8] = [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"];

/// What kind of file a listed file is, from its name (spec LIST-6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A script in TypeScript or JavaScript.
    Script,
    /// A type declaration file.
    Declaration,
    /// A Vue component.
    Vue,
    /// An MDX document.
    Mdx,
    /// An HTML page.
    Page,
    /// A `package.json`.
    Manifest,
    /// A tsconfig or a `jsconfig.json`.
    Tsconfig,
    /// Any other file, which is never read.
    Other,
}

impl Kind {
    /// The kind of the file with the path given.
    fn of(path: &str) -> Self {
        let name = path.rsplit('/').next().unwrap_or(path);

        if DECLARATION_ENDINGS
            .iter()
            .any(|ending| name.ends_with(ending))
        {
            Self::Declaration
        } else if SCRIPT_ENDINGS.iter().any(|ending| name.ends_with(ending)) {
            Self::Script
        } else if name.ends_with(".vue") {
            Self::Vue
        } else if name.ends_with(".mdx") {
            Self::Mdx
        } else if name.ends_with(".html") {
            Self::Page
        } else if name == "package.json" {
            Self::Manifest
        } else if name == "jsconfig.json"
            || (name.starts_with("tsconfig") && name.ends_with(".json"))
        {
            Self::Tsconfig
        } else {
            Self::Other
        }
    }

    /// The kind's name in what `graph` prints.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Declaration => "declaration",
            Self::Vue => "vue",
            Self::Mdx => "mdx",
            Self::Page => "page",
            Self::Manifest => "manifest",
            Self::Tsconfig => "tsconfig",
            Self::Other => "other",
        }
    }
}

/// One file of the list.
#[derive(Debug)]
pub struct File {
    /// Its path, relative to the root, with forward slashes.
    pub path: String,
    /// Its kind.
    pub kind: Kind,
    /// Whether git tracks it.
    pub tracked: bool,
}

/// Stops the run when the root is a sub-folder that git holds files under by another spelling of its
/// path, which would make git list every file under it as untracked (spec LIST-11).
fn root_spelling(root: &Path) -> Result<(), Stop> {
    // A root that holds a `.git` of its own is the top of its work tree: there is nothing to find.
    if fs::symlink_metadata(root.join(".git")).is_ok() {
        return Ok(());
    }

    let answer = git::read(root, &["rev-parse", "--show-toplevel", "--show-prefix"])?;
    let answer = String::from_utf8_lossy(&answer.output);
    let mut lines = answer.lines();
    let (Some(top), Some(prefix)) = (lines.next(), lines.next()) else {
        return Ok(());
    };
    let pathspec = format!(":(top,icase){prefix}");
    let held = git::read(
        root,
        &["ls-files", "-z", "--cached", "--full-name", "--", &pathspec],
    )?;
    let depth = prefix.trim_end_matches('/').split('/').count();
    let spellings: BTreeSet<String> = held
        .output
        .split(|byte| *byte == 0)
        .filter_map(|path| std::str::from_utf8(path).ok())
        .filter(|path| !path.starts_with(prefix))
        .map(|path| path.split('/').take(depth).collect::<Vec<_>>().join("/"))
        .filter(|spelling| lowered(&format!("{spelling}/")) == lowered(prefix))
        .collect();

    for spelling in spellings {
        if one_folder(root, &Path::new(top).join(&spelling)) {
            return Err(Stop::new(format!(
                "the root {} is spelt `{}` on disk, and git holds the files under it as `{spelling}`; rename the \
                 folder on disk to git's spelling, or record the disk's with `git mv`, and run again",
                root.display(),
                prefix.trim_end_matches('/')
            )));
        }
    }

    Ok(())
}

/// Whether two paths are one folder on disk, by their identities (spec LIST-11).
fn one_folder(first: &Path, second: &Path) -> bool {
    let folder =
        |path: &Path| fs::symlink_metadata(path).is_ok_and(|facts| facts.file_type().is_dir());

    folder(first) && folder(second) && same_file::is_same_file(first, second).unwrap_or(false)
}

/// A path with each character put in lower case by itself, so that no letter's mapping depends on its
/// neighbours (spec LIST-10).
fn lowered(path: &str) -> String {
    path.chars().flat_map(char::to_lowercase).collect()
}

/// Whether two paths are one file on disk (spec LIST-10). Both are looked at first without following a
/// link, and are compared only when both are ordinary files, so no link is followed and no special file
/// is opened. When either cannot be looked at or opened, they are not taken for one file.
fn one_file(first: &Path, second: &Path) -> bool {
    let ordinary =
        |path: &Path| fs::symlink_metadata(path).is_ok_and(|facts| facts.file_type().is_file());

    ordinary(first) && ordinary(second) && same_file::is_same_file(first, second).unwrap_or(false)
}

/// What git records a tracked path as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    /// A file, which is read.
    File,
    /// A symbolic link, which is never followed or read, but is on disk.
    Link,
    /// A submodule, which is never entered.
    Submodule,
}

/// Asks git for the file list of the root. The files come back sorted by path, byte by byte; what was
/// left out on the way is added to the notices.
pub fn list(root: &Path, notices: &mut Vec<Notice>) -> Result<Vec<File>, Stop> {
    root_spelling(root)?;

    let answer = git::read(root, &LIST)?;

    if let Some(warning) = answer.warning {
        notices.push(Notice::new(
            "git-warning",
            None,
            format!("git warned while it listed the files: {warning}"),
        ));
    }

    // The tracked names with what each is, the untracked names, and the tracked names that are missing
    // from the working tree.
    let mut tracked: BTreeMap<String, Entry> = BTreeMap::new();
    let mut untracked: BTreeSet<String> = BTreeSet::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let mut links = 0_usize;
    let mut unreadable_names = 0_usize;

    for record in answer
        .output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let Ok(record) = std::str::from_utf8(record) else {
            unreadable_names += 1;

            continue;
        };

        if let Some(name) = record.strip_prefix("? ") {
            // A name that ends in a slash is a nested repository, which is never entered (spec LIST-3).
            if !name.ends_with('/') {
                untracked.insert(name.to_owned());
            }

            continue;
        }

        let entry = record.split_once('\t').and_then(|(facts, name)| {
            let mut facts = facts.split(' ');

            Some((facts.next()?, facts.next()?, name))
        });
        let Some((letter, mode, name)) = entry else {
            return Err(Stop::new(format!(
                "git listed an entry the analyzer cannot read, `{record}`; this is a fault in the analyzer, \
                 so report it with the git version"
            )));
        };

        // `R` marks a tracked file missing from the working tree, and `S` one a sparse checkout left out.
        if matches!(letter, "R" | "S") {
            missing.insert(name.to_owned());
        }

        if mode == LINK_MODE {
            links += usize::from(tracked.get(name) != Some(&Entry::Link));
            tracked.insert(name.to_owned(), Entry::Link);
        } else {
            tracked
                .entry(name.to_owned())
                .or_insert(if mode == SUBMODULE_MODE {
                    Entry::Submodule
                } else {
                    Entry::File
                });
        }
    }

    // A symbolic link is never followed and never read, tracked or not (spec LIST-3).
    untracked.retain(|name| {
        let link =
            fs::symlink_metadata(root.join(name)).is_ok_and(|facts| facts.file_type().is_symlink());

        links += usize::from(link);

        !link
    });

    // An untracked path that is the same file on disk as a tracked file in the list is that file under
    // the disk's spelling, and is left out (spec LIST-10). The tracked files whose paths differ from it
    // in letter case alone are the candidates; the files' identities decide.
    let mut candidates: BTreeMap<String, Vec<&String>> = BTreeMap::new();

    for (name, entry) in &tracked {
        if *entry == Entry::File && !missing.contains(name) {
            candidates.entry(lowered(name)).or_default().push(name);
        }
    }

    let mut pairs: Vec<(String, String)> = Vec::new();

    untracked.retain(|name| {
        let held = candidates.get(&lowered(name)).and_then(|held| {
            held.iter()
                .find(|held| one_file(&root.join(name), &root.join(held)))
        });

        match held {
            Some(held) => {
                pairs.push(((*held).clone(), name.clone()));

                false
            }
            None => true,
        }
    });

    if let Some((held, other)) = pairs.first() {
        notices.push(Notice::new(
            "paths-differ-in-case",
            Some(held.clone()),
            format!(
                "{} untracked path(s) were left out of the file list because each is a tracked file under \
                 another spelling; the first is `{other}`, which git holds as `{held}`. A rename git was not \
                 told of, or an index that spells a folder two ways, causes this; the spellings agree again \
                 once the file or folder is renamed on disk to git's spelling, or git is given the disk's \
                 with `git mv`.",
                pairs.len()
            ),
        ));
    }

    if links > 0 {
        notices.push(Notice::new(
            "links-left-out",
            None,
            format!("{links} symbolic link(s) were left out of the file list; a link is never followed or read."),
        ));
    }

    if unreadable_names > 0 {
        notices.push(Notice::new(
            "names-left-out",
            None,
            format!("{unreadable_names} file(s) whose names are not valid UTF-8 were left out of the file list."),
        ));
    }

    let mut files: Vec<File> = tracked
        .into_iter()
        .filter(|(name, entry)| *entry == Entry::File && !missing.contains(name))
        .map(|(path, _)| (path, true))
        .chain(untracked.into_iter().map(|path| (path, false)))
        .map(|(path, tracked)| File {
            kind: Kind::of(&path),
            path,
            tracked,
        })
        .collect();

    files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));

    Ok(files)
}
