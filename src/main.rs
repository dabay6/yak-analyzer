//! `yak-analyzer`: a static analyzer for TypeScript and JavaScript, built for yakshaver.
//!
//! This is the walking skeleton. It answers its version and analyzes nothing yet.

use std::env;
use std::process::ExitCode;

/// The option that prints the version. yak runs it after an install to read the installed version.
const VERSION_OPTION: &str = "--version";

/// The exit code for a command the binary does not have.
const REFUSED: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match args.as_slice() {
        [only] if only == VERSION_OPTION => {
            println!("yak-analyzer {}", env!("CARGO_PKG_VERSION"));

            ExitCode::SUCCESS
        }
        [] => {
            eprintln!("yak-analyzer: nothing to do; it answers only {VERSION_OPTION}");

            ExitCode::from(REFUSED)
        }
        _ => {
            let given = args.join(" ");

            eprintln!("yak-analyzer: cannot do `{given}`; it answers only {VERSION_OPTION}");

            ExitCode::from(REFUSED)
        }
    }
}
