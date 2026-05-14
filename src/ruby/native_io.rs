use std::ffi::{c_char, c_void};

use super::native_helpers::{
    DEFAULT_LINEAR_DEFLECTION, cstr_arg, resolve_path, set_err, shape_result_to_ptr, split_csv_list,
};
use crate::occt::{GdtDatumSpec, GdtFeatureControlSpec, GdtRenderSpec, GdtStandard, Shape};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_import_step(
    path: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return std::ptr::null_mut();
    };
    let safe_str = safe.to_string_lossy();
    unsafe { shape_result_to_ptr(Shape::import_step(&safe_str), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_import_stl(
    path: *const c_char,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return std::ptr::null_mut();
    };
    let safe_str = safe.to_string_lossy();
    unsafe { shape_result_to_ptr(Shape::import_stl(&safe_str), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_step(
    ptr: *mut c_void,
    path: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    if let Err(e) = shape.export_step(&safe_str) {
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
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    if let Err(e) = shape.export_stl(&safe_str) {
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
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    if let Err(e) = shape.export_gltf(&safe_str, DEFAULT_LINEAR_DEFLECTION) {
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
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    if let Err(e) = shape.export_glb(&safe_str, DEFAULT_LINEAR_DEFLECTION) {
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
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    if let Err(e) = shape.export_obj(&safe_str, DEFAULT_LINEAR_DEFLECTION) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_gdt_apply(
    ptr: *mut c_void,
    standard: *const c_char,
    datum_label: *const c_char,
    datum_selector: *const c_char,
    datum_anchor_valid: i32,
    datum_anchor_x: f64,
    datum_anchor_y: f64,
    datum_anchor_z: f64,
    feature_control_text: *const c_char,
    feature_control_selector: *const c_char,
    feature_control_anchor_valid: i32,
    feature_control_anchor_x: f64,
    feature_control_anchor_y: f64,
    feature_control_anchor_z: f64,
    feature_control_datums: *const c_char,
    feature_control_modifiers: *const c_char,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };

    let standard = match unsafe { cstr_arg(standard) } {
        Ok(s) => s,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            return;
        }
    };
    let standard = match GdtStandard::parse(&standard) {
        Ok(s) => s,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            return;
        }
    };

    let datum = if datum_label.is_null() {
        None
    } else {
        let label = match unsafe { cstr_arg(datum_label) } {
            Ok(s) => s,
            Err(e) => {
                unsafe { set_err(error_out, &e) };
                return;
            }
        };
        let selector = if datum_selector.is_null() {
            None
        } else {
            match unsafe { cstr_arg(datum_selector) } {
                Ok(s) if !s.is_empty() => Some(s),
                Ok(_) => None,
                Err(e) => {
                    unsafe { set_err(error_out, &e) };
                    return;
                }
            }
        };
        let anchor = if datum_anchor_valid != 0 {
            Some([datum_anchor_x, datum_anchor_y, datum_anchor_z])
        } else {
            None
        };
        Some(GdtDatumSpec {
            label,
            selector,
            anchor,
        })
    };

    let feature_control = if feature_control_text.is_null() {
        None
    } else {
        let text = match unsafe { cstr_arg(feature_control_text) } {
            Ok(s) => s,
            Err(e) => {
                unsafe { set_err(error_out, &e) };
                return;
            }
        };
        let selector = if feature_control_selector.is_null() {
            None
        } else {
            match unsafe { cstr_arg(feature_control_selector) } {
                Ok(s) if !s.is_empty() => Some(s),
                Ok(_) => None,
                Err(e) => {
                    unsafe { set_err(error_out, &e) };
                    return;
                }
            }
        };
        let anchor = if feature_control_anchor_valid != 0 {
            Some([
                feature_control_anchor_x,
                feature_control_anchor_y,
                feature_control_anchor_z,
            ])
        } else {
            None
        };
        let datums = if feature_control_datums.is_null() {
            Vec::new()
        } else {
            match unsafe { cstr_arg(feature_control_datums) } {
                Ok(s) => split_csv_list(&s),
                Err(e) => {
                    unsafe { set_err(error_out, &e) };
                    return;
                }
            }
        };
        let modifiers = if feature_control_modifiers.is_null() {
            Vec::new()
        } else {
            match unsafe { cstr_arg(feature_control_modifiers) } {
                Ok(s) => split_csv_list(&s),
                Err(e) => {
                    unsafe { set_err(error_out, &e) };
                    return;
                }
            }
        };
        Some(GdtFeatureControlSpec {
            text,
            selector,
            anchor,
            datums,
            modifiers,
        })
    };

    shape.gdt_apply(GdtRenderSpec {
        standard,
        datum,
        feature_control,
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_svg(
    ptr: *mut c_void,
    path: *const c_char,
    view: *const c_char,
    scale: f64,
    hidden: i32,
    center_marks: i32,
    dimensions: i32,
    title_block: i32,
    callouts: i32,
    datum: *const c_char,
    datum_anchor_valid: i32,
    datum_anchor_x: f64,
    datum_anchor_y: f64,
    datum_anchor_z: f64,
    feature_control: *const c_char,
    feature_control_anchor_valid: i32,
    feature_control_anchor_x: f64,
    feature_control_anchor_y: f64,
    feature_control_anchor_z: f64,
    tolerance_plus: f64,
    tolerance_minus: f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    let view_str = unsafe { std::ffi::CStr::from_ptr(view) }
        .to_str()
        .unwrap_or("top");
    if let Err(e) = shape.export_svg_with_anchor(
        &safe_str,
        view_str,
        scale,
        hidden != 0,
        center_marks != 0,
        dimensions != 0,
        title_block != 0,
        callouts != 0,
        unsafe { std::ffi::CStr::from_ptr(datum) }
            .to_str()
            .unwrap_or(""),
        datum_anchor_valid != 0,
        datum_anchor_x,
        datum_anchor_y,
        datum_anchor_z,
        unsafe { std::ffi::CStr::from_ptr(feature_control) }
            .to_str()
            .unwrap_or(""),
        feature_control_anchor_valid != 0,
        feature_control_anchor_x,
        feature_control_anchor_y,
        feature_control_anchor_z,
        tolerance_plus,
        tolerance_minus,
    ) {
        unsafe { set_err(error_out, &e) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_export_dxf(
    ptr: *mut c_void,
    path: *const c_char,
    view: *const c_char,
    scale: f64,
    hidden: i32,
    center_marks: i32,
    dimensions: i32,
    title_block: i32,
    callouts: i32,
    datum: *const c_char,
    datum_anchor_valid: i32,
    datum_anchor_x: f64,
    datum_anchor_y: f64,
    datum_anchor_z: f64,
    feature_control: *const c_char,
    feature_control_anchor_valid: i32,
    feature_control_anchor_x: f64,
    feature_control_anchor_y: f64,
    feature_control_anchor_z: f64,
    tolerance_plus: f64,
    tolerance_minus: f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
        return;
    };
    let safe_str = safe.to_string_lossy();
    let view_str = unsafe { std::ffi::CStr::from_ptr(view) }
        .to_str()
        .unwrap_or("top");
    if let Err(e) = shape.export_dxf_with_anchor(
        &safe_str,
        view_str,
        scale,
        hidden != 0,
        center_marks != 0,
        dimensions != 0,
        title_block != 0,
        callouts != 0,
        unsafe { std::ffi::CStr::from_ptr(datum) }
            .to_str()
            .unwrap_or(""),
        datum_anchor_valid != 0,
        datum_anchor_x,
        datum_anchor_y,
        datum_anchor_z,
        unsafe { std::ffi::CStr::from_ptr(feature_control) }
            .to_str()
            .unwrap_or(""),
        feature_control_anchor_valid != 0,
        feature_control_anchor_x,
        feature_control_anchor_y,
        feature_control_anchor_z,
        tolerance_plus,
        tolerance_minus,
    ) {
        unsafe { set_err(error_out, &e) };
    }
}
