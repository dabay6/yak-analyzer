//! The `audit` command: prints what a reading shows as one SARIF 2.1.0 document with one run (spec
//! CMD-1, REPORT-1).
//!
//! The run names the tool and lists the five rules, the findings and the notices.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::INTERFACE_VERSION;
use crate::findings::{self, Finding, Location, Rule};
use crate::notice::{self, Notice};
use crate::reading::Reading;

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

/// The report `audit` prints for a reading.
pub fn printed(mut reading: Reading) -> String {
    let (found, notices) = findings::find(&reading);

    reading.notices.extend(notices);

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
