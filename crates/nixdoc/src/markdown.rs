//! CommonMark parsing for documentation comments.
//!
//! Parsing is delegated to the CommonMark-compliant `markdown` crate. This
//! module exposes the block information needed by nixdoc while preserving all
//! other Markdown verbatim in section contents.

use alloc::{
  string::{String, ToString},
  vec::Vec,
};

use markdown::{ParseOptions, mdast::Node};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, SourceSpan};

/// A recognized top-level CommonMark block used by convention extractors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Block {
  /// A CommonMark heading, including ATX and setext headings.
  Heading {
    /// Heading level from one through six.
    level: u8,
    /// Plain textual heading content.
    text:  String,
    /// Byte range of the complete heading.
    span:  SourceSpan,
  },
  /// A fenced code block.
  FencedCode {
    /// Complete info string after the opening fence.
    info:   Option<String>,
    /// Code block contents.
    code:   String,
    /// Byte range including both fences when closed.
    span:   SourceSpan,
    /// Whether an explicit closing fence was found.
    closed: bool,
  },
}

fn span(node: &Node) -> Option<SourceSpan> {
  node
    .position()
    .map(|position| SourceSpan::new(position.start.offset, position.end.offset))
}

fn fenced_source(source: &str, span: SourceSpan) -> Option<(u8, usize)> {
  let line = source[span.start..]
    .split_once('\n')
    .map_or(&source[span.start..], |(line, _)| line);
  let rest = line.trim_start_matches(' ');
  if line.len() - rest.len() > 3 {
    return None;
  }
  let marker = *rest.as_bytes().first()?;
  if marker != b'`' && marker != b'~' {
    return None;
  }
  let length = rest.bytes().take_while(|byte| *byte == marker).count();
  (length >= 3).then_some((marker, length))
}

fn explicitly_closed(
  source: &str,
  span: SourceSpan,
  marker: u8,
  length: usize,
) -> bool {
  source[span.start..span.end].lines().skip(1).any(|line| {
    let rest = line.trim_start_matches(' ');
    let indent = line.len() - rest.len();
    let count = rest.bytes().take_while(|byte| *byte == marker).count();
    indent <= 3
      && count >= length
      && rest[count..]
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t'))
  })
}

/// Parses CommonMark and returns the top-level blocks used by nixdoc.
///
/// The parser implements CommonMark rather than a nixdoc-specific Markdown
/// subset. Syntax not represented by [`Block`] remains available unchanged in
/// the original section text.
pub fn scan(input: &str) -> (Vec<Block>, Vec<Diagnostic>) {
  let Ok(document) = markdown::to_mdast(input, &ParseOptions::default()) else {
    return (Vec::new(), vec![Diagnostic {
      code:     DiagnosticCode::UnclosedCodeFence,
      severity: Severity::Error,
      message:  "CommonMark input could not be parsed".into(),
      span:     SourceSpan::new(0, input.len()),
    }]);
  };
  let Node::Root(root) = document else {
    return (Vec::new(), Vec::new());
  };
  let mut blocks = Vec::new();
  let mut diagnostics = Vec::new();

  for node in root.children {
    let Some(node_span) = span(&node) else {
      continue;
    };
    match node {
      Node::Heading(heading) => {
        blocks.push(Block::Heading {
          level: heading.depth,
          text:  heading.children.iter().map(ToString::to_string).collect(),
          span:  node_span,
        })
      },
      Node::Code(code) => {
        let Some((marker, length)) = fenced_source(input, node_span) else {
          continue;
        };
        let closed = explicitly_closed(input, node_span, marker, length);
        if !closed {
          diagnostics.push(Diagnostic {
            code:     DiagnosticCode::UnclosedCodeFence,
            severity: Severity::Warning,
            message:  "fenced code block has no explicit closing fence".into(),
            span:     node_span,
          });
        }
        let info = code.lang.map(|language| {
          code.meta.map_or(language.clone(), |meta| {
            let mut info = language;
            info.push(' ');
            info.push_str(&meta);
            info
          })
        });
        let mut value = code.value;
        if !value.is_empty() {
          value.push('\n');
        }
        blocks.push(Block::FencedCode {
          info,
          code: value,
          span: node_span,
          closed,
        });
      },
      _ => {},
    }
  }
  (blocks, diagnostics)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scans_commonmark_headings_and_fences_without_scanning_code_as_headings() {
    let (blocks, diagnostics) =
      scan("  # Type ###\n\n ```nix\n # code\n ```\n");
    assert!(diagnostics.is_empty());
    assert_eq!(blocks.len(), 2);
    assert!(
      matches!(&blocks[0], Block::Heading { level: 1, text, .. } if text == "Type")
    );
    assert!(
      matches!(&blocks[1], Block::FencedCode { info: Some(info), code, closed: true, .. } if info == "nix" && code == "# code\n")
    );
  }

  #[test]
  fn recognizes_setext_headings() {
    let (blocks, _) = scan("Type\n====\n");
    assert!(
      matches!(&blocks[0], Block::Heading { level: 1, text, .. } if text == "Type")
    );
  }

  #[test]
  fn diagnoses_unclosed_fence() {
    let (_, diagnostics) = scan("```\nvalue\n");
    assert_eq!(diagnostics[0].code, DiagnosticCode::UnclosedCodeFence);
    assert_eq!(diagnostics[0].span, SourceSpan::new(0, 10));
  }
}
