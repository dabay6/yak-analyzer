//! The `data`, `generated` and `unreported` declarations, applied to the file list, and the notices that
//! say how many findings the `unreported` globs left out (spec KEY-3, KEY-4, KEY-5; ignoring spec
//! KEY-6, COUNT-1).

use std::collections::BTreeSet;

use serde_json::Value;

use crate::files::File;
use crate::glob::Glob;
use crate::hand_off::HandOff;
use crate::notice::Notice;
use crate::stop::Stop;

/// The key whose globs name the files no finding is reported in (ignoring spec KEY-6).
pub const UNREPORTED: &str = "unreported";

/// The kind of the notices that say how many findings the `unreported` globs left out (ignoring spec
/// COUNT-1).
#[expect(
    dead_code,
    reason = "the counts are made once the checks hand back what they left out"
)]
const LEFT_OUT: &str = "findings-unreported";

/// The keys whose globs mark files, and what they mark them as.
const KEYS: [&str; 3] = ["data", "generated", UNREPORTED];

/// One glob of the `unreported` declaration, however many stacks state it, with the files it matches
/// (ignoring spec COUNT-1).
#[derive(Debug)]
pub struct Unreported {
    /// The glob, as it is written.
    pub written: String,
    /// The stacks that state it, in the order of the hand-off file.
    pub stacks: Vec<String>,
    /// The files it matches, by path.
    pub files: BTreeSet<String>,
}

/// Which files the declarations mark, by path.
#[derive(Debug, Default)]
pub struct Marks {
    /// Files that are not the project's code, which are never parsed or read (spec KEY-4).
    pub data: BTreeSet<String>,
    /// Committed files a generator writes, which are read like any other.
    pub generated: BTreeSet<String>,
    /// Files that are read like any other, in which no finding is reported (ignoring spec KEY-6).
    pub unreported: BTreeSet<String>,
    /// The globs of the `unreported` declaration, each once, in the order they are first stated.
    pub globs: Vec<Unreported>,
}

impl Marks {
    /// Notes a glob of the `unreported` declaration that a stack states, with the files it matches: a
    /// glob another stack states too, written the same, is one glob (ignoring spec COUNT-1).
    fn note_unreported(&mut self, written: &str, stack: &str, paths: BTreeSet<String>) {
        self.unreported.extend(paths.iter().cloned());

        let at = self
            .globs
            .iter()
            .position(|glob| glob.written == written)
            .unwrap_or_else(|| {
                self.globs.push(Unreported {
                    written: written.to_owned(),
                    stacks: Vec::new(),
                    files: BTreeSet::new(),
                });
                self.globs.len() - 1
            });
        let glob = &mut self.globs[at];

        if !glob.stacks.iter().any(|named| named == stack) {
            glob.stacks.push(stack.to_owned());
        }

        glob.files.extend(paths);
    }

    /// The notices that say how many findings the `unreported` globs left out, from each finding left
    /// out given by the files that tie it to the globs: one for each glob that matches a file, counting
    /// each finding one of whose files it matches, and one more for the findings none of whose files is
    /// unreported, where there are any (ignoring spec COUNT-1, COUNT-2).
    #[expect(
        dead_code,
        reason = "the checks hand back what they left out in a later change"
    )]
    pub fn counts(&self, left: &[Vec<String>]) -> Vec<Notice> {
        let mut notices: Vec<Notice> = self
            .globs
            .iter()
            .filter(|glob| !glob.files.is_empty())
            .map(|glob| {
                let count = left
                    .iter()
                    .filter(|files| files.iter().any(|file| glob.files.contains(file)))
                    .count();
                let stacks = if glob.stacks.len() == 1 {
                    "stack"
                } else {
                    "stacks"
                };

                Notice::new(
                    LEFT_OUT,
                    None,
                    format!(
                        "The `{UNREPORTED}` glob `{}` that the project states for the {stacks} `{}` left \
                         out {count} finding(s).",
                        glob.written,
                        glob.stacks.join("`, `")
                    ),
                )
            })
            .collect();
        let outside = left
            .iter()
            .filter(|files| !files.iter().any(|file| self.unreported.contains(file)))
            .count();

        if outside > 0 {
            notices.push(Notice::new(
                LEFT_OUT,
                None,
                format!(
                    "{outside} more finding(s) outside the `{UNREPORTED}` files are not reported, since \
                     only those files use what they name."
                ),
            ));
        }

        notices
    }
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
            // the root (spec KEY-3). Only a project states `unreported` (ignoring spec KEY-7).
            let sources = [
                ("pack", &stack.pack, handed.folders.as_slice()),
                ("project", &stack.project, &[".".to_owned()][..]),
            ];

            for (source, declarations, folders) in sources {
                for written in globs(declarations.get(key)) {
                    let glob = Glob::new(written).map_err(Stop::new)?;
                    let matched: BTreeSet<String> = files
                        .iter()
                        .filter(|file| {
                            folders
                                .iter()
                                .any(|folder| under(folder, &file.path, &glob))
                        })
                        .map(|file| file.path.clone())
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

                    match key {
                        "data" => marks.data.extend(matched),
                        "generated" => marks.generated.extend(matched),
                        _ => marks.note_unreported(written, &stack.name, matched),
                    }
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
