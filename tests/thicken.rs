// `thicken` — give a surface a wall, turning it into a solid.
//
// The counterpart to `shell`: `shell` takes material out of a solid, `thicken`
// puts a solid around a surface that has none. It is how a lofted or filled
// surface becomes a part that can be machined or printed.
//
// The tests check volume against the arithmetic for the same shape, because
// the failure mode here is not an exception. A thickening that offsets the
// faces without closing the sides produces a shape that has the right bounding
// box, draws correctly in a viewer, and encloses nothing — and one built with
// its faces facing inward reports a negative volume and cuts material where it
// should add it. Both look fine until something asks how much material there
// is.

use rrcad::ruby::vm::MrubyVm;

/// Evaluate `script` and return its final value as text.
fn eval(script: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(script)
        .unwrap_or_else(|e| panic!("script failed: {e}\n{script}"))
        .trim()
        .to_string()
}

/// Evaluate `script`, expecting it to raise, and return the message.
fn err(script: &str) -> String {
    let mut vm = MrubyVm::new();
    match vm.eval(script) {
        Ok(v) => panic!("expected a refusal, got {v:?}"),
        Err(e) => e.to_string(),
    }
}

fn number(script: &str) -> f64 {
    eval(script).parse().expect("expected a number")
}

#[test]
fn a_flat_face_thickens_into_a_plate() {
    // The simplest case, and the one with an unarguable answer: a 20 × 10 face
    // given a 2 mm wall is 400 mm³ of material and nothing else.
    assert_eq!(eval("rect(20, 10).thicken(2).shape_type"), ":solid");
    assert!((number("rect(20, 10).thicken(2).volume") - 400.0).abs() < 1e-9);
}

#[test]
fn the_wall_grows_along_the_surface_normal() {
    // Where the material lands, not just how much of it there is. The face sits
    // in the z = 0 plane, so a positive thickness has to occupy 0..2.
    let bb = eval("rect(20, 10).thicken(2).bounding_box");
    assert!(
        bb.contains("z: 0.0") && bb.contains("dz: 2.0"),
        "expected the plate to sit above the face: {bb}"
    );
}

#[test]
fn a_negative_thickness_builds_on_the_other_side() {
    // Same material, opposite side — which is the only way to thicken a surface
    // whose normal points away from where the part should be.
    let bb = eval("rect(20, 10).thicken(-2).bounding_box");
    assert!(
        bb.contains("z: -2.0") && bb.contains("dz: 2.0"),
        "expected the plate below the face: {bb}"
    );
    assert!((number("rect(20, 10).thicken(-2).volume") - 400.0).abs() < 1e-9);
}

#[test]
fn a_curved_surface_keeps_its_curvature() {
    // A cylinder's side face at r = 10, h = 20, walled outward by 1, is the
    // tube between r = 10 and r = 11: π(11² − 10²) × 20 = 1319.469…
    //
    // This is the case that separates a real offset from an extrusion along one
    // direction: extruding this face would not produce a tube at all.
    let v = number("cylinder(10, 20).faces(:side).first.thicken(1).volume");
    let expected = std::f64::consts::PI * (11.0f64.powi(2) - 10.0f64.powi(2)) * 20.0;
    assert!(
        (v - expected).abs() < 1e-6,
        "expected the tube volume {expected}, got {v}"
    );
}

#[test]
fn the_result_is_a_solid_that_takes_part_in_booleans() {
    // Producing a solid is the whole point: a closed shell would report zero
    // volume and cut nothing, so a part built on one would silently ignore
    // every feature applied to it afterwards. Checking a boolean rather than
    // `shape_type` tests the property that matters instead of the label.
    let v = number("rect(20, 10).thicken(2).cut(cylinder(2, 10).translate(10, 5, -1)).volume");
    let expected = 400.0 - std::f64::consts::PI * 4.0 * 2.0;
    assert!(
        (v - expected).abs() < 1e-6,
        "expected {expected} after the bore, got {v}"
    );
}

#[test]
fn the_solid_is_not_inside_out() {
    // An inverted solid has a negative volume and adds material where it should
    // remove it. `volume` alone would not catch it if the sign were dropped, so
    // this checks the fused result: fusing an inside-out solid with a box that
    // sits inside it does not give the box back.
    assert!(number("rect(20, 10).thicken(2).volume") > 0.0);
    let fused = number("rect(20, 10).thicken(2).fuse(box(4, 4, 2)).volume");
    assert!(
        (fused - 400.0).abs() < 1e-6,
        "a box already inside the plate should add nothing, got {fused}"
    );
}

#[test]
fn a_solid_is_refused_and_told_what_to_use_instead() {
    // A solid already has thickness. Quietly offsetting or hollowing it would
    // be a different operation than the one that was asked for, so the error
    // names both of the things the caller might have meant.
    let e = err("box(10, 10, 10).thicken(1)");
    assert!(
        e.contains("Face or Shell"),
        "the error should say what thicken accepts: {e}"
    );
    assert!(
        e.contains("offset") && e.contains("shell"),
        "the error should point at the two operations for a solid: {e}"
    );
}

#[test]
fn the_thickness_may_be_given_in_units() {
    // Consistency with the rest of the DSL — every other length accepts a unit.
    let plain = number("rect(20, 10).thicken(2).volume");
    let united = number("rect(20, 10).thicken(2.mm).volume");
    assert!((plain - united).abs() < 1e-9);
}

#[test]
fn a_thickened_surface_can_be_rebuilt_from_its_feature_graph() {
    // `thicken` records itself as a feature, so a part built on one still
    // replays. A missing rebuild arm would only show up here.
    let v = number("rect(20, 10).thicken(2).rebuild.volume");
    assert!((v - 400.0).abs() < 1e-9, "rebuilt volume was {v}");
}
