/// Phase 11 Track A — profile offset.
///
/// `Shape#offset_2d` grows or shrinks a 2-D profile in its own plane, keeping
/// every edge parallel to where it started. The result is a Face, so an offset
/// profile can be extruded, padded, or pocketed like any other — before this
/// it came back as a bare Wire, which extruded into an open shell rather than
/// a solid. `offset` inside `sketch do … end` is the sketch-level spelling.
use rrcad::ruby::vm::MrubyVm;
use std::f64::consts::PI;

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

/// Assert two areas/volumes match to within a tolerance that ignores OCCT's
/// curve discretisation noise.
fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() < 1.0e-6,
        "{what}: expected {expected}, got {actual}"
    );
}

/// The planar area of a profile, measured by extruding it 1 mm.
fn area_of(profile: &str) -> f64 {
    eval_num(&format!("({profile}).extrude(1.0).volume"))
}

// ---------------------------------------------------------------------------
// Shape#offset_2d on a simple profile
// ---------------------------------------------------------------------------

#[test]
fn an_outward_offset_grows_the_profile_and_rounds_its_corners() {
    // A 40 x 20 rectangle grown by 3 becomes 46 x 26 with quarter-round
    // corners of r = 3: 46 * 26 - (36 - 9pi) = 1160 + 9pi.
    assert_close(
        area_of("rect(40, 20).offset_2d(3)"),
        1160.0 + 9.0 * PI,
        "outward offset area",
    );
}

#[test]
fn an_inward_offset_shrinks_the_profile_and_keeps_corners_square() {
    // Shrinking pulls the edges in without rounding: 34 x 14 = 476.
    assert_close(
        area_of("rect(40, 20).offset_2d(-3)"),
        476.0,
        "inward offset area",
    );
}

#[test]
fn an_offset_profile_extrudes_into_a_solid() {
    // Regression: the offsetter returns a Wire, which extrudes into an open
    // shell whose measured volume is meaningless (it came out negative for an
    // inward offset). Rebuilding a Face keeps the result a real profile.
    let volume = eval_num("rect(40, 20).offset_2d(-3).extrude(5).volume");
    assert_close(volume, 2380.0, "inward offset solid volume");

    let grown = eval_num("rect(40, 20).offset_2d(3).extrude(5).volume");
    assert_close(grown, (1160.0 + 9.0 * PI) * 5.0, "outward offset volume");
}

#[test]
fn a_circular_profile_offsets_to_a_concentric_circle() {
    assert_close(
        area_of("circle(10).offset_2d(2)"),
        PI * 144.0,
        "grown circle",
    );
    assert_close(
        area_of("circle(10).offset_2d(-2)"),
        PI * 64.0,
        "shrunk circle",
    );
}

#[test]
fn an_offset_profile_can_be_padded_onto_a_solid() {
    // The offset result has to behave as a profile everywhere, not just under
    // `extrude` — `pad` repositions it onto a face of an existing solid.
    let volume = eval_num(
        r#"
        base = box(60, 40, 5)
        pad = base.pad(:top, height: 4.0) { rect(20, 10).offset_2d(2) }
        pad.volume
        "#,
    );
    // 60 * 40 * 5 = 12000 plus a 24 x 14 round-cornered pad 4 mm tall.
    let profile = 24.0 * 14.0 - (16.0 - 4.0 * PI);
    assert_close(volume, 12000.0 + profile * 4.0, "padded volume");
}

// ---------------------------------------------------------------------------
// Profiles with holes
// ---------------------------------------------------------------------------

#[test]
fn growing_a_holed_profile_shrinks_its_hole() {
    // A 40 x 20 plate with a central r = 5 hole, grown by 2: the outer
    // boundary reaches 44 x 24 with r = 2 corners while the hole closes to
    // r = 3, because both boundaries move by 2 into the empty space.
    let plate = "rect(40, 20).cut(circle(5).translate(20, 10, 0))";
    assert_close(
        area_of(&format!("{plate}.offset_2d(2)")),
        1040.0 + 4.0 * PI - 9.0 * PI,
        "grown holed profile",
    );
}

#[test]
fn shrinking_a_holed_profile_grows_its_hole() {
    let plate = "rect(40, 20).cut(circle(5).translate(20, 10, 0))";
    assert_close(
        area_of(&format!("{plate}.offset_2d(-2)")),
        576.0 - 49.0 * PI,
        "shrunk holed profile",
    );
}

#[test]
fn an_annulus_offsets_in_both_directions() {
    // OCCT cannot offset an all-circular annulus as a single face, so this
    // exercises the per-wire fallback — including its sign flip, without which
    // the hole moves the same way as the outer boundary.
    let annulus = "circle(10).cut(circle(4))";
    assert_close(
        area_of(&format!("{annulus}.offset_2d(1)")),
        PI * (121.0 - 9.0),
        "grown annulus",
    );
    assert_close(
        area_of(&format!("{annulus}.offset_2d(-1)")),
        PI * (81.0 - 25.0),
        "shrunk annulus",
    );
}

// ---------------------------------------------------------------------------
// Sketch-level offset
// ---------------------------------------------------------------------------

/// A 40 x 20 constrained sketch with `body` appended inside the block.
fn rect_sketch(body: &str) -> String {
    format!(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 40.0, 0.0)
          c = point(:c, 40.0, 20.0)
          d = point(:d, 0.0, 20.0)
          line a, b
          line b, c
          line c, d
          line d, a
          {body}
        end
        "#
    )
}

#[test]
fn a_sketch_can_offset_its_own_profile() {
    assert_close(
        area_of(&rect_sketch("offset 3.0")),
        1160.0 + 9.0 * PI,
        "sketch offset outward",
    );
    assert_close(
        area_of(&rect_sketch("offset(-3.0)")),
        476.0,
        "sketch offset inward",
    );
}

#[test]
fn a_sketch_offset_accepts_unit_values() {
    assert_close(
        area_of(&rect_sketch("offset 3.mm")),
        1160.0 + 9.0 * PI,
        "sketch offset in mm",
    );
}

#[test]
fn a_sketch_offset_applies_to_a_circle_profile() {
    // The offset is the last step of building the profile, so it reaches the
    // `circle_at` / `arc_at` / `slot_between` profiles too, not just polygons.
    let area = area_of(
        r#"
        sketch do
          ctr = point(:ctr, 0.0, 0.0)
          circle_at ctr, 10.0
          offset 2.0
        end
        "#,
    );
    assert_close(area, PI * 144.0, "offset circle_at profile");
}

#[test]
fn a_sketch_offset_runs_after_corner_modifiers_and_segment_edits() {
    // fillet and trim shape the polygon; the offset then grows whatever they
    // produced, so the result must differ from offsetting the plain rectangle.
    let combined = area_of(&rect_sketch(
        r#"
        fillet a, 5.0
        trim c, d, by: 10.0
        offset 2.0
        "#,
    ));
    let plain = area_of(&rect_sketch("offset 2.0"));
    assert!(
        combined < plain,
        "the filleted and trimmed profile should be smaller than the plain one: {combined} vs {plain}"
    );
    assert!(combined > 0.0, "expected a real profile, got {combined}");
}

#[test]
fn an_offset_sketch_still_reports_clean_diagnostics() {
    let status = eval(
        &rect_sketch("offset 2.0")
            .replace("sketch do", "sketch(diagnostics: true) do")
            .replace("end\n        ", "end.sketch_diagnostics[:status].inspect"),
    );
    assert_eq!(status, "\":ok\"", "got {status}");
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn an_inward_offset_that_consumes_the_profile_is_rejected() {
    let err = eval_err("rect(40, 20).offset_2d(-15).extrude(5)");
    assert!(
        err.contains("leaves no profile"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_zero_sketch_offset_is_rejected() {
    let err = eval_err(&rect_sketch("offset 0"));
    assert!(
        err.contains("offset distance must be non-zero"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_sketch_takes_only_one_offset() {
    let err = eval_err(&rect_sketch("offset 1.0\n          offset 2.0"));
    assert!(err.contains("only one offset"), "unexpected message: {err}");
}

#[test]
fn a_non_numeric_sketch_offset_is_rejected() {
    let err = eval_err(&rect_sketch("offset :big"));
    assert!(
        err.contains("offset distance must be a number"),
        "unexpected message: {err}"
    );
}

#[test]
fn offsetting_a_solid_is_rejected() {
    let err = eval_err("box(10, 10, 10).offset_2d(1)");
    assert!(
        err.contains("must be a Face or Wire"),
        "unexpected message: {err}"
    );
}
