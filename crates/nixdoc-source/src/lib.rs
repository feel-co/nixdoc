//! Dependency-free RFC 145 source association.
//!
//! This crate lexes enough of Nix to associate documentation comments without
//! evaluating Nix or constructing an AST. The lexer remains an internal
//! implementation detail rather than a general-purpose Nix token API.

#![no_std]

extern crate alloc;

#[cfg(any(feature = "std", test))] extern crate std;

use alloc::{
  string::{String, ToString},
  vec,
  vec::Vec,
};

use nixdoc::{DocComment, ParseError, Severity, SourceSpan};

/// A stable, machine-readable source diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticCode {
  /// A documentation comment is empty.
  EmptyComment,
  /// Nix source contains an unterminated block comment.
  UnclosedBlockComment,
  /// A documentation comment has no following documentable node.
  OrphanDocComment,
}

/// A source-association diagnostic tied to an input byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
  /// Stable diagnostic code.
  pub code:     DiagnosticCode,
  /// Severity of the problem.
  pub severity: Severity,
  /// Human-readable explanation.
  pub message:  String,
  /// Relevant byte range in the original Nix source.
  pub span:     SourceSpan,
}

/// The kind of Nix construct documented by a comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocumentedNodeKind {
  /// An expression, including an anonymous lambda or an attribute value.
  Expression,
  /// An attribute or `let` binding.
  Binding,
  /// A formal in an attribute-pattern lambda.
  LambdaFormal,
}

/// A documentation comment associated with a Nix construct.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentedItem {
  /// Kind of documented construct.
  pub kind:         DocumentedNodeKind,
  /// Parsed documentation body.
  pub comment:      DocComment,
  /// Range of the complete `/** ... */` comment.
  pub comment_span: SourceSpan,
  /// Range of the construct's first lexical token.
  pub node_span:    SourceSpan,
  /// Static binding or formal name when one is available.
  pub name:         Option<String>,
}

/// Version of the serialized [`SourceDocument`] contract.
pub const SOURCE_SCHEMA_VERSION: u32 = 1;

/// Result of extracting documentation from one Nix source string.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceDocument {
  /// Version of this machine-readable structure.
  pub schema_version: u32,
  /// Associated documentation in source order.
  pub items:          Vec<DocumentedItem>,
  /// Non-fatal extraction diagnostics.
  pub diagnostics:    Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
  Word,
  Punctuation(u8),
  DocComment,
  Comment,
  String,
}

#[derive(Debug, Clone, Copy)]
struct Token {
  kind: TokenKind,
  span: SourceSpan,
}

impl Token {
  fn text(self, source: &str) -> &str {
    &source[self.span.start..self.span.end]
  }

  fn is_punctuation(self, byte: u8) -> bool {
    self.kind == TokenKind::Punctuation(byte)
  }

  fn is_trivia(self) -> bool {
    matches!(self.kind, TokenKind::Comment | TokenKind::DocComment)
  }
}

fn is_word_byte(byte: u8) -> bool {
  byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'\'')
}

fn lex_raw(source: &str, diagnostics: &mut Vec<Diagnostic>) -> Vec<Token> {
  let bytes = source.as_bytes();
  let mut tokens = Vec::new();
  let mut index = 0;

  while index < bytes.len() {
    if bytes[index].is_ascii_whitespace() {
      index += 1;
      continue;
    }
    let start = index;
    if bytes[index..].starts_with(b"#") {
      index += 1;
      while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
      }
      tokens.push(Token {
        kind: TokenKind::Comment,
        span: SourceSpan::new(start, index),
      });
      continue;
    }
    if bytes[index..].starts_with(b"/*") {
      let is_doc = bytes[index..].starts_with(b"/**")
        && !bytes[index..].starts_with(b"/**/");
      index += 2;
      let mut depth = 1usize;
      while index < bytes.len() && depth != 0 {
        if bytes[index..].starts_with(b"/*") {
          depth += 1;
          index += 2;
        } else if bytes[index..].starts_with(b"*/") {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      if depth != 0 {
        diagnostics.push(Diagnostic {
          code:     DiagnosticCode::UnclosedBlockComment,
          severity: Severity::Error,
          message:  "block comment is not closed".into(),
          span:     SourceSpan::new(start, bytes.len()),
        });
      }
      tokens.push(Token {
        kind: if is_doc {
          TokenKind::DocComment
        } else {
          TokenKind::Comment
        },
        span: SourceSpan::new(start, index),
      });
      continue;
    }
    if bytes[index..].starts_with(b"''") {
      index += 2;
      while index < bytes.len() {
        if bytes[index..].starts_with(b"''$") {
          index += 3;
        } else if bytes[index..].starts_with(b"''") {
          index += 2;
          break;
        } else {
          index += 1;
        }
      }
      tokens.push(Token {
        kind: TokenKind::String,
        span: SourceSpan::new(start, index),
      });
      continue;
    }
    if bytes[index] == b'"' {
      index += 1;
      while index < bytes.len() {
        match bytes[index] {
          b'\\' => index = (index + 2).min(bytes.len()),
          b'"' => {
            index += 1;
            break;
          },
          _ => index += 1,
        }
      }
      tokens.push(Token {
        kind: TokenKind::String,
        span: SourceSpan::new(start, index),
      });
      continue;
    }
    if is_word_byte(bytes[index]) {
      index += 1;
      while index < bytes.len() && is_word_byte(bytes[index]) {
        index += 1;
      }
      tokens.push(Token {
        kind: TokenKind::Word,
        span: SourceSpan::new(start, index),
      });
      continue;
    }
    index += 1;
    tokens.push(Token {
      kind: TokenKind::Punctuation(bytes[start]),
      span: SourceSpan::new(start, index),
    });
  }
  tokens
}

fn shifted_tokens(
  source: &str,
  base: usize,
  diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Token> {
  let mut local_diagnostics = Vec::new();
  let mut tokens = lex_raw(source, &mut local_diagnostics);
  for token in &mut tokens {
    token.span.start += base;
    token.span.end += base;
  }
  diagnostics.extend(local_diagnostics.into_iter().map(|mut diagnostic| {
    diagnostic.span.start += base;
    diagnostic.span.end += base;
    diagnostic
  }));
  tokens
}

fn interpolation_start(
  text: &str,
  from: usize,
  indented: bool,
) -> Option<usize> {
  let bytes = text.as_bytes();
  let mut index = from;
  while index + 1 < bytes.len() {
    if !indented && bytes[index] == b'\\' {
      index += 2;
    } else if indented && bytes[index..].starts_with(b"''$") {
      index += 3;
    } else if bytes[index..].starts_with(b"${") {
      return Some(index);
    } else {
      index += 1;
    }
  }
  None
}

fn expand_string(
  source: &str,
  token: Token,
  diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Token> {
  let text = token.text(source);
  let indented = text.starts_with("''");
  let delimiter = if indented { 2 } else { 1 };
  let mut output = Vec::new();
  let mut cursor = delimiter;
  let mut segment_start = 0;

  while let Some(open) = interpolation_start(text, cursor, indented) {
    let base = token.span.start + open + 2;
    let fragment_end = text.len().saturating_sub(delimiter);
    let fragment = &text[open + 2..fragment_end];
    let diagnostic_start = diagnostics.len();
    let raw = shifted_tokens(fragment, base, diagnostics);
    let mut depth = 1usize;
    let mut close = None;
    for (index, candidate) in raw.iter().enumerate() {
      if candidate.is_punctuation(b'{') {
        depth += 1;
      } else if candidate.is_punctuation(b'}') {
        depth -= 1;
        if depth == 0 {
          close = Some((index, candidate.span.end - token.span.start));
          break;
        }
      }
    }
    let Some((close_index, close_end)) = close else {
      break;
    };
    let close_offset = token.span.start + close_end;
    let fragment_diagnostics = diagnostics.split_off(diagnostic_start);
    diagnostics.extend(
      fragment_diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.span.start < close_offset),
    );
    if segment_start < open {
      output.push(Token {
        kind: TokenKind::String,
        span: SourceSpan::new(
          token.span.start + segment_start,
          token.span.start + open,
        ),
      });
    }
    output.push(Token {
      kind: TokenKind::Punctuation(b'$'),
      span: SourceSpan::new(
        token.span.start + open,
        token.span.start + open + 1,
      ),
    });
    output.push(Token {
      kind: TokenKind::Punctuation(b'{'),
      span: SourceSpan::new(
        token.span.start + open + 1,
        token.span.start + open + 2,
      ),
    });
    for inner in raw.into_iter().take(close_index) {
      if inner.kind == TokenKind::String {
        output.extend(expand_string(source, inner, diagnostics));
      } else {
        output.push(inner);
      }
    }
    output.push(Token {
      kind: TokenKind::Punctuation(b'}'),
      span: SourceSpan::new(
        token.span.start + close_end - 1,
        token.span.start + close_end,
      ),
    });
    cursor = close_end;
    segment_start = close_end;
  }

  if output.is_empty() {
    return vec![token];
  }
  if segment_start < text.len() {
    output.push(Token {
      kind: TokenKind::String,
      span: SourceSpan::new(token.span.start + segment_start, token.span.end),
    });
  }
  output
}

fn lex(source: &str, diagnostics: &mut Vec<Diagnostic>) -> Vec<Token> {
  let raw = lex_raw(source, diagnostics);
  let mut tokens = Vec::new();
  for token in raw {
    if token.kind == TokenKind::String {
      tokens.extend(expand_string(source, token, diagnostics));
    } else {
      tokens.push(token);
    }
  }
  tokens
}

fn next_non_trivia(tokens: &[Token], from: usize) -> Option<usize> {
  (from..tokens.len()).find(|index| !tokens[*index].is_trivia())
}

fn assignment_after(tokens: &[Token], from: usize) -> Option<usize> {
  let mut nesting = 0usize;
  for (index, token) in tokens.iter().enumerate().skip(from) {
    let neighboring_operator = token.is_punctuation(b'=')
      && (index.checked_sub(1).is_some_and(|previous| {
        matches!(
          tokens[previous].kind,
          TokenKind::Punctuation(b'=' | b'!' | b'<' | b'>')
        )
      }) || tokens
        .get(index + 1)
        .is_some_and(|next| next.is_punctuation(b'=')));
    match token.kind {
      TokenKind::Punctuation(b'{' | b'[' | b'(') => nesting += 1,
      TokenKind::Punctuation(b'}' | b']' | b')') if nesting > 0 => nesting -= 1,
      TokenKind::Punctuation(b'=') if nesting == 0 && !neighboring_operator => {
        return Some(index);
      },
      TokenKind::Punctuation(b';' | b',' | b':') if nesting == 0 => {
        return None;
      },
      _ => {},
    }
  }
  None
}

fn pattern_ranges(tokens: &[Token]) -> Vec<(usize, usize)> {
  let mut stack = Vec::new();
  let mut ranges = Vec::new();
  for (index, token) in tokens.iter().enumerate() {
    if token.is_punctuation(b'{') {
      stack.push(index);
    } else if token.is_punctuation(b'}') {
      let Some(open) = stack.pop() else {
        continue;
      };
      let Some(next) = next_non_trivia(tokens, index + 1) else {
        continue;
      };
      let lambda = tokens[next].is_punctuation(b':')
        || (tokens[next].is_punctuation(b'@')
          && next_non_trivia(tokens, next + 1)
            .and_then(|name| next_non_trivia(tokens, name + 1))
            .is_some_and(|colon| tokens[colon].is_punctuation(b':')));
      if lambda {
        ranges.push((open, index));
      }
    }
  }
  ranges
}

fn is_formal_target(
  tokens: &[Token],
  index: usize,
  ranges: &[(usize, usize)],
) -> bool {
  if !ranges
    .iter()
    .any(|(start, end)| *start < index && index < *end)
  {
    return false;
  }
  let previous = (0..index)
    .rev()
    .find(|candidate| !tokens[*candidate].is_trivia());
  previous.is_some_and(|previous| {
    tokens[previous].is_punctuation(b'{')
      || tokens[previous].is_punctuation(b',')
  })
}

fn parse_comment(source: &str, token: Token) -> Result<DocComment, ParseError> {
  DocComment::parse(token.text(source))
}

/// Associates RFC 145 documentation comments with Nix source constructs.
///
/// Direct documentation before an attribute value takes precedence over
/// documentation before its binding. Non-documentation comments and whitespace
/// may appear between a documentation comment and its target.
pub fn extract(source: &str) -> SourceDocument {
  let mut diagnostics = Vec::new();
  let tokens = lex(source, &mut diagnostics);
  let patterns = pattern_ranges(&tokens);
  let mut items = Vec::new();

  for (doc_index, doc_token) in tokens.iter().copied().enumerate() {
    if doc_token.kind != TokenKind::DocComment {
      continue;
    }
    let mut cursor = doc_index + 1;
    while cursor < tokens.len() && tokens[cursor].kind == TokenKind::Comment {
      cursor += 1;
    }
    if cursor < tokens.len() && tokens[cursor].kind == TokenKind::DocComment {
      diagnostics.push(Diagnostic {
        code:     DiagnosticCode::OrphanDocComment,
        severity: Severity::Warning,
        message:  "documentation comment is superseded by a closer \
                   documentation comment"
          .into(),
        span:     doc_token.span,
      });
      continue;
    }
    let Some(target_index) = next_non_trivia(&tokens, cursor) else {
      diagnostics.push(Diagnostic {
        code:     DiagnosticCode::OrphanDocComment,
        severity: Severity::Warning,
        message:  "documentation comment has no following construct".into(),
        span:     doc_token.span,
      });
      continue;
    };
    let target = tokens[target_index];
    let assignment = assignment_after(&tokens, target_index);
    let is_inherit =
      target.kind == TokenKind::Word && target.text(source) == "inherit";
    let kind = if is_formal_target(&tokens, target_index, &patterns) {
      DocumentedNodeKind::LambdaFormal
    } else if assignment.is_some() || is_inherit {
      DocumentedNodeKind::Binding
    } else {
      DocumentedNodeKind::Expression
    };

    if let Some(equal) = assignment {
      let rhs_doc = (equal + 1..tokens.len())
        .find(|index| tokens[*index].kind != TokenKind::Comment);
      if rhs_doc
        .is_some_and(|index| tokens[index].kind == TokenKind::DocComment)
      {
        continue;
      }
    }

    match parse_comment(source, doc_token) {
      Ok(comment) => {
        let name_token = if is_inherit {
          next_non_trivia(&tokens, target_index + 1)
            .filter(|index| tokens[*index].kind == TokenKind::Word)
            .map(|index| tokens[index])
        } else {
          (target.kind == TokenKind::Word).then_some(target)
        };
        let name = name_token.map(|token| token.text(source).into());
        items.push(DocumentedItem {
          kind,
          comment,
          comment_span: doc_token.span,
          node_span: target.span,
          name,
        });
      },
      Err(error) => {
        diagnostics.push(Diagnostic {
          code:     DiagnosticCode::EmptyComment,
          severity: Severity::Error,
          message:  error.to_string(),
          span:     doc_token.span,
        })
      },
    }
  }

  SourceDocument {
    schema_version: SOURCE_SCHEMA_VERSION,
    items,
    diagnostics,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn implements_rfc_placement_and_precedence_examples() {
    let source = r#"
      /** anonymous */ x: x;
      { /** binding */ assigned = x: x;
        value = /** expression */ 1;
        /** farther */ preferred = /** closer */ 2;
      }
      let /** let binding */ id = x: x; in id
      /** lambda */ { /** formal */ a }: a
    "#;
    let document = extract(source);
    assert!(
      document.diagnostics.is_empty(),
      "{:?}",
      document.diagnostics
    );
    let titles: Vec<_> = document
      .items
      .iter()
      .filter_map(|item| item.comment.title())
      .collect();
    assert_eq!(titles, [
      "anonymous",
      "binding",
      "expression",
      "closer",
      "let binding",
      "lambda",
      "formal"
    ]);
    assert!(document.items.iter().any(|item| {
      item.kind == DocumentedNodeKind::LambdaFormal
        && item.name.as_deref() == Some("a")
    }));
  }

  #[test]
  fn ignores_comment_markers_in_strings() {
    let document =
      extract(r#"{ text = "/** not docs */"; /** docs */ value = 1; }"#);
    assert_eq!(document.items.len(), 1);
    assert_eq!(document.items[0].name.as_deref(), Some("value"));
  }

  #[test]
  fn extracts_documentation_inside_string_interpolations() {
    let document = extract(r#""prefix ${/** interpolation */ 1} suffix""#);
    assert_eq!(document.items.len(), 1);
    assert_eq!(document.items[0].comment.title(), Some("interpolation"));
    assert_eq!(document.items[0].kind, DocumentedNodeKind::Expression);
  }

  #[test]
  fn does_not_confuse_equality_with_a_binding() {
    let document = extract("/** comparison */ left == right");
    assert_eq!(document.items[0].kind, DocumentedNodeKind::Expression);
  }

  #[test]
  fn recognizes_inherit_as_a_binding() {
    let document = extract("{ /** inherited binding */ inherit value; }");
    assert_eq!(document.items[0].kind, DocumentedNodeKind::Binding);
    assert_eq!(document.items[0].name.as_deref(), Some("value"));
  }

  #[test]
  fn distinguishes_formals_from_documented_default_expressions() {
    let document = extract("{ /** formal */ value ? /** default */ 1 }: value");
    assert_eq!(document.items.len(), 2);
    assert_eq!(document.items[0].kind, DocumentedNodeKind::LambdaFormal);
    assert_eq!(document.items[1].kind, DocumentedNodeKind::Expression);
  }
}
