/// Phase 11 Track A — spline segments in sketch profiles.
///
/// `spline a, b, through: [...]` draws a curved segment of a constraint
/// sketch. The curve reaches the finished profile as a real interpolated
/// BSpline edge rather than a polyline standing in for one, which is what
/// keeps it smooth through export and every later feature.
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

fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() < 1.0e-6,
        "{what}: expected {expected}, got {actual}"
    );
}

/// A 10 x 10 square whose top edge bows up through (5, 14), with `body`
/// appended inside the block.
fn bowed_square(body: &str) -> String {
    format!(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 10.0)
          d = point(:d, 0.0, 10.0)
          line a, b
          line b, c
          spline c, d, through: [[5.0, 14.0]]
          line d, a
          {body}
        end
        "#
    )
}

// ---------------------------------------------------------------------------
// The curve is a real spline
// ---------------------------------------------------------------------------

#[test]
fn a_spline_segment_bows_the_profile_out() {
    // A quadratic through (10,10), (5,14), (0,10) encloses 2/3 * 10 * 4 = 26.67
    // above the chord, on top of the 100 the square already had. A polyline
    // through the same three points would only add the 20 of a triangle, so
    // this value is itself the evidence that the edge is curved.
    let area = eval_num(&format!("({}).extrude(1.0).volume", bowed_square("")));
    assert_close(area, 100.0 + 2.0 / 3.0 * 10.0 * 4.0, "bowed profile area");
}

#[test]
fn a_spline_segment_is_one_edge_not_a_polyline() {
    // Four segments in, four edges out: the curve was not flattened into a
    // run of short straight edges.
    let edges = eval_num(&format!("({}).edges(:all).length", bowed_square("")));
    assert_close(edges, 4.0, "edge count");
}

#[test]
fn the_profile_reaches_the_splines_peak() {
    let bbox = eval(&format!(
        "({}).extrude(1.0).bounding_box.inspect",
        bowed_square("")
    ));
    assert!(
        bbox.contains("dy: 14.0"),
        "expected the profile to reach y = 14, got {bbox}"
    );
}

#[test]
fn a_spline_can_pass_through_several_interior_points() {
    let area = eval_num(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 20.0, 0.0)
          c = point(:c, 20.0, 10.0)
          d = point(:d, 0.0, 10.0)
          line a, b
          line b, c
          spline c, d, through: [[14.0, 13.0], [6.0, 13.0]]
          line d, a
        end.extrude(1.0).volume
        "#,
    );
    assert!(
        area > 200.0 && area < 260.0,
        "expected a bowed 20 x 10 profile, got {area}"
    );
}

#[test]
fn a_spline_and_one_line_can_close_a_loop() {
    // Two segments are enough once one of them is curved, so the
    // three-segment floor has to lift for curved sketches.
    let area = eval_num(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 20.0, 0.0)
          spline a, b, through: [[10.0, 8.0]]
          line b, a
        end.extrude(1.0).volume
        "#,
    );
    assert_close(area, 2.0 / 3.0 * 20.0 * 8.0, "two-segment loop area");
}

#[test]
fn an_all_straight_sketch_still_needs_three_segments() {
    let err = eval_err(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          line a, b
          line b, a
        end
        "#,
    );
    assert!(
        err.contains("at least 3 line segments"),
        "unexpected message: {err}"
    );
}

// ---------------------------------------------------------------------------
// Interior points and the solver
// ---------------------------------------------------------------------------

#[test]
fn an_interior_point_can_be_a_solved_sketch_point() {
    // :peak takes its x from the midpoint of the top edge through a vertical
    // constraint, landing at (5, 14) — the same curve as the literal form.
    let area = eval_num(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 10.0)
          d = point(:d, 0.0, 10.0)
          mid = midpoint(:mid, c, d)
          peak = point(:peak, nil, 14.0)
          vertical mid, peak
          line a, b
          line b, c
          spline c, d, through: [peak]
          line d, a
        end.extrude(1.0).volume
        "#,
    );
    assert_close(
        area,
        100.0 + 2.0 / 3.0 * 10.0 * 4.0,
        "solved interior point",
    );
}

#[test]
fn an_under_constrained_interior_point_is_named() {
    let err = eval_err(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 10.0)
          d = point(:d, 0.0, 10.0)
          peak = point(:peak, nil, 14.0)
          line a, b
          line b, c
          spline c, d, through: [peak]
          line d, a
        end.extrude(1.0)
        "#,
    );
    assert!(
        err.contains("under-constrained") && err.contains(":peak") && err.contains("missing x"),
        "unexpected message: {err}"
    );
}

// ---------------------------------------------------------------------------
// Composition with the rest of the sketcher
// ---------------------------------------------------------------------------

#[test]
fn a_corner_away_from_the_spline_can_still_be_filleted() {
    // :a joins two straight segments, so rounding it is fine even though the
    // sketch also carries a curve.
    let filleted = eval_num(&format!(
        "({}).extrude(1.0).volume",
        bowed_square("fillet a, 2.0")
    ));
    let plain = eval_num(&format!("({}).extrude(1.0).volume", bowed_square("")));
    assert!(
        filleted < plain && plain - filleted < 1.0,
        "the fillet should shave a corner off the bowed profile: {filleted} vs {plain}"
    );
}

#[test]
fn an_offset_grows_a_curved_profile() {
    let grown = eval_num(&format!(
        "({}).extrude(1.0).volume",
        bowed_square("offset 1.0")
    ));
    let plain = eval_num(&format!("({}).extrude(1.0).volume", bowed_square("")));
    assert!(
        grown > plain,
        "expected the offset to grow the curved profile: {grown} vs {plain}"
    );
}

#[test]
fn a_pattern_replicates_a_curved_profile() {
    let patterned = eval_num(&format!(
        "({}).extrude(1.0).volume",
        bowed_square("linear_pattern count: 2, dx: 20.0")
    ));
    let plain = eval_num(&format!("({}).extrude(1.0).volume", bowed_square("")));
    assert_close(patterned, plain * 2.0, "patterned curved profile");
}

#[test]
fn a_curved_sketch_still_reports_clean_diagnostics() {
    let status = eval(
        &bowed_square("")
            .replace("sketch do", "sketch(diagnostics: true) do")
            .replace("end\n        ", "end.sketch_diagnostics[:status].inspect"),
    );
    assert_eq!(status, "\":ok\"", "got {status}");
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn filleting_a_corner_that_joins_a_spline_is_rejected() {
    // The setback is measured along a straight run, so there is nothing
    // meaningful to measure at :d.
    let err = eval_err(&bowed_square("fillet d, 2.0"));
    assert!(
        err.contains("it joins a spline segment"),
        "unexpected message: {err}"
    );
}

#[test]
fn trimming_a_spline_segment_is_rejected() {
    let err = eval_err(&bowed_square("trim c, d, by: 2.0"));
    assert!(
        err.contains("it is a spline segment"),
        "unexpected message: {err}"
    );
}

#[test]
fn extending_a_spline_segment_is_rejected() {
    let err = eval_err(&bowed_square("extend c, d, by: 2.0"));
    assert!(
        err.contains("it is a spline segment"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_spline_without_interior_points_is_rejected() {
    let err = eval_err(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 10.0)
          spline a, b
          line b, c
          line c, a
        end
        "#,
    );
    assert!(
        err.contains("at least one interior point"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_bad_interior_point_is_rejected() {
    let err = eval_err(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 10.0)
          spline a, b, through: [:peak]
          line b, c
          line c, a
        end
        "#,
    );
    assert!(
        err.contains("sketch points or [x, y] pairs"),
        "unexpected message: {err}"
    );
}

// ---------------------------------------------------------------------------
// The native profile builder beneath the sketcher
// ---------------------------------------------------------------------------

/// Four corner points of a 10 x 10 square as segment pairs, for the internal
/// `__rrcad_profile_2d(points, counts, kinds)` primitive.
const SQUARE_SEGMENTS: &str = "[[0,0],[10,0], [10,0],[10,10], [10,10],[0,10], [0,10],[0,0]]";

#[test]
fn the_profile_builder_accepts_straight_and_curved_segments() {
    let square = eval_num(&format!(
        "__rrcad_profile_2d({SQUARE_SEGMENTS}, [2,2,2,2], [0,0,0,0]).extrude(1.0).volume"
    ));
    assert_close(square, 100.0, "all-straight profile");

    // The same square with its top edge replaced by a curve through (5, 14).
    let bowed = eval_num(
        "__rrcad_profile_2d([[0,0],[10,0], [10,0],[10,10], [10,10],[5,14],[0,10], [0,10],[0,0]], \
         [2,2,3,2], [0,0,1,0]).extrude(1.0).volume",
    );
    assert_close(bowed, 100.0 + 2.0 / 3.0 * 10.0 * 4.0, "curved profile");
}

#[test]
fn the_profile_builder_rejects_mismatched_segment_arrays() {
    let uneven = eval_err(&format!(
        "__rrcad_profile_2d({SQUARE_SEGMENTS}, [2,2,2], [0,0,0,0])"
    ));
    assert!(
        uneven.contains("counts and kinds must have equal length"),
        "unexpected message: {uneven}"
    );

    let short = eval_err(&format!(
        "__rrcad_profile_2d({SQUARE_SEGMENTS}, [1,3,2,2], [0,0,0,0])"
    ));
    assert!(
        short.contains("every segment needs at least 2 points"),
        "unexpected message: {short}"
    );

    let miscounted = eval_err(&format!(
        "__rrcad_profile_2d({SQUARE_SEGMENTS}, [2,2,2,3], [0,0,0,0])"
    ));
    assert!(
        miscounted.contains("point count does not match the segments"),
        "unexpected message: {miscounted}"
    );
}

#[test]
fn spline_endpoints_must_be_sketch_points() {
    let err = eval_err(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 10.0)
          spline a, [10.0, 0.0], through: [[5.0, 2.0]]
          line b, c
          line c, a
        end
        "#,
    );
    assert!(
        err.contains("spline endpoints must be sketch points"),
        "unexpected message: {err}"
    );
}
