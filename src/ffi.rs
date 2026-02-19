#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::panic::catch_unwind;
use std::ptr;
use std::slice;

use crate::DocComment;

const NIXDOC_SUCCESS: c_int = 0;
const NIXDOC_ERROR_PARSE: c_int = 1;
const NIXDOC_ERROR_NULL: c_int = 2;
const NIXDOC_ERROR_PANIC: c_int = 3;

#[repr(C)]
pub struct NixdocDocComment {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NixdocStringArray {
    pub data: *mut *mut c_char,
    pub len: usize,
}

/// Parses a Nix doc comment string.
///
/// # Safety
///
/// `input` must be a valid, null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_parse(input: *const c_char) -> c_int {
    if input.is_null() {
        return NIXDOC_ERROR_NULL;
    }

    let result = catch_unwind(|| {
        let input_str = std::ffi::CStr::from_ptr(input)
            .to_string_lossy()
            .into_owned();
        DocComment::parse(&input_str).is_ok()
    });

    match result {
        Ok(true) => NIXDOC_SUCCESS,
        Ok(false) => NIXDOC_ERROR_PARSE,
        Err(_) => NIXDOC_ERROR_PANIC,
    }
}

/// Parses a Nix doc comment string and stores the result in `out_doc`.
///
/// # Safety
///
/// - `input` must be a valid, null-terminated C string.
/// - `out_doc` must point to a valid `*mut NixdocDocComment` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_parse_into(
    input: *const c_char,
    out_doc: *mut *mut NixdocDocComment,
) -> c_int {
    if input.is_null() || out_doc.is_null() {
        return NIXDOC_ERROR_NULL;
    }

    let result = catch_unwind(|| {
        let input_str = std::ffi::CStr::from_ptr(input)
            .to_string_lossy()
            .into_owned();
        DocComment::parse(&input_str).map(|doc| {
            let boxed = Box::new(doc);
            let ptr = Box::into_raw(boxed) as *mut NixdocDocComment;
            *out_doc = ptr;
        })
    });

    match result {
        Ok(Ok(())) => NIXDOC_SUCCESS,
        Ok(Err(_)) => NIXDOC_ERROR_PARSE,
        Err(_) => NIXDOC_ERROR_PANIC,
    }
}

/// Frees a `NixdocDocComment` pointer returned by `nixdoc_parse_into`.
///
/// # Safety
///
/// `ptr` must be a valid pointer returned by `nixdoc_parse_into`, and must not be
/// called more than once on the same pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_free(ptr: *mut NixdocDocComment) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// Checks whether the given input is a valid Nix doc comment.
///
/// # Safety
///
/// `input` must be a valid, null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_is_doc_comment(input: *const c_char) -> bool {
    if input.is_null() {
        return false;
    }

    let result = catch_unwind(|| {
        let input_str = std::ffi::CStr::from_ptr(input)
            .to_string_lossy()
            .into_owned();
        DocComment::is_doc_comment(&input_str)
    });

    result.unwrap_or(false)
}

fn rust_string_to_cstring(s: &str) -> *mut c_char {
    use std::ffi::CString;
    CString::new(s)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

/// Gets the title from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_title(doc: *const NixdocDocComment) -> *mut c_char {
    if doc.is_null() {
        return rust_string_to_cstring("");
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        doc.title()
            .map(rust_string_to_cstring)
            .unwrap_or(ptr::null_mut())
    });

    result.unwrap_or(ptr::null_mut())
}

/// Gets the description from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_description(doc: *const NixdocDocComment) -> *mut c_char {
    if doc.is_null() {
        return rust_string_to_cstring("");
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        rust_string_to_cstring(doc.description())
    });

    result.unwrap_or(rust_string_to_cstring(""))
}

/// Gets the type signature from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_type_sig(doc: *const NixdocDocComment) -> *mut c_char {
    if doc.is_null() {
        return rust_string_to_cstring("");
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        doc.type_sig()
            .map(|s| rust_string_to_cstring(&s))
            .unwrap_or(ptr::null_mut())
    });

    result.unwrap_or(ptr::null_mut())
}

/// Checks whether a parsed doc comment is deprecated.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_is_deprecated(doc: *const NixdocDocComment) -> bool {
    if doc.is_null() {
        return false;
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        doc.is_deprecated()
    });

    result.unwrap_or(false)
}

/// Gets the deprecation notice from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_deprecation_notice(doc: *const NixdocDocComment) -> *mut c_char {
    if doc.is_null() {
        return rust_string_to_cstring("");
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        doc.deprecation_notice()
            .map(rust_string_to_cstring)
            .unwrap_or(ptr::null_mut())
    });

    result.unwrap_or(ptr::null_mut())
}

/// Gets the arguments from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`. The returned
/// `NixdocStringArray` must be freed with `nixdoc_free_string_array`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_arguments(doc: *const NixdocDocComment) -> *mut NixdocStringArray {
    if doc.is_null() {
        return ptr::null_mut();
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        let args = doc.arguments();

        let len = args.len();
        if len == 0 {
            return Box::into_raw(Box::new(NixdocStringArray {
                data: ptr::null_mut(),
                len: 0,
            }));
        }

        let items: Vec<*mut c_char> = args
            .iter()
            .map(|arg| {
                let combined = format!("{}: {}", arg.name, arg.description);
                rust_string_to_cstring(&combined)
            })
            .collect();

        let data = items.as_ptr() as *mut *mut c_char;
        std::mem::forget(items);

        Box::into_raw(Box::new(NixdocStringArray { data, len }))
    });

    result.unwrap_or(ptr::null_mut())
}

/// Gets the examples from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`. The returned
/// `NixdocStringArray` must be freed with `nixdoc_free_string_array`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_examples(doc: *const NixdocDocComment) -> *mut NixdocStringArray {
    if doc.is_null() {
        return ptr::null_mut();
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        let examples = doc.examples();

        let len = examples.len();
        if len == 0 {
            return Box::into_raw(Box::new(NixdocStringArray {
                data: ptr::null_mut(),
                len: 0,
            }));
        }

        let items: Vec<*mut c_char> = examples
            .iter()
            .map(|ex| {
                let lang = ex.language.as_deref().unwrap_or("");
                let combined = format!("{}: {}", lang, ex.code);
                rust_string_to_cstring(&combined)
            })
            .collect();

        let data = items.as_ptr() as *mut *mut c_char;
        std::mem::forget(items);

        Box::into_raw(Box::new(NixdocStringArray { data, len }))
    });

    result.unwrap_or(ptr::null_mut())
}

/// Gets the notes from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`. The returned
/// `NixdocStringArray` must be freed with `nixdoc_free_string_array`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_notes(doc: *const NixdocDocComment) -> *mut NixdocStringArray {
    if doc.is_null() {
        return ptr::null_mut();
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        let notes = doc.notes();

        let len = notes.len();
        if len == 0 {
            return Box::into_raw(Box::new(NixdocStringArray {
                data: ptr::null_mut(),
                len: 0,
            }));
        }

        let items: Vec<*mut c_char> = notes
            .iter()
            .map(|note| rust_string_to_cstring(note))
            .collect();

        let data = items.as_ptr() as *mut *mut c_char;
        std::mem::forget(items);

        Box::into_raw(Box::new(NixdocStringArray { data, len }))
    });

    result.unwrap_or(ptr::null_mut())
}

/// Gets the warnings from a parsed doc comment.
///
/// # Safety
///
/// `doc` must be a valid pointer returned by `nixdoc_parse_into`. The returned
/// `NixdocStringArray` must be freed with `nixdoc_free_string_array`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_warnings(doc: *const NixdocDocComment) -> *mut NixdocStringArray {
    if doc.is_null() {
        return ptr::null_mut();
    }

    let result = catch_unwind(|| {
        let doc = &*(doc as *const DocComment);
        let warnings = doc.warnings_content();

        let len = warnings.len();
        if len == 0 {
            return Box::into_raw(Box::new(NixdocStringArray {
                data: ptr::null_mut(),
                len: 0,
            }));
        }

        let items: Vec<*mut c_char> = warnings.iter().map(|w| rust_string_to_cstring(w)).collect();

        let data = items.as_ptr() as *mut *mut c_char;
        std::mem::forget(items);

        Box::into_raw(Box::new(NixdocStringArray { data, len }))
    });

    result.unwrap_or(ptr::null_mut())
}

/// Frees a C string returned by any string-returning function.
///
/// # Safety
///
/// `ptr` must be a valid pointer returned by a `nixdoc_*` function that returns
/// a `*mut c_char`, and must not be called more than once on the same pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Frees a `NixdocStringArray` returned by the accessor functions.
///
/// # Safety
///
/// `arr` must be a valid pointer returned by `nixdoc_arguments`, `nixdoc_examples`,
/// `nixdoc_notes`, or `nixdoc_warnings`, and must not be called more than once
/// on the same pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nixdoc_free_string_array(arr: *mut NixdocStringArray) {
    if arr.is_null() {
        return;
    }

    let arr = &mut *arr;
    if !arr.data.is_null() && arr.len > 0 {
        let slice = slice::from_raw_parts_mut(arr.data, arr.len);
        for ptr in slice.iter() {
            if !ptr.is_null() {
                drop(CString::from_raw(*ptr));
            }
        }
        drop(Vec::from_raw_parts(slice.as_mut_ptr(), arr.len, arr.len));
    }
    drop(Box::from_raw(arr));
}
