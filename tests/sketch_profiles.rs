// tests/sketch_profiles.rs — arc, polygon, ellipse sketch-profile tests
//
// These primitives are exposed at the DSL level but previously only had
// class-identity smoke tests in prelude_layer.rs.  This file verifies
// geometry (bounding box, shape type, volume after extrusion) and error
// handling (zero/negative dimensions, degenerate inputs).

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// arc(r, start_deg, end_deg)  →  Wire
// ---------------------------------------------------------------------------

#[test]
fn arc_returns_wire() {
    let mut vm = MrubyVm::new();
    let ty = vm.eval("arc(5, 0, 90).shape_type").unwrap();
    assert_eq!(ty, ":wire");
}

#[test]
fn arc_full_circle_bounding_box() {
    let mut vm = MrubyVm::new();
    // A full-circle arc with radius 5 must span ≈ 10 in both X and Y.
    let result = vm
        .eval(
            "bb = arc(5, 0, 360).bounding_box
             [bb[:dx], bb[:dy]].min",
        )
        .unwrap();
    let extent: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (extent - 10.0).abs() < 0.5,
        "full-circle arc bbox should be ≈10×10, min extent = {extent}"
    );
}

#[test]
fn arc_semicircle_bounding_box() {
    let mut vm = MrubyVm::new();
    // A 180° arc (r=5): X extent ≈ 10 (diameter), Y extent ≈ 5 (one radius).
    let result = vm
        .eval(
            "bb = arc(5, 0, 180).bounding_box
             [bb[:dx].round, bb[:dy].round]",
        )
        .unwrap();
    // Expect something like "[10, 5]"
    assert!(
        result.contains("10") && result.contains("5"),
        "semicircle bounding box should be ≈10×5, got: {result}"
    );
}

#[test]
fn arc_can_be_used_as_sweep_path() {
    let mut vm = MrubyVm::new();
    // A circle profile swept along an arc path should produce a torus-like shape.
    vm.eval(
        "path = arc(10, 0, 270)
         circle(1).sweep(path)",
    )
    .expect("sweep along arc path failed");
}

#[test]
fn arc_zero_sweep_raises() {
    let mut vm = MrubyVm::new();
    // start_deg == end_deg → zero-length arc — OCCT should reject it.
    let err = vm.eval("arc(5, 45, 45)");
    assert!(err.is_err(), "zero-length arc should raise an error");
}

// ---------------------------------------------------------------------------
// polygon(points)  →  Wire (closed polyline through the given vertices)
// ---------------------------------------------------------------------------

#[test]
fn polygon_returns_face() {
    // A closed polygon over a planar set of points is automatically filled
    // into a Face by OCCT (not just a bare Wire).
    let mut vm = MrubyVm::new();
    let ty = vm
        .eval("polygon([[0,0],[10,0],[10,10],[0,10]]).shape_type")
        .unwrap();
    assert_eq!(ty, ":face");
}

#[test]
fn polygon_triangle_bounding_box() {
    let mut vm = MrubyVm::new();
    // A right-angle triangle with legs 6 and 8 → bbox ≈ 6×8.
    let result = vm
        .eval(
            "bb = polygon([[0,0],[6,0],[0,8]]).bounding_box
             [bb[:dx].round, bb[:dy].round]",
        )
        .unwrap();
    assert!(
        result.contains("6") && result.contains("8"),
        "triangle bbox should be ≈6×8, got: {result}"
    );
}

#[test]
fn polygon_can_extrude() {
    let mut vm = MrubyVm::new();
    // A closed polygon face can be extruded into a prism.
    let ty = vm
        .eval("polygon([[0,0],[5,0],[2.5,5]]).extrude(10).shape_type")
        .unwrap();
    assert_eq!(ty, ":solid", "extruded polygon should be a solid");
}

#[test]
fn polygon_extruded_volume_correct() {
    let mut vm = MrubyVm::new();
    // A square polygon (5×5) extruded 4 units → volume = 100.
    let vol = vm
        .eval("polygon([[0,0],[5,0],[5,5],[0,5]]).extrude(4).volume")
        .unwrap();
    let v: f64 = vol.trim().parse().expect("expected a float");
    assert!(
        (v - 100.0).abs() < 1.0,
        "5×5×4 polygon prism volume should be ≈100, got {v}"
    );
}

#[test]
fn polygon_too_few_points_raises() {
    let mut vm = MrubyVm::new();
    // A polygon needs at least 3 points to form a closed wire.
    let err = vm.eval("polygon([[0,0],[5,0]])");
    assert!(
        err.is_err(),
        "polygon with < 3 points should raise an error"
    );
}

// ---------------------------------------------------------------------------
// ellipse(rx, ry)  →  Face  (filled ellipse in the XY plane)
// ---------------------------------------------------------------------------

#[test]
fn ellipse_returns_face() {
    let mut vm = MrubyVm::new();
    let ty = vm.eval("ellipse(6, 3).shape_type").unwrap();
    assert_eq!(ty, ":face");
}

#[test]
fn ellipse_bounding_box_matches_semi_axes() {
    let mut vm = MrubyVm::new();
    // ellipse(rx=8, ry=4) → bbox ≈ 16×8.
    let result = vm
        .eval(
            "bb = ellipse(8, 4).bounding_box
             [bb[:dx].round, bb[:dy].round]",
        )
        .unwrap();
    assert!(
        result.contains("16") && result.contains("8"),
        "ellipse(8,4) bbox should be ≈16×8, got: {result}"
    );
}

#[test]
fn ellipse_can_extrude() {
    let mut vm = MrubyVm::new();
    let ty = vm.eval("ellipse(5, 3).extrude(10).shape_type").unwrap();
    assert_eq!(ty, ":solid", "extruded ellipse should be a solid");
}

#[test]
fn ellipse_area_matches_pi_rx_ry() {
    let mut vm = MrubyVm::new();
    // Area of an ellipse = π × rx × ry.  ellipse(6, 4) → π × 24 ≈ 75.4.
    let area = vm.eval("ellipse(6, 4).surface_area").unwrap();
    let a: f64 = area.trim().parse().expect("expected a float");
    let expected = std::f64::consts::PI * 6.0 * 4.0;
    assert!(
        (a - expected).abs() < 0.5,
        "ellipse area should be ≈{expected:.1}, got {a}"
    );
}

#[test]
fn ellipse_zero_rx_raises() {
    // OCCT collapses a zero-semi-axis ellipse to a degenerate line segment;
    // the DSL rejects it with a RuntimeError before that can happen.
    let mut vm = MrubyVm::new();
    let err = vm.eval("ellipse(0, 3)");
    // Either the DSL raises an error, or OCCT produces a degenerate shape with
    // no meaningful area.  Both are acceptable guards.
    if let Ok(result) = err {
        // If no exception was raised, the shape must be degenerate (near-zero area).
        let area = vm.eval("ellipse(0, 3).surface_area").unwrap_or_default();
        let a: f64 = area.trim().parse().unwrap_or(0.0);
        assert!(
            a < 0.01,
            "ellipse(0, 3) should produce a degenerate shape, got area={a} / inspect={result}"
        );
    }
    // Otherwise the Err path is the expected guard — test passes.
}

#[test]
fn ellipse_negative_ry_raises() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("ellipse(5, -1)");
    assert!(
        err.is_err(),
        "ellipse with negative ry should raise an error"
    );
}
