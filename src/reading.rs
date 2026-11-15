//! Reading a project: its hand-off file, its files, its packages, its scripts' imports and where they
//! lead, its entry points, and which export of which file is used. `graph` prints a reading, and
//! `audit` reports what one shows (spec OUT-1, CMD-1, CMD-4).

use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off::{self, HandOff};
use crate::marks::{self, Marks};
use crate::notice::Notice;
use crate::packages::{self, Packages};
use crate::resolve::{Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;
use crate::uses::{self, Uses};

/// What reading a project found.
pub struct Reading {
    /// The hand-off file, where one was given.
    pub handed: Option<HandOff>,
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

/// Reads the project, with the hand-off file at the path given or with none.
pub fn read(hand_off_path: Option<&Path>) -> Result<Reading, Stop> {
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
    let listing = files::list(&root, &mut notices)?;
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed.as_ref(), &mut notices)?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = packages::load(&root, files, &marks.data)?.checked(
        files,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    );
    let configs = tsconfig::load(
        &root,
        files,
        &marks.data,
        packages.as_ref().ok(),
        &mut notices,
    )?
    .checked();
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            root: &root,
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed: handed.as_ref(),
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);

    Ok(Reading {
        handed,
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
