//! A made-up project as a git repository of its own in a temporary folder, for the tests that run the
//! binary on one (spec FIX-1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Shuts the machine's and the user's own git settings out of a command, global excludes included, so
/// that a test's result does not depend on the machine.
fn alone(command: &mut Command) -> &mut Command {
    let none = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-git-settings");

    if !none.exists() {
        fs::write(&none, "").expect("an empty settings file can be written");
    }

    command
        .env("GIT_CONFIG_GLOBAL", &none)
        .env("GIT_CONFIG_SYSTEM", &none)
        .env("GIT_CONFIG_NOSYSTEM", "1")
}

/// One made-up project.
pub struct Project {
    /// The folder that holds the checkout and, beside it, the project's hand-off files and whatever
    /// else a test makes outside the checkout.
    pub home: PathBuf,
    /// The checkout's root.
    pub root: PathBuf,
}

impl Project {
    /// A new project of the name given, with an empty git repository as its checkout.
    pub fn new(name: &str) -> Self {
        let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);

        // The folder is left over from an earlier run, or is not there: either way it is made new.
        let _ = fs::remove_dir_all(&home);

        let project = Self {
            root: home.join("checkout"),
            home,
        };

        fs::create_dir_all(&project.root).expect("the project's folder can be made");
        project.git(&["init", "--quiet", "--initial-branch=main"]);

        project
    }

    /// Writes a file of the project, with the folders above it.
    pub fn write(&self, path: &str, content: &str) -> &Self {
        let file = self.root.join(path);

        fs::create_dir_all(file.parent().expect("a file has a folder"))
            .expect("the folder can be made");
        fs::write(file, content).expect("the file can be written");

        self
    }

    /// Runs git in the checkout and returns what it printed. The command must succeed.
    pub fn git(&self, args: &[&str]) -> String {
        let mut git = Command::new("git");

        git.args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(["-c", "protocol.file.allow=always"])
        .args(args)
        .current_dir(&self.root);

        let ran = alone(&mut git).output().expect("git starts");

        assert!(
            ran.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&ran.stderr)
        );

        String::from_utf8_lossy(&ran.stdout).into_owned()
    }

    /// Tracks everything that is not ignored, in one commit.
    pub fn commit(&self) -> &Self {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--quiet", "--message", "the project"]);

        self
    }

    /// Runs `graph` in the checkout, after the change the test asks for, and returns how it ended.
    pub fn run(&self, change: impl FnOnce(&mut Command)) -> Output {
        let mut binary = Command::new(env!("CARGO_BIN_EXE_yak-analyzer"));

        binary.arg("graph").current_dir(&self.root);
        alone(&mut binary);
        change(&mut binary);

        binary.output().expect("the binary starts")
    }

    /// The document `graph` prints for the checkout. The run must succeed and write no line.
    pub fn graph(&self) -> Value {
        printed(&self.run(|_| {}))
    }

    /// Writes a hand-off file that names the folder given as the root, with the folders yak's detection
    /// looked in and the stacks given, and returns the file's path.
    pub fn hand_off(&self, root: &Path, folders: &[&str], stacks: Value) -> String {
        let folder = self.home.join("hand-off");
        let file = folder.join("hand-off.json");
        let text = |path: &Path| {
            path.to_str()
                .expect("the test's folder is valid UTF-8")
                .to_owned()
        };
        let document = json!({
            "formatVersion": 1,
            "root": text(root),
            "detection": { "folders": folders, "truncated": false },
            "stacks": stacks,
            "siblings": text(&folder),
        });

        fs::create_dir_all(&folder).expect("the folder can be made");
        fs::write(&file, document.to_string()).expect("the hand-off file can be written");

        text(&file)
    }
}

/// The document a successful run printed.
pub fn printed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).expect("graph prints JSON")
}

/// The one line a stopped run wrote, after checking that it stopped as a stop does.
pub fn stopped(output: &Output) -> String {
    let error = String::from_utf8_lossy(&output.stderr).into_owned();

    assert_eq!(output.status.code(), Some(1), "{error}");
    assert!(output.stdout.is_empty(), "{error}");
    assert_eq!(error.lines().count(), 1, "{error}");

    error.trim_end().to_owned()
}

/// The paths of the files a document lists, in its order.
pub fn paths(document: &Value) -> Vec<&str> {
    document["files"]
        .as_array()
        .expect("a list of files")
        .iter()
        .map(|file| file["path"].as_str().expect("a path"))
        .collect()
}

/// The kinds of the notices a document holds, in its order.
pub fn notices(document: &Value) -> Vec<&str> {
    document["notices"]
        .as_array()
        .expect("a list of notices")
        .iter()
        .map(|notice| notice["kind"].as_str().expect("a kind"))
        .collect()
}
