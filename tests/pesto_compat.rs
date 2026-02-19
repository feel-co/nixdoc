use std::fs;
use std::path::{Path, PathBuf};

use expect_test::expect;
use nixdoc::{DocComment, ParseError, WarningKind};

fn collect_nix_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_nix_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("nix") {
            out.push(path);
        }
    }
    out
}

fn extract_doc_comments(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 2 < len {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' && bytes[i + 2] == b'*' {
            let start = i;
            i += 3;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    out.push(src[start..i].to_string());
                    break;
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn pesto_test_data() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/fixtures");

    assert!(
        assets.exists(),
        "fixtures not found at {}",
        assets.display()
    );

    let files = collect_nix_files(&assets);
    assert!(!files.is_empty(), "no .nix files found under assets/");

    let mut total = 0usize;
    let mut ok = 0usize;
    let mut empty = 0usize;
    let mut warn_unknown = 0usize;
    let mut warn_empty_section = 0usize;
    let mut unclosed: Vec<(PathBuf, String)> = Vec::new();

    for file in &files {
        let src = fs::read_to_string(file).expect("read file");
        for raw in extract_doc_comments(&src) {
            total += 1;
            match DocComment::parse(&raw) {
                Ok(doc) => {
                    ok += 1;
                    assert!(
                        !doc.raw_content.is_empty(),
                        "{}: empty raw_content",
                        file.display()
                    );
                    if !doc.description().is_empty() {
                        assert!(
                            doc.title().is_some(),
                            "{}: non-empty description but no title",
                            file.display()
                        );
                    }
                    for s in &doc.sections {
                        assert!(
                            !s.heading.is_empty(),
                            "{}: section with empty heading",
                            file.display()
                        );
                    }
                    for w in &doc.warnings {
                        match w.kind {
                            WarningKind::UnknownSection => warn_unknown += 1,
                            WarningKind::EmptySection => warn_empty_section += 1,
                        }
                    }
                }
                Err(ParseError::EmptyComment) => empty += 1,
                Err(ParseError::NotDocComment) => {}
                Err(ParseError::UnclosedComment) => {
                    unclosed.push((file.clone(), raw));
                }
            }
        }
    }

    for (path, raw) in &unclosed {
        eprintln!(
            "UNCLOSED {} - {:?}",
            path.display(),
            &raw[..raw.len().min(80)]
        );
    }

    assert!(
        unclosed.is_empty(),
        "{} unclosed comment(s)",
        unclosed.len()
    );

    // Snapshot corpus-wide statistics. If these numbers change, the diff makes
    // it obvious whether we regressed (counts go up unexpectedly) or improved
    // (warning counts drop after a parser fix).
    let stats = format!(
        "files: {}\ncomments: {}\nok: {}\nempty: {}\nwarn_unknown_section: {}\nwarn_empty_section: {}",
        files.len(),
        total,
        ok,
        empty,
        warn_unknown,
        warn_empty_section,
    );
    expect![[r#"
        files: 180
        comments: 402
        ok: 402
        empty: 0
        warn_unknown_section: 6
        warn_empty_section: 10"#]]
    .assert_eq(&stats);
}
