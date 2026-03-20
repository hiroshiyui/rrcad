/// Integration tests for the DSL prelude + native binding layer.
///
/// The prelude is auto-loaded and native methods are registered during
/// `MrubyVm::new()`.  These tests verify the expected DSL surface without
/// any `require` statement.
///
/// Test groups:
///   auto-load        — prelude + native bindings present without require
///   Shape class      — class identity, stub-instance construction
///   native prims     — box / cylinder / sphere return real Shape objects
///   native methods   — export / fuse / cut / common work end-to-end
///   stubs (Phase 2+) — translate, rotate, scale, fillet, chamfer, solid,
///                      preview still raise NotImplementedError with phase tag
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
    let mut vm = MrubyVm::new();
    vm.eval("Shape")
        .expect("Shape should be defined without require");
}

#[test]
fn box_available_without_require() {
    let mut vm = MrubyVm::new();
    // box() is now native — it should return a Shape, not raise.
    assert_contains(&mut vm, "box(10.0, 20.0, 30.0).class", "Shape");
}

// ---------------------------------------------------------------------------
// Shape class identity
// ---------------------------------------------------------------------------

#[test]
fn shape_class_is_defined() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "Shape.class", "Class");
}

#[test]
fn shape_new_returns_shape_instance() {
    let mut vm = MrubyVm::new();
    // Shape.new still works (uses the prelude initialize) and returns a Shape.
    assert_contains(&mut vm, "Shape.new(:box, 10, 20).class", "Shape");
}

#[test]
fn shape_to_s_same_as_inspect() {
    let mut vm = MrubyVm::new();
    // Native Shape: both to_s and inspect return "#<Shape>".
    let result = vm
        .eval("s = box(5.0, 5.0, 5.0); s.to_s == s.inspect")
        .unwrap();
    assert_eq!(result, "true");
}

// ---------------------------------------------------------------------------
// Native primitives
// ---------------------------------------------------------------------------

#[test]
fn native_box_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(10.0, 20.0, 30.0).class", "Shape");
}

#[test]
fn native_box_integer_args_coerced() {
    // mRuby coerces integer args to float for format "f" in mrb_get_args.
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(10, 20, 30).class", "Shape");
}

#[test]
fn native_cylinder_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "cylinder(5.0, 15.0).class", "Shape");
}

#[test]
fn native_sphere_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "sphere(8.0).class", "Shape");
}

#[test]
fn native_shape_inspect_returns_shape_string() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(1.0, 2.0, 3.0).inspect", "#<Shape>");
}

// ---------------------------------------------------------------------------
// Native methods: export
// ---------------------------------------------------------------------------

#[test]
fn native_shape_export_step_creates_file() {
    let out = std::env::temp_dir().join("rrcad_prelude_export_test.step");
    let code = format!("box(10.0, 20.0, 30.0).export('{}')", out.display());
    let mut vm = MrubyVm::new();
    vm.eval(&code).expect("export should succeed");
    assert!(out.exists(), "STEP file not created");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("ISO-10303-21"), "not a valid STEP file");
}

// ---------------------------------------------------------------------------
// Native methods: boolean operations
// ---------------------------------------------------------------------------

#[test]
fn native_shape_fuse_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(
        &mut vm,
        "box(20.0, 20.0, 20.0).fuse(sphere(12.0)).class",
        "Shape",
    );
}

#[test]
fn native_shape_cut_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(
        &mut vm,
        "box(20.0, 20.0, 20.0).cut(cylinder(5.0, 25.0)).class",
        "Shape",
    );
}

#[test]
fn native_shape_common_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(
        &mut vm,
        "box(20.0, 20.0, 20.0).common(sphere(12.0)).class",
        "Shape",
    );
}

// ---------------------------------------------------------------------------
// Stubs — Phase 2 methods still raise NotImplementedError
// ---------------------------------------------------------------------------

#[test]
fn stub_shape_translate_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "$s = box(5.0, 5.0, 5.0); $s.translate(1, 2, 3)",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_rotate_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "$s = box(5.0, 5.0, 5.0); $s.rotate(0, 0, 1, 45)",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_scale_raises_not_implemented() {
    let mut vm = MrubyVm::new();
    assert_err_contains(
        &mut vm,
        "$s = box(5.0, 5.0, 5.0); $s.scale(2)",
        "NotImplementedError",
    );
}

#[test]
fn stub_shape_fillet_mentions_phase2() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "$s = box(5.0, 5.0, 5.0); $s.fillet(1)", "Phase 2");
}

#[test]
fn stub_shape_chamfer_mentions_phase2() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "$s = box(5.0, 5.0, 5.0); $s.chamfer(1)", "Phase 2");
}

#[test]
fn stub_solid_block_mentions_phase2() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "solid {}", "Phase 2");
}

#[test]
fn stub_preview_mentions_phase3() {
    let mut vm = MrubyVm::new();
    assert_err_contains(&mut vm, "$s = box(5.0, 5.0, 5.0); preview($s)", "Phase 3");
}
