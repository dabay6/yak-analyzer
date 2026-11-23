//! The declaration keys the analyzer knows, and the check that it can use every declaration a hand-off
//! file holds (spec KEY-1, KEY-2, KEY-5).
//!
//! The keys and their shapes are fixed under interface version 1 since stage 2 ended (comparison spec
//! FREEZE-1): a change that breaks any raises the version, and one that only adds does not
//! (complexity spec VER-1), as the keys that tune duplication only add (duplication spec VERS-1). This
//! part checks them, the shapes of the three that describe what a tool does included (spec ENTRY-6a;
//! comparison spec CI-1); the parts that read a project use them.

use serde_json::Value;

use crate::findings::FINGERPRINT_DIGITS;
use crate::glob::Glob;
use crate::hand_off::{Stack, leaves_its_folder};
use crate::stop::Stop;

/// What kind of value a key takes.
#[derive(Clone, Copy)]
enum Kind {
    /// A list of globs.
    Globs,
    /// A list of paths.
    Paths,
    /// A positive whole number.
    Number,
    /// A whole number of 2 or more: the copies a group needs (duplication spec TUNE-3).
    Copies,
    /// A list of the words `duplicationIgnore` takes (duplication spec TUNE-4).
    Words,
    /// A list of callee chains (duplication spec TUNE-5).
    Chains,
    /// A list of accepted groups, each a fingerprint, `/` and a number of copies (duplication spec
    /// TUNE-6).
    Accepted,
    /// What a tool reads from its configuration file, which only a pack may state (spec ENTRY-6).
    ConfigEntries,
    /// The programs whose file arguments in scripts are entry points, which only a pack may state (spec
    /// ENTRY-7).
    ScriptCommands,
    /// The files a CI system reads commands from, and where in them, which only a pack may state
    /// (comparison spec CI-1).
    CommandFiles,
}

/// The fields an object of `configEntries` may hold, each required but `pattern` and `options` (spec
/// ENTRY-6, CONF-1).
const CONFIG_FIELDS: [&str; 7] = [
    "file",
    "property",
    "value",
    "pattern",
    "relativeTo",
    "default",
    "options",
];

/// The fields the `options` of a `configEntries` object may hold, each required but `relativeTo`
/// (spec CONF-1).
const OPTIONS_FIELDS: [&str; 3] = ["program", "names", "relativeTo"];

/// The fields an object of `commandFiles` may hold, each required but `folder` (comparison spec CI-1).
const COMMAND_FIELDS: [&str; 3] = ["file", "commands", "folder"];

/// The words `duplicationIgnore` takes, in the order the `duplication` command prints them (duplication
/// spec TUNE-4, SHOW-2).
pub const WORDS: [&str; 3] = ["names", "strings", "numbers"];

/// Every key the analyzer knows, with the kind of its value (spec KEY-1; duplication spec TUNE-1 to
/// TUNE-6).
const KEYS: [(&str, Kind); 15] = [
    ("entries", Kind::Globs),
    ("data", Kind::Globs),
    ("generated", Kind::Globs),
    ("internal", Kind::Paths),
    ("cyclomaticLimit", Kind::Number),
    ("cognitiveLimit", Kind::Number),
    ("duplicationMinimum", Kind::Number),
    ("duplicationMinimumLines", Kind::Number),
    ("duplicationCopies", Kind::Copies),
    ("duplicationIgnore", Kind::Words),
    ("duplicationTestCallees", Kind::Chains),
    ("duplicationAccepted", Kind::Accepted),
    ("configEntries", Kind::ConfigEntries),
    ("scriptCommands", Kind::ScriptCommands),
    ("commandFiles", Kind::CommandFiles),
];

/// Who states a declaration: the hand-off file keeps the two apart.
#[derive(Clone, Copy)]
enum Source {
    /// The pack that carries the stack.
    Pack,
    /// The project itself.
    Project,
}

impl Source {
    /// The word for the line.
    const fn word(self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::Project => "project",
        }
    }

    /// Where a person changes the declaration. The line names no `yak` command: those names are yak's.
    const fn fix(self) -> &'static str {
        match self {
            Self::Pack => "change it in the pack",
            Self::Project => "change it in the project's own declarations",
        }
    }
}

/// Checks every declaration of every stack, the stacks in the file's order and the pack's before the
/// project's. The first that cannot be used stops the run (spec KEY-2).
pub fn check(stacks: &[Stack]) -> Result<(), Stop> {
    for stack in stacks {
        for (source, declarations) in [
            (Source::Pack, &stack.pack),
            (Source::Project, &stack.project),
        ] {
            for (key, value) in declarations {
                if let Some(why) = problem(key, value, source) {
                    return Err(Stop::new(format!(
                        "cannot use the declaration `{key}` that the {} states for the stack `{}`: {why}; {}",
                        source.word(),
                        stack.name,
                        source.fix()
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Why a declaration cannot be used, or nothing when it can.
fn problem(key: &str, value: &Value, source: Source) -> Option<String> {
    let Some((_, kind)) = KEYS.iter().find(|(known, _)| *known == key) else {
        let known = KEYS.map(|(known, _)| known).join(", ");

        return Some(format!("the analyzer knows no such key, only {known}"));
    };

    match kind {
        Kind::Globs => match texts(value) {
            Some(globs) => globs.iter().find_map(|glob| glob_problem(glob)),
            None => Some("its value must be a list of globs".to_owned()),
        },
        Kind::Paths => match texts(value) {
            Some(paths) => paths.iter().find_map(|path| path_problem(path)),
            None => Some("its value must be a list of paths".to_owned()),
        },
        Kind::Number => match value.as_u64() {
            Some(number) if number > 0 => None,
            _ => Some("its value must be a positive whole number".to_owned()),
        },
        Kind::Copies => match value.as_u64() {
            Some(number) if number >= 2 => None,
            _ => Some(
                "its value must be a whole number of 2 or more, since a group is two copies at the least"
                    .to_owned(),
            ),
        },
        Kind::Words => match texts(value) {
            Some(words) => words.iter().find(|word| !WORDS.contains(*word)).map(|word| {
                format!(
                    "`{word}` is not one of the words it takes, `names`, `strings` and `numbers`"
                )
            }),
            None => Some(
                "its value must be a list of the words `names`, `strings` and `numbers`".to_owned(),
            ),
        },
        Kind::Chains => match texts(value) {
            Some(chains) => chains.iter().find(|chain| !is_chain(chain)).map(|chain| {
                format!("`{chain}` is not a callee chain, such as `it`, `it.only` or `it.each()`")
            }),
            None => Some("its value must be a list of callee chains".to_owned()),
        },
        Kind::Accepted => match texts(value) {
            Some(entries) => entries
                .iter()
                .find(|entry| !is_accepted(entry))
                .map(|entry| {
                    format!(
                        "`{entry}` is not a group's fingerprint, its {FINGERPRINT_DIGITS} lower-case \
                         hexadecimal digits, then `/` and its number of copies, 2 or more"
                    )
                }),
            None => Some("its value must be a list of accepted groups".to_owned()),
        },
        Kind::ConfigEntries | Kind::ScriptCommands | Kind::CommandFiles
            if matches!(source, Source::Project) =>
        {
            Some("only a pack may state this key".to_owned())
        }
        Kind::ConfigEntries => config_entries_problem(value),
        Kind::CommandFiles => command_files_problem(value),
        Kind::ScriptCommands => match texts(value) {
            Some(names) => names
                .iter()
                .find(|name| name.is_empty() || name.contains(char::is_whitespace))
                .map(|name| format!("the program name `{name}` is empty or holds white space")),
            None => Some("its value must be a list of program names".to_owned()),
        },
    }
}

/// Why a `configEntries` value cannot be used, or nothing when it can: a list of objects, each with the
/// fields ENTRY-6 gives and no other (spec ENTRY-6, ENTRY-6a).
fn config_entries_problem(value: &Value) -> Option<String> {
    let Some(list) = value.as_array() else {
        return Some("its value must be a list of objects".to_owned());
    };

    list.iter().enumerate().find_map(|(index, entry)| {
        let number = index + 1;
        let Some(fields) = entry.as_object() else {
            return Some(format!("its entry {number} must be an object"));
        };
        let field =
            |name: &str, why: &str| Some(format!("its entry {number}'s field `{name}` {why}"));

        if let Some(unknown) = fields
            .keys()
            .find(|key| !CONFIG_FIELDS.contains(&key.as_str()))
        {
            return field(unknown, "is not one it may hold");
        }

        let folder = fields.get("value") == Some(&Value::from("folder"));

        match (fields.get("pattern"), folder) {
            (None, true) => return field("pattern", "is missing, and a `folder` value needs it"),
            (Some(_), false) => return field("pattern", "is only for a `folder` value"),
            _ => {}
        }

        if let Some(missing) = CONFIG_FIELDS
            .iter()
            .find(|name| !matches!(**name, "pattern" | "options") && !fields.contains_key(**name))
        {
            return field(missing, "is missing");
        }

        let property = fields["property"]
            .as_array()
            .filter(|keys| !keys.is_empty() && keys.iter().all(Value::is_string));

        if property.is_none() {
            return field("property", "must be a list of keys, one or more");
        }

        if !matches!(fields["value"].as_str(), Some("globs" | "folder")) {
            return field("value", "must be `globs` or `folder`");
        }

        if !matches!(fields["relativeTo"].as_str(), Some("config" | "detection")) {
            return field("relativeTo", "must be `config` or `detection`");
        }

        for name in ["file", "pattern"] {
            match fields.get(name).map(Value::as_str) {
                None => {}
                Some(Some(glob)) => {
                    if let Some(why) = glob_problem(glob) {
                        return field(name, &format!("cannot be used: {why}"));
                    }
                }
                Some(None) => return field(name, "must be a glob"),
            }
        }

        if let Some(why) = fields.get("options").and_then(options_problem) {
            return field("options", &why);
        }

        match texts(&fields["default"]) {
            Some(globs) => globs
                .iter()
                .find_map(|glob| glob_problem(glob))
                .and_then(|why| field("default", &format!("cannot be used: {why}"))),
            None => field("default", "must be a list of globs"),
        }
    })
}

/// Why a `commandFiles` value cannot be used, or nothing when it can: a list of objects, each with a
/// `file` glob and its `commands` paths, optionally its `folder` paths, and no other field (comparison
/// spec CI-1, CI-3).
fn command_files_problem(value: &Value) -> Option<String> {
    let Some(list) = value.as_array() else {
        return Some("its value must be a list of objects".to_owned());
    };

    list.iter().enumerate().find_map(|(index, entry)| {
        let number = index + 1;
        let Some(fields) = entry.as_object() else {
            return Some(format!("its entry {number} must be an object"));
        };
        let field =
            |name: &str, why: &str| Some(format!("its entry {number}'s field `{name}` {why}"));

        if let Some(unknown) = fields
            .keys()
            .find(|key| !COMMAND_FIELDS.contains(&key.as_str()))
        {
            return field(unknown, "is not one it may hold");
        }

        if let Some(missing) = ["file", "commands"]
            .iter()
            .find(|name| !fields.contains_key(**name))
        {
            return field(missing, "is missing");
        }

        let Some(glob) = fields["file"].as_str() else {
            return field("file", "must be a glob");
        };

        if let Some(why) = glob_problem(glob) {
            return field("file", &format!("cannot be used: {why}"));
        }

        ["commands", "folder"]
            .iter()
            .find(|name| fields.get(**name).is_some_and(|paths| !is_paths(paths)))
            .and_then(|name| {
                field(
                    name,
                    "must be a list of property paths, one or more, each a list of keys, one or more",
                )
            })
    })
}

/// Whether a value is a list of one or more property paths, each a list of one or more keys.
fn is_paths(value: &Value) -> bool {
    value.as_array().is_some_and(|paths| {
        !paths.is_empty()
            && paths
                .iter()
                .all(|path| texts(path).is_some_and(|keys| !keys.is_empty()))
    })
}

/// Why the `options` of a `configEntries` object cannot be used, or nothing when they can: an object
/// with the program's name, a list of one or more option names, and, optionally, what a named file's
/// value is relative to (spec CONF-1).
fn options_problem(options: &Value) -> Option<String> {
    let Some(fields) = options.as_object() else {
        return Some("must be an object".to_owned());
    };

    if let Some(unknown) = fields
        .keys()
        .find(|key| !OPTIONS_FIELDS.contains(&key.as_str()))
    {
        return Some(format!("holds `{unknown}`, which it may not"));
    }

    if !fields
        .get("program")
        .and_then(Value::as_str)
        .is_some_and(is_word)
    {
        return Some("must name its `program`, one word with no white space".to_owned());
    }

    let Some(names) = fields
        .get("names")
        .and_then(texts)
        .filter(|names| !names.is_empty())
    else {
        return Some("must list its option `names`, one or more".to_owned());
    };

    if let Some(name) = names
        .iter()
        .find(|name| !is_word(name) || name.contains('='))
    {
        return Some(format!(
            "names the option `{name}`, which is empty or holds white space or `=`"
        ));
    }

    match fields.get("relativeTo").map(Value::as_str) {
        None | Some(Some("config" | "detection" | "script")) => None,
        Some(_) => {
            Some("has a `relativeTo` that is not `config`, `detection` or `script`".to_owned())
        }
    }
}

/// Whether a text is one word: not empty, with no white space.
fn is_word(text: &str) -> bool {
    !text.is_empty() && !text.contains(char::is_whitespace)
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<&str>> {
    value.as_array()?.iter().map(Value::as_str).collect()
}

/// Whether a text is a callee chain as the duplication spec's terms write one: a name, then any number
/// of `.` and a name (`#` and a name for a private one), `()` for a call, `[]` for a computed member,
/// and two backticks for a tagged template (duplication spec TUNE-5).
fn is_chain(text: &str) -> bool {
    let Some(mut rest) = after_name(text) else {
        return false;
    };

    while !rest.is_empty() {
        rest = if let Some(after) = ["()", "[]", "``"]
            .iter()
            .find_map(|piece| rest.strip_prefix(piece))
        {
            after
        } else if let Some(after) = rest.strip_prefix('.') {
            match after_name(after.strip_prefix('#').unwrap_or(after)) {
                Some(after) => after,
                None => return false,
            }
        } else {
            return false;
        };
    }

    true
}

/// What follows the name a text starts with: one or more letters, digits, `_` or `$`, not starting with
/// a digit; nothing where the text does not start with a name.
fn after_name(text: &str) -> Option<&str> {
    let end = text
        .find(|letter: char| !(letter.is_alphanumeric() || letter == '_' || letter == '$'))
        .unwrap_or(text.len());

    (end > 0 && !text.starts_with(|letter: char| letter.is_ascii_digit())).then(|| &text[end..])
}

/// Whether a text is an accepted group: its fingerprint in lower-case hexadecimal digits, `/`, and its
/// number of copies, 2 or more, written with no sign and no leading zero (duplication spec TUNE-6).
fn is_accepted(text: &str) -> bool {
    text.split_once('/').is_some_and(|(fingerprint, copies)| {
        fingerprint.len() == FINGERPRINT_DIGITS
            && fingerprint
                .bytes()
                .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
            && !copies.is_empty()
            && !copies.starts_with('0')
            && copies.bytes().all(|digit| digit.is_ascii_digit())
            && copies.parse::<u64>().is_ok_and(|copies| copies >= 2)
    })
}

/// Why a path cannot be used, or nothing when it can.
fn path_problem(path: &str) -> Option<String> {
    if path.is_empty() {
        Some("a path in it is empty".to_owned())
    } else if leaves_its_folder(path) {
        Some(format!(
            "the path `{path}` is absolute or steps out of its folder with `..`"
        ))
    } else {
        None
    }
}

/// Why a glob cannot be used, or nothing when it can (spec KEY-5).
fn glob_problem(glob: &str) -> Option<String> {
    if glob.is_empty() {
        return Some("a glob in it is empty".to_owned());
    }

    if leaves_its_folder(glob) {
        return Some(format!(
            "the glob `{glob}` is absolute or steps out of its folder with `..`"
        ));
    }

    Glob::new(glob).err()
}
