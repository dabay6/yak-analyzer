//! Uses and reach: which export of which file is used, and from where; and which files the entry
//! points reach (spec USE-2 to USE-6).
//!
//! A use begins at a reference in a file, at an import that uses every export of its target, or at an
//! entry point, whose exports whatever starts it consumes. From there it is passed on through each
//! `export … from`, `export *` and imported binding exported again, to the file that declares the
//! name. Every walk here is a loop over a list of work, so no chain of files is too long for the stack
//! (spec LIFE-3).

use std::collections::{BTreeMap, BTreeSet};

use crate::entries::{Entries, Source};
use crate::imports::{DEFAULT, EVERY, Form, Read};
use crate::resolve::{Outcome, Resolution};
use crate::scripts::Scripts;

/// Where a use counts from (spec CMD-4).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// A reference in a file, or an import that uses every export, at a line.
    Reference { file: String, line: usize },
    /// An entry point, whose exports whatever starts it consumes, with the source that made it one.
    Entry(Source),
}

/// One use of an export.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Use {
    /// The file whose export is used.
    pub file: String,
    /// The export's name, or none for every export of the file.
    pub name: Option<String>,
    /// Where it counts from.
    pub origin: Origin,
}

/// How far a file is reached from the entry points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// By an entry point's source or an import: its own imports are followed.
    Code,
    /// Only by imports that carry a query, which read it as text: it loads nothing (spec USE-6).
    Text,
}

/// The uses of a project and what its entry points reach.
#[derive(Debug, Default)]
pub struct Uses {
    /// Every use that was counted, in order.
    pub list: BTreeSet<Use>,
    /// Each file an entry point reaches, by path, with how far.
    pub reached: BTreeMap<String, Reach>,
}

/// What a file offers under the names it exports.
#[derive(Default)]
struct Offer<'s> {
    /// The names it declares itself.
    own: BTreeSet<&'s str>,
    /// The names it passes on, each with the import it comes through and the target's name for it,
    /// none for the target's namespace.
    passed: BTreeMap<&'s str, Vec<(usize, Option<&'s str>)>>,
    /// The imports whose every name it passes on under the same name: `export * from`.
    stars: Vec<usize>,
}

impl<'s> Offer<'s> {
    /// What a file that was read offers.
    fn of(read: &'s Read) -> Self {
        let mut offer = Self {
            own: read
                .exports
                .iter()
                .map(|export| export.name.as_str())
                .collect(),
            ..Self::default()
        };

        for (index, import) in read.imports.iter().enumerate() {
            for name in &import.names {
                let from = (name.name != EVERY).then_some(name.name.as_str());

                for exported in &name.exported {
                    offer
                        .passed
                        .entry(exported)
                        .or_default()
                        .push((index, from));
                }

                if import.form == Form::ExportAll && name.exported.is_empty() {
                    offer.stars.push(index);
                }
            }
        }

        offer
    }
}

/// What of a file a use is of.
#[derive(Clone, Copy)]
enum Wanted<'s> {
    /// Every export.
    Every,
    /// Every name an `export *` passes on: every export but `default`.
    Starred,
    /// One export, by its name.
    One(&'s str),
}

impl<'s> Wanted<'s> {
    /// What a name passed on is a use of in the file it comes from: that file's name for it, or its
    /// every export where the name is its namespace.
    fn through(from: Option<&'s str>) -> Self {
        from.map_or(Self::Every, Self::One)
    }
}

/// One use still to be followed: the file it reaches, what of it, where it counts from by its place in
/// the list of origins, and whether it reads the file as text.
struct Pending<'s> {
    file: &'s str,
    wanted: Wanted<'s>,
    origin: usize,
    text: bool,
}

/// Finds every use and what the entry points reach.
pub fn find(
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
    entries: &Entries,
) -> Uses {
    Uses {
        list: uses(scripts, resolved, entries),
        reached: reached(scripts, resolved, entries),
    }
}

/// The files an import of a file leads to.
fn targets<'s>(
    resolved: &'s BTreeMap<String, Vec<Resolution>>,
    file: &str,
    import: usize,
) -> &'s [String] {
    match resolved.get(file).and_then(|imports| imports.get(import)) {
        Some(Resolution {
            outcome: Outcome::Files(files),
            ..
        }) => files,
        _ => &[],
    }
}

/// The uses that begin in the files and at the entry points, each with where it counts from: what an
/// import uses of each file it leads to, what a file's own template names, and every export of an
/// entry point (spec USE-2, USE-3, USE-4, USE-6).
fn begun<'s>(
    scripts: &'s Scripts,
    resolved: &'s BTreeMap<String, Vec<Resolution>>,
    entries: &'s Entries,
) -> (Vec<Origin>, Vec<Pending<'s>>) {
    let mut origins: Vec<Origin> = Vec::new();
    let mut work: Vec<Pending<'s>> = Vec::new();

    for (path, read) in &scripts.read {
        for (index, import) in read.imports.iter().enumerate() {
            // A file imported with a query is read as text: its exports are never reported, and it
            // passes nothing on (spec USE-6).
            let text = import.has_query();
            let uses: Vec<(Wanted<'s>, usize)> = if text {
                vec![(Wanted::Every, import.line)]
            } else {
                import
                    .uses
                    .iter()
                    .map(|used| (Wanted::through(used.name.as_deref()), used.line))
                    .collect()
            };

            for (wanted, line) in uses {
                origins.push(Origin::Reference {
                    file: path.clone(),
                    line,
                });

                for file in targets(resolved, path, index) {
                    work.push(Pending {
                        file,
                        wanted,
                        origin: origins.len() - 1,
                        text,
                    });
                }
            }
        }

        for used in &read.named {
            origins.push(Origin::Reference {
                file: path.clone(),
                line: used.line,
            });
            work.push(Pending {
                file: path,
                wanted: Wanted::through(used.name.as_deref()),
                origin: origins.len() - 1,
                text: false,
            });
        }
    }

    for (path, sources) in &entries.points {
        for source in sources {
            origins.push(Origin::Entry(source.clone()));
            work.push(Pending {
                file: path,
                wanted: Wanted::Every,
                origin: origins.len() - 1,
                text: false,
            });
        }
    }

    (origins, work)
}

/// Every use, passed on to the file that declares the name (spec USE-2, USE-2a, USE-3).
fn uses(
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
    entries: &Entries,
) -> BTreeSet<Use> {
    let offers: BTreeMap<&str, Offer<'_>> = scripts
        .read
        .iter()
        .map(|(path, read)| (path.as_str(), Offer::of(read)))
        .collect();
    let (origins, mut work) = begun(scripts, resolved, entries);
    // What was counted, and what was only looked through on the way to the names behind an `export *`.
    let mut counted: BTreeSet<(&str, Option<&str>, usize)> = BTreeSet::new();
    let mut looked: BTreeSet<(&str, &str, usize)> = BTreeSet::new();

    while let Some(Pending {
        file,
        wanted,
        origin,
        text,
    }) = work.pop()
    {
        // A file that was not read has no exports.
        let Some(offer) = offers.get(file) else {
            continue;
        };
        // The imports the use is passed on through, each with what it is a use of in the import's
        // target.
        let mut on: Vec<(usize, Wanted<'_>)> = Vec::new();
        let starred = |wanted| offer.stars.iter().map(move |import| (*import, wanted));

        match wanted {
            Wanted::Every => {
                if counted.insert((file, None, origin)) && !text {
                    on.extend(
                        offer
                            .passed
                            .values()
                            .flatten()
                            .map(|(import, from)| (*import, Wanted::through(*from))),
                    );
                    on.extend(starred(Wanted::Starred));
                }
            }
            Wanted::Starred => {
                if looked.insert((file, EVERY, origin)) {
                    for name in offer.own.iter().filter(|name| **name != DEFAULT) {
                        counted.insert((file, Some(name), origin));
                    }

                    for (name, passed) in &offer.passed {
                        if *name != DEFAULT && counted.insert((file, Some(name), origin)) {
                            on.extend(
                                passed
                                    .iter()
                                    .map(|(import, from)| (*import, Wanted::through(*from))),
                            );
                        }
                    }

                    on.extend(starred(Wanted::Starred));
                }
            }
            Wanted::One(name) => {
                if offer.own.contains(name) {
                    counted.insert((file, Some(name), origin));
                } else if let Some(passed) = offer.passed.get(name) {
                    if counted.insert((file, Some(name), origin)) {
                        on.extend(
                            passed
                                .iter()
                                .map(|(import, from)| (*import, Wanted::through(*from))),
                        );
                    }
                } else if name != DEFAULT && looked.insert((file, name, origin)) {
                    // `export *` passes every name on but `default`.
                    on.extend(starred(Wanted::One(name)));
                }
            }
        }

        for (import, wanted) in on {
            for target in targets(resolved, file, import) {
                work.push(Pending {
                    file: target,
                    wanted,
                    origin,
                    text: false,
                });
            }
        }
    }

    counted
        .into_iter()
        .map(|(file, name, origin)| Use {
            file: file.to_owned(),
            name: name.map(str::to_owned),
            origin: origins[origin].clone(),
        })
        .collect()
}

/// The files the entry points reach, through every import that leads to files, of every form. A file
/// that only imports with a query reach is read as text, and its own imports are not followed (spec
/// USE-6).
fn reached(
    scripts: &Scripts,
    resolved: &BTreeMap<String, Vec<Resolution>>,
    entries: &Entries,
) -> BTreeMap<String, Reach> {
    let mut reached: BTreeMap<String, Reach> = BTreeMap::new();
    let mut work: Vec<&str> = Vec::new();

    for path in entries.points.keys() {
        reached.insert(path.clone(), Reach::Code);
        work.push(path);
    }

    while let Some(path) = work.pop() {
        let Some(read) = scripts.read.get(path) else {
            continue;
        };

        for (index, import) in read.imports.iter().enumerate() {
            let reach = if import.has_query() {
                Reach::Text
            } else {
                Reach::Code
            };

            for target in targets(resolved, path, index) {
                match (reached.get(target), reach) {
                    (Some(Reach::Code), _) | (Some(Reach::Text), Reach::Text) => {}
                    (_, Reach::Code) => {
                        reached.insert(target.clone(), Reach::Code);
                        work.push(target);
                    }
                    (None, Reach::Text) => {
                        reached.insert(target.clone(), Reach::Text);
                    }
                }
            }
        }
    }

    reached
}
