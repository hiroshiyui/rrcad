// Clippy's `missing_safety_doc` lint is suppressed for this module because
// all `extern "C"` functions here share the same safety contract (documented
// in the module-level doc comment below), and repeating it on every one of
// the ~45 entry points would be pure noise.
#![allow(clippy::missing_safety_doc)] // shared safety contract documented once in the module doc below

//! Rust-side extern "C" functions called from `glue.c`.
//!
//! Each constructor allocates a heap `Box<occt::Shape>` and returns the raw
//! pointer cast to `*mut c_void`.  The C `dfree` callback (`rrcad_shape_drop`)
//! reclaims that memory when mRuby GC collects the `RData` object.
//!
//! Error reporting: when an OCCT operation fails the function writes a pointer
//! to a thread-local `CString` into `*error_out` and returns null.  The C
//! caller checks `error_out` and raises a Ruby `RuntimeError` before the
//! thread-local slot is overwritten.
//!
//! # Safety contract (applies to every `extern "C"` function in this file)
//!
//! All functions in this module are C FFI entry points; they are only called
//! from `glue.c`, never from safe Rust.  Callers must ensure:
//! - `ptr` / `a` / `b` / `profile` / `path` point to a live `Box<Shape>` that
//!   was produced by this module and has not yet been freed.
//! - `error_out` is a valid non-null pointer to a `*const c_char` slot.
//! - All string/slice pointers (`path`, `pts`, `selector`, `plane`) are valid
//!   for the duration of the call.

// ---------------------------------------------------------------------------
// FFI wrapper macros
//
// Almost every entry point in this module (and its sub-modules) follows one of
// a handful of shapes: clear `*error_out`, borrow the receiver, call exactly
// one `Shape` method, then convert the result to its C representation.  The
// macros below generate that code verbatim so each wrapper stays a single
// declarative line.
//
// Every macro emits the same `#[unsafe(no_mangle)] pub unsafe extern "C" fn`
// signature that `glue.c` declares, and the `unsafe` blocks they generate rely
// on exactly the module-level safety contract documented above: the caller
// (always `glue.c`) guarantees live `Box<Shape>` pointers, valid string
// arguments and a writable `error_out` slot.  Nothing about the memory model
// changes — Rust still owns the `Box<Shape>`, and `rrcad_shape_drop` remains
// the only drop path.
//
// The macros are defined here, before the `mod` declarations, so that they are
// textually in scope for every sub-module of `native`.
//
// Argument *kind* tags used by the macros:
//   `shape` — opaque `*mut c_void` holding a `Box<Shape>`; borrowed as `&Shape`
//   `str`   — `*const c_char`; decoded as UTF-8, bailing out on invalid input
//   `f64`   — plain double
//   `int`   — plain `c_int`
// ---------------------------------------------------------------------------

/// C ABI type for an argument kind tag.
macro_rules! ffi_ty {
    (shape) => {
        *mut ::std::ffi::c_void
    };
    (str) => {
        *const ::std::ffi::c_char
    };
    (f64) => {
        f64
    };
    (int) => {
        ::std::ffi::c_int
    };
}

/// Statement prologue that rebinds a raw FFI argument to the value handed to
/// the `Shape` method.
///
/// `shape` borrows the opaque pointer as `&Shape` and `str` decodes the C
/// string (bailing out with `$bail`, the caller's error sentinel, when the
/// bytes are not UTF-8).  Plain scalars need no prologue and expand to nothing.
macro_rules! ffi_prologue {
    (shape, $arg:ident, $error_out:ident, $bail:expr) => {
        let $arg = unsafe { &*($arg as *const $crate::occt::Shape) };
    };
    (str, $arg:ident, $error_out:ident, $bail:expr) => {
        let Some($arg) = (unsafe {
            $crate::ruby::native::native_helpers::utf8_arg($arg, stringify!($arg), $error_out)
        }) else {
            return $bail;
        };
    };
    (f64, $arg:ident, $error_out:ident, $bail:expr) => {};
    (int, $arg:ident, $error_out:ident, $bail:expr) => {};
}

/// Wrap a `Shape` *associated function* that returns `Result<Shape, String>`.
///
/// Generates `fn NAME(args..., error_out) -> *mut c_void`.
macro_rules! shape_ctor {
    (
        $(#[$meta:meta])*
        $name:ident($($arg:ident : $kind:ident),* $(,)?) => $ctor:path
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            $($arg: ffi_ty!($kind),)*
            error_out: *mut *const ::std::ffi::c_char,
        ) -> *mut ::std::ffi::c_void {
            unsafe { *error_out = ::std::ptr::null() };
            $(ffi_prologue!($kind, $arg, error_out, ::std::ptr::null_mut());)*
            unsafe {
                $crate::ruby::native::native_helpers::shape_result_to_ptr(
                    $ctor($($arg),*),
                    error_out,
                )
            }
        }
    };
}

/// Wrap a `&Shape` *method* that returns `Result<Shape, String>`.
///
/// Generates `fn NAME(ptr, args..., error_out) -> *mut c_void`.
macro_rules! shape_method {
    (
        $(#[$meta:meta])*
        $name:ident($($arg:ident : $kind:ident),* $(,)?) => $method:ident
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            ptr: *mut ::std::ffi::c_void,
            $($arg: ffi_ty!($kind),)*
            error_out: *mut *const ::std::ffi::c_char,
        ) -> *mut ::std::ffi::c_void {
            unsafe { *error_out = ::std::ptr::null() };
            let shape = unsafe { &*(ptr as *const $crate::occt::Shape) };
            $(ffi_prologue!($kind, $arg, error_out, ::std::ptr::null_mut());)*
            unsafe {
                $crate::ruby::native::native_helpers::shape_result_to_ptr(
                    shape.$method($($arg),*),
                    error_out,
                )
            }
        }
    };
}

/// Wrap a `&Shape` method that returns `Result<f64, String>`.
///
/// `on_error` is the value returned when the method fails (the existing
/// wrappers use `0.0` for measurements and `NAN` for distances).
macro_rules! shape_scalar {
    (
        $(#[$meta:meta])*
        $name:ident($($arg:ident : $kind:ident),* $(,)?) => $method:ident, on_error = $fallback:expr
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            ptr: *mut ::std::ffi::c_void,
            $($arg: ffi_ty!($kind),)*
            error_out: *mut *const ::std::ffi::c_char,
        ) -> f64 {
            unsafe { *error_out = ::std::ptr::null() };
            let shape = unsafe { &*(ptr as *const $crate::occt::Shape) };
            $(ffi_prologue!($kind, $arg, error_out, $fallback);)*
            match shape.$method($($arg),*) {
                Ok(v) => v,
                Err(e) => {
                    unsafe { $crate::ruby::native::native_helpers::set_err(error_out, &e) };
                    $fallback
                }
            }
        }
    };
}

/// Wrap a `&Shape` predicate that returns `Result<bool, String>`.
///
/// Returns 1 / 0, or -1 with `*error_out` set on failure.
macro_rules! shape_flag {
    (
        $(#[$meta:meta])*
        $name:ident => $method:ident
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            ptr: *mut ::std::ffi::c_void,
            error_out: *mut *const ::std::ffi::c_char,
        ) -> ::std::ffi::c_int {
            unsafe { *error_out = ::std::ptr::null() };
            let shape = unsafe { &*(ptr as *const $crate::occt::Shape) };
            match shape.$method() {
                Ok(b) => b as ::std::ffi::c_int,
                Err(e) => {
                    unsafe { $crate::ruby::native::native_helpers::set_err(error_out, &e) };
                    -1
                }
            }
        }
    };
}

/// Wrap a `&Shape` method that returns `Result<[f64; N], String>`, copying the
/// `$len` doubles into the caller-allocated `out` buffer.
macro_rules! shape_array {
    (
        $(#[$meta:meta])*
        $name:ident => $method:ident, len = $len:expr
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            ptr: *mut ::std::ffi::c_void,
            out: *mut f64,
            error_out: *mut *const ::std::ffi::c_char,
        ) {
            unsafe { *error_out = ::std::ptr::null() };
            let shape = unsafe { &*(ptr as *const $crate::occt::Shape) };
            match shape.$method() {
                Ok(arr) => unsafe { ::std::ptr::copy_nonoverlapping(arr.as_ptr(), out, $len) },
                Err(e) => unsafe {
                    $crate::ruby::native::native_helpers::set_err(error_out, &e)
                },
            }
        }
    };
}

/// Wrap a `&Shape` method that returns `Result<String, String>`.
///
/// The returned pointer borrows the thread-local string slot and stays valid
/// until the next string-returning call on this thread.
macro_rules! shape_string {
    (
        $(#[$meta:meta])*
        $name:ident => $method:ident
    ) => {
        $(#[$meta])*
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            ptr: *mut ::std::ffi::c_void,
            error_out: *mut *const ::std::ffi::c_char,
        ) -> *const ::std::ffi::c_char {
            unsafe { *error_out = ::std::ptr::null() };
            let shape = unsafe { &*(ptr as *const $crate::occt::Shape) };
            match shape.$method() {
                Ok(s) => unsafe { $crate::ruby::native::native_helpers::owned_str_ptr(&s) },
                Err(e) => {
                    unsafe { $crate::ruby::native::native_helpers::set_err(error_out, &e) };
                    ::std::ptr::null()
                }
            }
        }
    };
}

#[path = "native_basic_ops.rs"]
mod native_basic_ops;
#[path = "native_helpers.rs"]
mod native_helpers;
#[path = "native_inspect.rs"]
mod native_inspect;
#[path = "native_io.rs"]
mod native_io;
#[path = "native_modeling_ops.rs"]
mod native_modeling_ops;
#[path = "native_output.rs"]
pub mod native_output;
#[path = "native_profile_ops.rs"]
mod native_profile_ops;
#[path = "native_selector_ops.rs"]
mod native_selector_ops;

use std::ffi::c_void;

use crate::occt::Shape;

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

shape_ctor!(rrcad_make_box(dx: f64, dy: f64, dz: f64) => Shape::make_box);
shape_ctor!(rrcad_make_cylinder(r: f64, h: f64) => Shape::make_cylinder);
shape_ctor!(rrcad_make_sphere(r: f64) => Shape::make_sphere);
shape_ctor!(rrcad_make_cone(r1: f64, r2: f64, h: f64) => Shape::make_cone);
shape_ctor!(rrcad_make_torus(r1: f64, r2: f64) => Shape::make_torus);
shape_ctor!(rrcad_make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64) => Shape::make_wedge);

// ---------------------------------------------------------------------------
// Destructor (called from mRuby dfree)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_drop(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr as *mut Shape)) };
    }
}

// ---------------------------------------------------------------------------
// Patterns (Phase 4)
// ---------------------------------------------------------------------------

shape_method!(rrcad_shape_linear_pattern(n: int, dx: f64, dy: f64, dz: f64) => linear_pattern);
shape_method!(rrcad_shape_polar_pattern(n: int, angle_deg: f64) => polar_pattern);
