/// Tests for Phase 3 spline/sweep primitives.
use rrcad::ruby::vm::MrubyVm;

fn eval(code: &str) -> Result<String, String> {
    let mut vm = MrubyVm::new();
    vm.eval(code)
}

// ---------------------------------------------------------------------------
// spline_2d
// ---------------------------------------------------------------------------

#[test]
fn spline_2d_returns_shape() {
    let r = eval("spline_2d([[0.0,0.0],[2.0,1.0],[3.0,3.0],[0.0,4.0]]).class").unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn spline_2d_revolve_full_returns_shape() {
    let r =
        eval("spline_2d([[0.0,0.0],[2.0,1.0],[3.0,3.0],[0.0,4.0]]).revolve(360).class").unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn spline_2d_revolve_exports_step() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_spline2d_revolve.step");
    let code = format!(
        r#"
        profile = spline_2d([[0.0,0.0],[2.0,1.0],[3.0,3.0],[0.0,4.0]])
        profile.revolve(360).export("{}")
        "#,
        out.display()
    );
    vm.eval(&code).expect("eval failed");
    assert!(out.exists(), "STEP file not created");
    assert!(
        std::fs::metadata(&out).unwrap().len() > 0,
        "STEP file empty"
    );
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("ISO-10303-21"), "not a valid STEP file");
}

// ---------------------------------------------------------------------------
// spline tangent constraints (Tier 4)
// ---------------------------------------------------------------------------

#[test]
fn spline_2d_with_tangents_returns_shape() {
    // Explicit start/end tangents; both pointing roughly in +X direction.
    let r = eval("spline_2d([[0,0],[2,1],[3,3],[0,4]], tangents: [[1,0],[1,0]]).class").unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn spline_2d_tangents_can_revolve() {
    // Constrained spline + revolve should produce a valid solid.
    let r =
        eval("spline_2d([[0,0],[2,1],[3,3],[0,4]], tangents: [[1,0],[0,1]]).revolve(360).class")
            .unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn spline_3d_with_tangents_returns_shape() {
    // Explicit tangents for a 3D sweep path.
    let r =
        eval("spline_3d([[0,0,0],[5,0,5],[10,0,0]], tangents: [[1,0,1],[1,0,-1]]).class").unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn spline_3d_tangents_can_sweep() {
    // A circle swept along a tangent-constrained 3D path must produce a Shape.
    let r = eval(
        r#"
        path = spline_3d([[0,0,0],[5,0,3],[10,0,0]], tangents: [[1,0,1],[1,0,-1]])
        circle(0.5).sweep(path).class
        "#,
    )
    .unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn spline_2d_bad_tangents_raises() {
    // Wrong number of tangent vectors should raise ArgumentError.
    let mut vm = rrcad::ruby::vm::MrubyVm::new();
    let result = vm.eval("spline_2d([[0,0],[2,1],[3,3]], tangents: [[1,0]])");
    assert!(
        result.is_err(),
        "expected ArgumentError for one tangent vector"
    );
}

// ---------------------------------------------------------------------------
// spline_3d
// ---------------------------------------------------------------------------

#[test]
fn spline_3d_returns_shape() {
    let r = eval("spline_3d([[0.0,0.0,0.0],[1.0,2.0,3.0],[4.0,0.0,6.0]]).class").unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

// ---------------------------------------------------------------------------
// sweep
// ---------------------------------------------------------------------------

#[test]
fn circle_sweep_returns_shape() {
    let r = eval(
        r#"
        path = spline_3d([[4.0,0.0,2.5],[5.5,0.0,3.5],[7.5,0.0,5.5]])
        circle(0.7).sweep(path).class
        "#,
    )
    .unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn circle_sweep_exports_step() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_sweep.step");
    let code = format!(
        r#"
        path = spline_3d([[4.0,0.0,2.5],[5.5,0.0,3.5],[7.5,0.0,5.5]])
        circle(0.7).sweep(path).export("{}")
        "#,
        out.display()
    );
    vm.eval(&code).expect("eval failed");
    assert!(out.exists(), "STEP file not created");
    assert!(
        std::fs::metadata(&out).unwrap().len() > 0,
        "STEP file empty"
    );
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("ISO-10303-21"), "not a valid STEP file");
}

// ---------------------------------------------------------------------------
// sweep_sections — variable-section pipe sweep
// ---------------------------------------------------------------------------

#[test]
fn sweep_sections_variable_radius_returns_shape() {
    let r = eval(
        r#"
        path = spline_3d([[-4.5,0,1.5], [-8.54,0,4.8], [-4.0,0,6.3]])
        sweep_sections(path, [circle(1.4), circle(0.7), circle(1.4)]).class
        "#,
    )
    .unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn sweep_sections_exports_valid_step() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_sweep_sections.step");
    let code = format!(
        r#"
        path = spline_3d([[0,0,0],[5,0,5],[10,0,0]])
        sweep_sections(path, [circle(2.0), circle(0.5), circle(2.0)]).export("{}")
        "#,
        out.display()
    );
    vm.eval(&code).expect("sweep_sections failed");
    assert!(out.exists(), "STEP file not created");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("ISO-10303-21"), "not a valid STEP file");
}

#[test]
fn sweep_sections_requires_at_least_two_profiles() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("sweep_sections(spline_3d([[0,0,0],[5,0,5]]), [circle(1.0)])");
    assert!(result.is_err(), "expected error with one profile");
}
