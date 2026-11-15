//! What `audit` reports: the findings a reading shows, each with its rule, its message, its location
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::imports::{Form, Import};
use crate::notice::Notice;
use crate::reading::Reading;
use crate::resolve::Outcome;
use crate::scripts;

/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
const FINGERPRINT_DIGITS: usize = 32;

/// How many files of a loop a circular dependency's message names before it says how many more there
/// are (spec KIND-4).
const LOOP_NAMED: usize = 10;

/// The rule a finding is of, in the order the report lists them (spec REPORT-1, REPORT-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// A script or a Vue component that no entry point reaches.
    UnusedFile,
    /// A value a reachable file exports that no other file uses.
    UnusedExport,
    /// A type a reachable file exports that no other file uses.
    UnusedType,
    /// An import that leads to no file.
    UnresolvedImport,
    /// Files that import each other in a loop.
    CircularDependency,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 5] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
    ];

    /// The rule's name in the report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedFile => "unused-file",
            Self::UnusedExport => "unused-export",
            Self::UnusedType => "unused-type",
            Self::UnresolvedImport => "unresolved-import",
            Self::CircularDependency => "circular-dependency",
        }
    }

    /// What the rule finds, as one sentence.
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnusedFile => "A script or a Vue component that no entry point reaches.",
            Self::UnusedExport => "A value a file exports that no other file uses.",
            Self::UnusedType => "A type a file exports that no other file uses.",
            Self::UnresolvedImport => "An import that leads to no file.",
            Self::CircularDependency => "Files that import each other in a loop.",
        }
    }

    /// The rule's place in the report's list.
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|rule| *rule == self)
            .unwrap_or_default()
    }
}

/// Where a finding is: a file, and a line and a column in it where the kind has them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    /// The file, by its path from the root.
    pub path: String,
    /// The line, counted from one.
    pub line: Option<usize>,
    /// The column, counted from one in UTF-16 code units.
    pub column: Option<usize>,
}

/// Whether a finding is new on the branch or the base has it too (spec REPORT-1, INH-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Baseline {
    /// The branch made it.
    New,
    /// The base has it too: it is inherited.
    Unchanged,
}

impl Baseline {
    /// Its name as SARIF's `baselineState` writes it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Unchanged => "unchanged",
        }
    }
}

/// One finding.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    /// Its rule.
    pub rule: Rule,
    /// What a person can act on, as sentences.
    pub message: String,
    /// Where it is.
    pub location: Location,
    /// The other places it involves.
    pub related: Vec<Location>,
    /// The digest that names the same finding on the base and on the branch (spec REPORT-3).
    pub fingerprint: String,
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
}

impl Finding {
    /// A finding, its fingerprint made from its rule and the parts given.
    fn new(rule: Rule, message: String, location: Location, parts: &[&str]) -> Self {
        Self {
            rule,
            message,
            location,
            related: Vec::new(),
            fingerprint: fingerprint(rule, parts),
            baseline: None,
        }
    }
}

/// The fingerprint of a finding: the first digits of the SHA-256 digest of its rule and its parts, each
/// written after its length in bytes and a colon, so that no two lists of parts give the same input
/// (spec REPORT-3).
fn fingerprint(rule: Rule, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for part in std::iter::once(rule.name()).chain(parts.iter().copied()) {
        hasher.update(format!("{}:", part.len()).as_bytes());
        hasher.update(part.as_bytes());
    }

    let mut digits = String::with_capacity(FINGERPRINT_DIGITS);

    for byte in hasher.finalize().iter().take(FINGERPRINT_DIGITS / 2) {
        // Writing into a String cannot fail.
        let _ = write!(digits, "{byte:02x}");
    }

    digits
}

/// The findings of a reading, in the report's order, and the notices that come with them.
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = Vec::new();
    let mut notices = Vec::new();

    unused(reading, &mut found);
    unresolved(reading, &mut found);
    circular(reading, &mut found);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);

    sort(&mut found);

    (found, notices)
}

/// Puts findings in the report's order: by rule, then path, line and column, then fingerprint (spec
/// REPORT-1).
pub fn sort(found: &mut [Finding]) {
    found.sort_by(|a, b| {
        (a.rule, &a.location, &a.fingerprint).cmp(&(b.rule, &b.location, &b.fingerprint))
    });
}

/// Whether the three kinds that depend on entry points are reported for a file: its package's own
/// folder is a detection folder (spec ENTRY-9).
fn reported(reading: &Reading, path: &str) -> bool {
    reading.packages.of_file.get(path).is_some_and(|folder| {
        reading
            .packages
            .list
            .iter()
            .any(|package| package.folder == *folder && package.detected)
    })
}

/// Whether a file is the project's own code to report on: read, and neither data nor generated.
fn own(reading: &Reading, file: &File) -> bool {
    !reading.marks.data.contains(&file.path)
        && !reading.marks.generated.contains(&file.path)
        && !reading.scripts.gone.contains(&file.path)
}

/// The unused files, exports and types (spec KIND-1, KIND-2).
fn unused(reading: &Reading, found: &mut Vec<Finding>) {
    let used: BTreeSet<(&str, Option<&str>)> = reading
        .uses
        .list
        .iter()
        .map(|used| (used.file.as_str(), used.name.as_deref()))
        .collect();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || !own(reading, file)
            || !reported(reading, &file.path)
        {
            continue;
        }

        let path = file.path.as_str();

        if !reading.uses.reached.contains_key(path) {
            found.push(Finding::new(
                Rule::UnusedFile,
                unused_file(reading, path),
                Location {
                    path: path.to_owned(),
                    line: None,
                    column: None,
                },
                &[path],
            ));

            // The unused-file finding covers its exports.
            continue;
        }

        let Some(read) = reading.scripts.read.get(path) else {
            continue;
        };

        for export in &read.exports {
            let name = export.name.as_str();

            if used.contains(&(path, None)) || used.contains(&(path, Some(name))) {
                continue;
            }

            let (rule, what) = if export.is_type {
                (Rule::UnusedType, "type")
            } else {
                (Rule::UnusedExport, "export")
            };
            let message = if export.used_in_file {
                format!(
                    "The {what} `{name}` is used only inside its own file, so its `export` can be dropped."
                )
            } else {
                format!("The {what} `{name}` is used nowhere, so it can be removed.")
            };

            found.push(Finding::new(
                rule,
                message,
                Location {
                    path: path.to_owned(),
                    line: Some(export.line),
                    column: Some(export.column),
                },
                &[path, name],
            ));
        }
    }
}

/// The message of an unused file: that it can go, and, in a package whose entry points are likely
/// incomplete, that the likely fault is a missing `entries` declaration (spec ENTRY-8).
fn unused_file(reading: &Reading, path: &str) -> String {
    let mut message =
        "No entry point reaches this file through any import, so it can be removed.".to_owned();
    let fault = reading.packages.of_file.get(path).and_then(|folder| {
        reading
            .entries
            .faults
            .get(folder)
            .map(|fault| (folder, fault))
    });

    if let Some((folder, fault)) = fault {
        let why = match fault {
            Fault::NoEntryPoint => "has no entry point at all",
            Fault::UnmappedTarget => "has a `bin` or `exports` target that maps to no source",
        };

        let _ = write!(
            message,
            " Its package `{folder}` {why}, so the likely fault is a missing `entries` declaration."
        );
    }

    message
}

/// One notice for each package that holds files the analyzer reads and whose folder is not a detection
/// folder, and one for the files in no package: the three kinds that depend on entry points are not
/// reported there (spec ENTRY-9).
fn not_reported(reading: &Reading, notices: &mut Vec<Notice>) {
    let analyzed = || {
        reading
            .files
            .iter()
            .filter(|file| scripts::reads(file.kind) && own(reading, file))
    };

    for package in reading
        .packages
        .list
        .iter()
        .filter(|package| !package.detected)
    {
        let holds = analyzed()
            .any(|file| reading.packages.of_file.get(&file.path) == Some(&package.folder));

        if holds {
            notices.push(Notice::new(
                "kinds-not-reported",
                None,
                format!(
                    "The package `{}` is not in a folder yak's detection looked in, so its unused files, \
                     exports and types are not reported.",
                    package.folder
                ),
            ));
        }
    }

    let outside = analyzed()
        .filter(|file| !reading.packages.of_file.contains_key(&file.path))
        .count();

    if outside > 0 {
        notices.push(Notice::new(
            "kinds-not-reported",
            None,
            format!(
                "{outside} file(s) the analyzer reads are in no package, so their unused files, exports and \
                 types are not reported."
            ),
        ));
    }
}

/// The kind of each listed file, by its path.
fn kinds(reading: &Reading) -> BTreeMap<&str, Kind> {
    reading
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.kind))
        .collect()
}

/// The imports that lead to no file, in any file under the root that is read and is neither generated
/// nor a type declaration file (spec KIND-3).
fn unresolved(reading: &Reading, found: &mut Vec<Finding>) {
    let kinds = kinds(reading);

    for (path, read) in &reading.scripts.read {
        if kinds.get(path.as_str()) == Some(&Kind::Declaration)
            || reading.marks.generated.contains(path)
        {
            continue;
        }

        let Some(resolutions) = reading.resolved.get(path) else {
            continue;
        };

        for (import, resolution) in read.imports.iter().zip(resolutions) {
            if resolution.outcome == Outcome::Unresolved {
                found.push(Finding::new(
                    Rule::UnresolvedImport,
                    format!(
                        "The import of `{}` leads to no file, no package and no built-in module.",
                        import.specifier
                    ),
                    Location {
                        path: path.clone(),
                        line: Some(import.line),
                        column: None,
                    },
                    &[path, &import.specifier],
                ));
            }
        }
    }
}

/// Whether an import loads its target when the script runs, and so is an edge of a cycle: a static
/// import, `export … from`, `import … = require` or `require()`, not of types alone, and not read as
/// text (spec KIND-4, USE-6).
fn loads(import: &Import) -> bool {
    matches!(
        import.form,
        Form::Import | Form::Export | Form::ExportAll | Form::ImportEquals | Form::Require
    ) && !import.type_only
        && !import.has_query()
}

/// For each file, by its place in a list of files, each file it loads, by its place, with the line of
/// its first import that does and that import's place among its imports.
type Edges = Vec<BTreeMap<usize, (usize, usize)>>;

/// The edges cycles are found over: for each file, by its place in the list of files, each file it
/// loads, by its place, with the line of its first import that does, and that import's place. A type
/// declaration file loads nothing and is loaded by nothing (spec KIND-4).
fn edges(reading: &Reading) -> (Vec<&str>, Edges) {
    let kinds = kinds(reading);
    let nodes: Vec<&str> = reading
        .scripts
        .read
        .keys()
        .map(String::as_str)
        .filter(|path| kinds.get(path) != Some(&Kind::Declaration))
        .collect();
    let places: BTreeMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(place, path)| (*path, place))
        .collect();
    let mut edges = vec![BTreeMap::new(); nodes.len()];

    for (from, path) in nodes.iter().enumerate() {
        let (Some(read), Some(resolutions)) =
            (reading.scripts.read.get(*path), reading.resolved.get(*path))
        else {
            continue;
        };

        for (index, (import, resolution)) in read.imports.iter().zip(resolutions).enumerate() {
            let Outcome::Files(targets) = &resolution.outcome else {
                continue;
            };

            if !loads(import) {
                continue;
            }

            for target in targets {
                if let Some(to) = places.get(target.as_str()) {
                    let first: &mut (usize, usize) =
                        edges[from].entry(*to).or_insert((import.line, index));

                    *first = (*first).min((import.line, index));
                }
            }
        }
    }

    (nodes, edges)
}

/// The strongly connected sets of a graph, each in the order of its nodes, found by Tarjan's walk
/// written as a loop over a stack of its own, so that no chain of files is too long for the stack (spec
/// LIFE-3).
fn strongly_connected(edges: &Edges) -> Vec<Vec<usize>> {
    let unvisited = usize::MAX;
    let targets: Vec<Vec<usize>> = edges
        .iter()
        .map(|from| from.keys().copied().collect())
        .collect();
    let mut index = vec![unvisited; edges.len()];
    let mut low = vec![0; edges.len()];
    let mut held = vec![false; edges.len()];
    let mut stack: Vec<usize> = Vec::new();
    let mut next = 0;
    let mut sets = Vec::new();

    for start in 0..edges.len() {
        if index[start] != unvisited {
            continue;
        }

        // Each node being walked, with how many of its edges have been followed.
        let mut walk: Vec<(usize, usize)> = vec![(start, 0)];

        index[start] = next;
        low[start] = next;
        next += 1;
        stack.push(start);
        held[start] = true;

        while let Some(&(node, followed)) = walk.last() {
            if let Some(target) = targets[node].get(followed).copied() {
                if let Some(top) = walk.last_mut() {
                    top.1 += 1;
                }

                if index[target] == unvisited {
                    index[target] = next;
                    low[target] = next;
                    next += 1;
                    stack.push(target);
                    held[target] = true;
                    walk.push((target, 0));
                } else if held[target] {
                    low[node] = low[node].min(index[target]);
                }

                continue;
            }

            walk.pop();

            if let Some(&(parent, _)) = walk.last() {
                low[parent] = low[parent].min(low[node]);
            }

            if low[node] == index[node] {
                let mut set = Vec::new();

                while let Some(member) = stack.pop() {
                    held[member] = false;
                    set.push(member);

                    if member == node {
                        break;
                    }
                }

                set.sort_unstable();
                sets.push(set);
            }
        }
    }

    sets
}

/// The circular dependencies: each strongly connected set of two or more files, or a file that loads
/// itself, as one finding (spec KIND-4).
fn circular(reading: &Reading, found: &mut Vec<Finding>) {
    let (nodes, edges) = edges(reading);

    for set in strongly_connected(&edges) {
        let first = set[0];

        if set.len() == 1 && !edges[first].contains_key(&first) {
            continue;
        }

        let members: BTreeSet<usize> = set.iter().copied().collect();
        // Where a file of the set is: its first import, by line, that loads another file of the set, or
        // itself in a set of one.
        let placed = |node: usize| Location {
            path: nodes[node].to_owned(),
            line: edges[node]
                .iter()
                .filter(|(to, _)| members.contains(to) && (set.len() == 1 || **to != node))
                .map(|(_, (line, index))| (*line, *index))
                .min()
                .map(|(line, _)| line),
            column: None,
        };
        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let mut finding = Finding::new(
            Rule::CircularDependency,
            looped(&nodes, &edges, &members, first),
            placed(first),
            &paths,
        );

        finding.related = set[1..].iter().map(|node| placed(*node)).collect();
        found.push(finding);
    }
}

/// The message of a circular dependency: the shortest loop through the set's first file, found by a
/// walk in breadth that takes each file's targets in the order of their paths, up to ten files and
/// then how many more; and how many files the set holds, where the loop is not all of them (spec
/// KIND-4).
fn looped(nodes: &[&str], edges: &Edges, members: &BTreeSet<usize>, first: usize) -> String {
    let mut before: BTreeMap<usize, usize> = BTreeMap::new();
    let mut queue = VecDeque::from([first]);
    let mut last = first;

    'walk: while let Some(node) = queue.pop_front() {
        for target in edges[node].keys().filter(|to| members.contains(to)) {
            if *target == first {
                last = node;

                break 'walk;
            }

            if !before.contains_key(target) {
                before.insert(*target, node);
                queue.push_back(*target);
            }
        }
    }

    let mut cycle = vec![last];

    while let Some(node) = cycle.last().and_then(|node| before.get(node)) {
        cycle.push(*node);
    }

    cycle.reverse();

    let mut named: Vec<String> = cycle
        .iter()
        .take(LOOP_NAMED)
        .map(|node| format!("`{}`", nodes[*node]))
        .collect();

    if cycle.len() > LOOP_NAMED {
        named.push(format!("{} more", cycle.len() - LOOP_NAMED));
    }

    named.push(format!("`{}`", nodes[first]));

    let mut message = format!(
        "These files import each other in a loop: {}.",
        named.join(" → ")
    );

    if members.len() > cycle.len() {
        let _ = write!(
            message,
            " The loop is part of a set of {} files that each reach all the others.",
            members.len()
        );
    }

    message
}

/// One notice for each file with an `import()` or `require()` it cannot follow: what it loads must be
/// declared (spec ENTRY-10).
fn computed(reading: &Reading, notices: &mut Vec<Notice>) {
    for (path, read) in &reading.scripts.read {
        if read.computed > 0 {
            notices.push(Notice::new(
                "computed-import",
                Some(path.clone()),
                format!(
                    "{} call(s) of `import()` or `require()` load a file the analyzer cannot work out, so \
                     what they load must be declared in `entries`.",
                    read.computed
                ),
            ));
        }
    }
}
