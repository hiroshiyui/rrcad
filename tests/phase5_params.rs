// tests/phase5_params.rs — Phase 5 parametric DSL integration tests
//
// Tests cover: param() default values, CLI overrides via set_params(),
// type coercion (string → Integer / Float), and range validation.
use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Default values (no CLI override)
// ---------------------------------------------------------------------------

#[test]
fn param_integer_default() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("param :width, default: 10").unwrap(), "10");
}

#[test]
fn param_float_default() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("param :scale, default: 1.0").unwrap(), "1.0");
}

#[test]
fn param_string_default() {
    let mut vm = MrubyVm::new();
    assert_eq!(
        vm.eval("param :label, default: \"part\"").unwrap(),
        "\"part\""
    );
}

// ---------------------------------------------------------------------------
// CLI overrides via set_params
// ---------------------------------------------------------------------------

#[test]
fn param_override_integer() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("width".into(), "42".into())]).unwrap();
    assert_eq!(vm.eval("param :width, default: 10").unwrap(), "42");
}

#[test]
fn param_override_float() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("scale".into(), "2.5".into())]).unwrap();
    assert_eq!(vm.eval("param :scale, default: 1.0").unwrap(), "2.5");
}

#[test]
fn param_override_string() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("label".into(), "bracket".into())])
        .unwrap();
    assert_eq!(
        vm.eval("param :label, default: \"part\"").unwrap(),
        "\"bracket\""
    );
}

#[test]
fn param_no_override_returns_default() {
    let mut vm = MrubyVm::new();
    // set_params with an unrelated key — the declared param falls back to default.
    vm.set_params(&[("other".into(), "99".into())]).unwrap();
    assert_eq!(vm.eval("param :width, default: 10").unwrap(), "10");
}

#[test]
fn param_multiple_overrides() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("w".into(), "20".into()), ("h".into(), "30".into())])
        .unwrap();
    let result = vm
        .eval("w = param :w, default: 10; h = param :h, default: 10; w + h")
        .unwrap();
    assert_eq!(result, "50");
}

// ---------------------------------------------------------------------------
// Range validation
// ---------------------------------------------------------------------------

#[test]
fn param_within_range_ok() {
    let mut vm = MrubyVm::new();
    assert_eq!(
        vm.eval("param :width, default: 50, range: 1..100").unwrap(),
        "50"
    );
}

#[test]
fn param_default_outside_range_raises() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("param :width, default: 200, range: 1..100")
        .unwrap_err();
    assert!(err.contains("outside range"), "unexpected error: {err}");
}

#[test]
fn param_override_outside_range_raises() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("width".into(), "999".into())]).unwrap();
    let err = vm
        .eval("param :width, default: 10, range: 1..100")
        .unwrap_err();
    assert!(err.contains("outside range"), "unexpected error: {err}");
}

#[test]
fn param_override_within_range_ok() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("width".into(), "75".into())]).unwrap();
    assert_eq!(
        vm.eval("param :width, default: 10, range: 1..100").unwrap(),
        "75"
    );
}

// ---------------------------------------------------------------------------
// Usable in geometry expressions
// ---------------------------------------------------------------------------

#[test]
fn param_drives_box_dimensions() {
    let mut vm = MrubyVm::new();
    vm.set_params(&[("size".into(), "5".into())]).unwrap();
    // box(size, size, size) should produce a valid shape without error.
    vm.eval("s = param :size, default: 10; b = box(s, s, s); b.class")
        .unwrap();
}
