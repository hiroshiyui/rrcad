// Clippy's `missing_safety_doc` lint is suppressed for this module because
// all `extern "C"` functions here share the same safety contract (documented
// in the module-level doc comment below), and repeating it on every one of
// the ~45 entry points would be pure noise.
#![allow(clippy::missing_safety_doc)]

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
#[path = "native_profile_ops.rs"]
mod native_profile_ops;
#[path = "native_selector_ops.rs"]
mod native_selector_ops;

use std::ffi::{c_char, c_void};

use self::native_helpers::shape_result_to_ptr;
use crate::occt::Shape;

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_box(
    dx: f64,
    dy: f64,
    dz: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_box(dx, dy, dz), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_cylinder(
    r: f64,
    h: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_cylinder(r, h), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_sphere(r: f64, error_out: *mut *const c_char) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_sphere(r), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_cone(
    r1: f64,
    r2: f64,
    h: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_cone(r1, r2, h), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_torus(
    r1: f64,
    r2: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_torus(r1, r2), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_wedge(
    dx: f64,
    dy: f64,
    dz: f64,
    ltx: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_wedge(dx, dy, dz, ltx), error_out) }
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_linear_pattern(
    ptr: *mut c_void,
    n: std::ffi::c_int,
    dx: f64,
    dy: f64,
    dz: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.linear_pattern(n, dx, dy, dz), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_polar_pattern(
    ptr: *mut c_void,
    n: std::ffi::c_int,
    angle_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.polar_pattern(n, angle_deg), error_out) }
}
