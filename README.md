<!--markdownlint-disable MD033 MD059 -->

# nixdoc

[RFC 145]: https://github.com/NixOS/rfcs/blob/master/rfcs/0145-nixdoc-language.md

Parser for Nixdoc documentation comments implementing the [RFC 145] format for
documenting Nix library functions. While the Nixdoc format is not enforced by
any tooling, it is somewhat widely adopted and is treated as a formal
specification. Nixdoc uses `/** … */` doc comments containing Markdown with
structured sections introduced by level-1 headings (`# Section`). This crate, in
turn, parses that format into a typed `DocComment` structure, extracting the
description, type signature, arguments, examples, and any other sections.

## Installation

Nixdoc is officially published on <https://crates.io>. You may install it by
adding it to your Cargo manifest with the latest version:

```toml
[dependencies]
nixdoc = "0.1"
```

## Usage

### Quick Start

```rust
use nixdoc::DocComment;

// Simple one-liner:
let doc = DocComment::parse("/** Returns the identity value. */").unwrap();
assert_eq!(doc.title(), Some("Returns the identity value."));
assert!(doc.sections.is_empty());
```

```rust
use nixdoc::DocComment;

// Multi-section comment:
let doc = DocComment::parse(
    "/**\n  Adds two numbers.\n\n  # Arguments\n\n  - [a] First\n  - [b] Second\n*/"
).unwrap();

assert_eq!(doc.title(), Some("Adds two numbers."));
let args = doc.arguments();
assert_eq!(args.len(), 2);
assert_eq!(args[0].name, "a");
assert_eq!(args[1].name, "b");
```

## Comment format

A Nixdoc comment starts with `/**` and ends with `*/`. Content is indented
(typically by two spaces) and the indentation is automatically stripped.
Sections are introduced by level-1 Markdown headings (`# Section`). The section
body is Markdown text and may contain fenced code blocks.

Recognised section headings (case-insensitive):

- `Type`
- `Arguments` / `Args`
- `Example` / `Examples`
- `Note` / `Notes`
- `Warning` / `Warnings` / `Caution`
- `Deprecated`

## Development

```bash
# Run with Nix flake
nix develop

# Run tests
cargo test
```

## License

This project is made available under Mozilla Public License (MPL) version 2.0.
See [LICENSE](LICENSE) for more details on the exact conditions. An online copy
is provided [here](https://www.mozilla.org/en-US/MPL/2.0/).

<div align="right">
  <a href="#doc-begin">Back to the Top</a>
  <br/>
</div>
