use crate::DocComment;
use crate::error::{ParseError, ParseWarning, WarningKind};
use crate::section::{Argument, Example, Section};

/// Parse a raw input string as a Nixdoc doc comment.
///
/// This is the entry point called by [`DocComment::parse`].
pub(crate) fn parse(input: &str) -> Result<DocComment, ParseError> {
    let trimmed = input.trim();

    // Strip delimiters, propagating appropriate errors.
    let inner = trimmed
        .strip_prefix("/**")
        .ok_or(ParseError::NotDocComment)?
        .strip_suffix("*/")
        .ok_or(ParseError::UnclosedComment)?;

    // Normalize indentation and trim surrounding blank lines.
    let content = normalize(inner);

    if content.trim().is_empty() {
        return Err(ParseError::EmptyComment);
    }

    let mut warnings = Vec::new();
    let (description, sections) = parse_sections(&content, &mut warnings);

    // Warn about any unrecognized section headings.
    for section in &sections {
        if !section.kind().is_known() {
            warnings.push(ParseWarning {
                kind: WarningKind::UnknownSection,
                message: format!("unrecognized section heading: '{}'", section.heading),
            });
        }
    }

    Ok(DocComment {
        raw_content: content,
        description,
        sections,
        warnings,
    })
}

/// Normalize the raw inner content of a doc comment by stripping consistent
/// leading whitespace and trimming surrounding blank lines.
///
/// This mirrors how Nix handles multiline strings (`''`): the common leading
/// whitespace is detected across all non-empty lines and removed from each line.
///
/// # Examples
///
/// ```
/// use nixdoc::parser::normalize;
///
/// // Two spaces of common indent are stripped:
/// assert_eq!(normalize("  hello\n  world"), "hello\nworld");
///
/// // Blank lines don't affect indent detection:
/// assert_eq!(normalize("\n  hello\n\n  world\n"), "hello\n\nworld");
///
/// // Mixed indent: minimum is preserved as relative offset:
/// assert_eq!(normalize("  a\n    b"), "a\n  b");
/// ```
pub fn normalize(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Minimum number of leading whitespace CHARACTERS across all non-empty lines.
    // Using character counts (not byte lengths) is safe for multi-byte Unicode
    // whitespace such as U+00A0 (non-breaking space) or U+3000 (ideographic space).
    let min_indent: usize = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Strip `min_indent` leading characters from each line.
    // Only-whitespace lines become empty strings.
    let dedented: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(min_indent).collect()
            }
        })
        .collect();

    // Trim leading/trailing whitespace (blank lines, stray spaces).
    let joined = dedented.join("\n");
    joined.trim().to_string()
}

/// If `trimmed` (a line with leading whitespace already stripped) starts an
/// opening code fence, return `(fence_char, fence_len, language)`.
///
/// Per CommonMark, a fence is 3+ identical backticks or tildes. The opening
/// line may be followed by an optional language info string.
fn parse_fence_open(trimmed: &str) -> Option<(char, usize, Option<String>)> {
    let fence_char = if trimmed.starts_with("```") {
        '`'
    } else if trimmed.starts_with("~~~") {
        '~'
    } else {
        return None;
    };

    let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();

    // Everything after the fence chars is the info string (language).
    // CommonMark: backtick info strings may not contain backticks.
    let after = trimmed[fence_len..].trim();
    let language = if after.is_empty() {
        None
    } else {
        // Take only the first whitespace-delimited token as the language.
        let lang = after.split_whitespace().next().unwrap_or("");
        if lang.is_empty() {
            None
        } else {
            Some(lang.to_string())
        }
    };

    Some((fence_char, fence_len, language))
}

/// Returns `true` if `trimmed` is a valid closing fence for a code block that
/// was opened with `fence_len` repetitions of `fence_char`.
///
/// Per CommonMark: the closing fence must consist of at least `fence_len`
/// occurrences of `fence_char`, optionally followed by spaces, with nothing
/// else on the line.
fn is_closing_fence(trimmed: &str, fence_char: char, fence_len: usize) -> bool {
    // All-ASCII fence characters, so char count == byte count here.
    let count = trimmed.chars().take_while(|&c| c == fence_char).count();
    if count < fence_len {
        return false;
    }
    // Anything after the fence characters must be spaces only.
    trimmed[count..].chars().all(|c| c == ' ')
}

/// Parse the normalized content into a (description, sections) pair.
///
/// A level-1 Markdown heading (`# Heading`) at the start of a line begins a
/// new section, except when inside a fenced code block where `# comment`
/// lines are not headings.
///
/// Everything before the first heading is the description.
fn parse_sections(content: &str, warnings: &mut Vec<ParseWarning>) -> (String, Vec<Section>) {
    let mut sections: Vec<Section> = Vec::new();

    // Lines accumulated before the first section heading.
    let mut description_lines: Vec<&str> = Vec::new();
    // True while we haven't yet seen any section heading.
    let mut in_description = true;

    // Current section being accumulated.
    let mut current_heading: Option<String> = None;
    let mut section_lines: Vec<&str> = Vec::new();

    // Fenced-code-block tracking to avoid treating `# comment` inside code
    // as section headings.  We store the fence character and its length so
    // that 4-backtick (or longer) fences close correctly per CommonMark.
    let mut in_code_block = false;
    let mut fence_char: char = '`';
    let mut fence_len: usize = 3;

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Update code-block state before deciding if the line is a heading.
        if !in_code_block {
            if let Some((fc, fl, _)) = parse_fence_open(trimmed) {
                in_code_block = true;
                fence_char = fc;
                fence_len = fl;
            }
        } else if is_closing_fence(trimmed, fence_char, fence_len) {
            in_code_block = false;
        }

        // Lines inside a code block are never section headings.
        let is_heading_candidate = !in_code_block && line.starts_with("# ");

        if is_heading_candidate {
            let heading = line["# ".len()..].trim().to_string();

            if !heading.is_empty() {
                // Finalize what we were accumulating.
                if in_description {
                    in_description = false;
                } else if let Some(h) = current_heading.take() {
                    flush_section(&h, &section_lines, &mut sections, warnings);
                    section_lines.clear();
                }
                current_heading = Some(heading);
                continue;
            }
        }

        // Accumulate the line into the active buffer.
        if in_description {
            description_lines.push(line);
        } else {
            section_lines.push(line);
        }
    }

    // Flush the last section or absorb remaining lines into the description.
    if let Some(h) = current_heading {
        flush_section(&h, &section_lines, &mut sections, warnings);
    } else {
        // No headings were ever seen; everything is the description.
        description_lines.extend_from_slice(&section_lines);
    }

    let description = description_lines.join("\n").trim().to_string();
    (description, sections)
}

fn flush_section(
    heading: &str,
    lines: &[&str],
    sections: &mut Vec<Section>,
    warnings: &mut Vec<ParseWarning>,
) {
    let content = lines.join("\n").trim().to_string();
    if content.is_empty() {
        warnings.push(ParseWarning {
            kind: WarningKind::EmptySection,
            message: format!("section '{}' has no content", heading),
        });
    }
    sections.push(Section {
        heading: heading.to_string(),
        content,
    });
}

/// Parse argument entries from the body of a `# Arguments` section.
///
/// Each argument is expected on a line in the form:
///
/// ```text
/// - [name] Description text
/// ```
///
/// The description may continue on subsequent indented lines:
///
/// ```text
/// - [name] First line of description.
///   Continuation of the description.
/// ```
///
/// Continuation lines must be indented (start with whitespace). Non-indented
/// lines that are not argument entries are treated as prose and ignored.
pub(crate) fn parse_arguments(content: &str) -> Vec<Argument> {
    let mut arguments: Vec<Argument> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_desc = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("- [") {
            // Flush the previous argument before starting a new one.
            if let Some(name) = current_name.take() {
                arguments.push(Argument {
                    name,
                    description: current_desc.trim().to_string(),
                });
                current_desc.clear();
            }

            if let Some(bracket_end) = rest.find(']') {
                let name = rest[..bracket_end].trim().to_string();
                if !name.is_empty() {
                    current_name = Some(name);
                    current_desc = rest[bracket_end + 1..].trim().to_string();
                }
            }
        } else if current_name.is_some()
            && !trimmed.is_empty()
            && line.starts_with(|c: char| c.is_whitespace())
        {
            // Indented continuation line: append to the current description.
            if !current_desc.is_empty() {
                current_desc.push(' ');
            }
            current_desc.push_str(trimmed);
        }
        // Non-indented non-argument lines (prose, blank lines) are ignored.
    }

    // Flush the last argument.
    if let Some(name) = current_name {
        arguments.push(Argument {
            name,
            description: current_desc.trim().to_string(),
        });
    }

    arguments
}

/// Parse code examples from the body of an `# Example` or `# Examples` section.
///
/// Each example is a fenced code block delimited by ` ``` ` or `~~~`. Multiple
/// examples may appear in a single section, separated by prose or other content.
/// Fences of 4 or more backticks/tildes are handled correctly.
pub(crate) fn parse_examples(content: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    let mut in_block = false;
    let mut current_language: Option<String> = None;
    let mut current_code = String::new();
    let mut block_fence_char: char = '`';
    let mut block_fence_len: usize = 3;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if !in_block {
            if let Some((fc, fl, lang)) = parse_fence_open(trimmed) {
                in_block = true;
                block_fence_char = fc;
                block_fence_len = fl;
                current_language = lang;
                current_code.clear();
            }
        } else if is_closing_fence(trimmed, block_fence_char, block_fence_len) {
            // Closing fence: save the accumulated example.
            examples.push(Example {
                language: current_language.take(),
                code: current_code.clone(),
            });
            current_code.clear();
            in_block = false;
        } else {
            current_code.push_str(line);
            current_code.push('\n');
        }
    }

    // Unclosed code block: return what we have rather than silently failing.
    if in_block && !current_code.is_empty() {
        examples.push(Example {
            language: current_language.take(),
            code: current_code,
        });
    }

    examples
}

/// Extract the content of the first fenced code block in a string.
///
/// Used by [`DocComment::type_sig`] to pull the type signature out of a
/// `# Type` section. Returns `None` if no code block is found.
/// Fences of 4 or more backticks/tildes are handled correctly.
pub(crate) fn extract_first_code_block(content: &str) -> Option<String> {
    let mut in_block = false;
    let mut result = String::new();
    let mut block_fence_char: char = '`';
    let mut block_fence_len: usize = 3;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if !in_block {
            if let Some((fc, fl, _)) = parse_fence_open(trimmed) {
                in_block = true;
                block_fence_char = fc;
                block_fence_len = fl;
            }
        } else if is_closing_fence(trimmed, block_fence_char, block_fence_len) {
            return Some(result);
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Unclosed code block: return what we have rather than silently failing.
    if in_block && !result.is_empty() {
        Some(result)
    } else {
        None
    }
}

/// Extract a legacy inline type annotation from a description string.
///
/// Some older Nixdoc comments embed the type signature directly in the
/// description body, without using a `# Type` section:
///
/// ```text
/// /**
///   Merge two attribute sets shallowly, right side trumps left
///   mergeAttrs :: attrs -> attrs -> attrs
/// */
/// ```
///
/// This function scans lines for the `identifier :: type` pattern and returns
/// the first matching line. Returns `None` if no such line is found.
///
/// This is a fallback for [`DocComment::type_sig`]; the modern `# Type` section
/// always takes precedence.
pub(crate) fn extract_inline_type_sig(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(sig) = parse_inline_type_line(line.trim()) {
            return Some(sig.to_string());
        }
    }
    None
}

/// If `line` looks like a legacy `identifier :: type` annotation, return it.
///
/// The identifier before `::` must be a valid Nix name (letters, digits,
/// underscores, hyphens, primes). An empty or multi-word prefix is rejected
/// to avoid false positives in prose descriptions.
fn parse_inline_type_line(line: &str) -> Option<&str> {
    // Must contain `::`
    let sep = line.find("::")?;
    let before = line[..sep].trim();
    let after = line[sep + 2..].trim();

    // Both sides must be non-empty.
    if before.is_empty() || after.is_empty() {
        return None;
    }

    // The name must look like a Nix identifier: alphanumeric, '_', '-', '\''.
    // Rejecting anything with spaces prevents matching prose like
    // "a function takes foo :: bar arguments".
    let is_valid_ident = !before.is_empty()
        && before
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '\'');

    if is_valid_ident { Some(line) } else { None }
}

#[cfg(test)]
#[path = "tests/parser.rs"]
mod tests;
