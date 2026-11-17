//! Notices: what a person should know about a run that went on (spec RUN-5).

use serde_json::{Value, json};

use crate::stop::Stop;

/// The kind of the notice for a file the base's reading skipped (dead-code spec COMMIT-4).
pub const SKIPPED: &str = "file-skipped";

/// Something a person should know about a run that went on.
#[derive(Debug, PartialEq, Eq)]
pub struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
    /// Whether the base's reading gave it (dead-code spec REPORT-4).
    base: bool,
}

impl Notice {
    /// A notice of the kind given, about a file or about none.
    pub const fn new(kind: &'static str, file: Option<String>, sentence: String) -> Self {
        Self {
            kind,
            file,
            sentence,
            base: false,
        }
    }

    /// The notice for a file the base's reading skips, with what would have stopped the run on the
    /// working tree (dead-code spec COMMIT-4).
    pub fn skipped(file: &str, stop: &Stop) -> Self {
        Self::new(
            SKIPPED,
            Some(file.to_owned()),
            format!(
                "The file is skipped at the base, where nobody can fix it: {}.",
                stop.reason()
            ),
        )
    }

    /// The notice, marked as the base's.
    pub const fn of_base(mut self) -> Self {
        self.base = true;
        self
    }

    /// The kind of notice.
    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    /// The file it is about, where there is one.
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// What happened, as a sentence.
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// Whether the base's reading gave it.
    pub const fn is_base(&self) -> bool {
        self.base
    }

    /// The notice as `graph` prints it.
    fn printed(&self) -> Value {
        json!({ "file": self.file, "kind": self.kind, "sentence": self.sentence })
    }
}

/// The notices in the order they are printed in: the working tree's before the base's, and of each,
/// those about no file first, then by file, by kind and by sentence (spec RUN-8).
pub fn sorted(mut notices: Vec<Notice>) -> Vec<Notice> {
    notices.sort_by(|a, b| {
        (a.base, &a.file, a.kind, &a.sentence).cmp(&(b.base, &b.file, b.kind, &b.sentence))
    });

    notices
}

/// The notices as `graph` prints them, in their order.
pub fn printed(notices: Vec<Notice>) -> Vec<Value> {
    sorted(notices).iter().map(Notice::printed).collect()
}
