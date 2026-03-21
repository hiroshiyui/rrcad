/// Rust-side extern "C" functions called from `glue.c`.
///
/// Each constructor allocates a heap `Box<occt::Shape>` and returns the raw
/// pointer cast to `*mut c_void`.  The C `dfree` callback (`rrcad_shape_drop`)
/// reclaims that memory when mRuby GC collects the `RData` object.
///
/// Error reporting: when an OCCT operation fails the function writes a pointer
/// to a thread-local `CString` into `*error_out` and returns null.  The C
/// caller checks `error_out` and raises a Ruby `RuntimeError` before the
/// thread-local slot is overwritten.
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
    match Shape::make_box(dx, dy, dz) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_cylinder(
    r: f64,
    h: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_cylinder(r, h) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_sphere(
    r: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_sphere(r) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_cone(
    r1: f64,
    r2: f64,
    h: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_cone(r1, r2, h) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_torus(
    r1: f64,
    r2: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_torus(r1, r2) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match Shape::make_wedge(dx, dy, dz, ltx) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match sa.fuse(sb) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match sa.cut(sb) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match sa.common(sb) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match shape.translate(dx, dy, dz) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match shape.rotate(ax, ay, az, angle_deg) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_scale(
    ptr: *mut c_void,
    factor: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.scale(factor) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_fillet(
    ptr: *mut c_void,
    radius: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.fillet(radius) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_chamfer(
    ptr: *mut c_void,
    dist: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.chamfer(dist) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match shape.mirror(plane_str) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match Shape::make_rect(w, h) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_circle_face(
    r: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_circle_face(r) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_polygon(
    pts: *const f64,
    n_pts: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 2) };
    match Shape::make_polygon(slice) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_ellipse_face(
    rx: f64,
    ry: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_ellipse_face(rx, ry) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_arc(
    r: f64,
    start_deg: f64,
    end_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    match Shape::make_arc(r, start_deg, end_deg) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match shape.extrude(height) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_shape_revolve(
    ptr: *mut c_void,
    angle_deg: f64,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let shape = unsafe { &*(ptr as *const Shape) };
    match shape.revolve(angle_deg) {
        Ok(s) => Box::into_raw(Box::new(s)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
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
    match Shape::make_spline_2d(slice) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_make_spline_3d(
    pts: *const f64,
    n_pts: usize,
    error_out: *mut *const c_char,
) -> *mut c_void {
    unsafe { *error_out = std::ptr::null() };
    let slice = unsafe { std::slice::from_raw_parts(pts, n_pts * 3) };
    match Shape::make_spline_3d(slice) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
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
    match sp.sweep(pa) {
        Ok(shape) => Box::into_raw(Box::new(shape)) as *mut c_void,
        Err(e) => {
            unsafe { set_err(error_out, &e) };
            std::ptr::null_mut()
        }
    }
}
