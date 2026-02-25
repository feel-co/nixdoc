// All tests in this file require `--features serde`.

#[cfg(feature = "serde")]
use nixdoc::{DocComment, ParseError, ParseWarning, SectionKind, WarningKind};

#[cfg(feature = "serde")]
fn parse(s: &str) -> DocComment {
    DocComment::parse(s).unwrap()
}

#[cfg(feature = "serde")]
fn json(v: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(v).unwrap()
}

#[test]
#[cfg(feature = "serde")]
fn roundtrip_minimal() {
    let doc = parse("/** The identity function. */");
    let serialized = serde_json::to_string(&doc).unwrap();
    let back: DocComment = serde_json::from_str(&serialized).unwrap();
    assert_eq!(doc, back);
}

#[test]
#[cfg(feature = "serde")]
fn roundtrip_full() {
    let doc = parse(
        "/**\n  f.\n\n  # Type\n\n  ```\n  f :: Int -> Int\n  ```\n\n  # Arguments\n\n  - [x] The input\n\n  # Example\n\n  ```nix\n  f 1\n  => 1\n  ```\n*/",
    );
    let serialized = serde_json::to_string(&doc).unwrap();
    let back: DocComment = serde_json::from_str(&serialized).unwrap();
    assert_eq!(doc, back);
}

#[test]
#[cfg(feature = "serde")]
fn roundtrip_with_warnings() {
    let doc = parse("/**\n  f.\n\n  # See Also\n\n  something\n*/");
    assert!(!doc.warnings.is_empty());
    let serialized = serde_json::to_string(&doc).unwrap();
    let back: DocComment = serde_json::from_str(&serialized).unwrap();
    assert_eq!(doc, back);
}

#[test]
#[cfg(feature = "serde")]
fn json_minimal() {
    use expect_test::expect;
    let doc = parse("/** The identity function. */");
    expect![[r#"
        {
          "raw_content": "The identity function.",
          "description": "The identity function.",
          "sections": [],
          "warnings": []
        }"#]]
    .assert_eq(&json(&doc));
}

#[test]
#[cfg(feature = "serde")]
fn json_with_sections() {
    use expect_test::expect;
    let doc = parse(
        "/**\n  f.\n\n  # Type\n\n  ```\n  f :: Int -> Int\n  ```\n\n  # Arguments\n\n  - [x] Input\n*/",
    );
    expect![[r#"
        {
          "raw_content": "f.\n\n# Type\n\n```\nf :: Int -> Int\n```\n\n# Arguments\n\n- [x] Input",
          "description": "f.",
          "sections": [
            {
              "heading": "Type",
              "content": "```\nf :: Int -> Int\n```"
            },
            {
              "heading": "Arguments",
              "content": "- [x] Input"
            }
          ],
          "warnings": []
        }"#]]
    .assert_eq(&json(&doc));
}

#[test]
#[cfg(feature = "serde")]
fn json_with_unknown_section_warning() {
    use expect_test::expect;
    let doc = parse("/**\n  f.\n\n  # See Also\n\n  something\n*/");
    expect![[r#"
        {
          "raw_content": "f.\n\n# See Also\n\nsomething",
          "description": "f.",
          "sections": [
            {
              "heading": "See Also",
              "content": "something"
            }
          ],
          "warnings": [
            {
              "kind": "UnknownSection",
              "message": "unrecognized section heading: 'See Also'"
            }
          ]
        }"#]]
    .assert_eq(&json(&doc));
}

#[test]
#[cfg(feature = "serde")]
fn parse_error_variants() {
    use expect_test::expect;
    expect![[r#"
        [
          "NotDocComment",
          "UnclosedComment",
          "EmptyComment"
        ]"#]]
    .assert_eq(&json(&vec![
        ParseError::NotDocComment,
        ParseError::UnclosedComment,
        ParseError::EmptyComment,
    ]));
}

#[test]
#[cfg(feature = "serde")]
fn warning_kind_variants() {
    use expect_test::expect;
    expect![[r#"
        [
          "EmptySection",
          "UnknownSection"
        ]"#]]
    .assert_eq(&json(&vec![
        WarningKind::EmptySection,
        WarningKind::UnknownSection,
    ]));
}

#[test]
#[cfg(feature = "serde")]
fn parse_warning_fields() {
    use expect_test::expect;
    let w = ParseWarning {
        kind: WarningKind::EmptySection,
        message: "empty section: \"Type\"".to_string(),
    };
    expect![[r#"
        {
          "kind": "EmptySection",
          "message": "empty section: \"Type\""
        }"#]]
    .assert_eq(&json(&w));
}

#[test]
#[cfg(feature = "serde")]
fn section_kind_known_variants() {
    use expect_test::expect;
    expect![[r#"
        [
          "Type",
          "Arguments",
          "Example",
          "Examples",
          "Note",
          "Notes",
          "Warning",
          "Deprecated"
        ]"#]]
    .assert_eq(&json(&vec![
        SectionKind::Type,
        SectionKind::Arguments,
        SectionKind::Example,
        SectionKind::Examples,
        SectionKind::Note,
        SectionKind::Notes,
        SectionKind::Warning,
        SectionKind::Deprecated,
    ]));
}

#[test]
#[cfg(feature = "serde")]
fn section_kind_unknown_variant() {
    use expect_test::expect;
    expect![[r#"
        {
          "Unknown": "see also"
        }"#]]
    .assert_eq(&json(&SectionKind::Unknown("see also".to_string())));
}

#[test]
#[cfg(feature = "serde")]
fn section_kind_unknown_roundtrip() {
    let original = SectionKind::Unknown("see also".to_string());
    let serialized = serde_json::to_string(&original).unwrap();
    let back: SectionKind = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original, back);
}
