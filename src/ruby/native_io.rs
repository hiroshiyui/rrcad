// NOTE: the module-level safety contract in `native.rs` applies to every `extern "C"` function in this file.

use std::ffi::{c_char, c_void};

use super::native_helpers::{
    DEFAULT_LINEAR_DEFLECTION, cstr_arg, resolve_path, set_err, shape_result_to_ptr, split_csv_list,
};
use crate::occt::{GdtDatumSpec, GdtFeatureControlSpec, GdtRenderSpec, GdtStandard, Shape};

// ---------------------------------------------------------------------------
// Single-file import / export
//
// Every one of these entry points validates the caller's path with
// `resolve_path` (the path-traversal guard) and then forwards to one `Shape`
// method, so both families are generated from a one-line declaration.
// ---------------------------------------------------------------------------

/// Wrap a `Shape` associated function that loads a file from a validated path.
macro_rules! shape_import {
    ($name:ident => $ctor:path) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            path: *const c_char,
            error_out: *mut *const c_char,
        ) -> *mut c_void {
            unsafe { *error_out = std::ptr::null() };
            let Some(safe) = (unsafe { resolve_path(path, error_out) }) else {
                return std::ptr::null_mut();
            };
            let safe_str = safe.to_string_lossy();
            unsafe { shape_result_to_ptr($ctor(&safe_str), error_out) }
        }
    };
}

/// Wrap a `&Shape` method that writes the shape to a validated path.
///
/// The tessellating exporters (glTF / GLB / OBJ) take one extra argument, the
/// mesh deflection, which is supplied as the optional trailing expression.
macro_rules! shape_export {
    ($name:ident => $method:ident $(, $extra:expr)?) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
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
            if let Err(e) = shape.$method(&safe_str $(, $extra)?) {
                unsafe { set_err(error_out, &e) };
            }
        }
    };
}

shape_import!(rrcad_import_step => Shape::import_step);
shape_import!(rrcad_import_stl => Shape::import_stl);

shape_export!(rrcad_shape_export_step => export_step);
shape_export!(rrcad_shape_export_stl => export_stl);
shape_export!(rrcad_shape_export_gltf => export_gltf, DEFAULT_LINEAR_DEFLECTION);
shape_export!(rrcad_shape_export_glb => export_glb, DEFAULT_LINEAR_DEFLECTION);
shape_export!(rrcad_shape_export_obj => export_obj, DEFAULT_LINEAR_DEFLECTION);

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

/// Which 2D drawing format a `DrawingExportOpts` should be rendered to.
enum DrawingFormat {
    Svg,
    Dxf,
}

/// Owned, parsed arguments shared by the SVG and DXF drawing exports.
/// Both extern "C" entry points take the same 23 raw parameters; this struct
/// holds the validated Rust-side equivalents so the parsing lives in one place.
struct DrawingExportOpts {
    path: String,
    view: String,
    scale: f64,
    hidden: bool,
    center_marks: bool,
    dimensions: bool,
    title_block: bool,
    callouts: bool,
    datum: String,
    datum_anchor_valid: bool,
    datum_anchor: [f64; 3],
    feature_control: String,
    feature_control_anchor_valid: bool,
    feature_control_anchor: [f64; 3],
    tolerance_plus: f64,
    tolerance_minus: f64,
    /// Section plane name: empty means "no section", otherwise "xy"/"xz"/"yz".
    section_plane: String,
    /// Offset of the section plane along its own normal.
    section_offset: f64,
}

impl DrawingExportOpts {
    /// Forward the parsed options to the matching `Shape` export method.
    fn export(&self, shape: &Shape, format: DrawingFormat) -> Result<(), String> {
        // Both methods share the exact same parameter list; only the target differs.
        let f = match format {
            DrawingFormat::Svg => Shape::export_svg_with_anchor,
            DrawingFormat::Dxf => Shape::export_dxf_with_anchor,
        };
        f(
            shape,
            &self.path,
            &self.view,
            self.scale,
            self.hidden,
            self.center_marks,
            self.dimensions,
            self.title_block,
            self.callouts,
            &self.datum,
            self.datum_anchor_valid,
            self.datum_anchor[0],
            self.datum_anchor[1],
            self.datum_anchor[2],
            &self.feature_control,
            self.feature_control_anchor_valid,
            self.feature_control_anchor[0],
            self.feature_control_anchor[1],
            self.feature_control_anchor[2],
            self.tolerance_plus,
            self.tolerance_minus,
            &self.section_plane,
            self.section_offset,
        )
    }
}

/// Parse the raw C parameters common to `rrcad_shape_export_svg` and
/// `rrcad_shape_export_dxf`. Returns `None` (with `error_out` set) if the
/// output path fails validation.
///
/// # Safety
/// All pointer arguments must satisfy the same invariants as the extern "C"
/// exporters that forward to this helper (valid NUL-terminated strings,
/// writable `error_out`).
#[allow(clippy::too_many_arguments)] // mirrors the flat scalar parameter list of the extern "C" exporters
unsafe fn parse_drawing_export_opts(
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
    section_plane: *const c_char,
    section_offset: f64,
    error_out: *mut *const c_char,
) -> Option<DrawingExportOpts> {
    let safe = unsafe { resolve_path(path, error_out) }?;
    let path = safe.to_string_lossy().into_owned();
    let view = unsafe { std::ffi::CStr::from_ptr(view) }
        .to_str()
        .unwrap_or("top")
        .to_owned();
    let datum = unsafe { std::ffi::CStr::from_ptr(datum) }
        .to_str()
        .unwrap_or("")
        .to_owned();
    let feature_control = unsafe { std::ffi::CStr::from_ptr(feature_control) }
        .to_str()
        .unwrap_or("")
        .to_owned();
    // A null or empty `section_plane` means the caller asked for no section.
    let section_plane = if section_plane.is_null() {
        String::new()
    } else {
        unsafe { std::ffi::CStr::from_ptr(section_plane) }
            .to_str()
            .unwrap_or("")
            .to_owned()
    };
    Some(DrawingExportOpts {
        path,
        view,
        scale,
        hidden: hidden != 0,
        center_marks: center_marks != 0,
        dimensions: dimensions != 0,
        title_block: title_block != 0,
        callouts: callouts != 0,
        datum,
        datum_anchor_valid: datum_anchor_valid != 0,
        datum_anchor: [datum_anchor_x, datum_anchor_y, datum_anchor_z],
        feature_control,
        feature_control_anchor_valid: feature_control_anchor_valid != 0,
        feature_control_anchor: [
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
        ],
        tolerance_plus,
        tolerance_minus,
        section_plane,
        section_offset,
    })
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
    section_plane: *const c_char,
    section_offset: f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let Some(opts) = (unsafe {
        parse_drawing_export_opts(
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            datum,
            datum_anchor_valid,
            datum_anchor_x,
            datum_anchor_y,
            datum_anchor_z,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
            tolerance_plus,
            tolerance_minus,
            section_plane,
            section_offset,
            error_out,
        )
    }) else {
        return;
    };
    if let Err(e) = opts.export(shape, DrawingFormat::Svg) {
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
    section_plane: *const c_char,
    section_offset: f64,
    error_out: *mut *const c_char,
) {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let Some(opts) = (unsafe {
        parse_drawing_export_opts(
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            datum,
            datum_anchor_valid,
            datum_anchor_x,
            datum_anchor_y,
            datum_anchor_z,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
            tolerance_plus,
            tolerance_minus,
            section_plane,
            section_offset,
            error_out,
        )
    }) else {
        return;
    };
    if let Err(e) = opts.export(shape, DrawingFormat::Dxf) {
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
    use crate::test_util::unique_test_dir;
    use std::{
        ffi::{CStr, CString},
        fs,
        os::raw::c_char,
        path::PathBuf,
        ptr,
    };

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
                    cstr("").as_ptr(),
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
                    cstr("").as_ptr(),
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

    /// Run the SVG and DXF exporters with a section plane, returning the raw
    /// error string (if any) plus the file contents produced.
    #[allow(clippy::type_complexity)] // one-off tuple, only used by the two section tests
    fn export_section(
        dir: &std::path::Path,
        plane: &str,
        offset: f64,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let shape = Box::into_raw(Box::new(Shape::make_box(10.0, 20.0, 30.0).unwrap()))
            as *mut std::ffi::c_void;

        let mut svg_err: *const c_char = ptr::null();
        let mut dxf_err: *const c_char = ptr::null();
        unsafe {
            rrcad_shape_export_svg(
                shape,
                cstr("section.svg").as_ptr(),
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
                cstr(plane).as_ptr(),
                offset,
                &mut svg_err,
            );
        }
        // `set_err` keeps a single thread-local buffer, so the SVG message has
        // to be copied out before the DXF call can overwrite it.
        let svg_message = unsafe { error_message(svg_err) };
        unsafe {
            rrcad_shape_export_dxf(
                shape,
                cstr("section.dxf").as_ptr(),
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
                cstr(plane).as_ptr(),
                offset,
                &mut dxf_err,
            );
        }
        let dxf_message = unsafe { error_message(dxf_err) };
        unsafe { reclaim_shape(shape) };
        (
            svg_message,
            dxf_message,
            fs::read_to_string(dir.join("section.svg")).ok(),
            fs::read_to_string(dir.join("section.dxf")).ok(),
        )
    }

    #[test]
    fn section_view_emits_hatched_cross_section() {
        let dir = unique_test_dir("rrcad-native-io-section");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            // Cut the 10x20x30 box halfway up Z; the cross-section is the
            // full 10x20 rectangle, so hatching must be produced.
            let (svg_err, dxf_err, svg, dxf) = export_section(&dir, "xy", 15.0);
            assert!(
                svg_err.is_none(),
                "unexpected SVG section error: {svg_err:?}"
            );
            assert!(
                dxf_err.is_none(),
                "unexpected DXF section error: {dxf_err:?}"
            );

            let svg = svg.expect("section SVG was not written");
            assert!(svg.contains("hatch\""), "SVG is missing the hatch group");
            assert!(
                svg.contains("<line"),
                "SVG hatch group has no line elements"
            );
            assert!(
                svg.contains("section\""),
                "SVG is missing the cut outline group"
            );

            let dxf = dxf.expect("section DXF was not written");
            assert!(dxf.contains("HATCH"), "DXF is missing the HATCH layer");
        });
    }

    #[test]
    fn section_plane_that_misses_the_solid_reports_an_error() {
        let dir = unique_test_dir("rrcad-native-io-section-miss");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            // The box spans z = 0..30, so a plane at z = 500 misses it entirely.
            let (svg_err, dxf_err, _, _) = export_section(&dir, "xy", 500.0);
            let svg_err = svg_err.expect("plane missing the solid should error");
            assert!(
                svg_err.contains("does not intersect"),
                "expected a 'does not intersect' SVG error, got: {svg_err}"
            );
            let dxf_err = dxf_err.expect("plane missing the solid should error");
            assert!(
                dxf_err.contains("does not intersect"),
                "expected a 'does not intersect' DXF error, got: {dxf_err}"
            );
        });
    }

    #[test]
    fn section_without_an_offset_cuts_through_the_middle() {
        let dir = unique_test_dir("rrcad-native-io-section-mid");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            // NaN is the "no offset given" sentinel.  The box spans z = 0..30,
            // so defaulting to 0 would put the plane on its bottom face and
            // remove the whole solid; the mid-plane at z = 15 must be used
            // instead.  Parts that start at the origin are the common case.
            let (svg_err, dxf_err, svg, dxf) = export_section(&dir, "xy", f64::NAN);
            assert!(
                svg_err.is_none(),
                "an omitted offset should cut the mid-plane, got: {svg_err:?}"
            );
            assert!(
                dxf_err.is_none(),
                "an omitted offset should cut the mid-plane, got: {dxf_err:?}"
            );

            let svg = svg.expect("section SVG was not written");
            assert!(
                svg.contains("hatch\"") && svg.contains("<line"),
                "mid-plane section SVG should be hatched"
            );
            let dxf = dxf.expect("section DXF was not written");
            assert!(
                dxf.contains("HATCH"),
                "mid-plane section DXF should have a HATCH layer"
            );
        });
    }

    #[test]
    fn unknown_section_plane_reports_an_error() {
        let dir = unique_test_dir("rrcad-native-io-section-bad");
        fs::create_dir_all(&dir).expect("create temp dir");

        with_cwd(&dir, || {
            let (svg_err, _, _, _) = export_section(&dir, "diagonal", 0.0);
            let svg_err = svg_err.expect("an unknown section plane should error");
            assert!(
                svg_err.contains("section plane must be"),
                "expected a plane-name error, got: {svg_err}"
            );
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
