//! Reads the hand-off file yak writes for one checkout (spec HO-1 to HO-4).
//!
//! The file is checked in a fixed order, so that two faults in one file always give the same line: that
//! it can be read, its format version, the shape of each field, and then each stack's interface version
//! (spec RUN-4).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::stop::Stop;
use crate::{HAND_OFF_FORMATS, INTERFACE_VERSION};

/// The largest whole number the format allows, which is the largest JSON carries exactly.
const LARGEST_NUMBER: u64 = 9_007_199_254_740_991;

/// The lengths, in hexadecimal digits, a commit's full name can have.
const COMMIT_NAME_LENGTHS: [usize; 2] = [40, 64];

/// What the analyzer takes from a hand-off file of format 1. Every field of the format is checked when
/// the file is read; a field is kept here once a part of the analyzer uses it.
#[derive(Debug)]
pub struct HandOff {
    /// The checkout's root, which is the folder to read whatever folder the analyzer was started in.
    pub root: PathBuf,
    /// Whether yak's detection stopped at its limit, so the folders it lists are not the whole workspace.
    pub truncated: bool,
    /// The stacks that carry declarations, in the order yak applies them.
    pub stacks: Vec<Stack>,
}

/// One stack and the declarations it carries, the pack's kept apart from the project's.
#[derive(Debug)]
pub struct Stack {
    /// The stack's name.
    pub name: String,
    /// The interface version the stack's declarations were written for.
    pub interface_version: Option<u64>,
    /// The pack's declarations: each key with its value.
    pub pack: Map<String, Value>,
    /// The project's own declarations under this stack.
    pub project: Map<String, Value>,
}

/// Reads and checks the hand-off file at the path given.
pub fn read(path: &Path) -> Result<HandOff, Stop> {
    let shown = path.display();
    let bytes = fs::read(path).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} cannot be read ({error}); pass the path of the file yak wrote for this \
             checkout"
        ))
    })?;
    // A byte-order mark at the start of a file is passed over (spec PARSE-1).
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let document: Value = serde_json::from_slice(text).map_err(|error| {
        Stop::new(format!(
            "the hand-off file {shown} is not JSON (line {}, column {}); yak writes this file, so write it again \
             with yak",
            error.line(),
            error.column()
        ))
    })?;
    let shape = |problem: String| {
        Stop::new(format!(
            "the hand-off file {shown} is not format 1: {problem}; yak writes this file, so write it again with \
             yak"
        ))
    };
    let fields = document
        .as_object()
        .ok_or_else(|| shape("the file must hold one object".to_owned()))?;

    format_version(fields, &shown.to_string()).map_err(|stopped| match stopped {
        Fault::Stop(stop) => stop,
        Fault::Shape(problem) => shape(problem),
    })?;

    // The fields are checked in the order the format's table lists them (spec HO-2).
    let root = absolute_path(fields, "root").map_err(shape)?;

    folders(fields).map_err(shape)?;

    let truncated = truncated(fields).map_err(shape)?;
    let stacks = stacks(fields).map_err(shape)?;

    absolute_path(fields, "siblings").map_err(shape)?;
    base(fields).map_err(shape)?;

    let handed = HandOff {
        root,
        truncated,
        stacks,
    };

    for stack in &handed.stacks {
        interface_version(stack, &shown.to_string())?;
    }

    Ok(handed)
}

/// What can be wrong with the format version: a format the analyzer does not read, or a shape.
enum Fault {
    /// The file states a format the analyzer does not read.
    Stop(Stop),
    /// The field is missing or is not a whole number.
    Shape(String),
}

/// Checks the format version, before any other field is looked at (spec HO-3).
fn format_version(fields: &Map<String, Value>, shown: &str) -> Result<(), Fault> {
    let stated = fields
        .get("formatVersion")
        .ok_or_else(|| Fault::Shape("`formatVersion` is missing".to_owned()))?
        .as_f64()
        .filter(|number| number.fract() == 0.0)
        .ok_or_else(|| Fault::Shape("`formatVersion` must be a whole number".to_owned()))?;
    let reads = HAND_OFF_FORMATS.map(|format| format.to_string()).join(", ");
    let oldest = f64::from(HAND_OFF_FORMATS[0]);
    let newest = f64::from(HAND_OFF_FORMATS[HAND_OFF_FORMATS.len() - 1]);

    if stated < oldest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, older than any this analyzer reads ({reads}); update \
             yak"
        ))));
    }

    if stated > newest {
        return Err(Fault::Stop(Stop::new(format!(
            "the hand-off file {shown} is format {stated}, newer than any this analyzer reads ({reads}); update \
             the analyzer's version pinned in the pack"
        ))));
    }

    Ok(())
}

/// A field that must be text holding an absolute path.
fn absolute_path(fields: &Map<String, Value>, name: &str) -> Result<PathBuf, String> {
    let text = fields
        .get(name)
        .ok_or_else(|| format!("`{name}` is missing"))?
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("`{name}` must be text that is not empty"))?;

    let path = PathBuf::from(text);

    if path.is_absolute() {
        Ok(path)
    } else {
        Err(format!("`{name}` must be an absolute path"))
    }
}

/// The object under `detection`, which allows two fields and no other.
fn detection(fields: &Map<String, Value>) -> Result<&Map<String, Value>, String> {
    let detection = fields
        .get("detection")
        .ok_or_else(|| "`detection` is missing".to_owned())?
        .as_object()
        .ok_or_else(|| "`detection` must be an object".to_owned())?;

    match detection
        .keys()
        .find(|key| !matches!(key.as_str(), "folders" | "truncated"))
    {
        Some(other) => Err(format!("`detection.{other}` is not a field of the format")),
        None => Ok(detection),
    }
}

/// Checks the folders yak's detection looked in: at least one, the first `.`, each inside the root.
fn folders(fields: &Map<String, Value>) -> Result<(), String> {
    let listed = detection(fields)?
        .get("folders")
        .ok_or_else(|| "`detection.folders` is missing".to_owned())?
        .as_array()
        .filter(|listed| !listed.is_empty())
        .ok_or_else(|| "`detection.folders` must be a list of at least one path".to_owned())?;

    for (index, folder) in listed.iter().enumerate() {
        let folder = folder
            .as_str()
            .filter(|folder| !folder.is_empty())
            .ok_or_else(|| {
                format!("`detection.folders[{index}]` must be text that is not empty")
            })?;

        if leaves_its_folder(folder) || folder.contains('\\') {
            return Err(format!(
                "`detection.folders[{index}]` must be a path inside the root, written with forward slashes"
            ));
        }
    }

    if listed[0] == "." {
        Ok(())
    } else {
        Err("`detection.folders` must begin with `.`, the root itself".to_owned())
    }
}

/// Whether yak's detection stopped at its limit.
fn truncated(fields: &Map<String, Value>) -> Result<bool, String> {
    detection(fields)?
        .get("truncated")
        .ok_or_else(|| "`detection.truncated` is missing".to_owned())?
        .as_bool()
        .ok_or_else(|| "`detection.truncated` must be true or false".to_owned())
}

/// The stacks, each of which allows four fields and no other.
fn stacks(fields: &Map<String, Value>) -> Result<Vec<Stack>, String> {
    let listed = fields
        .get("stacks")
        .ok_or_else(|| "`stacks` is missing".to_owned())?
        .as_array()
        .ok_or_else(|| "`stacks` must be a list".to_owned())?;
    let mut stacks = Vec::with_capacity(listed.len());

    for (index, stack) in listed.iter().enumerate() {
        let stack = stack
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}]` must be an object"))?;
        let name = stack
            .get("name")
            .ok_or_else(|| format!("`stacks[{index}].name` is missing"))?
            .as_str()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| format!("`stacks[{index}].name` must be text that is not empty"))?;
        let interface_version = match stack.get("interfaceVersion") {
            Some(version) => Some(
                version
                    .as_u64()
                    .filter(|version| (1..=LARGEST_NUMBER).contains(version))
                    .ok_or_else(|| {
                        format!(
                            "`stacks[{index}].interfaceVersion` must be a positive whole number"
                        )
                    })?,
            ),
            None => None,
        };
        let pack = stack
            .get("pack")
            .ok_or_else(|| format!("`stacks[{index}].pack` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].pack` must be an object"))?;
        let project = stack
            .get("project")
            .ok_or_else(|| format!("`stacks[{index}].project` is missing"))?
            .as_object()
            .ok_or_else(|| format!("`stacks[{index}].project` must be an object"))?;

        if let Some((key, _)) = project.iter().find(|(_, value)| !is_project_value(value)) {
            return Err(format!(
                "`stacks[{index}].project.{key}` must be a list of text or a positive whole number"
            ));
        }

        if let Some(other) = stack.keys().find(|key| {
            !matches!(
                key.as_str(),
                "name" | "interfaceVersion" | "pack" | "project"
            )
        }) {
            return Err(format!(
                "`stacks[{index}].{other}` is not a field of the format"
            ));
        }

        stacks.push(Stack {
            name: name.to_owned(),
            interface_version,
            pack: pack.clone(),
            project: project.clone(),
        });
    }

    Ok(stacks)
}

/// Whether a value is one the format lets a project state: a list of text, or a positive whole number.
fn is_project_value(value: &Value) -> bool {
    match value {
        Value::Array(entries) => entries.iter().all(Value::is_string),
        other => other
            .as_u64()
            .is_some_and(|number| (1..=LARGEST_NUMBER).contains(&number)),
    }
}

/// Checks the base commit, where the file states one: a commit's full name in lower-case hexadecimal
/// digits.
fn base(fields: &Map<String, Value>) -> Result<(), String> {
    let Some(base) = fields.get("base") else {
        return Ok(());
    };

    base.as_str()
        .filter(|name| {
            COMMIT_NAME_LENGTHS.contains(&name.len())
                && name
                    .bytes()
                    .all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
        })
        .map(|_| ())
        .ok_or_else(|| {
            "`base` must be a commit's full name, 40 or 64 hexadecimal digits".to_owned()
        })
}

/// Checks that a stack's declarations were written for the interface version this analyzer speaks, and
/// that a stack which states no version holds no declarations (spec HO-4).
fn interface_version(stack: &Stack, shown: &str) -> Result<(), Stop> {
    let name = &stack.name;

    match stack.interface_version {
        Some(INTERFACE_VERSION) => Ok(()),
        Some(stated) => {
            let update = if stated > INTERFACE_VERSION {
                "the analyzer's version pinned in the pack"
            } else {
                "the stack's declarations"
            };

            Err(Stop::new(format!(
                "the stack `{name}` in {shown} states declarations for interface version {stated}, and this \
                 analyzer speaks version {INTERFACE_VERSION}; update {update}"
            )))
        }
        None if stack.pack.is_empty() && stack.project.is_empty() => Ok(()),
        None => Err(Stop::new(format!(
            "the stack `{name}` in {shown} holds declarations and states no interface version; state the \
             version they were written for"
        ))),
    }
}

/// Whether a path written with forward slashes is absolute or steps out of its folder with `..`.
pub fn leaves_its_folder(path: &str) -> bool {
    let has_drive = path.as_bytes().get(1) == Some(&b':');

    path.starts_with('/') || has_drive || path.split('/').any(|segment| segment == "..")
}
