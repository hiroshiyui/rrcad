use std::ffi::{c_char, c_void};

use crate::occt::Shape;

use super::native_helpers::{DEFAULT_LINEAR_DEFLECTION, set_err};

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
// Phase 3: Live preview
// ---------------------------------------------------------------------------

/// Tessellate `ptr` to binary glTF (GLB) and notify the WebSocket clients.
/// No-op (returns success) when not in `--preview` mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_preview_shape(ptr: *mut c_void, error_out: *mut *const c_char) {
    unsafe { *error_out = std::ptr::null() };

    let Some(state) = crate::preview::PREVIEW.get() else {
        // Not in --preview mode — silently ignore.
        return;
    };

    let shape = unsafe { &*(ptr as *const Shape) };
    let path = state.glb_path.to_string_lossy();
    if let Err(e) = shape.export_glb(&path, DEFAULT_LINEAR_DEFLECTION) {
        unsafe { set_err(error_out, &e) };
        let metadata_path = crate::preview::metadata_path_for_glb(&state.glb_path);
        let metadata = crate::preview::metadata_json_for_shape_with_error(shape, Some(&e));
        match serde_json::to_vec_pretty(&metadata) {
            Ok(bytes) => {
                if let Err(write_err) = std::fs::write(metadata_path, bytes) {
                    eprintln!("rrcad preview: failed to write error metadata: {write_err}");
                }
            }
            Err(encode_err) => {
                eprintln!("rrcad preview: failed to encode error metadata: {encode_err}")
            }
        }
        state.reload_tx.send(()).ok();
        return;
    }

    let metadata_path = crate::preview::metadata_path_for_glb(&state.glb_path);
    let metadata = crate::preview::metadata_json_for_shape(shape);
    match serde_json::to_vec_pretty(&metadata) {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(metadata_path, bytes) {
                eprintln!("rrcad preview: failed to write metadata: {e}");
            }
        }
        Err(e) => eprintln!("rrcad preview: failed to encode metadata: {e}"),
    }

    state.reload_tx.send(()).ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occt::Shape;
    use std::{
        ffi::{CStr, CString},
        os::raw::c_char,
        ptr,
    };

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
    fn selector_wrappers_cover_counts_getters_and_errors() {
        let solid = boxed_shape(Shape::make_box(10.0, 20.0, 30.0).unwrap());
        let face = boxed_shape(
            Shape::make_box(10.0, 20.0, 30.0)
                .unwrap()
                .faces("top")
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
        );
        let edge = boxed_shape(
            Shape::make_box(10.0, 20.0, 30.0)
                .unwrap()
                .edges("vertical")
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
        );
        let vertex = boxed_shape(
            Shape::make_box(10.0, 20.0, 30.0)
                .unwrap()
                .vertices("all")
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
        );
        let mut err: *const c_char = ptr::null();

        let faces = unsafe {
            rrcad_shape_faces_count(solid, CString::new("top").unwrap().as_ptr(), &mut err)
        };
        assert_eq!(faces, 1);
        assert!(unsafe { error_message(err) }.is_none());

        let face_ptr = unsafe {
            rrcad_shape_faces_get(solid, CString::new("top").unwrap().as_ptr(), 0, &mut err)
        };
        assert!(!face_ptr.is_null());
        assert!(unsafe { error_message(err) }.is_none());

        let edges = unsafe {
            rrcad_shape_edges_count(solid, CString::new("vertical").unwrap().as_ptr(), &mut err)
        };
        assert_eq!(edges, 4);
        assert!(unsafe { error_message(err) }.is_none());

        let edge_ptr = unsafe {
            rrcad_shape_edges_get(
                solid,
                CString::new("vertical").unwrap().as_ptr(),
                0,
                &mut err,
            )
        };
        assert!(!edge_ptr.is_null());
        assert!(unsafe { error_message(err) }.is_none());

        let vertices = unsafe {
            rrcad_shape_vertices_count(solid, CString::new("all").unwrap().as_ptr(), &mut err)
        };
        assert_eq!(vertices, 8);
        assert!(unsafe { error_message(err) }.is_none());

        let vertex_ptr = unsafe {
            rrcad_shape_vertices_get(solid, CString::new("all").unwrap().as_ptr(), 0, &mut err)
        };
        assert!(!vertex_ptr.is_null());
        assert!(unsafe { error_message(err) }.is_none());

        let mut err2: *const c_char = ptr::null();
        let bad = unsafe {
            rrcad_shape_faces_count(solid, CString::new("bad").unwrap().as_ptr(), &mut err2)
        };
        assert_eq!(bad, -1);
        assert!(unsafe { error_message(err2) }.is_some());

        let utf8 = b"\xff\0";
        let mut err3: *const c_char = ptr::null();
        let bad =
            unsafe { rrcad_shape_edges_count(solid, utf8.as_ptr() as *const c_char, &mut err3) };
        assert_eq!(bad, -1);
        assert!(unsafe { error_message(err3) }.is_some());

        let mut err4: *const c_char = ptr::null();
        let bad = unsafe {
            rrcad_shape_vertices_get(solid, CString::new("all").unwrap().as_ptr(), 99, &mut err4)
        };
        assert!(bad.is_null());
        assert!(unsafe { error_message(err4) }.is_some());

        let mut preview_err: *const c_char = ptr::null();
        unsafe { rrcad_preview_shape(solid, &mut preview_err) };
        assert!(unsafe { error_message(preview_err) }.is_none());

        unsafe {
            reclaim_shape(solid);
            reclaim_shape(face);
            reclaim_shape(edge);
            reclaim_shape(vertex);
            reclaim_shape(face_ptr);
            reclaim_shape(edge_ptr);
            reclaim_shape(vertex_ptr);
        }
    }
}
