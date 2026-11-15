//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: its hand-off file, its files,
//! its packages, its scripts' imports and where they lead, its entry points, and which export of which
//! file is used. `graph` prints what was read, and `audit` prints a report of the findings it shows,
//! compared with a base commit where it is given one.

mod audit;
mod cli;
mod declarations;
mod entries;
mod files;
mod findings;
mod git;
mod glob;
mod graph;
mod hand_off;
mod imports;
mod inherit;
mod marks;
mod notice;
mod packages;
mod pages;
mod reading;
mod resolve;
mod scripts;
mod stop;
mod tools;
mod tree;
mod tsconfig;
mod uses;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;
use stop::Stop;

/// The interface version: what yak and the analyzer must agree on (spec RUN-7).
const INTERFACE_VERSION: u64 = 1;

/// The hand-off formats the analyzer reads, oldest first (spec RUN-1).
const HAND_OFF_FORMATS: [u32; 1] = [1];

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

    panic::catch_unwind(run).unwrap_or_else(|_| ExitCode::from(stop::FAULT))
}

/// Does what the command line asks.
fn run() -> ExitCode {
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
            ended(reading::working(hand_off.as_deref()).map(graph::printed))
        }
        Ok(Command::Audit { hand_off, base }) => {
            ended(audit::run(hand_off.as_deref(), base.as_deref()))
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
