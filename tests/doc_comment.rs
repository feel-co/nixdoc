use nixdoc::*;

#[test]
fn is_doc_comment_true() {
    assert!(DocComment::is_doc_comment("/** hello */"));
    assert!(DocComment::is_doc_comment("  /** hello */  "));
}

#[test]
fn is_doc_comment_false() {
    assert!(!DocComment::is_doc_comment("/* not doc */"));
    assert!(!DocComment::is_doc_comment("// line comment"));
    assert!(!DocComment::is_doc_comment("plain text"));
}

#[test]
fn error_not_doc_comment() {
    assert_eq!(
        DocComment::parse("/* regular */"),
        Err(ParseError::NotDocComment)
    );
}

#[test]
fn error_unclosed() {
    assert_eq!(
        DocComment::parse("/** unclosed"),
        Err(ParseError::UnclosedComment)
    );
}

#[test]
fn error_empty() {
    assert_eq!(DocComment::parse("/** */"), Err(ParseError::EmptyComment));
    assert_eq!(DocComment::parse("/***/"), Err(ParseError::EmptyComment));
    assert_eq!(
        DocComment::parse("/**\n   \n*/"),
        Err(ParseError::EmptyComment)
    );
}

#[test]
fn single_line_simple() {
    let doc = DocComment::parse("/** The identity function. */").unwrap();
    assert_eq!(doc.title(), Some("The identity function."));
    assert_eq!(doc.description(), "The identity function.");
    assert!(doc.sections.is_empty());
}

#[test]
fn single_line_no_sections() {
    let doc = DocComment::parse("/** bitwise \"and\" */").unwrap();
    assert_eq!(doc.title(), Some("bitwise \"and\""));
}

#[test]
fn multiline_description_only() {
    let input = "/**\n  Line one.\n  Line two.\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.title(), Some("Line one."));
    assert!(doc.description().contains("Line one."));
    assert!(doc.description().contains("Line two."));
    assert!(doc.sections.is_empty());
}

#[test]
fn multiline_with_blank_separator() {
    let input = "/**\n  Short summary.\n\n  Longer description here.\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.title(), Some("Short summary."));
    assert!(doc.description().contains("Longer description here."));
}

#[test]
fn type_sig_extracted() {
    let input = "/**\n  f.\n\n  # Type\n\n  ```\n  f :: Int -> Int\n  ```\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.type_sig(), Some("f :: Int -> Int\n".to_string()));
}

#[test]
fn type_sig_with_lang_specifier() {
    let input = "/**\n  f.\n\n  # Type\n\n  ```nix\n  id :: a -> a\n  ```\n*/";
    let doc = DocComment::parse(input).unwrap();
    // Language specifier on type blocks is unusual but should not break extraction.
    assert_eq!(doc.type_sig(), Some("id :: a -> a\n".to_string()));
}

#[test]
fn type_sig_none_when_absent() {
    let doc = DocComment::parse("/** Simple. */").unwrap();
    assert_eq!(doc.type_sig(), None);
}

#[test]
fn type_sig_legacy_inline() {
    // Pre-RFC145 style: type embedded directly in the description.
    let input = r#"/**
  Merge two attribute sets shallowly, right side trumps left
  mergeAttrs :: attrs -> attrs -> attrs
*/"#;
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(
        doc.type_sig(),
        Some("mergeAttrs :: attrs -> attrs -> attrs".to_string())
    );
}

#[test]
fn type_sig_modern_takes_precedence_over_legacy() {
    // When both a `# Type` section and a legacy annotation are present,
    // the modern format wins.
    let input = "/**\n  f.\n  f :: LegacyType\n\n  # Type\n\n  ```\n  f :: ModernType\n  ```\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.type_sig(), Some("f :: ModernType\n".to_string()));
}

#[test]
fn type_sig_prose_with_double_colon_not_extracted() {
    // A sentence containing `::` with spaces around the identifier must
    // not be mistakenly treated as a type annotation.
    let input = "/** A value of type foo :: bar is sometimes returned. */";
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.type_sig(), None);
}

#[test]
fn type_sig_four_backtick_fence() {
    // 4-backtick type fences (needed when the type contains ```) must work.
    let input = "/**\n  f.\n\n  # Type\n\n  ````\n  f :: Int -> Int\n  ````\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.type_sig(), Some("f :: Int -> Int\n".to_string()));
}

#[test]
fn arguments_basic() {
    let input = "/**\n  f.\n\n  # Arguments\n\n  - [a] First number\n  - [b] Second number\n*/";
    let doc = DocComment::parse(input).unwrap();
    let args = doc.arguments();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name, "a");
    assert_eq!(args[0].description, "First number");
    assert_eq!(args[1].name, "b");
    assert_eq!(args[1].description, "Second number");
}

#[test]
fn arguments_empty_description() {
    let input = "/**\n  f.\n\n  # Arguments\n\n  - [val]\n  - [functions]\n*/";
    let doc = DocComment::parse(input).unwrap();
    let args = doc.arguments();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name, "val");
    assert_eq!(args[0].description, "");
    assert_eq!(args[1].name, "functions");
    assert_eq!(args[1].description, "");
}

#[test]
fn arguments_empty_when_no_section() {
    let doc = DocComment::parse("/** No args. */").unwrap();
    assert!(doc.arguments().is_empty());
}

#[test]
fn examples_basic() {
    let input = "/**\n  f.\n\n  # Example\n\n  ```nix\n  f 1\n  => 1\n  ```\n*/";
    let doc = DocComment::parse(input).unwrap();
    let examples = doc.examples();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].language, Some("nix".to_string()));
    assert!(examples[0].code.contains("f 1"));
}

#[test]
fn examples_empty_when_no_section() {
    let doc = DocComment::parse("/** No examples. */").unwrap();
    assert!(doc.examples().is_empty());
}

#[test]
fn deprecated_detected() {
    let input = "/**\n  Old fn.\n\n  # Deprecated\n\n  Use `newFn` instead.\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert!(doc.is_deprecated());
    assert_eq!(doc.deprecation_notice(), Some("Use `newFn` instead."));
}

#[test]
fn not_deprecated_by_default() {
    let doc = DocComment::parse("/** Active fn. */").unwrap();
    assert!(!doc.is_deprecated());
    assert_eq!(doc.deprecation_notice(), None);
}

#[test]
fn notes_extracted() {
    let input = "/**\n  f.\n\n  # Note\n\n  Be careful.\n*/";
    let doc = DocComment::parse(input).unwrap();
    let notes = doc.notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0], "Be careful.");
}

#[test]
fn warnings_extracted() {
    let input = "/**\n  f.\n\n  # Warning\n\n  Don't use lightly.\n*/";
    let doc = DocComment::parse(input).unwrap();
    let warnings = doc.warnings_content();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0], "Don't use lightly.");
}

#[test]
fn section_case_insensitive() {
    let input = "/**\n  f.\n\n  # Type\n\n  ```\n  a\n  ```\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert!(doc.section("type").is_some());
    assert!(doc.section("TYPE").is_some());
    assert!(doc.section("Type").is_some());
}

#[test]
fn section_not_found_returns_none() {
    let doc = DocComment::parse("/** f. */").unwrap();
    assert!(doc.section("Type").is_none());
}

#[test]
fn trivial_nix_id() {
    let input = r#"/**
  The identity function
  For when you need a function that does "nothing".

  # Type

  ```
  id :: a -> a
  ```

  # Arguments

  - [x] The value to return

*/"#;
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.title(), Some("The identity function"));
    assert_eq!(doc.type_sig(), Some("id :: a -> a\n".to_string()));
    let args = doc.arguments();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].name, "x");
    assert_eq!(args[0].description, "The value to return");
}

#[test]
fn trivial_nix_const() {
    let input = r#"/**
  The constant function
  Ignores the second argument. If called with only one argument,
  constructs a function that always returns a static value.

  # Example

  ```nix
  let f = const 5; in f 10
  => 5
  ```

  # Type

  ```
  const :: a -> b -> a
  ```

  # Arguments

  - [x] Value to return
  - [y] Value to ignore

*/"#;
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.title(), Some("The constant function"));
    assert_eq!(doc.type_sig(), Some("const :: a -> b -> a\n".to_string()));
    let args = doc.arguments();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name, "x");
    assert_eq!(args[0].description, "Value to return");
    assert_eq!(args[1].name, "y");
    assert_eq!(args[1].description, "Value to ignore");
    let examples = doc.examples();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].language, Some("nix".to_string()));
}

#[test]
fn trivial_nix_minimal() {
    // bitAnd, bitOr etc. have no sections but just a one-line description.
    let input = r#"/**
  bitwise "and"
*/"#;
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(doc.title(), Some("bitwise \"and\""));
    assert!(doc.sections.is_empty());
    assert!(doc.type_sig().is_none());
    assert!(doc.arguments().is_empty());
}

#[test]
fn trivial_nix_legacy_merge_attrs() {
    // Real example from nixpkgs trivial.nix: no `# Type` section, type
    // annotation embedded in the description body.
    let input = r#"/**
  Merge two attribute sets shallowly, right side trumps left
  mergeAttrs :: attrs -> attrs -> attrs
*/"#;
    let doc = DocComment::parse(input).unwrap();
    assert_eq!(
        doc.title(),
        Some("Merge two attribute sets shallowly, right side trumps left")
    );
    assert_eq!(
        doc.type_sig(),
        Some("mergeAttrs :: attrs -> attrs -> attrs".to_string())
    );
    assert!(doc.sections.is_empty());
}

#[test]
fn code_hash_inside_example_not_a_heading() {
    // Nix comments inside example code blocks must not be parsed as headings.
    let input = r#"/**
  pipe.

  # Example

  ```nix
  pipe 2 [
      (x: x + 2)  # 2 + 2 = 4
      (x: x * 2)  # 4 * 2 = 8
    ]
    => 8
  ```

  # Type

  ```
  pipe :: a -> [<functions>] -> <return type of last function>
  ```

*/"#;
    let doc = DocComment::parse(input).unwrap();
    // Should have exactly 2 sections, not more from the `#` inside the code.
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "Example");
    assert_eq!(doc.sections[1].heading, "Type");
    let examples = doc.examples();
    assert_eq!(examples.len(), 1);
    // The inline Nix comment must be preserved in the example code.
    assert!(examples[0].code.contains("# 2 + 2 = 4"));
}

#[test]
fn warns_on_unknown_section() {
    let input = "/**\n  f.\n\n  # See Also\n\n  Some content.\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert!(
        doc.warnings
            .iter()
            .any(|w| w.kind == WarningKind::UnknownSection)
    );
}

#[test]
fn warns_on_empty_section() {
    let input = "/**\n  f.\n\n  # Type\n\n  # Arguments\n\n  - [x] arg\n*/";
    let doc = DocComment::parse(input).unwrap();
    assert!(
        doc.warnings
            .iter()
            .any(|w| w.kind == WarningKind::EmptySection)
    );
}

#[test]
fn section_kind_from_heading() {
    assert_eq!(SectionKind::from_heading("Type"), SectionKind::Type);
    assert_eq!(SectionKind::from_heading("type"), SectionKind::Type);
    assert_eq!(
        SectionKind::from_heading("Arguments"),
        SectionKind::Arguments
    );
    assert_eq!(SectionKind::from_heading("args"), SectionKind::Arguments);
    assert_eq!(SectionKind::from_heading("Example"), SectionKind::Example);
    assert_eq!(SectionKind::from_heading("Examples"), SectionKind::Examples);
    assert_eq!(SectionKind::from_heading("Note"), SectionKind::Note);
    assert_eq!(SectionKind::from_heading("Warning"), SectionKind::Warning);
    assert_eq!(SectionKind::from_heading("caution"), SectionKind::Warning);
    assert_eq!(
        SectionKind::from_heading("Deprecated"),
        SectionKind::Deprecated
    );
    assert_eq!(
        SectionKind::from_heading("See Also"),
        SectionKind::Unknown("see also".to_string())
    );
}

#[test]
fn section_kind_is_known() {
    assert!(SectionKind::Type.is_known());
    assert!(!SectionKind::Unknown("x".to_string()).is_known());
}
