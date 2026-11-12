//! How the binary speaks on standard error: one line, and an exit code (spec RUN-2, RUN-3).

use std::fmt::Write as _;

/// The exit code for a command or an option the binary does not have (spec RUN-2).
pub const REFUSED: u8 = 2;

/// The exit code of a fault in the analyzer itself (spec RUN-3).
pub const FAULT: u8 = 3;

/// The name every line on standard error begins with.
const NAME: &str = "yak-analyzer";

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
