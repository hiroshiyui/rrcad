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
