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
