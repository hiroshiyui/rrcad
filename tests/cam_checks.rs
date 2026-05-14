// Phase 10 — CAM / 3-D printing checks.
//
// Tests for top-level pure-Ruby DSL helpers:
//   mass_estimate(part, density:)       → Float (grams)
//   print_volume_check(part, x:, y:, z:) → Hash with :fits and overflow info

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// mass_estimate
// ---------------------------------------------------------------------------

#[test]
fn mass_estimate_pla_default_density() {
    // 100×100×10 mm = 100 000 mm³ = 100 cm³. At PLA density 1.24 g/cm³
    // ⇒ ~124 g.
    let mut vm = MrubyVm::new();
    let result = vm.eval("mass_estimate(box(100, 100, 10))").unwrap();
    let grams: f64 = result.trim().parse().expect("number");
    assert!(
        (grams - 124.0).abs() < 0.5,
        "expected ≈124 g at PLA default, got {grams}"
    );
}

#[test]
fn mass_estimate_custom_density() {
    // 50×50×10 = 25 000 mm³ = 25 cm³. With density 7.85 (steel) ⇒ 196.25 g.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("mass_estimate(box(50, 50, 10), density: 7.85)")
        .unwrap();
    let grams: f64 = result.trim().parse().expect("number");
    assert!(
        (grams - 196.25).abs() < 0.5,
        "expected ≈196.25 g for steel, got {grams}"
    );
}

#[test]
fn mass_estimate_rejects_non_positive_density() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("mass_estimate(box(10, 10, 10), density: 0)")
        .unwrap_err();
    assert!(err.contains("must be > 0"), "got: {err}");
}

#[test]
fn mass_estimate_rejects_non_numeric_density() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("mass_estimate(box(10, 10, 10), density: :pla)")
        .unwrap_err();
    assert!(err.contains("must be > 0"), "got: {err}");
}

// ---------------------------------------------------------------------------
// print_volume_check
// ---------------------------------------------------------------------------

#[test]
fn print_volume_check_reports_fit() {
    // 50×50×30 part inside a 220×220×250 bed ⇒ fits.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("print_volume_check(box(50, 50, 30), x: 220, y: 220, z: 250)[:fits]")
        .unwrap();
    assert_eq!(result.trim(), "true");
}

#[test]
fn print_volume_check_detects_overflow() {
    // 300×80×30 part on a 220×220×250 bed ⇒ x overflows by 80, y/z fit.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "r = print_volume_check(box(300, 80, 30), x: 220, y: 220, z: 250)
             \"#{r[:fits]} #{r[:overflow_x]} #{r[:overflow_y]} #{r[:overflow_z]}\"",
        )
        .unwrap();
    let stripped = result.trim().trim_matches('"');
    let parts: Vec<&str> = stripped.split_whitespace().collect();
    assert_eq!(parts[0], "false");
    let ox: f64 = parts[1].parse().expect("ox number");
    let oy: f64 = parts[2].parse().expect("oy number");
    let oz: f64 = parts[3].parse().expect("oz number");
    assert!((ox - 80.0).abs() < 0.1, "expected x overflow 80, got {ox}");
    assert!(oy.abs() < 0.1, "expected y overflow 0, got {oy}");
    assert!(oz.abs() < 0.1, "expected z overflow 0, got {oz}");
}

#[test]
fn print_volume_check_reports_extents() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("print_volume_check(box(40, 60, 80), x: 220, y: 220, z: 250)[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("number");
    assert!((dx - 40.0).abs() < 0.01, "expected dx 40, got {dx}");
}

#[test]
fn print_volume_check_rejects_non_positive_bed() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("print_volume_check(box(10, 10, 10), x: 220, y: 0, z: 250)")
        .unwrap_err();
    assert!(
        err.contains("y must be > 0"),
        "expected y validation error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Shape#normal (face outward normal)
// ---------------------------------------------------------------------------

#[test]
fn face_normal_top_face_of_box_points_up() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10, 10, 10).faces(:top).first.normal[2]")
        .unwrap();
    let nz: f64 = result.trim().parse().expect("number");
    assert!((nz - 1.0).abs() < 1e-6, "expected nz ≈ 1, got {nz}");
}

#[test]
fn face_normal_bottom_face_of_box_points_down() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10, 10, 10).faces(:bottom).first.normal[2]")
        .unwrap();
    let nz: f64 = result.trim().parse().expect("number");
    assert!((nz - (-1.0)).abs() < 1e-6, "expected nz ≈ -1, got {nz}");
}

#[test]
fn face_normal_rejects_non_face() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("box(10, 10, 10).normal").unwrap_err();
    assert!(
        err.contains("not a face"),
        "expected 'not a face' error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// overhang_faces
// ---------------------------------------------------------------------------

#[test]
fn overhang_faces_empty_for_plain_box() {
    // A solid axis-aligned box has only the bottom face pointing −Z, but the
    // bottom face sits on the build plate — overhang_faces should still
    // flag it (it's downward-facing). However the default threshold is 45°,
    // and a fully downward face is at 90°, so it IS flagged.
    let mut vm = MrubyVm::new();
    let result = vm.eval("overhang_faces(box(10, 10, 10)).length").unwrap();
    assert_eq!(
        result.trim(),
        "1",
        "expected 1 overhang face (the bottom), got {result}"
    );
}

#[test]
fn overhang_faces_finds_overhang_on_t_shape() {
    // A T-shaped solid: a base, with a wider cap on top.  The cap's
    // underside face points −Z and is an overhang.  We expect to find at
    // least one overhanging face (the cap's underside, plus the base
    // bottom).
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "base = box(10, 10, 10)
             cap  = box(30, 10, 5).translate(-10, 0, 10)
             part = base.fuse(cap)
             overhang_faces(part).length",
        )
        .unwrap();
    let n: i32 = result.trim().parse().expect("number");
    assert!(n >= 2, "expected ≥2 overhang faces on T-shape, got {n}");
}

#[test]
fn overhang_faces_threshold_filters_results() {
    // At max_angle_deg: 89 (only steeply downward faces count), a box's
    // bottom is at 90° from horizontal → still flagged. At 90, only a
    // perfectly downward face exceeds → still 1. Verify the threshold path.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("overhang_faces(box(10, 10, 10), max_angle_deg: 89).length")
        .unwrap();
    assert_eq!(result.trim(), "1", "expected 1 overhang, got {result}");
}

#[test]
fn overhang_faces_rejects_out_of_range_angle() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("overhang_faces(box(10, 10, 10), max_angle_deg: -1)")
        .unwrap_err();
    assert!(
        err.contains("must be in [0, 90]"),
        "expected range error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// draft_faces
// ---------------------------------------------------------------------------

#[test]
fn draft_faces_flags_all_four_walls_of_a_plain_box() {
    // A vertical-walled box has 4 side faces with 0° draft.  Top and bottom
    // are perpendicular to the pull axis (90° draft) and are NOT flagged.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("draft_faces(box(10, 10, 10), min_draft_deg: 1).length")
        .unwrap();
    assert_eq!(
        result.trim(),
        "4",
        "expected 4 zero-draft side walls, got {result}"
    );
}

#[test]
fn draft_faces_excludes_tapered_walls_above_threshold() {
    // Extrude a unit square with a 5° draft.  Side faces tilt outward by 5°,
    // so |n·z| = sin(5°) ≈ 0.087.  With min_draft_deg: 1, all sides pass
    // (5° > 1°) → 0 flagged faces.  With min_draft_deg: 10, sides fail
    // (5° < 10°) → 4 flagged.
    let mut vm = MrubyVm::new();
    let ok = vm
        .eval(
            "part = rect(20, 20).extrude(10, draft: 5)
             draft_faces(part, min_draft_deg: 1).length",
        )
        .unwrap();
    assert_eq!(ok.trim(), "0", "5° drafted box: no flags at 1° threshold");

    let mut vm2 = MrubyVm::new();
    let strict = vm2
        .eval(
            "part = rect(20, 20).extrude(10, draft: 5)
             draft_faces(part, min_draft_deg: 10).length",
        )
        .unwrap();
    assert_eq!(
        strict.trim(),
        "4",
        "5° drafted box: 4 flags at 10° threshold"
    );
}

#[test]
fn draft_faces_accepts_alternate_pull_axis() {
    // Same plain box, but pulled along +X instead of +Z.  Then the two
    // X-facing faces have 90° draft (perpendicular to pull) and the four
    // Y/Z-facing faces have 0° → 4 flagged.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("draft_faces(box(10, 10, 10), axis: [1, 0, 0], min_draft_deg: 1).length")
        .unwrap();
    assert_eq!(
        result.trim(),
        "4",
        "expected 4 side walls along +X pull, got {result}"
    );
}

#[test]
fn draft_faces_rejects_zero_axis() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("draft_faces(box(10, 10, 10), axis: [0, 0, 0])")
        .unwrap_err();
    assert!(
        err.contains("non-zero"),
        "expected non-zero error, got: {err}"
    );
}

#[test]
fn draft_faces_rejects_bad_axis_shape() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("draft_faces(box(10, 10, 10), axis: [0, 0])")
        .unwrap_err();
    assert!(
        err.contains("3-element"),
        "expected 3-element error, got: {err}"
    );
}

#[test]
fn draft_faces_rejects_out_of_range_threshold() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("draft_faces(box(10, 10, 10), min_draft_deg: 91)")
        .unwrap_err();
    assert!(
        err.contains("must be in [0, 90]"),
        "expected range error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Shape#cylinder_axis & hole_axes
// ---------------------------------------------------------------------------

#[test]
fn cylinder_axis_reports_axis_and_radius() {
    // cylinder(r=5, h=10) builds along +Z; its side face is cylindrical.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "info = cylinder(5, 10).faces(:side).first.cylinder_axis
             \"#{info[:axis][0]} #{info[:axis][1]} #{info[:axis][2]} #{info[:radius]}\"",
        )
        .unwrap();
    let stripped = result.trim().trim_matches('"');
    let parts: Vec<f64> = stripped
        .split_whitespace()
        .map(|s| s.parse().expect("number"))
        .collect();
    assert!(parts[0].abs() < 1e-6, "ax ≈ 0, got {}", parts[0]);
    assert!(parts[1].abs() < 1e-6, "ay ≈ 0, got {}", parts[1]);
    assert!(
        (parts[2].abs() - 1.0).abs() < 1e-6,
        "|az| ≈ 1, got {}",
        parts[2]
    );
    assert!(
        (parts[3] - 5.0).abs() < 0.01,
        "radius ≈ 5, got {}",
        parts[3]
    );
}

#[test]
fn cylinder_axis_rejects_non_cylindrical_face() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("box(10, 10, 10).faces(:top).first.cylinder_axis")
        .unwrap_err();
    assert!(
        err.contains("not a cylindrical"),
        "expected cylindricity error, got: {err}"
    );
}

#[test]
fn cylinder_axis_rejects_non_face() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("box(10, 10, 10).cylinder_axis").unwrap_err();
    assert!(
        err.contains("not a face"),
        "expected face error, got: {err}"
    );
}

#[test]
fn hole_axes_finds_vertical_hole_in_plate() {
    // A plate with a vertical (+Z) cylindrical bore drilled through it.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(40, 40, 10)
             drill = cylinder(3, 12).translate(20, 20, -1)
             part  = plate.cut(drill)
             hole_axes(part).length",
        )
        .unwrap();
    let n: i32 = result.trim().parse().expect("number");
    assert!(n >= 1, "expected at least one cylindrical face, got {n}");
}

#[test]
fn hole_axes_filters_by_orientation() {
    // Same drilled plate as above, but filter to vertical only — should find
    // the bore (Z-axis). Filter to horizontal — none.
    let mut vm = MrubyVm::new();
    let vert = vm
        .eval(
            "plate = box(40, 40, 10)
             drill = cylinder(3, 12).translate(20, 20, -1)
             hole_axes(plate.cut(drill), orientation: :vertical).length",
        )
        .unwrap();
    assert!(
        vert.trim().parse::<i32>().unwrap() >= 1,
        "expected ≥1 vertical bore, got {vert}"
    );

    let mut vm2 = MrubyVm::new();
    let horiz = vm2
        .eval(
            "plate = box(40, 40, 10)
             drill = cylinder(3, 12).translate(20, 20, -1)
             hole_axes(plate.cut(drill), orientation: :horizontal).length",
        )
        .unwrap();
    assert_eq!(
        horiz.trim(),
        "0",
        "expected 0 horizontal bores, got {horiz}"
    );
}

#[test]
fn hole_axes_reports_radius() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(40, 40, 10)
             drill = cylinder(3, 12).translate(20, 20, -1)
             part  = plate.cut(drill)
             hole_axes(part).first[:radius]",
        )
        .unwrap();
    let r: f64 = result.trim().parse().expect("number");
    assert!((r - 3.0).abs() < 0.01, "expected radius ≈ 3, got {r}");
}

#[test]
fn hole_axes_empty_for_plain_box() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("hole_axes(box(10, 10, 10)).length").unwrap();
    assert_eq!(result.trim(), "0", "no cylinders on a plain box");
}

#[test]
fn hole_axes_rejects_bad_orientation() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("hole_axes(box(10, 10, 10), orientation: :diagonal)")
        .unwrap_err();
    assert!(
        err.contains("orientation"),
        "expected orientation error, got: {err}"
    );
}
