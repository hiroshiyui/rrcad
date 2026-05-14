// Phase 11 — Assembly constraint solver.
//
// Tests for the declarative assembly DSL:
//   assembly { |a| a.ground ...; a.part ... { |p| p.mate ... } }
//   Assembly#solve via lazy `to_shape`

use rrcad::ruby::vm::MrubyVm;

#[test]
fn assembly_solver_places_chain_from_fixed_root() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8)
             cap = box(6, 6, 2)
             asm = assembly(\"rig\") do |a|
               a.ground :base, base
               a.part :post, post do
                 mate from: :bottom, to: face(:base, :top)
               end
               a.part :cap, cap do
                 mate from: :bottom, to: face(:post, :top), offset: 2.0
               end
             end
             bb = asm.to_shape.bounding_box
             [bb[:z], bb[:dz]].inspect",
        )
        .unwrap();
    assert!(
        result.contains("[0.0, 17.0]"),
        "expected chain bbox result, got: {result}"
    );
}

#[test]
fn assembly_solver_reports_under_constrained_parts() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "base = box(20, 20, 5)
             loose = box(3, 3, 3)
             asm = assembly(\"rig\") do |a|
               a.ground :base, base
               a.part :loose, loose
             end
             asm.to_shape",
        )
        .unwrap_err();
    assert!(
        err.contains("under-constrained") && err.contains(":loose"),
        "expected under-constrained assembly error, got: {err}"
    );
}

#[test]
fn assembly_solver_detects_conflicting_constraints() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            "base = box(20, 20, 5)
             post = box(4, 4, 8)
             asm = assembly(\"rig\") do |a|
               a.ground :base, base
               a.part :post, post do
                 mate from: :bottom, to: face(:base, :top)
                 distance_mate from: :bottom, to: face(:base, :top), distance: 5.0
               end
             end
             asm.to_shape",
        )
        .unwrap_err();
    assert!(
        err.contains("conflicting mate constraint"),
        "expected conflicting constraint error, got: {err}"
    );
}
