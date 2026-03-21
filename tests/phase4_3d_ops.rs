/// End-to-end integration tests for Phase 4 3-D operations.
///
/// Covers: loft, shell, offset, and extrude with twist/scale.
/// All tests go through the full Ruby DSL → mRuby → Rust → OCCT stack.
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
// loft
// ---------------------------------------------------------------------------

#[test]
fn loft_two_circles_makes_cone_like_solid() {
    let out = tmp("rrcad_p4_loft_circles.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        p0 = circle(5.0)
        p1 = circle(2.0).translate(0.0, 0.0, 10.0)
        solid = loft([p0, p1])
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("loft two circles failed");
    assert_valid_step(&out);
}

#[test]
fn loft_three_profiles_smooth() {
    let out = tmp("rrcad_p4_loft_three.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        p0 = circle(3.0)
        p1 = circle(6.0).translate(0.0, 0.0, 5.0)
        p2 = circle(2.0).translate(0.0, 0.0, 10.0)
        solid = loft([p0, p1, p2])
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("loft three profiles failed");
    assert_valid_step(&out);
}

#[test]
fn loft_ruled_two_rects() {
    let out = tmp("rrcad_p4_loft_ruled.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        p0 = rect(10.0, 5.0)
        p1 = rect(5.0, 10.0).translate(0.0, 0.0, 8.0)
        solid = loft([p0, p1], ruled: true)
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("loft ruled two rects failed");
    assert_valid_step(&out);
}

#[test]
fn loft_requires_at_least_two_profiles() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("loft([circle(5.0)])");
    assert!(result.is_err(), "loft with one profile should raise");
}

// ---------------------------------------------------------------------------
// shell
// ---------------------------------------------------------------------------

#[test]
fn shell_box_produces_hollow_solid() {
    let out = tmp("rrcad_p4_shell_box.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        b = box(20.0, 20.0, 20.0)
        hollow = b.shell(2.0)
        hollow.export('{}')
        "#,
        out.display()
    ))
    .expect("shell box failed");
    assert_valid_step(&out);
}

#[test]
fn shell_extruded_circle() {
    let out = tmp("rrcad_p4_shell_cylinder.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        cyl = circle(8.0).extrude(15.0)
        hollow = cyl.shell(1.5)
        hollow.export('{}')
        "#,
        out.display()
    ))
    .expect("shell extruded circle failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// offset
// ---------------------------------------------------------------------------

#[test]
fn offset_inflates_box() {
    let out = tmp("rrcad_p4_offset_inflate.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        b = box(10.0, 10.0, 10.0)
        bigger = b.offset(2.0)
        bigger.export('{}')
        "#,
        out.display()
    ))
    .expect("offset inflate box failed");
    assert_valid_step(&out);
}

#[test]
fn offset_deflates_box() {
    let out = tmp("rrcad_p4_offset_deflate.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        b = box(20.0, 20.0, 20.0)
        smaller = b.offset(-2.0)
        smaller.export('{}')
        "#,
        out.display()
    ))
    .expect("offset deflate box failed");
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// extrude with twist and scale
// ---------------------------------------------------------------------------

#[test]
fn extrude_with_twist_produces_shape() {
    let out = tmp("rrcad_p4_extrude_twist.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        solid = rect(5.0, 5.0).extrude(10.0, twist_deg: 45.0)
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("extrude with twist failed");
    assert_valid_step(&out);
}

#[test]
fn extrude_with_scale_tapers_shape() {
    let out = tmp("rrcad_p4_extrude_scale.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        solid = circle(5.0).extrude(12.0, scale: 0.3)
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("extrude with scale failed");
    assert_valid_step(&out);
}

#[test]
fn extrude_with_twist_and_scale() {
    let out = tmp("rrcad_p4_extrude_twist_scale.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        solid = rect(4.0, 4.0).extrude(10.0, twist_deg: 90.0, scale: 0.5)
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("extrude with twist and scale failed");
    assert_valid_step(&out);
}

#[test]
fn extrude_no_kwargs_unchanged() {
    // Without kwargs, extrude must behave identically to the Phase 2 version.
    let out = tmp("rrcad_p4_extrude_plain.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        r#"
        solid = rect(6.0, 4.0).extrude(8.0)
        solid.export('{}')
        "#,
        out.display()
    ))
    .expect("plain extrude via extrude_ex failed");
    assert_valid_step(&out);
}
