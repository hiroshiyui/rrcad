// Flat cut-file export — `Shape#export_outline`.
//
// Distinct from `export("part.dxf")`, which draws an HLR projection of a 3-D
// shape. A cut file carries only the closed loops of one planar face, at 1:1,
// with circular edges as true CIRCLE / ARC entities rather than chord
// approximations — which is what a laser or CNC controller consumes.
//
// The tests parse the emitted DXF and check real geometry: hole positions and
// radii, arc centres and sweeps, and the overall bounding box. A swapped arc
// start/end would still produce a valid-looking file, so sweep direction is
// asserted explicitly.

use rrcad::ruby::vm::MrubyVm;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A throwaway working directory, removed on drop.
///
/// `safe_path` confines exports to the process CWD, so these tests write into
/// the CWD rather than a temp dir, and clean up after themselves.
struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/cutfile_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Run `script`, with `out` substituted for `OUT` as a string literal.
    fn run(&self, script: &str, out: &Path) -> Result<String, String> {
        let literal = format!("{:?}", out.to_string_lossy());
        let mut vm = MrubyVm::new();
        vm.eval(&script.replace("OUT", &literal))
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// One parsed DXF entity: its type and its group-code map.
type Entity = (String, HashMap<String, String>);

/// Parse the ENTITIES of an ASCII DXF into (type, group-code map) pairs.
fn parse_dxf(path: &Path) -> Vec<Entity> {
    let text = std::fs::read_to_string(path).expect("read dxf");
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i] == "0" && i + 1 < lines.len() {
            let kind = lines[i + 1].to_string();
            if matches!(kind.as_str(), "LINE" | "CIRCLE" | "ARC") {
                let mut map = HashMap::new();
                let mut j = i + 2;
                while j + 1 < lines.len() && lines[j] != "0" {
                    map.insert(lines[j].to_string(), lines[j + 1].to_string());
                    j += 2;
                }
                out.push((kind, map));
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn num(entity: &Entity, code: &str) -> f64 {
    entity
        .1
        .get(code)
        .unwrap_or_else(|| panic!("group code {code} missing from {:?}", entity.0))
        .parse()
        .expect("numeric group code")
}

fn of_kind<'a>(entities: &'a [Entity], kind: &str) -> Vec<&'a Entity> {
    entities.iter().filter(|(k, _)| k == kind).collect()
}

/// A 60×40×2 plate with 5 mm rounded corners and four Ø3.4 holes — the shape
/// this feature exists for.
const PLATE: &str = r#"
    plate = box(60, 40, 2).fillet(5, :vertical)
    [[10, 10], [50, 10], [10, 30], [50, 30]].each do |x, y|
      plate = plate.cut(cylinder(1.7, 10).translate(x, y, -1))
    end
    plate.faces(:top).first.export_outline(OUT)
"#;

// ---------------------------------------------------------------------------
// Entity fidelity
// ---------------------------------------------------------------------------

#[test]
fn holes_become_true_circles_not_chord_approximations() {
    // The core promise. A polyline stand-in would show up as many LINEs and
    // no CIRCLEs, and a controller could not cut it as a hole.
    let ws = Workspace::new("circles");
    let out = ws.path("plate.dxf");
    ws.run(PLATE, &out).expect("export should succeed");

    let entities = parse_dxf(&out);
    let circles = of_kind(&entities, "CIRCLE");
    assert_eq!(circles.len(), 4, "expected one CIRCLE per hole");

    let mut found: Vec<(f64, f64, f64)> = circles
        .iter()
        .map(|c| (num(c, "10"), num(c, "20"), num(c, "40")))
        .collect();
    found.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let expected = [
        (10.0, 10.0, 1.7),
        (10.0, 30.0, 1.7),
        (50.0, 10.0, 1.7),
        (50.0, 30.0, 1.7),
    ];
    for (got, want) in found.iter().zip(expected.iter()) {
        assert!(
            (got.0 - want.0).abs() < 1e-6
                && (got.1 - want.1).abs() < 1e-6
                && (got.2 - want.2).abs() < 1e-6,
            "hole at {got:?} should be {want:?}"
        );
    }
}

#[test]
fn rounded_corners_become_arcs_sweeping_the_short_way() {
    // Arc direction is the subtle part: DXF arcs always run CCW from start to
    // end, so a swapped pair describes the 270° complement instead of the 90°
    // corner — a file that still looks valid but cuts a different part.
    let ws = Workspace::new("arcs");
    let out = ws.path("plate.dxf");
    ws.run(PLATE, &out).expect("export should succeed");

    let entities = parse_dxf(&out);
    let arcs = of_kind(&entities, "ARC");
    assert_eq!(arcs.len(), 4, "expected one ARC per rounded corner");

    for arc in &arcs {
        let sweep = (num(arc, "51") - num(arc, "50")).rem_euclid(360.0);
        assert!(
            (sweep - 90.0).abs() < 1e-6,
            "corner arc should sweep 90°, got {sweep}° (270° means start/end are swapped)"
        );
        assert!(
            (num(arc, "40") - 5.0).abs() < 1e-6,
            "corner radius should be 5"
        );
    }

    // Each arc must also sit at the right corner.
    let mut centres: Vec<(f64, f64)> = arcs.iter().map(|a| (num(a, "10"), num(a, "20"))).collect();
    centres.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let expected = [(5.0, 5.0), (5.0, 35.0), (55.0, 5.0), (55.0, 35.0)];
    for (got, want) in centres.iter().zip(expected.iter()) {
        assert!(
            (got.0 - want.0).abs() < 1e-6 && (got.1 - want.1).abs() < 1e-6,
            "arc centre {got:?} should be {want:?}"
        );
    }
}

#[test]
fn straight_edges_become_lines() {
    let ws = Workspace::new("lines");
    let out = ws.path("plate.dxf");
    ws.run(PLATE, &out).expect("export should succeed");
    let entities = parse_dxf(&out);
    assert_eq!(
        of_kind(&entities, "LINE").len(),
        4,
        "a rounded rectangle has four straight runs"
    );
}

#[test]
fn a_plain_rectangle_emits_only_lines() {
    let ws = Workspace::new("rect");
    let out = ws.path("rect.dxf");
    ws.run("box(30, 20, 2).faces(:top).first.export_outline(OUT)", &out)
        .expect("export should succeed");
    let entities = parse_dxf(&out);
    assert_eq!(of_kind(&entities, "LINE").len(), 4);
    assert!(of_kind(&entities, "ARC").is_empty());
    assert!(of_kind(&entities, "CIRCLE").is_empty());
}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

#[test]
fn the_outline_is_shifted_to_the_origin_at_true_size() {
    let ws = Workspace::new("bbox");
    let out = ws.path("plate.dxf");
    ws.run(PLATE, &out).expect("export should succeed");

    let entities = parse_dxf(&out);
    let (mut xmin, mut ymin) = (f64::MAX, f64::MAX);
    let (mut xmax, mut ymax) = (f64::MIN, f64::MIN);
    for e in &entities {
        let mut note = |x: f64, y: f64| {
            xmin = xmin.min(x);
            ymin = ymin.min(y);
            xmax = xmax.max(x);
            ymax = ymax.max(y);
        };
        match e.0.as_str() {
            "LINE" => {
                note(num(e, "10"), num(e, "20"));
                note(num(e, "11"), num(e, "21"));
            }
            _ => {
                let (cx, cy, r) = (num(e, "10"), num(e, "20"), num(e, "40"));
                note(cx - r, cy - r);
                note(cx + r, cy + r);
            }
        }
    }
    assert!(
        xmin.abs() < 1e-6 && ymin.abs() < 1e-6,
        "expected the outline shifted to the origin, got min ({xmin}, {ymin})"
    );
    assert!(
        (xmax - 60.0).abs() < 1e-6 && (ymax - 40.0).abs() < 1e-6,
        "expected 60 × 40 at 1:1, got {xmax} × {ymax}"
    );
}

#[test]
fn a_tilted_face_still_exports_at_true_size() {
    // The outline is taken in the face's own plane, so an arbitrarily oriented
    // face must come out undistorted — a projection onto XY would foreshorten
    // it instead.
    let ws = Workspace::new("tilted");
    let out = ws.path("tilted.dxf");
    ws.run(
        "box(60, 40, 2).rotate(1, 1, 0, 37).rotate(0, 0, 1, 22)
           .faces(:top).first.export_outline(OUT)",
        &out,
    )
    .expect("export should succeed");

    let entities = parse_dxf(&out);
    let xs: Vec<f64> = entities
        .iter()
        .flat_map(|e| vec![num(e, "10"), num(e, "11")])
        .collect();
    let ys: Vec<f64> = entities
        .iter()
        .flat_map(|e| vec![num(e, "20"), num(e, "21")])
        .collect();
    let w =
        xs.iter().cloned().fold(f64::MIN, f64::max) - xs.iter().cloned().fold(f64::MAX, f64::min);
    let h =
        ys.iter().cloned().fold(f64::MIN, f64::max) - ys.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        (w - 60.0).abs() < 1e-4 && (h - 40.0).abs() < 1e-4,
        "tilted face should still measure 60 × 40, got {w} × {h}"
    );
}

// ---------------------------------------------------------------------------
// Layers and file structure
// ---------------------------------------------------------------------------

#[test]
fn holes_and_profile_go_on_separate_layers() {
    // Shops cut inside features before the outside profile, so the two must be
    // distinguishable without geometric analysis.
    let ws = Workspace::new("layers");
    let out = ws.path("plate.dxf");
    ws.run(PLATE, &out).expect("export should succeed");

    let entities = parse_dxf(&out);
    for circle in of_kind(&entities, "CIRCLE") {
        assert_eq!(
            circle.1.get("8").map(String::as_str),
            Some("HOLES"),
            "hole circles belong on the HOLES layer"
        );
    }
    for arc in of_kind(&entities, "ARC") {
        assert_eq!(
            arc.1.get("8").map(String::as_str),
            Some("PROFILE"),
            "outer boundary belongs on the PROFILE layer"
        );
    }
}

#[test]
fn the_dxf_declares_millimetres() {
    let ws = Workspace::new("units");
    let out = ws.path("rect.dxf");
    ws.run("box(10, 10, 2).faces(:top).first.export_outline(OUT)", &out)
        .expect("export should succeed");
    let text = std::fs::read_to_string(&out).expect("read");
    assert!(
        text.contains("$INSUNITS"),
        "a cut file must state its units so the controller need not guess"
    );
}

#[test]
fn svg_output_is_well_formed_and_sized_in_millimetres() {
    let ws = Workspace::new("svg");
    let out = ws.path("plate.svg");
    ws.run(PLATE, &out).expect("export should succeed");
    let text = std::fs::read_to_string(&out).expect("read svg");
    assert!(text.starts_with("<?xml"), "expected an XML declaration");
    assert!(
        text.trim_end().ends_with("</svg>"),
        "expected a closed root"
    );
    assert!(
        text.contains("60.000000mm") && text.contains("40.000000mm"),
        "SVG should be sized in millimetres at 1:1"
    );
    assert_eq!(
        text.matches("<circle").count(),
        4,
        "holes should be real circles in SVG too"
    );
    assert!(
        text.contains("class=\"holes\"") && text.contains("class=\"profile\""),
        "SVG should separate holes from the profile"
    );
}

// ---------------------------------------------------------------------------
// A face that is only a profile, not part of a solid
// ---------------------------------------------------------------------------

#[test]
fn a_sketch_profile_exports_directly() {
    // A lone Face has exactly one face, so it needs no selector.
    let ws = Workspace::new("profile");
    let out = ws.path("profile.dxf");
    ws.run("circle(12).export_outline(OUT)", &out)
        .expect("export should succeed");
    let entities = parse_dxf(&out);
    let circles = of_kind(&entities, "CIRCLE");
    assert_eq!(circles.len(), 1, "a circular profile is one CIRCLE");
    assert!(
        (num(circles[0], "40") - 12.0).abs() < 1e-6,
        "radius should survive at 1:1"
    );
}

#[test]
fn free_form_curves_are_approximated_within_the_deflection() {
    // A spline cannot be an exact DXF primitive, so it is sampled; a tighter
    // deflection must produce a finer approximation.
    let ws = Workspace::new("spline");
    let coarse = ws.path("coarse.dxf");
    let fine = ws.path("fine.dxf");
    let script = "sk = sketch do
                    a = point(:a, 0, 0)
                    b = point(:b, 40, 0)
                    line a, b
                    spline b, a, through: [[20, 18]]
                  end
                  sk.export_outline(OUT, deflection: DEF)";
    ws.run(&script.replace("DEF", "1.0"), &coarse)
        .expect("coarse export");
    ws.run(&script.replace("DEF", "0.01"), &fine)
        .expect("fine export");

    let coarse_n = of_kind(&parse_dxf(&coarse), "LINE").len();
    let fine_n = of_kind(&parse_dxf(&fine), "LINE").len();
    assert!(
        fine_n > coarse_n,
        "a tighter deflection should use more segments ({fine_n} vs {coarse_n})"
    );
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn a_multi_face_shape_asks_which_face_to_cut() {
    let ws = Workspace::new("ambiguous");
    let out = ws.path("x.dxf");
    let err = ws
        .run("box(10, 10, 10).export_outline(OUT)", &out)
        .expect_err("a solid is ambiguous");
    assert!(
        err.contains("more than one face") && err.contains("faces(:top)"),
        "the error should say how to disambiguate: {err}"
    );
}

#[test]
fn a_curved_face_is_rejected() {
    let ws = Workspace::new("curved");
    let out = ws.path("x.dxf");
    let err = ws
        .run(
            "cylinder(5, 10).faces(:side).first.export_outline(OUT)",
            &out,
        )
        .expect_err("a cylindrical face is not flat");
    assert!(
        err.contains("not planar"),
        "the error should name the problem: {err}"
    );
}

#[test]
fn an_unsupported_extension_is_rejected() {
    let ws = Workspace::new("format");
    let out = ws.path("x.stl");
    let err = ws
        .run("box(10, 10, 2).faces(:top).first.export_outline(OUT)", &out)
        .expect_err("stl is not a cut format");
    assert!(
        err.contains(".dxf or .svg"),
        "the error should list the supported formats: {err}"
    );
}

#[test]
fn a_non_positive_deflection_is_rejected() {
    let ws = Workspace::new("deflection");
    let out = ws.path("x.dxf");
    let err = ws
        .run(
            "box(10, 10, 2).faces(:top).first.export_outline(OUT, deflection: 0)",
            &out,
        )
        .expect_err("zero deflection is meaningless");
    assert!(err.contains("deflection must be > 0"), "unexpected: {err}");
}

#[test]
fn export_outline_is_confined_to_the_working_directory() {
    // Shares safe_path with every other export, so this adds no new way to
    // write outside the CWD.
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("box(10, 10, 2).faces(:top).first.export_outline(\"/etc/rrcad_escape.dxf\")")
        .expect_err("writing outside the CWD must be refused");
    assert!(
        !Path::new("/etc/rrcad_escape.dxf").exists(),
        "the file must not have been created"
    );
    assert!(!err.is_empty(), "expected an explanatory error");
}
