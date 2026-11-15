//! Starts git, and only to read (spec RUN-6, RUN-6a, DEP-2; dead-code spec CMD-2, COMMIT-2, INH-2,
//! LIFE-2).
//!
//! How git is started was settled by a trial, whose report is in the work item's report folder:
//! every start gets the settings below and loses the variables by which a caller's environment could
//! point git at another repository.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::thread::{self, JoinHandle};

use crate::stop::Stop;

/// The oldest git these commands work with: the release that brought `--no-optional-locks`.
const OLDEST: (u32, u32) = (2, 15);

/// The oldest git that never fetches a missing object when told not to, through the variable below
/// (dead-code spec COMMIT-2).
pub const NO_LAZY_FETCH: (u32, u32) = (2, 44);

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

/// The variable that tells git never to fetch an object a partial clone lacks, set for every start
/// (dead-code spec COMMIT-2). A git older than `NO_LAZY_FETCH` passes it over.
const NO_LAZY_FETCH_VARIABLE: &str = "GIT_NO_LAZY_FETCH";

/// How many files git compares when it looks for renames, whatever the user's or the repository's
/// settings say: git's own default (dead-code spec INH-2).
pub const RENAME_LIMIT: u32 = 1000;

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

/// The root's folder from the top of its work tree, ending in `/`, or empty at the top.
pub fn prefix(root: &Path) -> Result<String, Stop> {
    let answer = read(root, &["rev-parse", "--show-prefix"])?;

    Ok(String::from_utf8_lossy(&answer.output).trim().to_owned())
}

/// Asks git about one path above the root, as the file list asks about every path under it (spec
/// LIST-4).
pub fn above(root: &Path, path: &str) -> Result<Above, Stop> {
    if leaves_work_tree(&prefix(root)?, path) {
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

/// Whether a path from the root climbs above the top of the work tree, the root lying at the prefix
/// given.
pub fn leaves_work_tree(prefix: &str, path: &str) -> bool {
    let depth = prefix
        .split('/')
        .filter(|segment| !segment.is_empty())
        .count();
    let up = path
        .split('/')
        .take_while(|segment| *segment == "..")
        .count();

    up > depth
}

/// The full name of the commit a name given on the command line names (dead-code spec CMD-2). The name
/// was checked before: it does not begin with `-` and holds no control character.
pub fn commit(root: &Path, name: &str) -> Result<String, Stop> {
    let asked = format!("{name}^{{commit}}");
    let ran = output(root, &["rev-parse", "--verify", "--quiet", &asked], None)?;
    let full = String::from_utf8_lossy(&ran.stdout).trim().to_owned();

    if ran.status.success() && !full.is_empty() {
        Ok(full)
    } else {
        Err(Stop::new(format!(
            "the base `{name}` is not a commit in this repository; fetch it with `git fetch`, or name another, \
             and run again"
        )))
    }
}

/// Whether the root's repository is a partial clone: its settings name a promisor remote (dead-code
/// spec COMMIT-2).
pub fn partial(root: &Path) -> Result<bool, Stop> {
    let answer = read(root, &["config", "--list", "-z"])?;

    // Each setting is its name, a line break, and its value; git writes the name in lower case.
    Ok(answer
        .output
        .split(|byte| *byte == 0)
        .filter_map(|setting| std::str::from_utf8(setting).ok())
        .any(|setting| {
            let (name, value) = setting.split_once('\n').unwrap_or((setting, ""));

            name == "extensions.partialclone"
                || (name.starts_with("remote.")
                    && name.ends_with(".promisor")
                    && matches!(value, "true" | "yes" | "on" | "1"))
        }))
}

/// The version of the git on the `PATH`, as git writes it, where it is older than the version given.
pub fn older_than(wanted: (u32, u32)) -> Option<String> {
    version()
        .filter(|(numbers, _)| *numbers < wanted)
        .map(|(_, written)| written)
}

/// The files git stores at a commit under the root, each with its mode, its type, its object and its
/// path from the root (dead-code spec COMMIT-2).
pub fn stored(root: &Path, commit: &str) -> Result<Answer, Stop> {
    read(root, &["ls-tree", "-r", "-z", commit])
}

/// The files renamed between a commit and the working tree, under the root, as git finds them with its
/// own rename detection and its default similarity (dead-code spec INH-2).
pub fn renamed(root: &Path, commit: &str) -> Result<Answer, Stop> {
    let limit = format!("-l{RENAME_LIMIT}");

    read(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--name-status",
            "-z",
            "--relative",
            "-M",
            &limit,
            commit,
            "--",
        ],
    )
}

/// A git that hands out the content of one stored object after another, started once for a whole
/// reading, so no file needs a start of git of its own (dead-code spec COMMIT-2). Its input is closed
/// when it is dropped, so it ends with the analyzer (dead-code spec LIFE-2).
pub struct Batch {
    /// The git that answers.
    child: Child,
    /// What it reads the names from; closed first when the batch is dropped.
    input: Option<ChildStdin>,
    /// What it writes each content on.
    output: BufReader<ChildStdout>,
    /// What it writes on standard error, read by a thread of its own so it never waits on a full pipe.
    errors: Option<JoinHandle<Vec<u8>>>,
}

impl Batch {
    /// Starts git's batch in the root.
    pub fn start(root: &Path) -> Result<Self, Stop> {
        let mut git = command(root);

        git.args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = git.spawn().map_err(not_started)?;
        let (Some(input), Some(output), Some(mut errors)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            return Err(Stop::new(
                "git's batch started without its pipes; this is a fault in the analyzer, so report it",
            ));
        };
        let errors = thread::spawn(move || {
            let mut said = Vec::new();
            let _ = errors.read_to_end(&mut said);

            said
        });

        Ok(Self {
            child,
            input: Some(input),
            output: BufReader::new(output),
            errors: Some(errors),
        })
    }

    /// The content of the stored file a name gives, an object's or a commit's and a path's, or none
    /// where it names no file.
    pub fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>, Stop> {
        if name.contains(['\n', '\r']) {
            return Ok(None);
        }

        let asked = self
            .input
            .as_mut()
            .map(|input| writeln!(input, "{name}").and_then(|()| input.flush()));
        let mut header = String::new();

        if !matches!(asked, Some(Ok(()))) || self.output.read_line(&mut header).unwrap_or(0) == 0 {
            return Err(self.failed());
        }

        // A file comes back as its object, its type and its size, then its content and a line break; a
        // name that names nothing, as the name and `missing`.
        let mut facts = header.trim_end().rsplitn(3, ' ');
        let (Some(size), Some(kind)) = (facts.next(), facts.next()) else {
            return Ok(None);
        };
        let Ok(size) = size.parse::<usize>() else {
            return Ok(None);
        };
        let mut content = vec![0; size + 1];

        if self.output.read_exact(&mut content).is_err() {
            return Err(self.failed());
        }

        content.pop();

        Ok((kind == "blob").then_some(content))
    }

    /// The stop for a batch that stopped answering, with git's own first line.
    fn failed(&mut self) -> Stop {
        self.input = None;

        let _ = self.child.wait();
        let said = self
            .errors
            .take()
            .and_then(|errors| errors.join().ok())
            .unwrap_or_default();
        let said = String::from_utf8_lossy(&said);
        let first = said
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("it gave no reason");

        Stop::new(format!("git cannot read the base: {first}"))
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        // Closing its input ends it.
        self.input = None;

        let _ = self.child.wait();

        if let Some(errors) = self.errors.take() {
            let _ = errors.join();
        }
    }
}

/// A git command in the root, with the settings and the variables every start gets (spec RUN-6a;
/// dead-code spec COMMIT-2).
fn command(root: &Path) -> Command {
    let mut git = Command::new("git");

    git.args(SETTINGS)
        .current_dir(root)
        .env(NO_LAZY_FETCH_VARIABLE, "1");

    for variable in REPOSITORY_VARIABLES {
        git.env_remove(variable);
    }

    git
}

/// The stop for a git that cannot be started.
fn not_started(error: std::io::Error) -> Stop {
    let (oldest_major, oldest_minor) = OLDEST;

    Stop::new(format!(
        "git cannot be started ({error}); the analyzer reads a project through git {oldest_major}.\
         {oldest_minor} or newer, which must be on the PATH"
    ))
}

/// Runs one git command in the root, with what it reads on standard input where there is any, and
/// returns how it ended, whatever its exit code.
fn output(root: &Path, command_line: &[&str], input: Option<&[u8]>) -> Result<Output, Stop> {
    if !root.is_dir() {
        return Err(Stop::new(format!(
            "the root {} is not a folder; run the analyzer in a checkout, or hand it a hand-off file that \
             names one",
            root.display()
        )));
    }

    let mut git = command(root);

    git.args(command_line)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = git.spawn().map_err(not_started)?;

    // The input is written from a thread of its own, so that git never waits on a full output pipe
    // while the analyzer waits to finish writing.
    let writer = match (input, child.stdin.take()) {
        (Some(input), Some(mut stdin)) => {
            let input = input.to_vec();

            Some(thread::spawn(move || stdin.write_all(&input)))
        }
        _ => None,
    };
    let ran = child.wait_with_output().map_err(not_started)?;

    // git stops reading when it fails, which breaks the pipe; its exit code says what happened.
    if let Some(writer) = writer {
        let _ = writer.join();
    }

    Ok(ran)
}

/// Runs one git command in the root, with what it reads on standard input where there is any. With
/// input, exit code 1 is an answer too: `check-ignore` gives it when no path is ignored.
fn run(root: &Path, command_line: &[&str], input: Option<&[u8]>) -> Result<Answer, Stop> {
    let ran = output(root, command_line, input)?;
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
    let (oldest_major, oldest_minor) = OLDEST;

    if let Some(version) = older_than(OLDEST) {
        return Err(Stop::new(format!(
            "git {version} is older than {oldest_major}.{oldest_minor}, the oldest the analyzer works with; \
             update git"
        )));
    }

    Err(Stop::new(format!(
        "git cannot read the root {}: {}; run the analyzer inside a git work tree",
        root.display(),
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
