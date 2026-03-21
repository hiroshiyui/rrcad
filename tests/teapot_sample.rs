/// Smoke test for the Utah Teapot sample script (samples/07_teapot.rb).
///
/// Runs the full Ruby DSL → mRuby → Rust → OCCT pipeline and verifies
/// that a valid STEP file is produced.  Individual part tests verify that
/// each component builds without error before the final assembly fuse.
use rrcad::ruby::vm::MrubyVm;
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
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
// Component tests — each part in isolation
// ---------------------------------------------------------------------------

#[test]
fn teapot_body_loft_succeeds() {
    let out = tmp("rrcad_teapot_body.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        body = loft([
          circle(3.00).translate(0, 0, 0.00),
          circle(5.25).translate(0, 0, 0.50),
          circle(5.80).translate(0, 0, 1.00),
          circle(7.00).translate(0, 0, 2.00),
          circle(6.80).translate(0, 0, 3.00),
          circle(6.13).translate(0, 0, 4.50),
          circle(4.90).translate(0, 0, 5.50),
          circle(4.90).translate(0, 0, 7.50),
        ])
        body.export('{}')
        "#,
        out.display()
    ))
    .expect("teapot body loft failed");
    assert_valid_step(&out);
}

#[test]
fn teapot_spout_sweep_succeeds() {
    let out = tmp("rrcad_teapot_spout.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        path = spline_3d([
          [ 4.00, 0.0, 1.50],
          [ 6.50, 0.0, 2.20],
          [ 9.50, 0.0, 5.00],
          [12.25, 0.0, 8.00],
        ])
        spout = circle(1.30).sweep(path)
        spout.export('{}')
        "#,
        out.display()
    ))
    .expect("teapot spout sweep failed");
    assert_valid_step(&out);
}

#[test]
fn teapot_handle_sweep_succeeds() {
    let out = tmp("rrcad_teapot_handle.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        path = spline_3d([
          [-3.50,  0.0, 1.50],
          [-7.00,  0.0, 2.00],
          [-10.50, 0.0, 4.50],
          [-7.00,  0.0, 6.80],
          [-3.50,  0.0, 7.00],
        ])
        handle = circle(1.00).sweep(path)
        handle.export('{}')
        "#,
        out.display()
    ))
    .expect("teapot handle sweep failed");
    assert_valid_step(&out);
}

#[test]
fn teapot_lid_loft_succeeds() {
    let out = tmp("rrcad_teapot_lid.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        lid = loft([
          circle(0.30).translate(0, 0, 8.70),
          circle(1.50).translate(0, 0, 8.50),
          circle(3.00).translate(0, 0, 8.10),
          circle(4.00).translate(0, 0, 7.70),
          circle(5.00).translate(0, 0, 7.40),
        ])
        lid.export('{}')
        "#,
        out.display()
    ))
    .expect("teapot lid loft failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// Full assembly test
// ---------------------------------------------------------------------------

#[test]
fn teapot_full_assembly_succeeds() {
    let out = tmp("rrcad_teapot_full.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        body = loft([
          circle(3.00).translate(0, 0, 0.00),
          circle(5.25).translate(0, 0, 0.50),
          circle(5.80).translate(0, 0, 1.00),
          circle(7.00).translate(0, 0, 2.00),
          circle(6.80).translate(0, 0, 3.00),
          circle(6.13).translate(0, 0, 4.50),
          circle(4.90).translate(0, 0, 5.50),
          circle(4.90).translate(0, 0, 7.50),
        ])

        spout_path = spline_3d([
          [ 4.00, 0.0, 1.50],
          [ 6.50, 0.0, 2.80],
          [ 9.50, 0.0, 4.50],
          [12.00, 0.0, 5.80],
          [14.00, 0.0, 6.50],
        ])
        spout = circle(1.80).sweep(spout_path)

        handle_path = spline_3d([
          [-3.50,  0.0, 1.50],
          [-7.00,  0.0, 2.00],
          [-10.50, 0.0, 4.50],
          [-7.00,  0.0, 6.80],
          [-3.50,  0.0, 7.00],
        ])
        handle = circle(1.00).sweep(handle_path)

        lid = loft([
          circle(0.30).translate(0, 0, 8.70),
          circle(1.50).translate(0, 0, 8.50),
          circle(3.00).translate(0, 0, 8.10),
          circle(4.00).translate(0, 0, 7.70),
          circle(5.00).translate(0, 0, 7.40),
        ])
        knob = sphere(1.20).translate(0, 0, 9.10)
        lid_assy = lid.fuse(knob)

        body_handle = body.fuse(handle)
        body_handle_spout = body_handle.fuse(spout)
        teapot = body_handle_spout.fuse(lid_assy)
        teapot.export('{}')
        "#,
        out.display()
    ))
    .expect("teapot full assembly failed");
    assert_valid_step(&out);
}
