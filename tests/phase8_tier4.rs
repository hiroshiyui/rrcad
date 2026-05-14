// Phase 8 Tier 4 — 2-D drawing output
//
// Tests for:
//   shape.export("out.svg")              → SVG via HLRBRep_PolyAlgo
//   shape.export("out.svg", view: :front|:side)
//   shape.export("out.dxf")              → DXF R12 via HLRBRep_PolyAlgo

use rrcad::ruby::vm::MrubyVm;

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from("target/e2e_test_outputs");
    std::fs::create_dir_all(&dir).expect("could not create e2e output directory");
    dir.join(name)
}

fn svg_width(path: &std::path::Path) -> f64 {
    let content = std::fs::read_to_string(path).unwrap();
    let start = content.find(" width=\"").expect("SVG width attribute") + " width=\"".len();
    let end = content[start..].find('"').expect("SVG width terminator") + start;
    content[start..end].parse().expect("numeric SVG width")
}

fn dxf_max_abs_xy(path: &std::path::Path) -> f64 {
    let content = std::fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let mut max: f64 = 0.0;
    for pair in lines.windows(2) {
        if matches!(pair[0].trim(), "10" | "11" | "20" | "21") {
            let value: f64 = pair[1].trim().parse().expect("numeric DXF coordinate");
            max = max.max(value.abs());
        }
    }
    max
}

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------

#[test]
fn svg_top_view_creates_file() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_top.svg");
    let code = format!("box(20,10,5).export('{}')", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "SVG file was not created");
    assert!(out.metadata().unwrap().len() > 0, "SVG file is empty");
}

#[test]
fn svg_contains_xml_header() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_xml.svg");
    let code = format!("box(10,10,10).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("<?xml") && content.contains("<svg"),
        "SVG must begin with XML declaration and <svg> element"
    );
}

#[test]
fn svg_contains_polyline_elements() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_poly.svg");
    let code = format!("box(10,10,10).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("<polyline"),
        "SVG must contain <polyline> elements"
    );
}

#[test]
fn svg_front_view() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_front.svg");
    let code = format!("box(10,10,10).export('{}', view: :front)", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "SVG front view file was not created");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<svg"), "front view SVG must be valid");
}

#[test]
fn svg_side_view() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_side.svg");
    let code = format!("box(10,10,10).export('{}', view: :side)", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "SVG side view file was not created");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<svg"), "side view SVG must be valid");
}

#[test]
fn svg_cylinder_top_view() {
    // Curved surfaces (circles) must be discretised into smooth polylines.
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_cyl.svg");
    let code = format!("cylinder(5, 20).export('{}')", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "cylinder SVG was not created");
}

#[test]
fn svg_scale_expands_output_geometry() {
    let mut vm = MrubyVm::new();
    let normal = tmp("rrcad_test_scale_1.svg");
    let scaled = tmp("rrcad_test_scale_2.svg");
    vm.eval(&format!(
        "box(10,10,10).export('{}', scale: 1.0)",
        normal.display()
    ))
    .unwrap();
    vm.eval(&format!(
        "box(10,10,10).export('{}', scale: 2.0)",
        scaled.display()
    ))
    .unwrap();

    assert!(
        svg_width(&scaled) > svg_width(&normal),
        "scale: 2.0 should increase SVG drawing width"
    );
}

#[test]
fn svg_rejects_non_positive_scale() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_bad_scale.svg");
    let err = vm
        .eval(&format!(
            "box(10,10,10).export('{}', scale: 0)",
            out.display()
        ))
        .expect_err("scale: 0 should fail");
    assert!(
        err.contains("scale") && err.contains("positive"),
        "expected actionable scale error, got: {err}"
    );
}

#[test]
fn svg_hidden_option_adds_dashed_hidden_layer() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_hidden.svg");
    vm.eval(&format!(
        "box(10,10,10).export('{}', hidden: true)",
        out.display()
    ))
    .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("class=\"hidden\"") && content.contains("stroke-dasharray"),
        "hidden: true should add a dashed hidden SVG layer"
    );
}

#[test]
fn svg_hidden_layer_is_off_by_default() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_no_hidden.svg");
    vm.eval(&format!("box(10,10,10).export('{}')", out.display()))
        .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("class=\"hidden\"") && !content.contains("stroke-dasharray"),
        "default SVG export should not include hidden-line styling"
    );
}

#[test]
fn svg_center_marks_adds_center_mark_group() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_center_marks.svg");
    vm.eval(&format!(
        "cylinder(5,20).export('{}', center_marks: true)",
        out.display()
    ))
    .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("class=\"center-marks\"") && content.contains("<line "),
        "center_marks: true should add a center-marks group"
    );
}

#[test]
fn svg_center_marks_off_by_default() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_no_center_marks.svg");
    vm.eval(&format!("cylinder(5,20).export('{}')", out.display()))
        .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("class=\"center-marks\""),
        "default SVG export should not include center marks"
    );
}

#[test]
fn svg_dimensions_adds_dimension_group() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_dimensions.svg");
    vm.eval(&format!(
        "box(20,10,5).export('{}', dimensions: true)",
        out.display()
    ))
    .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("class=\"dimensions\"") && content.contains(">20"),
        "dimensions: true should add a dimensions group with width text"
    );
}

#[test]
fn svg_dimensions_off_by_default() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_no_dimensions.svg");
    vm.eval(&format!("box(20,10,5).export('{}')", out.display()))
        .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("class=\"dimensions\""),
        "default SVG export should not include dimension annotations"
    );
}

// ---------------------------------------------------------------------------
// DXF export
// ---------------------------------------------------------------------------

#[test]
fn dxf_top_view_creates_file() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_top.dxf");
    let code = format!("box(20,10,5).export('{}')", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "DXF file was not created");
    assert!(out.metadata().unwrap().len() > 0, "DXF file is empty");
}

#[test]
fn dxf_contains_entities_section() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_entities.dxf");
    let code = format!("box(10,10,10).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("ENTITIES"),
        "DXF must contain ENTITIES section"
    );
}

#[test]
fn dxf_contains_line_entities() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_lines.dxf");
    let code = format!("box(10,10,10).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("LINE"), "DXF must contain LINE entities");
}

#[test]
fn dxf_front_view() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_front.dxf");
    let code = format!("box(10,10,10).export('{}', view: :front)", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "DXF front view file was not created");
}

#[test]
fn dxf_ends_with_eof_marker() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_eof.dxf");
    let code = format!("box(5,5,5).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("EOF"), "DXF must end with EOF marker");
}

#[test]
fn dxf_scale_expands_output_geometry() {
    let mut vm = MrubyVm::new();
    let normal = tmp("rrcad_test_scale_1.dxf");
    let scaled = tmp("rrcad_test_scale_2.dxf");
    vm.eval(&format!(
        "box(10,10,10).export('{}', scale: 1.0)",
        normal.display()
    ))
    .unwrap();
    vm.eval(&format!(
        "box(10,10,10).export('{}', scale: 2.0)",
        scaled.display()
    ))
    .unwrap();

    assert!(
        dxf_max_abs_xy(&scaled) > dxf_max_abs_xy(&normal),
        "scale: 2.0 should increase DXF drawing coordinates"
    );
}

#[test]
fn dxf_rejects_non_positive_scale() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_bad_scale.dxf");
    let err = vm
        .eval(&format!(
            "box(10,10,10).export('{}', scale: 0)",
            out.display()
        ))
        .expect_err("scale: 0 should fail");
    assert!(
        err.contains("scale") && err.contains("positive"),
        "expected actionable scale error, got: {err}"
    );
}

#[test]
fn dxf_center_marks_adds_center_layer() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_center_marks.dxf");
    vm.eval(&format!(
        "cylinder(5,20).export('{}', center_marks: true)",
        out.display()
    ))
    .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("\nCENTER\n") && content.contains("  0\nLINE\n"),
        "center_marks: true should add center-layer DXF lines"
    );
}

#[test]
fn dxf_center_marks_off_by_default() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_no_center_marks.dxf");
    vm.eval(&format!("cylinder(5,20).export('{}')", out.display()))
        .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("\nCENTER\n"),
        "default DXF export should not include center marks"
    );
}

#[test]
fn dxf_dimensions_adds_dimension_layer() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_dimensions.dxf");
    vm.eval(&format!(
        "box(20,10,5).export('{}', dimensions: true)",
        out.display()
    ))
    .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("\nDIMENSION\n") && content.contains("\nTEXT\n"),
        "dimensions: true should add a DIMENSION layer with text"
    );
}

#[test]
fn dxf_dimensions_off_by_default() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_no_dimensions.dxf");
    vm.eval(&format!("box(20,10,5).export('{}')", out.display()))
        .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("\nDIMENSION\n"),
        "default DXF export should not include dimension annotations"
    );
}

#[test]
fn dxf_hidden_option_adds_hidden_layer() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_hidden.dxf");
    vm.eval(&format!(
        "box(10,10,10).export('{}', hidden: true)",
        out.display()
    ))
    .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("\nHIDDEN\n"),
        "hidden: true should write hidden DXF entities on a HIDDEN layer"
    );
}

#[test]
fn dxf_hidden_layer_is_off_by_default() {
    let mut vm = MrubyVm::new();
    let out = tmp("rrcad_test_no_hidden.dxf");
    vm.eval(&format!("box(10,10,10).export('{}')", out.display()))
        .unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("\nHIDDEN\n"),
        "default DXF export should not include hidden-line entities"
    );
}
