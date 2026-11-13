//! How a run ends when it cannot go on: one line on standard error and an exit code (spec RUN-2, RUN-3).

use std::fmt::Write as _;

/// The exit code of a run that could not go on (spec RUN-3).
pub const STOPPED: u8 = 1;

/// The exit code for a command or an option the binary does not have (spec RUN-2).
pub const REFUSED: u8 = 2;

/// The exit code of a fault in the analyzer itself (spec RUN-3).
pub const FAULT: u8 = 3;

/// The name every line on standard error begins with.
const NAME: &str = "yak-analyzer";

/// A run that cannot go on, as the one line that says what failed and what fixes it.
#[derive(Debug)]
pub struct Stop(String);

impl Stop {
    /// A stop whose line is the text given, without the program's name.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The line for standard error.
    pub fn line(&self) -> String {
        line(&self.0)
    }
}

/// One line for standard error: the program's name, then the text with every line break and control
/// character escaped, so that one line stays one line whatever a file is named.
pub fn line(text: &str) -> String {
    let mut escaped = String::with_capacity(NAME.len() + 2 + text.len());

    escaped.push_str(NAME);
    escaped.push_str(": ");

    for character in text.chars() {
        if character.is_control() {
            // Writing into a String cannot fail.
            let _ = write!(escaped, "{}", character.escape_default());
        } else {
            escaped.push(character);
        }
    }

    escaped
}

/// A stop of RUN-4's eleventh step, a field of a form it does not have, with the file it is about: where
/// several files fail that step, the first by path is the one reported (spec RUN-4).
#[derive(Debug)]
pub struct Fault {
    /// The file, by its path from the root.
    file: String,
    /// The stop.
    stop: Stop,
}

impl Fault {
    /// A fault about the file given.
    pub fn new(file: impl Into<String>, stop: Stop) -> Self {
        Self {
            file: file.into(),
            stop,
        }
    }

    /// The first of the faults by path, compared byte by byte; of one file's, the first found.
    pub fn first(faults: impl IntoIterator<Item = Self>) -> Option<Self> {
        faults.into_iter().reduce(Self::earlier)
    }

    /// The earlier of two faults by path; the first given when both are about one file.
    pub fn earlier(self, other: Self) -> Self {
        if other.file < self.file { other } else { self }
    }

    /// The stop.
    pub fn stop(self) -> Stop {
        self.stop
    }
}
