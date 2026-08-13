use std::ffi::{CStr, CString};

use super::{
  NixdocDocComment, nixdoc_arguments, nixdoc_deprecation_notice,
  nixdoc_description, nixdoc_examples, nixdoc_extract_json, nixdoc_free,
  nixdoc_free_string, nixdoc_free_string_array, nixdoc_is_deprecated,
  nixdoc_is_doc_comment, nixdoc_notes, nixdoc_parse, nixdoc_parse_into,
  nixdoc_title, nixdoc_type_sig, nixdoc_version, nixdoc_warnings,
};

const NIXDOC_SUCCESS: i32 = 0;
const NIXDOC_ERROR_PARSE: i32 = 1;
const NIXDOC_ERROR_NULL: i32 = 2;

fn to_cstring(s: &str) -> CString {
  CString::new(s).unwrap()
}

fn from_cstring(ptr: *mut std::os::raw::c_char) -> String {
  if ptr.is_null() {
    return String::new();
  }
  unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn version_is_static_utf8() {
  let version = unsafe { CStr::from_ptr(nixdoc_version()) };
  assert_eq!(version.to_str().unwrap(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn extract_json_exposes_versioned_source_contract() {
  let input = to_cstring("/** docs */ value = 1;");
  let output = unsafe { nixdoc_extract_json(input.as_ptr()) };
  let json = from_cstring(output);
  assert!(json.contains("\"schema_version\":1"));
  assert!(json.contains("\"name\":\"value\""));
  unsafe { nixdoc_free_string(output) };
}

#[test]
fn parse_null_input() {
  let result = unsafe { nixdoc_parse(std::ptr::null()) };
  assert_eq!(result, NIXDOC_ERROR_NULL);
}

#[test]
fn parse_into_null_input() {
  let result =
    unsafe { nixdoc_parse_into(std::ptr::null(), std::ptr::null_mut()) };
  assert_eq!(result, NIXDOC_ERROR_NULL);
}

#[test]
fn parse_into_null_out_ptr() {
  let input = to_cstring("/** Hello */");
  let result =
    unsafe { nixdoc_parse_into(input.as_ptr(), std::ptr::null_mut()) };
  assert_eq!(result, NIXDOC_ERROR_NULL);
}

#[test]
fn parse_valid_comment() {
  let input = to_cstring("/** A simple comment. */");
  let result = unsafe { nixdoc_parse(input.as_ptr()) };
  assert_eq!(result, NIXDOC_SUCCESS);
}

#[test]
fn parse_invalid_comment() {
  let input = to_cstring("/* not a doc comment */");
  let result = unsafe { nixdoc_parse(input.as_ptr()) };
  assert_eq!(result, NIXDOC_ERROR_PARSE);
}

#[test]
fn parse_into_valid_comment() {
  let input = to_cstring("/** A simple comment. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  let result = unsafe { nixdoc_parse_into(input.as_ptr(), &mut doc) };
  assert_eq!(result, NIXDOC_SUCCESS);
  assert!(!doc.is_null());
  unsafe { nixdoc_free(doc) };
}

#[test]
fn parse_into_invalid_comment() {
  let input = to_cstring("/* not a doc comment */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  let result = unsafe { nixdoc_parse_into(input.as_ptr(), &mut doc) };
  assert_eq!(result, NIXDOC_ERROR_PARSE);
  assert!(doc.is_null());
}

#[test]
fn free_null_ptr() {
  unsafe { nixdoc_free(std::ptr::null_mut()) };
}

#[test]
fn is_doc_comment_null_input() {
  let result = unsafe { nixdoc_is_doc_comment(std::ptr::null()) };
  assert!(!result);
}

#[test]
fn is_doc_comment_valid() {
  let input = to_cstring("/** Hello */");
  let result = unsafe { nixdoc_is_doc_comment(input.as_ptr()) };
  assert!(result);
}

#[test]
fn is_doc_comment_invalid() {
  let input = to_cstring("/* not doc */");
  let result = unsafe { nixdoc_is_doc_comment(input.as_ptr()) };
  assert!(!result);
}

#[test]
fn title_null_doc() {
  let result = unsafe { nixdoc_title(std::ptr::null()) };
  assert_eq!(from_cstring(result), "");
  unsafe { nixdoc_free_string(result) };
}

#[test]
fn title_from_parsed_doc() {
  let input = to_cstring("/** Returns the identity value. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let title = nixdoc_title(doc);
    assert_eq!(from_cstring(title), "Returns the identity value.");
    nixdoc_free_string(title);
    nixdoc_free(doc);
  }
}

#[test]
fn description_null_doc() {
  let result = unsafe { nixdoc_description(std::ptr::null()) };
  assert_eq!(from_cstring(result), "");
  unsafe { nixdoc_free_string(result) };
}

#[test]
fn description_from_parsed_doc() {
  let input = to_cstring("/** A description. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let desc = nixdoc_description(doc);
    assert_eq!(from_cstring(desc), "A description.");
    nixdoc_free_string(desc);
    nixdoc_free(doc);
  }
}

#[test]
fn type_sig_null_doc() {
  let result = unsafe { nixdoc_type_sig(std::ptr::null()) };
  assert_eq!(from_cstring(result), "");
  unsafe { nixdoc_free_string(result) };
}

#[test]
fn type_sig_from_parsed_doc() {
  let input = to_cstring("/** f.\n\n# Type\n\n```\nf :: Int -> Int\n```\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let sig = nixdoc_type_sig(doc);
    assert_eq!(from_cstring(sig), "f :: Int -> Int\n");
    nixdoc_free_string(sig);
    nixdoc_free(doc);
  }
}

#[test]
fn type_sig_none_when_absent() {
  let input = to_cstring("/** Just a description. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let sig = nixdoc_type_sig(doc);
    assert_eq!(from_cstring(sig), "");
    nixdoc_free_string(sig);
    nixdoc_free(doc);
  }
}

#[test]
fn is_deprecated_null_doc() {
  let result = unsafe { nixdoc_is_deprecated(std::ptr::null()) };
  assert!(!result);
}

#[test]
fn is_deprecated_true() {
  let input = to_cstring("/** Old.\n\n# Deprecated\n\nUse new instead.\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    assert!(nixdoc_is_deprecated(doc));
    nixdoc_free(doc);
  }
}

#[test]
fn is_deprecated_false() {
  let input = to_cstring("/** Current. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    assert!(!nixdoc_is_deprecated(doc));
    nixdoc_free(doc);
  }
}

#[test]
fn deprecation_notice_null_doc() {
  let result = unsafe { nixdoc_deprecation_notice(std::ptr::null()) };
  assert_eq!(from_cstring(result), "");
  unsafe { nixdoc_free_string(result) };
}

#[test]
fn deprecation_notice_present() {
  let input = to_cstring("/** Old.\n\n# Deprecated\n\nUse new instead.\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let notice = nixdoc_deprecation_notice(doc);
    assert_eq!(from_cstring(notice), "Use new instead.");
    nixdoc_free_string(notice);
    nixdoc_free(doc);
  }
}

#[test]
fn deprecation_notice_absent() {
  let input = to_cstring("/** Current. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let notice = nixdoc_deprecation_notice(doc);
    assert_eq!(from_cstring(notice), "");
    nixdoc_free_string(notice);
    nixdoc_free(doc);
  }
}

#[test]
fn arguments_null_doc() {
  let result = unsafe { nixdoc_arguments(std::ptr::null()) };
  assert!(result.is_null());
}

#[test]
fn arguments_with_args() {
  let input =
    to_cstring("/** f.\n\n# Arguments\n\n- [x] First\n- [y] Second\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let args = nixdoc_arguments(doc);
    assert!(!args.is_null());
    assert_eq!((*args).len, 2);
    nixdoc_free_string_array(args);
    nixdoc_free(doc);
  }
}

#[test]
fn arguments_no_args() {
  let input = to_cstring("/** Simple. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let args = nixdoc_arguments(doc);
    assert!(!args.is_null());
    assert_eq!((*args).len, 0);
    assert!((*args).data.is_null());
    nixdoc_free_string_array(args);
    nixdoc_free(doc);
  }
}

#[test]
fn examples_null_doc() {
  let result = unsafe { nixdoc_examples(std::ptr::null()) };
  assert!(result.is_null());
}

#[test]
fn examples_with_examples() {
  let input = to_cstring("/** f.\n\n# Example\n\n```nix\nf 1\n```\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let examples = nixdoc_examples(doc);
    assert!(!examples.is_null());
    assert_eq!((*examples).len, 1);
    nixdoc_free_string_array(examples);
    nixdoc_free(doc);
  }
}

#[test]
fn notes_null_doc() {
  let result = unsafe { nixdoc_notes(std::ptr::null()) };
  assert!(result.is_null());
}

#[test]
fn notes_with_notes() {
  let input = to_cstring("/** f.\n\n# Note\n\nImportant info.\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let notes = nixdoc_notes(doc);
    assert!(!notes.is_null());
    assert_eq!((*notes).len, 1);
    nixdoc_free_string_array(notes);
    nixdoc_free(doc);
  }
}

#[test]
fn warnings_null_doc() {
  let result = unsafe { nixdoc_warnings(std::ptr::null()) };
  assert!(result.is_null());
}

#[test]
fn warnings_with_warnings() {
  let input = to_cstring("/** f.\n\n# Warning\n\nThis is a warning.\n*/");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let warnings = nixdoc_warnings(doc);
    assert!(!warnings.is_null());
    assert_eq!((*warnings).len, 1);
    nixdoc_free_string_array(warnings);
    nixdoc_free(doc);
  }
}

#[test]
fn free_string_null_ptr() {
  unsafe { nixdoc_free_string(std::ptr::null_mut()) };
}

#[test]
fn free_string_array_null_ptr() {
  unsafe { nixdoc_free_string_array(std::ptr::null_mut()) };
}

#[test]
fn free_string_array_empty() {
  let input = to_cstring("/** Simple. */");
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();
  unsafe {
    nixdoc_parse_into(input.as_ptr(), &mut doc);
    let args = nixdoc_arguments(doc);
    nixdoc_free_string_array(args);
    nixdoc_free(doc);
  }
}

#[test]
fn roundtrip_parse_and_free() {
  let input = to_cstring(
    "/**\n  Test function.\n\n  # Arguments\n\n  - [x] Input value\n\n  # Example\n\n  ```nix\n  test 1\n  ```\n*/",
  );
  let mut doc: *mut NixdocDocComment = std::ptr::null_mut();

  unsafe {
    let parse_result = nixdoc_parse_into(input.as_ptr(), &mut doc);
    assert_eq!(parse_result, NIXDOC_SUCCESS);
    assert!(!doc.is_null());

    let title = nixdoc_title(doc);
    assert_eq!(from_cstring(title), "Test function.");
    nixdoc_free_string(title);

    let desc = nixdoc_description(doc);
    assert_eq!(from_cstring(desc), "Test function.");
    nixdoc_free_string(desc);

    let args = nixdoc_arguments(doc);
    assert_eq!((*args).len, 1);
    nixdoc_free_string_array(args);

    let examples = nixdoc_examples(doc);
    assert_eq!((*examples).len, 1);
    nixdoc_free_string_array(examples);

    nixdoc_free(doc);
  }
}
