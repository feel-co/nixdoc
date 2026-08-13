use alloc::string::{String, ToString};

/// A section in a Nixdoc comment.
///
/// Sections are delimited by level-1 Markdown headings (`# Section Name`).
/// The content is the normalized Markdown text of the section body.
///
/// # Example
///
/// Given a doc comment like (inner fence lines abbreviated as `...code...`):
///
/// ```nix
/// /**
///   My function.
///
///   # Type
///
///   (fenced code block)
///   foo :: Int -> Int
///   (end of fenced code block)
/// */
/// ```
///
/// The `# Type` heading produces a `Section` with `heading = "Type"` whose
/// `content` is the fenced code block for the type signature.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Section {
  /// The heading text (without the leading `# `).
  pub heading: String,
  /// The section body as normalized Markdown text.
  pub content: String,
}

impl Section {
  /// Returns the semantic kind of this section based on the heading.
  pub fn kind(&self) -> SectionKind {
    SectionKind::from_heading(&self.heading)
  }
}

/// The semantic kind of a nixpkgs-convention section.
///
/// RFC 145 permits arbitrary CommonMark. These variants describe optional
/// ecosystem conventions; any other heading produces [`Self::Unknown`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SectionKind {
  /// `# Type` - the Haskell-style type signature of the function.
  Type,

  /// `# Arguments` or `# Args` - documentation for each argument.
  Arguments,

  /// `# Example` - a single usage example.
  Example,

  /// `# Examples` - multiple usage examples.
  Examples,

  /// `# Note` - an informational note for readers.
  Note,

  /// `# Notes` - multiple informational notes.
  Notes,

  /// `# Warning`, `# Warnings`, or `# Caution` - an important caveat.
  Warning,

  /// `# Deprecated` - a deprecation notice.
  Deprecated,

  /// Any valid custom CommonMark section heading.
  Unknown(String),
}

impl SectionKind {
  /// Identify the section kind from a heading string (case-insensitive).
  ///
  /// # Examples
  ///
  /// ```rust
  /// use nixdoc::SectionKind;
  ///
  /// assert_eq!(SectionKind::from_heading("Type"), SectionKind::Type);
  /// assert_eq!(SectionKind::from_heading("type"), SectionKind::Type);
  /// assert_eq!(
  ///   SectionKind::from_heading("ARGUMENTS"),
  ///   SectionKind::Arguments
  /// );
  /// assert_eq!(
  ///   SectionKind::from_heading("See Also"),
  ///   SectionKind::Unknown("see also".to_string()),
  /// );
  /// ```
  pub fn from_heading(heading: &str) -> Self {
    match heading.to_lowercase().as_str() {
      "type" => Self::Type,
      "arguments" | "args" => Self::Arguments,
      "example" => Self::Example,
      "examples" => Self::Examples,
      "note" => Self::Note,
      "notes" => Self::Notes,
      "warning" | "warnings" | "caution" => Self::Warning,
      "deprecated" => Self::Deprecated,
      other => Self::Unknown(other.to_string()),
    }
  }

  /// Returns `true` if this is a recognized/known section kind.
  pub fn is_known(&self) -> bool {
    !matches!(self, Self::Unknown(_))
  }
}

/// A parsed function argument from the `# Arguments` section.
///
/// Arguments are expected in the form `- [name] Description text` where
/// `name` is the argument identifier and the rest is an optional description.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Argument {
  /// The argument name, as written inside `[...]`.
  pub name:        String,
  /// The argument description text (may be empty).
  pub description: String,
}

/// A code example extracted from an `# Example` or `# Examples` section.
///
/// Each example corresponds to a single fenced code block (` ``` ` or `~~~`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Example {
  /// The language specifier from the fenced code block, if present (e.g.,
  /// `"nix"`).
  pub language: Option<String>,
  /// The raw code content.
  pub code:     String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn section_kind_from_heading_type() {
    assert_eq!(SectionKind::from_heading("Type"), SectionKind::Type);
    assert_eq!(SectionKind::from_heading("type"), SectionKind::Type);
    assert_eq!(SectionKind::from_heading("TYPE"), SectionKind::Type);
  }

  #[test]
  fn section_kind_from_heading_arguments() {
    assert_eq!(
      SectionKind::from_heading("Arguments"),
      SectionKind::Arguments
    );
    assert_eq!(
      SectionKind::from_heading("arguments"),
      SectionKind::Arguments
    );
    assert_eq!(SectionKind::from_heading("Args"), SectionKind::Arguments);
    assert_eq!(SectionKind::from_heading("args"), SectionKind::Arguments);
    assert_eq!(
      SectionKind::from_heading("ARGUMENTS"),
      SectionKind::Arguments
    );
  }

  #[test]
  fn section_kind_from_heading_example() {
    assert_eq!(SectionKind::from_heading("Example"), SectionKind::Example);
    assert_eq!(SectionKind::from_heading("example"), SectionKind::Example);
    assert_eq!(SectionKind::from_heading("Examples"), SectionKind::Examples);
    assert_eq!(SectionKind::from_heading("examples"), SectionKind::Examples);
  }

  #[test]
  fn section_kind_from_heading_note() {
    assert_eq!(SectionKind::from_heading("Note"), SectionKind::Note);
    assert_eq!(SectionKind::from_heading("note"), SectionKind::Note);
    assert_eq!(SectionKind::from_heading("Notes"), SectionKind::Notes);
    assert_eq!(SectionKind::from_heading("notes"), SectionKind::Notes);
  }

  #[test]
  fn section_kind_from_heading_warning() {
    assert_eq!(SectionKind::from_heading("Warning"), SectionKind::Warning);
    assert_eq!(SectionKind::from_heading("warning"), SectionKind::Warning);
    assert_eq!(SectionKind::from_heading("Warnings"), SectionKind::Warning);
    assert_eq!(SectionKind::from_heading("warnings"), SectionKind::Warning);
    assert_eq!(SectionKind::from_heading("Caution"), SectionKind::Warning);
    assert_eq!(SectionKind::from_heading("caution"), SectionKind::Warning);
  }

  #[test]
  fn section_kind_from_heading_deprecated() {
    assert_eq!(
      SectionKind::from_heading("Deprecated"),
      SectionKind::Deprecated
    );
    assert_eq!(
      SectionKind::from_heading("deprecated"),
      SectionKind::Deprecated
    );
  }

  #[test]
  fn section_kind_from_heading_unknown() {
    assert_eq!(
      SectionKind::from_heading("See Also"),
      SectionKind::Unknown("see also".to_string())
    );
    assert_eq!(
      SectionKind::from_heading("Related"),
      SectionKind::Unknown("related".to_string())
    );
    assert_eq!(
      SectionKind::from_heading("Custom Section"),
      SectionKind::Unknown("custom section".to_string())
    );
  }

  #[test]
  fn section_kind_is_known() {
    assert!(SectionKind::Type.is_known());
    assert!(SectionKind::Arguments.is_known());
    assert!(SectionKind::Example.is_known());
    assert!(SectionKind::Examples.is_known());
    assert!(SectionKind::Note.is_known());
    assert!(SectionKind::Notes.is_known());
    assert!(SectionKind::Warning.is_known());
    assert!(SectionKind::Deprecated.is_known());
  }

  #[test]
  fn section_kind_unknown_is_not_known() {
    assert!(!SectionKind::Unknown("foo".to_string()).is_known());
  }

  #[test]
  fn section_kind_unknown_preserves_case() {
    assert_eq!(
      SectionKind::from_heading("See Also"),
      SectionKind::Unknown("see also".to_string())
    );
  }

  #[test]
  fn section_kind_eq() {
    assert_eq!(SectionKind::Type, SectionKind::Type);
    assert_eq!(SectionKind::Arguments, SectionKind::Arguments);
    assert_eq!(
      SectionKind::Unknown("foo".to_string()),
      SectionKind::Unknown("foo".to_string())
    );
  }

  #[test]
  fn section_kind_ne() {
    assert_ne!(SectionKind::Type, SectionKind::Arguments);
    assert_ne!(SectionKind::Type, SectionKind::Unknown("type".to_string()));
  }

  #[test]
  fn section_new() {
    let section = Section {
      heading: "Type".to_string(),
      content: "f :: Int -> Int".to_string(),
    };
    assert_eq!(section.heading, "Type");
    assert_eq!(section.content, "f :: Int -> Int");
  }

  #[test]
  fn section_kind() {
    let section = Section {
      heading: "Type".to_string(),
      content: "f :: Int -> Int".to_string(),
    };
    assert_eq!(section.kind(), SectionKind::Type);
  }

  #[test]
  fn section_clone() {
    let section = Section {
      heading: "Type".to_string(),
      content: "content".to_string(),
    };
    let cloned = section.clone();
    assert_eq!(section, cloned);
  }

  #[test]
  fn argument_new() {
    let arg = Argument {
      name:        "x".to_string(),
      description: "Input value".to_string(),
    };
    assert_eq!(arg.name, "x");
    assert_eq!(arg.description, "Input value");
  }

  #[test]
  fn argument_empty_description() {
    let arg = Argument {
      name:        "x".to_string(),
      description: String::new(),
    };
    assert_eq!(arg.name, "x");
    assert_eq!(arg.description, "");
  }

  #[test]
  fn argument_clone() {
    let arg = Argument {
      name:        "x".to_string(),
      description: "desc".to_string(),
    };
    let cloned = arg.clone();
    assert_eq!(arg, cloned);
  }

  #[test]
  fn example_new_with_language() {
    let ex = Example {
      language: Some("nix".to_string()),
      code:     "f 1".to_string(),
    };
    assert_eq!(ex.language, Some("nix".to_string()));
    assert_eq!(ex.code, "f 1");
  }

  #[test]
  fn example_new_without_language() {
    let ex = Example {
      language: None,
      code:     "some code".to_string(),
    };
    assert_eq!(ex.language, None);
    assert_eq!(ex.code, "some code");
  }

  #[test]
  fn example_clone() {
    let ex = Example {
      language: Some("nix".to_string()),
      code:     "code".to_string(),
    };
    let cloned = ex.clone();
    assert_eq!(ex, cloned);
  }

  #[test]
  fn section_kind_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SectionKind::Type);
    set.insert(SectionKind::Arguments);
    set.insert(SectionKind::Unknown("foo".to_string()));
    set.insert(SectionKind::Unknown("foo".to_string()));
    assert_eq!(set.len(), 3);
  }
}
