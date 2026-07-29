// `volume` on shapes that are not solids.
//
// An open surface encloses nothing, but asking OCCT for its volume anyway does
// not produce zero — it produces a confident, plausible, wrong number. The
// divergence integral over a surface with a boundary depends on where that
// boundary happens to be, so a ruled surface between two square loops reported
// 517.9 for a region whose honest answer is "not a volume". Nothing said so,
// and the number flowed on into `mass_estimate` and an assembly's mass rollup.
//
// The guard has to be narrow, and these tests exist mostly to pin the cases
// that must NOT be caught by it. Two obvious wider rules are both wrong:
//
//   * "the shape must be closed?" would reject a sphere, every boolean result
//     and every imported mesh — OCCT reports all of them as not closed, since
//     seam and degenerate edges belong to a single face, while computing their
//     volumes perfectly.
//   * "the shape must contain a Solid" would zero out `import_stl(...).volume`,
//     which is a Compound of triangles containing no Solid at all and whose
//     volume is exactly that of the box it came from.
//
// So the test is specifically: a Shell carrying a free boundary edge.

use rrcad::ruby::vm::MrubyVm;

fn eval(script: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(script)
        .unwrap_or_else(|e| panic!("script failed: {e}\n{script}"))
        .trim()
        .to_string()
}

fn number(script: &str) -> f64 {
    eval(script).parse().expect("expected a number")
}

/// Two square loops at different heights, joined by a ruled surface: a tube
/// with no top or bottom, and the shape that exposed this.
const OPEN_TUBE: &str = "ruled_surface(\
    spline_3d([[0,0,0],[10,0,0],[10,10,0],[0,10,0],[0,0,0]]), \
    spline_3d([[0,0,5],[10,0,5],[10,10,5],[0,10,5],[0,0,5]]))";

#[test]
fn an_open_shell_has_no_volume() {
    // The defect. 517.918... was the previous answer: close enough to the 500
    // of the region it appears to bound that it would survive a glance.
    assert_eq!(eval(&format!("{OPEN_TUBE}.shape_type")), ":shell");
    let v = number(&format!("{OPEN_TUBE}.volume"));
    assert_eq!(v, 0.0, "an open surface encloses nothing, got {v}");
}

#[test]
fn an_open_shell_weighs_nothing() {
    // The consequence that made it worth fixing rather than documenting: the
    // fictional volume became a fictional mass with no indication anywhere.
    let mass = number(&format!("mass_estimate({OPEN_TUBE})"));
    assert_eq!(mass, 0.0, "a surface has no material to weigh, got {mass}");
}

#[test]
fn thickening_the_surface_gives_it_a_real_volume() {
    // The guard must point somewhere. `thicken` is the documented answer, and
    // the result has to measure normally rather than stay stuck at zero.
    let v = number(&format!("{OPEN_TUBE}.thicken(1).volume"));
    assert!(
        v > 0.0,
        "a thickened surface is a solid and must have volume, got {v}"
    );
}

#[test]
fn a_sphere_still_reports_its_volume() {
    // The regression a "must be closed?" rule would have caused. A sphere is a
    // perfectly good solid that OCCT reports as not closed, because its seam
    // edge belongs to one face.
    assert_eq!(eval("sphere(5).closed?"), "false");
    let v = number("sphere(5).volume");
    let expected = 4.0 / 3.0 * std::f64::consts::PI * 125.0;
    assert!(
        (v - expected).abs() < 1e-9,
        "expected {expected}, got {v} — the closure test must not catch solids"
    );
}

#[test]
fn an_imported_mesh_still_reports_its_volume() {
    // The regression a "must contain a Solid" rule would have caused. An STL
    // comes back as a Compound of triangle faces with no Solid anywhere in it,
    // and its volume is exactly the box that produced it.
    let dir = std::env::current_dir()
        .expect("cwd")
        .join("target/volume_surfaces_mesh");
    std::fs::create_dir_all(&dir).expect("create workspace");
    let path = dir.join("box.stl");
    let literal = format!("{:?}", path.to_string_lossy());

    let mut vm = MrubyVm::new();
    let kind = vm
        .eval(&format!(
            "box(10, 20, 30).export({literal})\n\
             import_stl({literal}).shape_type"
        ))
        .expect("export and re-import");
    let volume = vm
        .eval(&format!("import_stl({literal}).volume"))
        .expect("measure the imported mesh");
    std::fs::remove_dir_all(&dir).ok();

    assert_eq!(
        kind.trim(),
        ":compound",
        "an imported STL is a compound of faces"
    );
    let volume: f64 = volume.trim().parse().expect("a number");
    assert!(
        (volume - 6000.0).abs() < 1e-6,
        "expected the source box volume back, got {volume}"
    );
}

#[test]
fn a_sewn_surface_measures_normally() {
    // Closing a surface is the other documented route out, and `sew` produces a
    // Solid, so the guard must not touch it.
    let v = number("sew(box(10, 10, 10).faces(:all)).volume");
    assert!(
        (v - 1000.0).abs() < 1e-9,
        "a sewn box should measure 1000, got {v}"
    );
}

#[test]
fn a_boolean_result_still_reports_its_volume() {
    // Another shape OCCT calls not closed while measuring it correctly.
    assert_eq!(eval("box(10,10,10).cut(sphere(4)).closed?"), "false");
    let v = number("box(10,10,10).cut(sphere(4)).volume");
    // The box spans 0..10 on every axis and the sphere is centred on the
    // origin, so only the one octant of it that reaches into the box is
    // removed.
    let expected = 1000.0 - (4.0 / 3.0 * std::f64::consts::PI * 64.0) / 8.0;
    assert!((v - expected).abs() < 1e-6, "expected {expected}, got {v}");
}

#[test]
fn a_flat_face_still_reports_zero() {
    // Unchanged behaviour, pinned so the guard's scope stays visible: a Face is
    // deliberately left to OCCT, because a single spherical face is a closed
    // surface with a genuine volume.
    assert_eq!(number("rect(20, 10).volume"), 0.0);
}
