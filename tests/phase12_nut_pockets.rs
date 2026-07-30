/// Phase 12 — nut-pocket and standoff-pocket helpers.
///
/// The hardware library (clearance_hole, tap_drill, heat_set_insert,
/// bearing_bore) stopped short of two staples of printed frames: hex
/// recesses for captive nuts and pockets that keep threaded standoffs from
/// spinning. Both are cut tools in the same table-driven pure-Ruby pattern,
/// sharing the nut across-flats table.
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
// nut_pocket geometry
// ---------------------------------------------------------------------------

#[test]
fn m3_pocket_is_a_solid_tool() {
    assert_eq!(eval("nut_pocket(:m3, depth: 3).shape_type"), ":solid");
}

#[test]
fn pocket_across_flats_is_nut_size_plus_clearance() {
    // M3 across-flats is 5.5; flats face ±X, so the bounding box X extent
    // reads the across-flats directly. Default clearance 0.2 adds 0.4.
    let dx = eval_num("nut_pocket(:m3, depth: 3).bounding_box[:dx]");
    assert!((dx - 5.9).abs() < 1e-6, "expected 5.9, got {dx}");
    let nominal = eval_num("nut_pocket(:m3, depth: 3, clearance: 0).bounding_box[:dx]");
    assert!((nominal - 5.5).abs() < 1e-6, "expected 5.5, got {nominal}");
}

#[test]
fn pocket_volume_matches_the_hex_prism() {
    // Regular hexagon area with across-flats a is (√3/2)·a².
    let v = eval_num("nut_pocket(:m3, depth: 3, clearance: 0).volume");
    let expected = 3.0_f64.sqrt() / 2.0 * 5.5 * 5.5 * 3.0;
    assert!(
        (v - expected).abs() < 0.5,
        "hex prism volume: expected ~{expected}, got {v}"
    );
}

#[test]
fn pocket_extrudes_upward_from_z_zero() {
    let z = eval_num("nut_pocket(:m3, depth: 3).bounding_box[:z]");
    let dz = eval_num("nut_pocket(:m3, depth: 3).bounding_box[:dz]");
    assert!(z.abs() < 1e-9 && (dz - 3.0).abs() < 1e-9, "z={z}, dz={dz}");
}

#[test]
fn square_style_makes_a_square_pocket() {
    let dx = eval_num("nut_pocket(:m3, depth: 3, style: :square, clearance: 0).bounding_box[:dx]");
    let dy = eval_num("nut_pocket(:m3, depth: 3, style: :square, clearance: 0).bounding_box[:dy]");
    assert!(
        (dx - 5.5).abs() < 1e-6 && (dy - 5.5).abs() < 1e-6,
        "square pocket should be 5.5×5.5, got {dx}×{dy}"
    );
}

#[test]
fn numeric_size_uses_the_nominal_thread_heuristic() {
    // Numeric = nominal thread diameter, across-flats estimated at 1.7×.
    let dx = eval_num("nut_pocket(6.0, depth: 4, clearance: 0).bounding_box[:dx]");
    assert!((dx - 10.2).abs() < 1e-6, "expected 1.7×6 = 10.2, got {dx}");
}

#[test]
fn imperial_sizes_are_supported() {
    let dx = eval_num(r#"nut_pocket(:"4-40", depth: 3, clearance: 0).bounding_box[:dx]"#);
    assert!((dx - 4.76).abs() < 1e-6, "4-40 across flats: {dx}");
}

// ---------------------------------------------------------------------------
// slot: — slide-in nut channel
// ---------------------------------------------------------------------------

#[test]
fn slot_extends_the_pocket_along_y() {
    // Channel runs from the hex centre to y = slot; the hex itself reaches
    // down to −R (R = across-corners/2 = af/√3). Width stays the across-flats.
    let code = "p = nut_pocket(:m3, depth: 3, clearance: 0, slot: 12)
                [p.bounding_box[:dx], p.bounding_box[:dy], p.bounding_box[:y]]";
    let out = eval(code);
    let v: Vec<f64> = out
        .trim_matches(['[', ']'])
        .split(',')
        .map(|t| t.trim().parse().unwrap())
        .collect();
    let r = 5.5 / 3.0_f64.sqrt();
    assert!((v[0] - 5.5).abs() < 1e-6, "channel width: {}", v[0]);
    assert!((v[1] - (12.0 + r)).abs() < 1e-6, "slot length: {}", v[1]);
    assert!((v[2] + r).abs() < 1e-6, "hex bottom: {}", v[2]);
}

#[test]
fn slotted_pocket_cuts_open_to_the_edge() {
    // The motivating use: a slide-in nut slot in a quad arm. Cutting the
    // slotted tool must remove more material than the plain pocket.
    let plain =
        eval_num("box(20, 20, 8).cut(nut_pocket(:m3, depth: 3).translate(10, 10, 5)).volume");
    let slotted = eval_num(
        "box(20, 20, 8).cut(nut_pocket(:m3, depth: 3, slot: 15).translate(10, 10, 5)).volume",
    );
    assert!(
        slotted < plain - 50.0,
        "slot should open a channel: plain {plain}, slotted {slotted}"
    );
}

#[test]
fn slot_length_must_be_positive() {
    let err = eval_err("nut_pocket(:m3, depth: 3, slot: -2)");
    assert!(err.contains("slot length"), "unexpected error: {err}");
}

// ---------------------------------------------------------------------------
// standoff_pocket
// ---------------------------------------------------------------------------

#[test]
fn standoff_pocket_matches_the_nut_pocket() {
    // Hex standoffs share the nut's across-flats, so the tools coincide.
    let a = eval_num("standoff_pocket(:m3, depth: 6).volume");
    let b = eval_num("nut_pocket(:m3, depth: 6).volume");
    assert!((a - b).abs() < 1e-9, "expected identical tools: {a} vs {b}");
}

#[test]
fn standoff_pocket_keeps_a_plate_workflow() {
    // Booleans hand back a compound wrapping the solid; the recess volume
    // is what proves the cut landed.
    let v =
        eval_num("box(40, 40, 3).cut(standoff_pocket(:m3, depth: 2).translate(5, 5, 1)).volume");
    let recess = 3.0_f64.sqrt() / 2.0 * 5.9 * 5.9 * 2.0;
    assert!(
        (v - (4800.0 - recess)).abs() < 1.0,
        "expected ~{}, got {v}",
        4800.0 - recess
    );
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[test]
fn unsupported_style_is_rejected() {
    let err = eval_err("nut_pocket(:m3, depth: 3, style: :round)");
    assert!(
        err.contains("unsupported style") && err.contains(":hex or :square"),
        "unexpected error: {err}"
    );
}

#[test]
fn unsupported_size_is_rejected() {
    let err = eval_err("nut_pocket(:m99, depth: 3)");
    assert!(err.contains("unsupported size"), "unexpected error: {err}");
}

#[test]
fn negative_clearance_is_rejected() {
    let err = eval_err("nut_pocket(:m3, depth: 3, clearance: -0.1)");
    assert!(err.contains("clearance"), "unexpected error: {err}");
}

#[test]
fn depth_must_be_positive() {
    let err = eval_err("nut_pocket(:m3, depth: 0)");
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

#[test]
fn nut_body_still_builds_after_the_table_refactor() {
    // nut() now reads its across-flats through the shared table; make sure
    // the body generator is unchanged (it returns a compound — a hex solid
    // with its clearance hole cut — so measure, don't type-check).
    let dx = eval_num("nut(:m3, thickness: 2.4).bounding_box[:dx]");
    assert!((dx - 5.5).abs() < 1e-6, "M3 nut across flats: {dx}");
    let v = eval_num("nut(:m3, thickness: 2.4).volume");
    assert!(v > 40.0 && v < 70.0, "M3 nut body volume out of range: {v}");
}
