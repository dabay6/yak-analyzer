//! Notices: what a person should know about a run that went on (spec RUN-5).

use serde_json::{Value, json};

/// Something a person should know about a run that went on.
#[derive(Debug)]
pub struct Notice {
    /// The kind of notice, a fixed name.
    kind: &'static str,
    /// The file it is about, relative to the root, where there is one.
    file: Option<String>,
    /// What happened, as a sentence.
    sentence: String,
}

impl Notice {
    /// A notice of the kind given, about a file or about none.
    pub const fn new(kind: &'static str, file: Option<String>, sentence: String) -> Self {
        Self {
            kind,
            file,
            sentence,
        }
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

    /// The notice as `graph` prints it.
    fn printed(&self) -> Value {
        json!({ "file": self.file, "kind": self.kind, "sentence": self.sentence })
    }
}

/// The notices in the order they are printed in: those about no file first, then by file, by kind and
/// by sentence (spec RUN-8).
pub fn sorted(mut notices: Vec<Notice>) -> Vec<Notice> {
    notices.sort_by(|a, b| (&a.file, a.kind, &a.sentence).cmp(&(&b.file, b.kind, &b.sentence)));

    notices
}

/// The notices as `graph` prints them, in their order.
pub fn printed(notices: Vec<Notice>) -> Vec<Value> {
    sorted(notices).iter().map(Notice::printed).collect()
}
