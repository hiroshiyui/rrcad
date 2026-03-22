// Phase 7 Tier 3 — Surface modeling
//
// Tests for:
//   ruled_surface(wire_a, wire_b)  → Shell between two wires
//   fill_surface(boundary_wire)    → smooth NURBS face filling a closed wire
//   shape.slice(plane: :xy, z: d)  → cross-section compound

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// ruled_surface
// ---------------------------------------------------------------------------

#[test]
fn ruled_surface_between_two_rect_wires() {
    let mut vm = MrubyVm::new();
    // Two rectangular wire profiles at different heights, connected by a ruled shell.
    let result = vm
        .eval(
            "wa = spline_3d([[0,0,0],[1,0,0],[1,1,0],[0,1,0],[0,0,0]])
             wb = spline_3d([[0,0,5],[1,0,5],[1,1,5],[0,1,5],[0,0,5]])
             ruled_surface(wa, wb).shape_type",
        )
        .unwrap();
    assert_eq!(result, ":shell");
}

#[test]
fn ruled_surface_requires_wire_inputs() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("ruled_surface(box(1,1,1), box(1,1,2))")
        .unwrap_err();
    assert!(
        err.contains("must be a Wire"),
        "expected wire error, got: {err}"
    );
}

#[test]
fn ruled_surface_type_is_shell() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "w1 = spline_3d([[0,0,0],[2,0,0],[2,2,0]])
             w2 = spline_3d([[0,0,3],[2,0,3],[2,2,3]])
             s = ruled_surface(w1, w2)
             s.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":shell");
}

// ---------------------------------------------------------------------------
// fill_surface
// ---------------------------------------------------------------------------

#[test]
fn fill_surface_arc_produces_face() {
    let mut vm = MrubyVm::new();
    // A full circle arc (360°) gives a closed wire; fill it to get a face.
    let result = vm
        .eval(
            "w = arc(5, 0, 360)
             fill_surface(w).shape_type",
        )
        .unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn fill_surface_requires_wire() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("fill_surface(box(1,1,1))").unwrap_err();
    assert!(
        err.contains("must be a Wire"),
        "expected wire error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// slice
// ---------------------------------------------------------------------------

#[test]
fn slice_box_xy_plane_at_midpoint() {
    let mut vm = MrubyVm::new();
    // Slice a 10×10×10 box at z=5 — should produce a compound of edges.
    let result = vm
        .eval("box(10, 10, 10).slice(plane: :xy, z: 5).shape_type")
        .unwrap();
    // BRepAlgoAPI_Section returns a compound of edges
    assert_eq!(result, ":compound");
}

#[test]
fn slice_box_xz_plane() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10, 10, 10).slice(plane: :xz, y: 5).shape_type")
        .unwrap();
    assert_eq!(result, ":compound");
}

#[test]
fn slice_box_yz_plane() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10, 10, 10).slice(plane: :yz, x: 5).shape_type")
        .unwrap();
    assert_eq!(result, ":compound");
}

#[test]
fn slice_defaults_offset_to_zero() {
    let mut vm = MrubyVm::new();
    // No z: key — offset defaults to 0.0 (slice at the bottom face).
    // The section at z=0 coincides with the face boundary, still returns a compound.
    let result = vm
        .eval("box(10, 10, 10).slice(plane: :xy).shape_type")
        .unwrap();
    assert_eq!(result, ":compound");
}

#[test]
fn slice_rejects_unknown_plane() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("box(10, 10, 10).slice(plane: :zw, z: 1)")
        .unwrap_err();
    assert!(
        err.contains(":xy") || err.contains("plane"),
        "expected plane error, got: {err}"
    );
}
