//! What `audit` reports: the findings a reading shows, each with its rule, its message, its location
//! and its fingerprint, and the notices that say where some kinds are not reported (spec KIND-1 to
//! KIND-5, ENTRY-8 to ENTRY-10, REPORT-2, REPORT-3; duplication spec DUP-1, DUP-2).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::complexity::{self, Detail, Scores};
use crate::duplication::{self, Group, Place};
use crate::entries::Fault;
use crate::files::{File, Kind};
use crate::imports::{Form, Import};
use crate::notice::Notice;
use crate::reading::Reading;
use crate::resolve::Outcome;
use crate::scripts;
use crate::uses::{self, Uses};

/// How many hexadecimal digits of the digest a fingerprint keeps (spec REPORT-3).
pub const FINGERPRINT_DIGITS: usize = 32;

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
    /// A function whose complexity is over a declared limit (complexity spec CPX-1).
    Complexity,
    /// A group of runs of statements that are the same (duplication spec DUP-1).
    Duplication,
}

impl Rule {
    /// Every rule, in the report's order.
    pub const ALL: [Self; 7] = [
        Self::UnusedFile,
        Self::UnusedExport,
        Self::UnusedType,
        Self::UnresolvedImport,
        Self::CircularDependency,
        Self::Complexity,
        Self::Duplication,
    ];

    /// The rule's name in the report.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedFile => "unused-file",
            Self::UnusedExport => "unused-export",
            Self::UnusedType => "unused-type",
            Self::UnresolvedImport => "unresolved-import",
            Self::CircularDependency => "circular-dependency",
            Self::Complexity => "complexity",
            Self::Duplication => "duplication",
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
            Self::Complexity => {
                "A function whose cyclomatic or cognitive complexity is over its limit."
            }
            Self::Duplication => "Runs of statements that are the same in two or more places.",
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
    /// The last line, where what it names runs over lines: a copy of a duplication group's (duplication
    /// spec DUP-2).
    pub end_line: Option<usize>,
}

impl From<&Place> for Location {
    /// Where a copy of a group is: its file, from its first line to its last (duplication spec DUP-2).
    fn from(copy: &Place) -> Self {
        Self {
            path: copy.path.clone(),
            line: Some(copy.start_line),
            column: None,
            end_line: Some(copy.end_line),
        }
    }
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
    /// What the fingerprint is made from, after the rule: paths first, as REPORT-3 lists them.
    parts: Vec<String>,
    /// Whether it is new or inherited, with a base; none without one.
    pub baseline: Option<Baseline>,
    /// What a complexity finding carries in its property bag; none for any other (complexity spec
    /// CPX-3).
    pub complexity: Option<Detail>,
    /// The group of a duplication finding, whose size its property bag carries; none for any other
    /// (duplication spec DUP-3).
    pub group: Option<Group>,
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
            parts: parts.iter().map(|part| (*part).to_owned()).collect(),
            baseline: None,
            complexity: None,
            group: None,
        }
    }

    /// A group's finding: at its first copy, with its other copies as related locations, and the
    /// fingerprint the group was given, which holds no path; none for a group with no copy (duplication
    /// spec DUP-1, DUP-2, DUP-4).
    fn grouped(group: Group) -> Option<Self> {
        let mut locations = group.copies.iter().map(Location::from);
        let location = locations.next()?;
        let related = locations.collect();

        Some(Self {
            rule: Rule::Duplication,
            message: duplication::message(&group),
            location,
            related,
            fingerprint: group.fingerprint.clone(),
            parts: Vec::new(),
            baseline: None,
            complexity: None,
            group: Some(group),
        })
    }

    /// The fingerprint the finding would have with each path it names moved as `moved` says, as a
    /// rename moves a base's file to its path on the branch (spec INH-2). A cycle's paths are sorted
    /// again after they move.
    pub fn moved(&self, moved: impl Fn(&str) -> String) -> String {
        let parts: Vec<String> = if self.rule == Rule::CircularDependency {
            let mut paths: Vec<String> = self.parts.iter().map(|path| moved(path)).collect();

            paths.sort();
            paths
        } else {
            // The first part is the path; a name or a specifier after it does not move.
            self.parts
                .iter()
                .enumerate()
                .map(|(place, part)| {
                    if place == 0 {
                        moved(part)
                    } else {
                        part.clone()
                    }
                })
                .collect()
        };
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

        fingerprint(self.rule, &parts)
    }

    /// The files of a circular dependency: its first location's, then its related locations'.
    pub fn files(&self) -> Vec<&str> {
        std::iter::once(&self.location)
            .chain(&self.related)
            .map(|location| location.path.as_str())
            .collect()
    }
}

/// The fingerprint of a finding: the first digits of the SHA-256 digest of its rule and its parts, each
/// written after its length in bytes and a colon, so that no two lists of parts give the same input
/// (spec REPORT-3; complexity spec CPX-4).
pub fn fingerprint(rule: Rule, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for part in std::iter::once(rule.name()).chain(parts.iter().copied()) {
        hasher.update(format!("{}:", part.len()).as_bytes());
        hasher.update(part.as_bytes());
    }

    hex(&hasher.finalize()[..FINGERPRINT_DIGITS / 2])
}

/// Bytes in lower-case hexadecimal digits, two to a byte.
pub fn hex(bytes: &[u8]) -> String {
    let mut digits = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        // Writing into a String cannot fail.
        let _ = write!(digits, "{byte:02x}");
    }

    digits
}

/// The findings of a reading, in the report's order, and the notices that come with them.
pub fn find(reading: &Reading) -> (Vec<Finding>, Vec<Notice>) {
    let mut found = counted(reading);
    let mut notices = Vec::new();
    // Each finding the `unreported` declaration left out, by the files that tie it to its globs
    // (ignoring spec COUNT-2).
    let mut left = left_dead(reading, &found);

    circular(reading, &mut found, &mut left);
    complex(reading, &mut found);
    duplicated(reading, &mut found, &mut notices);
    not_reported(reading, &mut notices);
    computed(reading, &mut notices);
    notices.extend(reading.limits.notice());
    notices.extend(reading.marks.counts(&left));

    sort(&mut found);

    (found, notices)
}

/// The findings of the kinds a base marks by counting their fingerprints: the unused files, exports and
/// types, and the unresolved imports (spec INH-1).
pub fn counted(reading: &Reading) -> Vec<Finding> {
    let mut found = Vec::new();

    unused(
        reading,
        &reading.uses,
        &reading.marks.unreported,
        &mut found,
    );
    unresolved(reading, &reading.marks.unreported, &mut found);

    found
}

/// The findings of the dead-code kinds the `unreported` declaration left out, each by its file: those
/// the run would report without it, worked out over the same reading from the entry points without the
/// reason `unreported` and with no file left out, that it does not report with it. None where the
/// declaration matches no file (ignoring spec COUNT-2).
fn left_dead(reading: &Reading, reported: &[Finding]) -> Vec<Vec<String>> {
    if reading.marks.unreported.is_empty() {
        return Vec::new();
    }

    let uses = uses::find(
        &reading.scripts,
        &reading.resolved,
        &reading.entries.declared(),
    );
    let none = BTreeSet::new();
    let mut plain = Vec::new();

    unused(reading, &uses, &none, &mut plain);
    unresolved(reading, &none, &mut plain);

    // How many findings of each fingerprint the run reports, which match as many it would report.
    let mut held: BTreeMap<&str, usize> = BTreeMap::new();

    for finding in reported {
        *held.entry(finding.fingerprint.as_str()).or_default() += 1;
    }

    plain
        .into_iter()
        .filter(|finding| {
            held.get_mut(finding.fingerprint.as_str())
                .filter(|count| **count > 0)
                .map(|count| *count -= 1)
                .is_none()
        })
        .map(|finding| vec![finding.location.path])
        .collect()
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

/// Whether a file is the project's own code to report on: read, neither data nor generated, and not one
/// of the files given, which are left out (ignoring spec READ-3).
fn own(reading: &Reading, file: &File, unreported: &BTreeSet<String>) -> bool {
    !reading.marks.data.contains(&file.path)
        && !reading.marks.generated.contains(&file.path)
        && !unreported.contains(&file.path)
        && !reading.scripts.gone.contains(&file.path)
}

/// The unused files, exports and types, by the uses given, in every file the analyzer reports on but
/// the files given (spec KIND-1, KIND-2; ignoring spec DEAD-1).
fn unused(reading: &Reading, uses: &Uses, unreported: &BTreeSet<String>, found: &mut Vec<Finding>) {
    let used: BTreeSet<(&str, Option<&str>)> = uses
        .list
        .iter()
        .map(|used| (used.file.as_str(), used.name.as_deref()))
        .collect();

    for file in &reading.files {
        if !matches!(file.kind, Kind::Script | Kind::Vue)
            || !own(reading, file, unreported)
            || !reported(reading, &file.path)
        {
            continue;
        }

        let path = file.path.as_str();

        if !uses.reached.contains_key(path) {
            found.push(Finding::new(
                Rule::UnusedFile,
                unused_file(reading, path),
                Location {
                    path: path.to_owned(),
                    line: None,
                    column: None,
                    end_line: None,
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
                    end_line: None,
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
        reading.files.iter().filter(|file| {
            scripts::reads(file.kind) && own(reading, file, &reading.marks.unreported)
        })
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
/// nor a type declaration file nor one of the files given (spec KIND-3; ignoring spec DEAD-1).
fn unresolved(reading: &Reading, unreported: &BTreeSet<String>, found: &mut Vec<Finding>) {
    let kinds = kinds(reading);

    for (path, read) in &reading.scripts.read {
        if kinds.get(path.as_str()) == Some(&Kind::Declaration)
            || reading.marks.generated.contains(path)
            || unreported.contains(path)
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
                        end_line: None,
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

/// The files cycles are found over, and which file loads which, by their paths (spec KIND-4, INH-3).
pub struct Graph<'r> {
    /// The files, in path order.
    nodes: Vec<&'r str>,
    /// What each file loads.
    edges: Edges,
}

impl<'r> Graph<'r> {
    /// The graph of a reading.
    pub fn of(reading: &'r Reading) -> Self {
        let (nodes, edges) = edges(reading);

        Self { nodes, edges }
    }

    /// Each pair of files where the first loads the second.
    pub fn pairs(&self) -> Vec<(&'r str, &'r str)> {
        self.edges
            .iter()
            .enumerate()
            .flat_map(|(from, targets)| {
                targets
                    .keys()
                    .map(move |to| (self.nodes[from], self.nodes[*to]))
            })
            .collect()
    }

    /// The strongly connected set each file is in, as the set's number, by the file's path.
    pub fn sets(&self) -> BTreeMap<&'r str, usize> {
        strongly_connected(&self.edges)
            .iter()
            .enumerate()
            .flat_map(|(number, set)| set.iter().map(move |node| (self.nodes[*node], number)))
            .collect()
    }

    /// The first import, by line, by which one file loads another: its line, and its place among the
    /// file's imports.
    pub fn first(&self, from: &str, to: &str) -> Option<(usize, usize)> {
        let from = self.nodes.binary_search(&from).ok()?;
        let to = self.nodes.binary_search(&to).ok()?;

        self.edges[from].get(&to).copied()
    }
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
/// itself, as one finding, at its first file by path that is not unreported; a set whose every file is
/// unreported is left out, by its files (spec KIND-4; ignoring spec CYCLE-1).
fn circular(reading: &Reading, found: &mut Vec<Finding>, left: &mut Vec<Vec<String>>) {
    let (nodes, edges) = edges(reading);

    for set in strongly_connected(&edges) {
        if set.len() == 1 && !edges[set[0]].contains_key(&set[0]) {
            continue;
        }

        let paths: Vec<&str> = set.iter().map(|node| nodes[*node]).collect();
        let Some(first) = set
            .iter()
            .copied()
            .find(|node| !reading.marks.unreported.contains(nodes[*node]))
        else {
            left.push(paths.iter().map(|path| (*path).to_owned()).collect());

            continue;
        };
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
            end_line: None,
        };
        let mut finding = Finding::new(
            Rule::CircularDependency,
            looped(&nodes, &edges, &members, first),
            placed(first),
            &paths,
        );

        finding.related = set
            .iter()
            .filter(|node| **node != first)
            .map(|node| placed(*node))
            .collect();
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

/// One finding for each function over a limit, whichever measures are over (complexity spec CPX-1 to
/// CPX-5).
fn complex(reading: &Reading, found: &mut Vec<Finding>) {
    let limits = reading.limits;

    for measured in complexity::measured(reading, limits) {
        if measured.over.is_empty() {
            continue;
        }

        let parts = complexity::parts(&measured.path, &measured.function);
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
        let scored = measured.function.scored;
        let mut finding = Finding::new(
            Rule::Complexity,
            complexity::message(&measured, limits),
            Location {
                path: measured.path.clone(),
                line: Some(scored.line),
                column: Some(scored.column),
                end_line: None,
            },
            &parts,
        );

        finding.complexity = Some(Detail {
            function: measured.function.readable(),
            scores: Scores::from(scored),
            limits,
            over: measured.over,
            base: None,
        });
        found.push(finding);
    }
}

/// One finding for each group of copies a person has not accepted, with the notices for an entry of
/// `duplicationAccepted` that matches no group and for a run with no `duplicationMinimum` (duplication
/// spec DUP-1, TUNE-1, QUIET-1, QUIET-2).
fn duplicated(reading: &Reading, found: &mut Vec<Finding>, notices: &mut Vec<Notice>) {
    let (groups, unmatched) = duplication::groups(reading);

    found.extend(
        groups
            .into_iter()
            .filter(|group| !group.accepted)
            .filter_map(Finding::grouped),
    );
    notices.extend(unmatched);
    notices.extend(reading.duplication.notice());
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
