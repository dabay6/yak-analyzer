//! Reading every file of the list that holds imports: its content from disk, parsed, and its imports
//! found. Scripts and type declaration files are parsed whole; Vue components, MDX documents and HTML
//! pages by their own readers (spec PARSE-1 to PARSE-6, LIST-3).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::files::{File, Kind};
use crate::imports::{self, Broken, Read};
use crate::notice::Notice;
use crate::pages::{self, Skipped};
use crate::stop::Stop;

/// What reading the scripts found.
#[derive(Debug, Default)]
pub struct Scripts {
    /// What each file that was read holds, by its path.
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

/// Reads every script, type declaration file, Vue component, MDX document and HTML page in the list
/// that is not data, in the order of the list. A tracked file that cannot be read stops the run; an
/// untracked one is skipped with a notice (spec PARSE-2).
pub fn read(
    root: &Path,
    files: &[File],
    data: &BTreeSet<String>,
    notices: &mut Vec<Notice>,
) -> Result<Scripts, Stop> {
    let mut scripts = Scripts::default();

    for file in files {
        if !reads(file.kind) || data.contains(&file.path) {
            continue;
        }

        match one(root, file) {
            Ok((read, skipped)) => {
                notices.extend(
                    skipped
                        .into_iter()
                        .map(|each| passed_over(&file.path, each)),
                );
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

/// Whether the analyzer reads the imports of a file of the kind given (spec PARSE-1, PARSE-3 to
/// PARSE-5).
pub const fn reads(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Script | Kind::Declaration | Kind::Vue | Kind::Mdx | Kind::Page
    )
}

/// The notice for what a reader passed over (spec PARSE-3, PARSE-4).
fn passed_over(path: &str, skipped: Skipped) -> Notice {
    match skipped {
        Skipped::Language { line, language } => Notice::new(
            "vue-block-not-read",
            Some(path.to_owned()),
            format!(
                "The script block on line {line} is in the language `{language}`, which the analyzer does not read, \
                 so its imports are not read."
            ),
        ),
        Skipped::Statement { line } => Notice::new(
            "mdx-line-not-read",
            Some(path.to_owned()),
            format!(
                "Line {line} begins like an `import` or `export` statement and does not parse as one, so it is \
                 read as text."
            ),
        ),
    }
}

/// Reads one file and finds its imports, with what its reader passed over.
fn one(root: &Path, file: &File) -> Result<(Read, Vec<Skipped>), Unread> {
    let path = file.path.as_str();
    let bytes = fs::read(root.join(path)).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Unread::Gone,
        _ => Unread::Broken(format!("cannot be read ({error})")),
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let text =
        std::str::from_utf8(bytes).map_err(|_| Unread::Broken("is not valid UTF-8".to_owned()))?;

    let broken = |Broken { line, message }| {
        Unread::Broken(match line {
            Some(line) => format!("does not parse (line {line}: {message})"),
            None => format!("does not parse ({message})"),
        })
    };

    // An MDX document and an HTML page hold nothing that fails to parse (spec PARSE-2).
    match file.kind {
        Kind::Vue => pages::vue(text).map_err(broken),
        Kind::Mdx => Ok(pages::mdx(text)),
        Kind::Page => Ok((pages::html(text), Vec::new())),
        _ => imports::read(path, text)
            .map(|read| (read, Vec::new()))
            .map_err(broken),
    }
}
