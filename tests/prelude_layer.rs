/// Integration tests for the DSL prelude layer.
///
/// The prelude is auto-loaded during `MrubyVm::new()`.  These tests verify
/// that the expected DSL surface is present and behaves correctly without any
/// `require` statement.
///
/// Test groups:
///   auto-load   — prelude is present without explicit require
///   Shape class — constructor, inspect format
///   stubs       — every stub raises NotImplementedError with the correct
///                 phase tag so users know when each feature lands
use rrcad::ruby::vm::MrubyVm;

// Convenience: assert eval returns Ok and the result string contains `needle`.
fn assert_contains(vm: &mut MrubyVm, code: &str, needle: &str) {
    let result = vm.eval(code).unwrap_or_else(|e| panic!("eval failed: {e}"));
    assert!(
        result.contains(needle),
        "expected {needle:?} in {result:?} (code: {code})"
    );
}

// Convenience: assert eval returns Err and the error string contains `needle`.
fn assert_err_contains(vm: &mut MrubyVm, code: &str, needle: &str) {
    let err = vm
        .eval(code)
        .expect_err(&format!("expected Err for: {code}"));
    assert!(
        err.contains(needle),
        "expected {needle:?} in error {err:?} (code: {code})"
    );
}

// ---------------------------------------------------------------------------
// Auto-load — no require needed
// ---------------------------------------------------------------------------

#[test]
fn prelude_loaded_without_require() {
    // If the prelude were not loaded, `Shape` would be undefined and this
    // would raise NameError.  A successful eval proves auto-loading works.
    let mut vm = MrubyVm::new();
    vm.eval("Shape")
        .expect("Shape should be defined without require");
}

#[test]
fn box_available_without_require() {
    let mut vm = MrubyVm::new();
    // Calling box() should raise NotImplementedError, not NameError.
    let err = vm.eval("box(1,2,3)").unwrap_err();
    assert!(
        !err.contains("NoMethodError"),
        "box() raised NoMethodError — prelude not loaded? err={err}"
    );
}

// ---------------------------------------------------------------------------
// Shape class
// ---------------------------------------------------------------------------

#[test]
fn shape_class_is_defined() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "Shape.class", "Class");
}

#[test]
fn shape_new_returns_shape_instance() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "Shape.new(:box, 10, 20).class", "Shape");
}

#[test]
fn shape_inspect_contains_kind() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "Shape.new(:sphere, 5).inspect", "sphere");
}

#[test]
fn shape_inspect_contains_args() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "Shape.new(:box, 10, 20, 30).inspect", "10");
    assert_contains(&mut vm, "Shape.new(:box, 10, 20, 30).inspect", "20");
    assert_contains(&mut vm, "Shape.new(:box, 10, 20, 30).inspect", "30");
}

#[test]
fn shape_to_s_same_as_inspect() {
    let mut vm = MrubyVm::new();
    // Single eval so the local variable `s` is in scope for both method calls.
    let result = vm
        .eval("s = Shape.new(:cyl, 5, 10); s.to_s == s.inspect")
        .unwrap();
    assert_eq!(
        result, "true",
        "to_s and inspect should return the same string"
    );
}

// ---------------------------------------------------------------------------
// Stubs — primitives raise NotImplementedError (Phase 1)
// ---------------------------------------------------------------------------

#[test]
fn stub_box_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "box(10, 20, 30)", "NotImplementedError");
}

#[test]
fn stub_box_error_mentions_phase() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "box(1, 2, 3)", "Phase 1");
}

#[test]
fn stub_cylinder_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "cylinder(5, 10)", "NotImplementedError");
}

#[test]
fn stub_sphere_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "sphere(8)", "NotImplementedError");
}

// ---------------------------------------------------------------------------
// Stubs — Shape instance methods raise NotImplementedError
// ---------------------------------------------------------------------------

#[test]
fn stub_shape_export_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "Shape.new(:box).export('out.step')",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_fuse_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "Shape.new(:box).fuse(Shape.new(:box))",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_cut_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "Shape.new(:box).cut(Shape.new(:box))",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_common_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "Shape.new(:box).common(Shape.new(:box))",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_translate_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "Shape.new(:box).translate(1,2,3)",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_rotate_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "Shape.new(:box).rotate(0,0,1,45)",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_scale_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "Shape.new(:box).scale(2)", "NotImplementedError");
}

// ---------------------------------------------------------------------------
// Stubs — Phase 2 / Phase 3 features carry the right phase tag
// ---------------------------------------------------------------------------

#[test]
fn stub_shape_fillet_mentions_phase2() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "Shape.new(:box).fillet(1)", "Phase 2");
}

#[test]
fn stub_shape_chamfer_mentions_phase2() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "Shape.new(:box).chamfer(1)", "Phase 2");
}

#[test]
fn stub_solid_block_mentions_phase2() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "solid {}", "Phase 2");
}

#[test]
fn stub_preview_mentions_phase3() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "preview(Shape.new(:box))", "Phase 3");
}
