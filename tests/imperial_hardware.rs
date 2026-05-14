// Phase 10 — imperial hardware sizes.
//
// Spot-checks that the hardware helpers accept UNC/UNF imperial sizes and
// produce the expected millimetre dimensions per ASME B18 / ANSI B18.3.

use rrcad::ruby::vm::MrubyVm;

#[test]
fn imperial_hardware_sizes() {
    let mut vm = MrubyVm::new();

    // #8 close-fit clearance = 0.170" = 4.32 mm.
    let dx: f64 = vm
        .eval("clearance_hole(:\"8-32\", depth: 10).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 4.32).abs() < 0.1,
        "expected #8 clearance ~4.32 mm, got {dx}"
    );

    // 1/4 close-fit clearance = 0.257" = 6.53 mm.
    let dx: f64 = vm
        .eval("clearance_hole(:\"1/4-20\", depth: 10).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 6.53).abs() < 0.1,
        "expected 1/4 clearance ~6.53 mm, got {dx}"
    );

    // #10-32 tap drill (#21) = 0.159" = 4.04 mm.
    let dx: f64 = vm
        .eval("tap_drill(:\"10-32\", depth: 10).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 4.04).abs() < 0.1,
        "expected #10-32 tap drill ~4.04 mm, got {dx}"
    );

    // #10-24 has a larger pilot than #10-32 (3.80 vs 4.04 — wait, actually
    // 10-24 is COARSER so 75% thread tap is SMALLER).
    let result = vm
        .eval(
            "fine = tap_drill(:\"10-32\", depth: 10).bounding_box[:dx]
             coarse = tap_drill(:\"10-24\", depth: 10).bounding_box[:dx]
             coarse < fine",
        )
        .unwrap();
    assert_eq!(result.trim(), "true");

    let dx: f64 = vm
        .eval("heat_set_insert(:\"6-32\", depth: 6).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!((dx - 4.5).abs() < 0.2, "got {dx}");

    // #8 socket-head OD = 0.270" = 6.86 mm.
    let dx: f64 = vm
        .eval("socket_head_cbore(:\"8-32\", depth: 10, head_depth: 3).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 6.86).abs() < 0.2,
        "expected #8 SHCS head ~6.86 mm, got {dx}"
    );

    // 1/4-20 flat-head OD = 0.507" = 12.88 mm. Imperial flat heads use 82°
    // included angle → 41° half-angle.
    let dx: f64 = vm
        .eval("flat_head_csink(:\"1/4-20\", depth: 10, angle: 41).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 12.88).abs() < 0.3,
        "expected 1/4 FH head ~12.88 mm, got {dx}"
    );

    // #10-32 socket-head OD = 0.312" = 7.92 mm.
    let dx: f64 = vm
        .eval("screw(:\"10-32\", length: 12, style: :socket).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 7.92).abs() < 0.2,
        "expected #10-32 SHCS head ~7.92 mm, got {dx}"
    );

    let dx: f64 = vm
        .eval("washer(:\"8-32\", thickness: 1.2).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 11.1).abs() < 0.3,
        "expected #8 washer OD ~11.1 mm, got {dx}"
    );

    let dz: f64 = vm
        .eval("washer(:\"8-32\", thickness: 1.2).bounding_box[:dz]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dz - 1.2).abs() < 0.1,
        "expected washer thickness ~1.2 mm, got {dz}"
    );

    let dx: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 11.11).abs() < 0.3,
        "expected 1/4-20 nut AF ~11.11 mm, got {dx}"
    );

    let dx: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0, style: :square).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 11.11).abs() < 0.3,
        "expected 1/4-20 square nut width ~11.11 mm, got {dx}"
    );

    let result = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0, style: :jam).shape_type")
        .unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected jam nut solid/compound, got {result}"
    );

    let dx: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0, style: :flange).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 20.0).abs() < 0.8,
        "expected 1/4-20 flange nut width ~20 mm, got {dx}"
    );

    let dz: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0, style: :flange).bounding_box[:dz]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dz - 5.0).abs() < 0.1,
        "expected flange nut thickness ~5.0 mm, got {dz}"
    );

    let dx: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0, style: :nyloc).bounding_box[:dx]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dx - 12.78).abs() < 0.4,
        "expected 1/4-20 nyloc width ~12.78 mm, got {dx}"
    );

    let dz: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0, style: :nyloc).bounding_box[:dz]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dz - 5.0).abs() < 0.1,
        "expected nyloc thickness ~5.0 mm, got {dz}"
    );

    let dz: f64 = vm
        .eval("nut(:\"1/4-20\", thickness: 5.0).bounding_box[:dz]")
        .unwrap()
        .trim()
        .parse()
        .expect("number");
    assert!(
        (dz - 5.0).abs() < 0.1,
        "expected nut thickness ~5.0 mm, got {dz}"
    );

    let result = vm
        .eval(
            "plate = box(40, 40, 8)
             washer = washer(:\"8-32\", thickness: 1.2).translate(20, 20, 3.4)
             plate.cut(washer).volume < plate.volume",
        )
        .unwrap();
    assert_eq!(result.trim(), "true");

    let result = vm.eval("screw(:\"4-40\", length: 6).shape_type").unwrap();
    assert!(
        result == ":solid" || result == ":compound",
        "expected solid/compound, got {result}"
    );

    let err = vm.eval("nut(:\"1/2-13\", thickness: 5)").unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported-size error, got: {err}"
    );

    let err = vm
        .eval("nut(:\"1/4-20\", thickness: 5, style: :acme)")
        .unwrap_err();
    assert!(
        err.contains("unsupported style"),
        "expected unsupported-style error, got: {err}"
    );

    let err = vm
        .eval("clearance_hole(:\"1/2-13\", depth: 10)")
        .unwrap_err();
    assert!(
        err.contains("unsupported size"),
        "expected unsupported-size error, got: {err}"
    );
}
