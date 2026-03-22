// Phase 8 Tier 4 — 2-D drawing output
//
// Tests for:
//   shape.export("out.svg")              → SVG via HLRBRep_PolyAlgo
//   shape.export("out.svg", view: :front|:side)
//   shape.export("out.dxf")              → DXF R12 via HLRBRep_PolyAlgo

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------

#[test]
fn svg_top_view_creates_file() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_top.svg");
    let code = format!("box(20,10,5).export('{}')", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "SVG file was not created");
    assert!(out.metadata().unwrap().len() > 0, "SVG file is empty");
}

#[test]
fn svg_contains_xml_header() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_xml.svg");
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
    let out = std::env::temp_dir().join("rrcad_test_poly.svg");
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
    let out = std::env::temp_dir().join("rrcad_test_front.svg");
    let code = format!("box(10,10,10).export('{}', view: :front)", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "SVG front view file was not created");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<svg"), "front view SVG must be valid");
}

#[test]
fn svg_side_view() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_side.svg");
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
    let out = std::env::temp_dir().join("rrcad_test_cyl.svg");
    let code = format!("cylinder(5, 20).export('{}')", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "cylinder SVG was not created");
}

// ---------------------------------------------------------------------------
// DXF export
// ---------------------------------------------------------------------------

#[test]
fn dxf_top_view_creates_file() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_top.dxf");
    let code = format!("box(20,10,5).export('{}')", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "DXF file was not created");
    assert!(out.metadata().unwrap().len() > 0, "DXF file is empty");
}

#[test]
fn dxf_contains_entities_section() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_entities.dxf");
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
    let out = std::env::temp_dir().join("rrcad_test_lines.dxf");
    let code = format!("box(10,10,10).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("LINE"), "DXF must contain LINE entities");
}

#[test]
fn dxf_front_view() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_front.dxf");
    let code = format!("box(10,10,10).export('{}', view: :front)", out.display());
    vm.eval(&code).unwrap();
    assert!(out.exists(), "DXF front view file was not created");
}

#[test]
fn dxf_ends_with_eof_marker() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_test_eof.dxf");
    let code = format!("box(5,5,5).export('{}')", out.display());
    vm.eval(&code).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("EOF"), "DXF must end with EOF marker");
}
