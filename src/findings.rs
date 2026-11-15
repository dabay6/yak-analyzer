//! What `audit` reports: the findings a reading shows, each with its rule, its message, its location
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::notice::Notice;
use crate::reading::Reading;
use crate::scripts;

/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
const FINGERPRINT_DIGITS: usize = 32;

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// A script or a Vue component that no entry point reaches.
    UnusedFile,
    /// A value a reachable file exports that no other file uses.
    UnusedExport,
    /// A type a reachable file exports that no other file uses.
    UnusedType,
    /// An import that leads to no file.
    UnresolvedImport,
    /// Files that import each other in a loop.
    CircularDependency,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 5] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
    ];

    /// The rule's name in the report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedFile => "unused-file",
            Self::UnusedExport => "unused-export",
            Self::UnusedType => "unused-type",
            Self::UnresolvedImport => "unresolved-import",
            Self::CircularDependency => "circular-dependency",
        }
    }

    /// What the rule finds, as one sentence.
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnusedFile => "A script or a Vue component that no entry point reaches.",
            Self::UnusedExport => "A value a file exports that no other file uses.",
            Self::UnusedType => "A type a file exports that no other file uses.",
            Self::UnresolvedImport => "An import that leads to no file.",
            Self::CircularDependency => "Files that import each other in a loop.",
        }
    }

    /// The rule's place in the report's list.
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|rule| *rule == self)
            .unwrap_or_default()
    }
}

/// Where a finding is: a file, and a line and a column in it where the kind has them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    /// The file, by its path from the root.
    pub path: String,
    /// The line, counted from one.
    pub line: Option<usize>,
    /// The column, counted from one in UTF-16 code units.
    pub column: Option<usize>,
}

/// One finding.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    /// Its rule.
    pub rule: Rule,
    /// What a person can act on, as sentences.
    pub message: String,
    /// Where it is.
    pub location: Location,
    /// The other places it involves.
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
}

impl Finding {
    /// A finding, its fingerprint made from its rule and the parts given.
    fn new(rule: Rule, message: String, location: Location, parts: &[&str]) -> Self {
        Self {
            rule,
            message,
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
        }
    }
}

/// The fingerprint of a finding: the first digits of the SHA-256 digest of its rule and its parts, each
/// written after its length in bytes and a colon, so that no two lists of parts give the same input
/// (spec REPORT-3).
fn fingerprint(rule: Rule, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for part in std::iter::once(rule.name()).chain(parts.iter().copied()) {
        hasher.update(format!("{}:", part.len()).as_bytes());
        hasher.update(part.as_bytes());
    }

    let mut digits = String::with_capacity(FINGERPRINT_DIGITS);

    for byte in hasher.finalize().iter().take(FINGERPRINT_DIGITS / 2) {
        // Writing into a String cannot fail.
        let _ = write!(digits, "{byte:02x}");
    }

    digits
}

/// The findings of a reading, in the report's order, and the notices that come with them.
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = Vec::new();
    let mut notices = Vec::new();

    unused(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

    found.sort_by(|a, b| {
        (a.rule, &a.location, &a.fingerprint).cmp(&(b.rule, &b.location, &b.fingerprint))
    });

    (found, notices)
}

/// Whether the three kinds that depend on entry points are reported for a file: its package's own
/// folder is a detection folder (spec ENTRY-9).
fn reported(reading: &Reading, path: &str) -> bool {
    reading.packages.of_file.get(path).is_some_and(|folder| {
        reading
            .packages
            .list
            .iter()
            .any(|package| package.folder == *folder && package.detected)
    })
}

/// Whether a file is the project's own code to report on: read, and neither data nor generated.
fn own(reading: &Reading, file: &File) -> bool {
    !reading.marks.data.contains(&file.path)
        && !reading.marks.generated.contains(&file.path)
        && !reading.scripts.gone.contains(&file.path)
}

/// The unused files, exports and types (spec KIND-1, KIND-2).
fn unused(reading: &Reading, found: &mut Vec<Finding>) {
    let used: BTreeSet<(&str, Option<&str>)> = reading
        .uses
        .list
        .iter()
        .map(|used| (used.file.as_str(), used.name.as_deref()))
        .collect();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || !own(reading, file)
            || !reported(reading, &file.path)
        {
            continue;
        }

        let path = file.path.as_str();

        if !reading.uses.reached.contains_key(path) {
            found.push(Finding::new(
                Rule::UnusedFile,
                unused_file(reading, path),
                Location {
                    path: path.to_owned(),
                    line: None,
                    column: None,
                },
                &[path],
            ));

            // The unused-file finding covers its exports.
            continue;
        }

        let Some(read) = reading.scripts.read.get(path) else {
            continue;
        };

        for export in &read.exports {
            let name = export.name.as_str();

            if used.contains(&(path, None)) || used.contains(&(path, Some(name))) {
                continue;
            }

            let (rule, what) = if export.is_type {
                (Rule::UnusedType, "type")
            } else {
                (Rule::UnusedExport, "export")
            };
            let message = if export.used_in_file {
                format!(
                    "The {what} `{name}` is used only inside its own file, so its `export` can be dropped."
                )
            } else {
                format!("The {what} `{name}` is used nowhere, so it can be removed.")
            };

            found.push(Finding::new(
                rule,
                message,
                Location {
                    path: path.to_owned(),
                    line: Some(export.line),
                    column: Some(export.column),
                },
                &[path, name],
            ));
        }
    }
}

/// The message of an unused file: that it can go, and, in a package whose entry points are likely
/// incomplete, that the likely fault is a missing `entries` declaration (spec ENTRY-8).
fn unused_file(reading: &Reading, path: &str) -> String {
    let mut message =
        "No entry point reaches this file through any import, so it can be removed.".to_owned();
    let fault = reading.packages.of_file.get(path).and_then(|folder| {
        reading
            .entries
            .faults
            .get(folder)
            .map(|fault| (folder, fault))
    });

    if let Some((folder, fault)) = fault {
        let why = match fault {
            Fault::NoEntryPoint => "has no entry point at all",
            Fault::UnmappedTarget => "has a `bin` or `exports` target that maps to no source",
        };

        let _ = write!(
            message,
            " Its package `{folder}` {why}, so the likely fault is a missing `entries` declaration."
        );
    }

    message
}

/// One notice for each package that holds files the analyzer reads and whose folder is not a detection
/// folder, and one for the files in no package: the three kinds that depend on entry points are not
/// reported there (spec ENTRY-9).
fn not_reported(reading: &Reading, notices: &mut Vec<Notice>) {
    let analyzed = || {
        reading
            .files
            .iter()
            .filter(|file| scripts::reads(file.kind) && own(reading, file))
    };

    for package in reading
        .packages
        .list
        .iter()
        .filter(|package| !package.detected)
    {
        let holds = analyzed()
            .any(|file| reading.packages.of_file.get(&file.path) == Some(&package.folder));

        if holds {
            notices.push(Notice::new(
                "kinds-not-reported",
                None,
                format!(
                    "The package `{}` is not in a folder yak's detection looked in, so its unused files, \
                     exports and types are not reported.",
                    package.folder
                ),
            ));
        }
    }

    let outside = analyzed()
        .filter(|file| !reading.packages.of_file.contains_key(&file.path))
        .count();

    if outside > 0 {
        notices.push(Notice::new(
            "kinds-not-reported",
            None,
            format!(
                "{outside} file(s) the analyzer reads are in no package, so their unused files, exports and \
                 types are not reported."
            ),
        ));
    }
}

/// One notice for each file with an `import()` or `require()` it cannot follow: what it loads must be
/// declared (spec ENTRY-10).
fn computed(reading: &Reading, notices: &mut Vec<Notice>) {
    for (path, read) in &reading.scripts.read {
        if read.computed > 0 {
            notices.push(Notice::new(
                "computed-import",
                Some(path.clone()),
                format!(
                    "{} call(s) of `import()` or `require()` load a file the analyzer cannot work out, so \
                     what they load must be declared in `entries`.",
                    read.computed
                ),
            ));
        }
    }
}
