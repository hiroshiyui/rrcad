// Phase 10 — Assembly constraints beyond `mate`.
//
// Tests for:
//   shape.rotate_about(point, axis_dir, angle_deg) — rotate around a non-origin pivot
//   Assembly#distance_mate(shape, from:, to:, distance:)
//   Assembly#axis_align(shape, from: [p1, p2], to: [q1, q2])
//   Assembly#angle_mate(shape, from:, to:, angle:, pivot:, axis_dir:)

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Shape#rotate_about
// ---------------------------------------------------------------------------

#[test]
fn rotate_about_keeps_pivot_point_fixed() {
    let mut vm = MrubyVm::new();
    // A 10×10×10 box centered at (5,5,5) rotated 180° about (5,5,5) along Z
    // should land in the same bounding box.
    let result = vm
        .eval(
            "b = box(10, 10, 10).rotate_about([5, 5, 5], [0, 0, 1], 180)
             b.bounding_box[:dx]",
        )
        .unwrap_or_else(|e| panic!("eval failed: {e}"));
    let dx: f64 = result
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("parse {result:?}: {e}"));
    assert!((dx - 10.0).abs() < 0.01, "expected dx ≈ 10, got {dx}");
}

#[test]
fn rotate_about_off_center_moves_bounding_box() {
    let mut vm = MrubyVm::new();
    // box(10,10,10) at min (0,0,0). Rotate 90° CCW about (10,0,0) along +Z:
    // corners map XY ∈ [0,10]×[-10,0]. y_min should now be ≈ -10.
    let result = vm
        .eval(
            "b = box(10, 10, 10).rotate_about([10, 0, 0], [0, 0, 1], 90)
             b.bounding_box[:y]",
        )
        .unwrap();
    let y_min: f64 = result.trim().parse().expect("number");
    assert!(
        (y_min - (-10.0)).abs() < 0.1,
        "expected y_min near -10, got {y_min}"
    );
}

#[test]
fn rotate_about_rejects_zero_axis() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("box(1, 1, 1).rotate_about([0, 0, 0], [0, 0, 0], 90)")
        .unwrap_err();
    assert!(
        err.to_lowercase().contains("non-zero"),
        "expected non-zero axis error, got: {err}"
    );
}

#[test]
fn rotate_about_rejects_bad_point() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("box(1, 1, 1).rotate_about([0, 0], [0, 0, 1], 90)")
        .unwrap_err();
    assert!(
        err.contains("3-element"),
        "expected 3-element error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Assembly#distance_mate
// ---------------------------------------------------------------------------

#[test]
fn distance_mate_places_shape_with_air_gap() {
    let mut vm = MrubyVm::new();
    // Post mated 5 mm above a base plate should produce a fused shape whose
    // Z-extent matches base height + gap + post height.
    let result = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8)
             asm = assembly(\"gap\") do |a|
               a.place base
               a.distance_mate post,
                 from: post.faces(:bottom).first,
                 to:   base.faces(:top).first,
                 distance: 5
             end
             bb = asm.to_shape.bounding_box
             bb[:z] + bb[:dz]",
        )
        .unwrap();
    let z_max: f64 = result.trim().parse().expect("number");
    assert!(
        (z_max - (5.0 + 5.0 + 8.0)).abs() < 0.1,
        "expected z_max ≈ 18, got {z_max}"
    );
}

#[test]
fn distance_mate_rejects_non_positive_distance() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8)
             assembly(\"\") do |a|
               a.place base
               a.distance_mate post,
                 from: post.faces(:bottom).first,
                 to:   base.faces(:top).first,
                 distance: 0
             end",
        )
        .unwrap_err();
    assert!(err.contains("> 0"), "expected > 0 error, got: {err}");
}

// ---------------------------------------------------------------------------
// Assembly#axis_align
// ---------------------------------------------------------------------------

#[test]
fn axis_align_aligns_two_collinear_axes() {
    let mut vm = MrubyVm::new();
    // Box centred at origin, source axis (0,0,0)→(0,0,1) along +Z.
    // Target axis (10,20,0)→(10,20,1) — same direction, shifted in XY.
    // Result: box translates by (10,20,0); bounding-box mins shift accordingly.
    let result = vm
        .eval(
            "b = box(2, 2, 2)
             asm = assembly(\"a\") do |a|
               a.axis_align b,
                 from: [[0, 0, 0], [0, 0, 1]],
                 to:   [[10, 20, 0], [10, 20, 1]]
             end
             bb = asm.to_shape.bounding_box
             bb[:x] * 1000.0 + bb[:y]",
        )
        .unwrap();
    // Encode two values in one float: x * 1000 + y. box(2,2,2) translated to (10,20,0)
    // ⇒ x=10, y=20 ⇒ 10020.
    let encoded: f64 = result.trim().parse().expect("number");
    assert!(
        (encoded - 10020.0).abs() < 0.5,
        "expected 10020, got {encoded}"
    );
}

#[test]
fn axis_align_rotates_z_axis_to_x_axis() {
    let mut vm = MrubyVm::new();
    // Tall cylinder along +Z (radius 1, height 10): bounding box is 2×2×10.
    // Align its axis (0,0,0)→(0,0,10) onto (0,0,0)→(10,0,0) so its long
    // dimension becomes X.
    let result = vm
        .eval(
            "c = cylinder(1, 10)
             asm = assembly(\"a\") do |a|
               a.axis_align c,
                 from: [[0, 0, 0], [0, 0, 10]],
                 to:   [[0, 0, 0], [10, 0, 0]]
             end
             bb = asm.to_shape.bounding_box
             bb[:dx]",
        )
        .unwrap();
    let dx: f64 = result.trim().parse().expect("number");
    // Long axis is now X (was Z); cylinder height 10 becomes X extent.
    assert!((dx - 10.0).abs() < 0.2, "expected dx ≈ 10, got {dx}");
}

#[test]
fn axis_align_handles_antiparallel_axes() {
    let mut vm = MrubyVm::new();
    // Source axis +Z (0,0,0→0,0,1), target axis −Z (0,0,0→0,0,−1).
    // A cylinder of height 5 along +Z should flip; bounding box becomes -5..0.
    let result = vm
        .eval(
            "c = cylinder(1, 5)
             asm = assembly(\"a\") do |a|
               a.axis_align c,
                 from: [[0, 0, 0], [0, 0, 1]],
                 to:   [[0, 0, 0], [0, 0, -1]]
             end
             bb = asm.to_shape.bounding_box
             bb[:z]",
        )
        .unwrap();
    let z_min: f64 = result.trim().parse().expect("number");
    assert!(
        (z_min - (-5.0)).abs() < 0.1,
        "expected z_min ≈ -5, got {z_min}"
    );
}

#[test]
fn axis_align_rejects_coincident_axis_points() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "b = box(1, 1, 1)
             assembly(\"\") do |a|
               a.axis_align b, from: [[0, 0, 0], [0, 0, 0]], to: [[1, 0, 0], [2, 0, 0]]
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("distinct"),
        "expected distinct-points error, got: {err}"
    );
}

#[test]
fn axis_align_rejects_malformed_pair() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "b = box(1, 1, 1)
             assembly(\"\") do |a|
               a.axis_align b, from: [[0, 0, 0]], to: [[1, 0, 0], [2, 0, 0]]
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("[point_a, point_b]"),
        "expected pair error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Assembly#angle_mate
// ---------------------------------------------------------------------------

#[test]
fn angle_mate_places_and_rotates_about_pivot() {
    let mut vm = MrubyVm::new();
    // Mate a 4×4×8 post onto the 20×20 top face of a 20×20×5 base, then
    // rotate the placed post 45° about the contact point (10,10,5) around Z.
    // The post's bounding box should remain centred on (10, 10) (it's square,
    // rotated about its own centre) and z_max ≈ 5 + 8 = 13.
    let result = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8).translate(-2, -2, 0)  # centre post X/Y around origin
             asm = assembly(\"a\") do |a|
               a.place base
               a.angle_mate post,
                 from:    post.faces(:bottom).first,
                 to:      base.faces(:top).first,
                 angle:   45,
                 pivot:   [10, 10, 5],
                 axis_dir:[0, 0, 1]
             end
             bb = asm.to_shape.bounding_box
             bb[:z] + bb[:dz]",
        )
        .unwrap();
    let z_max: f64 = result.trim().parse().expect("number");
    assert!(
        (z_max - 13.0).abs() < 0.5,
        "expected z_max ≈ 13, got {z_max}"
    );
}

#[test]
fn angle_mate_with_offset_adds_gap() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8)
             asm = assembly(\"a\") do |a|
               a.place base
               a.angle_mate post,
                 from:    post.faces(:bottom).first,
                 to:      base.faces(:top).first,
                 angle:   0,
                 pivot:   [10, 10, 5],
                 axis_dir:[0, 0, 1],
                 offset:  2
             end
             bb = asm.to_shape.bounding_box
             bb[:z] + bb[:dz]",
        )
        .unwrap();
    let z_max: f64 = result.trim().parse().expect("number");
    assert!(
        (z_max - (5.0 + 2.0 + 8.0)).abs() < 0.1,
        "expected z_max ≈ 15, got {z_max}"
    );
}

#[test]
fn angle_mate_rejects_non_numeric_angle() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8)
             assembly(\"\") do |a|
               a.place base
               a.angle_mate post,
                 from:    post.faces(:bottom).first,
                 to:      base.faces(:top).first,
                 angle:   :ninety,
                 pivot:   [10, 10, 5],
                 axis_dir:[0, 0, 1]
             end",
        )
        .unwrap_err();
    assert!(
        err.to_lowercase().contains("angle"),
        "expected angle error, got: {err}"
    );
}
