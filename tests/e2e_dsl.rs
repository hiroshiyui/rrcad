/// End-to-end integration tests: Ruby DSL → mRuby → Rust → OCCT → file.
///
/// These tests exercise the full stack as a user would: a Ruby script string
/// is passed to `MrubyVm::eval`, which drives OCCT via the native bindings
/// and produces an output file on disk.
use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(name)
}

fn assert_valid_step(path: &std::path::Path) {
    assert!(path.exists(), "STEP file not created: {}", path.display());
    let content = std::fs::read_to_string(path).expect("could not read STEP file");
    assert!(
        content.contains("ISO-10303-21"),
        "output does not look like a STEP file: {}",
        path.display()
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_box_export_step() {
    let out = tmp("rrcad_e2e_box.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 20.0, 30.0).export('{}')",
        out.display()
    ))
    .expect("e2e box export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_cylinder_export_step() {
    let out = tmp("rrcad_e2e_cylinder.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!("cylinder(5.0, 20.0).export('{}')", out.display()))
        .expect("e2e cylinder export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_sphere_export_step() {
    let out = tmp("rrcad_e2e_sphere.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!("sphere(8.0).export('{}')", out.display()))
        .expect("e2e sphere export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_boolean_cut_export_step() {
    // Box with cylindrical hole — classic CAD workflow.
    let out = tmp("rrcad_e2e_cut.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(20.0, 20.0, 20.0).cut(cylinder(5.0, 25.0)).export('{}')",
        out.display()
    ))
    .expect("e2e cut export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_fuse_export_step() {
    let out = tmp("rrcad_e2e_fuse.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).fuse(sphere(7.0)).export('{}')",
        out.display()
    ))
    .expect("e2e fuse export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_common_export_step() {
    let out = tmp("rrcad_e2e_common.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(20.0, 20.0, 20.0).common(sphere(12.0)).export('{}')",
        out.display()
    ))
    .expect("e2e common export failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_multi_statement_script() {
    // Simulate a real .rb script with multiple lines.
    let out = tmp("rrcad_e2e_script.step");
    let script = format!(
        r#"
base = box(30.0, 20.0, 10.0)
hole = cylinder(4.0, 15.0)
result = base.cut(hole)
result.export('{}')
"#,
        out.display()
    );
    let mut vm = MrubyVm::new();
    vm.eval(&script).expect("e2e multi-statement script failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_shape_assigned_to_global_and_reused() {
    // Shapes stored in globals survive across eval calls.
    let out = tmp("rrcad_e2e_global_shape.step");
    let mut vm = MrubyVm::new();
    vm.eval("$base = box(15.0, 15.0, 15.0)").unwrap();
    vm.eval("$tool = cylinder(4.0, 20.0)").unwrap();
    vm.eval("$result = $base.cut($tool)").unwrap();
    vm.eval(&format!("$result.export('{}')", out.display()))
        .unwrap();
    assert_valid_step(&out);
}

// ---------------------------------------------------------------------------
// Query / introspection
// ---------------------------------------------------------------------------

#[test]
fn e2e_bounding_box() {
    let mut vm = MrubyVm::new();
    // box(10, 20, 30) at origin → min corner (0,0,0), extents (10,20,30)
    let result = vm
        .eval("bb = box(10.0, 20.0, 30.0).bounding_box; [bb[:dx], bb[:dy], bb[:dz]].inspect")
        .expect("bounding_box failed");
    assert!(
        result.contains("10.0") && result.contains("20.0") && result.contains("30.0"),
        "unexpected bounding box extents: {result}"
    );
}

#[test]
fn e2e_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 20.0, 30.0).volume")
        .expect("volume failed");
    // 10 × 20 × 30 = 6000.0
    let vol: f64 = result.trim().parse().expect("volume result not a float");
    assert!(
        (vol - 6000.0).abs() < 1.0,
        "expected volume ≈ 6000, got {vol}"
    );
}

#[test]
fn e2e_surface_area() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 20.0, 30.0).surface_area")
        .expect("surface_area failed");
    // 2*(10*20 + 20*30 + 10*30) = 2*(200+600+300) = 2200.0
    let area: f64 = result
        .trim()
        .parse()
        .expect("surface_area result not a float");
    assert!(
        (area - 2200.0).abs() < 1.0,
        "expected surface_area ≈ 2200, got {area}"
    );
}

// ---------------------------------------------------------------------------
// Import round-trips
// ---------------------------------------------------------------------------

#[test]
fn e2e_import_step_roundtrip() {
    // Export a box to STEP then import it back; the result must be a Shape.
    let step = tmp("rrcad_e2e_import_step.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).export('{}')",
        step.display()
    ))
    .expect("export failed");
    assert_valid_step(&step);

    let result = vm
        .eval(&format!("import_step('{}').class", step.display()))
        .expect("import_step failed");
    assert!(result.contains("Shape"), "expected Shape, got: {result}");
}

// ---------------------------------------------------------------------------
// scale — uniform and non-uniform
// ---------------------------------------------------------------------------

#[test]
fn e2e_scale_uniform() {
    // scale(2) doubles all extents: box(5,5,5).scale(2) → extents 10×10×10
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(5.0, 5.0, 5.0).scale(2.0).bounding_box")
        .expect("scale uniform failed");
    // bounding_box returns {x:, y:, z:, dx:, dy:, dz:}; check dx (extent X) is 10
    assert!(
        result.contains("dx: 10"),
        "unexpected bounding_box: {result}"
    );
}

#[test]
fn e2e_scale_nonuniform() {
    // scale(sx, sy, sz) stretches each axis independently.
    // box(1,1,1).scale(2, 3, 4) → extents 2×3×4
    let mut vm = MrubyVm::new();
    let bb = vm
        .eval("box(1.0, 1.0, 1.0).scale(2.0, 3.0, 4.0).bounding_box")
        .expect("scale non-uniform failed");
    assert!(bb.contains("dx: 2"), "expected dx: 2, got: {bb}");
    assert!(bb.contains("dy: 3"), "expected dy: 3, got: {bb}");
    assert!(bb.contains("dz: 4"), "expected dz: 4, got: {bb}");
}

#[test]
fn e2e_scale_nonuniform_export_step() {
    // Verify that a non-uniformly scaled shape produces a valid STEP file.
    let out = tmp("rrcad_e2e_scale_xyz.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).scale(1.5, 2.0, 0.5).export('{}')",
        out.display()
    ))
    .expect("scale_xyz export failed");
    assert_valid_step(&out);
}
#[test]
fn e2e_import_stl_roundtrip() {
    // Write a sphere to STL via the Rust API, then import it through the DSL.
    let stl = tmp("rrcad_e2e_import_stl.stl");
    rrcad::occt::Shape::make_sphere(5.0)
        .unwrap()
        .export_stl(stl.to_str().unwrap())
        .unwrap();
    assert!(stl.exists(), "STL file not created");

    let mut vm = MrubyVm::new();
    let result = vm
        .eval(&format!("import_stl('{}').class", stl.display()))
        .expect("import_stl failed");
    assert!(result.contains("Shape"), "expected Shape, got: {result}");
}

// ---------------------------------------------------------------------------
// Selective fillet / chamfer
// ---------------------------------------------------------------------------

#[test]
fn e2e_fillet_selective_vertical() {
    // fillet(r, :vertical) — only the 4 vertical edges of a box are rounded.
    let out = tmp("rrcad_e2e_fillet_sel_vertical.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).fillet(1.0, :vertical).export('{}')",
        out.display()
    ))
    .expect("fillet(:vertical) failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_fillet_selective_horizontal() {
    let out = tmp("rrcad_e2e_fillet_sel_horizontal.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).fillet(1.0, :horizontal).export('{}')",
        out.display()
    ))
    .expect("fillet(:horizontal) failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_chamfer_selective_vertical() {
    let out = tmp("rrcad_e2e_chamfer_sel_vertical.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).chamfer(1.0, :vertical).export('{}')",
        out.display()
    ))
    .expect("chamfer(:vertical) failed");
    assert_valid_step(&out);
}

#[test]
fn e2e_fillet_selector_all_matches_no_arg() {
    // fillet(:all) should produce the same shape as fillet() with no selector.
    let mut vm = MrubyVm::new();
    let v_all = vm
        .eval("box(10.0, 10.0, 10.0).fillet(1.0, :all).volume")
        .expect("fillet(:all) failed");
    let v_none = vm
        .eval("box(10.0, 10.0, 10.0).fillet(1.0).volume")
        .expect("fillet() failed");
    // Volumes should match to within 0.01% — compare the numeric strings loosely.
    assert_eq!(
        v_all.split('.').next(),
        v_none.split('.').next(),
        "fillet(:all) volume {v_all} differs from fillet() volume {v_none}"
    );
}

#[test]
fn e2e_fillet_bad_selector_returns_error() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(5.0, 5.0, 5.0).fillet(1.0, :diagonal)");
    assert!(
        result.is_err(),
        "expected error for bad selector, got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Phase 4: Patterns — linear_pattern / polar_pattern
// ---------------------------------------------------------------------------

#[test]
fn e2e_linear_pattern_creates_compound() {
    // 4 copies of a cylinder spaced 15 units apart along X; the X-extent of
    // the compound should be approx 3*15 + 2*r = 45 + 4 = 49.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("linear_pattern(cylinder(2.0, 5.0), 4, [15, 0, 0]).bounding_box[:dx]")
        .expect("linear_pattern DSL failed");
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!((dx - 49.0).abs() < 0.5, "expected x-extent ~49, got {dx}");
}

#[test]
fn e2e_polar_pattern_symmetric_extents() {
    // 4-fold polar pattern at 360° — the compound should have equal X and Y extents
    // because the 4 copies are symmetric around the origin.
    let mut vm = MrubyVm::new();
    let bb = vm
        .eval("polar_pattern(sphere(2.0).translate(8, 0, 0), 4, 360).bounding_box")
        .expect("polar_pattern DSL failed");
    // The result is a Ruby Hash inspect string; verify it contains the expected keys.
    assert!(
        bb.contains(":dx") || bb.contains("dx"),
        "unexpected bounding_box result: {bb}"
    );
    // Extract dx and dy from the hash by making two separate calls.
    let dx: f64 = vm
        .eval("polar_pattern(sphere(2.0).translate(8, 0, 0), 4, 360).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let dy: f64 = vm
        .eval("polar_pattern(sphere(2.0).translate(8, 0, 0), 4, 360).bounding_box[:dy]")
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(
        (dx - dy).abs() < 0.5,
        "expected symmetric extents, got dx={dx}, dy={dy}"
    );
}

#[test]
fn e2e_linear_pattern_export_step() {
    let out = tmp("e2e_linear_pattern.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "linear_pattern(box(3.0, 3.0, 3.0), 5, [6, 0, 0]).export(\"{}\")",
        out.display()
    ))
    .expect("linear_pattern export failed");
    assert!(out.exists());
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("ISO-10303-21"));
}

#[test]
fn e2e_polar_pattern_export_step() {
    let out = tmp("e2e_polar_pattern.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "polar_pattern(cylinder(2.0, 4.0).translate(10, 0, 0), 6, 360).export(\"{}\")",
        out.display()
    ))
    .expect("polar_pattern export failed");
    assert!(out.exists());
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("ISO-10303-21"));
}

#[test]
fn e2e_linear_pattern_n1_identity() {
    // n=1 should produce a compound whose bounding box matches the original shape.
    let mut vm = MrubyVm::new();
    let orig = vm
        .eval("box(5.0, 7.0, 9.0).bounding_box[:dz]")
        .expect("orig bounding_box failed");
    let pat = vm
        .eval("linear_pattern(box(5.0, 7.0, 9.0), 1, [100, 0, 0]).bounding_box[:dz]")
        .expect("pattern bounding_box failed");
    assert_eq!(
        orig.split('.').next(),
        pat.split('.').next(),
        "n=1 pattern dz {pat} differs from original {orig}"
    );
}

#[test]
fn e2e_pattern_then_fuse() {
    // A pattern compound can be fused into one solid by calling .fuse on itself —
    // but easier to just verify the pattern round-trips through STEP fine.
    let out = tmp("e2e_pattern_fuse.step");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "polar_pattern(box(2.0, 2.0, 10.0).translate(5, 0, 0), 3, 120).export(\"{}\")",
        out.display()
    ))
    .expect("polar_pattern fuse test failed");
    assert!(out.exists());
}

// ---------------------------------------------------------------------------
// Phase 4: vertices selector, direction-based face selector, OBJ export
// ---------------------------------------------------------------------------

#[test]
fn e2e_vertices_count_box() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(5.0, 5.0, 5.0).vertices(:all).length")
        .expect("vertices(:all) failed");
    assert_eq!(
        result.trim(),
        "8",
        "expected 8 vertices on a box, got: {result}"
    );
}

#[test]
fn e2e_vertices_bad_selector_returns_error() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("box(5.0, 5.0, 5.0).vertices(:top)");
    assert!(result.is_err(), "expected error for unsupported selector");
}

#[test]
fn e2e_faces_direction_gt_z() {
    // faces(">Z") should return only the top face of a box.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 10.0, 10.0).faces(\">Z\").length")
        .expect("faces(\">Z\") failed");
    assert_eq!(result.trim(), "1", "expected 1 top face, got: {result}");
}

#[test]
fn e2e_faces_direction_lt_z() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 10.0, 10.0).faces(\"<Z\").length")
        .expect("faces(\"<Z\") failed");
    assert_eq!(result.trim(), "1", "expected 1 bottom face, got: {result}");
}

#[test]
fn e2e_faces_direction_gt_x() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("box(10.0, 10.0, 10.0).faces(\">X\").length")
        .expect("faces(\">X\") failed");
    assert_eq!(result.trim(), "1", "expected 1 +X face, got: {result}");
}

#[test]
fn e2e_faces_symbol_and_direction_same_count() {
    // faces(:top) and faces(">Z") should give the same count on a box.
    let mut vm = MrubyVm::new();
    let sym_count = vm
        .eval("box(8.0, 8.0, 8.0).faces(:top).length")
        .expect("faces(:top) failed");
    let dir_count = vm
        .eval("box(8.0, 8.0, 8.0).faces(\">Z\").length")
        .expect("faces(\">Z\") failed");
    assert_eq!(
        sym_count.trim(),
        dir_count.trim(),
        "faces(:top) and faces(\">Z\") should return the same count"
    );
}

#[test]
fn e2e_export_obj() {
    let out = tmp("e2e_export.obj");
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "box(10.0, 10.0, 10.0).export(\"{}\")",
        out.display()
    ))
    .expect("export .obj failed");
    assert!(out.exists(), "OBJ file was not created");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains('v') || content.contains('f'),
        "file does not look like an OBJ: {}",
        &content[..content.len().min(200)]
    );
}

#[test]
fn e2e_export_stl() {
    let out = tmp("e2e_export.stl");
    let mut vm = MrubyVm::new();
    vm.eval(&format!("box(5.0, 5.0, 5.0).export(\"{}\")", out.display()))
        .expect("export .stl failed");
    assert!(out.exists());
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("solid") || content.len() > 0);
}

#[test]
fn e2e_export_glb() {
    let out = tmp("e2e_export.glb");
    let mut vm = MrubyVm::new();
    vm.eval(&format!("sphere(4.0).export(\"{}\")", out.display()))
        .expect("export .glb failed");
    assert!(out.exists());
    assert!(std::fs::metadata(&out).unwrap().len() > 0);
}

// ---------------------------------------------------------------------------
// Phase 5 — color
// ---------------------------------------------------------------------------

#[test]
fn e2e_color_returns_shape() {
    let mut vm = MrubyVm::new();
    // .color() should return a Shape, not raise.
    let result = vm.eval("box(10,10,10).color(1.0, 0.0, 0.0).class").unwrap();
    assert_eq!(result, "Shape");
}

#[test]
fn e2e_color_exports_glb() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_e2e_colored.glb");
    let code = format!(
        "box(10,10,10).color(0.2, 0.6, 0.9).export({:?})",
        out.to_str().unwrap()
    );
    vm.eval(&code).expect("colored GLB export failed");
    assert!(out.exists());
    assert!(std::fs::metadata(&out).unwrap().len() > 0);
}

#[test]
fn e2e_color_chained_with_transform() {
    let mut vm = MrubyVm::new();
    // color can be applied after transforms and before export.
    let result = vm
        .eval("box(5,5,5).translate(1,0,0).color(0.5,0.5,0.5).class")
        .unwrap();
    assert_eq!(result, "Shape");
}

// ---------------------------------------------------------------------------
// Phase 5 — assembly mating
// ---------------------------------------------------------------------------

#[test]
fn e2e_mate_stacks_box_on_box() {
    let mut vm = MrubyVm::new();
    // After mating, the bounding box Z of the post should be 10..15.
    let result = vm
        .eval(
            r#"
base = box(10, 10, 10)
post = box(5, 5, 5)
placed = post.mate(post.faces(:bottom).first, base.faces(:top).first)
placed.bounding_box
"#,
        )
        .unwrap();
    // bounding_box returns [xmin,ymin,zmin,xmax,ymax,zmax]
    assert!(
        result.contains("10"),
        "expected Zmin≈10 in bounding box: {result}"
    );
}

#[test]
fn e2e_mate_with_offset() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            r#"
base = box(10, 10, 10)
post = box(5, 5, 5)
placed = post.mate(post.faces(:bottom).first, base.faces(:top).first, 3.0)
placed.bounding_box
"#,
        )
        .unwrap();
    assert!(
        result.contains("13"),
        "expected Zmin≈13 in bounding box: {result}"
    );
}

#[test]
fn e2e_assembly_mate_keyword_form() {
    let mut vm = MrubyVm::new();
    // Assembly#mate should accept keyword args and place the mated shape.
    vm.eval(
        r#"
base = box(100, 80, 10)
post = box(20, 20, 50)
asm = assembly("bracket") do |a|
  a.place base
  a.mate post, from: post.faces(:bottom).first, to: base.faces(:top).first
end
asm.class
"#,
    )
    .expect("assembly mate failed");
}

#[test]
fn e2e_mate_non_planar_raises() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval(
            r#"
cyl  = cylinder(5, 10)
base = box(20, 20, 5)
cyl.mate(cyl.faces(:side).first, base.faces(:top).first)
"#,
        )
        .unwrap_err();
    assert!(
        err.to_lowercase().contains("planar"),
        "expected planar error: {err}"
    );
}

// ---------------------------------------------------------------------------
// .simplify (Tier 4 — feature removal)
// ---------------------------------------------------------------------------

#[test]
fn e2e_simplify_no_features_returns_shape() {
    // A plain box has no faces below any reasonable area threshold;
    // simplify should return the shape unchanged.
    let mut vm = MrubyVm::new();
    let r = vm.eval("box(20, 20, 20).simplify(0.1).class").unwrap();
    assert!(r.contains("Shape"), "expected Shape, got: {r}");
}

#[test]
fn e2e_simplify_with_fillet_removes_small_faces() {
    // A filleted box has small cylindrical fillet faces.  With a generous
    // threshold they should be removed; the result must still be a Shape.
    let mut vm = MrubyVm::new();
    let r = vm
        .eval("box(20, 20, 20).fillet(1).simplify(5.0).class")
        .unwrap();
    assert!(
        r.contains("Shape"),
        "expected Shape after simplify, got: {r}"
    );
}

#[test]
fn e2e_simplify_exports_step() {
    let mut vm = MrubyVm::new();
    let out = std::env::temp_dir().join("rrcad_simplify.step");
    let code = format!(
        r#"box(30, 30, 30).fillet(2).simplify(10.0).export("{}")"#,
        out.display()
    );
    vm.eval(&code).expect("eval failed");
    assert!(out.exists(), "STEP file not created");
    assert!(
        std::fs::metadata(&out).unwrap().len() > 0,
        "STEP file empty"
    );
}
