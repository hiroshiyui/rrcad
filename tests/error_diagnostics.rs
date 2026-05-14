// Phase 10 — broader OCCT diagnostics.
//
// Verifies that errors raised by Boolean operations, fillets/chamfers, and
// import/export carry operation context (op name, parameters, operand
// shape kinds, file paths) so users can diagnose failures without
// guessing which call site raised.

use rrcad::occt::Shape;

fn err_of<T, E>(r: Result<T, E>) -> E {
    match r {
        Ok(_) => panic!("expected error, got Ok"),
        Err(e) => e,
    }
}

#[test]
fn fillet_error_includes_operation_and_radius() {
    // 1×1×1 cube with a 10 mm fillet radius is geometrically impossible.
    let cube = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(cube.fillet(10.0));
    assert!(
        err.contains("fillet(r=10"),
        "expected fillet(r=10) prefix, got: {err}"
    );
    assert!(
        err.contains("solid"),
        "expected operand kind 'solid' in message, got: {err}"
    );
}

#[test]
fn chamfer_error_includes_operation_and_distance() {
    let cube = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(cube.chamfer(10.0));
    assert!(
        err.contains("chamfer(d=10"),
        "expected chamfer(d=10) prefix, got: {err}"
    );
}

#[test]
fn fillet_sel_error_includes_selector() {
    let cube = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(cube.fillet_sel(10.0, "vertical"));
    assert!(
        err.contains("edges:") && err.contains("vertical"),
        "expected edge selector in message, got: {err}"
    );
}

#[test]
fn import_step_error_includes_path() {
    let err = err_of(Shape::import_step("/tmp/rrcad_does_not_exist_zzz.step"));
    assert!(
        err.contains("import_step("),
        "expected import_step(...) prefix, got: {err}"
    );
    assert!(
        err.contains("zzz.step"),
        "expected the path in the error, got: {err}"
    );
}

#[test]
fn import_stl_error_includes_path() {
    let err = err_of(Shape::import_stl("/tmp/rrcad_does_not_exist_zzz.stl"));
    assert!(
        err.contains("import_stl("),
        "expected import_stl(...) prefix, got: {err}"
    );
}

#[test]
fn export_step_error_includes_path() {
    let cube = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(cube.export_step("/nonexistent_dir/rrcad_zzz.step"));
    assert!(
        err.contains("export_step("),
        "expected export_step(...) prefix, got: {err}"
    );
    assert!(
        err.contains("/nonexistent_dir/"),
        "expected the path in the error, got: {err}"
    );
}

#[test]
fn export_svg_error_includes_view() {
    let cube = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(cube.export_svg("/nonexistent_dir/rrcad_zzz.svg", "isometric"));
    assert!(
        err.contains("export_svg(") && err.contains("isometric"),
        "expected export_svg + view in error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Sweeps, lofts, Part Design, and other 3-D operations
// ---------------------------------------------------------------------------

#[test]
fn extrude_error_includes_height() {
    // A solid can't be extruded — only Faces / Wires.
    let cube = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(cube.extrude(5.0));
    assert!(
        err.contains("extrude(h=5"),
        "expected extrude(h=5) prefix, got: {err}"
    );
}

#[test]
fn loft_error_includes_profile_count() {
    // Lofting through a single profile is invalid (needs ≥ 2).
    let only = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(Shape::loft(&[&only], true));
    assert!(
        err.contains("loft(profiles=1"),
        "expected loft(profiles=1, …) prefix, got: {err}"
    );
}

#[test]
fn sweep_sections_error_for_too_few_profiles() {
    let path = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let only = Shape::make_box(1.0, 1.0, 1.0).expect("make_box");
    let err = err_of(Shape::sweep_sections(&[&only], &path));
    assert!(
        err.contains("sweep_sections"),
        "expected sweep_sections error, got: {err}"
    );
}
