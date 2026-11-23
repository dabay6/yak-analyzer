//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, `audit` prints a report of the findings it shows,
//! compared with a base commit where it is given one, and `complexity` prints the scores of each
//! function.

mod audit;
mod cli;
mod command_files;
mod complexity;
mod declarations;
mod entries;
mod files;
mod findings;
mod functions;
mod git;
mod glob;
mod graph;
mod hand_off;
mod imports;
mod inherit;
mod launcher;
mod marks;
mod notice;
mod packages;
mod pages;
mod reading;
mod resolve;
mod scripts;
mod stop;
mod timings;
mod tools;
mod tree;
mod tsconfig;
mod uses;
mod yaml;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;
use std::thread;
use std::time::Instant;

use cli::Command;
use stop::Stop;
use timings::Stages;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;

/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

/// The stack of each thread that reads a project: far more than the deepest nesting of a file or the
/// longest chain of files needs, so none runs out of it, on Windows' small main thread least of all
/// (dead-code spec LIFE-3). It is reserved, and the system gives it memory only as it is used.
pub const STACK: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    // A fault in the analyzer itself gets one line and its own exit code, never a stack trace (spec RUN-3).
    panic::set_hook(Box::new(|fault| {
        eprintln!(
            "{}",
            stop::line(&format!(
                "a fault in the analyzer itself, not in the project: {fault}"
            ))
        );
    }));

    // Started by the npm launcher, the binary ends soon after the launcher does (dead-code spec LIFE-1).
    launcher::watch();

    // The work runs on a thread with a stack of its own size; a fault there ends that thread, which the
    // join reports.
    thread::Builder::new()
        .stack_size(STACK)
        .spawn(run)
        .map_or(ExitCode::from(stop::FAULT), |work| {
            work.join().unwrap_or_else(|_| ExitCode::from(stop::FAULT))
        })
}

/// Does what the command line asks.
fn run() -> ExitCode {
    // When the timings are asked for, the whole run is timed from here, before the arguments are read
    // (spec TIME-2a, TIME-2c).
    let started = timings::asked().then(Instant::now);
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match cli::parse(&args) {
        Ok(Command::Version) => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        Ok(Command::Interface) => {
            println!(
                "{}",
                serde_json::json!({ "handOffFormats": HAND_OFF_FORMATS, "interfaceVersion": INTERFACE_VERSION })
            );

            ExitCode::SUCCESS
        }
        Ok(Command::Graph { hand_off }) => {
            let mut stages = Stages::new(started.is_some());
            let document = reading::working(hand_off.as_deref(), &mut stages)
                .map(|reading| stages.time("build", || graph::printed(reading)));
            let succeeded = document.is_ok();
            let code = stages.time("print", || ended(document));

            // Only a run that succeeded writes the line (spec TIME-3).
            if succeeded && let Some(started) = started {
                timings::write(&timings::line(&stages, started.elapsed()));
            }

            code
        }
        Ok(Command::Audit { hand_off, base }) => {
            ended(audit::run(hand_off.as_deref(), base.as_deref()))
        }
        Ok(Command::Complexity { hand_off, all }) => {
            ended(complexity::run(hand_off.as_deref(), all))
        }
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

            ExitCode::from(stop::REFUSED)
        }
    }
}

/// Ends a run that reads a project: the document it made, on standard output, or the one line of a
/// run that could not go on, on standard error (spec RUN-3, CMD-3).
fn ended(document: Result<String, Stop>) -> ExitCode {
    match document {
        Ok(document) => {
            println!("{document}");

            ExitCode::SUCCESS
        }
        Err(stopped) => {
            eprintln!("{}", stopped.line());

            ExitCode::from(stop::STOPPED)
        }
    }
}
