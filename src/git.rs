//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

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
    run(root, command, None)
}

/// Asks git which of the paths given, relative to the root, it ignores (spec LIST-5). It answers from
/// its patterns, whether or not a path is on disk. A path inside a submodule must not be given: git
/// stops on one.
pub fn ignored(root: &Path, paths: &[&str]) -> Result<Vec<String>, Stop> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let input: Vec<u8> = paths
        .iter()
        .flat_map(|path| path.bytes().chain([0]))
        .collect();
    let answer = run(
        root,
        &[
            "check-ignore",
            "-z",
            "--stdin",
            "--verbose",
            "--non-matching",
        ],
        Some(&input),
    )?;
    let fields: Vec<&[u8]> = answer.output.split(|byte| *byte == 0).collect();

    // Each path comes back as four fields: the file of the pattern that decided, its line, the pattern,
    // and the path. A path no pattern matched has the first three empty; a pattern that begins with `!`
    // says the path is not ignored.
    Ok(fields
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|answer| !answer[0].is_empty() && !answer[2].starts_with(b"!"))
        .map(|answer| String::from_utf8_lossy(answer[3]).into_owned())
        .collect())
}

/// What git says of a path above the root, given from the root with `..` (spec LIST-4, CFG-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Above {
    /// A file of the work tree that git does not ignore: tracked, or untracked and not ignored.
    File,
    /// A path git ignores.
    Ignored,
    /// A path of the work tree that is neither: nothing is there.
    Missing,
    /// A path above the top of the work tree.
    OutsideWorkTree,
}

/// Asks git about one path above the root, as the file list asks about every path under it (spec
/// LIST-4).
pub fn above(root: &Path, path: &str) -> Result<Above, Stop> {
    let prefix = read(root, &["rev-parse", "--show-prefix"])?;
    let depth = String::from_utf8_lossy(&prefix.output)
        .trim()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .count();
    let up = path
        .split('/')
        .take_while(|segment| *segment == "..")
        .count();

    if up > depth {
        return Ok(Above::OutsideWorkTree);
    }

    let pathspec = format!(":(literal){path}");
    let listed = read(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            &pathspec,
        ],
    )?;

    if !listed.output.is_empty() {
        Ok(Above::File)
    } else if ignored(root, &[path])?.is_empty() {
        Ok(Above::Missing)
    } else {
        Ok(Above::Ignored)
    }
}

/// Runs one git command in the root, with what it reads on standard input where there is any. With
/// input, exit code 1 is an answer too: `check-ignore` gives it when no path is ignored.
fn run(root: &Path, command: &[&str], input: Option<&[u8]>) -> Result<Answer, Stop> {
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
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    let (oldest_major, oldest_minor) = OLDEST;
    let started = |error: std::io::Error| {
        Stop::new(format!(
            "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
             {oldest_minor} or newer, which must be on the PATH"
        ))
    };
    let mut child = git.spawn().map_err(started)?;

    // The input is written from a thread of its own, so that git never waits on a full output pipe
    // while the analyzer waits to finish writing.
    let writer = match (input, child.stdin.take()) {
        (Some(input), Some(mut stdin)) => {
            let input = input.to_vec();

            Some(thread::spawn(move || stdin.write_all(&input)))
        }
        _ => None,
    };
    let ran = child.wait_with_output().map_err(started)?;

    // git stops reading when it fails, which breaks the pipe; its exit code says what happened.
    if let Some(writer) = writer {
        let _ = writer.join();
    }

    let answered = ran.status.success() || (input.is_some() && ran.status.code() == Some(1));
    let said = String::from_utf8_lossy(&ran.stderr);
    let first = said.lines().find(|line| !line.trim().is_empty());

    if answered {
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
