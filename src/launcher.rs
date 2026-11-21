//! The npm launcher that started the binary: when it ends, as when yak stops it, the binary ends soon
//! after, writing nothing (dead-code spec LIFE-1).
//!
//! The launcher names its own process in a variable of the binary's environment. On macOS and Linux
//! the binary watches it: the launcher is the binary's parent until it ends, when the system gives the
//! binary another. On Windows nothing needs watching: Node starts every child in a job that the system
//! ends with Node. A binary started without the variable is stopped directly, and nothing here changes
//! how it runs.

/// The variable the launcher names its process id in.
#[cfg(unix)]
const VARIABLE: &str = "YAK_ANALYZER_LAUNCHER";

/// How often, in milliseconds, the binary looks whether the launcher is still its parent.
#[cfg(unix)]
const EVERY: u64 = 200;

/// Watches the launcher that the environment names, where it names one, from a thread of its own, and
/// ends the process when the launcher is gone.
#[cfg(unix)]
pub fn watch() {
    use std::os::unix::process::parent_id;
    use std::time::Duration;
    use std::{env, process, thread};

    use crate::stop;

    let Some(launcher) = env::var(VARIABLE)
        .ok()
        .and_then(|id| id.parse::<u32>().ok())
    else {
        return;
    };

    // The thread lives as long as the process; it is never joined.
    let _ = thread::Builder::new()
        .name("launcher".to_owned())
        .spawn(move || {
            while parent_id() == launcher {
                thread::sleep(Duration::from_millis(EVERY));
            }

            process::exit(i32::from(stop::STOPPED));
        });
}

/// Nothing to watch: Node starts the binary in a job that the system ends with Node.
#[cfg(not(unix))]
pub const fn watch() {}
