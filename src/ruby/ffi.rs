/// Raw `extern "C"` bindings to libmruby and our glue shims.
///
/// `mrb_value` is intentionally not exposed here — all value manipulation
/// is handled in `glue.c`.  Rust only sees opaque pointers and plain C types.
use std::ffi::c_char;

/// Opaque handle to the mRuby interpreter state (`mrb_state *`).
#[repr(C)]
pub struct MrbState {
    _private: [u8; 0],
}

unsafe extern "C" {
    /// Open a new mRuby interpreter.  Returns NULL on allocation failure.
    pub fn mrb_open() -> *mut MrbState;

    /// Close and free an mRuby interpreter.
    pub fn mrb_close(mrb: *mut MrbState);

    /// Evaluate `code` and return the `inspect` string of the result.
    ///
    /// See `glue.c` for the full contract.
    pub fn rrcad_mrb_eval(
        mrb: *mut MrbState,
        code: *const c_char,
        error_out: *mut *const c_char,
    ) -> *const c_char;

    /// Register the native `Shape` class and the `box` / `cylinder` / `sphere`
    /// top-level methods.  Must be called after the DSL prelude is evaluated so
    /// that native methods shadow the Ruby stubs.
    pub fn rrcad_register_shape_class(mrb: *mut MrbState);
}
