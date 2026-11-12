//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! It answers its version and its interface version, and reads a project: so far its hand-off file, its
//! files and its packages. It reports no findings yet.

mod cli;
mod declarations;
mod files;
mod git;
mod glob;
mod graph;
mod hand_off;
mod marks;
mod notice;
mod packages;
mod stop;

use std::env;
use std::ffi::OsString;
use std::panic;
use std::process::ExitCode;

use cli::Command;

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
        Ok(Command::Graph { hand_off }) => match graph::read(hand_off.as_deref()) {
            Ok(document) => {
                println!("{document}");

                ExitCode::SUCCESS
            }
            Err(stopped) => {
                eprintln!("{}", stopped.line());

                ExitCode::from(stop::STOPPED)
            }
        },
        Err(refusal) => {
            eprintln!("{}", stop::line(&refusal));

            ExitCode::from(stop::REFUSED)
        }
    }
}
