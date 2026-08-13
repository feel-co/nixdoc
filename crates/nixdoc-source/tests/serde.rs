#![cfg(feature = "serde")]

use nixdoc_source::{SOURCE_SCHEMA_VERSION, SourceDocument};

#[test]
fn source_document_schema_roundtrip() {
  let document = nixdoc_source::extract("/** docs */ value = 1;");
  let serialized = serde_json::to_string(&document).unwrap();
  let back: SourceDocument = serde_json::from_str(&serialized).unwrap();
  assert_eq!(back, document);
  assert_eq!(back.schema_version, SOURCE_SCHEMA_VERSION);
}
