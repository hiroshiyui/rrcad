use rrcad::ruby::vm::MrubyVm;

#[test]
fn named_face_selector_survives_transform() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "part = box(10, 20, 30)
part.name_face(:mounting_face, :top)
moved = part.translate(5, 0, 0)
[moved.faces(:mounting_face).length, moved.ref(:mounting_face).shape_type].inspect",
        )
        .unwrap();
    assert!(
        result.contains("[1, :face]"),
        "expected named face lookup after transform, got {result}"
    );
}

#[test]
fn named_edge_selector_survives_transform() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "part = box(10, 20, 30)
part.name_edge(:vertical_edges, :vertical)
moved = part.rotate(0, 0, 1, 90)
[moved.edges(:vertical_edges).length, moved.ref(:vertical_edges).shape_type].inspect",
        )
        .unwrap();
    assert!(
        result.contains("[4, :edge]"),
        "expected named edge lookup after transform, got {result}"
    );
}

#[test]
fn datum_reference_is_returned_by_ref() {
    let mut vm = MrubyVm::new();
    let result = vm
        .eval(
            "part = box(10, 20, 30)
plane = datum_plane(origin: [0, 0, 0], normal: [0, 0, 1], x_dir: [1, 0, 0])
part.datum(:fixture_plane, plane)
part.ref(:fixture_plane).shape_type == :face",
        )
        .unwrap();
    assert_eq!(result.trim(), "true");
}
