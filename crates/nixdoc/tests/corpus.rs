use std::{ffi::OsStr, fs, path::PathBuf};

use expect_test::expect_file;
use nixdoc::DocComment;

fn dir_tests(dir: &str, ext: &str, get_actual: impl Fn(&PathBuf) -> String) {
    let base: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", dir]
        .iter()
        .collect();

    let mut entries: Vec<_> = fs::read_dir(&base)
        .unwrap_or_else(|_| panic!("missing fixture dir: {}", base.display()))
        .flatten()
        .filter(|e| e.path().extension() == Some(OsStr::new(ext)))
        .collect();
    entries.sort_by_key(|e| e.path());

    assert!(!entries.is_empty(), "no .{ext} files in {}", base.display());

    for entry in entries {
        let path = entry.path();
        let actual = get_actual(&path);
        expect_file![path.with_extension("expect")].assert_eq(&actual);
    }
}

/// Snapshot the full `DocComment` parse output for each `.txt` fixture file.
///
/// Each `.txt` file in `tests/fixtures/doc_comments/` should contain a raw
/// `/** ... */` comment exactly as extracted from Nix source. The test
/// parses it and compares against a sibling `.expect` file.
///
/// To regenerate expected files after an intentional change, run:
///
///   UPDATE_EXPECT=1 cargo test doc_comment_snapshots
#[test]
fn doc_comment_snapshots() {
    dir_tests("doc_comments", "txt", |path| {
        let input = fs::read_to_string(path).expect("read fixture");
        format!("{:#?}", DocComment::parse(&input))
    });
}
