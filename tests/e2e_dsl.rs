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
