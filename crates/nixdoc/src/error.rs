use alloc::string::String;

use thiserror::Error;

/// Errors that can occur while parsing a Nixdoc comment.
#[derive(Debug, Error, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseError {
  /// The input is not a doc comment; it does not start with `/**`.
  #[error("not a doc comment: input must start with '/**'")]
  NotDocComment,

  /// The doc comment is missing its closing `*/` terminator.
  #[error("unclosed doc comment: missing '*/' terminator")]
  UnclosedComment,

  /// The doc comment has no content after stripping delimiters and normalizing.
  #[error("empty doc comment")]
  EmptyComment,
}

/// A non-fatal warning produced during parsing.
///
/// Warnings indicate structurally valid but potentially problematic content,
/// such as an empty convention section.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseWarning {
  /// The category of this warning.
  pub kind:    WarningKind,
  /// A human-readable message describing the issue.
  pub message: String,
}

/// The category of a [`ParseWarning`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WarningKind {
  /// A section heading was found but the section has no body content.
  EmptySection,
  /// Retained for serialized API compatibility; custom headings are valid.
  UnknownSection,
}

#[cfg(test)]
mod tests {
  use alloc::string::ToString;

  use super::*;

  #[test]
  fn parse_error_not_doc_comment_display() {
    let err = ParseError::NotDocComment;
    assert_eq!(
      err.to_string(),
      "not a doc comment: input must start with '/**'"
    );
  }

  #[test]
  fn parse_error_unclosed_comment_display() {
    let err = ParseError::UnclosedComment;
    assert_eq!(
      err.to_string(),
      "unclosed doc comment: missing '*/' terminator"
    );
  }

  #[test]
  fn parse_error_empty_comment_display() {
    let err = ParseError::EmptyComment;
    assert_eq!(err.to_string(), "empty doc comment");
  }

  #[test]
  fn parse_error_variants() {
    assert_eq!(ParseError::NotDocComment, ParseError::NotDocComment);
    assert_eq!(ParseError::UnclosedComment, ParseError::UnclosedComment);
    assert_eq!(ParseError::EmptyComment, ParseError::EmptyComment);
  }

  #[test]
  fn parse_error_not_equal() {
    assert_ne!(ParseError::NotDocComment, ParseError::UnclosedComment);
    assert_ne!(ParseError::NotDocComment, ParseError::EmptyComment);
    assert_ne!(ParseError::UnclosedComment, ParseError::EmptyComment);
  }

  #[test]
  fn warning_kind_empty_section() {
    let warning = ParseWarning {
      kind:    WarningKind::EmptySection,
      message: "empty section: \"Type\"".to_string(),
    };
    assert_eq!(warning.kind, WarningKind::EmptySection);
    assert_eq!(warning.message, "empty section: \"Type\"");
  }

  #[test]
  fn warning_kind_unknown_section() {
    let warning = ParseWarning {
      kind:    WarningKind::UnknownSection,
      message: "unrecognized section heading: 'See Also'".to_string(),
    };
    assert_eq!(warning.kind, WarningKind::UnknownSection);
    assert_eq!(warning.message, "unrecognized section heading: 'See Also'");
  }

  #[test]
  fn warning_kinds_not_equal() {
    assert_ne!(WarningKind::EmptySection, WarningKind::UnknownSection);
  }

  #[test]
  fn parse_warning_clone() {
    let warning = ParseWarning {
      kind:    WarningKind::EmptySection,
      message: "test".to_string(),
    };
    let cloned = warning.clone();
    assert_eq!(warning, cloned);
  }
}
