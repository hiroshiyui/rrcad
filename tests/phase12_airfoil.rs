/// Phase 12 — airfoil profile primitive.
///
/// `airfoil(naca:/coordinates:/dat:)` returns a closed aerofoil Face in the
/// XY plane, chord along +X, built as two interpolated BSpline segments so
/// the section stays smooth through extrude and loft while the trailing edge
/// stays a sharp corner. The section primitive for propeller blades.
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

// A small hand-made Selig-ordered section: TE → upper → LE → lower → TE.
// Coarse but a legal airfoil shape (a lens with a sharp TE).
const SELIG_POINTS: &str = "[[1.0, 0.0], [0.75, 0.04], [0.5, 0.06], [0.25, 0.05], [0.05, 0.02], \
     [0.0, 0.0], [0.05, -0.02], [0.25, -0.05], [0.5, -0.06], [0.75, -0.04], [1.0, 0.0]]";

// ---------------------------------------------------------------------------
// NACA generation
// ---------------------------------------------------------------------------

#[test]
fn naca_airfoil_is_a_face() {
    assert_eq!(
        eval(r#"airfoil(naca: "2412", chord: 20).shape_type"#),
        ":face"
    );
}

#[test]
fn naca_code_accepts_an_integer() {
    assert_eq!(eval("airfoil(naca: 2412, chord: 20).shape_type"), ":face");
}

#[test]
fn naca_chord_sets_the_x_extent_exactly() {
    let dx = eval_num(r#"airfoil(naca: "0012", chord: 25).bounding_box[:dx]"#);
    assert!((dx - 25.0).abs() < 1e-6, "chord should be exact, got {dx}");
}

#[test]
fn naca_leading_edge_sits_at_the_origin() {
    // The sampled LE lands at x=0 exactly; the interpolated spline may bulge
    // a hair (~0.01% of chord) past it on a cambered nose.
    let x = eval_num(r#"airfoil(naca: "2412", chord: 20).bounding_box[:x]"#);
    assert!(x.abs() < 0.01, "LE should be at x≈0, got {x}");
}

#[test]
fn symmetric_naca_is_symmetric_about_y() {
    // NACA 0012 has no camber: centroid on the chord line, bbox split evenly.
    let cy = eval_num(r#"airfoil(naca: "0012", chord: 20).centroid[1]"#);
    assert!(cy.abs() < 1e-6, "symmetric section centroid y: {cy}");
    let y = eval_num(r#"airfoil(naca: "0012", chord: 20).bounding_box[:y]"#);
    let dy = eval_num(r#"airfoil(naca: "0012", chord: 20).bounding_box[:dy]"#);
    assert!(
        (y + dy / 2.0).abs() < 1e-3,
        "bbox should straddle y=0: y={y}, dy={dy}"
    );
}

#[test]
fn naca_0012_thickness_is_twelve_percent() {
    // Max thickness of a NACA 00xx is xx% of chord; sampled splines land
    // within a fraction of a percent.
    let dy = eval_num(r#"airfoil(naca: "0012", chord: 100).bounding_box[:dy]"#);
    assert!(
        (dy - 12.0).abs() < 0.1,
        "0012 at chord 100 should be ~12 thick, got {dy}"
    );
}

#[test]
fn cambered_naca_bulges_upward() {
    let cy = eval_num(r#"airfoil(naca: "2412", chord: 20).centroid[1]"#);
    assert!(
        cy > 0.05,
        "2412 centroid should sit above the chord line: {cy}"
    );
}

#[test]
fn naca_airfoil_extrudes_to_a_solid() {
    assert_eq!(
        eval(r#"airfoil(naca: "2412", chord: 20).extrude(5).shape_type"#),
        ":solid"
    );
    // Section area of a 4-digit foil is ~0.685·t·c²: 0.685·0.12·400 ≈ 32.9,
    // times height 5 ≈ 164 mm³. Accept a generous band.
    let v = eval_num(r#"airfoil(naca: "2412", chord: 20).extrude(5).volume"#);
    assert!(
        (140.0..190.0).contains(&v),
        "extruded 2412 volume out of range: {v}"
    );
}

#[test]
fn naca_airfoil_is_valid_geometry() {
    assert_eq!(eval(r#"airfoil(naca: "2412", chord: 20).validate"#), ":ok");
}

#[test]
fn airfoils_loft_into_a_blade() {
    // The whole point of the primitive: sections placed in 3-D loft into a
    // twisted blade solid.
    let code = r#"
      root = airfoil(naca: "2412", chord: 24)
      tip  = airfoil(naca: "2412", chord: 12)
               .rotate(1, 0, 0, 15)
               .translate(0, 0, 60)
      loft([root, tip]).shape_type
    "#;
    assert_eq!(eval(code), ":solid");
}

#[test]
fn samples_controls_smoothness_and_validates() {
    assert_eq!(
        eval(r#"airfoil(naca: "0012", chord: 20, samples: 80).shape_type"#),
        ":face"
    );
    let err = eval_err(r#"airfoil(naca: "0012", chord: 20, samples: 4)"#);
    assert!(err.contains("samples"), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// NACA validation
// ---------------------------------------------------------------------------

#[test]
fn naca_requires_a_chord() {
    let err = eval_err(r#"airfoil(naca: "2412")"#);
    assert!(
        err.contains("chord: is required"),
        "unexpected error: {err}"
    );
}

#[test]
fn naca_code_must_be_four_digits() {
    for bad in [r#""241""#, r#""24123""#, r#""24a2""#, r#""naca""#] {
        let err = eval_err(&format!("airfoil(naca: {bad}, chord: 20)"));
        assert!(err.contains("four-digit"), "for {bad}: {err}");
    }
}

#[test]
fn naca_zero_thickness_is_rejected() {
    let err = eval_err(r#"airfoil(naca: "2400", chord: 20)"#);
    assert!(err.contains("zero thickness"), "unexpected error: {err}");
}

#[test]
fn naca_camber_without_position_is_rejected() {
    // m > 0 with p == 0 divides by zero in the camber line; say so up front.
    let err = eval_err(r#"airfoil(naca: "2012", chord: 20)"#);
    assert!(err.contains("malformed"), "unexpected error: {err}");
}

#[test]
fn exactly_one_source_is_required() {
    let err = eval_err("airfoil(chord: 20)");
    assert!(err.contains("exactly one"), "unexpected error: {err}");
    let err = eval_err(&format!(
        r#"airfoil(naca: "2412", coordinates: {SELIG_POINTS}, chord: 20)"#
    ));
    assert!(
        err.contains("naca and coordinates"),
        "unexpected error: {err}"
    );
}

#[test]
fn chord_must_be_positive() {
    let err = eval_err(r#"airfoil(naca: "2412", chord: -5)"#);
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// coordinates: — Selig-ordered point lists
// ---------------------------------------------------------------------------

#[test]
fn coordinates_build_a_face() {
    assert_eq!(
        eval(&format!("airfoil(coordinates: {SELIG_POINTS}).shape_type")),
        ":face"
    );
}

#[test]
fn coordinates_are_used_as_is_without_chord() {
    let dx = eval_num(&format!(
        "airfoil(coordinates: {SELIG_POINTS}).bounding_box[:dx]"
    ));
    assert!((dx - 1.0).abs() < 1e-6, "unit-chord data left alone: {dx}");
}

#[test]
fn coordinates_scale_to_the_requested_chord() {
    let dx = eval_num(&format!(
        "airfoil(coordinates: {SELIG_POINTS}, chord: 30).bounding_box[:dx]"
    ));
    assert!((dx - 30.0).abs() < 1e-6, "expected chord 30, got {dx}");
}

#[test]
fn coordinates_reject_malformed_points() {
    let err = eval_err("airfoil(coordinates: [[1, 0], [0.5, 0.1], :nope, [0, 0], [0.5, -0.1]])");
    assert!(err.contains("entry 2"), "unexpected error: {err}");
}

#[test]
fn coordinates_reject_non_selig_order() {
    // Leading edge (minimum x) first means the list is one surface LE→TE,
    // not a Selig ring.
    let err =
        eval_err("airfoil(coordinates: [[0, 0], [0.25, 0.05], [0.5, 0.06], [0.75, 0.04], [1, 0]])");
    assert!(err.contains("Selig order"), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// dat: — Selig-format file content
// ---------------------------------------------------------------------------

#[test]
fn dat_with_name_line_builds_a_face() {
    let code = r#"
      dat = "LENS TEST SECTION\n" \
            "1.0 0.0\n0.75 0.04\n0.5 0.06\n0.25 0.05\n0.05 0.02\n" \
            "0.0 0.0\n" \
            "0.05 -0.02\n0.25 -0.05\n0.5 -0.06\n0.75 -0.04\n1.0 0.0\n"
      airfoil(dat: dat, chord: 20).shape_type
    "#;
    assert_eq!(eval(code), ":face");
}

#[test]
fn dat_with_blunt_trailing_edge_still_closes() {
    // First and last points differ: the profile builder closes the gap with
    // a straight TE base, and the face still extrudes to a solid.
    let code = r#"
      dat = "1.0 0.005\n0.75 0.04\n0.5 0.06\n0.25 0.05\n0.05 0.02\n" \
            "0.0 0.0\n" \
            "0.05 -0.02\n0.25 -0.05\n0.5 -0.06\n0.75 -0.04\n1.0 -0.005\n"
      airfoil(dat: dat, chord: 20).extrude(3).shape_type
    "#;
    assert_eq!(eval(code), ":solid");
}

#[test]
fn dat_duplicate_leading_edge_point_is_deduped() {
    // Some published files repeat the LE point where the surfaces meet;
    // GeomAPI_Interpolate would refuse the duplicate.
    let code = r#"
      dat = "1.0 0.0\n0.75 0.04\n0.5 0.06\n0.25 0.05\n0.05 0.02\n" \
            "0.0 0.0\n0.0 0.0\n" \
            "0.05 -0.02\n0.25 -0.05\n0.5 -0.06\n0.75 -0.04\n1.0 0.0\n"
      airfoil(dat: dat).shape_type
    "#;
    assert_eq!(eval(code), ":face");
}

#[test]
fn dat_rejects_garbage_after_the_points_start() {
    let code = r#"airfoil(dat: "1.0 0.0\n0.5 0.06\nnot a point\n0.0 0.0\n0.5 -0.06\n1.0 0.0")"#;
    let err = eval_err(code);
    assert!(err.contains("line 3"), "unexpected error: {err}");
}

#[test]
fn dat_rejects_lednicer_format() {
    let code = r#"airfoil(dat: "EXAMPLE\n61. 61.\n0.0 0.0\n0.5 0.06\n1.0 0.0")"#;
    let err = eval_err(code);
    assert!(err.contains("Lednicer"), "unexpected error: {err}");
}

#[test]
fn dat_rejects_too_few_points() {
    let err = eval_err(r#"airfoil(dat: "TINY\n1.0 0.0\n0.0 0.0\n1.0 0.0")"#);
    assert!(err.contains("coordinate pairs"), "unexpected error: {err}");
}
