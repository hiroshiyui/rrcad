/// Phase 11 Track A — sketch segment edits (`trim` / `extend`).
///
/// These exercise the DSL-level shortening and lengthening of individual
/// sketch segments. Both operations move one endpoint of a drawn segment
/// along its own direction — either by a distance (`by:`) or up to where the
/// segment's infinite line meets another segment's (`to:`) — so the corner
/// shared with the neighbouring segment moves with it and the loop stays
/// closed.
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

/// A 40 x 20 rectangle sketch (corners `:a`..`:d` counter-clockwise from the
/// origin) with `edit` applied, extruded 5 mm. Unmodified volume is 4000.
fn rect_with_edit(edit: &str) -> String {
    format!(
        r#"
        sk = sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 40.0, 0.0)
          c = point(:c, 40.0, 20.0)
          d = point(:d, 0.0, 20.0)
          bottom = line a, b
          right = line b, c
          top = line c, d
          left = line d, a
          {edit}
        end
        sk.extrude(5.0)
        "#
    )
}

// ---------------------------------------------------------------------------
// by: — slide an endpoint along the segment
// ---------------------------------------------------------------------------

#[test]
fn trim_by_distance_shortens_the_segment() {
    // Trimming the top edge 10 mm pulls corner :d in to x = 10, turning the
    // rectangle into a trapezoid: (40 + 30) / 2 * 20 * 5 = 3500.
    let volume = eval_num(&format!("{}.volume", rect_with_edit("trim c, d, by: 10.0")));
    assert!(
        (volume - 3500.0).abs() < 1.0e-6,
        "expected 3500, got {volume}"
    );
}

#[test]
fn extend_by_distance_lengthens_the_segment() {
    // Extending the bottom edge 10 mm pushes corner :b out to x = 50:
    // (40 + 50) / 2 * 20 * 5 = 4500.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit("extend a, b, by: 10.0")
    ));
    assert!(
        (volume - 4500.0).abs() < 1.0e-6,
        "expected 4500, got {volume}"
    );
}

#[test]
fn at_start_moves_the_other_endpoint() {
    // `at: :start` anchors :d and moves :c instead, so the top edge shortens
    // from the right-hand end. Both trims cut the same 500 mm³ triangle off
    // the top, so volume alone cannot tell them apart — a probe box in the
    // top-right corner can: it survives when :d moved and vanishes when :c
    // did.
    let probe = "common(box(7.0, 5.0, 5.0).translate(33.0, 15.0, 0.0)).volume";

    let at_end = eval_num(&format!(
        "{}.{probe}",
        rect_with_edit("trim c, d, by: 10.0")
    ));
    assert!(
        (at_end - 175.0).abs() < 1.0e-6,
        "moving :d should leave the top-right corner intact, got {at_end}"
    );

    let at_start = eval_num(&format!(
        "{}.{probe}",
        rect_with_edit("trim c, d, by: 10.0, at: :start")
    ));
    assert!(
        at_start < 1.0e-6,
        "moving :c should clear the top-right corner, got {at_start}"
    );
}

#[test]
fn a_segment_may_be_passed_as_the_array_line_returns() {
    // `line` returns [a, b], so a named segment can be edited directly.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit("extend bottom, by: 10.0")
    ));
    assert!(
        (volume - 4500.0).abs() < 1.0e-6,
        "expected 4500, got {volume}"
    );
}

#[test]
fn edits_accept_unit_values() {
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit("extend a, b, by: 10.mm")
    ));
    assert!(
        (volume - 4500.0).abs() < 1.0e-6,
        "expected 4500, got {volume}"
    );
}

#[test]
fn either_argument_order_identifies_the_segment() {
    // `line d, a` was drawn in that order; editing it as (a, d) is accepted
    // and the argument order chooses which end moves — here :d, which drops
    // from y = 20 to y = 10: (20 + 10) / 2 * 40 * 5 = 3000.
    let volume = eval_num(&format!("{}.volume", rect_with_edit("trim a, d, by: 10.0")));
    assert!(
        (volume - 3000.0).abs() < 1.0e-6,
        "expected the left edge to shorten from the :d end, got {volume}"
    );
}

// ---------------------------------------------------------------------------
// to: — run an endpoint out to an intersection
// ---------------------------------------------------------------------------

#[test]
fn extend_to_a_construction_line_meets_its_intersection() {
    // The bottom edge runs out to x = 50 where it crosses the vertical
    // reference line: (50 + 40) / 2 * 20 * 5 = 4500.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            r1 = point(50.0, 0.0)
            r2 = point(50.0, 20.0)
            extend a, b, to: construction_line(r1, r2)
            "#
        )
    ));
    assert!(
        (volume - 4500.0).abs() < 1.0e-6,
        "expected 4500, got {volume}"
    );
}

#[test]
fn trim_to_a_construction_line_meets_its_intersection() {
    // The bottom edge comes back to x = 25: (25 + 40) / 2 * 20 * 5 = 3250.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            r1 = point(25.0, 0.0)
            r2 = point(25.0, 20.0)
            trim a, b, to: construction_line(r1, r2)
            "#
        )
    ));
    assert!(
        (volume - 3250.0).abs() < 1.0e-6,
        "expected 3250, got {volume}"
    );
}

#[test]
fn extend_to_a_slanted_reference_uses_the_infinite_line() {
    // A reference segment from (60, 0) to (40, 20) has the line x = 60 - y,
    // which crosses y = 0 at x = 60 — beyond the reference's own endpoint, so
    // this only works if the intersection uses the infinite line.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            r1 = point(60.0, 0.0)
            r2 = point(40.0, 20.0)
            extend a, b, to: construction_line(r1, r2)
            "#
        )
    ));
    assert!(
        (volume - 5000.0).abs() < 1.0e-6,
        "expected (60 + 40) / 2 * 20 * 5 = 5000, got {volume}"
    );
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

#[test]
fn a_later_edit_sees_an_earlier_one() {
    // Extending the bottom edge to x = 50 then trimming 10 back off it must
    // land at x = 40, restoring the original 4000 volume.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            extend a, b, by: 10.0
            trim a, b, by: 10.0
            "#
        )
    ));
    assert!(
        (volume - 4000.0).abs() < 1.0e-6,
        "expected the edits to cancel back to 4000, got {volume}"
    );
}

#[test]
fn a_corner_modifier_applies_to_the_moved_corner() {
    // The trim leaves a 3500 trapezoid; filleting the moved corner :d must
    // then round the corner at its new position, taking a little material.
    let volume = eval_num(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            trim c, d, by: 10.0
            fillet d, 3.0
            "#
        )
    ));
    assert!(
        volume < 3500.0 && volume > 3490.0,
        "expected a slightly rounded 3500 trapezoid, got {volume}"
    );
}

#[test]
fn an_edited_sketch_still_reports_clean_diagnostics() {
    let status = eval(
        &rect_with_edit("extend a, b, by: 5.0")
            .replace("sketch do", "sketch(diagnostics: true) do")
            .replace("sk.extrude(5.0)", "sk.sketch_diagnostics[:status].inspect"),
    );
    assert_eq!(status, "\":ok\"", "got {status}");
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn trimming_more_than_the_segment_has_is_rejected() {
    let err = eval_err(&format!("{}.volume", rect_with_edit("trim a, b, by: 50.0")));
    assert!(
        err.contains("leaves no segment"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_parallel_reference_has_no_intersection() {
    let err = eval_err(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            r1 = point(0.0, 30.0)
            r2 = point(40.0, 30.0)
            extend a, b, to: construction_line(r1, r2)
            "#
        )
    ));
    assert!(err.contains("parallel"), "unexpected message: {err}");
}

#[test]
fn a_trim_that_would_lengthen_suggests_extend() {
    let err = eval_err(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            r1 = point(50.0, 0.0)
            r2 = point(50.0, 20.0)
            trim a, b, to: construction_line(r1, r2)
            "#
        )
    ));
    assert!(
        err.contains("use extend instead"),
        "unexpected message: {err}"
    );
}

#[test]
fn an_extend_that_would_shorten_suggests_trim() {
    let err = eval_err(&format!(
        "{}.volume",
        rect_with_edit(
            r#"
            r1 = point(25.0, 0.0)
            r2 = point(25.0, 20.0)
            extend a, b, to: construction_line(r1, r2)
            "#
        )
    ));
    assert!(
        err.contains("use trim instead"),
        "unexpected message: {err}"
    );
}

#[test]
fn editing_a_pair_that_is_not_a_segment_is_rejected() {
    // :a and :c are opposite corners — no line was drawn between them.
    let err = eval_err(&format!("{}.volume", rect_with_edit("trim a, c, by: 5.0")));
    assert!(
        err.contains("is not a segment of this sketch"),
        "unexpected message: {err}"
    );
}

#[test]
fn exactly_one_of_to_or_by_is_required() {
    let missing = eval_err(&format!("{}.volume", rect_with_edit("trim a, b")));
    assert!(
        missing.contains("exactly one of to: or by:"),
        "unexpected message: {missing}"
    );

    let both = eval_err(&format!(
        "{}.volume",
        rect_with_edit("trim a, b, by: 5.0, to: right")
    ));
    assert!(
        both.contains("exactly one of to: or by:"),
        "unexpected message: {both}"
    );
}

#[test]
fn a_non_positive_distance_is_rejected() {
    let err = eval_err(&format!("{}.volume", rect_with_edit("trim a, b, by: -5.0")));
    assert!(err.contains("by: must be > 0"), "unexpected message: {err}");
}

#[test]
fn an_unknown_at_target_is_rejected() {
    let err = eval_err(&format!(
        "{}.volume",
        rect_with_edit("trim a, b, by: 5.0, at: :middle")
    ));
    assert!(
        err.contains("at: must be :start, :end, or an endpoint"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_single_point_is_not_a_segment() {
    let err = eval_err(&format!("{}.volume", rect_with_edit("trim a, by: 5.0")));
    assert!(
        err.contains("needs two points"),
        "unexpected message: {err}"
    );
}
