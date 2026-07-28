/// Phase 11 Track A — sketch-level patterns.
///
/// `linear_pattern`, `polar_pattern`, and `grid_pattern` inside
/// `sketch do … end` replicate the finished profile into a single compound
/// profile, so one `extrude`, `pad`, or `pocket` applies to every copy — six
/// bolt holes become one pocket rather than six.
///
/// The same three names exist as top-level functions taking a shape; those
/// still work inside a sketch block, which is what lets a block build compound
/// geometry directly and return it.
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

fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() < 1.0e-6,
        "{what}: expected {expected}, got {actual}"
    );
}

/// A 4 x 4 square sketch with `body` appended inside the block.
fn square_sketch(body: &str) -> String {
    format!(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 4.0, 0.0)
          c = point(:c, 4.0, 4.0)
          d = point(:d, 0.0, 4.0)
          line a, b
          line b, c
          line c, d
          line d, a
          {body}
        end
        "#
    )
}

/// An r = 3 circle sketched 20 mm out from the origin, with `body` appended.
fn bolt_circle_sketch(body: &str) -> String {
    format!(
        r#"
        sketch do
          hub = construction_point(:hub, 0.0, 0.0)
          c = point(:c, 20.0, 0.0)
          circle_at c, 3.0
          {body}
        end
        "#
    )
}

// ---------------------------------------------------------------------------
// Linear and grid
// ---------------------------------------------------------------------------

#[test]
fn a_linear_pattern_repeats_the_profile_along_a_row() {
    // Three 4 x 4 squares, 2 mm thick: 3 * 16 * 2 = 96.
    let volume = eval_num(&format!(
        "({}).extrude(2.0).volume",
        square_sketch("linear_pattern count: 3, dx: 10.0")
    ));
    assert_close(volume, 96.0, "linear pattern volume");
}

#[test]
fn a_linear_pattern_can_step_along_both_axes() {
    // A diagonal row still yields three separate squares; the bounding box
    // proves they stepped in x and y rather than only one of them.
    let bbox = eval(&format!(
        "({}).extrude(2.0).bounding_box.inspect",
        square_sketch("linear_pattern count: 3, dx: 10.0, dy: 6.0")
    ));
    assert!(
        bbox.contains("dx: 24.0") && bbox.contains("dy: 16.0"),
        "expected a 24 x 16 diagonal span, got {bbox}"
    );
}

#[test]
fn a_grid_pattern_fills_two_axes() {
    // 3 columns x 2 rows of the 4 x 4 square: 6 * 16 * 2 = 192.
    let volume = eval_num(&format!(
        "({}).extrude(2.0).volume",
        square_sketch("grid_pattern nx: 3, ny: 2, dx: 10.0, dy: 10.0")
    ));
    assert_close(volume, 192.0, "grid pattern volume");
}

#[test]
fn a_count_of_one_leaves_the_profile_alone() {
    // A parametric script may drive the count down to a single copy; that is
    // a no-op, not an error.
    let volume = eval_num(&format!(
        "({}).extrude(2.0).volume",
        square_sketch("linear_pattern count: 1, dx: 10.0")
    ));
    assert_close(volume, 32.0, "single-copy volume");
}

#[test]
fn pattern_distances_accept_unit_values() {
    let volume = eval_num(&format!(
        "({}).extrude(2.0).volume",
        square_sketch("linear_pattern count: 3, dx: 10.mm")
    ));
    assert_close(volume, 96.0, "unit-valued spacing");
}

// ---------------------------------------------------------------------------
// Polar
// ---------------------------------------------------------------------------

#[test]
fn a_polar_pattern_spaces_copies_around_the_origin() {
    // Six r = 3 holes on a 20 mm bolt circle: 6 * pi * 9 * 5.
    let volume = eval_num(&format!(
        "({}).extrude(5.0).volume",
        bolt_circle_sketch("polar_pattern count: 6")
    ));
    assert_close(volume, 6.0 * PI * 9.0 * 5.0, "polar pattern volume");
}

#[test]
fn a_polar_pattern_can_turn_about_a_sketch_point() {
    // The ring is built 20 mm out from a hub at (50, 50), so the pattern must
    // stay centred there: 50 +/- (20 + 3).
    let bbox = eval(
        r#"
        sketch do
          hub = construction_point(:hub, 50.0, 50.0)
          c = point(:c, 70.0, 50.0)
          circle_at c, 3.0
          polar_pattern count: 6, center: hub
        end.extrude(5.0).bounding_box.inspect
        "#,
    );
    assert!(
        bbox.contains("x: 27.0") && bbox.contains("dx: 46.0"),
        "expected a ring centred on the hub, got {bbox}"
    );
}

#[test]
fn a_polar_centre_can_be_given_as_a_coordinate_pair() {
    let bbox = eval(
        r#"
        sketch do
          c = point(:c, 70.0, 50.0)
          circle_at c, 3.0
          polar_pattern count: 6, center: [50.0, 50.0]
        end.extrude(5.0).bounding_box.inspect
        "#,
    );
    assert!(
        bbox.contains("x: 27.0") && bbox.contains("dx: 46.0"),
        "expected the same ring as the sketch-point centre, got {bbox}"
    );
}

#[test]
fn a_partial_angle_sweeps_only_part_of_the_circle() {
    // count 3 over 180 degrees puts copies at 0, 60, and 120 degrees — centres
    // at (20, 0), (10, 17.32), and (-10, 17.32). With r = 3 that spans x from
    // -13 to 23, short of the -23..23 a full circle would give.
    let bbox = eval(&format!(
        "({}).extrude(1.0).bounding_box.inspect",
        bolt_circle_sketch("polar_pattern count: 3, angle: 180")
    ));
    assert!(
        bbox.contains("x: -13.0") && bbox.contains("dx: 36.0"),
        "expected a half-circle spread, got {bbox}"
    );

    // The same three copies spread over a full circle instead sit at 0, 120,
    // and 240 degrees, reaching below the axis: y from -20.32 to 20.32 rather
    // than the half sweep's -3 to 20.32.
    let full = eval(&format!(
        "({}).extrude(1.0).bounding_box.inspect",
        bolt_circle_sketch("polar_pattern count: 3")
    ));
    assert!(
        bbox.contains("y: -3.0") && full.contains("y: -20.32"),
        "a full sweep should drop below the axis; half: {bbox}, full: {full}"
    );
}

#[test]
fn a_polar_centre_follows_a_trimmed_corner() {
    // :d starts at (0, 6) and the trim moves it to (4, 6); rotating the
    // profile 90 degrees about it lands the copy at x 4..10, y 2..12, so the
    // union spans 10 x 12. Centring on the *old* position would not.
    let bbox = eval(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 6.0)
          d = point(:d, 0.0, 6.0)
          line a, b
          line b, c
          line c, d
          line d, a
          trim c, d, by: 4.0
          polar_pattern count: 2, center: d, angle: 180
        end.extrude(1.0).bounding_box.inspect
        "#,
    );
    assert!(
        bbox.contains("dx: 10.0") && bbox.contains("dy: 12.0"),
        "expected the pattern to turn about the moved corner, got {bbox}"
    );
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

#[test]
fn one_pocket_cuts_every_copy() {
    // The reason sketch patterns exist: six holes in a single pocket call.
    let volume = eval_num(&format!(
        r#"
        plate = box(60, 60, 10).translate(-30, -30, 0)
        plate.pocket(:top, depth: 3) {{ {} }}.volume
        "#,
        bolt_circle_sketch("polar_pattern count: 6")
    ));
    assert_close(volume, 36000.0 - 6.0 * PI * 9.0 * 3.0, "pocketed volume");
}

#[test]
fn a_pattern_replicates_the_offset_profile() {
    // The offset runs first, so each copy carries it: two 6 x 6 squares with
    // r = 1 rounded corners, 1 mm thick.
    let volume = eval_num(&format!(
        "({}).extrude(1.0).volume",
        square_sketch(
            r#"
            offset 1.0
            linear_pattern count: 2, dx: 20.0
            "#
        )
    ));
    assert_close(volume, 2.0 * (36.0 - (4.0 - PI)), "offset then patterned");
}

#[test]
fn a_pattern_replicates_corner_modifiers_and_segment_edits() {
    // Each copy must carry the fillet and the trim, so the volume is twice a
    // single shaped profile and strictly less than two plain rectangles.
    let patterned = eval_num(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 6.0)
          d = point(:d, 0.0, 6.0)
          line a, b
          line b, c
          line c, d
          line d, a
          fillet a, 1.0
          trim c, d, by: 2.0
          linear_pattern count: 2, dx: 20.0
        end.extrude(1.0).volume
        "#,
    );
    let single = eval_num(
        r#"
        sketch do
          a = point(:a, 0.0, 0.0)
          b = point(:b, 10.0, 0.0)
          c = point(:c, 10.0, 6.0)
          d = point(:d, 0.0, 6.0)
          line a, b
          line b, c
          line c, d
          line d, a
          fillet a, 1.0
          trim c, d, by: 2.0
        end.extrude(1.0).volume
        "#,
    );
    assert_close(patterned, single * 2.0, "patterned shaped profile");
    assert!(
        patterned < 120.0,
        "expected less than two plain 10 x 6 rectangles, got {patterned}"
    );
}

#[test]
fn the_top_level_pattern_functions_still_work_inside_a_sketch_block() {
    // The builder methods shadow the Kernel functions of the same name; a
    // Shape as the first argument has to reach the original.
    let volume = eval_num(
        "sketch { polar_pattern(circle(3).translate(20, 0, 0), 6, 360) }.extrude(5.0).volume",
    );
    assert_close(volume, 6.0 * PI * 9.0 * 5.0, "Kernel-form pattern");
}

#[test]
fn a_patterned_sketch_still_reports_clean_diagnostics() {
    let status = eval(
        &square_sketch("linear_pattern count: 3, dx: 10.0")
            .replace("sketch do", "sketch(diagnostics: true) do")
            .replace("end\n        ", "end.sketch_diagnostics[:status].inspect"),
    );
    assert_eq!(status, "\":ok\"", "got {status}");
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn a_linear_pattern_without_a_step_is_rejected() {
    let err = eval_err(&square_sketch("linear_pattern count: 3"));
    assert!(
        err.contains("non-zero dx: or dy:"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_grid_pattern_without_the_matching_step_is_rejected() {
    let err = eval_err(&square_sketch("grid_pattern nx: 3, ny: 2, dx: 10.0"));
    assert!(
        err.contains("non-zero dy: to space its 2 rows"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_count_below_one_is_rejected() {
    let err = eval_err(&square_sketch("linear_pattern count: 0, dx: 5.0"));
    assert!(
        err.contains("count: must be an Integer >= 1"),
        "unexpected message: {err}"
    );
}

#[test]
fn an_unknown_pattern_option_is_rejected() {
    let err = eval_err(&square_sketch("linear_pattern count: 3, step: 5.0"));
    assert!(
        err.contains("unknown option :step"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_sketch_takes_only_one_pattern() {
    let err = eval_err(&square_sketch(
        r#"
        linear_pattern count: 3, dx: 5.0
        polar_pattern count: 2
        "#,
    ));
    assert!(
        err.contains("only one pattern"),
        "unexpected message: {err}"
    );
}

#[test]
fn an_unusable_polar_centre_is_rejected() {
    let err = eval_err(&square_sketch("polar_pattern count: 3, center: :hub"));
    assert!(
        err.contains("center: must be a sketch point"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_zero_sweep_angle_is_rejected() {
    let err = eval_err(&square_sketch("polar_pattern count: 3, angle: 0"));
    assert!(
        err.contains("angle: must be non-zero"),
        "unexpected message: {err}"
    );
}

#[test]
fn a_positional_call_that_is_not_the_kernel_form_is_rejected() {
    let err = eval_err(&square_sketch("linear_pattern 3"));
    assert!(
        err.contains("takes keyword arguments"),
        "unexpected message: {err}"
    );
}
