//! The `graph` command: reads a project and prints what it read as one JSON document (spec OUT-1).
//!
//! So far it reads the hand-off file, the file list, the `data` and `generated` declarations, the
//! packages, the tsconfig files, the imports and the names of every script, where each import leads,
//! the entry points with the source of each, every use of an export, and the files the entry points
//! reach (spec CMD-4).

use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use serde_json::{Value, json};

use crate::declarations;
use crate::entries::{self, Entries};
use crate::files::{self, File};
use crate::hand_off;
use crate::imports::{EVERY, Export, Import, Name};
use crate::marks::{self, Marks};
use crate::notice::{self, Notice};
use crate::packages::{self, Package, Packages};
use crate::resolve::{Outcome, Place, Resolution, Resolver};
use crate::scripts::{self, Scripts};
use crate::stop::Stop;
use crate::tsconfig;
use crate::uses::{self, Origin, Reach, Use, Uses};

/// Reads the project, with the hand-off file at the path given or with none, and returns the document
/// to print.
pub fn read(hand_off_path: Option<&Path>) -> Result<String, Stop> {
    let mut notices = Vec::new();
    let handed = match hand_off_path {
        Some(path) => {
            let handed = hand_off::read(path)?;

            declarations::check(&handed.stacks)?;

            if handed.truncated {
                notices.push(Notice::new(
                    "detection-cut-short",
                    None,
                    "yak's detection stopped at its limit of folders, so the folders the hand-off file lists \
                     are not the whole workspace."
                        .to_owned(),
                ));
            }

            Some(handed)
        }
        None => {
            notices.push(Notice::new(
                "no-hand-off-file",
                None,
                "No hand-off file was given, so the project is read from its own standard files, with no \
                 declarations."
                    .to_owned(),
            ));

            None
        }
    };

    // The root is the folder the hand-off file names, whatever folder the analyzer was started in, and
    // without a hand-off file the folder it was started in (spec LIST-1).
    let root = match &handed {
        Some(handed) => handed.root.clone(),
        None => env::current_dir().map_err(|error| {
            Stop::new(format!(
                "the folder the analyzer was started in cannot be read ({error}); start it in a checkout"
            ))
        })?,
    };
    let listing = files::list(&root, &mut notices)?;
    let files = listing.files.as_slice();
    let marks = marks::apply(files, handed.as_ref(), &mut notices)?;
    // The `package.json` files are read before the tsconfig files, and the fields of both are checked
    // after, so that two faults always give the same line (spec RUN-4).
    let packages = packages::load(&root, files, &marks.data)?.checked(
        files,
        handed.as_ref().map(|handed| handed.folders.as_slice()),
        &mut notices,
    );
    let configs = tsconfig::load(
        &root,
        files,
        &marks.data,
        packages.as_ref().ok(),
        &mut notices,
    )?
    .checked();
    let (packages, configs) = match (packages, configs) {
        (Ok(packages), Ok(configs)) => (packages, configs),
        (Err(fault), Ok(_)) | (Ok(_), Err(fault)) => return Err(fault.stop()),
        (Err(first), Err(second)) => return Err(first.earlier(second).stop()),
    };
    let scripts = scripts::read(&root, files, &marks.data, &mut notices)?;
    let resolver = Resolver::new(&listing, &packages, &configs);
    let resolved = resolver.resolve(&root, &scripts)?;
    let entries = entries::find(
        &entries::Project {
            root: &root,
            files,
            packages: &packages,
            configs: &configs,
            resolver: &resolver,
            resolved: &resolved,
            handed: handed.as_ref(),
        },
        &mut notices,
    )?;
    let uses = uses::find(&scripts, &resolved, &entries);
    let mut document = json!({
        "entries": entry_points(&entries),
        "entryFaults": entries
            .faults
            .iter()
            .map(|(folder, fault)| json!({ "fault": fault.name(), "package": folder }))
            .collect::<Vec<Value>>(),
        "files": files
            .iter()
            .filter(|each| !scripts.gone.contains(&each.path))
            .map(|each| file(each, &marks, &packages, &scripts, &resolved))
            .collect::<Vec<Value>>(),
        "handOff": handed.is_some(),
        "notices": notice::printed(notices),
        "packages": packages.list.iter().map(package).collect::<Vec<Value>>(),
        "reachable": reachable(&uses),
        "uses": uses.list.iter().map(used).collect::<Vec<Value>>(),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// The entry points as they are printed, by path, each with every source that made it one (spec
/// CMD-4).
fn entry_points(entries: &Entries) -> Vec<Value> {
    entries
        .points
        .iter()
        .map(|(path, sources)| {
            json!({
                "path": path,
                "sources": sources.iter().map(entries::Source::printed).collect::<Vec<Value>>(),
            })
        })
        .collect()
}

/// The files the entry points reach, by path, each with whether it is reached only as text, by
/// imports that carry a query (spec USE-6).
fn reachable(uses: &Uses) -> Vec<Value> {
    uses.reached
        .iter()
        .map(|(path, reach)| json!({ "path": path, "text": *reach == Reach::Text }))
        .collect()
}

/// One use as it is printed: the file whose export is used, the export's name, `*` for every export,
/// and where it counts from, the file and line of a reference or an entry point's source (spec CMD-4).
fn used(used: &Use) -> Value {
    let (by, line, entry) = match &used.origin {
        Origin::Reference { file, line } => (Some(file), Some(line), None),
        Origin::Entry(source) => (None, None, Some(source.printed())),
    };

    json!({
        "by": by,
        "entry": entry,
        "file": used.file,
        "line": line,
        "name": used.name.as_deref().unwrap_or(EVERY),
    })
}

/// One file as it is printed. A file that was read has its imports, each with where it leads, the
/// count of those that cannot be followed, and its own exports.
fn file(
    file: &File,
    marks: &Marks,
    packages: &Packages,
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
) -> Value {
    let mut printed = json!({
        "data": marks.data.contains(&file.path),
        "generated": marks.generated.contains(&file.path),
        "kind": file.kind.name(),
        "package": packages.of_file.get(&file.path),
        "path": file.path,
        "tracked": file.tracked,
    });

    if let (Some(read), Some(resolutions)) =
        (scripts.read.get(&file.path), resolved.get(&file.path))
    {
        printed["computedImports"] = json!(read.computed);
        printed["exports"] = read.exports.iter().map(export).collect();
        printed["imports"] = read
            .imports
            .iter()
            .zip(resolutions)
            .map(|(each, resolution)| import(each, resolution))
            .collect();
    }

    printed
}

/// One export of a file as it is printed: its name, whether it is a value or a type, where its name
/// is written, and whether its own file refers to it (spec USE-1, USE-2).
fn export(export: &Export) -> Value {
    json!({
        "column": export.column,
        "kind": if export.is_type { "type" } else { "value" },
        "line": export.line,
        "name": export.name,
        "usedInFile": export.used_in_file,
    })
}

/// One name an import brings in as it is printed: the target's name for it, `*` for the namespace,
/// its local binding, and the names the importing file passes it on under (spec USE-1, USE-2a).
fn name(name: &Name) -> Value {
    json!({ "exported": name.exported, "local": name.local, "name": name.name })
}

/// One import as it is printed, with where it leads and the rule that decided it (spec OUT-1).
fn import(import: &Import, resolution: &Resolution) -> Value {
    let mut printed = json!({
        "form": import.form.name(),
        "line": import.line,
        "names": import.names.iter().map(name).collect::<Vec<Value>>(),
        "query": import.has_query(),
        "rule": resolution.rule.name(),
        "specifier": import.specifier,
        "typeOnly": import.type_only,
    });

    match &resolution.outcome {
        Outcome::Files(files) => {
            printed["outcome"] = json!("files");
            printed["files"] = json!(files);
        }
        Outcome::Outside(place) => {
            printed["outcome"] = json!("outside");
            printed["outside"] = json!(place.name());

            if let Place::Package { name, listed } = place {
                printed["package"] = json!(name);
                printed["listed"] = json!(listed);
            }
        }
        Outcome::Ignored { path, from } => {
            printed["outcome"] = json!("ignored");
            printed["ignored"] = json!(path);

            if let Some(from) = from {
                printed["from"] = json!(from);
            }
        }
        Outcome::Unresolved => printed["outcome"] = json!("unresolved"),
    }

    printed
}

/// One package as it is printed.
fn package(package: &Package) -> Value {
    json!({
        "detected": package.detected,
        "folder": package.folder,
        "name": package.name,
        "workspace": package.workspace,
    })
}
