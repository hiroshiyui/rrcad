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

#[cfg(test)]
mod tests {
    use super::{
        rrcad_import_step, rrcad_import_stl, rrcad_shape_export_dxf, rrcad_shape_export_step,
        rrcad_shape_export_svg,
    };
    use crate::occt::Shape;
    use std::{
        ffi::{CStr, CString},
        fs,
        os::raw::c_char,
        path::PathBuf,
        ptr,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()))
    }

    fn with_cwd<T>(dir: &PathBuf, f: impl FnOnce() -> T) -> T {
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(dir).expect("enter temp dir");
        let result = f();
        std::env::set_current_dir(original).expect("restore cwd");
        result
    }

    fn cstr(value: &str) -> CString {
        CString::new(value).expect("CString")
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

    unsafe fn reclaim_shape(ptr: *mut std::ffi::c_void) {
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr as *mut Shape));
            }
        }
    }

    #[test]
    fn export_step_creates_file_and_import_step_round_trips() {
        let dir = unique_test_dir("rrcad-native-io-step");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            let shape = Box::into_raw(Box::new(Shape::make_box(10.0, 20.0, 30.0).unwrap()))
                as *mut std::ffi::c_void;
            let mut err: *const c_char = ptr::null();

            unsafe {
                rrcad_shape_export_step(shape, cstr("part.step").as_ptr(), &mut err);
            }
            assert!(
                unsafe { error_message(err) }.is_none(),
                "unexpected export error"
            );
            assert!(dir.join("part.step").exists(), "STEP file was not created");

            let mut import_err: *const c_char = ptr::null();
            let imported =
                unsafe { rrcad_import_step(cstr("part.step").as_ptr(), &mut import_err) };
            assert!(
                !imported.is_null(),
                "importing the exported STEP file should succeed"
            );
            assert!(unsafe { error_message(import_err) }.is_none());

            unsafe {
                reclaim_shape(shape);
                reclaim_shape(imported);
            }
        });
    }

    #[test]
    fn export_svg_and_dxf_create_documents() {
        let dir = unique_test_dir("rrcad-native-io-draw");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            let shape = Box::into_raw(Box::new(Shape::make_box(10.0, 20.0, 30.0).unwrap()))
                as *mut std::ffi::c_void;
            let mut err: *const c_char = ptr::null();

            unsafe {
                rrcad_shape_export_svg(
                    shape,
                    cstr("part.svg").as_ptr(),
                    cstr("top").as_ptr(),
                    1.0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    cstr("").as_ptr(),
                    0,
                    0.0,
                    0.0,
                    0.0,
                    cstr("").as_ptr(),
                    0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    &mut err,
                );
            }
            assert!(
                unsafe { error_message(err) }.is_none(),
                "unexpected SVG error"
            );
            let svg = fs::read_to_string(dir.join("part.svg")).expect("read SVG");
            assert!(
                svg.contains("<svg"),
                "SVG output is missing the root element"
            );

            let mut err: *const c_char = ptr::null();
            unsafe {
                rrcad_shape_export_dxf(
                    shape,
                    cstr("part.dxf").as_ptr(),
                    cstr("top").as_ptr(),
                    1.0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    cstr("").as_ptr(),
                    0,
                    0.0,
                    0.0,
                    0.0,
                    cstr("").as_ptr(),
                    0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    &mut err,
                );
            }
            assert!(
                unsafe { error_message(err) }.is_none(),
                "unexpected DXF error"
            );
            let dxf = fs::read_to_string(dir.join("part.dxf")).expect("read DXF");
            assert!(
                dxf.contains("SECTION"),
                "DXF output is missing section markers"
            );

            unsafe { reclaim_shape(shape) };
        });
    }

    #[test]
    fn import_missing_step_and_stl_report_errors() {
        let dir = unique_test_dir("rrcad-native-io-missing");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            let mut err: *const c_char = ptr::null();
            let step = unsafe { rrcad_import_step(cstr("missing.step").as_ptr(), &mut err) };
            assert!(step.is_null(), "missing STEP file should not import");
            let step_err = unsafe { error_message(err) }.expect("missing STEP error");
            assert!(step_err.contains("missing.step") || step_err.contains("not found"));

            let mut err: *const c_char = ptr::null();
            let stl = unsafe { rrcad_import_stl(cstr("missing.stl").as_ptr(), &mut err) };
            assert!(stl.is_null(), "missing STL file should not import");
            let stl_err = unsafe { error_message(err) }.expect("missing STL error");
            assert!(stl_err.contains("missing.stl") || stl_err.contains("not found"));
        });
    }
}
