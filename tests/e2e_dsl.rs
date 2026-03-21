/// End-to-end integration tests: Ruby DSL → mRuby → Rust → OCCT → file.
///
/// These tests exercise the full stack as a user would: a Ruby script string
/// is passed to `MrubyVm::eval`, which drives OCCT via the native bindings
/// and produces an output file on disk.
use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(name)
}

fn assert_valid_step(path: &std::path::Path) {
    assert!(path.exists(), "STEP file not created: {}", path.display());
    let content = std::fs::read_to_string(path).expect("could not read STEP file");
    assert!(
        content.contains("ISO-10303-21"),
        "output does not look like a STEP file: {}",
        path.display()
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_box_export_step() {
    let out = tmp("rrcad_e2e_box.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 20.0, 30.0).export('{}')",
        out.display()
    ))
    .expect("e2e box export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_cylinder_export_step() {
    let out = tmp("rrcad_e2e_cylinder.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!("cylinder(5.0, 20.0).export('{}')", out.display()))
        .expect("e2e cylinder export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_sphere_export_step() {
    let out = tmp("rrcad_e2e_sphere.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!("sphere(8.0).export('{}')", out.display()))
        .expect("e2e sphere export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_boolean_cut_export_step() {
    // Box with cylindrical hole — classic CAD workflow.
    let out = tmp("rrcad_e2e_cut.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(20.0, 20.0, 20.0).cut(cylinder(5.0, 25.0)).export('{}')",
        out.display()
    ))
    .expect("e2e cut export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_fuse_export_step() {
    let out = tmp("rrcad_e2e_fuse.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).fuse(sphere(7.0)).export('{}')",
        out.display()
    ))
    .expect("e2e fuse export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_common_export_step() {
    let out = tmp("rrcad_e2e_common.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(20.0, 20.0, 20.0).common(sphere(12.0)).export('{}')",
        out.display()
    ))
    .expect("e2e common export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_multi_statement_script() {
    // Simulate a real .rb script with multiple lines.
    let out = tmp("rrcad_e2e_script.step");
    let script = format!(
        r#"
base = box(30.0, 20.0, 10.0)
hole = cylinder(4.0, 15.0)
result = base.cut(hole)
result.export('{}')
"#,
        out.display()
    );
    let mut vm = MrubyVm::new();
    vm.eval(&script).expect("e2e multi-statement script failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_shape_assigned_to_global_and_reused() {
    // Shapes stored in globals survive across eval calls.
    let out = tmp("rrcad_e2e_global_shape.step");
    let mut vm = MrubyVm::new();
    vm.eval("$base = box(15.0, 15.0, 15.0)").unwrap();
    vm.eval("$tool = cylinder(4.0, 20.0)").unwrap();
    vm.eval("$result = $base.cut($tool)").unwrap();
    vm.eval(&format!("$result.export('{}')", out.display()))
        .unwrap();
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// Query / introspection
// ---------------------------------------------------------------------------

#[test]
fn e2e_bounding_box() {
    let mut vm = MrubyVm::new();
    // box(10, 20, 30) at origin → min corner (0,0,0), extents (10,20,30)
    let result = vm
        .eval("bb = box(10.0, 20.0, 30.0).bounding_box; [bb[:dx], bb[:dy], bb[:dz]].inspect")
        .expect("bounding_box failed");
    assert!(
        result.contains("10.0") && result.contains("20.0") && result.contains("30.0"),
        "unexpected bounding box extents: {result}"
    );
}

#[test]
fn e2e_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 20.0, 30.0).volume")
        .expect("volume failed");
    // 10 × 20 × 30 = 6000.0
    let vol: f64 = result.trim().parse().expect("volume result not a float");
    assert!(
        (vol - 6000.0).abs() < 1.0,
        "expected volume ≈ 6000, got {vol}"
    );
}

#[test]
fn e2e_surface_area() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 20.0, 30.0).surface_area")
        .expect("surface_area failed");
    // 2*(10*20 + 20*30 + 10*30) = 2*(200+600+300) = 2200.0
    let area: f64 = result
        .trim()
        .parse()
        .expect("surface_area result not a float");
    assert!(
        (area - 2200.0).abs() < 1.0,
        "expected surface_area ≈ 2200, got {area}"
    );
}

// ---------------------------------------------------------------------------
// Import round-trips
// ---------------------------------------------------------------------------

#[test]
fn e2e_import_step_roundtrip() {
    // Export a box to STEP then import it back; the result must be a Shape.
    let step = tmp("rrcad_e2e_import_step.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).export('{}')",
        step.display()
    ))
    .expect("export failed");
    assert_valid_step(&step);

    let result = vm
        .eval(&format!("import_step('{}').class", step.display()))
        .expect("import_step failed");
    assert!(result.contains("Shape"), "expected Shape, got: {result}");
}

// ---------------------------------------------------------------------------
// scale — uniform and non-uniform
// ---------------------------------------------------------------------------

#[test]
fn e2e_scale_uniform() {
    // scale(2) doubles all extents: box(5,5,5).scale(2) → extents 10×10×10
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(5.0, 5.0, 5.0).scale(2.0).bounding_box")
        .expect("scale uniform failed");
    // bounding_box returns {x:, y:, z:, dx:, dy:, dz:}; check dx (extent X) is 10
    assert!(
        result.contains("dx: 10"),
        "unexpected bounding_box: {result}"
    );
}

#[test]
fn e2e_scale_nonuniform() {
    // scale(sx, sy, sz) stretches each axis independently.
    // box(1,1,1).scale(2, 3, 4) → extents 2×3×4
    let mut vm = MrubyVm::new();
    let bb = vm
        .eval("box(1.0, 1.0, 1.0).scale(2.0, 3.0, 4.0).bounding_box")
        .expect("scale non-uniform failed");
    assert!(bb.contains("dx: 2"), "expected dx: 2, got: {bb}");
    assert!(bb.contains("dy: 3"), "expected dy: 3, got: {bb}");
    assert!(bb.contains("dz: 4"), "expected dz: 4, got: {bb}");
}

#[test]
fn e2e_scale_nonuniform_export_step() {
    // Verify that a non-uniformly scaled shape produces a valid STEP file.
    let out = tmp("rrcad_e2e_scale_xyz.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).scale(1.5, 2.0, 0.5).export('{}')",
        out.display()
    ))
    .expect("scale_xyz export failed");
    assert_valid_step(&out);
}
#[test]
fn e2e_import_stl_roundtrip() {
    // Write a sphere to STL via the Rust API, then import it through the DSL.
    let stl = tmp("rrcad_e2e_import_stl.stl");
    rrcad::occt::Shape::make_sphere(5.0)
        .unwrap()
        .export_stl(stl.to_str().unwrap())
        .unwrap();
    assert!(stl.exists(), "STL file not created");

    let mut vm = MrubyVm::new();
    let result = vm
        .eval(&format!("import_stl('{}').class", stl.display()))
        .expect("import_stl failed");
    assert!(result.contains("Shape"), "expected Shape, got: {result}");
}
