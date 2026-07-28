// NOTE: the module-level safety contract in `native.rs` applies to every `extern "C"` function in this file.

use std::ffi::{CString, c_char, c_void};
use std::path::{Path, PathBuf};

use crate::occt::Shape;

// ---------------------------------------------------------------------------
// Path-traversal guard
// ---------------------------------------------------------------------------

/// Resolve `raw` to a canonical absolute path and verify it is inside the
/// current working directory.
pub(crate) fn safe_path(raw: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(raw);
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let canon_cwd = cwd
        .canonicalize()
        .map_err(|e| format!("cannot resolve cwd: {e}"))?;

    let canonical = if p.exists() {
        p.canonicalize()
            .map_err(|e| format!("cannot resolve path '{raw}': {e}"))?
    } else {
        let parent = p.parent().unwrap_or(Path::new(""));
        let canon_parent = if parent == Path::new("") {
            canon_cwd.clone()
        } else {
            parent
                .canonicalize()
                .map_err(|e| format!("cannot resolve directory for '{raw}': {e}"))?
        };
        canon_parent.join(
            p.file_name()
                .ok_or_else(|| format!("invalid path (no filename component): '{raw}'"))?,
        )
    };

    if !canonical.starts_with(&canon_cwd) {
        return Err(format!(
            "path '{raw}' is outside the working directory (path traversal rejected)"
        ));
    }
    Ok(canonical)
}

// ---------------------------------------------------------------------------
// Thread-local error / string slots
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERR: std::cell::RefCell<Option<CString>> =
        const { std::cell::RefCell::new(None) };
    static LAST_STR: std::cell::RefCell<Option<CString>> =
        const { std::cell::RefCell::new(None) };
}

/// Store `msg` in the thread-local error slot and write its pointer to
/// `*error_out`. The pointer is valid until the next call to `set_err` on this
/// thread.
pub(crate) unsafe fn set_err(error_out: *mut *const c_char, msg: &str) {
    let cstr = CString::new(msg).unwrap_or_else(|_| c"<error contains nul>".to_owned());
    LAST_ERR.with(|cell| {
        // Rust 2024: an `unsafe fn` body is not implicitly unsafe.
        unsafe {
            *error_out = cstr.as_ptr();
        }
        *cell.borrow_mut() = Some(cstr);
    });
}

/// Store `s` in the thread-local string slot and write its pointer to `*out`.
/// The pointer is valid until the next call to `set_str` on this thread.
pub(crate) unsafe fn set_str(out: *mut *const c_char, s: &str) {
    let cstr = CString::new(s).unwrap_or_else(|_| c"<string contains nul>".to_owned());
    LAST_STR.with(|cell| {
        unsafe {
            *out = cstr.as_ptr();
        }
        *cell.borrow_mut() = Some(cstr);
    });
}

/// Store `s` in the thread-local string slot and return a pointer to it.
/// The pointer is valid until the next string-returning call on this thread.
///
/// # Safety
/// The returned pointer must not outlive the next `set_str` / `owned_str_ptr`
/// call on the same thread (see the module-level safety contract).
pub(crate) unsafe fn owned_str_ptr(s: &str) -> *const c_char {
    let mut raw: *const c_char = std::ptr::null();
    unsafe { set_str(&mut raw as *mut *const c_char, s) };
    raw
}

/// Convert a `Result<Shape, String>` into a raw pointer for the C FFI return
/// value. Callers must clear `*error_out` before calling this.
pub(crate) unsafe fn shape_result_to_ptr(
    result: Result<Shape, String>,
    error_out: *mut *const c_char,
) -> *mut c_void {
    match result {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

/// Default linear deflection for mesh tessellation (export_gltf, export_glb,
/// export_obj).
pub(crate) const DEFAULT_LINEAR_DEFLECTION: f64 = 0.1;

/// Decode the raw C `path` pointer and validate it with `safe_path`.
pub(crate) unsafe fn resolve_path(
    path: *const c_char,
    error_out: *mut *const c_char,
) -> Option<PathBuf> {
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return None;
        }
    };
    match safe_path(path_str) {
        Ok(p) => Some(p),
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            None
        }
    }
}

/// Decode a borrowed C string argument, reporting
/// `"<what> is not valid UTF-8"` through `*error_out` when the bytes are not
/// UTF-8. Returns `None` in that case so the caller can bail out with its own
/// error sentinel (null pointer, `-1`, …).
///
/// # Safety
/// `ptr` must be a valid NUL-terminated string that stays alive for at least
/// as long as the returned `&str` is used, and `error_out` must be writable.
/// The returned lifetime is unbounded; callers only ever use it inside the
/// single FFI call that produced it (see the module-level safety contract).
pub(crate) unsafe fn utf8_arg<'a>(
    ptr: *const c_char,
    what: &str,
    error_out: *mut *const c_char,
) -> Option<&'a str> {
    match unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => Some(s),
        Err(_) => {
            unsafe { set_err(error_out, &format!("{what} is not valid UTF-8")) };
            None
        }
    }
}

/// Borrow `n` `Shape`s from a C array of raw `Box<Shape>` pointers.
///
/// # Safety
/// `ptrs` must point to `n` consecutive, live `Box<Shape>` pointers produced by
/// this module (see the module-level safety contract).
pub(crate) unsafe fn shape_refs<'a>(ptrs: *const *const c_void, n: usize) -> Vec<&'a Shape> {
    (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect()
}

pub(crate) unsafe fn cstr_arg(ptr: *const c_char) -> Result<String, String> {
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| "value is not valid UTF-8".to_string())
}

pub(crate) fn split_csv_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{cstr_arg, resolve_path, safe_path, split_csv_list};
    use crate::test_util::unique_test_dir;
    use std::{ffi::CString, fs, os::raw::c_char, ptr};

    #[test]
    fn safe_path_accepts_relative_paths_inside_cwd() {
        let dir = unique_test_dir("rrcad-safe-path");
        fs::create_dir_all(&dir).expect("create temp dir");
        fs::create_dir_all(dir.join("exports")).expect("create exports dir");
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(&dir).expect("enter temp dir");

        let path = safe_path("exports/part.step").expect("path inside cwd");
        assert_eq!(path, dir.join("exports/part.step"));

        std::env::set_current_dir(original).expect("restore cwd");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn safe_path_rejects_path_traversal() {
        let dir = unique_test_dir("rrcad-safe-path");
        fs::create_dir_all(&dir).expect("create temp dir");

        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(&dir).expect("enter temp dir");

        let err = safe_path("../outside.step").expect_err("path traversal should be rejected");
        assert!(
            err.contains("outside the working directory"),
            "unexpected error: {err}"
        );

        std::env::set_current_dir(original).expect("restore cwd");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn split_csv_list_trims_and_drops_empty_items() {
        let values = split_csv_list(" alpha, ,beta,, gamma ,");
        assert_eq!(values, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn cstr_arg_accepts_valid_utf8() {
        let c = CString::new("hello").expect("CString");
        let value = unsafe { cstr_arg(c.as_ptr()) }.expect("utf8");
        assert_eq!(value, "hello");
    }

    #[test]
    fn cstr_arg_rejects_invalid_utf8() {
        let bytes = [0xffu8, 0x00];
        let ptr = bytes.as_ptr() as *const c_char;
        let err = unsafe { cstr_arg(ptr) }.expect_err("invalid utf8 should fail");
        assert!(err.contains("valid UTF-8"));
    }

    #[test]
    fn resolve_path_accepts_safe_relative_path() {
        let dir = unique_test_dir("rrcad-resolve-path");
        fs::create_dir_all(&dir).expect("create temp dir");
        fs::create_dir_all(dir.join("exports")).expect("create exports dir");
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(&dir).expect("enter temp dir");

        let c_path = CString::new("exports/shape.step").expect("CString");
        let mut err: *const c_char = ptr::null();
        let resolved = unsafe { resolve_path(c_path.as_ptr(), &mut err as *mut *const c_char) }
            .expect("safe path");
        assert_eq!(resolved, dir.join("exports/shape.step"));
        assert!(err.is_null(), "error pointer should remain null");

        std::env::set_current_dir(original).expect("restore cwd");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_path_rejects_invalid_utf8() {
        let bytes = [0xffu8, 0x00];
        let mut err: *const c_char = ptr::null();
        let result = unsafe { resolve_path(bytes.as_ptr() as *const c_char, &mut err) };
        assert!(result.is_none());
        assert!(!err.is_null());
    }
}
