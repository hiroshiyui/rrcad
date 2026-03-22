/// Phase 7 Tier 1 integration tests
///
/// Covers: asymmetric chamfer, 2D profile offset, grid_pattern, fuse_all, cut_all.
use rrcad::ruby::vm::MrubyVm;
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from("target/e2e_test_outputs");
    std::fs::create_dir_all(&dir).expect("could not create e2e output directory");
    dir.join(name)
}

fn assert_valid_step(path: &std::path::Path) {
    assert!(path.exists(), "STEP file not created: {path:?}");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("ISO-10303"),
        "not a valid STEP file: {path:?}"
    );
}

// ---------------------------------------------------------------------------
// Asymmetric chamfer
// ---------------------------------------------------------------------------

#[test]
fn chamfer_asym_all_edges() {
    let mut vm = MrubyVm::new();
    let out = tmp("chamfer_asym_all.step");
    let code = format!(
        r#"
b = box(20, 20, 20)
c = b.chamfer_asym(4, 1)
c.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("chamfer_asym all edges failed");
    assert_valid_step(&out);
}

#[test]
fn chamfer_asym_selective() {
    let mut vm = MrubyVm::new();
    let out = tmp("chamfer_asym_sel.step");
    let code = format!(
        r#"
b = box(20, 20, 20)
c = b.chamfer_asym(3, 1, :vertical)
c.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("chamfer_asym selective failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// 2D profile offset
// ---------------------------------------------------------------------------

#[test]
fn offset_2d_face_outward() {
    let mut vm = MrubyVm::new();
    let out = tmp("offset_2d_face.step");
    let code = format!(
        r#"
f = rect(10, 10)
g = f.offset_2d(2.0)
g.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("offset_2d face outward failed");
    assert_valid_step(&out);
}

#[test]
fn offset_2d_face_inward() {
    let mut vm = MrubyVm::new();
    let out = tmp("offset_2d_inward.step");
    let code = format!(
        r#"
f = rect(20, 20)
g = f.offset_2d(-3.0)
g.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("offset_2d face inward failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// grid_pattern
// ---------------------------------------------------------------------------

#[test]
fn grid_pattern_2x3() {
    let mut vm = MrubyVm::new();
    let out = tmp("grid_pattern_2x3.step");
    let code = format!(
        r#"
bolt = cylinder(2, 8)
grid = grid_pattern(bolt, 2, 3, 15, 15)
grid.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("grid_pattern 2x3 failed");
    assert_valid_step(&out);
}

#[test]
fn grid_pattern_1x1_is_identity() {
    let mut vm = MrubyVm::new();
    // A 1×1 grid should produce the original shape (as a compound).
    let result = vm
        .eval("grid_pattern(box(5, 5, 5), 1, 1, 0, 0).class")
        .expect("grid_pattern 1x1 failed");
    assert_eq!(result.trim(), "Shape");
}

#[test]
fn grid_pattern_invalid_dims() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("grid_pattern(box(5, 5, 5), 0, 3, 10, 10)")
        .unwrap_err();
    assert!(
        err.contains("nx") || err.contains("ny") || err.contains(">= 1"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// fuse_all
// ---------------------------------------------------------------------------

#[test]
fn fuse_all_two_shapes() {
    let mut vm = MrubyVm::new();
    let out = tmp("fuse_all_two.step");
    let code = format!(
        r#"
a = box(10, 10, 10)
b = box(10, 10, 10).translate(8, 0, 0)
result = fuse_all([a, b])
result.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("fuse_all two shapes failed");
    assert_valid_step(&out);
}

#[test]
fn fuse_all_three_shapes() {
    let mut vm = MrubyVm::new();
    let out = tmp("fuse_all_three.step");
    let code = format!(
        r#"
a = sphere(5)
b = sphere(5).translate(6, 0, 0)
c = sphere(5).translate(3, 5, 0)
result = fuse_all([a, b, c])
result.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("fuse_all three shapes failed");
    assert_valid_step(&out);
}

#[test]
fn fuse_all_requires_two_shapes() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("fuse_all([box(5, 5, 5)])").unwrap_err();
    assert!(
        err.contains("2") || err.contains("least"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// cut_all
// ---------------------------------------------------------------------------

#[test]
fn cut_all_two_tools() {
    let mut vm = MrubyVm::new();
    let out = tmp("cut_all_two.step");
    let code = format!(
        r#"
base = box(40, 40, 20)
h1 = cylinder(3, 25).translate(10, 10, -2)
h2 = cylinder(3, 25).translate(30, 10, -2)
result = cut_all(base, [h1, h2])
result.export("{}")
"#,
        out.display()
    );
    vm.eval(&code).expect("cut_all two tools failed");
    assert_valid_step(&out);
}

#[test]
fn cut_all_requires_one_tool() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("cut_all(box(10, 10, 10), [])").unwrap_err();
    assert!(
        err.contains("1") || err.contains("tool") || err.contains("least"),
        "unexpected error: {err}"
    );
}
