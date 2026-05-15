use std::ffi::{c_char, c_void};

use crate::occt::Shape;

use super::native_helpers::{set_err, shape_result_to_ptr};

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_name_face(
    ptr: *mut c_void,
    name: *const c_char,
    selector: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let name = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "name is not valid UTF-8") };
            return;
        }
    };
    let selector = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.name_face(name, selector) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_name_edge(
    ptr: *mut c_void,
    name: *const c_char,
    selector: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let name = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "name is not valid UTF-8") };
            return;
        }
    };
    let selector = match unsafe { std::ffi::CStr::from_ptr(selector) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "selector is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.name_edge(name, selector) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_datum(
    ptr: *mut c_void,
    name: *const c_char,
    datum_ptr: *mut c_void,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let datum = unsafe { &*(datum_ptr as *const Shape) };
    let name = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "name is not valid UTF-8") };
            return;
        }
    };
    if let Err(e) = shape.datum(name, datum) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_ref(
    ptr: *mut c_void,
    name: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let name = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "name is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(shape.ref_named(name), error_out) }
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fillet_var(
    ptr: *mut c_void,
    r1: f64,
    r2: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.fillet_var(r1, r2), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fillet_var_sel(
    ptr: *mut c_void,
    r1: f64,
    r2: f64,
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
    unsafe { shape_result_to_ptr(shape.fillet_var_sel(r1, r2, sel), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 7 Tier 1: asymmetric chamfer
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_chamfer_asym(
    ptr: *mut c_void,
    d1: f64,
    d2: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.chamfer_asym(d1, d2), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_chamfer_asym_sel(
    ptr: *mut c_void,
    d1: f64,
    d2: f64,
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
    unsafe { shape_result_to_ptr(shape.chamfer_asym_sel(d1, d2, sel), error_out) }
}

#[cfg(test)]
mod tests {
    use super::{
        rrcad_shape_chamfer, rrcad_shape_chamfer_asym, rrcad_shape_chamfer_asym_sel,
        rrcad_shape_chamfer_sel, rrcad_shape_common, rrcad_shape_cut, rrcad_shape_datum,
        rrcad_shape_fillet, rrcad_shape_fillet_sel, rrcad_shape_fillet_var,
        rrcad_shape_fillet_var_sel, rrcad_shape_fuse, rrcad_shape_mate, rrcad_shape_name_edge,
        rrcad_shape_name_face, rrcad_shape_ref, rrcad_shape_rotate, rrcad_shape_scale,
        rrcad_shape_scale_xyz, rrcad_shape_translate,
    };
    use crate::occt::Shape;
    use std::{
        ffi::{CStr, CString},
        os::raw::c_char,
        ptr,
    };

    unsafe fn reclaim_shape(ptr: *mut std::ffi::c_void) {
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr as *mut Shape));
            }
        }
    }

    fn boxed_shape(shape: Shape) -> *mut std::ffi::c_void {
        Box::into_raw(Box::new(shape)) as *mut std::ffi::c_void
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
    fn boolean_and_transform_wrappers_return_shapes() {
        let a = boxed_shape(Shape::make_box(10.0, 10.0, 10.0).unwrap());
        let b = boxed_shape(
            Shape::make_box(10.0, 10.0, 10.0)
                .unwrap()
                .translate(5.0, 0.0, 0.0)
                .unwrap(),
        );
        let mut err: *const c_char = ptr::null();

        let fused = unsafe { rrcad_shape_fuse(a, b, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!fused.is_null());

        let cut = unsafe { rrcad_shape_cut(a, b, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!cut.is_null());

        let common = unsafe { rrcad_shape_common(a, b, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!common.is_null());

        let translated = unsafe { rrcad_shape_translate(a, 1.0, 2.0, 3.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!translated.is_null());

        let rotated = unsafe { rrcad_shape_rotate(a, 0.0, 0.0, 1.0, 45.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!rotated.is_null());

        let scaled = unsafe { rrcad_shape_scale(a, 2.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!scaled.is_null());

        let scaled_xyz = unsafe { rrcad_shape_scale_xyz(a, 2.0, 3.0, 4.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!scaled_xyz.is_null());

        let filleted = unsafe { rrcad_shape_fillet(a, 1.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!filleted.is_null());

        let chamfered = unsafe { rrcad_shape_chamfer(a, 1.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!chamfered.is_null());

        let fillet_var = unsafe { rrcad_shape_fillet_var(a, 0.5, 2.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!fillet_var.is_null());

        let chamfer_asym = unsafe { rrcad_shape_chamfer_asym(a, 1.0, 2.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!chamfer_asym.is_null());

        unsafe {
            reclaim_shape(a);
            reclaim_shape(b);
            reclaim_shape(fused);
            reclaim_shape(cut);
            reclaim_shape(common);
            reclaim_shape(translated);
            reclaim_shape(rotated);
            reclaim_shape(scaled);
            reclaim_shape(scaled_xyz);
            reclaim_shape(filleted);
            reclaim_shape(chamfered);
            reclaim_shape(fillet_var);
            reclaim_shape(chamfer_asym);
        }
    }

    #[test]
    fn selector_and_reference_wrappers_accept_valid_names() {
        let shape = boxed_shape(Shape::make_box(10.0, 10.0, 10.0).unwrap());
        let datum = boxed_shape(Shape::make_box(1.0, 1.0, 1.0).unwrap());
        let mut err: *const c_char = ptr::null();

        unsafe {
            rrcad_shape_name_face(
                shape,
                CString::new("mounting_face").unwrap().as_ptr(),
                CString::new("top").unwrap().as_ptr(),
                &mut err,
            );
        }
        assert!(unsafe { error_message(err) }.is_none());

        unsafe {
            rrcad_shape_name_edge(
                shape,
                CString::new("vertical_edges").unwrap().as_ptr(),
                CString::new("vertical").unwrap().as_ptr(),
                &mut err,
            );
        }
        assert!(unsafe { error_message(err) }.is_none());

        unsafe {
            rrcad_shape_datum(
                shape,
                CString::new("origin_datum").unwrap().as_ptr(),
                datum,
                &mut err,
            );
        }
        assert!(unsafe { error_message(err) }.is_none());

        let named_face = unsafe {
            rrcad_shape_ref(
                shape,
                CString::new("mounting_face").unwrap().as_ptr(),
                &mut err,
            )
        };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!named_face.is_null());

        let named_edge = unsafe {
            rrcad_shape_ref(
                shape,
                CString::new("vertical_edges").unwrap().as_ptr(),
                &mut err,
            )
        };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!named_edge.is_null());

        let mate = unsafe { rrcad_shape_mate(shape, named_face, named_face, 0.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!mate.is_null());

        unsafe {
            reclaim_shape(shape);
            reclaim_shape(datum);
            reclaim_shape(named_face);
            reclaim_shape(named_edge);
            reclaim_shape(mate);
        }
    }

    #[test]
    fn wrappers_reject_invalid_utf8_for_names_and_selectors() {
        let shape = boxed_shape(Shape::make_box(10.0, 10.0, 10.0).unwrap());
        let invalid = unsafe { CString::from_vec_unchecked(vec![0xff, 0xfe]) };
        let valid = CString::new("top").unwrap();
        let mut err: *const c_char = ptr::null();

        unsafe {
            rrcad_shape_name_face(shape, invalid.as_ptr(), valid.as_ptr(), &mut err);
        }
        assert!(unsafe { error_message(err) }.is_some());

        let invalid_sel = unsafe { CString::from_vec_unchecked(vec![0xff, 0xfe]) };
        let mut err: *const c_char = ptr::null();
        unsafe {
            rrcad_shape_fillet_sel(shape, 1.0, invalid_sel.as_ptr(), &mut err);
        }
        assert!(unsafe { error_message(err) }.is_some());

        let mut err: *const c_char = ptr::null();
        unsafe {
            rrcad_shape_chamfer_sel(shape, 1.0, invalid_sel.as_ptr(), &mut err);
        }
        assert!(unsafe { error_message(err) }.is_some());

        let mut err: *const c_char = ptr::null();
        unsafe {
            rrcad_shape_fillet_var_sel(shape, 0.5, 2.0, invalid_sel.as_ptr(), &mut err);
        }
        assert!(unsafe { error_message(err) }.is_some());

        let mut err: *const c_char = ptr::null();
        unsafe {
            rrcad_shape_chamfer_asym_sel(shape, 1.0, 2.0, invalid_sel.as_ptr(), &mut err);
        }
        assert!(unsafe { error_message(err) }.is_some());

        unsafe { reclaim_shape(shape) };
    }
}
