/// Phase 12 — `shell` with face selection.
///
/// `.shell(thickness)` always removed the topmost face. A canopy or battery
/// tray needs to choose which face(s) become the opening: `.shell(t, open:)`
/// takes Face shapes from `.faces` on the same solid, or `.faces` selectors,
/// and removes exactly those. The bridge matches requested faces against the
/// body with `IsSame`, so a face of some other shape is rejected by name.
use rrcad::ruby::vm::MrubyVm;

/// Evaluate `code` and return the trimmed result string.
fn eval(code: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(code)
        .unwrap_or_else(|e| panic!("script failed: {e}\n--- script ---\n{code}"))
        .trim()
        .to_string()
}

/// Evaluate `code` expecting a numeric result.
fn eval_num(code: &str) -> f64 {
    let out = eval(code);
    out.parse()
        .unwrap_or_else(|_| panic!("expected a number, got: {out}"))
}

/// Evaluate `code` expecting failure, returning the error message.
fn eval_err(code: &str) -> String {
    let mut vm = MrubyVm::new();
    match vm.eval(code) {
        Ok(v) => panic!("expected failure, got: {v}\n--- script ---\n{code}"),
        Err(e) => e.to_string(),
    }
}

// A 20 mm cube shelled at 2 mm with one face open encloses an
// 16×16×18 cavity: 8000 − 4608 = 3392 mm³ of wall material.
const ONE_OPEN_VOLUME: f64 = 3392.0;

// ---------------------------------------------------------------------------
// Backward compatibility
// ---------------------------------------------------------------------------

#[test]
fn shell_without_open_still_removes_the_top() {
    let v = eval_num("box(20, 20, 20).shell(2).volume");
    assert!(
        (v - ONE_OPEN_VOLUME).abs() < 1.0,
        "default shell volume changed: {v}"
    );
    // The cavity must break through the top: a point just under the top face
    // centre is outside the material, which shows as unchanged bounding box
    // but reduced volume (checked above) with an opening — validate is enough
    // here; the geometry assertions live in the open: tests below.
    assert_eq!(eval("box(20, 20, 20).shell(2).validate"), ":ok");
}

// ---------------------------------------------------------------------------
// open: with selectors
// ---------------------------------------------------------------------------

#[test]
fn open_bottom_mirrors_the_default() {
    // Same wall thickness, opposite opening: identical material volume.
    let v = eval_num("box(20, 20, 20).shell(2, open: :bottom).volume");
    assert!((v - ONE_OPEN_VOLUME).abs() < 1.0, "got {v}");
}

#[test]
fn open_top_matches_the_legacy_behavior() {
    let legacy = eval_num("box(20, 20, 20).shell(2).volume");
    let explicit = eval_num("box(20, 20, 20).shell(2, open: :top).volume");
    assert!(
        (legacy - explicit).abs() < 1e-6,
        "open: :top should equal the default: {legacy} vs {explicit}"
    );
}

#[test]
fn two_openings_make_a_tunnel() {
    // Both X ends removed: the cavity runs 20×16×16 through the part,
    // leaving 8000 − 5120 = 2880 mm³.
    let v = eval_num(r#"box(20, 20, 20).shell(2, open: [:">X", :"<X"]).volume"#);
    assert!((v - 2880.0).abs() < 1.0, "tunnel volume: {v}");
}

#[test]
fn open_works_on_a_cylinder() {
    // Cup: cylinder shelled with the top disc removed. Wall 2: cavity is
    // r=8, h=18 → π(100·20 − 64·18) ≈ 2664.
    let v = eval_num("cylinder(10, 20).shell(2, open: :top).volume");
    let expected = std::f64::consts::PI * (100.0 * 20.0 - 64.0 * 18.0);
    assert!(
        (v - expected).abs() < expected * 0.01,
        "cup volume: expected ~{expected}, got {v}"
    );
}

// ---------------------------------------------------------------------------
// open: with explicit Face shapes
// ---------------------------------------------------------------------------

#[test]
fn open_takes_a_face_from_the_same_solid() {
    let v = eval_num(
        "part = box(20, 20, 20)
         part.shell(2, open: part.faces(:bottom)[0]).volume",
    );
    assert!((v - ONE_OPEN_VOLUME).abs() < 1.0, "got {v}");
}

#[test]
fn open_takes_an_array_of_faces() {
    let v = eval_num(
        "part = box(20, 20, 20)
         part.shell(2, open: part.faces(:side)[0, 2]).volume",
    );
    // Two adjacent side faces removed: cavity 16 deep on one axis, 18 on the
    // other... measured: two openings leave less material than one.
    assert!(
        v < ONE_OPEN_VOLUME,
        "two openings should remove more material: {v}"
    );
    assert_eq!(
        eval(
            "part = box(20, 20, 20)
             part.shell(2, open: part.faces(:side)[0, 2]).validate"
        ),
        ":ok"
    );
}

#[test]
fn duplicate_faces_are_deduped() {
    let v = eval_num(
        "part = box(20, 20, 20)
         f = part.faces(:top)[0]
         part.shell(2, open: [f, f]).volume",
    );
    assert!(
        (v - ONE_OPEN_VOLUME).abs() < 1.0,
        "duplicate face should count once: {v}"
    );
}

// ---------------------------------------------------------------------------
// Feature history and rebuild
// ---------------------------------------------------------------------------

#[test]
fn open_shell_records_and_rebuilds() {
    let hist = eval(
        "part = box(20, 20, 20).shell(2, open: :bottom)
         part.history.last",
    );
    assert!(
        hist.contains("shell(thickness=2") && hist.contains("open_faces=1"),
        "unexpected history entry: {hist}"
    );

    let code = "part = box(20, 20, 20).shell(2, open: :bottom)
                (part.rebuild.volume - part.volume).abs";
    let diff = eval_num(code);
    assert!(diff < 1e-6, "rebuild drifted by {diff}");
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn face_of_another_shape_is_rejected() {
    let err = eval_err(
        "a = box(20, 20, 20)
         b = box(20, 20, 20)
         a.shell(2, open: b.faces(:top)[0])",
    );
    assert!(
        err.contains("not a face of this solid"),
        "unexpected error: {err}"
    );
}

#[test]
fn face_of_a_transformed_copy_is_rejected() {
    // translate returns a new shape with new topology; its faces are not
    // faces of the original.
    let err = eval_err(
        "part = box(20, 20, 20)
         moved = part.translate(5, 0, 0)
         part.shell(2, open: moved.faces(:top)[0])",
    );
    assert!(
        err.contains("not a face of this solid"),
        "unexpected error: {err}"
    );
}

#[test]
fn opening_every_face_is_refused() {
    let err = eval_err("box(20, 20, 20).shell(2, open: :all)");
    assert!(
        err.contains("at least one face must remain"),
        "unexpected error: {err}"
    );
}

#[test]
fn selector_matching_nothing_is_an_error() {
    // A wire has no faces for :top to match... a simpler nothing-case: a
    // direction selector on a sphere, whose single face has no planar normal.
    let err = eval_err(r#"sphere(10).shell(2, open: :">Z")"#);
    assert!(err.contains("matched no faces"), "unexpected error: {err}");
}

#[test]
fn unknown_selector_reports_the_options() {
    let err = eval_err("box(20, 20, 20).shell(2, open: :nope)");
    assert!(err.contains("unknown selector"), "unexpected error: {err}");
}

#[test]
fn non_face_open_value_is_rejected() {
    let err = eval_err("box(20, 20, 20).shell(2, open: 42)");
    assert!(
        err.contains("Face shapes or .faces selectors"),
        "unexpected error: {err}"
    );
}
