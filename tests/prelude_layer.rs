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
// Phase 2: Transforms
// ---------------------------------------------------------------------------

#[test]
fn native_translate_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(
        &mut vm,
        "box(10.0,10.0,10.0).translate(5.0,0.0,0.0).class",
        "Shape",
    );
}

#[test]
fn native_rotate_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(
        &mut vm,
        "box(10.0,10.0,10.0).rotate(0.0,0.0,1.0,45.0).class",
        "Shape",
    );
}

#[test]
fn native_scale_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(10.0,10.0,10.0).scale(2.0).class", "Shape");
}

#[test]
fn native_fillet_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(10.0,10.0,10.0).fillet(1.0).class", "Shape");
}

#[test]
fn native_chamfer_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(10.0,10.0,10.0).chamfer(1.0).class", "Shape");
}

#[test]
fn native_mirror_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "box(10.0,10.0,10.0).mirror(:xy).class", "Shape");
}

// ---------------------------------------------------------------------------
// Phase 2: Sketch + extrude/revolve
// ---------------------------------------------------------------------------

#[test]
fn native_rect_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "rect(10.0, 20.0).class", "Shape");
}

#[test]
fn native_circle_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "circle(5.0).class", "Shape");
}

#[test]
fn native_rect_extrude_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "rect(10.0, 20.0).extrude(5.0).class", "Shape");
}

#[test]
fn native_circle_revolve_returns_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "circle(5.0).revolve(360.0).class", "Shape");
}

#[test]
fn native_revolve_default_angle_is_full() {
    let mut vm = MrubyVm::new();
    // revolve() with no args should succeed (defaults to 360°)
    assert_contains(&mut vm, "circle(5.0).revolve.class", "Shape");
}

// ---------------------------------------------------------------------------
// Phase 2: solid builder
// ---------------------------------------------------------------------------

#[test]
fn solid_block_returns_last_shape() {
    let mut vm = MrubyVm::new();
    assert_contains(&mut vm, "solid { box(5.0, 5.0, 5.0) }.class", "Shape");
}

#[test]
fn solid_block_can_chain_operations() {
    let mut vm = MrubyVm::new();
    let code = "solid { box(10.0, 10.0, 10.0).fuse(sphere(6.0)) }.class";
    assert_contains(&mut vm, code, "Shape");
}

// ---------------------------------------------------------------------------
// Phase 2: Assembly
// ---------------------------------------------------------------------------

#[test]
fn assembly_can_place_shapes() {
    let mut vm = MrubyVm::new();
    let code = r#"
      asm = assembly("test") do |a|
        a.place box(5.0, 5.0, 5.0)
        a.place sphere(3.0)
      end
      asm.class
    "#;
    assert_contains(&mut vm, code, "Assembly");
}

#[test]
fn assembly_inspect_contains_name() {
    let mut vm = MrubyVm::new();
    let code = r#"assembly("bracket") { }.inspect"#;
    assert_contains(&mut vm, code, "bracket");
}

// ---------------------------------------------------------------------------
// Phase 3+ stubs still raise NotImplementedError
// ---------------------------------------------------------------------------

#[test]
fn native_preview_no_op_outside_preview_mode() {
    // When PREVIEW is not initialised (non --preview run), preview(shape)
    // should succeed silently rather than raising an error.
    let mut vm = MrubyVm::new();
    let r = vm.eval("$s = box(5.0, 5.0, 5.0); preview($s)");
    assert!(r.is_ok(), "preview() raised outside --preview mode: {r:?}");
}

#[test]
fn native_faces_selector_returns_array() {
    let mut vm = MrubyVm::new();
    let r = vm
        .eval("box(5.0, 5.0, 5.0).faces(:top).class")
        .expect("faces(:top) should not raise");
    assert!(r.contains("Array"), "expected Array, got: {r}");
}

#[test]
fn native_edges_selector_returns_array() {
    let mut vm = MrubyVm::new();
    let r = vm
        .eval("box(5.0, 5.0, 5.0).edges(:vertical).class")
        .expect("edges(:vertical) should not raise");
    assert!(r.contains("Array"), "expected Array, got: {r}");
}

#[test]
fn stub_assembly_mate_mentions_phase5() {
    let mut vm = MrubyVm::new();
    let code = r#"
      $asm = assembly("test") {}
      $s = box(5.0, 5.0, 5.0)
      $asm.mate($s)
    "#;
    assert_err_contains(&mut vm, code, "Phase 5");
}
