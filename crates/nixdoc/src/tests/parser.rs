use super::*;

#[test]
fn normalize_strips_common_indent() {
    assert_eq!(normalize("  hello\n  world"), "hello\nworld");
}

#[test]
fn normalize_ignores_empty_lines_for_indent() {
    assert_eq!(normalize("\n  hello\n\n  world\n"), "hello\n\nworld");
}

#[test]
fn normalize_preserves_relative_indent() {
    assert_eq!(normalize("  a\n    b"), "a\n  b");
}

#[test]
fn normalize_single_line() {
    // Surrounding spaces are trimmed as part of normalizing the raw content.
    assert_eq!(normalize(" hello "), "hello");
    assert_eq!(normalize("  trimmed  "), "trimmed");
}

#[test]
fn normalize_no_indent() {
    assert_eq!(normalize("hello\nworld"), "hello\nworld");
}

#[test]
fn normalize_only_empty_lines() {
    assert_eq!(normalize("\n\n\n"), "");
}

#[test]
fn normalize_zero_indent() {
    // Lines with zero leading spaces: min_indent == 0, nothing stripped.
    assert_eq!(normalize("foo\nbar"), "foo\nbar");
}

#[test]
fn fence_open_three_backticks() {
    let (fc, fl, lang) = parse_fence_open("```").unwrap();
    assert_eq!(fc, '`');
    assert_eq!(fl, 3);
    assert_eq!(lang, None);
}

#[test]
fn fence_open_with_language() {
    let (fc, fl, lang) = parse_fence_open("```nix").unwrap();
    assert_eq!(fc, '`');
    assert_eq!(fl, 3);
    assert_eq!(lang, Some("nix".to_string()));
}

#[test]
fn fence_open_four_backticks() {
    let (fc, fl, lang) = parse_fence_open("````").unwrap();
    assert_eq!(fc, '`');
    assert_eq!(fl, 4);
    assert_eq!(lang, None);
}

#[test]
fn fence_open_tildes() {
    let (fc, fl, lang) = parse_fence_open("~~~nix").unwrap();
    assert_eq!(fc, '~');
    assert_eq!(fl, 3);
    assert_eq!(lang, Some("nix".to_string()));
}

#[test]
fn fence_open_not_a_fence() {
    assert!(parse_fence_open("  code").is_none());
    assert!(parse_fence_open("# heading").is_none());
    assert!(parse_fence_open("``").is_none()); // only 2 backticks
}

#[test]
fn fence_open_two_backticks_not_a_fence() {
    // Exactly 2 backticks does NOT start a fence per CommonMark.
    assert!(parse_fence_open("``nix").is_none());
}

#[test]
fn closing_fence_exact_match() {
    assert!(is_closing_fence("```", '`', 3));
}

#[test]
fn closing_fence_longer_is_valid() {
    // 4+ backtick closing is valid for a 3-backtick opening per CommonMark.
    assert!(is_closing_fence("````", '`', 3));
    assert!(is_closing_fence("`````", '`', 3));
}

#[test]
fn closing_fence_trailing_spaces_allowed() {
    assert!(is_closing_fence("```  ", '`', 3));
    assert!(is_closing_fence("```   ", '`', 3));
}

#[test]
fn closing_fence_info_string_rejected() {
    // A closing fence cannot have an info string per CommonMark.
    assert!(!is_closing_fence("```nix", '`', 3));
}

#[test]
fn closing_fence_too_short_rejected() {
    assert!(!is_closing_fence("``", '`', 3));
}

#[test]
fn closing_fence_wrong_char_rejected() {
    assert!(!is_closing_fence("~~~", '`', 3));
    assert!(!is_closing_fence("```", '~', 3));
}

#[test]
fn closing_fence_four_backtick_block() {
    assert!(is_closing_fence("````", '`', 4));
    assert!(!is_closing_fence("```", '`', 4)); // too short
}

#[test]
fn parse_arguments_basic() {
    let content = "- [a] First\n- [b] Second";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name, "a");
    assert_eq!(args[0].description, "First");
    assert_eq!(args[1].name, "b");
    assert_eq!(args[1].description, "Second");
}

#[test]
fn parse_arguments_empty_description() {
    let content = "- [x]\n- [y] ";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].description, "");
    assert_eq!(args[1].description, "");
}

#[test]
fn parse_arguments_ignores_non_arg_lines() {
    let content = "Some intro text.\n- [a] Arg\nMore text.";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].name, "a");
}

#[test]
fn parse_arguments_skips_empty_name() {
    // `- []` has an empty name and should be skipped.
    let content = "- [] Not an arg\n- [x] Valid";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].name, "x");
}

#[test]
fn parse_arguments_multiline_continuation() {
    let content = "- [root] First line.\n  Continuation here.\n- [fileset] Single line.";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].name, "root");
    assert_eq!(args[0].description, "First line. Continuation here.");
    assert_eq!(args[1].name, "fileset");
    assert_eq!(args[1].description, "Single line.");
}

#[test]
fn parse_arguments_multiline_no_inline_desc() {
    // Description starts entirely on the continuation line.
    let content = "- [x]\n  Description on next line.";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].description, "Description on next line.");
}

#[test]
fn parse_arguments_non_indented_prose_ignored() {
    // A non-indented non-argument line after an argument is NOT a continuation.
    let content = "- [a] Arg\nThis is prose, not a continuation.";
    let args = parse_arguments(content);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].description, "Arg");
}

#[test]
fn parse_examples_single_no_lang() {
    let content = "```\nfoo 1\n```";
    let examples = parse_examples(content);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].language, None);
    assert_eq!(examples[0].code, "foo 1\n");
}

#[test]
fn parse_examples_with_language() {
    let content = "```nix\nfoo 1\n```";
    let examples = parse_examples(content);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].language, Some("nix".to_string()));
    assert_eq!(examples[0].code, "foo 1\n");
}

#[test]
fn parse_examples_multiple() {
    let content = "```nix\nfoo 1\n```\n\nSome prose.\n\n```\nbar 2\n```";
    let examples = parse_examples(content);
    assert_eq!(examples.len(), 2);
    assert_eq!(examples[0].language, Some("nix".to_string()));
    assert_eq!(examples[1].language, None);
}

#[test]
fn parse_examples_tilde_fence() {
    let content = "~~~nix\nfoo\n~~~";
    let examples = parse_examples(content);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].language, Some("nix".to_string()));
}

#[test]
fn parse_examples_four_backtick_fence() {
    // A 4-backtick fence must close with 4+ backticks.
    let content = "````nix\nsome code\n````";
    let examples = parse_examples(content);
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].language, Some("nix".to_string()));
    assert_eq!(examples[0].code, "some code\n");
}

#[test]
fn parse_examples_four_backtick_with_inner_three() {
    // A 4-backtick fence containing a 3-backtick sequence stays open.
    let content = "````nix\n```\nnested\n```\n````";
    let examples = parse_examples(content);
    assert_eq!(examples.len(), 1);
    // The inner ``` lines are part of the code, not fence markers.
    assert!(examples[0].code.contains("```"));
}

#[test]
fn extract_code_block_basic() {
    let content = "```\nfoo :: Int -> Int\n```";
    assert_eq!(
        extract_first_code_block(content),
        Some("foo :: Int -> Int\n".to_string())
    );
}

#[test]
fn extract_code_block_skips_lang_specifier() {
    let content = "```nix\nfoo 1\n```";
    assert_eq!(
        extract_first_code_block(content),
        Some("foo 1\n".to_string())
    );
}

#[test]
fn extract_code_block_none_when_absent() {
    assert_eq!(extract_first_code_block("just text"), None);
}

#[test]
fn extract_code_block_unclosed_returns_content() {
    // An unclosed block returns accumulated content. Each line has a '\n'
    // appended (consistent with closed blocks), so the result ends in '\n'.
    let content = "```\nfoo :: Int";
    assert_eq!(
        extract_first_code_block(content),
        Some("foo :: Int\n".to_string())
    );
}

#[test]
fn extract_code_block_four_backtick_fence() {
    let content = "````\nfoo :: Int -> Int\n````";
    assert_eq!(
        extract_first_code_block(content),
        Some("foo :: Int -> Int\n".to_string())
    );
}

#[test]
fn extract_code_block_closing_fence_trailing_spaces() {
    // CommonMark allows trailing spaces on a closing fence line.
    let content = "```\nfoo\n```  ";
    assert_eq!(extract_first_code_block(content), Some("foo\n".to_string()));
}

#[test]
fn parse_sections_does_not_treat_code_hash_as_heading() {
    let content = "Desc.\n\n# Example\n\n```nix\n# This is a Nix comment\nfoo\n```";
    let mut warnings = Vec::new();
    let (desc, sections) = parse_sections(content, &mut warnings);

    assert_eq!(desc, "Desc.");
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].heading, "Example");
    // The `# This is a Nix comment` must be inside the section content,
    // NOT extracted as a new section.
    assert!(sections[0].content.contains("# This is a Nix comment"));
}

#[test]
fn parse_sections_four_backtick_fence() {
    // A 4-backtick fence containing a `# comment` and 3-backtick inner
    // sequences must not produce spurious sections.
    let content = "Desc.\n\n# Example\n\n````nix\n# not a heading\n```\ninner\n```\n````";
    let mut warnings = Vec::new();
    let (_, sections) = parse_sections(content, &mut warnings);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].heading, "Example");
    assert!(sections[0].content.contains("# not a heading"));
    assert!(sections[0].content.contains("```"));
}

#[test]
fn parse_sections_closing_fence_with_trailing_spaces() {
    let content = "Desc.\n\n# Type\n\n```\nfoo :: Int\n```  \n\n# Arguments\n\n- [x] x";
    let mut warnings = Vec::new();
    let (_, sections) = parse_sections(content, &mut warnings);

    // Both sections must be parsed; the trailing-spaces closing fence
    // must not leave the parser stuck inside a code block.
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].heading, "Type");
    assert_eq!(sections[1].heading, "Arguments");
}

#[test]
fn inline_type_sig_simple() {
    assert_eq!(
        extract_inline_type_sig("mergeAttrs :: attrs -> attrs -> attrs"),
        Some("mergeAttrs :: attrs -> attrs -> attrs".to_string())
    );
}

#[test]
fn inline_type_sig_in_multiline_description() {
    let content = "Merge two attribute sets.\nmergeAttrs :: attrs -> attrs -> attrs";
    assert_eq!(
        extract_inline_type_sig(content),
        Some("mergeAttrs :: attrs -> attrs -> attrs".to_string())
    );
}

#[test]
fn inline_type_sig_first_match_wins() {
    let content = "foo :: a -> a\nbar :: b -> b";
    assert_eq!(
        extract_inline_type_sig(content),
        Some("foo :: a -> a".to_string())
    );
}

#[test]
fn inline_type_sig_rejected_for_prose() {
    // A sentence that happens to contain `::` but has spaces before it.
    assert_eq!(
        extract_inline_type_sig("A value of type foo :: bar is returned."),
        None
    );
}

#[test]
fn inline_type_sig_rejected_empty_before() {
    assert_eq!(extract_inline_type_sig(":: attrs -> attrs"), None);
}

#[test]
fn inline_type_sig_rejected_empty_after() {
    assert_eq!(extract_inline_type_sig("mergeAttrs ::"), None);
}

#[test]
fn inline_type_sig_none_when_absent() {
    assert_eq!(extract_inline_type_sig("Just a plain description."), None);
}

#[test]
fn inline_type_sig_primes_in_name() {
    // Nix identifiers may contain primes: `f'`.
    assert_eq!(
        extract_inline_type_sig("f' :: a -> a"),
        Some("f' :: a -> a".to_string())
    );
}
