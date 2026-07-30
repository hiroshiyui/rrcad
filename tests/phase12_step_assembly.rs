/// Phase 12 — structured (non-fused) assembly STEP export.
///
/// `asm.export("drone.step", structured: true)` writes each component as a
/// named PRODUCT under one root assembly via STEPCAFControl_Writer + XCAF,
/// so FreeCAD/Fusion — or whoever machines the plates — sees the parts
/// individually. The default export still fuses, unchanged.
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

/// Export with `script`, read the produced STEP text, delete the file.
fn step_text(script: &str, file: &str) -> String {
    eval(script);
    let text = std::fs::read_to_string(file)
        .unwrap_or_else(|e| panic!("could not read exported {file}: {e}"));
    std::fs::remove_file(file).ok();
    text
}

// ---------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------

#[test]
fn structured_export_writes_an_assembly_tree() {
    let text = step_text(
        r#"
          a = assembly("drone")
          a.place box(30, 30, 3), name: :top_plate
          a.place cylinder(14, 20).translate(8, 8, 3), name: :motor_fl
          a.export("t_structured.step", structured: true)
        "#,
        "t_structured.step",
    );
    // One occurrence per component under the root assembly.
    let nauo = text.matches("NEXT_ASSEMBLY_USAGE_OCCURRENCE").count();
    assert_eq!(nauo, 2, "expected 2 component occurrences, got {nauo}");
    for product in ["'drone'", "'top_plate'", "'motor_fl'"] {
        assert!(
            text.contains(&format!("PRODUCT({product}")),
            "missing product {product}"
        );
    }
}

#[test]
fn unnamed_parts_get_part_n_names() {
    let text = step_text(
        r#"
          a = assembly("rig")
          a.place box(10, 10, 10)
          a.place sphere(4).translate(20, 0, 0)
          a.export("t_autoname.step", structured: true)
        "#,
        "t_autoname.step",
    );
    assert!(text.contains("PRODUCT('part_1'"), "missing part_1");
    assert!(text.contains("PRODUCT('part_2'"), "missing part_2");
}

#[test]
fn part_color_travels_into_the_file() {
    let text = step_text(
        r#"
          a = assembly("colored")
          a.place box(10, 10, 10).color(0.8, 0.2, 0.2), name: :red_part
          a.place box(10, 10, 10).translate(20, 0, 0), name: :plain_part
          a.export("t_color.step", structured: true)
        "#,
        "t_color.step",
    );
    assert!(
        text.contains("COLOUR_RGB") && text.contains("STYLED_ITEM"),
        "expected colour records in the STEP file"
    );
}

// ---------------------------------------------------------------------------
// Round trip
// ---------------------------------------------------------------------------

#[test]
fn structured_file_reimports_with_all_solids() {
    let code = r#"
      a = assembly("rt")
      a.place box(20, 20, 4), name: :plate
      a.place cylinder(5, 30).translate(40, 0, 0), name: :standoff
      a.export("t_roundtrip.step", structured: true)
      back = import_step("t_roundtrip.step")
      expected = 20 * 20 * 4 + Math::PI * 25 * 30
      (back.volume - expected).abs / expected
    "#;
    let rel_err = eval_num(code);
    std::fs::remove_file("t_roundtrip.step").ok();
    assert!(
        rel_err < 1e-3,
        "reimported volume off by {:.4}%",
        rel_err * 100.0
    );
}

// ---------------------------------------------------------------------------
// Default unchanged
// ---------------------------------------------------------------------------

#[test]
fn default_export_still_fuses() {
    let text = step_text(
        r#"
          a = assembly("fused")
          a.place box(10, 10, 10), name: :a
          a.place box(10, 10, 10).translate(5, 0, 0), name: :b
          a.export("t_fused.step")
        "#,
        "t_fused.step",
    );
    assert!(
        !text.contains("NEXT_ASSEMBLY_USAGE_OCCURRENCE"),
        "default export must stay a single fused solid"
    );
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn structured_is_step_only() {
    let err = eval_err(
        r#"
          a = assembly("x")
          a.place box(5, 5, 5)
          a.export("t_bad.stl", structured: true)
        "#,
    );
    assert!(err.contains("STEP-only"), "unexpected error: {err}");
}

#[test]
fn empty_assembly_is_rejected() {
    let err = eval_err(r#"assembly("empty").export("t_empty.step", structured: true)"#);
    assert!(
        err.contains("contains no shapes"),
        "unexpected error: {err}"
    );
}

#[test]
fn structured_export_respects_path_confinement() {
    let err = eval_err(
        r#"
          a = assembly("x")
          a.place box(5, 5, 5)
          a.export("../t_escape.step", structured: true)
        "#,
    );
    assert!(
        err.contains("path traversal rejected"),
        "unexpected error: {err}"
    );
}
