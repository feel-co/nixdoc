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

/// The semantic kind of a Nixdoc section, derived from its heading.
///
/// The Nixdoc specification (RFC145) defines a set of well-known section
/// names. Any heading not in this set produces `SectionKind::Unknown`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// Any other section heading not covered above.
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
    /// assert_eq!(SectionKind::from_heading("ARGUMENTS"), SectionKind::Arguments);
    /// assert_eq!(
    ///     SectionKind::from_heading("See Also"),
    ///     SectionKind::Unknown("see also".to_string()),
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
pub struct Argument {
    /// The argument name, as written inside `[...]`.
    pub name: String,
    /// The argument description text (may be empty).
    pub description: String,
}

/// A code example extracted from an `# Example` or `# Examples` section.
///
/// Each example corresponds to a single fenced code block (` ``` ` or `~~~`).
#[derive(Debug, Clone, PartialEq)]
pub struct Example {
    /// The language specifier from the fenced code block, if present (e.g., `"nix"`).
    pub language: Option<String>,
    /// The raw code content.
    pub code: String,
}
