/// End-to-end integration tests for Phase 2 DSL features.
///
/// Tests the full Ruby DSL → mRuby → Rust → OCCT stack for all Phase 2 ops.
use rrcad::ruby::vm::MrubyVm;

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(name)
}

fn assert_valid_step(path: &std::path::Path) {
    assert!(path.exists(), "STEP file not created: {}", path.display());
    let content = std::fs::read_to_string(path).expect("could not read STEP file");
    assert!(
        content.contains("ISO-10303-21"),
        "not a valid STEP file: {}",
        path.display()
    );
}

// ---------------------------------------------------------------------------
// Transforms
// ---------------------------------------------------------------------------

#[test]
fn e2e_translate_export() {
    let out = tmp("rrcad_p2_translate.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).translate(5.0, 0.0, 0.0).export('{}')",
        out.display()
    ))
    .expect("translate export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_rotate_export() {
    let out = tmp("rrcad_p2_rotate.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).rotate(0.0, 0.0, 1.0, 45.0).export('{}')",
        out.display()
    ))
    .expect("rotate export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_scale_export() {
    let out = tmp("rrcad_p2_scale.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).scale(2.0).export('{}')",
        out.display()
    ))
    .expect("scale export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_fillet_export() {
    let out = tmp("rrcad_p2_fillet.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(20.0, 20.0, 20.0).fillet(2.0).export('{}')",
        out.display()
    ))
    .expect("fillet export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_chamfer_export() {
    let out = tmp("rrcad_p2_chamfer.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(20.0, 20.0, 20.0).chamfer(2.0).export('{}')",
        out.display()
    ))
    .expect("chamfer export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_mirror_xy_export() {
    let out = tmp("rrcad_p2_mirror_xy.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 5.0).translate(0.0, 0.0, 1.0).mirror(:xy).export('{}')",
        out.display()
    ))
    .expect("mirror export failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// Sketch + extrude / revolve
// ---------------------------------------------------------------------------

#[test]
fn e2e_rect_extrude_export() {
    let out = tmp("rrcad_p2_rect_extrude.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "rect(15.0, 10.0).extrude(8.0).export('{}')",
        out.display()
    ))
    .expect("rect extrude export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_circle_extrude_export() {
    let out = tmp("rrcad_p2_circle_extrude.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "circle(6.0).extrude(12.0).export('{}')",
        out.display()
    ))
    .expect("circle extrude export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_circle_revolve_full_export() {
    let out = tmp("rrcad_p2_revolve_full.step");
    let mut vm = MrubyVm::new();
    // Translate the circle away from the axis first, then revolve to make a torus-like shape.
    vm.eval(&format!(
        "circle(2.0).translate(5.0, 0.0, 0.0).revolve(360.0).export('{}')",
        out.display()
    ))
    .expect("revolve full export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_circle_revolve_partial_export() {
    let out = tmp("rrcad_p2_revolve_partial.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "circle(3.0).translate(8.0, 0.0, 0.0).revolve(180.0).export('{}')",
        out.display()
    ))
    .expect("revolve partial export failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// solid builder
// ---------------------------------------------------------------------------

#[test]
fn e2e_solid_block_export() {
    let out = tmp("rrcad_p2_solid_block.step");
    let script = format!(
        r#"
result = solid do
  base = box(30.0, 20.0, 10.0)
  hole = cylinder(4.0, 15.0)
  base.cut(hole)
end
result.export('{}')
"#,
        out.display()
    );
    let mut vm = MrubyVm::new();
    vm.eval(&script).expect("solid block export failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// Assembly
// ---------------------------------------------------------------------------

#[test]
fn e2e_assembly_export() {
    let out = tmp("rrcad_p2_assembly.step");
    let script = format!(
        r#"
asm = assembly("bracket") do |a|
  a.place box(20.0, 5.0, 5.0)
  a.place cylinder(2.0, 10.0).translate(10.0, 2.5, 5.0)
end
asm.export('{}')
"#,
        out.display()
    );
    let mut vm = MrubyVm::new();
    vm.eval(&script).expect("assembly export failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// Combined operations
// ---------------------------------------------------------------------------

#[test]
fn e2e_combined_transform_and_boolean() {
    let out = tmp("rrcad_p2_combined.step");
    let script = format!(
        r#"
base   = box(40.0, 40.0, 10.0)
peg    = cylinder(5.0, 20.0).translate(20.0, 20.0, 0.0)
result = base.fuse(peg).fillet(1.0)
result.export('{}')
"#,
        out.display()
    );
    let mut vm = MrubyVm::new();
    vm.eval(&script).expect("combined export failed");
    assert_valid_step(&out);
}
