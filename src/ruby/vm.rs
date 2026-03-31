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

impl Default for MrubyVm {
    fn default() -> Self {
        Self::new()
    }
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
        // SAFETY: mrb_open() allocates a new mRuby state and returns a valid
        // pointer, or NULL on OOM (caught by the assert below).
        let mrb = unsafe { ffi::mrb_open() };
        assert!(!mrb.is_null(), "mrb_open() failed: out of memory");
        let mut vm = Self { mrb };
        vm.eval(PRELUDE)
            .unwrap_or_else(|e| panic!("rrcad prelude failed to load: {e}"));
        // Register native Shape class and top-level methods *after* the prelude
        // so native implementations shadow the Ruby stubs.
        // SAFETY: mrb is a valid, fully initialised MrbState pointer as
        // confirmed by the non-null assert above.
        unsafe { ffi::rrcad_register_shape_class(mrb) };
        vm
    }

    /// Inject CLI parameter overrides into the VM as the `$_rrcad_params` global.
    ///
    /// Must be called after `new()` and before evaluating the user script.
    /// Values are injected as Ruby strings; the `param` DSL method coerces them
    /// to the type of the declared default.
    pub fn set_params(&mut self, params: &[(String, String)]) -> Result<(), String> {
        if params.is_empty() {
            return Ok(());
        }
        let entries: String = params
            .iter()
            .map(|(k, v)| {
                // Escape backslashes first, then double-quotes, so the generated
                // Ruby string literal is well-formed for any input value.
                // Order matters: escaping `\` after `"` would double-escape the
                // backslashes we just inserted for the quote escapes.
                let k = k.replace('\\', "\\\\").replace('"', "\\\"");
                let v = v.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{k}\" => \"{v}\"")
            })
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!("$_rrcad_params = {{{entries}}}");
        self.eval(&code).map(|_| ())
    }

    /// Evaluate `code` as Ruby source and return the `inspect` string of
    /// the result, or an error description if an exception was raised.
    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        let c_code = CString::new(code).map_err(|e| e.to_string())?;
        let mut error_ptr: *const std::ffi::c_char = ptr::null();

        // SAFETY: self.mrb is a valid MrbState for the lifetime of this MrubyVm,
        // and c_code is a valid null-terminated C string for the duration of this call.
        let result_ptr = unsafe { ffi::rrcad_mrb_eval(self.mrb, c_code.as_ptr(), &mut error_ptr) };

        if result_ptr.is_null() {
            let msg = if error_ptr.is_null() {
                "unknown error".to_string()
            } else {
                // SAFETY: error_ptr is a GC-managed mRuby string; rrcad_mrb_eval
                // guarantees it is valid until the next mRuby allocation.
                // into_owned() copies it into Rust immediately.
                unsafe { CStr::from_ptr(error_ptr).to_string_lossy().into_owned() }
            };
            Err(msg)
        } else {
            // SAFETY: result_ptr is a GC-managed mRuby string; same lifetime
            // guarantee as error_ptr above — copied immediately via into_owned().
            let val = unsafe { CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
            Ok(val)
        }
    }
}

impl Drop for MrubyVm {
    fn drop(&mut self) {
        // SAFETY: self.mrb is the unique owner of this mRuby state.
        // mrb_close frees all associated memory; no other reference exists.
        unsafe { ffi::mrb_close(self.mrb) }
    }
}
