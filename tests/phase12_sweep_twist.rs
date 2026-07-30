/// Phase 12 — per-section twist and scale in `sweep_sections`.
///
/// The Phase 6 variable-section sweep placed origin-centred profiles on the
/// spine but could not rotate or scale them per station, so a propeller blade
/// or tapered arm meant manual loft bookkeeping. `twist:` and `scale:` are
/// applied in the prelude wrapper by rotating/scaling each profile in its own
/// plane before the native sweep places it.
use rrcad::ruby::vm::MrubyVm;

/// Evaluate `code` and return the trimmed result string.
fn eval(code: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(code)
        .unwrap_or_else(|e| panic!("script failed: {e}\n--- script ---\n{code}"))
        .trim()
        .to_string()
}

/// Evaluate `code` expecting a numeric result.
fn eval_num(code: &str) -> f64 {
    let out = eval(code);
    out.parse()
        .unwrap_or_else(|_| panic!("expected a number, got: {out}"))
}

/// Evaluate `code` expecting failure, returning the error message.
fn eval_err(code: &str) -> String {
    let mut vm = MrubyVm::new();
    match vm.eval(code) {
        Ok(v) => panic!("expected failure, got: {v}\n--- script ---\n{code}"),
        Err(e) => e.to_string(),
    }
}

// A straight spine up the Z axis keeps the geometry checks simple: profiles
// stay parallel to the XY plane, so twist and scale read directly off the
// solid's bounding box and volume.
const STRAIGHT: &str = "spline_3d([[0, 0, 0], [0, 0, 10], [0, 0, 20]])";

// ---------------------------------------------------------------------------
// Behaviour without the new keywords is unchanged
// ---------------------------------------------------------------------------

#[test]
fn plain_sweep_still_works() {
    let code = format!("sweep_sections({STRAIGHT}, [circle(5), circle(3), circle(5)]).shape_type");
    assert_eq!(eval(&code), ":solid");
}

// ---------------------------------------------------------------------------
// twist:
// ---------------------------------------------------------------------------

#[test]
fn total_twist_rotates_the_last_section() {
    // A 10×2 bar twisted 90° over the sweep: the bottom section spans X, the
    // top section spans Y, so the solid's bounding box opens up to ~10 in
    // both directions.
    let code = format!(
        "s = sweep_sections({STRAIGHT}, [rect(10, 2), rect(10, 2), rect(10, 2)], twist: 90)
         [s.bounding_box[:dx], s.bounding_box[:dy]]"
    );
    let out = eval(&code);
    let dims: Vec<f64> = out
        .trim_matches(['[', ']'])
        .split(',')
        .map(|t| t.trim().parse().unwrap())
        .collect();
    assert!(
        dims[0] > 8.0 && dims[1] > 8.0,
        "90° twist should widen both axes, got {out}"
    );
}

#[test]
fn zero_twist_matches_no_twist() {
    let plain = format!("sweep_sections({STRAIGHT}, [rect(10, 2), rect(10, 2)]).volume");
    let zeroed = format!("sweep_sections({STRAIGHT}, [rect(10, 2), rect(10, 2)], twist: 0).volume");
    let a = eval_num(&plain);
    let b = eval_num(&zeroed);
    assert!(
        (a - b).abs() < 1e-6,
        "twist: 0 changed the sweep: {a} vs {b}"
    );
}

#[test]
fn twist_array_gives_each_section_its_own_angle() {
    let code = format!(
        "sweep_sections({STRAIGHT},
                        [rect(10, 2), rect(10, 2), rect(10, 2)],
                        twist: [0, 30, 45]).shape_type"
    );
    assert_eq!(eval(&code), ":solid");
}

#[test]
fn twist_array_length_must_match() {
    let code = format!("sweep_sections({STRAIGHT}, [circle(5), circle(5)], twist: [0, 10, 20])");
    let err = eval_err(&code);
    assert!(
        err.contains("one value per profile") && err.contains("3 for 2"),
        "unexpected error: {err}"
    );
}

#[test]
fn twist_rejects_non_numeric_entries() {
    let code = format!("sweep_sections({STRAIGHT}, [circle(5), circle(5)], twist: [0, :lots])");
    let err = eval_err(&code);
    assert!(err.contains("entry 1"), "unexpected error: {err}");
}

#[test]
fn twist_rejects_wrong_types() {
    let code = format!(r#"sweep_sections({STRAIGHT}, [circle(5), circle(5)], twist: "90")"#);
    let err = eval_err(&code);
    assert!(
        err.contains("Numeric or an Array"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// scale:
// ---------------------------------------------------------------------------

#[test]
fn end_scale_tapers_the_sweep() {
    // Circle r=5 swept 20 up Z, tapering linearly to half size: the result
    // sits between the full cylinder (~1571) and something implausibly small.
    // The analytic frustum is ~916 mm³; the swept surface blends, so accept
    // a band around it.
    let v = eval_num(&format!(
        "sweep_sections({STRAIGHT}, [circle(5), circle(5), circle(5)], scale: 0.5).volume"
    ));
    let full = eval_num(&format!(
        "sweep_sections({STRAIGHT}, [circle(5), circle(5), circle(5)]).volume"
    ));
    assert!(
        v > 700.0 && v < full * 0.75,
        "tapered volume out of range: {v} (full {full})"
    );
}

#[test]
fn scale_array_sets_each_section() {
    let code = format!(
        "s = sweep_sections({STRAIGHT}, [circle(5), circle(5), circle(5)], scale: [1, 0.8, 0.4])
         s.shape_type"
    );
    assert_eq!(eval(&code), ":solid");
}

#[test]
fn scale_must_be_positive() {
    let code = format!("sweep_sections({STRAIGHT}, [circle(5), circle(5)], scale: [1, 0])");
    let err = eval_err(&code);
    assert!(
        err.contains("scale: entry 1 must be > 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn scale_array_length_must_match() {
    let code = format!("sweep_sections({STRAIGHT}, [circle(5), circle(5)], scale: [1])");
    let err = eval_err(&code);
    assert!(err.contains("1 for 2"), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// The point of it all
// ---------------------------------------------------------------------------

#[test]
fn propeller_blade_in_one_call() {
    // Airfoil sections shrink and unwind toward the tip: the blade shape
    // that motivated the Phase 12 sweep upgrade.
    let code = r#"
      spine = spline_3d([[0, 0, 0], [0, 0, 40], [0, 0, 80]])
      section = airfoil(naca: "2412", chord: 24)
      blade = sweep_sections(spine, [section, section, section],
                             twist: [30, 20, 12], scale: [1.0, 0.75, 0.4])
      blade.shape_type
    "#;
    assert_eq!(eval(code), ":solid");
}

#[test]
fn profiles_are_not_mutated_by_the_keywords() {
    // The wrapper must transform copies: the same profile object used twice
    // keeps its original size after the sweep.
    let code = format!(
        "c = circle(5)
         sweep_sections({STRAIGHT}, [c, c, c], twist: 45, scale: 0.5)
         c.bounding_box[:dx]"
    );
    let dx = eval_num(&code);
    assert!((dx - 10.0).abs() < 1e-6, "profile was mutated: dx = {dx}");
}

#[test]
fn too_few_profiles_still_raises() {
    let code = format!("sweep_sections({STRAIGHT}, [circle(5)], twist: 10)");
    let err = eval_err(&code);
    assert!(
        err.contains("at least 2 profiles"),
        "unexpected error: {err}"
    );
}
