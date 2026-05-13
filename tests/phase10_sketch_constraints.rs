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
