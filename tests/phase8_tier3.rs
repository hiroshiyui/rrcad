// Phase 8 Tier 3 — Inspection & clearance
//
// Tests for:
//   shape.distance_to(other)  → Float  (BRepExtrema_DistShapeShape)
//   shape.inertia             → Hash   (BRepGProp + MatrixOfInertia)
//   shape.min_thickness       → Float  (shell offset + DistShapeShape)

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// distance_to
// ---------------------------------------------------------------------------

#[test]
fn distance_to_overlapping_returns_zero() {
    let mut vm = MrubyVm::new();
    // Two boxes at the same position overlap → distance = 0.
    let result = vm.eval("box(5,5,5).distance_to(box(5,5,5))").unwrap();
    let d: f64 = result.trim().parse().expect("expected float");
    assert!(
        d.abs() < 1e-6,
        "overlapping shapes should have distance 0, got {d}"
    );
}

#[test]
fn distance_to_touching_returns_zero() {
    let mut vm = MrubyVm::new();
    // Two boxes that share a face — distance is 0 (touching).
    let result = vm
        .eval(
            "a = box(5,5,5)
             b = box(5,5,5).translate(5, 0, 0)
             a.distance_to(b)",
        )
        .unwrap();
    let d: f64 = result.trim().parse().expect("expected float");
    assert!(
        d.abs() < 1e-3,
        "touching shapes should have distance ≈0, got {d}"
    );
}

#[test]
fn distance_to_separated_shapes() {
    let mut vm = MrubyVm::new();
    // Two unit boxes separated by 3 units along X → distance = 3.
    let result = vm
        .eval(
            "a = box(1,1,1)
             b = box(1,1,1).translate(4, 0, 0)
             a.distance_to(b)",
        )
        .unwrap();
    let d: f64 = result.trim().parse().expect("expected float");
    assert!(
        (d - 3.0).abs() < 0.1,
        "separated shapes should have distance ≈3, got {d}"
    );
}

#[test]
fn distance_to_is_symmetric() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "a = box(2,2,2)
             b = box(2,2,2).translate(5, 0, 0)
             (a.distance_to(b) - b.distance_to(a)).abs < 1e-9",
        )
        .unwrap();
    assert_eq!(result, "true", "distance_to should be symmetric");
}

// ---------------------------------------------------------------------------
// inertia
// ---------------------------------------------------------------------------

#[test]
fn inertia_returns_hash_with_six_keys() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "t = box(4,4,4).inertia
             [:ixx,:iyy,:izz,:ixy,:ixz,:iyz].all? { |k| t.key?(k) }",
        )
        .unwrap();
    assert_eq!(result, "true", "inertia hash must contain all six keys");
}

#[test]
fn inertia_diagonal_positive_for_solid() {
    let mut vm = MrubyVm::new();
    // Principal moments of inertia (diagonal) must be positive for any solid.
    let result = vm
        .eval(
            "t = box(4,4,4).inertia
             t[:ixx] > 0 && t[:iyy] > 0 && t[:izz] > 0",
        )
        .unwrap();
    assert_eq!(result, "true", "diagonal inertia terms must be > 0");
}

#[test]
fn inertia_cube_ixx_equals_iyy() {
    let mut vm = MrubyVm::new();
    // A cube centred at origin has Ixx = Iyy = Izz by symmetry.
    // box(a, a, a) is NOT centred — it goes from (0,0,0) to (a,a,a).
    // We use a sphere which IS symmetric about origin.
    let result = vm
        .eval(
            "t = sphere(5).inertia
             (t[:ixx] - t[:iyy]).abs < 1.0 && (t[:iyy] - t[:izz]).abs < 1.0",
        )
        .unwrap();
    assert_eq!(result, "true", "sphere inertia: Ixx ≈ Iyy ≈ Izz");
}

// ---------------------------------------------------------------------------
// min_thickness
// ---------------------------------------------------------------------------

#[test]
fn min_thickness_hollow_box_returns_wall_thickness() {
    let mut vm = MrubyVm::new();
    // A box shelled with thickness 2 should have min_thickness ≈ 2.
    let result = vm
        .eval(
            "s = box(20, 20, 20).shell(2)
             s.min_thickness",
        )
        .unwrap();
    let t: f64 = result.trim().parse().expect("expected float");
    // Allow generous tolerance — the offset + distance measure is approximate.
    assert!(
        t > 0.5 && t < 5.0,
        "hollow box wall thickness should be ≈2, got {t}"
    );
}

#[test]
fn min_thickness_solid_box_larger_than_zero() {
    let mut vm = MrubyVm::new();
    // A solid box has no inner cavity; min_thickness returns a positive value
    // (the half-size of the shortest dimension, approximately).
    let result = vm.eval("box(10, 10, 10).min_thickness > 0").unwrap();
    assert_eq!(result, "true", "min_thickness must be > 0 for a solid box");
}

#[test]
fn min_thickness_rejects_wire() {
    let mut vm = MrubyVm::new();
    // A Wire is not a solid or shell — should raise an error.
    let err = vm.eval("arc(5, 0, 180).min_thickness").unwrap_err();
    assert!(
        err.contains("Solid")
            || err.contains("Shell")
            || err.contains("solid")
            || err.contains("shell"),
        "expected type error, got: {err}"
    );
}
