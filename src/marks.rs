//! The `data` and `generated` declarations, applied to the file list (spec KEY-3, KEY-4, KEY-5).

use std::collections::BTreeSet;

use serde_json::Value;

use crate::files::File;
use crate::glob::Glob;
use crate::hand_off::HandOff;
use crate::notice::Notice;
use crate::stop::Stop;

/// The keys whose globs mark files, and what they mark them as.
const KEYS: [&str; 2] = ["data", "generated"];

/// Which files the declarations mark, by path.
#[derive(Debug, Default)]
pub struct Marks {
    /// Files that are not the project's code, which are never parsed or read (spec KEY-4).
    pub data: BTreeSet<String>,
    /// Committed files a generator writes, which are read like any other.
    pub generated: BTreeSet<String>,
}

/// Marks the files the declarations of the hand-off file name. Without a hand-off file there are no
/// declarations, and nothing is marked.
pub fn apply(
    files: &[File],
    handed: Option<&HandOff>,
    notices: &mut Vec<Notice>,
) -> Result<Marks, Stop> {
    let mut marks = Marks::default();
    let Some(handed) = handed else {
        return Ok(marks);
    };

    for stack in &handed.stacks {
        for key in KEYS {
            // A pack's globs are relative to each folder yak's detection looked in, and a project's to
            // the root (spec KEY-3).
            let sources = [
                ("pack", &stack.pack, handed.folders.as_slice()),
                ("project", &stack.project, &[".".to_owned()][..]),
            ];

            for (source, declarations, folders) in sources {
                for written in globs(declarations.get(key)) {
                    let glob = Glob::new(written).map_err(Stop::new)?;
                    let matched: Vec<&File> = files
                        .iter()
                        .filter(|file| {
                            folders
                                .iter()
                                .any(|folder| under(folder, &file.path, &glob))
                        })
                        .collect();

                    if matched.is_empty() {
                        notices.push(Notice::new(
                            "glob-matches-nothing",
                            None,
                            format!(
                                "The `{key}` glob `{written}` that the {source} states for the stack `{}` \
                                 matches no file.",
                                stack.name
                            ),
                        ));
                    }

                    let marked = if key == "data" {
                        &mut marks.data
                    } else {
                        &mut marks.generated
                    };

                    marked.extend(matched.into_iter().map(|file| file.path.clone()));
                }
            }
        }
    }

    Ok(marks)
}

/// The globs a declaration holds. The declaration was checked when the hand-off file was read, so
/// anything else holds none.
fn globs(declaration: Option<&Value>) -> Vec<&str> {
    declaration
        .and_then(Value::as_array)
        .map(|globs| globs.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// Whether a path lies under the folder and the glob matches it relative to that folder.
fn under(folder: &str, path: &str, glob: &Glob) -> bool {
    if folder == "." {
        return glob.matches(path);
    }

    path.strip_prefix(folder)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|rest| glob.matches(rest))
}
