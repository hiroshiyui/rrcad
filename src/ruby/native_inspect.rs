use std::ffi::{c_char, c_void};

use crate::occt::Shape;

use super::native_helpers::{set_err, set_str, shape_result_to_ptr};

/// Returns a C string pointer to the shape type name (e.g. "solid", "shell").
/// The pointer is valid until the next call on this thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_type_name(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *const c_char {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.shape_type_name() {
        Ok(s) => {
            let mut raw: *const c_char = std::ptr::null();
            unsafe {
                set_str(&mut raw as *mut *const c_char, &s);
            }
            raw
        }
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null()
        }
    }
}

/// Fill `out[0..3]` with the centroid (x, y, z) of the shape.
/// `out` must point to a caller-allocated array of at least 3 doubles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_centroid(
    ptr: *mut c_void,
    out: *mut f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.centroid() {
        Ok(arr) => unsafe { std::ptr::copy_nonoverlapping(arr.as_ptr(), out, 3) },
        Err(e) => unsafe { set_err(error_out, &e) },
    }
}

/// Fill `out[0..3]` with the outward unit normal of a face shape.
/// `out` must point to a caller-allocated array of at least 3 doubles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_face_normal(
    ptr: *mut c_void,
    out: *mut f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.face_normal() {
        Ok(arr) => unsafe { std::ptr::copy_nonoverlapping(arr.as_ptr(), out, 3) },
        Err(e) => unsafe { set_err(error_out, &e) },
    }
}

/// Fill `out[0..7]` with `[ox, oy, oz, ax, ay, az, radius]` for a
/// cylindrical face. `out` must point to a caller-allocated array of at
/// least 7 doubles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_cylinder_axis(
    ptr: *mut c_void,
    out: *mut f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.cylinder_axis() {
        Ok(arr) => unsafe { std::ptr::copy_nonoverlapping(arr.as_ptr(), out, 7) },
        Err(e) => unsafe { set_err(error_out, &e) },
    }
}

/// Returns 1 if the shape is closed (every edge shared by ≥2 faces), 0 otherwise.
/// Returns -1 and sets *error_out on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_is_closed(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> std::ffi::c_int {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.is_closed() {
        Ok(b) => b as std::ffi::c_int,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            -1
        }
    }
}

/// Returns 1 if the shape is manifold (every edge shared by exactly 2 faces), 0 otherwise.
/// Returns -1 and sets *error_out on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_is_manifold(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> std::ffi::c_int {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.is_manifold() {
        Ok(b) => b as std::ffi::c_int,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_bounding_box(
    ptr: *mut c_void,
    out: *mut f64,
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_distance_to(
    a_ptr: *mut c_void,
    b_ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> f64 {
    unsafe { *error_out = std::ptr::null() };
    let a = unsafe { &*(a_ptr as *const Shape) };
    let b = unsafe { &*(b_ptr as *const Shape) };
    match a.distance_to(b) {
        Ok(d) => d,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            f64::NAN
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_inertia(
    ptr: *mut c_void,
    out_ptr: *mut f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.inertia() {
        Ok(arr) => {
            for (i, &v) in arr.iter().enumerate() {
                unsafe { *out_ptr.add(i) = v };
            }
        }
        Err(e) => unsafe { set_err(error_out, &e) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_min_thickness(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> f64 {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.min_thickness() {
        Ok(t) => t,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            f64::NAN
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_validate(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *const c_char {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.validate() {
        Ok(s) => {
            let mut raw: *const c_char = std::ptr::null();
            unsafe {
                set_str(&mut raw as *mut *const c_char, &s);
            }
            raw
        }
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_history(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *const c_char {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let joined = shape.history().join("\n");
    let mut raw: *const c_char = std::ptr::null();
    unsafe {
        set_str(&mut raw as *mut *const c_char, &joined);
    }
    raw
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_feature_graph(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *const c_char {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let joined = shape.feature_graph();
    let mut raw: *const c_char = std::ptr::null();
    unsafe {
        set_str(&mut raw as *mut *const c_char, &joined);
    }
    raw
}

/// Rebuild the shape by replaying its feature graph from the stored parents.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_rebuild(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.rebuild(), error_out) }
}
