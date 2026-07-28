// NOTE: the module-level safety contract in `native.rs` applies to every `extern "C"` function in this file.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occt::Shape;
    use std::{ffi::CStr, os::raw::c_char, ptr};

    fn boxed_shape(shape: Shape) -> *mut c_void {
        Box::into_raw(Box::new(shape)) as *mut c_void
    }

    unsafe fn reclaim_shape(ptr: *mut c_void) {
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr as *mut Shape));
            }
        }
    }

    unsafe fn error_message(err: *const c_char) -> Option<String> {
        if err.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(err) }
                    .to_str()
                    .expect("utf8")
                    .to_string(),
            )
        }
    }

    #[test]
    fn inspect_wrappers_cover_success_and_error_paths() {
        let solid = boxed_shape(Shape::make_box(10.0, 20.0, 30.0).unwrap());
        let top_face = boxed_shape(
            Shape::make_box(10.0, 20.0, 30.0)
                .unwrap()
                .faces("top")
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
        );
        let cyl_face = boxed_shape(
            Shape::make_cylinder(5.0, 10.0)
                .unwrap()
                .faces("all")
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
        );
        let translated = boxed_shape(
            Shape::make_box(10.0, 20.0, 30.0)
                .unwrap()
                .translate(15.0, 0.0, 0.0)
                .unwrap(),
        );
        let mut err: *const c_char = ptr::null();

        let ty = unsafe { rrcad_shape_type_name(solid, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert_eq!(unsafe { CStr::from_ptr(ty) }.to_str().unwrap(), "solid");

        let mut centroid = [0.0; 3];
        unsafe { rrcad_shape_centroid(solid, centroid.as_mut_ptr(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert_eq!(centroid, [5.0, 10.0, 15.0]);

        let mut normal = [0.0; 3];
        unsafe { rrcad_shape_face_normal(top_face, normal.as_mut_ptr(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(normal[2] > 0.0);

        let mut axis = [0.0; 7];
        unsafe { rrcad_shape_cylinder_axis(cyl_face, axis.as_mut_ptr(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!((axis[6] - 5.0).abs() < 1.0e-6);

        assert_eq!(unsafe { rrcad_shape_is_closed(solid, &mut err) }, 1);
        assert!(unsafe { error_message(err) }.is_none());
        assert_eq!(unsafe { rrcad_shape_is_manifold(solid, &mut err) }, 1);
        assert!(unsafe { error_message(err) }.is_none());

        let mut bb = [0.0; 6];
        unsafe { rrcad_shape_bounding_box(solid, bb.as_mut_ptr(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert_eq!(bb, [0.0, 0.0, 0.0, 10.0, 20.0, 30.0]);

        let vol = unsafe { rrcad_shape_volume(solid, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!((vol - 6000.0).abs() < 0.1);

        let area = unsafe { rrcad_shape_surface_area(solid, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!((area - 2200.0).abs() < 1.0);

        let dist = unsafe { rrcad_shape_distance_to(solid, translated, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!((dist - 5.0).abs() < 0.1);

        let mut inertia = [0.0; 6];
        unsafe { rrcad_shape_inertia(solid, inertia.as_mut_ptr(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(inertia.iter().all(|v| *v >= 0.0));

        let thickness = unsafe { rrcad_shape_min_thickness(solid, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(thickness > 0.0);

        let validate = unsafe { rrcad_shape_validate(solid, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert_eq!(unsafe { CStr::from_ptr(validate) }.to_str().unwrap(), "ok");

        let history = unsafe { rrcad_shape_history(translated, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(
            unsafe { CStr::from_ptr(history) }
                .to_str()
                .unwrap()
                .contains("translate")
        );

        let graph = unsafe { rrcad_shape_feature_graph(translated, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(
            unsafe { CStr::from_ptr(graph) }
                .to_str()
                .unwrap()
                .contains("translate")
        );

        let rebuilt = unsafe { rrcad_shape_rebuild(translated, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!rebuilt.is_null());

        let mut bad_err: *const c_char = ptr::null();
        let mut out = [0.0; 3];
        unsafe { rrcad_shape_face_normal(solid, out.as_mut_ptr(), &mut bad_err) };
        assert!(unsafe { error_message(bad_err) }.is_some());

        let mut bad_err: *const c_char = ptr::null();
        unsafe { rrcad_shape_cylinder_axis(solid, axis.as_mut_ptr(), &mut bad_err) };
        assert!(unsafe { error_message(bad_err) }.is_some());

        unsafe {
            reclaim_shape(solid);
            reclaim_shape(top_face);
            reclaim_shape(cyl_face);
            reclaim_shape(translated);
            reclaim_shape(rebuilt);
        }
    }
}
