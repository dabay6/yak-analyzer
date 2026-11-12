//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::stop::Stop;

/// The oldest git these commands work with: the release that brought `--no-optional-locks`.
const OLDEST: (u32, u32) = (2, 15);

/// What every start of git is given before its command (spec RUN-6a): no lock is taken, so nothing is
/// written inside `.git`; no pager and no file-system monitor is started; and letter case counts in an
/// ignore pattern on every platform.
const SETTINGS: [&str; 8] = [
    "--no-optional-locks",
    "--no-pager",
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.untrackedCache=false",
    "-c",
    "core.ignorecase=false",
];

/// The variables git itself calls local to a repository, which `git rev-parse --local-env-vars` prints.
/// Left in place, they would make git read another repository than the root's.
const REPOSITORY_VARIABLES: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// What git printed, and the first line of a warning it gave while succeeding.
pub struct Answer {
    /// What git wrote on standard output.
    pub output: Vec<u8>,
    /// The first line git wrote on standard error, where it wrote one.
    pub warning: Option<String>,
}

/// Runs one git command in the root and returns what it printed.
pub fn read(root: &Path, command: &[&str]) -> Result<Answer, Stop> {
    let shown = root.display();

    if !root.is_dir() {
        return Err(Stop::new(format!(
            "the root {shown} is not a folder; run the analyzer in a checkout, or hand it a hand-off file \
             that names one"
        )));
    }

    let mut git = Command::new("git");

    git.args(SETTINGS)
        .args(command)
        .current_dir(root)
        .stdin(Stdio::null());

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    let (oldest_major, oldest_minor) = OLDEST;
    let ran = git.output().map_err(|error| {
        Stop::new(format!(
            "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
             {oldest_minor} or newer, which must be on the PATH"
        ))
    })?;
    let said = String::from_utf8_lossy(&ran.stderr);
    let first = said.lines().find(|line| !line.trim().is_empty());

    if ran.status.success() {
        return Ok(Answer {
            output: ran.stdout,
            warning: first.map(str::to_owned),
        });
    }

    // git failed. An old git fails on the settings themselves, so the version is asked only now.
    if let Some(version) = version().filter(|version| version.0 < OLDEST) {
        return Err(Stop::new(format!(
            "git {} is older than {oldest_major}.{oldest_minor}, the oldest the analyzer works with; update git",
            version.1
        )));
    }

    Err(Stop::new(format!(
        "git cannot read the root {shown}: {}; run the analyzer inside a git work tree",
        first.unwrap_or("it gave no reason")
    )))
}

/// The version of the git on the `PATH`: its first two numbers, and the version as git writes it.
fn version() -> Option<((u32, u32), String)> {
    let answer = Command::new("git")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&answer.stdout);
    // git prints `git version 2.56.0`, and some builds add more after the numbers.
    let written = text
        .trim()
        .strip_prefix("git version ")?
        .split(' ')
        .next()?;
    let mut numbers = written.split('.').map(str::parse::<u32>);
    let major = numbers.next()?.ok()?;
    let minor = numbers.next()?.ok()?;

    Some(((major, minor), written.to_owned()))
}
