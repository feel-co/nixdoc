{
  mkShell,
  rustc,
  cargo,
  rustfmt,
  clippy,
  taplo,
  rust-analyzer-unwrapped,
  cargo-nextest,
  rustPlatform,
}:
mkShell {
  name = "nixdoc";

  strictDeps = true;
  packages = [
    rustc
    cargo

    # Tools
    (rustfmt.override {asNightly = true;})
    clippy
    cargo
    taplo
    rust-analyzer-unwrapped

    # Additional Cargo tooling
    cargo-nextest
  ];

  RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
