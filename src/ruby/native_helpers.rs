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
    use super::safe_path;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()))
    }

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
}
