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

// ---------------------------------------------------------------------------
// Design table integration: run a script multiple times with different params
// ---------------------------------------------------------------------------

#[test]
fn design_table_batch_export() {
    use std::fs;

    let dir = std::env::temp_dir().join("rrcad_design_table_test");
    fs::create_dir_all(&dir).unwrap();

    // Script: export a box whose dimensions come from params.
    // Note: "#{" must not appear in a Rust raw string literal, so we build
    // the Ruby interpolation via string concatenation.
    let script_path = dir.join("box.rb");
    let ruby_script = concat!(
        "name = param :name, default: \"out\"\n",
        "w    = param :w,    default: 10\n",
        "h    = param :h,    default: 10\n",
        "d    = param :d,    default: 10\n",
        "box(w, h, d).export(\"#{name}.step\")\n",
    );
    fs::write(&script_path, ruby_script).unwrap();

    // CSV design table with three variants.
    let csv_path = dir.join("sizes.csv");
    fs::write(
        &csv_path,
        "name,w,h,d\nsmall,10,10,10\nmedium,20,20,20\nlarge,40,40,40\n",
    )
    .unwrap();

    // Change working directory to the temp dir so .step files land there.
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let result = {
        // Re-implement the core loop inline (run_design_table is not pub).
        let code = fs::read_to_string(&script_path).unwrap();
        let rows = vec![
            vec![
                "small".to_string(),
                "10".to_string(),
                "10".to_string(),
                "10".to_string(),
            ],
            vec![
                "medium".to_string(),
                "20".to_string(),
                "20".to_string(),
                "20".to_string(),
            ],
            vec![
                "large".to_string(),
                "40".to_string(),
                "40".to_string(),
                "40".to_string(),
            ],
        ];
        let headers = vec!["name", "w", "h", "d"];
        let mut ok = 0usize;
        for row in &rows {
            let params: Vec<(String, String)> = headers
                .iter()
                .zip(row.iter())
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();
            let mut vm = MrubyVm::new();
            if vm.set_params(&params).and_then(|_| vm.eval(&code)).is_ok() {
                ok += 1;
            }
        }
        ok
    };

    std::env::set_current_dir(&orig).unwrap();

    assert_eq!(result, 3, "all three rows should succeed");

    // Each row's script exports a STEP file named after the `name` param.
    for stem in &["small", "medium", "large"] {
        let p = dir.join(format!("{stem}.step"));
        assert!(p.exists(), "{stem}.step was not created");
        assert!(p.metadata().unwrap().len() > 0, "{stem}.step is empty");
    }
}
