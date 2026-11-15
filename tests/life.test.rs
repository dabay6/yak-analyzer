//! The life of a run: the binary ends soon after the npm launcher that started it, a killed audit
//! leaves nothing behind, and no project's size or depth runs it out of stack (dead-code spec LIFE-1
//! to LIFE-3).

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{BufRead as _, BufReader, Read as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use serde_json::Value;

/// The built binary.
const BINARY: &str = env!("CARGO_BIN_EXE_yak-analyzer");

/// How many files the long chain, the long cycle and the long re-export chain each hold (dead-code spec
/// LIFE-3).
const LENGTH: usize = 10_000;

/// How deep the deeply nested file nests its expressions and its blocks (dead-code spec LIFE-3).
const DEPTH: usize = 1_000;

/// A new, empty folder of one test's own, under cargo's folder for the files tests make.
fn folder(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);

    // The folder is left over from an earlier run, or is not there: either way it is made new.
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the test's folder can be made");

    folder
}

/// Runs git in a folder, shut off from the machine's and the user's own settings. It must succeed.
fn git(root: &Path, args: &[&str]) {
    let none = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-git-settings");

    if !none.exists() {
        fs::write(&none, "").expect("an empty settings file can be written");
    }

    let ran = Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(["-c", "maintenance.auto=false", "-c", "gc.auto=0"])
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_GLOBAL", &none)
        .env("GIT_CONFIG_SYSTEM", &none)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("git starts");

    assert!(
        ran.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&ran.stderr)
    );
}

/// The project too long and too deep for a small stack, made once for the tests that share it and
/// committed, so it is its own base: a chain of files, each of which passes on everything the next one
/// exports with `export *`, whose last file imports the first. It is at once an import chain, a cycle
/// and a re-export chain of `LENGTH` files. A file beside it nests `DEPTH` levels deep in parentheses
/// and in blocks (dead-code spec LIFE-3). It is made by the tests, and never committed here.
fn long_project() -> &'static Path {
    static PROJECT: OnceLock<PathBuf> = OnceLock::new();

    PROJECT.get_or_init(|| {
        let root = folder("life-long-project");
        let chain = root.join("src").join("chain");

        fs::create_dir_all(&chain).expect("the folder can be made");
        git(&root, &["init", "--quiet", "--initial-branch=main"]);
        fs::write(
            root.join("package.json"),
            r#"{ "name": "long", "main": "./src/index.ts" }"#,
        )
        .expect("the file can be written");
        fs::write(
            root.join("src").join("index.ts"),
            "import { value } from './chain/f0';\nimport { deep } from './deep';\n\
             export const total = value + deep;\n",
        )
        .expect("the file can be written");

        for place in 0..LENGTH {
            let content = if place + 1 < LENGTH {
                format!("export * from './f{}';\n", place + 1)
            } else {
                "import './f0';\nexport const value = 1;\n".to_owned()
            };

            fs::write(chain.join(format!("f{place}.ts")), content)
                .expect("the file can be written");
        }

        let mut deep = String::from("export const deep = ");

        deep.push_str(&"(".repeat(DEPTH));
        deep.push('1');
        deep.push_str(&")".repeat(DEPTH));
        deep.push_str(";\n");
        deep.push_str(&"if (deep) {\n".repeat(DEPTH));
        deep.push_str(&"}\n".repeat(DEPTH));
        fs::write(root.join("src").join("deep.ts"), deep).expect("the file can be written");

        git(&root, &["add", "--all"]);
        git(
            &root,
            &["commit", "--quiet", "--message", "the long project"],
        );

        root
    })
}

/// Runs a command of the binary in a folder and returns how it ended.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BINARY)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .expect("the binary starts")
}

/// The document a run printed, which must have succeeded and written no line.
fn printed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(&output.stdout).expect("the binary prints JSON")
}

#[test]
fn long_chains_a_long_cycle_and_deep_nesting_do_not_run_the_analyzer_out_of_stack() {
    let root = long_project();

    // `graph` and `audit` read it whole, and `audit` with a base reads it twice, each on a thread of
    // its own (dead-code spec LIFE-3, INH-4).
    printed(&run(root, &["graph"]));

    for options in [&[][..], &["--base", "main"][..]] {
        let args: Vec<&str> = std::iter::once("audit")
            .chain(options.iter().copied())
            .collect();
        let report = printed(&run(root, &args));
        let results = report["runs"][0]["results"]
            .as_array()
            .expect("a list of results");

        // The one finding is the cycle, all its files in one set; the chain and the re-exports bring
        // `value` to the entry point, so nothing else is unused.
        assert_eq!(results.len(), 1, "{options:?}");
        assert_eq!(results[0]["ruleId"], "circular-dependency");
        assert_eq!(
            results[0]["relatedLocations"].as_array().map(Vec::len),
            Some(LENGTH - 1)
        );
        assert!(
            results[0]["message"]["text"]
                .as_str()
                .expect("a message")
                .ends_with(&format!("→ {} more → `src/chain/f0.ts`.", LENGTH - 10)),
            "{}",
            results[0]["message"]["text"]
        );
    }
}

/// The name of the platform package's target on this machine, as Node names it.
fn target() -> String {
    let system = match env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        other => other,
    };
    let processor = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };

    format!("{system}-{processor}")
}

/// The launcher and the binary laid out in a folder as npm installs them: the main package, and the
/// platform package beside it. Returns the launcher's script.
fn installed(home: &Path) -> PathBuf {
    let scope = home.join("node_modules").join("@dabay6");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("npm")
        .join("yak-analyzer");
    let main = scope.join("yak-analyzer");

    for part in ["bin/yak-analyzer.js", "lib/launch.js", "package.json"] {
        let to = main.join(part);

        fs::create_dir_all(to.parent().expect("a file has a folder"))
            .expect("the folder can be made");
        fs::copy(source.join(part), &to).expect("the launcher's file can be copied");
    }

    let platform = scope.join(format!("yak-analyzer-{}", target()));
    let binary = if cfg!(windows) {
        "yak-analyzer.exe"
    } else {
        "yak-analyzer"
    };

    fs::create_dir_all(&platform).expect("the folder can be made");
    fs::write(
        platform.join("package.json"),
        format!(
            r#"{{ "name": "@dabay6/yak-analyzer-{}", "version": "0.0.0" }}"#,
            target()
        ),
    )
    .expect("the file can be written");
    fs::copy(BINARY, platform.join(binary)).expect("the binary can be copied");

    main.join("bin").join("yak-analyzer.js")
}

/// A hand-off file that nobody ever writes, so that a run waits on it for as long as it runs: a named
/// pipe nobody writes to. On Windows a Node process holds the pipe open; it is returned so the test can
/// end it.
fn never_written(home: &Path) -> (String, Option<Child>) {
    if cfg!(windows) {
        let name = format!(r"\\.\pipe\yak-analyzer-life-{}", std::process::id());
        let script = format!(
            "require('node:net').createServer(() => {{}}).listen({}, () => console.log('listening'));",
            serde_json::to_string(&name).expect("a name is JSON")
        );
        let mut server = Command::new("node")
            .args(["-e", &script])
            .stdout(Stdio::piped())
            .spawn()
            .expect("node starts");
        let mut line = String::new();

        BufReader::new(server.stdout.take().expect("the server's output"))
            .read_line(&mut line)
            .expect("the server says it listens");
        assert_eq!(line.trim(), "listening");

        (name, Some(server))
    } else {
        let fifo = home.join("hand-off.json");
        let made = Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("mkfifo starts");

        assert!(made.success());

        (fifo.display().to_string(), None)
    }
}

#[test]
fn the_binary_ends_soon_after_the_launcher_that_started_it_and_writes_nothing() {
    let home = folder("life-launcher");
    let launcher = installed(&home);
    let (hand_off, server) = never_written(&home);
    let mut node = Command::new("node")
        .arg(&launcher)
        .args(["audit", "--hand-off", &hand_off])
        .current_dir(&home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the launcher starts");

    // The launcher starts the binary, whose audit waits on the hand-off file.
    thread::sleep(Duration::from_secs(2));
    assert!(
        node.try_wait()
            .expect("the launcher can be asked")
            .is_none(),
        "the audit is still running"
    );

    // The binary holds the launcher's output and its errors open until it ends, so both close when it
    // does.
    let mut output = node.stdout.take().expect("the launcher's output");
    let mut errors = node.stderr.take().expect("the launcher's errors");
    let (sender, closed) = mpsc::channel();

    thread::spawn(move || {
        let mut written = Vec::new();
        let _ = output.read_to_end(&mut written);
        let _ = errors.read_to_end(&mut written);
        let _ = sender.send(written);
    });

    node.kill().expect("the launcher can be stopped");
    node.wait().expect("the launcher ends");

    let stopped = Instant::now();
    let written = closed
        .recv_timeout(Duration::from_secs(10))
        .expect("the binary ends after the launcher");

    // It ends within five seconds, and writes nothing (dead-code spec LIFE-1).
    assert!(
        stopped.elapsed() < Duration::from_secs(5),
        "{:?}",
        stopped.elapsed()
    );
    assert_eq!(String::from_utf8_lossy(&written), "");

    if let Some(mut server) = server {
        let _ = server.kill();
        let _ = server.wait();
    }
}

#[test]
fn a_binary_started_directly_does_not_wait_on_its_standard_input() {
    let root = folder("life-direct");

    git(&root, &["init", "--quiet"]);

    // Its standard input open and never written to, the binary still runs to its end (dead-code spec
    // LIFE-1).
    let started = Command::new(BINARY)
        .arg("audit")
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");
    let output = started.wait_with_output().expect("the binary ends");

    assert_eq!(printed(&output)["version"], "2.1.0");
    assert_eq!(output.stdout, run(&root, &["audit"]).stdout);
}

/// Every path under a folder, `.git` included, with the size of each file and when it was last
/// changed. A folder's own time is left out: the system may record a change made in it a little
/// later, and a change shows in its files anyway.
fn snapshot(folder: &Path) -> BTreeMap<PathBuf, Option<(u64, Option<SystemTime>)>> {
    let mut found = BTreeMap::new();
    let mut waiting = vec![folder.to_path_buf()];

    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).expect("the folder can be read") {
            let entry = entry.expect("an entry can be read");
            let facts = entry.metadata().expect("an entry can be looked at");
            let path = entry.path();

            if facts.is_dir() {
                found.insert(path.clone(), None);
                waiting.push(path);
            } else {
                found.insert(path, Some((facts.len(), facts.modified().ok())));
            }
        }
    }

    found
}

/// A command that prints a line once the binary with the process id given has started git's batch,
/// which reads the base's files, and prints nothing before.
fn watching(binary: u32) -> Command {
    if cfg!(windows) {
        let mut watcher = Command::new("powershell");

        watcher.args([
            "-NoProfile",
            "-Command",
            &format!(
                "while (-not (Get-CimInstance Win32_Process -Filter 'ParentProcessId={binary}' | \
                 Where-Object {{ $_.CommandLine -like '*cat-file*' }})) {{ }}; 'reading'"
            ),
        ]);

        watcher
    } else {
        // The binary leads a process group of its own, which every git it starts joins.
        let mut watcher = Command::new("sh");

        watcher.args([
            "-c",
            &format!(
                "while ! ps -A -o pgid=,args= | grep -q '^ *{binary} .*cat-file'; do :; done; echo reading"
            ),
        ]);

        watcher
    }
}

/// How many git processes the binary with the process id given started, and their own, are still
/// running, leaving out any that have ended and wait to be reaped.
fn gits_running(binary: u32) -> usize {
    let output = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "$all = Get-CimInstance Win32_Process; $ids = @({binary}); $found = 0; \
                     do {{ $more = @($all | Where-Object {{ $ids -contains $_.ParentProcessId -and \
                     $ids -notcontains $_.ProcessId }}); $ids += $more.ProcessId; \
                     $found += @($more | Where-Object {{ $_.Name -like 'git*' }}).Count }} \
                     while ($more.Count -gt 0); $found"
                ),
            ])
            .output()
    } else {
        // A git that has ended is not running, though it is still listed, as a zombie, until it is
        // reaped: in a container whose first process reaps nothing, it stays one (state `Z`).
        Command::new("sh")
            .args([
                "-c",
                &format!(
                    "ps -A -o pgid=,stat=,comm= | awk '$1 == {binary} && $2 !~ /^Z/ && $3 ~ /git/' | wc -l"
                ),
            ])
            .output()
    }
    .expect("the process list can be read");

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("a count")
}

/// Starts an audit of the long project with its own commit as the base, leading a process group of its
/// own where there are process groups.
fn audit_with_base(root: &Path) -> Child {
    let mut binary = Command::new(BINARY);

    binary
        .args(["audit", "--base", "main"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut binary, 0);

    binary.spawn().expect("the binary starts")
}

#[test]
fn an_audit_killed_while_it_reads_a_base_leaves_nothing_changed_and_no_git_running() {
    let root = long_project();
    let before = snapshot(root);
    let mut binary = audit_with_base(root);
    let id = binary.id();
    let mut watcher = watching(id)
        .stdout(Stdio::piped())
        .spawn()
        .expect("the watcher starts");
    let said = watcher.stdout.take().expect("the watcher's output");
    let (sender, reading) = mpsc::channel();

    thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(said).read_line(&mut line);
        let _ = sender.send(line);
    });

    // It is killed as soon as git's batch is reading the base's files.
    loop {
        if reading.try_recv().is_ok() {
            break;
        }

        if binary
            .try_wait()
            .expect("the binary can be asked")
            .is_some()
        {
            let _ = watcher.kill();

            panic!("the audit ended before the test saw it read the base");
        }

        thread::sleep(Duration::from_millis(5));
    }

    let _ = watcher.wait();
    binary.kill().expect("the binary can be killed");

    let ended = binary.wait_with_output().expect("the binary ends");

    assert!(!ended.status.success());
    assert!(ended.stdout.is_empty());

    // Within thirty seconds no git it started is still running, and nothing under the root, `.git`
    // included, has changed: no lock is left (dead-code spec LIFE-2).
    let killed = Instant::now();

    while gits_running(id) > 0 {
        assert!(
            killed.elapsed() < Duration::from_secs(30),
            "a git the audit started is still running"
        );
        thread::sleep(Duration::from_millis(200));
    }

    assert_eq!(snapshot(root), before);
}
