// Phase 8 Tier 1 — Core Part Design
//
// Tests for:
//   shape.pad(face_sel, height:) { sketch }  → Solid (fused)
//   shape.pocket(face_sel, depth:) { sketch } → Solid (cut)
//   shape.fillet_wire(r)                      → Wire or Face with rounded corners
//   datum_plane(origin:, normal:, x_dir:)     → Face (reference plane)

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// pad
// ---------------------------------------------------------------------------

#[test]
fn pad_on_top_face_increases_height() {
    let mut vm = MrubyVm::new();
    // A 10×10×10 box padded with a 5×5 rect on top by 5 units should reach z=15.
    let result = vm
        .eval(
            "b = box(10, 10, 10)
             result = b.pad(:top, height: 5) { rect(5, 5) }
             bb = result.bounding_box
             bb[:z] + bb[:dz]", // zmax = zmin + height
        )
        .unwrap();
    // zmax should be ≈15.0 (original 10 + pad 5).
    let zmax: f64 = result.trim().parse().expect("expected a float");
    assert!((zmax - 15.0).abs() < 0.5, "expected zmax ≈ 15, got {zmax}");
}

#[test]
fn pad_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "b = box(10, 10, 10)
             b.pad(:top, height: 3) { rect(4, 4) }.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn pad_face_selector_as_shape() {
    let mut vm = MrubyVm::new();
    // Pass an explicit face Shape instead of a Symbol selector.
    let result = vm
        .eval(
            "b = box(10, 10, 10)
             top = b.faces(:top).first
             b.pad(top, height: 2) { rect(3, 3) }.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":solid");
}

// ---------------------------------------------------------------------------
// pocket
// ---------------------------------------------------------------------------

#[test]
fn pocket_on_top_face_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "b = box(10, 10, 10)
             b.pocket(:top, depth: 3) { rect(4, 4) }.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn pocket_reduces_volume() {
    let mut vm = MrubyVm::new();
    // A 10×10×10 box has volume 1000; cutting a 4×4×3 pocket removes 48 units.
    let result = vm
        .eval(
            "b = box(10, 10, 10)
             with_pocket = b.pocket(:top, depth: 3) { rect(4, 4) }
             with_pocket.volume < b.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "pocket should reduce volume");
}

// ---------------------------------------------------------------------------
// fillet_wire
// ---------------------------------------------------------------------------

#[test]
fn fillet_wire_on_rect_face_returns_face() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("rect(10, 10).fillet_wire(2.0).shape_type").unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn fillet_wire_increases_edge_count() {
    let mut vm = MrubyVm::new();
    // A rect has 4 edges; after filleting 4 corners the count rises.
    let result = vm
        .eval(
            "r = rect(10, 10)
             f = r.fillet_wire(2.0)
             f.edges(:all).length > r.edges(:all).length",
        )
        .unwrap();
    assert_eq!(result, "true", "fillet_wire should add arc edges");
}

#[test]
fn fillet_wire_rejects_solid() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("box(5, 5, 5).fillet_wire(1.0)").unwrap_err();
    assert!(
        err.contains("Wire") || err.contains("Face"),
        "expected type error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// datum_plane
// ---------------------------------------------------------------------------

#[test]
fn datum_plane_returns_face() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("datum_plane(origin: [0,0,5], normal: [0,0,1], x_dir: [1,0,0]).shape_type")
        .unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn datum_plane_at_correct_z() {
    let mut vm = MrubyVm::new();
    // A datum plane with origin z=7 should have a centroid near z=7.
    let result = vm
        .eval("datum_plane(origin: [0,0,7], normal: [0,0,1], x_dir: [1,0,0]).centroid[2]")
        .unwrap();
    let z: f64 = result.trim().parse().expect("expected a float");
    assert!((z - 7.0).abs() < 0.5, "expected centroid z ≈ 7, got {z}");
}

#[test]
fn datum_plane_tilted_normal() {
    let mut vm = MrubyVm::new();
    // A tilted plane (normal = [0,1,0]) should still produce a Face.
    let result = vm
        .eval("datum_plane(origin: [0,5,0], normal: [0,1,0], x_dir: [1,0,0]).shape_type")
        .unwrap();
    assert_eq!(result, ":face");
}
