//! The `audit` command: reads the project, and with a base the base commit too, and prints what the
//! reading shows as one SARIF 2.1.0 document with one run (spec CMD-1, REPORT-1, COMMIT-1, INH-4).
//!
//! The run names the tool and lists the five rules, the findings and the notices.

use std::fmt::Write as _;
use std::path::Path;
use std::thread;

use serde_json::{Value, json};

use crate::files;
use crate::findings::{self, Finding, Location, Rule};
use crate::git;
use crate::inherit::{self, Base};
use crate::notice::{self, Notice};
use crate::reading::{self, Reading, Start};
use crate::stop::Stop;
use crate::tree::Tree;
use crate::{INTERFACE_VERSION, STACK};

/// The key of `partialFingerprints` that holds a finding's fingerprint, with the scheme's version in
/// it, so that a later scheme can sit beside it (spec REPORT-3).
const FINGERPRINT_KEY: &str = "yakAnalyzer/v1";

/// Where the schema of SARIF 2.1.0 is published.
const SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

/// The version of SARIF the report is written in.
const SARIF_VERSION: &str = "2.1.0";

/// The name SARIF gives the folder every path of the report is relative to: the root.
const ROOT: &str = "%SRCROOT%";

/// The characters of a path that a URI holds as they are, besides letters and digits: the unreserved
/// ones, the sub-delimiters, `@`, and the `/` between segments (RFC 3986).
const PLAIN: &str = "-._~!$&'()*+,;=@/";

/// Reads the project, with the hand-off file given or with none, and with the base given the base
/// commit too, and returns the report (spec CMD-1, CMD-3). The stops come in RUN-4's order: the base is
/// asked about after the working tree's file list, as its steps 7b and 7c.
pub fn run(hand_off: Option<&Path>, base: Option<&str>) -> Result<String, Stop> {
    let Start {
        handed,
        root,
        mut notices,
    } = reading::start(hand_off)?;
    let listing = files::list(&root, &mut notices)?;
    let Some(base) = base else {
        return reading::read(handed.as_ref(), &Tree::working(root), listing, notices)
            .map(|reading| printed(reading, None));
    };
    let commit = git::commit(&root, base)?;

    if git::partial(&root)?
        && let Some(version) = git::older_than(git::NO_LAZY_FETCH)
    {
        let (major, minor) = git::NO_LAZY_FETCH;

        return Err(Stop::new(format!(
            "the root is in a partial clone, and git {version} cannot read the base without fetching over the \
             network; a partial clone needs git {major}.{minor} or newer, so update git"
        )));
    }

    let mut base_notices = Vec::new();
    let (base_tree, base_listing) = Tree::base(root.clone(), &commit, &mut base_notices)?;
    let working_tree = Tree::working(root);

    // The two sides are read at once, each on a thread with the stack every reading has (dead-code spec
    // INH-4, LIFE-3).
    let (working, base) = thread::scope(|scope| {
        let base = thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(scope, || {
                let reading =
                    reading::read(handed.as_ref(), &base_tree, base_listing, base_notices)?;
                let renamed = git::renamed(base_tree.root(), &commit)?;

                Ok::<_, Stop>(Base {
                    commit: commit.clone(),
                    reading,
                    renames: inherit::renames(&renamed.output),
                    renames_cut_short: renamed
                        .warning
                        .is_some_and(|warning| warning.contains("rename")),
                })
            })
            .map_err(|error| {
                Stop::new(format!(
                    "the base cannot be read on a thread of its own ({error})"
                ))
            })?;
        let working = reading::read(handed.as_ref(), &working_tree, listing, notices);
        let base = base
            .join()
            .unwrap_or_else(|fault| std::panic::resume_unwind(fault));

        Ok::<_, Stop>((working?, base?))
    })?;

    Ok(printed(working, Some(base)))
}

/// The report `audit` prints for a reading, compared with the base's where there is one.
pub fn printed(mut reading: Reading, base: Option<Base>) -> String {
    let (mut found, notices) = findings::find(&reading);

    reading.notices.extend(notices);

    if let Some(base) = base {
        let compared = inherit::mark(&reading, &mut found, &base);

        reading.notices.extend(compared);
        reading
            .notices
            .extend(base.reading.notices.into_iter().map(Notice::of_base));
    }

    let mut document = json!({
        "$schema": SCHEMA,
        "version": SARIF_VERSION,
        "runs": [{
            "columnKind": "utf16CodeUnits",
            "invocations": [{
                "executionSuccessful": true,
                "toolExecutionNotifications": notice::sorted(reading.notices)
                    .iter()
                    .map(notification)
                    .collect::<Vec<Value>>(),
            }],
            "properties": { "interfaceVersion": INTERFACE_VERSION },
            "results": found.iter().map(result).collect::<Vec<Value>>(),
            "tool": {
                "driver": {
                    "informationUri": env!("CARGO_PKG_REPOSITORY"),
                    "name": env!("CARGO_PKG_NAME"),
                    "rules": Rule::ALL.iter().map(|rule| described(*rule)).collect::<Vec<Value>>(),
                    "version": env!("CARGO_PKG_VERSION"),
                },
            },
        }],
    });

    // An object's keys in order, whatever order they were added in.
    document.sort_all_objects();

    format!("{document:#}")
}

/// One rule as the run lists it.
fn described(rule: Rule) -> Value {
    json!({ "id": rule.name(), "shortDescription": { "text": rule.description() } })
}

/// One finding as a result of the run: its rule, its message, its location, the other places it
/// involves, and its fingerprint (spec REPORT-1).
fn result(finding: &Finding) -> Value {
    let mut result = json!({
        "level": "warning",
        "locations": [placed(&finding.location)],
        "message": { "text": finding.message },
        "partialFingerprints": { FINGERPRINT_KEY: finding.fingerprint },
        "ruleId": finding.rule.name(),
        "ruleIndex": finding.rule.index(),
    });

    if let Some(baseline) = finding.baseline {
        result["baselineState"] = json!(baseline.name());
    }

    if !finding.related.is_empty() {
        result["relatedLocations"] = finding
            .related
            .iter()
            .enumerate()
            .map(|(id, location)| {
                let mut related = placed(location);

                related["id"] = json!(id);
                related
            })
            .collect();
    }

    result
}

/// A location as SARIF writes one: the file, and the line and column where there are some.
fn placed(location: &Location) -> Value {
    let mut physical = json!({ "artifactLocation": artifact(&location.path) });

    if let Some(line) = location.line {
        physical["region"] = json!({ "startLine": line });

        if let Some(column) = location.column {
            physical["region"]["startColumn"] = json!(column);
        }
    }

    json!({ "physicalLocation": physical })
}

/// One notice as a notification of the run's invocation: its kind, its sentence, and its file where
/// it has one (spec REPORT-4).
fn notification(notice: &Notice) -> Value {
    let mut notification = json!({
        "descriptor": { "id": notice.kind() },
        "level": "note",
        "message": { "text": notice.sentence() },
    });

    if let Some(file) = notice.file() {
        notification["locations"] =
            json!([{ "physicalLocation": { "artifactLocation": artifact(file) } }]);
    }

    // A notice of the base's reading says so in its property bag (dead-code spec REPORT-4).
    if notice.is_base() {
        notification["properties"] = json!({ "base": true });
    }

    notification
}

/// A file as SARIF names one: its path from the root, as a URI.
fn artifact(path: &str) -> Value {
    json!({ "uri": uri(path), "uriBaseId": ROOT })
}

/// A path as a relative URI: each byte a URI cannot hold as it is, is written as `%` and its two
/// hexadecimal digits (RFC 3986).
fn uri(path: &str) -> String {
    let mut uri = String::with_capacity(path.len());

    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || PLAIN.as_bytes().contains(&byte) {
            uri.push(char::from(byte));
        } else {
            // Writing into a String cannot fail.
            let _ = write!(uri, "%{byte:02X}");
        }
    }

    uri
}
