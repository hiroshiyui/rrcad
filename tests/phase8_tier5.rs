// Phase 8 Tier 5 — Advanced composition
//
// Tests for:
//   fragment([a, b, c])              → Compound (all non-overlapping pieces)
//   shape.convex_hull                → Shape   (3-D convex hull)
//   path_pattern(shape, path, n)     → Compound (n copies along path)
//   shape.sweep(path, guide: guide)  → Shape   (guided sweep)

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// fragment
// ---------------------------------------------------------------------------

#[test]
fn fragment_two_overlapping_boxes_returns_shape() {
    let mut vm = MrubyVm::new();
    // Two overlapping boxes → fragment should succeed and return a compound.
    vm.eval("fragment([box(10,10,10), box(10,10,10).translate(5,5,5)])")
        .expect("fragment of two overlapping boxes failed");
}

#[test]
fn fragment_single_shape_succeeds() {
    let mut vm = MrubyVm::new();
    vm.eval("fragment([box(5,5,5)])")
        .expect("fragment of a single shape failed");
}

#[test]
fn fragment_non_overlapping_boxes_preserves_volume() {
    let mut vm = MrubyVm::new();
    // Two non-overlapping boxes: fragment should produce the same total volume.
    let vol = vm
        .eval(
            "a = box(10,10,10)
b = box(10,10,10).translate(20,0,0)
fragment([a, b]).volume",
        )
        .unwrap();
    let v: f64 = vol.trim().parse().unwrap();
    // Each box is 10³ = 1000; total ≈ 2000.
    assert!((v - 2000.0).abs() < 10.0, "expected volume ≈ 2000, got {v}");
}

// ---------------------------------------------------------------------------
// convex_hull
// ---------------------------------------------------------------------------

#[test]
fn convex_hull_of_box_has_correct_volume() {
    let mut vm = MrubyVm::new();
    // The convex hull of a box is the box itself (already convex).
    let result = vm
        .eval(
            "b = box(10,10,10)
b.convex_hull.volume",
        )
        .unwrap();
    let v: f64 = result.trim().parse().unwrap();
    // box volume = 1000; hull should be close (mesh rounding only)
    assert!(
        (v - 1000.0).abs() < 100.0,
        "convex hull of box should have volume ≈ 1000, got {v}"
    );
}

#[test]
fn convex_hull_of_cylinder_succeeds() {
    let mut vm = MrubyVm::new();
    vm.eval("cylinder(5, 10).convex_hull")
        .expect("convex_hull of cylinder failed");
}

#[test]
fn convex_hull_of_l_shape_is_larger() {
    let mut vm = MrubyVm::new();
    // An L-shaped union: the convex hull fills in the concavity.
    // Hull volume must be >= the original volume.
    let result = vm
        .eval(
            "a = box(10, 5, 5)
b = box(5, 10, 5).translate(0, 5, 0)
l = fuse_all([a, b])
orig_vol = l.volume
hull_vol = l.convex_hull.volume
hull_vol >= orig_vol",
        )
        .unwrap();
    assert_eq!(
        result.trim(),
        "true",
        "hull volume should be >= original (concave) volume"
    );
}

// ---------------------------------------------------------------------------
// path_pattern
// ---------------------------------------------------------------------------

#[test]
fn path_pattern_n1_succeeds() {
    let mut vm = MrubyVm::new();
    vm.eval("path_pattern(box(2,2,2), spline_3d([[0,0,0],[10,0,0],[20,0,10]]), 1)")
        .expect("path_pattern with n=1 failed");
}

#[test]
fn path_pattern_n3_succeeds() {
    let mut vm = MrubyVm::new();
    vm.eval("path_pattern(box(2,2,2), spline_3d([[0,0,0],[50,0,0]]), 3)")
        .expect("path_pattern with n=3 failed");
}

#[test]
fn path_pattern_volume_scales_with_n() {
    let mut vm = MrubyVm::new();
    // n non-overlapping copies of a 2×2×2 box along a long straight path.
    let vol = vm
        .eval(
            "path = spline_3d([[0,0,0],[0,0,100]])
path_pattern(box(2,2,2), path, 5).volume",
        )
        .unwrap();
    let v: f64 = vol.trim().parse().unwrap();
    // Each box is 8; 5 × 8 = 40.
    assert!(
        (v - 40.0).abs() < 5.0,
        "expected volume ≈ 40 for 5 copies of 2×2×2 box, got {v}"
    );
}

// ---------------------------------------------------------------------------
// sweep with guide: (guided sweep)
// ---------------------------------------------------------------------------

#[test]
fn sweep_with_guide_returns_shape() {
    let mut vm = MrubyVm::new();
    // A rectangular profile swept along a vertical spline with a guide that
    // tilts the profile orientation as it sweeps.
    vm.eval(
        "profile = rect(4, 2)
path  = spline_3d([[0,0,0],[0,0,20]])
guide = spline_3d([[4,0,0],[4,2,20]])
profile.sweep(path, guide: guide)",
    )
    .expect("guided sweep failed");
}

#[test]
fn sweep_without_guide_still_works() {
    // Ensure the refactored mrb_rrcad_shape_sweep still handles the plain case.
    let mut vm = MrubyVm::new();
    vm.eval(
        "profile = circle(3)
path = spline_3d([[0,0,0],[0,0,15]])
profile.sweep(path)",
    )
    .expect("plain sweep (no guide) regression failed");
}
