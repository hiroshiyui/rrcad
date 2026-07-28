// NOTE: the module-level safety contract in `native.rs` applies to every `extern "C"` function in this file.

//! `require_relative` — the native half of multi-file script loading.
//!
//! The work is split so that `mrb_value` never crosses into Rust (the
//! invariant `glue.c` exists to maintain): Rust resolves the path and reads
//! the source, `glue.c` evaluates it. Because evaluation happens on the C
//! side, the include stack is pushed and popped by two separate calls, and
//! `glue.c` must pair them even when evaluation raises.
//!
//! See `src/ruby/loader.rs` for the semantics and the security rationale.

use std::ffi::c_char;

use crate::ruby::loader::{self, Begin};

/// Begin a `require_relative`.
///
/// Writes the source and filename to evaluate into `*code_out` / `*name_out`
/// and returns 1; returns 0 when the file was already loaded (nothing to do);
/// returns -1 on error, with `*error_out` set.
///
/// The pointers written to `*code_out` and `*name_out` are owned by the load
/// stack and stay valid until `rrcad_require_end`. On a return value of 1 the
/// caller **must** call `rrcad_require_end` once evaluation finishes, whether
/// or not it raised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_require_begin(
    arg: *const c_char,
    code_out: *mut *const c_char,
    name_out: *mut *const c_char,
    error_out: *mut *const c_char,
) -> i32 {
    unsafe {
        *error_out = std::ptr::null();
        *code_out = std::ptr::null();
        *name_out = std::ptr::null();
    }

    // On invalid UTF-8, utf8_arg has already reported through error_out.
    let Some(arg) =
        (unsafe { super::native_helpers::utf8_arg(arg, "require_relative path", error_out) })
    else {
        return -1;
    };

    match loader::begin_require(arg) {
        Ok(Begin::AlreadyLoaded) => 0,
        Ok(Begin::Evaluate { code, filename }) => {
            unsafe {
                *code_out = code;
                *name_out = filename;
            }
            1
        }
        Err(msg) => {
            unsafe { super::native_helpers::set_err(error_out, &msg) };
            -1
        }
    }
}

/// Pop the include stack. Must be called exactly once for each
/// `rrcad_require_begin` that returned 1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_require_end() {
    loader::end_require();
}
