//! New against inherited: each finding of the working tree marked `new`, or `unchanged` where the base
//! commit has it too (dead-code spec INH-1, COMMIT-4, REPORT-4).

use std::collections::BTreeMap;

use crate::findings::{self, Baseline, Finding};
use crate::notice::{Notice, SKIPPED};
use crate::reading::Reading;

/// What the base gave: its commit and its reading.
pub struct Base {
    /// The commit's full name.
    pub commit: String,
    /// The commit's reading.
    pub reading: Reading,
}

/// Marks each finding of the working tree new or inherited, and gives the notices the comparison makes
/// (dead-code spec INH-1, COMMIT-4, REPORT-4).
pub fn mark(found: &mut [Finding], base: &Base) -> Vec<Notice> {
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

    // A file skipped at the base can leave it with findings it never had, which would make the same
    // findings on the branch look inherited: so then every finding is new (dead-code spec COMMIT-4).
    if !skipped.is_empty() {
        notices.push(Notice::new(
            "base-files-skipped",
            None,
            format!(
                "{} file(s) were skipped at the base, so no finding is marked unchanged: `{}`.",
                skipped.len(),
                skipped.join("`, `")
            ),
        ));

        for finding in found.iter_mut() {
            finding.baseline = Some(Baseline::New);
        }

        return notices;
    }

    let (base_found, _) = findings::find(&base.reading);
    // How many findings of each fingerprint the base has.
    let mut held: BTreeMap<&str, usize> = BTreeMap::new();

    for finding in &base_found {
        *held.entry(&finding.fingerprint).or_default() += 1;
    }

    // The findings are in the report's order, so of those that share a fingerprint, the base's count
    // are taken first by line and column (dead-code spec INH-1).
    for finding in found.iter_mut() {
        let inherited = held
            .get_mut(finding.fingerprint.as_str())
            .filter(|count| **count > 0)
            .map(|count| *count -= 1)
            .is_some();

        finding.baseline = Some(if inherited {
            Baseline::Unchanged
        } else {
            Baseline::New
        });
    }

    notices
}
