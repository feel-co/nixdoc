//! # nixdoc
//!
//! A spec-based parser for Nixdoc documentation comments (RFC145).
//!
//! Nixdoc uses `/** … */` doc comments containing Markdown with structured
//! sections introduced by level-1 headings (`# Section`). This crate parses
//! that format into a typed [`DocComment`] structure, extracting the
//! description, type signature, arguments, examples, and any other sections.
//!
//! ## Quick start
//!
//! ```rust
//! use nixdoc::DocComment;
//!
//! // Simple one-liner:
//! let doc = DocComment::parse("/** Returns the identity value. */").unwrap();
//! assert_eq!(doc.title(), Some("Returns the identity value."));
//! assert!(doc.sections.is_empty());
//! ```
//!
//! ```rust
//! use nixdoc::DocComment;
//!
//! // Multi-section comment (without inner code fences for this example):
//! let doc = DocComment::parse(
//!     "/**\n  Adds two numbers.\n\n  # Arguments\n\n  - [a] First\n  - [b] Second\n*/"
//! ).unwrap();
//!
//! assert_eq!(doc.title(), Some("Adds two numbers."));
//! let args = doc.arguments();
//! assert_eq!(args.len(), 2);
//! assert_eq!(args[0].name, "a");
//! assert_eq!(args[1].name, "b");
//! ```
//!
//! ## Comment format
//!
//! A Nixdoc comment starts with `/**` and ends with `*/`. Content is indented
//! (typically by two spaces) and the indentation is automatically stripped.
//! Sections are introduced by level-1 Markdown headings (`# Section`). The
//! section body is Markdown text and may contain fenced code blocks.
//!
//! Recognised section headings (case-insensitive):
//! `Type`, `Arguments`/`Args`, `Example`, `Examples`, `Note`, `Notes`,
//! `Warning`/`Warnings`/`Caution`, `Deprecated`.

pub mod error;
pub mod ffi;
pub mod parser;
pub mod section;

pub use error::{ParseError, ParseWarning, WarningKind};
pub use section::{Argument, Example, Section, SectionKind};

/// A fully parsed Nixdoc documentation comment.
///
/// Obtain one via [`DocComment::parse`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocComment {
    /// The normalized comment body with delimiters stripped and indentation removed.
    pub raw_content: String,

    /// Markdown text appearing before the first section heading.
    pub description: String,

    /// Sections in document order.
    pub sections: Vec<Section>,

    /// Non-fatal warnings produced during parsing.
    pub warnings: Vec<ParseWarning>,
}

impl DocComment {
    /// Parse a string as a Nixdoc doc comment.
    ///
    /// The input should be the raw text of a `/** … */` comment, exactly as it
    /// appears in the Nix source. Leading and trailing whitespace on the input
    /// is ignored.
    ///
    /// # Errors
    ///
    /// | Error                           | Cause                                      |
    /// | ------------------------------- | ------------------------------------------ |
    /// | [`ParseError::NotDocComment`]   | Input doesn't start with `/**`             |
    /// | [`ParseError::UnclosedComment`] | Input doesn't end with `*/`                |
    /// | [`ParseError::EmptyComment`]    | Comment has no content after normalization |
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::{DocComment, ParseError};
    ///
    /// assert!(DocComment::parse("/** hello */").is_ok());
    /// assert_eq!(DocComment::parse("/* not doc */"), Err(ParseError::NotDocComment));
    /// assert_eq!(DocComment::parse("/** unclosed"), Err(ParseError::UnclosedComment));
    /// assert_eq!(DocComment::parse("/** */"), Err(ParseError::EmptyComment));
    /// ```
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parser::parse(input)
    }

    /// Returns `true` if the given string looks like a Nixdoc doc comment.
    ///
    /// This is a cheap syntactic check. For full validation, use [`Self::parse`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// assert!(DocComment::is_doc_comment("/** hello */"));
    /// assert!(!DocComment::is_doc_comment("/* not doc */"));
    /// assert!(!DocComment::is_doc_comment("// line comment"));
    /// ```
    pub fn is_doc_comment(input: &str) -> bool {
        let t = input.trim();
        t.starts_with("/**") && t.ends_with("*/")
    }

    /// Returns the title, the first non-empty line of the description.
    ///
    /// The title is the short one-line summary that appears at the top of the
    /// comment, before any further prose or section headings.
    ///
    /// Returns `None` if the description is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// let doc = DocComment::parse("/** The identity function. */").unwrap();
    /// assert_eq!(doc.title(), Some("The identity function."));
    /// ```
    pub fn title(&self) -> Option<&str> {
        let first_line = self.description.trim().lines().next()?;
        let title = first_line.trim();
        if title.is_empty() { None } else { Some(title) }
    }

    /// Returns the full description. Description is the content before the first section heading.
    ///
    /// The description is trimmed of leading and trailing whitespace but
    /// otherwise preserved as Markdown.
    pub fn description(&self) -> &str {
        self.description.trim()
    }

    /// Alias for [`Self::description`], matching the proposed API in the spec.
    pub fn main_content(&self) -> &str {
        self.description()
    }

    /// Returns the first section with the given heading, case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// let doc = DocComment::parse("/**\n  Desc.\n\n  # Type\n\n  ```\n  a\n  ```\n*/").unwrap();
    /// assert!(doc.section("type").is_some());
    /// assert!(doc.section("TYPE").is_some());
    /// assert!(doc.section("missing").is_none());
    /// ```
    pub fn section(&self, name: &str) -> Option<&Section> {
        let name_lower = name.to_lowercase();
        self.sections
            .iter()
            .find(|s| s.heading.to_lowercase() == name_lower)
    }

    /// Returns the type signature, if one can be found.
    ///
    /// Two formats are recognised, in order of priority:
    ///
    /// 1. **Modern format** (RFC145): the first fenced code block inside a
    ///    `# Type` section.
    /// 2. **Legacy format**: an `identifier :: type` annotation embedded
    ///    directly in the description text, without a `# Type` section.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// let input = "/**\n  f.\n\n  # Type\n\n  ```\n  f :: Int -> Int\n  ```\n*/";
    /// let doc = DocComment::parse(input).unwrap();
    /// assert_eq!(doc.type_sig(), Some("f :: Int -> Int\n".to_string()));
    /// ```
    pub fn type_sig(&self) -> Option<String> {
        // Modern format: first fenced code block inside a `# Type` section.
        if let Some(section) = self.section("Type") {
            return parser::extract_first_code_block(&section.content);
        }
        // Legacy format: inline `identifier :: type` in the description.
        parser::extract_inline_type_sig(&self.description)
    }

    /// Returns the parsed arguments from the `# Arguments` (or `# Args`) section.
    ///
    /// Each `- [name] description` line in the section becomes an [`Argument`].
    /// Returns an empty vector if there is no arguments section.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// let input = "/**\n  f.\n\n  # Arguments\n\n  - [a] First\n  - [b] Second\n*/";
    /// let doc = DocComment::parse(input).unwrap();
    /// let args = doc.arguments();
    /// assert_eq!(args.len(), 2);
    /// assert_eq!(args[0].name, "a");
    /// assert_eq!(args[0].description, "First");
    /// ```
    pub fn arguments(&self) -> Vec<Argument> {
        match self.section("Arguments").or_else(|| self.section("Args")) {
            Some(s) => parser::parse_arguments(&s.content),
            None => Vec::new(),
        }
    }

    /// Returns all code examples from `# Example` and `# Examples` sections.
    ///
    /// Multiple examples within a single section (multiple code blocks) are
    /// returned as separate [`Example`] values.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// let input = "/**\n  f.\n\n  # Example\n\n  ```nix\n  f 1\n  => 1\n  ```\n*/";
    /// let doc = DocComment::parse(input).unwrap();
    /// let examples = doc.examples();
    /// assert_eq!(examples.len(), 1);
    /// assert_eq!(examples[0].language, Some("nix".to_string()));
    /// ```
    pub fn examples(&self) -> Vec<Example> {
        self.sections
            .iter()
            .filter(|s| {
                let h = s.heading.to_lowercase();
                h == "example" || h == "examples"
            })
            .flat_map(|s| parser::parse_examples(&s.content))
            .collect()
    }

    /// Returns the trimmed content of all `# Note` and `# Notes` sections.
    pub fn notes(&self) -> Vec<&str> {
        self.sections
            .iter()
            .filter(|s| {
                let h = s.heading.to_lowercase();
                h == "note" || h == "notes"
            })
            .map(|s| s.content.trim())
            .collect()
    }

    /// Returns the trimmed content of all warning sections
    /// (`# Warning`, `# Warnings`, `# Caution`).
    pub fn warnings_content(&self) -> Vec<&str> {
        self.sections
            .iter()
            .filter(|s| {
                let h = s.heading.to_lowercase();
                h == "warning" || h == "warnings" || h == "caution"
            })
            .map(|s| s.content.trim())
            .collect()
    }

    /// Returns `true` if a `# Deprecated` section is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixdoc::DocComment;
    ///
    /// let input = "/**\n  Old fn.\n\n  # Deprecated\n\n  Use `newFn` instead.\n*/";
    /// let doc = DocComment::parse(input).unwrap();
    /// assert!(doc.is_deprecated());
    /// ```
    pub fn is_deprecated(&self) -> bool {
        self.section("Deprecated").is_some()
    }

    /// Returns the trimmed content of the `# Deprecated` section, if present.
    pub fn deprecation_notice(&self) -> Option<&str> {
        self.section("Deprecated").map(|s| s.content.trim())
    }
}
