//! Phase 10 — constraint sketching MVP.

use rrcad::ruby::vm::MrubyVm;

#[test]
fn sketch_closed_line_loop_returns_face() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "sketch do
               p1 = point(0, 0)
               p2 = point(20, 0)
               p3 = point(20, 10)
               p4 = point(0, 10)
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn sketch_profile_can_extrude() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               p1 = point(0, 0)
               p2 = point(20, 0)
               p3 = point(20, 10)
               p4 = point(0, 10)
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(5).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 1000.0).abs() < 1.0,
        "expected 20x10x5 sketch extrusion volume near 1000, got {volume}"
    );
}

#[test]
fn rectangle_helper_builds_constrained_profile() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               origin = point(:origin, 0, 0)
               rectangle origin, 24, 7
             end
             profile.extrude(3).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 504.0).abs() < 1.0,
        "expected 24x7x3 rectangle volume near 504, got {volume}"
    );
}

#[test]
fn rectangle_helper_origin_can_be_constraint_resolved() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               anchor = point(0, 0)
               origin = point(:origin, nil, nil)
               coincident origin, anchor
               rectangle origin, 10, 5
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 100.0).abs() < 1.0,
        "expected 10x5x2 rectangle volume near 100, got {volume}"
    );
}

#[test]
fn rectangle_rejects_non_positive_size() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               origin = point(0, 0)
               rectangle origin, -10, 5
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("rectangle width must be > 0"),
        "expected rectangle width error, got: {err}"
    );
}

#[test]
fn centered_rectangle_helper_builds_profile_around_center() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "bb = sketch do
               center = point(:center, 0, 0)
               centered_rectangle center, 20, 8
             end.bounding_box
             [bb[:x], bb[:y], bb[:dx], bb[:dy]].inspect",
        )
        .unwrap();
    assert!(
        result.contains("-10"),
        "expected xmin near -10, got {result}"
    );
    assert!(result.contains("-4"), "expected ymin near -4, got {result}");
    assert!(result.contains("20"), "expected dx near 20, got {result}");
    assert!(result.contains("8"), "expected dy near 8, got {result}");
}

#[test]
fn centered_rectangle_center_can_be_constraint_resolved() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               a = point(-5, 0)
               b = point(5, 0)
               center = midpoint(:center, a, b)
               centered_rectangle center, 10, 6
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 120.0).abs() < 1.0,
        "expected 10x6x2 centered rectangle volume near 120, got {volume}"
    );
}

#[test]
fn sketch_can_return_existing_exact_profile() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("sketch { circle(5) }.extrude(2).shape_type")
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn sketch_can_return_existing_profile_from_builder_block_arg() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("sketch { |s| rect(10, 5) }.extrude(2).volume")
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 100.0).abs() < 1.0,
        "expected returned rect profile volume near 100, got {volume}"
    );
}

#[test]
fn circle_at_builds_exact_profile_at_resolved_center() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "bb = sketch do
               c = point(10, 5)
               circle_at c, 3
             end.bounding_box
             [bb[:x], bb[:y], bb[:dx], bb[:dy]].inspect",
        )
        .unwrap();
    assert!(result.contains("7"), "expected xmin near 7, got {result}");
    assert!(result.contains("2"), "expected ymin near 2, got {result}");
    assert!(
        result.contains("6"),
        "expected diameter near 6, got {result}"
    );
}

#[test]
fn circle_at_center_can_be_constraint_resolved() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               left = point(-5, 0)
               right = point(5, 0)
               center = midpoint(:center, left, right)
               circle_at center, 2
             end
             profile.extrude(1).shape_type",
        )
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn circle_at_rejects_non_positive_radius() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               c = point(0, 0)
               circle_at c, 0
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("circle_at radius must be > 0"),
        "expected circle radius error, got: {err}"
    );
}

#[test]
fn arc_at_builds_translated_wire() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "bb = sketch do
               c = point(10, 5)
               arc_at c, 3, 0.deg, 180.deg
             end.bounding_box
             [bb[:x], bb[:y], bb[:dx], bb[:dy]].inspect",
        )
        .unwrap();
    assert!(result.contains("7"), "expected xmin near 7, got {result}");
    assert!(result.contains("5"), "expected ymin near 5, got {result}");
    assert!(result.contains("6"), "expected dx near 6, got {result}");
    assert!(result.contains("3"), "expected dy near 3, got {result}");
}

#[test]
fn arc_at_center_can_be_constraint_resolved() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "sketch do
               left = point(-5, 0)
               right = point(5, 0)
               center = midpoint(:center, left, right)
               arc_at center, 2, 0, 90
             end.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":wire");
}

#[test]
fn slot_between_builds_horizontal_face() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               a = point(0, 0)
               b = point(20, 0)
               slot_between a, b, 3
             end
             profile.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn slot_between_can_extrude() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               a = point(0, 0)
               b = point(20, 0)
               slot_between a, b, 3
             end
             profile.extrude(1).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        volume > 145.0 && volume < 152.0,
        "expected slot extrusion volume near 148.3, got {volume}"
    );
}

#[test]
fn slot_between_requires_axis_aligned_points() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               a = point(0, 0)
               b = point(20, 5)
               slot_between a, b, 3
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("horizontal or vertical"),
        "expected axis-aligned slot error, got: {err}"
    );
}

#[test]
fn slot_between_rejects_non_positive_radius() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               a = point(0, 0)
               b = point(20, 0)
               slot_between a, b, -1
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("slot_between radius must be > 0"),
        "expected slot radius error, got: {err}"
    );
}

#[test]
fn sketch_requires_closed_loop() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               p1 = point(0, 0)
               p2 = point(20, 0)
               p3 = point(20, 10)
               p4 = point(0, 10)
               line p1, p2
               line p2, p3
               line p3, p4
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("closed loop"),
        "expected closed-loop error, got: {err}"
    );
}

#[test]
fn horizontal_vertical_constraints_resolve_missing_coordinates() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               p1 = point(0, 0)
               p2 = point(20, nil)
               p3 = point(nil, 10)
               p4 = point(0, nil)
               horizontal p1, p2
               vertical p2, p3
               horizontal p3, p4
               vertical p4, p1
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(5).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 1000.0).abs() < 1.0,
        "expected constrained 20x10x5 profile volume near 1000, got {volume}"
    );
}

#[test]
fn coincident_constraint_closes_open_loop() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "sketch do
               p1 = point(0, 0)
               p2 = point(20, 0)
               p3 = point(20, 10)
               p4 = point(0, 10)
               p5 = point(nil, nil)
               coincident p5, p1
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p5
             end.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn conflicting_horizontal_constraint_reports_error() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               p1 = point(0, 0)
               p2 = point(20, 1)
               p3 = point(20, 10)
               p4 = point(0, 10)
               horizontal p1, p2
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("conflicting horizontal constraint"),
        "expected horizontal conflict, got: {err}"
    );
}

#[test]
fn dimension_constraint_sets_axis_aligned_length() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               p1 = point(0, 0)
               p2 = point(nil, nil)
               p3 = point(nil, nil)
               p4 = point(nil, nil)
               horizontal p1, p2
               vertical p2, p3
               horizontal p3, p4
               vertical p4, p1
               dimension p1, p2, 30
               dimension p2, p3, 12
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 720.0).abs() < 1.0,
        "expected 30x12x2 profile volume near 720, got {volume}"
    );
}

#[test]
fn equal_length_constraint_sets_missing_matching_segment() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               p1 = point(0, 0)
               p2 = point(10, 0)
               p3 = point(nil, nil)
               p4 = point(0, nil)
               horizontal p1, p2
               vertical p2, p3
               horizontal p3, p4
               vertical p4, p1
               equal_length p1, p2, p2, p3
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(1).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 100.0).abs() < 1.0,
        "expected 10x10x1 profile volume near 100, got {volume}"
    );
}

#[test]
fn conflicting_dimension_constraint_reports_error() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               p1 = point(0, 0)
               p2 = point(20, 0)
               p3 = point(20, 10)
               p4 = point(0, 10)
               horizontal p1, p2
               dimension p1, p2, 30
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("conflicting dimension constraint"),
        "expected dimension conflict, got: {err}"
    );
}

#[test]
fn dimension_rejects_non_positive_length() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               p1 = point(0, 0)
               p2 = point(nil, 0)
               dimension p1, p2, 0
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("dimension length must be > 0"),
        "expected positive dimension error, got: {err}"
    );
}

#[test]
fn parallel_constraint_propagates_axis_orientation() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               p1 = point(0, 0)
               p2 = point(15, 0)
               p3 = point(15, 8)
               p4 = point(0, nil)
               parallel p1, p2, p3, p4
               vertical p4, p1
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 240.0).abs() < 1.0,
        "expected 15x8x2 profile volume near 240, got {volume}"
    );
}

#[test]
fn perpendicular_constraint_propagates_axis_orientation() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               p1 = point(0, 0)
               p2 = point(12, 0)
               p3 = point(12, nil)
               p4 = point(0, 6)
               perpendicular p1, p2, p2, p3
               horizontal p3, p4
               vertical p4, p1
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 144.0).abs() < 1.0,
        "expected 12x6x2 profile volume near 144, got {volume}"
    );
}

#[test]
fn named_points_can_be_referenced_by_name() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               point :origin, 0, 0
               point :right, 20, nil
               point :top_right, nil, 10
               point :top_left, 0, nil
               horizontal ref(:origin), ref(:right)
               vertical ref(:right), ref(:top_right)
               horizontal ref(:top_right), ref(:top_left)
               vertical ref(:top_left), ref(:origin)
               line ref(:origin), ref(:right)
               line ref(:right), ref(:top_right)
               line ref(:top_right), ref(:top_left)
               line ref(:top_left), ref(:origin)
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 400.0).abs() < 1.0,
        "expected 20x10x2 profile volume near 400, got {volume}"
    );
}

#[test]
fn construction_point_alias_can_be_referenced_with_brackets() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "sketch do
               construction_point :a, 0, 0
               construction_point :b, 5, 0
               construction_point :c, 5, 5
               construction_point :d, 0, 5
               line self[:a], self[:b]
               line self[:b], self[:c]
               line self[:c], self[:d]
               line self[:d], self[:a]
             end.shape_type",
        )
        .unwrap();
    assert_eq!(result, ":face");
}

#[test]
fn construction_line_does_not_add_profile_edges() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               a = construction_point(:a, 0, 0)
               b = construction_point(:b, 10, 0)
               construction_line a, b
               origin = point(0, 0)
               rectangle origin, 4, 3
             end
             profile.edges(:all).length",
        )
        .unwrap();
    assert_eq!(result, "4");
}

#[test]
fn construction_line_can_drive_constraints_without_closing_loop() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               a = construction_point(:a, 0, 0)
               b = construction_point(:b, 8, 0)
               construction_line a, b
               origin = point(nil, nil)
               coincident origin, a
               rectangle origin, 8, 2
             end
             profile.extrude(1).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 16.0).abs() < 1.0,
        "expected construction-guided rectangle volume near 16, got {volume}"
    );
}

#[test]
fn unknown_sketch_reference_reports_name() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               ref(:missing)
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("unknown sketch reference: missing"),
        "expected unknown reference error, got: {err}"
    );
}

#[test]
fn under_constrained_error_names_missing_coordinate() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               p1 = point(:origin, 0, 0)
               p2 = point(:right, 20, nil)
               p3 = point(:top, 0, 10)
               line p1, p2
               line p2, p3
               line p3, p1
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("under-constrained") && err.contains(":right") && err.contains("missing y"),
        "expected named under-constrained point error, got: {err}"
    );
}

#[test]
fn midpoint_construction_point_resolves_from_endpoints() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               left = point(nil, 0)
               right = point(10, 0)
               center = midpoint(:center, left, right)
               fixed center, 0, 0
               top = point(0, 5)
               line left, right
               line right, top
               line top, left
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 100.0).abs() < 1.0,
        "expected symmetric triangle volume near 100, got {volume}"
    );
}

#[test]
fn midpoint_can_drive_symmetric_profile() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               left = point(-10, 0)
               right = point(10, 0)
               center = midpoint(:center, left, right)
               top_right = point(nil, 8)
               top_left = point(nil, 8)
               vertical right, top_right
               vertical left, top_left
               horizontal top_left, top_right
               fixed center, 0, 0
               line left, right
               line right, top_right
               line top_right, top_left
               line top_left, left
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 320.0).abs() < 1.0,
        "expected 20x8x2 profile volume near 320, got {volume}"
    );
}

#[test]
fn symmetric_constraint_resolves_opposite_point() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               center = point(:center, 0, 0)
               bottom_left = point(-10, -4)
               top_right = point(nil, nil)
               bottom_right = point(nil, -4)
               top_left = point(-10, nil)
               symmetric bottom_left, top_right, center
               vertical bottom_right, top_right
               horizontal top_left, top_right
               vertical bottom_left, top_left
               horizontal bottom_left, bottom_right
               line bottom_left, bottom_right
               line bottom_right, top_right
               line top_right, top_left
               line top_left, bottom_left
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 320.0).abs() < 1.0,
        "expected symmetric 20x8x2 rectangle volume near 320, got {volume}"
    );
}

#[test]
fn symmetric_constraint_can_resolve_center() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               left = point(-6, 0)
               right = point(6, 0)
               center = point(:center, nil, nil)
               symmetric left, right, center
               circle_at center, 2
             end
             profile.bounding_box",
        )
        .unwrap();
    assert!(
        result.contains("x: -2"),
        "expected center-resolved circle xmin near -2, got {result}"
    );
    assert!(
        result.contains("y: -2"),
        "expected center-resolved circle ymin near -2, got {result}"
    );
}

#[test]
fn conflicting_symmetric_constraint_reports_error() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               left = point(-6, 0)
               right = point(8, 0)
               center = point(0, 0)
               symmetric left, right, center
               circle_at center, 1
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("conflicting symmetric constraint"),
        "expected symmetric conflict, got: {err}"
    );
}

#[test]
fn mirror_x_reflects_point_across_horizontal_axis() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               top_left = point(0, 5)
               bottom_left = point(nil, nil)
               top_right = point(10, 5)
               bottom_right = point(10, nil)
               mirror_x top_left, bottom_left, 0
               mirror_x top_right, bottom_right, 0
               line bottom_left, bottom_right
               line bottom_right, top_right
               line top_right, top_left
               line top_left, bottom_left
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 200.0).abs() < 1.0,
        "expected mirrored 10x10x2 profile volume near 200, got {volume}"
    );
}

#[test]
fn mirror_y_reflects_point_across_vertical_axis() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               left_bottom = point(-6, 0)
               right_bottom = point(nil, nil)
               left_top = point(-6, 4)
               right_top = point(nil, 4)
               mirror_y left_bottom, right_bottom, 0
               mirror_y left_top, right_top, 0
               line left_bottom, right_bottom
               line right_bottom, right_top
               line right_top, left_top
               line left_top, left_bottom
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 96.0).abs() < 1.0,
        "expected mirrored 12x4x2 profile volume near 96, got {volume}"
    );
}

#[test]
fn conflicting_mirror_constraint_reports_error() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               a = point(0, 5)
               b = point(0, -6)
               mirror_x a, b, 0
               line a, b
               line b, point(1, -6)
               line point(1, -6), a
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("conflicting mirror_x constraint"),
        "expected mirror conflict, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Tangent constraint
// ---------------------------------------------------------------------------

#[test]
fn tangent_horizontal_line_resolves_y_above_center() {
    // Horizontal line tangent to circle centered at (5, 0), r=3, side: :above
    // → line.y must be 3. Box profile 10×3×2 ⇒ volume 60.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               c  = point(5, 0)
               p1 = point(0, 0)
               p2 = point(10, 0)
               p3 = point(10, nil)
               p4 = point(0, nil)
               horizontal p3, p4
               vertical p1, p4
               vertical p2, p3
               tangent p3, p4, c, 3, side: :above
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 60.0).abs() < 1.0,
        "expected 10×3×2 = 60, got {volume}"
    );
}

#[test]
fn tangent_vertical_line_resolves_x_to_right_of_center() {
    // Vertical line tangent to circle centered at (0, 0), r=4, side: :right
    // → line.x must be 4. Box profile 4×5×2 ⇒ volume 40.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               c  = point(0, 0)
               p1 = point(0, 0)
               p2 = point(nil, 0)
               p3 = point(nil, 5)
               p4 = point(0, 5)
               horizontal p1, p2
               horizontal p3, p4
               vertical p2, p3
               tangent p2, p3, c, 4, side: :right
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!(
        (volume - 40.0).abs() < 1.0,
        "expected 4×5×2 = 40, got {volume}"
    );
}

#[test]
fn tangent_constraint_verifies_resolved_geometry() {
    // All coords known, distance matches radius — should be accepted (no error).
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "profile = sketch do
               c  = point(5, 0)
               p1 = point(0, 0)
               p2 = point(10, 0)
               p3 = point(10, 3)
               p4 = point(0, 3)
               tangent p3, p4, c, 3
               line p1, p2
               line p2, p3
               line p3, p4
               line p4, p1
             end
             profile.extrude(2).volume",
        )
        .unwrap();
    let volume: f64 = result.trim().parse().expect("expected a volume");
    assert!((volume - 60.0).abs() < 1.0, "got {volume}");
}

#[test]
fn conflicting_tangent_constraint_reports_error() {
    // Line at y=2, center at (5,0), radius 3 ⇒ distance 2 ≠ 3 ⇒ conflict.
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               c  = point(5, 0)
               p1 = point(0, 2)
               p2 = point(10, 2)
               tangent p1, p2, c, 3
               line p1, p2
               line p2, point(10, 5)
               line point(10, 5), p1
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("conflicting tangent constraint"),
        "expected tangent conflict, got: {err}"
    );
}

#[test]
fn tangent_rejects_non_positive_radius() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               c  = point(0, 0)
               a  = point(0, 5)
               b  = point(10, 5)
               tangent a, b, c, 0
               line a, b
               line b, point(10, 0)
               line point(10, 0), a
             end",
        )
        .unwrap_err();
    assert!(err.contains("must be > 0"), "got: {err}");
}

#[test]
fn tangent_rejects_invalid_side() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "sketch do
               c  = point(0, 0)
               a  = point(0, 5)
               b  = point(10, 5)
               tangent a, b, c, 3, side: :outside
             end",
        )
        .unwrap_err();
    assert!(
        err.contains("side:"),
        "expected side validation error, got: {err}"
    );
}
