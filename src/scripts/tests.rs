//! The pre-read: which files it reads, that its results don't depend on the number of threads or on
//! threads that could not start, and how many threads it takes (parallel reading spec PTEST-1,
//! PTEST-1a, PTEST-1b).

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, ErrorKind};
use std::num::NonZeroUsize;
use std::path::PathBuf;

use super::{ahead, threads, wanted};
use crate::files::{File, Kind};
use crate::tree::Tree;

/// What reading one file gave, in a form two readings can be compared by: its bytes, or its error's
/// kind, since `io::Error` can't be compared.
type Seen = Option<Result<Vec<u8>, ErrorKind>>;

/// Which threads are refused, in words, and the refusal: whether the thread of each number may start.
type Refusal = (&'static str, fn(usize) -> bool);

/// A folder of the test's own under the system's temporary folder, removed when the test ends.
struct Folder(PathBuf);

impl Folder {
    /// A new, empty folder named for the test and this process.
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("yak-analyzer-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        fs::create_dir_all(&path).expect("the folder can be made");

        Self(path)
    }

    /// Writes a file at the path given, from the folder, making its folders.
    fn write(&self, path: &str, content: &str) {
        let full = self.0.join(path);

        fs::create_dir_all(full.parent().expect("a file has a folder"))
            .expect("the folders can be made");
        fs::write(full, content).expect("the file can be written");
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A tracked file of the list.
fn listed(path: &str, kind: Kind) -> File {
    File {
        path: path.to_owned(),
        kind,
        tracked: true,
    }
}

/// A project of twelve scripts, a script declared as data, a `package.json`, a listed script that is
/// gone and a folder named like a script, with its file list in path order.
fn project(folder: &Folder) -> Vec<File> {
    let mut files: Vec<File> = (0..12)
        .map(|number| {
            let path = format!("src/f{number:02}.ts");

            folder.write(&path, &format!("export const n{number} = {number};\n"));

            listed(&path, Kind::Script)
        })
        .collect();

    folder.write("src/data.ts", "export const data = 1;\n");
    folder.write("package.json", "{}\n");
    fs::create_dir_all(folder.0.join("src/folder.ts")).expect("the folder can be made");
    files.push(listed("src/data.ts", Kind::Script));
    files.push(listed("package.json", Kind::Manifest));
    files.push(listed("src/gone.ts", Kind::Script));
    files.push(listed("src/folder.ts", Kind::Script));
    files.sort_by(|one, other| one.path.cmp(&other.path));

    files
}

/// The files declared as data in `project`.
fn data() -> BTreeSet<String> {
    BTreeSet::from(["src/data.ts".to_owned()])
}

/// What the pre-read gave, as it can be compared.
fn seen(results: Vec<Option<io::Result<Vec<u8>>>>) -> Vec<Seen> {
    results
        .into_iter()
        .map(|result| result.map(|bytes| bytes.map_err(|error| error.kind())))
        .collect()
}

/// Reading each file one after another, as the loop did before the pre-read.
fn one_by_one(tree: &Tree, files: &[&File]) -> Vec<Seen> {
    files
        .iter()
        .map(|file| Some(tree.read(&file.path).map_err(|error| error.kind())))
        .collect()
}

#[test]
fn reads_ahead_only_the_files_the_loop_reads() {
    let folder = Folder::new("wanted");
    let files = project(&folder);
    let data = data();
    let paths: Vec<&str> = wanted(&files, &data)
        .iter()
        .map(|file| file.path.as_str())
        .collect();

    assert_eq!(paths.len(), 14, "{paths:?}");
    assert!(
        !paths.contains(&"src/data.ts"),
        "a data file is never read: {paths:?}"
    );
    assert!(
        !paths.contains(&"package.json"),
        "a `package.json` is not a script: {paths:?}"
    );
    assert!(
        paths.contains(&"src/gone.ts") && paths.contains(&"src/folder.ts"),
        "{paths:?}"
    );
}

#[test]
fn gives_the_same_results_in_the_same_places_whatever_the_number_of_threads() {
    let folder = Folder::new("threads");
    let files = project(&folder);
    let data = data();
    let wanted = wanted(&files, &data);
    let tree = Tree::working(folder.0.clone());
    let expected = one_by_one(&tree, &wanted);
    let place = |path: &str| {
        wanted
            .iter()
            .position(|file| file.path == path)
            .expect("the file is wanted")
    };

    assert_eq!(
        expected[place("src/gone.ts")],
        Some(Err(ErrorKind::NotFound))
    );
    assert!(
        matches!(expected[place("src/folder.ts")], Some(Err(_))),
        "a folder cannot be read as a file"
    );

    for count in [1, 2, 3, 8] {
        assert_eq!(
            seen(ahead(&tree, &wanted, count, |_| true)),
            expected,
            "{count} threads"
        );
    }
}

#[test]
fn reads_every_file_when_threads_cannot_start() {
    let folder = Folder::new("refused");
    let files = project(&folder);
    let data = data();
    let wanted = wanted(&files, &data);
    let tree = Tree::working(folder.0.clone());
    let expected = one_by_one(&tree, &wanted);
    let refusals: [Refusal; 2] = [
        ("every thread", |_| false),
        ("the second thread", |number| number != 2),
    ];

    for (refused, may_start) in refusals {
        assert_eq!(
            seen(ahead(&tree, &wanted, 4, may_start)),
            expected,
            "{refused} refused"
        );
    }
}

#[test]
fn counts_the_threads_from_the_machine_and_the_files() {
    let sixteen = || Ok(NonZeroUsize::new(16).expect("16 is not 0"));
    let unknown = || Err(io::Error::other("the parallelism is unknown"));

    assert_eq!(threads(unknown(), 2_744), 1);
    assert_eq!(threads(sixteen(), 3), 3);
    assert_eq!(threads(sixteen(), 2_744), 16);
    assert_eq!(threads(sixteen(), 0), 0);
    assert_eq!(threads(unknown(), 0), 0);
}
