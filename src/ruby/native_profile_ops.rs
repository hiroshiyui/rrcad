use std::ffi::{c_char, c_void};

use crate::occt::Shape;

use super::native_helpers::{set_err, shape_result_to_ptr};

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
    unsafe {
        shape_result_to_ptr(
            Shape::make_spline_2d_tan(slice, t[0], t[1], t[2], t[3]),
            error_out,
        )
    }
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
