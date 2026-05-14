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
