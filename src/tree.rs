//! Where a reading reads its files from: the working tree on disk, or the files git stores at a commit
//! (spec LIST-9; dead-code spec COMMIT-1 to COMMIT-3).
//!
//! Every reader reads a listed file through a tree, so one reading works on either. Nothing is checked
//! out and nothing is written: a commit's files come from one batch of git's, started once.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::files::{self, Listing};
use crate::git::{self, Above, Batch};
use crate::notice::Notice;
use crate::stop::Stop;

/// A commit's stored files, and the git that reads them.
struct Stored {
    /// The commit's full name.
    commit: String,
    /// The root's folder from the top of the work tree, ending in `/`, or empty at the top.
    prefix: String,
    /// The object that holds each listed file's content, by the file's path.
    objects: BTreeMap<String, String>,
    /// The git that hands out the contents.
    batch: Mutex<Batch>,
    /// The first time git failed while the commit was read, which stops the run (dead-code spec
    /// COMMIT-4).
    failure: Mutex<Option<Stop>>,
}

/// Where a reading reads from.
pub struct Tree {
    /// The root.
    root: PathBuf,
    /// The commit's files, for the base; none for the working tree.
    stored: Option<Stored>,
}

impl Tree {
    /// The working tree under the root, as it is on disk.
    pub const fn working(root: PathBuf) -> Self {
        Self { root, stored: None }
    }

    /// The files git stores at a commit under the root, with their list (dead-code spec COMMIT-2).
    pub fn base(
        root: PathBuf,
        commit: &str,
        notices: &mut Vec<Notice>,
    ) -> Result<(Self, Listing), Stop> {
        let (listing, objects) = files::stored(&root, commit, notices)?;
        let stored = Stored {
            commit: commit.to_owned(),
            prefix: git::prefix(&root)?,
            objects,
            batch: Mutex::new(Batch::start(&root)?),
            failure: Mutex::new(None),
        };

        Ok((
            Self {
                root,
                stored: Some(stored),
            },
            listing,
        ))
    }

    /// The root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether this is the base, where a file that cannot be read or does not have its form is skipped
    /// rather than stopping the run (dead-code spec COMMIT-4).
    pub const fn is_base(&self) -> bool {
        self.stored.is_some()
    }

    /// The content of a file, by its path from the root; a path above the root begins with `..`.
    pub fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        let Some(stored) = &self.stored else {
            return fs::read(self.root.join(path));
        };
        let listed = stored.objects.get(path);
        let name = match listed {
            Some(object) => object.clone(),
            None => format!("{}:{}", stored.commit, from_top(&stored.prefix, path)),
        };
        let content = stored
            .batch
            .lock()
            .map_err(|_| io::Error::other("git's batch was left broken"))
            .and_then(|mut batch| batch.read(&name).map_err(|stop| self.failed(stop)))?;

        match content {
            Some(content) => Ok(content),
            // A listed file whose content git does not have is in a partial clone that lacks it, and
            // the analyzer never fetches it (dead-code spec COMMIT-2).
            None if listed.is_some() => Err(self.failed(Stop::new(format!(
                "git does not have the content of {path} at the base, and the analyzer does not fetch it; fetch \
                 the base commit's files, and run again"
            )))),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "not stored at the base",
            )),
        }
    }

    /// What git says of a path above the root (spec LIST-4, CFG-3). At the base, a file is one the commit
    /// stores; whether git ignores a path is asked of the working tree's ignore rules (dead-code spec
    /// COMMIT-3).
    pub fn above(&self, path: &str) -> Result<Above, Stop> {
        let Some(stored) = &self.stored else {
            return git::above(&self.root, path);
        };

        if git::leaves_work_tree(&stored.prefix, path) {
            return Ok(Above::OutsideWorkTree);
        }

        let name = format!("{}:{}", stored.commit, from_top(&stored.prefix, path));
        let found = stored
            .batch
            .lock()
            .map_err(|_| Stop::new("git's batch was left broken; this is a fault in the analyzer"))?
            .read(&name)?;

        Ok(if found.is_some() {
            Above::File
        } else if git::ignored(&self.root, &[path])?.is_empty() {
            Above::Missing
        } else {
            Above::Ignored
        })
    }

    /// The stop of the first time git failed while the base was read, where it failed.
    pub fn failure(&self) -> Result<(), Stop> {
        match self
            .stored
            .as_ref()
            .and_then(|stored| stored.failure.lock().ok()?.take())
        {
            Some(stop) => Err(stop),
            None => Ok(()),
        }
    }

    /// Keeps the first failure of git's, and gives the reader an error that makes it skip the file;
    /// the run stops once the reading ends.
    fn failed(&self, stop: Stop) -> io::Error {
        if let Some(stored) = &self.stored
            && let Ok(mut failure) = stored.failure.lock()
        {
            failure.get_or_insert(stop);
        }

        io::Error::other("git failed while the base was read")
    }
}

/// A path from the root as a path from the top of the work tree, with `..` worked out.
fn from_top(prefix: &str, path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();

    for segment in prefix.split('/').chain(path.split('/')) {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    segments.join("/")
}
