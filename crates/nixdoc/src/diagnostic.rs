//! Structured diagnostics emitted while parsing documentation.

use alloc::string::String;
use core::ops::Range;

/// A byte range in an input string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceSpan {
  /// Inclusive byte offset.
  pub start: usize,
  /// Exclusive byte offset.
  pub end:   usize,
}

impl SourceSpan {
  /// Creates a span from byte offsets.
  pub const fn new(start: usize, end: usize) -> Self {
    Self { start, end }
  }

  /// Returns the span as a standard range.
  pub const fn range(self) -> Range<usize> {
    self.start..self.end
  }
}

impl From<Range<usize>> for SourceSpan {
  fn from(range: Range<usize>) -> Self {
    Self::new(range.start, range.end)
  }
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
  /// Parsing cannot produce the requested result.
  Error,
  /// Input is accepted but probably unintended.
  Warning,
}

/// A stable, machine-readable diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticCode {
  /// A convention section has no content.
  EmptySection,
  /// A fenced code block has no closing fence.
  UnclosedCodeFence,
}

/// A structured diagnostic tied to an input byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
  /// Stable diagnostic code.
  pub code:     DiagnosticCode,
  /// Severity of the problem.
  pub severity: Severity,
  /// Human-readable explanation.
  pub message:  String,
  /// Relevant byte range in the original input.
  pub span:     SourceSpan,
}
