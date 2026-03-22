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
use std::ffi::{CString, c_char, c_void};

use crate::occt::Shape;

// ---------------------------------------------------------------------------
// Thread-local error slot
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERR: std::cell::RefCell<Option<CString>> =
        const { std::cell::RefCell::new(None) };
}

/// Store `msg` in the thread-local slot and write its pointer to `*error_out`.
/// The pointer is valid until the next call to `set_err` on this thread.
unsafe fn set_err(error_out: *mut *const c_char, msg: &str) {
    let cstr = CString::new(msg).unwrap_or_else(|_| c"<error contains nul>".to_owned());
    LAST_ERR.with(|cell| {
        unsafe {
            *error_out = cstr.as_ptr();
        }
        *cell.borrow_mut() = Some(cstr);
    });
}

/// Convert a `Result<Shape, String>` into a raw pointer for the C FFI return value.
///
/// On success: boxes the shape and returns the raw pointer.
/// On error: writes the error message into `*error_out` and returns null.
///
/// Callers must clear `*error_out` (set it to null) before calling this.
unsafe fn shape_result_to_ptr(
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
pub unsafe extern "C" fn rrcad_make_sphere(
    r: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
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
// Import
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_import_step(
    path: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(Shape::import_step(path_str), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_import_stl(
    path: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(Shape::import_stl(path_str), error_out) }
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_step(
    ptr: *mut c_void,
    path: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.export_step(path_str) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_stl(
    ptr: *mut c_void,
    path: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.export_stl(path_str) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_gltf(
    ptr: *mut c_void,
    path: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.export_gltf(path_str, 0.1) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_glb(
    ptr: *mut c_void,
    path: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.export_glb(path_str, 0.1) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_obj(
    ptr: *mut c_void,
    path: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let path_str = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "path is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.export_obj(path_str, 0.1) {
        unsafe { set_err(error_out, &e) };
    }
}

// ---------------------------------------------------------------------------
// Boolean operations
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fuse(
    a: *mut c_void,
    b: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let sa = unsafe { &*(a as *const Shape) };
    let sb = unsafe { &*(b as *const Shape) };
    unsafe { shape_result_to_ptr(sa.fuse(sb), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_cut(
    a: *mut c_void,
    b: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let sa = unsafe { &*(a as *const Shape) };
    let sb = unsafe { &*(b as *const Shape) };
    unsafe { shape_result_to_ptr(sa.cut(sb), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_common(
    a: *mut c_void,
    b: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let sa = unsafe { &*(a as *const Shape) };
    let sb = unsafe { &*(b as *const Shape) };
    unsafe { shape_result_to_ptr(sa.common(sb), error_out) }
}

// ---------------------------------------------------------------------------
// Assembly mating (Phase 5)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_mate(
    ptr: *mut c_void,
    from_ptr: *mut c_void,
    to_ptr: *mut c_void,
    offset: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let from_face = unsafe { &*(from_ptr as *const Shape) };
    let to_face = unsafe { &*(to_ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.mate(from_face, to_face, offset), error_out) }
}

// ---------------------------------------------------------------------------
// Color (Phase 5)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_set_color(
    ptr: *mut c_void,
    r: f64,
    g: f64,
    b: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.set_color(r, g, b), error_out) }
}

// ---------------------------------------------------------------------------
// Transforms (Phase 2 — wiring existing OCCT ops)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_translate(
    ptr: *mut c_void,
    dx: f64,
    dy: f64,
    dz: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.translate(dx, dy, dz), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_rotate(
    ptr: *mut c_void,
    ax: f64,
    ay: f64,
    az: f64,
    angle_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.rotate(ax, ay, az, angle_deg), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_scale(
    ptr: *mut c_void,
    factor: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.scale(factor), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_scale_xyz(
    ptr: *mut c_void,
    sx: f64,
    sy: f64,
    sz: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.scale_xyz(sx, sy, sz), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fillet(
    ptr: *mut c_void,
    radius: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.fillet(radius), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_chamfer(
    ptr: *mut c_void,
    dist: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.chamfer(dist), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fillet_sel(
    ptr: *mut c_void,
    radius: f64,
    selector: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(shape.fillet_sel(radius, sel), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_chamfer_sel(
    ptr: *mut c_void,
    dist: f64,
    selector: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(shape.chamfer_sel(dist, sel), error_out) }
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

// ---------------------------------------------------------------------------
// Mirror (Phase 2)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_mirror(
    ptr: *mut c_void,
    plane: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let plane_str = match unsafe { std::ffi::CStr::from_ptr(plane) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "plane name is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(shape.mirror(plane_str), error_out) }
}

// ---------------------------------------------------------------------------
// 2D sketch constructors (Phase 2)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_rect(
    w: f64,
    h: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_rect(w, h), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_circle_face(
    r: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_circle_face(r), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_polygon(
    pts: *const f64,
    n_pts: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 2) };
    unsafe { shape_result_to_ptr(Shape::make_polygon(slice), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_ellipse_face(
    rx: f64,
    ry: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_ellipse_face(rx, ry), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_arc(
    r: f64,
    start_deg: f64,
    end_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_arc(r, start_deg, end_deg), error_out) }
}

// ---------------------------------------------------------------------------
// Extrude / Revolve (Phase 2)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_extrude(
    ptr: *mut c_void,
    height: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.extrude(height), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_revolve(
    ptr: *mut c_void,
    angle_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.revolve(angle_deg), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 3: Spline profiles and sweep
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_spline_2d(
    pts: *const f64,
    n_pts: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 2) };
    unsafe { shape_result_to_ptr(Shape::make_spline_2d(slice), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_spline_3d(
    pts: *const f64,
    n_pts: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 3) };
    unsafe { shape_result_to_ptr(Shape::make_spline_3d(slice), error_out) }
}

/// Tangent-constrained 2D spline: explicit start/end tangents in the XZ plane.
/// `tangents` points to a flat array [t0x, t0z, t1x, t1z] (4 doubles).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_spline_2d_tan(
    pts: *const f64,
    n_pts: usize,
    tangents: *const f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 2) };
    let t = unsafe { std::slice::from_raw_parts(tangents, 4) };
    unsafe { shape_result_to_ptr(Shape::make_spline_2d_tan(slice, t[0], t[1], t[2], t[3]), error_out) }
}

/// Tangent-constrained 3D spline: explicit start/end tangent vectors.
/// `tangents` points to a flat array [t0x, t0y, t0z, t1x, t1y, t1z] (6 doubles).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_spline_3d_tan(
    pts: *const f64,
    n_pts: usize,
    tangents: *const f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 3) };
    let t = unsafe { std::slice::from_raw_parts(tangents, 6) };
    unsafe {
        shape_result_to_ptr(
            Shape::make_spline_3d_tan(slice, t[0], t[1], t[2], t[3], t[4], t[5]),
            error_out,
        )
    }
}

// ---------------------------------------------------------------------------
// Phase 3: Sub-shape selectors — faces and edges
// ---------------------------------------------------------------------------

/// Returns the count of matching faces, or -1 on error (sets *error_out).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_faces_count(
    ptr: *mut c_void,
    selector: *const c_char,
    error_out: *mut *const c_char,
) -> i32 {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return -1;
        }
    };
    match shape.faces(sel) {
        Ok(v) => v.len() as i32,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            -1
        }
    }
}

/// Returns the idx-th matching face as an owned Shape pointer, or null on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_faces_get(
    ptr: *mut c_void,
    selector: *const c_char,
    idx: i32,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    match shape.faces(sel) {
        Ok(mut v) => {
            let i = idx as usize;
            if i < v.len() {
                Box::into_raw(Box::new(v.swap_remove(i))) as *mut c_void
            } else {
                unsafe { set_err(error_out, "face index out of range") };
                std::ptr::null_mut()
            }
        }
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

/// Returns the count of matching edges, or -1 on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_edges_count(
    ptr: *mut c_void,
    selector: *const c_char,
    error_out: *mut *const c_char,
) -> i32 {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return -1;
        }
    };
    match shape.edges(sel) {
        Ok(v) => v.len() as i32,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            -1
        }
    }
}

/// Returns the idx-th matching edge as an owned Shape pointer, or null on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_edges_get(
    ptr: *mut c_void,
    selector: *const c_char,
    idx: i32,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    match shape.edges(sel) {
        Ok(mut v) => {
            let i = idx as usize;
            if i < v.len() {
                Box::into_raw(Box::new(v.swap_remove(i))) as *mut c_void
            } else {
                unsafe { set_err(error_out, "edge index out of range") };
                std::ptr::null_mut()
            }
        }
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 4: Vertices selector
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_vertices_count(
    ptr: *mut c_void,
    selector: *const c_char,
    error_out: *mut *const c_char,
) -> i32 {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return 0;
        }
    };
    match shape.vertices(sel) {
        Ok(v) => v.len() as i32,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_vertices_get(
    ptr: *mut c_void,
    selector: *const c_char,
    idx: i32,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let sel = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    match shape.vertices(sel) {
        Ok(mut v) => {
            let i = idx as usize;
            if i < v.len() {
                Box::into_raw(Box::new(v.swap_remove(i))) as *mut c_void
            } else {
                unsafe { set_err(error_out, "vertex index out of range") };
                std::ptr::null_mut()
            }
        }
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 4: Query / introspection
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_bounding_box(
    ptr: *mut c_void,
    out: *mut f64, // caller-allocated array of 6 doubles: xmin,ymin,zmin,xmax,ymax,zmax
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.bounding_box() {
        Ok(arr) => unsafe { std::ptr::copy_nonoverlapping(arr.as_ptr(), out, 6) },
        Err(e) => unsafe { set_err(error_out, &e) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_volume(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> f64 {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.volume() {
        Ok(v) => v,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            0.0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_surface_area(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> f64 {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.surface_area() {
        Ok(a) => a,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            0.0
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 3: Live preview
// ---------------------------------------------------------------------------

/// Tessellate `ptr` to binary glTF (GLB) and notify the WebSocket clients.
/// No-op (returns success) when not in `--preview` mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_preview_shape(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };

    let Some(state) = crate::preview::PREVIEW.get() else {
        // Not in --preview mode — silently ignore.
        return;
    };

    let shape = unsafe { &*(ptr as *const Shape) };
    let path = state.glb_path.to_string_lossy();
    if let Err(e) = shape.export_glb(&path, 0.1) {
        unsafe { set_err(error_out, &e) };
        return;
    }

    state.reload_tx.send(()).ok();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_sweep(
    profile: *mut c_void,
    path: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let sp = unsafe { &*(profile as *const Shape) };
    let pa = unsafe { &*(path as *const Shape) };
    unsafe { shape_result_to_ptr(sp.sweep(pa), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 4: 3-D operations — loft, shell, offset, extrude_ex
// ---------------------------------------------------------------------------

/// Loft through N profiles passed as an array of raw Shape pointers.
/// `ruled` is 0 for smooth loft, non-zero for ruled (straight) sections.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_loft(
    ptrs: *const *const c_void,
    n: usize,
    ruled: std::ffi::c_int,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    // Rebuild a slice of &Shape references from the raw pointer array.
    let profiles: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    unsafe { shape_result_to_ptr(Shape::loft(&profiles, ruled != 0), error_out) }
}

/// Hollow out a solid, removing the topmost face and creating walls of
/// the given `thickness`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_shell(
    ptr: *mut c_void,
    thickness: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.shell(thickness), error_out) }
}

/// Inflate (distance>0) or deflate (distance<0) a solid uniformly.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_offset(
    ptr: *mut c_void,
    distance: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.offset(distance), error_out) }
}

/// Remove small features (holes/fillets) by defeaturing.
/// Faces with area < min_feature_size² are passed to BRepAlgoAPI_Defeaturing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_simplify(
    ptr: *mut c_void,
    min_feature_size: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.simplify(min_feature_size), error_out) }
}

/// Extrude with optional end-twist (degrees around Z) and end-scale factor.
/// Falls back to MakePrism when twist≈0 and scale≈1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_extrude_ex(
    ptr: *mut c_void,
    height: f64,
    twist_deg: f64,
    scale: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.extrude_ex(height, twist_deg, scale), error_out) }
}
