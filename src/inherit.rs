//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too, through the files git finds renamed between them (dead-code spec INH-1 to INH-3,
//! COMMIT-4, REPORT-4); a complexity finding by its function's scores there (complexity spec NEW-1 to
//! NEW-5).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use crate::complexity::{self, Scores};
use crate::findings::{self, Baseline, Finding, Graph, Location, Rule};
use crate::git;
use crate::notice::{Notice, SKIPPED};
use crate::reading::Reading;

/// What the base gave: its commit, its reading, and the files renamed since.
pub struct Base {
    /// The commit's full name.
    pub commit: String,
    /// The commit's reading.
    pub reading: Reading,
    /// Each base path git found renamed, with its path on the branch.
    pub renames: BTreeMap<String, String>,
    /// Whether git gave up looking for renames for its limit.
    pub renames_cut_short: bool,
}

/// The files git found renamed, from what `git diff --name-status -z` printed: each base path with its
/// path on the branch (dead-code spec INH-2).
pub fn renames(output: &[u8]) -> BTreeMap<String, String> {
    let mut fields = output
        .split(|byte| *byte == 0)
        .map(|field| String::from_utf8_lossy(field).into_owned());
    let mut renames = BTreeMap::new();

    // A rename or a copy is its letter and score, then two paths; any other change, its letter and one.
    while let Some(status) = fields.next() {
        match status.chars().next() {
            Some('R') => {
                if let (Some(old), Some(new)) = (fields.next(), fields.next()) {
                    renames.insert(old, new);
                }
            }
            Some('C') => {
                fields.next();
                fields.next();
            }
            Some(_) => {
                fields.next();
            }
            None => {}
        }
    }

    renames
}

/// Marks each finding of the working tree new or inherited, and gives the notices the comparison makes
/// (dead-code spec INH-1, INH-3, COMMIT-4, REPORT-4).
pub fn mark(working: &Reading, found: &mut [Finding], base: &Base) -> Vec<Notice> {
    let skipped: Vec<&str> = base
        .reading
        .notices
        .iter()
        .filter(|notice| notice.kind() == SKIPPED)
        .filter_map(Notice::file)
        .collect();
    let mut notices = vec![Notice::new(
        "base",
        None,
        format!(
            "The findings are compared with the base commit {}: one the base has too is marked unchanged, \
             and any other new.",
            base.commit
        ),
    )];

    if base.renames_cut_short {
        notices.push(Notice::new(
            "renames-cut-short",
            None,
            format!(
                "git stopped looking for renamed files at its limit of {}, so a finding in a file renamed \
                 since the base may be marked new.",
                git::RENAME_LIMIT
            ),
        ));
    }

    let moved = |path: &str| {
        base.renames
            .get(path)
            .cloned()
            .unwrap_or_else(|| path.to_owned())
    };

    // One function's scores depend on no other file, so a file skipped at the base leaves the
    // complexity findings compared; a skipped script has no functions there (complexity spec NEW-4).
    complex(working, found, base, moved);

    // A file skipped at the base can leave it with findings it never had, which would make the same
    // findings on the branch look inherited: so then every other finding is new (dead-code spec
    // COMMIT-4).
    if !skipped.is_empty() {
        notices.push(Notice::new(
            "base-files-skipped",
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding of the five dead-code kinds is marked \
                 unchanged; complexity findings are still compared, and those in a skipped script are new: \
                 `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
        ));

        for finding in found
            .iter_mut()
            .filter(|finding| finding.rule != Rule::Complexity)
        {
            finding.baseline = Some(Baseline::New);
        }

        return notices;
    }

    let (base_found, _) = findings::find(&base.reading);
    // How many findings of each fingerprint the base has, with its paths moved to the branch's.
    let mut held: BTreeMap<String, usize> = BTreeMap::new();

    for finding in base_found
        .iter()
        .filter(|finding| !matches!(finding.rule, Rule::CircularDependency | Rule::Complexity))
    {
        *held.entry(finding.moved(moved)).or_default() += 1;
    }

    let working_graph = Graph::of(working);
    let base_graph = Graph::of(&base.reading);
    let base_pairs: BTreeSet<(String, String)> = base_graph
        .pairs()
        .into_iter()
        .map(|(from, to)| (moved(from), moved(to)))
        .collect();
    let base_sets: BTreeMap<String, usize> = base_graph
        .sets()
        .into_iter()
        .map(|(path, set)| (moved(path), set))
        .collect();

    // The findings are in the report's order, so of those that share a fingerprint, the base's count
    // are taken first by line and column (dead-code spec INH-1).
    for finding in found.iter_mut() {
        if finding.rule == Rule::CircularDependency {
            cycle(working, &working_graph, finding, &base_pairs, &base_sets);

            continue;
        }

        if finding.rule == Rule::Complexity {
            continue;
        }

        let inherited = held
            .get_mut(&finding.fingerprint)
            .filter(|count| **count > 0)
            .map(|count| *count -= 1)
            .is_some();

        finding.baseline = Some(if inherited {
            Baseline::Unchanged
        } else {
            Baseline::New
        });
    }

    findings::sort(found);

    notices
}

/// Marks each complexity finding: unchanged where the base has its function, the path moved through
/// the renames, over each limit the finding is over with a score at least as high, and otherwise new,
/// with what the base had in its message. Every function of the base is read, and functions of one
/// fingerprint pair in source order (complexity spec NEW-1 to NEW-3, NEW-5).
fn complex(working: &Reading, found: &mut [Finding], base: &Base, moved: impl Fn(&str) -> String) {
    let limits = working.limits;
    let mut held: BTreeMap<String, VecDeque<Scores>> = BTreeMap::new();

    for measured in complexity::measured(&base.reading, limits) {
        held.entry(complexity::fingerprint(
            &moved(&measured.path),
            &measured.function,
        ))
        .or_default()
        .push_back(Scores::from(measured.function.scored));
    }

    for finding in found
        .iter_mut()
        .filter(|finding| finding.rule == Rule::Complexity)
    {
        let Some(detail) = &mut finding.complexity else {
            continue;
        };
        let at_base = held
            .get_mut(&finding.fingerprint)
            .and_then(VecDeque::pop_front);
        let inherited = at_base.is_some_and(|scores| {
            complexity::inherited(&detail.over, detail.scores, scores, limits)
        });

        if let Some(scores) = at_base
            && !inherited
        {
            finding
                .message
                .push_str(&complexity::at_base(&detail.over, scores, limits));
        }

        detail.base = Some(at_base);
        finding.baseline = Some(if inherited {
            Baseline::Unchanged
        } else {
            Baseline::New
        });
    }
}

/// Marks a circular dependency: inherited when one strongly connected set of the base holds all its
/// files and the branch added no edge between two of them; otherwise new, placed at the first import, by
/// path and line, that makes an edge the branch added (dead-code spec INH-3).
fn cycle(
    working: &Reading,
    graph: &Graph<'_>,
    finding: &mut Finding,
    base_pairs: &BTreeSet<(String, String)>,
    base_sets: &BTreeMap<String, usize>,
) {
    let files: Vec<String> = finding.files().into_iter().map(str::to_owned).collect();
    let one_set = files
        .iter()
        .map(|file| base_sets.get(file))
        .collect::<Option<BTreeSet<_>>>()
        .is_some_and(|sets| sets.len() == 1);
    // Each edge between two of its files that the base lacks: the importing file, the line and place
    // of its first import that makes it, and the file it leads to.
    let mut added: Vec<(&str, usize, usize, &str)> = Vec::new();

    for from in &files {
        for to in &files {
            if let Some((line, place)) = graph.first(from, to)
                && !base_pairs.contains(&(from.clone(), to.clone()))
            {
                added.push((from, line, place, to));
            }
        }
    }

    added.sort_unstable();

    let Some(&(from, line, place, to)) = added.first() else {
        finding.baseline = Some(if one_set {
            Baseline::Unchanged
        } else {
            Baseline::New
        });

        return;
    };
    let specifier = working
        .scripts
        .read
        .get(from)
        .and_then(|read| read.imports.get(place))
        .map_or("", |import| import.specifier.as_str());
    let mut locations: Vec<Location> = std::iter::once(finding.location.clone())
        .chain(finding.related.drain(..))
        .filter(|location| location.path != from)
        .collect();

    locations.sort();
    finding.location = Location {
        path: from.to_owned(),
        line: Some(line),
        column: None,
        end_line: None,
    };
    finding.related = locations;
    let _ = write!(
        finding.message,
        " The branch added the import of `{specifier}` on line {line} of `{from}`, which leads to `{to}`."
    );
    finding.baseline = Some(Baseline::New);
}
