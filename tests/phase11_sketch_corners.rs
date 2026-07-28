/// Phase 11 Track A — sketch corner modifiers (`fillet` / `chamfer`).
///
/// These exercise the DSL-level corner rounding and bevelling that happens
/// while the 2-D profile is built, as opposed to `Shape#fillet`, which rounds
/// edges of an existing 3-D solid.
use rrcad::ruby::vm::MrubyVm;

/// Evaluate `code` and return the trimmed result string.
fn eval(code: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(code)
        .unwrap_or_else(|e| panic!("script failed: {e}\n--- script ---\n{code}"))
        .trim()
        .to_string()
}

/// Evaluate `code` that yields a Ruby String, stripping the quotes that
/// `inspect` adds around the returned value.
fn eval_str(code: &str) -> String {
    eval(code).trim_matches('"').to_string()
}

/// Evaluate `code` expecting failure, returning the error message.
fn eval_err(code: &str) -> String {
    let mut vm = MrubyVm::new();
    match vm.eval(code) {
        Ok(v) => panic!("expected failure, got: {v}\n--- script ---\n{code}"),
        Err(e) => e.to_string(),
    }
}

/// A 40 x 20 rectangle sketch with a corner modifier applied to `:a`
/// (the bottom-left corner), extruded 5 mm.
fn rect_with_modifier(modifier: &str) -> String {
    format!(
        r#"
        sk = sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 40.0, 0.0)
          c = point(:c, 40.0, 20.0)
          d = point(:d, 0.0, 20.0)
          line a, b
          line b, c
          line c, d
          line d, a
          {modifier}
        end
        sk.extrude(5.0)
        "#
    )
}

// ---------------------------------------------------------------------------
// Fillet
// ---------------------------------------------------------------------------

#[test]
fn sketch_fillet_removes_material_from_the_corner() {
    // A filleted corner must cut area off the square corner, so the extruded
    // volume is strictly less than the unmodified 40 x 20 x 5 = 4000.
    let volume = eval(&format!("{}.volume", rect_with_modifier("fillet a, 5.0")));
    let volume: f64 = volume.parse().expect("volume should be numeric");

    assert!(
        volume < 4000.0,
        "filleted profile should lose material, got {volume}"
    );
    // A quarter-circle of r=5 removes (25 - 25*pi/4) * 5 ≈ 26.8 mm³.
    let expected = 4000.0 - (25.0 - 25.0 * std::f64::consts::PI / 4.0) * 5.0;
    assert!(
        (volume - expected).abs() < 2.0,
        "expected ≈{expected:.1}, got {volume}"
    );
}

#[test]
fn sketch_fillet_produces_a_valid_solid() {
    let status = eval_str(&format!(
        "{}.validate.to_s",
        rect_with_modifier("fillet a, 4.0")
    ));
    assert_eq!(
        status, "ok",
        "filleted profile should extrude to a valid solid"
    );
}

#[test]
fn sketch_fillet_accepts_unit_values() {
    let volume = eval(&format!(
        "{}.volume",
        rect_with_modifier("fillet a, 3.0.mm")
    ));
    assert!(
        volume.parse::<f64>().expect("numeric volume") < 4000.0,
        "unit-typed radius should behave like a plain number"
    );
}

#[test]
fn multiple_corners_can_be_filleted() {
    let script = rect_with_modifier("fillet a, 3.0\n          fillet c, 3.0");
    let volume: f64 = eval(&format!("{script}.volume"))
        .parse()
        .expect("numeric volume");
    let single: f64 = eval(&format!("{}.volume", rect_with_modifier("fillet a, 3.0")))
        .parse()
        .expect("numeric volume");

    assert!(
        volume < single,
        "two fillets should remove more than one ({volume} vs {single})"
    );
}

// ---------------------------------------------------------------------------
// Chamfer
// ---------------------------------------------------------------------------

#[test]
fn sketch_chamfer_removes_a_triangular_corner() {
    let volume: f64 = eval(&format!("{}.volume", rect_with_modifier("chamfer a, 6.0")))
        .parse()
        .expect("numeric volume");

    // A 6 mm setback on both edges removes a right triangle of area 18.
    let expected = 4000.0 - 18.0 * 5.0;
    assert!(
        (volume - expected).abs() < 0.5,
        "expected ≈{expected}, got {volume}"
    );
}

#[test]
fn chamfer_and_fillet_can_be_mixed_in_one_sketch() {
    let script = rect_with_modifier("fillet a, 4.0\n          chamfer b, 4.0");
    let status = eval_str(&format!("{script}.validate.to_s"));
    assert_eq!(
        status, "ok",
        "mixed modifiers should extrude to a valid solid"
    );
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[test]
fn oversized_modifier_is_rejected_with_a_clear_message() {
    // A 30 mm setback cannot fit on the 20 mm tall side.
    let err = eval_err(&rect_with_modifier("chamfer a, 30.0"));
    assert!(
        err.contains("too large"),
        "error should explain the size problem: {err}"
    );
    assert!(
        err.contains(":a"),
        "error should name the offending corner: {err}"
    );
}

#[test]
fn overlapping_modifiers_on_one_segment_are_rejected() {
    // Both ends of the 20 mm side set back 15 mm: 30 > 20.
    let err = eval_err(&rect_with_modifier(
        "chamfer a, 15.0\n          chamfer d, 15.0",
    ));
    assert!(
        err.contains("overlap"),
        "error should report the overlap: {err}"
    );
}

#[test]
fn two_modifiers_on_the_same_corner_are_rejected() {
    let err = eval_err(&rect_with_modifier(
        "fillet a, 3.0\n          chamfer a, 3.0",
    ));
    assert!(
        err.contains("one corner modifier"),
        "error should reject the duplicate: {err}"
    );
}

#[test]
fn non_positive_radius_is_rejected() {
    let err = eval_err(&rect_with_modifier("fillet a, 0.0"));
    assert!(
        err.contains("must be > 0"),
        "error should reject zero: {err}"
    );
}

#[test]
fn modifier_on_a_non_corner_point_is_rejected() {
    // `:loose` is a sketch point that is not part of the closed loop.
    let script = r#"
        sk = sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 40.0, 0.0)
          c = point(:c, 40.0, 20.0)
          d = point(:d, 0.0, 20.0)
          loose = point(:loose, 5.0, 5.0)
          line a, b
          line b, c
          line c, d
          line d, a
          fillet loose, 2.0
        end
        sk.extrude(5.0)
    "#;
    let err = eval_err(script);
    assert!(
        err.contains("not a corner"),
        "error should explain the target is not a corner: {err}"
    );
}
