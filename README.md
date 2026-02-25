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
nixdoc = "0.2"
```

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Link against the standard library. When disabled the crate is `no_std` (requires `alloc`). |
| `serde` | no      | Derive `serde::Serialize`/`Deserialize` on all public types. |

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

### Type Signatures

```rust
use nixdoc::DocComment;

let input = "/**\n  f.\n\n  # Type\n\n  ```\n  f :: Int -> Int\n  ```\n*/";
let doc = DocComment::parse(input).unwrap();
assert_eq!(doc.type_sig(), Some("f :: Int -> Int\n".to_string()));
```

### Examples

```rust
use nixdoc::DocComment;

let input = "/**\n  f.\n\n  # Example\n\n  ```nix\n  f 1\n  => 1\n  ```\n*/";
let doc = DocComment::parse(input).unwrap();
let examples = doc.examples();
assert_eq!(examples.len(), 1);
assert_eq!(examples[0].language, Some("nix".to_string()));
```

### Notes, Warnings, and Deprecation

```rust
use nixdoc::DocComment;

let input = "/**\n  Old fn.\n\n  # Note\n\n  This is informational.\n\n  # Warning\n\n  Use with caution.\n\n  # Deprecated\n\n  Use `newFn` instead.\n*/";
let doc = DocComment::parse(input).unwrap();

assert_eq!(doc.notes(), vec!["This is informational."]);
assert_eq!(doc.warnings_content(), vec!["Use with caution."]);
assert!(doc.is_deprecated());
assert_eq!(doc.deprecation_notice(), Some("Use `newFn` instead."));
```

### no_std Support

This crate supports `no_std`. Disable default features to use it in embedded environments:

```toml
[dependencies]
nixdoc = { version = "0.2", default-features = false }
```

When using `no_std`, you must enable the `alloc` crate and link `libcore` or `liballoc`.

### C FFI Bindings

For C/C++ integration, use the companion `nixdoc-ffi` crate:

```toml
[dependencies]
nixdoc-ffi = "0.2"
```

```c
#include <nixdoc.h>

int main() {
    NixdocDocComment *doc = NULL;
    int result = nixdoc_parse_into("/** Hello */", &doc);

    if (result == 0) {
        char *title = nixdoc_title(doc);
        printf("Title: %s\n", title);
        nixdoc_free_string(title);
        nixdoc_free(doc);
    }

    return result;
}
```

Build the shared library:

```bash
cargo build -p nixdoc-ffi
# Produces libnixdoc.so (shared) and libnixdoc.a (static)
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

### Attributions

[noogle]: https://github.com/nix-community/noogle?

This project is greatly inspired by [noogle]'s Pesto module. While pesto is a
CLI and prefers to consume the `rnix` parser, `nixdoc` is completely standalone
and is designed as a library with the integration and extraction left to the
user.
