use std::{
    ffi::{CStr, CString},
    ptr,
};

use super::ffi;

/// The DSL prelude is embedded at compile time so the binary is self-contained.
/// It is evaluated once during `MrubyVm::new()` — users never need `require`.
const PRELUDE: &str = include_str!("prelude.rb");

/// Safe wrapper around a live mRuby interpreter instance.
///
/// Automatically calls `mrb_close` when dropped.
pub struct MrubyVm {
    mrb: *mut ffi::MrbState,
}

impl MrubyVm {
    /// Open a new mRuby interpreter with the rrcad DSL prelude pre-loaded.
    ///
    /// All DSL classes and top-level methods (`box`, `cylinder`, `sphere`, …)
    /// are available immediately — no `require` statement is needed.
    ///
    /// # Panics
    /// Panics if `mrb_open()` returns null (out of memory), or if the
    /// built-in prelude fails to parse (indicates a bug in prelude.rb).
    pub fn new() -> Self {
        let mrb = unsafe { ffi::mrb_open() };
        assert!(!mrb.is_null(), "mrb_open() failed: out of memory");
        let mut vm = Self { mrb };
        vm.eval(PRELUDE)
            .unwrap_or_else(|e| panic!("rrcad prelude failed to load: {e}"));
        vm
    }

    /// Evaluate `code` as Ruby source and return the `inspect` string of
    /// the result, or an error description if an exception was raised.
    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        let c_code = CString::new(code).map_err(|e| e.to_string())?;
        let mut error_ptr: *const std::ffi::c_char = ptr::null();

        let result_ptr = unsafe { ffi::rrcad_mrb_eval(self.mrb, c_code.as_ptr(), &mut error_ptr) };

        if result_ptr.is_null() {
            let msg = if error_ptr.is_null() {
                "unknown error".to_string()
            } else {
                unsafe { CStr::from_ptr(error_ptr).to_string_lossy().into_owned() }
            };
            Err(msg)
        } else {
            let val = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
            Ok(val)
        }
    }
}

impl Drop for MrubyVm {
    fn drop(&mut self) {
        unsafe { ffi::mrb_close(self.mrb) }
    }
}
