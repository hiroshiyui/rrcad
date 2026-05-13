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
