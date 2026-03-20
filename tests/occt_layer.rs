/// Integration tests for the OCCT geometry layer (`rrcad::occt::Shape`).
///
/// Each test group covers one facet of the bridge:
///   primitives  — make_box / make_cylinder / make_sphere
///   booleans    — fuse / cut / common
///   modifiers   — fillet / chamfer
///   transforms  — translate / rotate / scale
///   export      — STEP / STL / glTF
///
/// All tests write output to `std::env::temp_dir()` so they leave no
/// artefacts in the source tree.
use rrcad::occt::Shape;
use std::fs;

fn tmp(name: &str) -> String {
    std::env::temp_dir().join(name).to_str().unwrap().to_owned()
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

#[test]
fn primitive_make_box() {
    Shape::make_box(10.0, 20.0, 30.0).expect("make_box failed");
}

#[test]
fn primitive_make_cylinder() {
    Shape::make_cylinder(5.0, 15.0).expect("make_cylinder failed");
}

#[test]
fn primitive_make_sphere() {
    Shape::make_sphere(8.0).expect("make_sphere failed");
}

// ---------------------------------------------------------------------------
// Boolean operations
// ---------------------------------------------------------------------------

#[test]
fn boolean_fuse() {
    let a = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    let b = Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .translate(5.0, 0.0, 0.0)
        .unwrap();
    a.fuse(&b).expect("fuse failed");
}

#[test]
fn boolean_cut() {
    let base = Shape::make_box(20.0, 20.0, 20.0).unwrap();
    let cyl = Shape::make_cylinder(5.0, 25.0).unwrap();
    base.cut(&cyl).expect("cut failed");
}

#[test]
fn boolean_common() {
    let a = Shape::make_box(20.0, 10.0, 10.0).unwrap();
    let b = Shape::make_box(10.0, 20.0, 10.0).unwrap();
    a.common(&b).expect("common failed");
}

// ---------------------------------------------------------------------------
// Modifiers
// ---------------------------------------------------------------------------

#[test]
fn modifier_fillet() {
    let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    b.fillet(1.0).expect("fillet failed");
}

#[test]
fn modifier_chamfer() {
    let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    b.chamfer(1.0).expect("chamfer failed");
}

// ---------------------------------------------------------------------------
// Transforms
// ---------------------------------------------------------------------------

#[test]
fn transform_translate() {
    let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    b.translate(5.0, -3.0, 0.0).expect("translate failed");
}

#[test]
fn transform_rotate() {
    let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    // 45° around Z axis
    b.rotate(0.0, 0.0, 1.0, 45.0).expect("rotate failed");
}

#[test]
fn transform_scale() {
    let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    b.scale(2.0).expect("scale failed");
}

// ---------------------------------------------------------------------------
// Export — STEP
// ---------------------------------------------------------------------------

#[test]
fn export_step_file_created() {
    let path = tmp("rrcad_test_export.step");
    Shape::make_box(5.0, 5.0, 5.0)
        .unwrap()
        .export_step(&path)
        .expect("export_step failed");
    assert!(std::path::Path::new(&path).exists());
}

#[test]
fn export_step_valid_header() {
    let path = tmp("rrcad_test_step_header.step");
    Shape::make_box(5.0, 5.0, 5.0)
        .unwrap()
        .export_step(&path)
        .unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("ISO-10303-21"),
        "STEP file missing ISO-10303-21 header"
    );
}

#[test]
fn export_step_filleted_box() {
    let path = tmp("rrcad_test_filleted.step");
    Shape::make_box(20.0, 20.0, 20.0)
        .unwrap()
        .fillet(2.0)
        .unwrap()
        .export_step(&path)
        .expect("export filleted box to STEP failed");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("ISO-10303-21"));
}

#[test]
fn export_step_boolean_result() {
    let path = tmp("rrcad_test_cut.step");
    let base = Shape::make_box(20.0, 20.0, 20.0).unwrap();
    let hole = Shape::make_cylinder(4.0, 25.0).unwrap();
    base.cut(&hole)
        .unwrap()
        .export_step(&path)
        .expect("export boolean cut to STEP failed");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("ISO-10303-21"));
}

// ---------------------------------------------------------------------------
// Export — STL
// ---------------------------------------------------------------------------

#[test]
fn export_stl_file_created() {
    let path = tmp("rrcad_test_export.stl");
    Shape::make_sphere(5.0)
        .unwrap()
        .export_stl(&path)
        .expect("export_stl failed");
    assert!(std::path::Path::new(&path).exists());
    assert!(fs::metadata(&path).unwrap().len() > 0, "STL file is empty");
}

// ---------------------------------------------------------------------------
// Export — glTF
// ---------------------------------------------------------------------------

#[test]
fn export_gltf_file_created() {
    let path = tmp("rrcad_test_export.glb");
    Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .export_gltf(&path, 0.1)
        .expect("export_gltf failed");
    assert!(std::path::Path::new(&path).exists());
    assert!(fs::metadata(&path).unwrap().len() > 0, "glTF file is empty");
}
