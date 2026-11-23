//! The `duplication` command, and the groups of copies a reading holds, which `audit` will report too:
//! the settings the declarations give, the runs of statements that repeat, the groups they make with
//! their sizes and fingerprints, the groups a person accepted, and the document the command prints
//! (duplication spec SAME-3, SAME-4, SIZE-1, SIZE-2, TUNE-1 to TUNE-7, DUP-4, QUIET-1 to QUIET-4,
//! SHOW-1 to SHOW-5).

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::INTERFACE_VERSION;
use crate::complexity;
use crate::declarations::WORDS;
use crate::files::{self, Kind};
use crate::findings::{self, Rule};
use crate::forms::{DIGEST, Item, List};
use crate::hand_off::{HandOff, Stack};
use crate::notice::{self, Notice};
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::timings::Stages;
use crate::tree::Tree;

/// The kind of the notice for a run with no `duplicationMinimum` (spec TUNE-1).
const NOT_CHECKED: &str = "duplication-not-checked";

/// The kind of the notice for an accepted group no group matches (spec QUIET-2).
const NOT_FOUND: &str = "accepted-group-not-found";

/// The fewest copies a group needs where no declaration says (spec TUNE-3).
const COPIES: u64 = 2;

/// What the parse needs of the declarations to read a script's statements (spec SAME-2, COPY-3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lens {
    /// Whether any name matches any other.
    pub names: bool,
    /// Whether any string matches any other.
    pub strings: bool,
    /// Whether any number matches any other.
    pub numbers: bool,
    /// The callee chains of the test callees.
    pub test_callees: BTreeSet<String>,
}

/// An entry of `duplicationAccepted`, with who states it (spec TUNE-6, QUIET-2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accepted {
    /// The entry: a group's fingerprint, `/`, and its number of copies.
    pub entry: String,
    /// Who states it: `pack` or `project`.
    pub source: &'static str,
}

/// The settings of duplication, one of each for the whole run (spec TUNE-1 to TUNE-7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The fewest tokens a group's run holds, from `duplicationMinimum`; none, and nothing is looked
    /// for, where it is not declared.
    pub minimum_tokens: Option<u64>,
    /// The fewest lines, from `duplicationMinimumLines`; none where it is not declared.
    pub minimum_lines: Option<u64>,
    /// The fewest copies, from `duplicationCopies`, 2 where it is not declared.
    pub copies: u64,
    /// What the parse loosens and which calls are test callees.
    pub lens: Lens,
    /// The entries of `duplicationAccepted`, the pack's before the project's.
    pub accepted: Vec<Accepted>,
}

impl Settings {
    /// The settings the hand-off file declares: a number is the project's, else the later stack's; the
    /// project's word list replaces the pack's, and of one source the later stack's wins; the test
    /// callees of one source join across its stacks, and the project's replace the pack's; accepted
    /// groups join (spec TUNE-1 to TUNE-7).
    pub fn of(handed: Option<&HandOff>) -> Self {
        let stacks = handed.map_or(&[][..], |handed| handed.stacks.as_slice());
        let ignore = replaced(stacks, "duplicationIgnore").unwrap_or_default();
        let loosened = |word: &str| ignore.iter().any(|listed| listed == word);
        let test_callees = joined(stacks, "duplicationTestCallees", true)
            .or_else(|| joined(stacks, "duplicationTestCallees", false))
            .unwrap_or_default();

        Self {
            minimum_tokens: complexity::declared(stacks, "duplicationMinimum"),
            minimum_lines: complexity::declared(stacks, "duplicationMinimumLines"),
            copies: complexity::declared(stacks, "duplicationCopies").unwrap_or(COPIES),
            lens: Lens {
                names: loosened("names"),
                strings: loosened("strings"),
                numbers: loosened("numbers"),
                test_callees: test_callees.into_iter().collect(),
            },
            accepted: [(false, "pack"), (true, "project")]
                .into_iter()
                .flat_map(|(project, source)| {
                    joined(stacks, "duplicationAccepted", project)
                        .unwrap_or_default()
                        .into_iter()
                        .map(move |entry| Accepted { entry, source })
                })
                .collect(),
        }
    }

    /// What the parse reads statements through, where duplication is looked for at all (spec TUNE-1).
    pub fn lens(&self) -> Option<&Lens> {
        self.minimum_tokens.map(|_| &self.lens)
    }

    /// The notice for a run with no `duplicationMinimum`, where it has none (spec TUNE-1).
    pub fn notice(&self) -> Option<Notice> {
        self.minimum_tokens.is_none().then(|| {
            Notice::new(
                NOT_CHECKED,
                None,
                "No `duplicationMinimum` is declared, so no duplication is looked for and no duplication \
                 finding is reported."
                    .to_owned(),
            )
        })
    }

    /// The settings as the command prints them (spec SHOW-2).
    fn printed(&self) -> Value {
        json!({
            "accepted": self
                .accepted
                .iter()
                .map(|accepted| accepted.entry.as_str())
                .collect::<Vec<&str>>(),
            "ignore": self.ignored(),
            "minimumCopies": self.copies,
            "minimumLines": self.minimum_lines,
            "minimumTokens": self.minimum_tokens,
            "testCallees": self.lens.test_callees,
        })
    }

    /// The words loosened, in the order `WORDS` gives them.
    fn ignored(&self) -> Vec<&'static str> {
        WORDS
            .into_iter()
            .zip([self.lens.names, self.lens.strings, self.lens.numbers])
            .filter_map(|(word, loosened)| loosened.then_some(word))
            .collect()
    }
}

/// The list the project states for a key in its last stack that states it, else the pack's in its last
/// stack that states it (spec TUNE-4).
fn replaced(stacks: &[Stack], key: &str) -> Option<Vec<String>> {
    let last = |project: bool| {
        stacks
            .iter()
            .rev()
            .find_map(|stack| declarations(stack, project).get(key).and_then(texts))
    };

    last(true).or_else(|| last(false))
}

/// The lists one source states for a key, joined across its stacks in their order; none where none of
/// its stacks states the key (spec TUNE-5, TUNE-6).
fn joined(stacks: &[Stack], key: &str, project: bool) -> Option<Vec<String>> {
    let lists: Vec<Vec<String>> = stacks
        .iter()
        .filter_map(|stack| declarations(stack, project).get(key).and_then(texts))
        .collect();

    (!lists.is_empty()).then(|| lists.concat())
}

/// The pack's or the project's declarations of a stack.
const fn declarations(stack: &Stack, project: bool) -> &Map<String, Value> {
    if project { &stack.project } else { &stack.pack }
}

/// The entries of a list of text, or nothing when the value is not one.
fn texts(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|entry| entry.as_str().map(str::to_owned))
        .collect()
}

/// One copy of a group: a run of statements of one list of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// The file, by its path from the root.
    pub path: String,
    /// The line its first statement starts on, counted from one.
    pub start_line: usize,
    /// The line its last statement ends on.
    pub end_line: usize,
    /// Its first byte in the file.
    pub start: u32,
    /// The byte after its last.
    pub end: u32,
}

/// One group of copies (spec SAME-1 to SAME-4, SIZE-1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// The fingerprint `audit` gives, or would give, its finding (spec DUP-4).
    pub fingerprint: String,
    /// Its tokens: the fewest of any of its copies.
    pub tokens: u64,
    /// Its lines: the fewest of any of its copies.
    pub lines: u64,
    /// Its copies, by path and first line.
    pub copies: Vec<Place>,
    /// Whether a person accepted it (spec QUIET-1).
    pub accepted: bool,
}

/// Every group of copies in the reading's own scripts and Vue components, each run as large as it goes,
/// sorted by its first copy and marked accepted where an entry names its fingerprint and copies, with a
/// notice for each entry no group matches; none where no `duplicationMinimum` is declared. A file
/// declared data or generated, and a type declaration file, holds no copy (spec COPY-4, SAME-3, SAME-4,
/// SIZE-2, QUIET-1, QUIET-2).
pub fn groups(reading: &Reading) -> (Vec<Group>, Vec<Notice>) {
    let settings = &reading.duplication;
    let Some(minimum) = settings.minimum_tokens else {
        return (Vec::new(), Vec::new());
    };
    let files: Vec<(&str, &[List])> = reading
        .files
        .iter()
        .filter(|file| {
            matches!(file.kind, Kind::Script | Kind::Vue)
                && !reading.marks.generated.contains(&file.path)
        })
        .filter_map(|file| {
            reading
                .scripts
                .read
                .get(&file.path)
                .map(|read| (file.path.as_str(), read.lists.as_slice()))
        })
        .collect();
    let text = Text::of(files);
    let found: Vec<Group> = text
        .repeats()
        .into_iter()
        .filter_map(|(length, at)| text.group(length, at, settings, minimum))
        .collect();
    let mut groups = widest(found);
    // Acceptance comes after the groups inside others are dropped, so an accepted group still hides
    // them (spec QUIET-1).
    let entries: BTreeSet<String> = groups.iter().map(accepting).collect();
    let accepted: BTreeSet<&str> = settings
        .accepted
        .iter()
        .map(|accepted| accepted.entry.as_str())
        .collect();
    let mut named = BTreeSet::new();
    let notices = settings
        .accepted
        .iter()
        .filter(|accepted| !entries.contains(&accepted.entry))
        .filter(|accepted| named.insert((accepted.entry.as_str(), accepted.source)))
        .map(|accepted| {
            Notice::new(
                NOT_FOUND,
                None,
                format!(
                    "The accepted group `{}` that the {} states matches no group found, so the entry can be \
                     removed.",
                    accepted.entry, accepted.source
                ),
            )
        })
        .collect();

    for group in &mut groups {
        group
            .copies
            .sort_by(|a, b| (&a.path, a.start_line).cmp(&(&b.path, b.start_line)));
        group.accepted = accepted.contains(accepting(group).as_str());
    }

    groups.sort_by(|a, b| order(a).cmp(&order(b)));

    (groups, notices)
}

/// The entry of `duplicationAccepted` that accepts a group: its fingerprint, `/`, and its number of
/// copies (spec TUNE-6).
fn accepting(group: &Group) -> String {
    format!("{}/{}", group.fingerprint, group.copies.len())
}

/// What groups are sorted by: their first copy's path and lines, then their number of copies.
fn order(group: &Group) -> (Option<(&str, usize, usize)>, usize) {
    (
        group
            .copies
            .first()
            .map(|copy| (copy.path.as_str(), copy.start_line, copy.end_line)),
        group.copies.len(),
    )
}

/// Every statement list of the files, end to end, as one sequence of symbols, in which the runs that
/// repeat are found.
struct Text<'r> {
    /// The files, by their paths, with their lists.
    files: Vec<(&'r str, &'r [List])>,
    /// Each place's symbol: one number for each form a statement that joins runs has, a number of its
    /// own for each statement that joins none, and one after each list, so no run crosses a list.
    symbols: Vec<u64>,
    /// Where each place is: its file, its list and its statement; none after a list.
    places: Vec<Option<(usize, usize, usize)>>,
}

impl<'r> Text<'r> {
    /// The sequence of the files' lists.
    fn of(files: Vec<(&'r str, &'r [List])>) -> Self {
        let mut forms: HashMap<[u8; DIGEST], u64> = HashMap::new();
        // The numbers of what joins no run start above any form's.
        let mut alone = u64::from(u32::MAX);
        let mut symbols = Vec::new();
        let mut places = Vec::new();

        for (file, (_, lists)) in files.iter().enumerate() {
            for (list, statements) in lists.iter().enumerate() {
                for (index, item) in statements.items.iter().enumerate() {
                    let symbol = if item.joins {
                        let next = u64::try_from(forms.len()).unwrap_or(u64::MAX);

                        *forms.entry(item.digest).or_insert(next)
                    } else {
                        alone += 1;
                        alone
                    };

                    symbols.push(symbol);
                    places.push(Some((file, list, index)));
                }

                alone += 1;
                symbols.push(alone);
                places.push(None);
            }
        }

        Self {
            files,
            symbols,
            places,
        }
    }

    /// Each run that repeats, as its length in statements and the places it starts at, where its
    /// copies are not all preceded by the same statement, so it cannot be made longer at its start
    /// (spec SAME-4).
    fn repeats(&self) -> Vec<(usize, Vec<usize>)> {
        let order = suffixes(&self.symbols);
        let common = common(&self.symbols, &order);

        intervals(&common)
            .into_iter()
            .filter_map(|(length, first, last)| {
                let at = order[first..=last].to_vec();
                let before: BTreeSet<Option<u64>> = at
                    .iter()
                    .map(|place| place.checked_sub(1).map(|before| self.symbols[before]))
                    .collect();

                (before.len() > 1 || before.contains(&None)).then_some((length, at))
            })
            .collect()
    }

    /// The group a run makes from the places given, where it has enough copies that do not overlap and
    /// is large enough (spec SAME-3, SIZE-1, SIZE-2, DUP-4).
    fn group(
        &self,
        length: usize,
        mut at: Vec<usize>,
        settings: &Settings,
        minimum: u64,
    ) -> Option<Group> {
        at.sort_unstable_by_key(|place| self.places[*place]);

        // Two copies never overlap: in one list, the copies are taken from its start (spec SAME-3).
        let mut kept: Vec<(usize, usize, usize)> = Vec::new();

        for (file, list, index) in at.into_iter().filter_map(|place| self.places[place]) {
            let overlaps = kept
                .last()
                .is_some_and(|&(last_file, last_list, last_index)| {
                    last_file == file && last_list == list && index < last_index + length
                });

            if !overlaps {
                kept.push((file, list, index));
            }
        }

        if u64::try_from(kept.len()).unwrap_or(u64::MAX) < settings.copies {
            return None;
        }

        let runs: Vec<(&str, &[Item])> = kept
            .iter()
            .map(|&(file, list, index)| {
                let (path, lists) = self.files[file];

                (path, &lists[list].items[index..index + length])
            })
            .collect();
        let tokens = runs.iter().map(|(_, run)| tokens(run)).min()?;
        let lines = runs.iter().map(|(_, run)| lines(run)).min()?;

        if tokens < minimum || settings.minimum_lines.is_some_and(|least| lines < least) {
            return None;
        }

        let parts: Vec<String> = runs
            .first()?
            .1
            .iter()
            .map(|item| hex(&item.digest))
            .collect();
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();

        Some(Group {
            fingerprint: findings::fingerprint(Rule::Duplication, &parts),
            tokens,
            lines,
            copies: runs.iter().map(|(path, run)| place(path, run)).collect(),
            accepted: false,
        })
    }
}

/// The suffix array of a sequence: its places, sorted by what follows each, by prefix doubling.
fn suffixes(symbols: &[u64]) -> Vec<usize> {
    let count = symbols.len();
    let mut order: Vec<usize> = (0..count).collect();

    if count < 2 {
        return order;
    }

    let mut sorted = symbols.to_vec();

    sorted.sort_unstable();
    sorted.dedup();

    let mut rank: Vec<usize> = symbols
        .iter()
        .map(|symbol| sorted.partition_point(|other| other < symbol))
        .collect();
    let mut next = vec![0; count];
    let mut step = 1;

    loop {
        let key = |place: usize| {
            (
                rank[place],
                rank.get(place + step).map_or(0, |after| after + 1),
            )
        };

        order.sort_unstable_by_key(|place| key(*place));
        next[order[0]] = 0;

        for pair in order.windows(2) {
            next[pair[1]] = next[pair[0]] + usize::from(key(pair[0]) != key(pair[1]));
        }

        std::mem::swap(&mut rank, &mut next);

        if rank[order[count - 1]] == count - 1 {
            break;
        }

        step *= 2;
    }

    order
}

/// For each place of the suffix array, the length of the prefix it shares with the place before it; 0
/// for the first (Kasai's method).
fn common(symbols: &[u64], order: &[usize]) -> Vec<usize> {
    let count = symbols.len();
    let mut rank = vec![0; count];

    for (at, place) in order.iter().enumerate() {
        rank[*place] = at;
    }

    let mut common = vec![0; count];
    let mut shared = 0;

    for (place, &at) in rank.iter().enumerate() {
        if at == 0 {
            shared = 0;

            continue;
        }

        let other = order[at - 1];

        while place + shared < count
            && other + shared < count
            && symbols[place + shared] == symbols[other + shared]
        {
            shared += 1;
        }

        common[at] = shared;
        shared = shared.saturating_sub(1);
    }

    common
}

/// The intervals of the suffix array whose places share a prefix longer than nothing: its length, and
/// the interval's first and last places in the array (the bottom-up walk of Abouelhoda, Kurtz and
/// Ohlebusch).
fn intervals(common: &[usize]) -> Vec<(usize, usize, usize)> {
    let mut found = Vec::new();
    let mut open: Vec<(usize, usize)> = vec![(0, 0)];

    for at in 1..=common.len() {
        let shared = common.get(at).copied().unwrap_or(0);
        let mut left = at - 1;

        while open.last().is_some_and(|&(length, _)| shared < length) {
            if let Some((length, first)) = open.pop() {
                found.push((length, first, at - 1));
                left = first;
            }
        }

        if open.last().is_none_or(|&(length, _)| shared > length) {
            open.push((shared, left));
        }
    }

    found
}

/// A run's tokens: its statements' together (spec SIZE-1).
fn tokens(run: &[Item]) -> u64 {
    run.iter().map(|item| item.tokens).sum()
}

/// A run's lines: its statements' together, a line two of them share counted once (spec SIZE-1).
fn lines(run: &[Item]) -> u64 {
    let mut count = 0;
    let mut last = None;

    for held in run.iter().filter_map(|item| item.lines) {
        count += held.count;

        if last == Some(held.first) {
            count -= 1;
        }

        last = Some(held.last);
    }

    count
}

/// Where a run is.
fn place(path: &str, run: &[Item]) -> Place {
    let first = run.first();
    let last = run.last();

    Place {
        path: path.to_owned(),
        start_line: first.map_or(0, |item| item.line),
        end_line: last.map_or(0, |item| item.end_line),
        start: first.map_or(0, |item| item.start),
        end: last.map_or(0, |item| item.end),
    }
}

/// A digest in lower-case hexadecimal digits.
fn hex(digest: &[u8]) -> String {
    let mut digits = String::with_capacity(digest.len() * 2);

    for byte in digest {
        // Writing into a String cannot fail.
        let _ = write!(digits, "{byte:02x}");
    }

    digits
}

/// The groups left once each that lies inside another is dropped: one whose every copy lies inside a
/// different copy of another group with at least as many copies. Of two that each lie inside the other,
/// the first stays (spec SAME-4).
fn widest(found: Vec<Group>) -> Vec<Group> {
    let inside = |inner: &Group, outer: &Group| {
        let mut used = BTreeSet::new();

        outer.copies.len() >= inner.copies.len()
            && inner.copies.iter().all(|copy| {
                outer
                    .copies
                    .iter()
                    .position(|around| {
                        around.path == copy.path
                            && around.start <= copy.start
                            && copy.end <= around.end
                    })
                    .is_some_and(|at| used.insert(at))
            })
    };
    let kept: Vec<bool> = found
        .iter()
        .enumerate()
        .map(|(at, group)| {
            !found.iter().enumerate().any(|(other_at, other)| {
                other_at != at && inside(group, other) && !(other_at > at && inside(other, group))
            })
        })
        .collect();

    found
        .into_iter()
        .zip(kept)
        .filter_map(|(group, kept)| kept.then_some(group))
        .collect()
}

/// Reads the working tree as `audit` does, with the hand-off file given or with none, and returns the
/// document `duplication` prints: the settings, the groups a person has not accepted, or with `all`
/// every group, and the notices (spec SHOW-1 to SHOW-3).
pub fn run(hand_off: Option<&Path>, all: bool) -> Result<String, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = reading::start(hand_off)?;
    let listing = files::list(&root, &mut notices)?;
    let mut reading = reading::read(
        handed.as_ref(),
        &Tree::working(root),
        listing,
        notices,
        &mut Stages::new(false),
    )?;
    let (groups, found) = groups(&reading);
    let listed: Vec<Value> = groups
        .iter()
        .filter(|group| all || !group.accepted)
        .map(entry)
        .collect();

    reading.notices.extend(found);
    reading.notices.extend(reading.duplication.notice());

    let mut document = json!({
        "groups": listed,
        "interfaceVersion": INTERFACE_VERSION,
        "notices": notice::printed(reading.notices),
        "settings": reading.duplication.printed(),
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    Ok(format!("{document:#}"))
}

/// One group as the document lists it (spec SHOW-2).
fn entry(group: &Group) -> Value {
    json!({
        "accepted": group.accepted,
        "copies": group
            .copies
            .iter()
            .map(|copy| json!({ "endLine": copy.end_line, "path": copy.path, "startLine": copy.start_line }))
            .collect::<Vec<Value>>(),
        "fingerprint": group.fingerprint,
        "lines": group.lines,
        "tokens": group.tokens,
    })
}
