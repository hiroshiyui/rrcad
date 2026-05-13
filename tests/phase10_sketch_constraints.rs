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
