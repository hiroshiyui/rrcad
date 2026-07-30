/// Phase 12 — text glyph profiles.
///
/// `text(str, size:, font: nil)` renders a string's glyph outlines as a
/// Compound of planar Faces in the XY plane (Font_BRepFont +
/// Font_BRepTextBuilder), baseline at the origin. Emboss = extrude + fuse,
/// engrave = extrude + cut — the labels, version numbers, and CW/CCW
/// motor-rotation arrows of printed frame plates.
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

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

#[test]
fn text_is_a_compound_of_faces() {
    assert_eq!(eval(r#"text("V2", size: 6).shape_type"#), ":compound");
    // Faces, not wires: the glyphs must extrude directly.
    assert_eq!(
        eval(r#"text("V2", size: 6).faces(:all).length >= 2"#),
        "true"
    );
}

#[test]
fn size_sets_the_glyph_height() {
    // Capital-letter height is a large fraction of the em size; fonts vary,
    // so accept a band rather than an exact ratio.
    let dy = eval_num(r#"text("X", size: 10).bounding_box[:dy]"#);
    assert!(
        dy > 4.0 && dy < 12.0,
        "size 10 capital should be roughly that tall, got {dy}"
    );
    // Doubling the size doubles the glyph.
    let dy2 = eval_num(r#"text("X", size: 20).bounding_box[:dy]"#);
    assert!(
        (dy2 / dy - 2.0).abs() < 0.05,
        "scaling should be linear: {dy} → {dy2}"
    );
}

#[test]
fn longer_strings_grow_along_x() {
    let short = eval_num(r#"text("A", size: 6).bounding_box[:dx]"#);
    let long = eval_num(r#"text("AAAA", size: 6).bounding_box[:dx]"#);
    assert!(
        long > short * 3.0,
        "four letters should be much wider than one: {short} vs {long}"
    );
}

#[test]
fn glyphs_sit_on_the_baseline_at_the_origin() {
    // A capital X starts at the baseline (y=0) and near x=0 (side bearing).
    let code = r#"bb = text("X", size: 10).bounding_box
                  [bb[:x], bb[:y]]"#;
    let out = eval(code);
    let v: Vec<f64> = out
        .trim_matches(['[', ']'])
        .split(',')
        .map(|t| t.trim().parse().unwrap())
        .collect();
    assert!(
        v[0].abs() < 2.0 && v[1].abs() < 1.0,
        "baseline origin expected, got {out}"
    );
}

// ---------------------------------------------------------------------------
// Emboss and engrave
// ---------------------------------------------------------------------------

#[test]
fn embossed_label_adds_material() {
    let code = r#"
      plate = box(40, 12, 3)
      label = text("V2", size: 6).extrude(0.6).translate(4, 3, 3)
      (plate.fuse(label).volume - plate.volume) > 0.5
    "#;
    assert_eq!(eval(code), "true");
}

#[test]
fn engraved_label_removes_material() {
    let code = r#"
      plate = box(40, 12, 3)
      tool  = text("V2", size: 6).extrude(0.6).translate(4, 3, 2.4)
      (plate.volume - plate.cut(tool).volume) > 0.5
    "#;
    assert_eq!(eval(code), "true");
}

#[test]
fn engraving_exports_cleanly() {
    // The full frame-plate path: engrave, then mesh for printing.
    let code = r#"
      plate = box(40, 12, 3)
      tool  = text("V2", size: 6).extrude(0.6).translate(4, 3, 2.4)
      part  = plate.cut(tool)
      part.export("rrcad_text_test.stl")
      part.validate
    "#;
    assert_eq!(eval(code), ":ok");
    std::fs::remove_file("rrcad_text_test.stl").ok();
}

// ---------------------------------------------------------------------------
// Feature history and rebuild
// ---------------------------------------------------------------------------

#[test]
fn text_records_and_rebuilds() {
    let hist = eval(r#"text("V2", size: 6).history.last"#);
    assert!(
        hist.contains("text(\\\"V2\\\"") || hist.contains("text("),
        "unexpected history entry: {hist}"
    );
    let diff = eval_num(
        r#"t = text("V2", size: 6)
           (t.rebuild.extrude(1).volume - t.extrude(1).volume).abs"#,
    );
    assert!(diff < 1e-9, "rebuild drifted by {diff}");
}

// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

#[test]
fn a_named_font_family_resolves() {
    // DejaVu Sans is installed in CI (fonts-dejavu-core) and nearly
    // everywhere else; the font manager may also alias it. Either way this
    // must render, not raise.
    assert_eq!(
        eval(r#"text("V2", size: 6, font: "DejaVu Sans").shape_type"#),
        ":compound"
    );
}

#[test]
fn a_missing_font_file_is_an_error() {
    let err = eval_err(r#"text("V2", size: 6, font: "/nonexistent/font.ttf")"#);
    assert!(
        err.contains("could not load font file"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn empty_string_is_rejected() {
    let err = eval_err(r#"text("", size: 6)"#);
    assert!(err.contains("string is empty"), "unexpected error: {err}");
}

#[test]
fn whitespace_only_string_is_rejected() {
    let err = eval_err(r#"text("   ", size: 6)"#);
    assert!(err.contains("no glyph faces"), "unexpected error: {err}");
}

#[test]
fn size_must_be_positive() {
    let err = eval_err(r#"text("V2", size: 0)"#);
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

#[test]
fn font_must_be_a_name_or_path() {
    let err = eval_err(r#"text("V2", size: 6, font: 42)"#);
    assert!(
        err.contains("family name or a .ttf/.otf path"),
        "unexpected error: {err}"
    );
}
