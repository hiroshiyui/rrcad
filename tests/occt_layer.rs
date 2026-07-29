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

fn assert_rebuild_round_trips(shape: &Shape) {
    let rebuilt = shape.rebuild().expect("rebuild failed");
    assert_eq!(
        shape.shape_type_name().unwrap(),
        rebuilt.shape_type_name().unwrap(),
        "rebuild changed shape type"
    );

    let a = shape.bounding_box().unwrap();
    let b = rebuilt.bounding_box().unwrap();
    for (lhs, rhs) in a.iter().zip(b.iter()) {
        assert!(
            (lhs - rhs).abs() < 1.0e-6,
            "rebuild changed bounding box: original={a:?}, rebuilt={b:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

#[test]
fn primitive_make_box() {
    Shape::make_box(10.0, 20.0, 30.0).expect("make_box failed");
}

#[test]
fn primitive_history_records_constructor() {
    let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box failed");
    let history = shape.history();
    assert_eq!(
        history.len(),
        1,
        "expected one history entry, got: {history:?}"
    );
    assert!(
        history[0].contains("box("),
        "expected box constructor in history, got: {history:?}"
    );
}

#[test]
fn feature_graph_records_dependencies() {
    let shape = Shape::make_box(10.0, 20.0, 30.0)
        .unwrap()
        .translate(5.0, 0.0, 0.0)
        .unwrap()
        .scale(2.0)
        .unwrap();
    let graph = shape.feature_graph();
    let lines: Vec<&str> = graph.lines().collect();
    assert!(
        lines.len() >= 3,
        "expected at least 3 graph nodes, got: {lines:?}"
    );
    let box_id = lines[0].split('\t').next().unwrap();
    let translate_fields: Vec<&str> = lines[1].split('\t').collect();
    let scale_fields: Vec<&str> = lines[2].split('\t').collect();
    assert!(
        translate_fields[2].contains("translate("),
        "missing translate node: {lines:?}"
    );
    assert!(
        scale_fields[2].contains("scale("),
        "missing scale node: {lines:?}"
    );
    assert_eq!(
        translate_fields[1], box_id,
        "translate node should point at the box parent"
    );
    assert_eq!(
        scale_fields[1], translate_fields[0],
        "scale node should point at the translate parent"
    );
}

#[test]
fn feature_rebuild_round_trips_geometry() {
    let shape = Shape::make_box(10.0, 20.0, 30.0)
        .unwrap()
        .translate(5.0, 0.0, 0.0)
        .unwrap()
        .scale(2.0)
        .unwrap();
    let rebuilt = shape.rebuild().expect("rebuild failed");
    assert_eq!(
        shape.shape_type_name().unwrap(),
        rebuilt.shape_type_name().unwrap()
    );
    let a = shape.bounding_box().unwrap();
    let b = rebuilt.bounding_box().unwrap();
    for (lhs, rhs) in a.iter().zip(b.iter()) {
        assert!(
            (lhs - rhs).abs() < 1.0e-6,
            "rebuild changed bounding box: original={a:?}, rebuilt={b:?}"
        );
    }
    let v0 = shape.volume().unwrap();
    let v1 = rebuilt.volume().unwrap();
    assert!(
        (v0 - v1).abs() < 1.0e-6,
        "rebuild changed volume: original={v0}, rebuilt={v1}"
    );
}

#[test]
fn feature_rebuild_round_trips_boolean_fuse() {
    let a = Shape::make_box(10.0, 10.0, 10.0).unwrap();
    let b = Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .translate(5.0, 0.0, 0.0)
        .unwrap();
    let fused = a.fuse(&b).expect("fuse failed");
    assert_rebuild_round_trips(&fused);
}

#[test]
fn feature_rebuild_round_trips_boolean_cut() {
    let base = Shape::make_box(20.0, 20.0, 20.0).unwrap();
    let cyl = Shape::make_cylinder(5.0, 25.0).unwrap();
    let cut = base.cut(&cyl).expect("cut failed");
    assert_rebuild_round_trips(&cut);
}

#[test]
fn feature_rebuild_round_trips_boolean_common() {
    let a = Shape::make_box(20.0, 10.0, 10.0).unwrap();
    let b = Shape::make_box(10.0, 20.0, 10.0).unwrap();
    let common = a.common(&b).expect("common failed");
    assert_rebuild_round_trips(&common);
}

#[test]
fn feature_rebuild_round_trips_rotate_and_mirror() {
    let rotated = Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .rotate(0.0, 0.0, 1.0, 45.0)
        .unwrap();
    assert_rebuild_round_trips(&rotated);

    let mirrored = Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .mirror("xy")
        .unwrap();
    assert_rebuild_round_trips(&mirrored);
}

#[test]
fn feature_rebuild_round_trips_extrude_and_revolve() {
    let extruded = Shape::make_circle_face(2.0).unwrap().extrude(5.0).unwrap();
    assert_rebuild_round_trips(&extruded);

    let revolved = Shape::make_rect(4.0, 2.0).unwrap().revolve(180.0).unwrap();
    assert_rebuild_round_trips(&revolved);
}

#[test]
fn feature_rebuild_round_trips_pattern_and_slice() {
    let linear = Shape::make_box(2.0, 2.0, 2.0)
        .unwrap()
        .linear_pattern(3, 5.0, 0.0, 0.0)
        .unwrap();
    assert_rebuild_round_trips(&linear);

    let polar = Shape::make_box(1.0, 1.0, 5.0)
        .unwrap()
        .translate(3.0, 0.0, 0.0)
        .unwrap()
        .polar_pattern(4, 360.0)
        .unwrap();
    assert_rebuild_round_trips(&polar);

    let sliced = Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .slice("xy", 5.0)
        .unwrap();
    assert_rebuild_round_trips(&sliced);
}

#[test]
fn feature_rebuild_round_trips_surface_ops() {
    let ruled_a = Shape::make_spline_3d(&[
        0.0, 0.0, 0.0, //
        2.0, 0.0, 0.0, //
        2.0, 1.0, 0.0, //
        0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0,
    ])
    .unwrap();
    let ruled_b = ruled_a.translate(0.0, 0.0, 3.0).unwrap();
    let ruled = Shape::ruled_surface(&ruled_a, &ruled_b).expect("ruled_surface failed");
    assert_rebuild_round_trips(&ruled);

    let boundary = Shape::make_arc(5.0, 0.0, 360.0).unwrap();
    let filled = Shape::fill_surface(&boundary).expect("fill_surface failed");
    assert_rebuild_round_trips(&filled);
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

#[test]
fn history_tracks_derivations() {
    let shape = Shape::make_box(10.0, 10.0, 10.0)
        .unwrap()
        .translate(5.0, 0.0, 0.0)
        .unwrap()
        .scale(2.0)
        .unwrap();
    let history = shape.history();
    assert!(
        history.len() >= 3,
        "expected at least three history entries, got: {history:?}"
    );
    assert!(
        history.iter().any(|entry| entry.contains("translate(")),
        "expected translate step in history, got: {history:?}"
    );
    assert!(
        history.iter().any(|entry| entry.contains("scale(")),
        "expected scale step in history, got: {history:?}"
    );
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
        .export_stl(&path, 0.1)
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
