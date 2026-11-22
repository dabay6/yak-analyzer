//! The time each stage of a run takes, written on standard error as one line when the variable
//! `YAK_ANALYZER_TIMINGS` is `1` (spec TIME-1 to TIME-6).

use std::env;
use std::io::{self, Write as _};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

/// The variable that turns the timings on.
const VARIABLE: &str = "YAK_ANALYZER_TIMINGS";

/// The stages of a reading, in the order `reading::working` takes them (spec TIME-2, TIME-2b). The line holds every
/// one, 0 for a stage that never ran.
const READING: [&str; 11] = [
    "handOff",
    "files",
    "marks",
    "packages",
    "tsconfig",
    "scriptsRead",
    "scriptsParse",
    "resolve",
    "entries",
    "uses",
    "build",
];

/// Whether the timings were asked for: the variable is set to `1`, and to nothing else (spec TIME-1).
pub fn asked() -> bool {
    env::var_os(VARIABLE).is_some_and(|value| value == "1")
}

/// The time each stage took. It reads the clock only when the timings were asked for.
#[derive(Debug)]
pub struct Stages {
    /// Whether the timings were asked for.
    on: bool,
    /// Each stage that ran, with the time it took, summed over every time it ran.
    taken: Vec<(&'static str, Duration)>,
}

impl Stages {
    /// Stages that keep their times when `on` is true, and do nothing but run the work otherwise.
    pub const fn new(on: bool) -> Self {
        Self {
            on,
            taken: Vec::new(),
        }
    }

    /// Runs the work given and adds the time it took to the stage named.
    pub fn time<T>(&mut self, stage: &'static str, work: impl FnOnce() -> T) -> T {
        if !self.on {
            return work();
        }

        let started = Instant::now();
        let done = work();

        self.add(stage, started.elapsed());

        done
    }

    /// Adds a time to the stage named, to what it already holds.
    fn add(&mut self, stage: &'static str, taken: Duration) {
        match self.taken.iter_mut().find(|(name, _)| *name == stage) {
            Some((_, sum)) => *sum += taken,
            None => self.taken.push((stage, taken)),
        }
    }

    /// The time the stage named took, or nothing when it never ran.
    fn of(&self, stage: &str) -> Duration {
        self.taken
            .iter()
            .find(|(name, _)| *name == stage)
            .map_or(Duration::ZERO, |(_, taken)| *taken)
    }
}

/// The line: the reading's stages, printing, and the whole run, in whole microseconds (spec TIME-5).
/// A later command adds its own reading under a key of its own beside `reading` (spec TIME-6).
pub fn line(stages: &Stages, total: Duration) -> String {
    let reading: Map<String, Value> = READING
        .iter()
        .map(|stage| ((*stage).to_owned(), micros(stages.of(stage))))
        .collect();

    json!({
        "timings": {
            "reading": reading,
            "print": micros(stages.of("print")),
            "total": micros(total),
        }
    })
    .to_string()
}

/// Writes the line on standard error. A standard error that cannot be written to is passed over, so a
/// run that succeeded never ends as a fault (spec TIME-4a).
pub fn write(line: &str) {
    let _ = writeln!(io::stderr(), "{line}");
}

/// A time as whole microseconds.
fn micros(taken: Duration) -> Value {
    Value::from(u64::try_from(taken.as_micros()).unwrap_or(u64::MAX))
}
