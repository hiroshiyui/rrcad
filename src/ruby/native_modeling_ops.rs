use std::ffi::{c_char, c_void};

use crate::occt::Shape;

use super::native_helpers::{set_err, shape_result_to_ptr};

// ---------------------------------------------------------------------------
// Phase 7 Tier 1: grid_pattern, fuse_all, cut_all
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_grid_pattern(
    ptr: *mut c_void,
    nx: std::ffi::c_int,
    ny: std::ffi::c_int,
    dx: f64,
    dy: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.grid_pattern(nx, ny, dx, dy), error_out) }
}

/// `fuse_all(ptrs, n)` - fold-left fuse over n shapes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fuse_all(
    ptrs: *const *const c_void,
    n: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shapes: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    let refs: Vec<&Shape> = shapes.to_vec();
    unsafe { shape_result_to_ptr(Shape::fuse_all(&refs), error_out) }
}

/// `cut_all(base, ptrs, n)` - fold-left cut: subtract n tools from base.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_cut_all(
    base: *mut c_void,
    ptrs: *const *const c_void,
    n: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let base_shape = unsafe { &*(base as *const Shape) };
    let shapes: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    let refs: Vec<&Shape> = shapes.to_vec();
    unsafe { shape_result_to_ptr(base_shape.cut_all(&refs), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 5: Advanced composition
// ---------------------------------------------------------------------------

/// `fragment(ptrs, n)` - split n shapes at all mutual intersections.
/// Returns a Compound of all non-overlapping pieces.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fragment(
    ptrs: *const *const c_void,
    n: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shapes: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    unsafe { shape_result_to_ptr(Shape::fragment_all(&shapes), error_out) }
}

/// `.convex_hull` - 3-D convex hull of the shape's tessellated mesh vertices.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_convex_hull(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.convex_hull(), error_out) }
}

/// `path_pattern(shape, path, n)` - distribute n copies of `shape` along `path`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_path_pattern(
    ptr: *mut c_void,
    path_ptr: *mut c_void,
    n: std::ffi::c_int,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let path = unsafe { &*(path_ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.path_pattern(path, n), error_out) }
}

/// `.sweep(path, guide:)` - guided sweep with auxiliary spine.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_sweep_guide(
    ptr: *mut c_void,
    path_ptr: *mut c_void,
    guide_ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let profile = unsafe { &*(ptr as *const Shape) };
    let path = unsafe { &*(path_ptr as *const Shape) };
    let guide = unsafe { &*(guide_ptr as *const Shape) };
    unsafe { shape_result_to_ptr(profile.sweep_guide(path, guide), error_out) }
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

/// Variable-section pipe sweep.
/// `ptrs` is an array of `n` raw Shape pointers (the section profiles);
/// `path` is the spine Wire produced by `spline_3d`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_sweep_sections(
    ptrs: *const *const c_void,
    n: usize,
    path: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let pa = unsafe { &*(path as *const Shape) };
    let profiles: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    unsafe { shape_result_to_ptr(Shape::sweep_sections(&profiles, pa), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 4: 3-D operations - loft, shell, offset, extrude_ex
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
    let profiles: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    unsafe { shape_result_to_ptr(Shape::loft(&profiles, ruled != 0), error_out) }
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_offset_2d(
    ptr: *mut c_void,
    distance: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.offset_2d(distance), error_out) }
}

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

// ---------------------------------------------------------------------------
// Bézier surface patch (Phase 7 / teapot rebuild)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_bezier_patch(
    pts: *const f64,
    n_pts: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts) };
    unsafe { shape_result_to_ptr(Shape::make_bezier_patch(slice), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_sew(
    ptrs: *const *const c_void,
    n: usize,
    tolerance: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let faces: Vec<&Shape> = (0..n)
        .map(|i| unsafe { &*(*ptrs.add(i) as *const Shape) })
        .collect();
    let face_refs: Vec<&Shape> = faces.to_vec();
    unsafe { shape_result_to_ptr(Shape::sew(&face_refs, tolerance), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 1: Core Part Design
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_pad(
    body_ptr: *mut c_void,
    face_ptr: *mut c_void,
    sketch_ptr: *mut c_void,
    height: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let body = unsafe { &*(body_ptr as *const Shape) };
    let face = unsafe { &*(face_ptr as *const Shape) };
    let sketch = unsafe { &*(sketch_ptr as *const Shape) };
    unsafe { shape_result_to_ptr(body.pad(face, sketch, height), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_pocket(
    body_ptr: *mut c_void,
    face_ptr: *mut c_void,
    sketch_ptr: *mut c_void,
    depth: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let body = unsafe { &*(body_ptr as *const Shape) };
    let face = unsafe { &*(face_ptr as *const Shape) };
    let sketch = unsafe { &*(sketch_ptr as *const Shape) };
    unsafe { shape_result_to_ptr(body.pocket(face, sketch, depth), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fillet_wire(
    ptr: *mut c_void,
    radius: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.fillet_wire(radius), error_out) }
}

#[allow(clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_datum_plane(
    ox: f64,
    oy: f64,
    oz: f64,
    nx: f64,
    ny: f64,
    nz: f64,
    xx: f64,
    xy: f64,
    xz: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe {
        shape_result_to_ptr(
            Shape::make_datum_plane(ox, oy, oz, nx, ny, nz, xx, xy, xz),
            error_out,
        )
    }
}

// ---------------------------------------------------------------------------
// Phase 7 Tier 3: surface modeling
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_ruled_surface(
    a: *mut c_void,
    b: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let wa = unsafe { &*(a as *const Shape) };
    let wb = unsafe { &*(b as *const Shape) };
    unsafe { shape_result_to_ptr(Shape::ruled_surface(wa, wb), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_fill_surface(
    ptr: *mut c_void,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(Shape::fill_surface(shape), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_slice(
    ptr: *mut c_void,
    plane: *const c_char,
    offset: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    let plane_str = match unsafe { std::ffi::CStr::from_ptr(plane) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { set_err(error_out, "plane is not valid UTF-8") };
            return std::ptr::null_mut();
        }
    };
    unsafe { shape_result_to_ptr(shape.slice(plane_str, offset), error_out) }
}

// ---------------------------------------------------------------------------
// Phase 8 Tier 2 - Manufacturing features
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_extrude_draft(
    ptr: *mut c_void,
    height: f64,
    draft_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    unsafe { shape_result_to_ptr(shape.extrude_draft(height, draft_deg), error_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_helix(
    radius: f64,
    pitch: f64,
    height: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    unsafe { shape_result_to_ptr(Shape::make_helix(radius, pitch, height), error_out) }
}

#[cfg(test)]
mod tests {
    use super::{
        rrcad_datum_plane, rrcad_fill_surface, rrcad_make_bezier_patch, rrcad_make_helix,
        rrcad_ruled_surface, rrcad_sew, rrcad_shape_convex_hull, rrcad_shape_cut_all,
        rrcad_shape_extrude_ex, rrcad_shape_fragment, rrcad_shape_fuse_all,
        rrcad_shape_grid_pattern, rrcad_shape_loft, rrcad_shape_offset, rrcad_shape_offset_2d,
        rrcad_shape_pad, rrcad_shape_path_pattern, rrcad_shape_pocket, rrcad_shape_shell,
        rrcad_shape_simplify, rrcad_shape_slice, rrcad_shape_sweep, rrcad_shape_sweep_sections,
    };
    use crate::occt::Shape;
    use std::{ffi::CStr, os::raw::c_char, ptr};

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
    fn composition_wrappers_return_shapes() {
        let box_a = boxed_shape(Shape::make_box(10.0, 10.0, 10.0).unwrap());
        let box_b = boxed_shape(
            Shape::make_box(10.0, 10.0, 10.0)
                .unwrap()
                .translate(5.0, 0.0, 0.0)
                .unwrap(),
        );
        let face_a = boxed_shape(Shape::make_rect(10.0, 10.0).unwrap());
        let face_b = boxed_shape(
            Shape::make_rect(12.0, 12.0)
                .unwrap()
                .translate(0.0, 0.0, 5.0)
                .unwrap(),
        );
        let path = boxed_shape(
            Shape::make_spline_3d(&[
                0.0, 0.0, 0.0, //
                5.0, 0.0, 2.0, //
                10.0, 0.0, 0.0,
            ])
            .unwrap(),
        );
        let mut err: *const c_char = ptr::null();

        let grid = unsafe { rrcad_shape_grid_pattern(box_a, 2, 2, 20.0, 20.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!grid.is_null());

        let ptrs = [box_a as *const _, box_b as *const _];
        let fused = unsafe { rrcad_shape_fuse_all(ptrs.as_ptr(), ptrs.len(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!fused.is_null());

        let cut = unsafe { rrcad_shape_cut_all(box_a, ptrs.as_ptr(), ptrs.len(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!cut.is_null());

        let fragment = unsafe { rrcad_shape_fragment(ptrs.as_ptr(), ptrs.len(), &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!fragment.is_null());

        let hull = unsafe { rrcad_shape_convex_hull(box_a, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!hull.is_null());

        let path_pattern = unsafe { rrcad_shape_path_pattern(face_a, path, 3, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!path_pattern.is_null());

        let sweep = unsafe { rrcad_shape_sweep(face_a, path, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!sweep.is_null());

        let sections = [face_a as *const _, face_b as *const _];
        let sweep_sections = unsafe {
            rrcad_shape_sweep_sections(sections.as_ptr(), sections.len(), path, &mut err)
        };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!sweep_sections.is_null());

        unsafe {
            reclaim_shape(box_a);
            reclaim_shape(box_b);
            reclaim_shape(face_a);
            reclaim_shape(face_b);
            reclaim_shape(path);
            reclaim_shape(grid);
            reclaim_shape(fused);
            reclaim_shape(cut);
            reclaim_shape(fragment);
            reclaim_shape(hull);
            reclaim_shape(path_pattern);
            reclaim_shape(sweep);
            reclaim_shape(sweep_sections);
        }
    }

    #[test]
    fn loft_and_surface_wrappers_return_shapes() {
        let profile_a = boxed_shape(Shape::make_rect(10.0, 10.0).unwrap());
        let profile_b = boxed_shape(
            Shape::make_rect(12.0, 12.0)
                .unwrap()
                .translate(0.0, 0.0, 5.0)
                .unwrap(),
        );
        let face = boxed_shape(Shape::make_circle_face(5.0).unwrap());
        let mut err: *const c_char = ptr::null();

        let profiles = [profile_a as *const _, profile_b as *const _];
        let loft = unsafe { rrcad_shape_loft(profiles.as_ptr(), profiles.len(), 0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!loft.is_null());

        let solid = boxed_shape(Shape::make_box(10.0, 10.0, 10.0).unwrap());

        let shell = unsafe { rrcad_shape_shell(solid, 2.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!shell.is_null());

        let offset = unsafe { rrcad_shape_offset(solid, 1.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!offset.is_null());

        let offset_2d = unsafe { rrcad_shape_offset_2d(face, 1.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!offset_2d.is_null());

        let simplify = unsafe { rrcad_shape_simplify(solid, 0.5, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!simplify.is_null());

        let extrude_ex = unsafe { rrcad_shape_extrude_ex(face, 5.0, 0.0, 1.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!extrude_ex.is_null());

        let body = boxed_shape(Shape::make_box(10.0, 10.0, 10.0).unwrap());
        let face_for_pad = boxed_shape(Shape::make_rect(10.0, 10.0).unwrap());
        let sketch = boxed_shape(Shape::make_circle_face(2.0).unwrap());
        let pad = unsafe { rrcad_shape_pad(body, face_for_pad, sketch, 5.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!pad.is_null());

        let pocket = unsafe { rrcad_shape_pocket(body, face_for_pad, sketch, 3.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!pocket.is_null());

        let slice = unsafe {
            rrcad_shape_slice(
                body,
                std::ffi::CString::new("xy").unwrap().as_ptr(),
                5.0,
                &mut err,
            )
        };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!slice.is_null());

        let helix = unsafe { rrcad_make_helix(5.0, 2.0, 10.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!helix.is_null());

        let ruled_a = boxed_shape(
            Shape::make_spline_3d(&[
                0.0, 0.0, 0.0, //
                2.0, 0.0, 0.0, //
                2.0, 1.0, 0.0, //
                0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0,
            ])
            .unwrap(),
        );
        let ruled_b = boxed_shape(
            Shape::make_spline_3d(&[
                0.0, 0.0, 3.0, //
                2.0, 0.0, 3.0, //
                2.0, 1.0, 3.0, //
                0.0, 1.0, 3.0, //
                0.0, 0.0, 3.0,
            ])
            .unwrap(),
        );
        let ruled = unsafe { rrcad_ruled_surface(ruled_a, ruled_b, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!ruled.is_null());

        let boundary_wire = boxed_shape(Shape::make_arc(5.0, 0.0, 360.0).unwrap());
        let filled = unsafe { rrcad_fill_surface(boundary_wire, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!filled.is_null());

        let bezier = unsafe {
            rrcad_make_bezier_patch(
                [
                    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, //
                    0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 2.0, 1.0, 0.0, 3.0, 1.0, 0.0, //
                    0.0, 2.0, 0.0, 1.0, 2.0, 0.0, 2.0, 2.0, 0.0, 3.0, 2.0, 0.0, //
                    0.0, 3.0, 0.0, 1.0, 3.0, 0.0, 2.0, 3.0, 0.0, 3.0, 3.0, 0.0,
                ]
                .as_ptr(),
                48,
                &mut err,
            )
        };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!bezier.is_null());

        let sew_ptrs = [face as *const _];
        let sewn = unsafe { rrcad_sew(sew_ptrs.as_ptr(), sew_ptrs.len(), 0.01, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!sewn.is_null());

        let datum =
            unsafe { rrcad_datum_plane(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, &mut err) };
        assert!(unsafe { error_message(err) }.is_none());
        assert!(!datum.is_null());

        unsafe {
            reclaim_shape(profile_a);
            reclaim_shape(profile_b);
            reclaim_shape(face);
            reclaim_shape(solid);
            reclaim_shape(loft);
            reclaim_shape(shell);
            reclaim_shape(offset);
            reclaim_shape(offset_2d);
            reclaim_shape(simplify);
            reclaim_shape(extrude_ex);
            reclaim_shape(body);
            reclaim_shape(face_for_pad);
            reclaim_shape(sketch);
            reclaim_shape(pad);
            reclaim_shape(pocket);
            reclaim_shape(slice);
            reclaim_shape(helix);
            reclaim_shape(ruled);
            reclaim_shape(filled);
            reclaim_shape(bezier);
            reclaim_shape(sewn);
            reclaim_shape(datum);
        }
    }
}
