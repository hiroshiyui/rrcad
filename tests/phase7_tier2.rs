// Phase 7 Tier 2 — Validation & introspection
//
// Tests for:
//   Shape#shape_type  → Ruby Symbol (:solid, :shell, :face, :wire, :edge, :vertex)
//   Shape#centroid    → [x, y, z]
//   Shape#closed?     → Boolean (true for a solid)
//   Shape#manifold?   → Boolean (true for a clean solid)
//   Shape#validate    → :ok for a well-formed box

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// shape_type
// ---------------------------------------------------------------------------

#[test]
fn shape_type_solid_returns_solid_symbol() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(10, 20, 30).shape_type").unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn shape_type_face_returns_face_symbol() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("rect(5, 5).shape_type").unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn shape_type_wire_returns_wire_symbol() {
    let mut vm = MrubyVm::new();
    // spline_3d produces a 3D Wire (spine for sweep), not a Face.
    let result = vm
        .eval("spline_3d([[0,0,0],[5,5,5],[10,0,0]]).shape_type")
        .unwrap();
    assert_eq!(result, ":wire");
}

// ---------------------------------------------------------------------------
// centroid
// ---------------------------------------------------------------------------

#[test]
fn centroid_box_at_origin() {
    let mut vm = MrubyVm::new();
    // A 10×20×30 box at origin — centroid should be at [5, 10, 15].
    let result = vm
        .eval("c = box(10, 20, 30).centroid; c.map { |v| v.round(6) }")
        .unwrap();
    // The result is a Ruby Array inspect, e.g. "[5.0, 10.0, 15.0]".
    assert!(
        result.contains("5.0") || result.contains("5"),
        "centroid x: {result}"
    );
    assert!(
        result.contains("10.0") || result.contains("10"),
        "centroid y: {result}"
    );
    assert!(
        result.contains("15.0") || result.contains("15"),
        "centroid z: {result}"
    );
}

#[test]
fn centroid_returns_array_of_three() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(5, 5, 5).centroid.length").unwrap();
    assert_eq!(result, "3");
}

// ---------------------------------------------------------------------------
// closed?
// ---------------------------------------------------------------------------

#[test]
fn closed_box_is_closed() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(10, 20, 30).closed?").unwrap();
    assert_eq!(result, "true");
}

#[test]
fn closed_face_is_not_closed() {
    let mut vm = MrubyVm::new();
    // A bare planar face has boundary edges shared by only one face.
    let result = vm.eval("rect(5, 5).closed?").unwrap();
    assert_eq!(result, "false");
}

// ---------------------------------------------------------------------------
// manifold?
// ---------------------------------------------------------------------------

#[test]
fn manifold_box_is_manifold() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(10, 20, 30).manifold?").unwrap();
    assert_eq!(result, "true");
}

#[test]
fn manifold_face_is_not_manifold() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("rect(5, 5).manifold?").unwrap();
    assert_eq!(result, "false");
}

// ---------------------------------------------------------------------------
// validate
// ---------------------------------------------------------------------------

#[test]
fn validate_box_returns_ok() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(10, 20, 30).validate").unwrap();
    assert_eq!(result, ":ok");
}

#[test]
fn validate_sphere_returns_ok() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("sphere(5).validate").unwrap();
    assert_eq!(result, ":ok");
}

#[test]
fn validate_fused_solid_returns_ok() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10,10,10).fuse(sphere(4).translate(5,5,10)).validate")
        .unwrap();
    assert_eq!(result, ":ok");
}
