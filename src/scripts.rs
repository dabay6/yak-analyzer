//! Reading every script of the file list: its content from disk, parsed, and its imports found (spec
//! PARSE-1, PARSE-2, PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::files::{File, Kind};
use crate::imports::{self, Broken, Read};
use crate::notice::Notice;
use crate::stop::Stop;

/// What reading the scripts found.
#[derive(Debug, Default)]
pub struct Scripts {
    /// What each script that was read holds, by its path.
    pub read: BTreeMap<String, Read>,
    /// The listed files that were gone by the time they were read, which leave the list (spec LIST-3).
    pub gone: BTreeSet<String>,
}

/// Why a file was not read.
enum Unread {
    /// It is gone from disk since git listed it.
    Gone,
    /// It cannot be read, does not hold UTF-8, or does not parse; the text says which.
    Broken(String),
}

/// Reads every script and type declaration file in the list that is not data, in the order of the list.
/// A tracked file that cannot be read stops the run; an untracked one is skipped with a notice (spec
/// PARSE-2).
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
) -> Result<Scripts, Stop> {
    let mut scripts = Scripts::default();

    for file in files {
        let script = matches!(file.kind, Kind::Script | Kind::Declaration);

        if !script || data.contains(&file.path) {
            continue;
        }

        match one(root, &file.path) {
            Ok(read) => {
                scripts.read.insert(file.path.clone(), read);
            }
            Err(Unread::Gone) => {
                notices.push(Notice::new(
                    "file-gone",
                    Some(file.path.clone()),
                    "The file was gone by the time it was read, so it is left out of the file list.".to_owned(),
                ));
                scripts.gone.insert(file.path.clone());
            }
            Err(Unread::Broken(why)) if file.tracked => {
                return Err(Stop::new(format!(
                    "the file {} {why}; fix it, or declare it as data if it is broken on purpose",
                    file.path
                )));
            }
            Err(Unread::Broken(why)) => notices.push(Notice::new(
                "untracked-file-skipped",
                Some(file.path.clone()),
                format!("The untracked file {why}, so its imports are not read."),
            )),
        }
    }

    Ok(scripts)
}

/// Reads and parses one script.
fn one(root: &Path, path: &str) -> Result<Read, Unread> {
    let bytes = fs::read(root.join(path)).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Unread::Gone,
        _ => Unread::Broken(format!("cannot be read ({error})")),
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let text =
        std::str::from_utf8(bytes).map_err(|_| Unread::Broken("is not valid UTF-8".to_owned()))?;

    imports::read(path, text).map_err(|Broken { line, message }| {
        Unread::Broken(match line {
            Some(line) => format!("does not parse (line {line}: {message})"),
            None => format!("does not parse ({message})"),
        })
    })
}
