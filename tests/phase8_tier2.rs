// Phase 8 Tier 2 — Manufacturing features
//
// Tests for:
//   shape.extrude(h, draft: angle_deg) → Solid with tapered side walls
//   helix(radius:, pitch:, height:)    → Wire path
//   thread(solid, :side, pitch:, depth:) → Solid (cut)  [pure Ruby DSL]
//   cbore(d:, cbore_d:, cbore_h:, depth:)               [pure Ruby DSL]
//   csink(d:, csink_d:, csink_angle:, depth:)            [pure Ruby DSL]

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// standard hardware helpers
// ---------------------------------------------------------------------------

#[test]
fn clearance_hole_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("clearance_hole(:m3, depth: 10).shape_type")
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn clearance_hole_uses_standard_size() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("clearance_hole(:m3, depth: 10).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 3.4).abs() < 0.2,
        "expected M3 clearance diameter near 3.4, got {dx}"
    );
}

#[test]
fn clearance_hole_accepts_numeric_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("clearance_hole(6.2, depth: 10).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 6.2).abs() < 0.2,
        "expected diameter near 6.2, got {dx}"
    );
}

#[test]
fn clearance_hole_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(20, 20, 4)
             hole = clearance_hole(:m3, depth: 6).translate(10, 10, -1)
             plate.cut(hole).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "clearance hole should reduce plate volume");
}

#[test]
fn clearance_hole_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("clearance_hole(:m9, depth: 10)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

#[test]
fn tap_drill_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("tap_drill(:m3, depth: 8).shape_type").unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn tap_drill_uses_standard_size() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("tap_drill(:m3, depth: 8).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 2.5).abs() < 0.2,
        "expected M3 tap drill diameter near 2.5, got {dx}"
    );
}

#[test]
fn tap_drill_accepts_numeric_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("tap_drill(2.8, depth: 8).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 2.8).abs() < 0.2,
        "expected diameter near 2.8, got {dx}"
    );
}

#[test]
fn tap_drill_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "block = box(20, 20, 6)
             tool = tap_drill(:m3, depth: 8).translate(10, 10, -1)
             block.cut(tool).volume < block.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "tap drill should reduce block volume");
}

#[test]
fn tap_drill_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("tap_drill(:m9, depth: 10)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

#[test]
fn heat_set_insert_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("heat_set_insert(:m3, depth: 5).shape_type")
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn heat_set_insert_uses_standard_size() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("heat_set_insert(:m3, depth: 5).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 4.6).abs() < 0.2,
        "expected M3 insert pilot diameter near 4.6, got {dx}"
    );
}

#[test]
fn heat_set_insert_accepts_numeric_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("heat_set_insert(5.1, depth: 5).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 5.1).abs() < 0.2,
        "expected diameter near 5.1, got {dx}"
    );
}

#[test]
fn heat_set_insert_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "boss = cylinder(5, 8)
             tool = heat_set_insert(:m3, depth: 6).translate(0, 0, 1)
             boss.cut(tool).volume < boss.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "insert pilot should reduce boss volume");
}

#[test]
fn heat_set_insert_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("heat_set_insert(:m5, depth: 5)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

#[test]
fn socket_head_cbore_returns_shape() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("socket_head_cbore(:m3, depth: 10, head_depth: 3).shape_type")
        .unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected :solid or :compound, got {result}"
    );
}

#[test]
fn socket_head_cbore_uses_standard_head_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("socket_head_cbore(:m3, depth: 10, head_depth: 3).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 6.0).abs() < 0.3,
        "expected M3 socket head cbore diameter near 6.0, got {dx}"
    );
}

#[test]
fn socket_head_cbore_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(30, 30, 8)
             tool = socket_head_cbore(:m3, depth: 10, head_depth: 3).translate(15, 15, -1)
             plate.cut(tool).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(
        result, "true",
        "socket-head counterbore should reduce plate volume"
    );
}

#[test]
fn socket_head_cbore_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("socket_head_cbore(:m9, depth: 10, head_depth: 3)")
        .unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

#[test]
fn flat_head_csink_returns_shape() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("flat_head_csink(:m3, depth: 10).shape_type")
        .unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected :solid or :compound, got {result}"
    );
}

#[test]
fn flat_head_csink_uses_standard_head_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("flat_head_csink(:m3, depth: 10).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 6.3).abs() < 0.4,
        "expected M3 flat head csink diameter near 6.3, got {dx}"
    );
}

#[test]
fn flat_head_csink_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(30, 30, 8)
             tool = flat_head_csink(:m3, depth: 10).translate(15, 15, -1)
             plate.cut(tool).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(
        result, "true",
        "flat-head countersink should reduce plate volume"
    );
}

#[test]
fn flat_head_csink_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("flat_head_csink(:m9, depth: 10)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Bearing bore
// ---------------------------------------------------------------------------

#[test]
fn bearing_bore_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("bearing_bore(:b608, depth: 7).shape_type").unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn bearing_bore_uses_standard_od() {
    let mut vm = MrubyVm::new();
    // 608 outer diameter is 22 mm; default :press fit shrinks by 0.01 mm.
    let result = vm
        .eval("bearing_bore(:b608, depth: 7).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 22.0).abs() < 0.1,
        "expected 608 bearing bore near 22 mm, got {dx}"
    );
}

#[test]
fn bearing_bore_slip_fit_is_larger_than_press_fit() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "press = bearing_bore(:b608, depth: 7, fit: :press).bounding_box[:dx]
             slip  = bearing_bore(:b608, depth: 7, fit: :slip).bounding_box[:dx]
             slip > press",
        )
        .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn bearing_bore_accepts_numeric_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("bearing_bore(20.0, depth: 5).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!((dx - 20.0).abs() < 0.1, "expected ~20 mm, got {dx}");
}

#[test]
fn bearing_bore_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(40, 40, 10)
             tool = bearing_bore(:b608, depth: 12).translate(20, 20, -1)
             plate.cut(tool).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "bearing bore should reduce plate volume");
}

#[test]
fn bearing_bore_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("bearing_bore(:b999, depth: 5)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

#[test]
fn bearing_bore_rejects_unknown_fit() {
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("bearing_bore(:b608, depth: 5, fit: :tight)")
        .unwrap_err();
    assert!(
        err.contains("unsupported fit"),
        "expected unsupported fit error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Shaft fits
// ---------------------------------------------------------------------------

#[test]
fn shaft_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("shaft(8, length: 20).shape_type").unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn shaft_nominal_matches_diameter() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("shaft(8, length: 20).bounding_box[:dx]").unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!((dx - 8.0).abs() < 0.01, "expected ~8 mm, got {dx}");
}

#[test]
fn shaft_press_fit_is_larger_than_nominal() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "shaft(8, length: 20, fit: :press).bounding_box[:dx] > \
             shaft(8, length: 20, fit: :nominal).bounding_box[:dx]",
        )
        .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn shaft_running_fit_is_smaller_than_slip_fit() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "shaft(8, length: 20, fit: :running).bounding_box[:dx] < \
             shaft(8, length: 20, fit: :slip).bounding_box[:dx]",
        )
        .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn shaft_height_matches_length() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("shaft(8, length: 20).bounding_box[:dz]").unwrap();
    let dz: f64 = result.trim().parse().expect("expected a float");
    assert!((dz - 20.0).abs() < 0.01, "expected length 20, got {dz}");
}

#[test]
fn shaft_rejects_unknown_fit() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("shaft(8, length: 20, fit: :tight)").unwrap_err();
    assert!(
        err.contains("unsupported fit"),
        "expected unsupported fit error, got: {err}"
    );
}

#[test]
fn shaft_rejects_non_positive_diameter() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("shaft(0, length: 20)").unwrap_err();
    assert!(err.contains("must be > 0"), "got: {err}");
}

// ---------------------------------------------------------------------------
// Standard fasteners (screw bodies)
// ---------------------------------------------------------------------------

#[test]
fn screw_socket_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("screw(:m3, length: 12).shape_type").unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected :solid or :compound, got {result}"
    );
}

#[test]
fn screw_socket_head_widens_shank() {
    let mut vm = MrubyVm::new();
    // M3 SHCS head OD is 5.5 mm, shank is 3 mm — overall bbox should match the head.
    let result = vm
        .eval("screw(:m3, length: 12, style: :socket).bounding_box[:dx]")
        .unwrap();
    let dx: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dx - 5.5).abs() < 0.1,
        "expected ~5.5 mm M3 SHCS head, got {dx}"
    );
}

#[test]
fn screw_socket_total_height_is_length_plus_head() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("screw(:m3, length: 12, style: :socket).bounding_box[:dz]")
        .unwrap();
    let dz: f64 = result.trim().parse().expect("expected a float");
    // M3 SHCS head height ≈ 3.0 mm, shank 12 mm.
    assert!((dz - 15.0).abs() < 0.1, "expected ~15 mm, got {dz}");
}

#[test]
fn screw_flat_head_is_wider_than_button_head() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "screw(:m3, length: 12, style: :flat).bounding_box[:dx] > \
             screw(:m3, length: 12, style: :button).bounding_box[:dx]",
        )
        .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn screw_button_head_is_shorter_than_socket_head() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "screw(:m3, length: 12, style: :button).bounding_box[:dz] < \
             screw(:m3, length: 12, style: :socket).bounding_box[:dz]",
        )
        .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn screw_rejects_unknown_size() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("screw(:m9, length: 12)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported size error, got: {err}"
    );
}

#[test]
fn screw_rejects_unknown_style() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("screw(:m3, length: 12, style: :pan)").unwrap_err();
    assert!(
        err.contains("unsupported style"),
        "expected unsupported style error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Draft angle extrude
// ---------------------------------------------------------------------------

#[test]
fn extrude_draft_returns_solid() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("rect(10, 10).extrude(20, draft: 5).shape_type")
        .unwrap();
    assert_eq!(result, ":solid");
}

#[test]
fn extrude_no_draft_unchanged() {
    // draft: 0 should behave identically to plain extrude.
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "a = rect(10, 10).extrude(20)
             b = rect(10, 10).extrude(20, draft: 0)
             (a.volume - b.volume).abs < 0.1",
        )
        .unwrap();
    assert_eq!(result, "true", "draft:0 should match plain extrude volume");
}

#[test]
fn extrude_draft_tapers_top_face() {
    let mut vm = MrubyVm::new();
    // A rect extruded with draft has a smaller top bounding box extent.
    // The base 10×10 extruded 20 mm with 5° draft means each side moves in
    // by 20*tan(5°) ≈ 1.75 mm → top face is roughly 6.5×6.5.
    // Check volume < plain extrude volume (it must be, since the solid tapers).
    let result = vm
        .eval(
            "plain = rect(10, 10).extrude(20)
             tapered = rect(10, 10).extrude(20, draft: 5)
             tapered.volume < plain.volume",
        )
        .unwrap();
    assert_eq!(
        result, "true",
        "tapered solid should have less volume than straight extrude"
    );
}

#[test]
fn extrude_draft_top_face_smaller() {
    let mut vm = MrubyVm::new();
    // Top face of a drafted solid must have smaller bounding extents than the base.
    // We measure by comparing the top face surface area to the base.
    let result = vm
        .eval(
            "s = rect(10, 10).extrude(20, draft: 5)
             top    = s.faces(:top).first
             bottom = s.faces(:bottom).first
             top.surface_area < bottom.surface_area",
        )
        .unwrap();
    assert_eq!(
        result, "true",
        "top face area should be < bottom face area after draft"
    );
}

// ---------------------------------------------------------------------------
// helix
// ---------------------------------------------------------------------------

#[test]
fn helix_returns_wire() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval("helix(radius: 5, pitch: 1.5, height: 6).shape_type")
        .unwrap();
    assert_eq!(result, ":wire");
}

#[test]
fn helix_has_correct_z_extent() {
    let mut vm = MrubyVm::new();
    // A helix of height 10 should have Z extent ≈ 10.
    let result = vm
        .eval(
            "h = helix(radius: 5, pitch: 2.0, height: 10)
             bb = h.bounding_box
             bb[:dz]",
        )
        .unwrap();
    let dz: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (dz - 10.0).abs() < 0.5,
        "helix height should be ≈10, got {dz}"
    );
}

#[test]
fn helix_xy_extent_matches_radius() {
    let mut vm = MrubyVm::new();
    // X and Y extents of the helix bounding box should be ≈ 2 * radius.
    let result = vm
        .eval(
            "h = helix(radius: 8, pitch: 1.0, height: 3)
             bb = h.bounding_box
             [bb[:dx], bb[:dy]].min",
        )
        .unwrap();
    let extent: f64 = result.trim().parse().expect("expected a float");
    assert!(
        (extent - 16.0).abs() < 1.0,
        "helix XY extent should be ≈16 (2×radius), got {extent}"
    );
}

// ---------------------------------------------------------------------------
// thread (pure Ruby DSL — uses helix + sweep + cut)
// ---------------------------------------------------------------------------

#[test]
fn thread_returns_solid() {
    let mut vm = MrubyVm::new();
    // thread cuts via BRepAlgoAPI_Cut; OCCT 7.6+ may wrap the result in a compound.
    let result = vm
        .eval(
            "bolt = cylinder(5, 12)
             thread(bolt, :side, pitch: 1.0, depth: 0.6).shape_type",
        )
        .unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected :solid or :compound, got {result}"
    );
}

#[test]
fn thread_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "bolt = cylinder(5, 12)
             threaded = thread(bolt, :side, pitch: 1.0, depth: 0.6)
             threaded.volume < bolt.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "threading should reduce the solid volume");
}

// ---------------------------------------------------------------------------
// cbore (pure Ruby DSL — produces a stepped hole tool)
// ---------------------------------------------------------------------------

#[test]
fn cbore_returns_3d_shape() {
    let mut vm = MrubyVm::new();
    // cbore returns a solid (or compound-of-solids on OCCT 7.6+).
    let result = vm
        .eval("cbore(d: 5, cbore_d: 9, cbore_h: 4, depth: 20).shape_type")
        .unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected :solid or :compound, got {result}"
    );
}

#[test]
fn cbore_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    // Cut a cbore tool from a plate — volume must decrease.
    let result = vm
        .eval(
            "plate = box(50, 50, 20)
             hole  = cbore(d: 5, cbore_d: 9, cbore_h: 4, depth: 20)
             plate.cut(hole).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "cbore cut should reduce plate volume");
}

// ---------------------------------------------------------------------------
// csink (pure Ruby DSL — produces a conical countersink tool)
// ---------------------------------------------------------------------------

#[test]
fn csink_returns_3d_shape() {
    let mut vm = MrubyVm::new();
    // csink returns a solid (or compound-of-solids on OCCT 7.6+).
    let result = vm
        .eval("csink(d: 4, csink_d: 8, csink_angle: 45, depth: 20).shape_type")
        .unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected :solid or :compound, got {result}"
    );
}

#[test]
fn csink_cut_reduces_volume() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "plate = box(50, 50, 20)
             hole  = csink(d: 4, csink_d: 8, csink_angle: 45, depth: 20)
             plate.cut(hole).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(result, "true", "csink cut should reduce plate volume");
}
