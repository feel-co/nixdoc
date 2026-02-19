use thiserror::Error;

/// Errors that can occur while parsing a Nixdoc comment.
#[derive(Debug, Error, PartialEq, Clone)]
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
/// Warnings indicate structurally valid but potentially problematic content
/// (e.g. an empty section, or an unrecognized section heading).
#[derive(Debug, Clone, PartialEq)]
pub struct ParseWarning {
    /// The category of this warning.
    pub kind: WarningKind,
    /// A human-readable message describing the issue.
    pub message: String,
}

/// The category of a [`ParseWarning`].
#[derive(Debug, Clone, PartialEq)]
pub enum WarningKind {
    /// A section heading was found but the section has no body content.
    EmptySection,
    /// The section heading is not one of the recognized Nixdoc section names.
    UnknownSection,
}
