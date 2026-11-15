//! Reading a project: its hand-off file, its files, its packages, its scripts' imports and where they
//! lead, its entry points, and which export of which file is used. `graph` prints a reading, and
//! `audit` reports what one shows (spec OUT-1, CMD-1, CMD-4).
//!
//! The working tree and a base commit are read the same way, each from its own tree and file list
//! (dead-code spec COMMIT-1).

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File, Listing};
use crate::hand_off::{self, HandOff};
use crate::marks::{self, Marks};
use crate::notice::Notice;
use crate::packages::{self, Packages};
use crate::resolve::{Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tree::Tree;
use crate::tsconfig;
use crate::uses::{self, Uses};

/// What reading a project found.
pub struct Reading {
    /// Whether a hand-off file was given.
    pub handed: bool,
    /// The files of the list, in its order.
    pub files: Vec<File>,
    /// Which files the declarations mark as data or as generated.
    pub marks: Marks,
    /// The packages, and which package each file belongs to.
    pub packages: Packages,
    /// What each file that was read holds.
    pub scripts: Scripts,
    /// Where each import of each file leads, by the file's path, in the order of its imports.
    pub resolved: BTreeMap<String, Vec<Resolution>>,
    /// The entry points, each with every source that made it one.
    pub entries: Entries,
    /// Every use of an export, and the files the entry points reach.
    pub uses: Uses,
    /// What a person should know about the run, in the order it was found.
    pub notices: Vec<Notice>,
}

/// What every reading of a run starts from: the hand-off file, the root, and the notices so far.
pub struct Start {
    /// The hand-off file, where one was given.
    pub handed: Option<HandOff>,
    /// The root.
    pub root: PathBuf,
    /// What a person should know about the run so far.
    pub notices: Vec<Notice>,
}

/// Reads the hand-off file at the path given, or notes that there is none, and finds the root (spec
/// RUN-4, steps 1 to 5; LIST-1).
pub fn start(hand_off_path: Option<&Path>) -> Result<Start, Stop> {
    let mut notices = Vec::new();
    let handed = match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice::new(
                    "detection-cut-short",
                    None,
                    "yak's detection stopped at its limit of folders, so the folders the hand-off file lists \
                     are not the whole workspace."
                        .to_owned(),
                ));
            }

            Some(handed)
        }
        None => {
            notices.push(Notice::new(
                "no-hand-off-file",
                None,
                "No hand-off file was given, so the project is read from its own standard files, with no \
                 declarations."
                    .to_owned(),
            ));

            None
        }
    };

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match &handed {
        Some(handed) => handed.root.clone(),
        None => env::current_dir().map_err(|error| {
            Stop::new(format!(
                "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
            ))
        })?,
    };

    Ok(Start {
        handed,
        root,
        notices,
    })
}

/// Reads the working tree, with the hand-off file at the path given or with none.
pub fn working(hand_off_path: Option<&Path>) -> Result<Reading, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = start(hand_off_path)?;
    let listing = files::list(&root, &mut notices)?;

    read(handed.as_ref(), &Tree::working(root), listing, notices)
}

/// Reads a project from a tree and its file list, with the hand-off file's declarations where there is
/// one (dead-code spec COMMIT-5).
pub fn read(
    handed: Option<&HandOff>,
    tree: &Tree,
    listing: Listing,
    mut notices: Vec<Notice>,
) -> Result<Reading, Stop> {
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed, &mut notices)?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = packages::load(tree, files, &marks.data, &mut notices)?.checked(
        files,
        handed.map(|handed| handed.folders.as_slice()),
        &mut notices,
    );
    let configs = tsconfig::load(
        tree,
        files,
        &marks.data,
        packages.as_ref().ok(),
        &mut notices,
    )?
    .checked(&mut notices);
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(tree, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(tree.root(), &scripts)?;
    let entries = entries::find(
        &entries::Project {
            tree,
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed,
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);

    // git failing while the base was read stops the run (dead-code spec COMMIT-4).
    tree.failure()?;

    Ok(Reading {
        handed: handed.is_some(),
        files: listing.files,
        marks,
        packages,
        scripts,
        resolved,
        entries,
        uses,
        notices,
    })
}
