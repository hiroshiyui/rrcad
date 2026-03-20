/// TDD tests for Phase 3 face/edge sub-shape selectors.
use rrcad::ruby::vm::MrubyVm;

fn eval(code: &str) -> Result<String, String> {
    let mut vm = MrubyVm::new();
    vm.eval(code)
}

// ---------------------------------------------------------------------------
// faces(:all)
// ---------------------------------------------------------------------------

#[test]
fn box_faces_all_returns_array() {
    let r = eval("box(10, 20, 30).faces(:all).class").unwrap();
    assert!(r.contains("Array"), "expected Array, got: {r}");
}

#[test]
fn box_has_six_faces_all() {
    let r = eval("box(10, 20, 30).faces(:all).length").unwrap();
    assert_eq!(r.trim(), "6", "box should have 6 faces total, got: {r}");
}

// ---------------------------------------------------------------------------
// faces(:top) / faces(:bottom)
// ---------------------------------------------------------------------------

#[test]
fn box_has_one_top_face() {
    let r = eval("box(10, 20, 30).faces(:top).length").unwrap();
    assert_eq!(r.trim(), "1", "box should have 1 top face, got: {r}");
}

#[test]
fn box_has_one_bottom_face() {
    let r = eval("box(10, 20, 30).faces(:bottom).length").unwrap();
    assert_eq!(r.trim(), "1", "box should have 1 bottom face, got: {r}");
}

#[test]
fn box_has_four_side_faces() {
    let r = eval("box(10, 20, 30).faces(:side).length").unwrap();
    assert_eq!(r.trim(), "4", "box should have 4 side faces, got: {r}");
}

#[test]
fn faces_returns_shapes() {
    let r = eval("box(10, 20, 30).faces(:top).first.class").unwrap();
    assert!(r.contains("Shape"), "expected Shape elements, got: {r}");
}

// ---------------------------------------------------------------------------
// faces: unknown selector raises RuntimeError
// ---------------------------------------------------------------------------

#[test]
fn faces_unknown_selector_raises() {
    let err = eval("box(10,10,10).faces(:bogus)").unwrap_err();
    assert!(
        err.contains("unknown selector"),
        "expected 'unknown selector' error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// edges(:all)
// ---------------------------------------------------------------------------

#[test]
fn box_edges_all_returns_array() {
    let r = eval("box(10, 20, 30).edges(:all).class").unwrap();
    assert!(r.contains("Array"), "expected Array, got: {r}");
}

#[test]
fn box_has_twelve_edges_all() {
    let r = eval("box(10, 20, 30).edges(:all).length").unwrap();
    assert_eq!(r.trim(), "12", "box should have 12 edges total, got: {r}");
}

// ---------------------------------------------------------------------------
// edges(:vertical) / edges(:horizontal)
// ---------------------------------------------------------------------------

#[test]
fn box_has_four_vertical_edges() {
    let r = eval("box(10, 20, 30).edges(:vertical).length").unwrap();
    assert_eq!(r.trim(), "4", "box should have 4 vertical edges, got: {r}");
}

#[test]
fn box_has_eight_horizontal_edges() {
    let r = eval("box(10, 20, 30).edges(:horizontal).length").unwrap();
    assert_eq!(
        r.trim(),
        "8",
        "box should have 8 horizontal edges, got: {r}"
    );
}

#[test]
fn edges_returns_shapes() {
    let r = eval("box(10, 20, 30).edges(:vertical).first.class").unwrap();
    assert!(r.contains("Shape"), "expected Shape elements, got: {r}");
}

// ---------------------------------------------------------------------------
// edges: unknown selector raises RuntimeError
// ---------------------------------------------------------------------------

#[test]
fn edges_unknown_selector_raises() {
    let err = eval("box(10,10,10).edges(:bogus)").unwrap_err();
    assert!(
        err.contains("unknown selector"),
        "expected 'unknown selector' error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Cylinder — mixed face types
// ---------------------------------------------------------------------------

#[test]
fn cylinder_has_one_top_face() {
    // BRepPrimAPI_MakeCylinder: 1 flat top, 1 flat bottom, 1 lateral face
    let r = eval("cylinder(5.0, 10.0).faces(:top).length").unwrap();
    assert_eq!(r.trim(), "1", "cylinder should have 1 top face, got: {r}");
}

#[test]
fn cylinder_has_one_bottom_face() {
    let r = eval("cylinder(5.0, 10.0).faces(:bottom).length").unwrap();
    assert_eq!(
        r.trim(),
        "1",
        "cylinder should have 1 bottom face, got: {r}"
    );
}

// ---------------------------------------------------------------------------
// Chaining: extrude a rect then select faces
// ---------------------------------------------------------------------------

#[test]
fn extruded_rect_top_face_is_one() {
    let r = eval("rect(10.0, 20.0).extrude(5.0).faces(:top).length").unwrap();
    assert_eq!(
        r.trim(),
        "1",
        "extruded rect should have 1 top face, got: {r}"
    );
}
