// Ordinate dimensioning — `export(..., ordinate: true)`.
//
// `dimensions: true` labels a drawing's overall width and height. That says
// nothing about where the holes are, which is the part of a plate drawing a
// shop actually needs. `ordinate: true` measures every located feature from a
// single datum corner, which is how a plate full of holes is dimensioned in
// practice: a chain of dimensions between neighbouring holes would accumulate
// tolerance and be unreadable once there are more than a few.
//
// The tests assert the measured values rather than the presence of markup —
// an ordinate that renders beautifully and states the wrong distance is the
// failure that matters. Scale invariance gets its own test because the labels
// and the geometry they annotate live in different coordinate systems.

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
            .join(format!("target/ordinate_{tag}"));
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

    /// Run `script` and return the exported SVG.
    fn svg(&self, script: &str, name: &str) -> String {
        let out = self.path(name);
        self.run(script, &out).expect("export");
        std::fs::read_to_string(&out).expect("read svg")
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// An 80x50x6 plate with four Ø5.2 holes on a 56 x 26 pattern, inset 12 mm
/// from each edge — a shape whose every ordinate can be checked by hand.
const PLATE: &str = "
    plate = box(80, 50, 6)
    [[12, 12], [68, 12], [12, 38], [68, 38]].each do |x, y|
      plate = plate.cut(cylinder(2.6, 12).translate(x, y, -1))
    end
";

// ---------------------------------------------------------------------------
// SVG parsing helpers
// ---------------------------------------------------------------------------

/// The contents of the `ordinates` group, or `None` when it was not emitted.
fn ordinate_group(svg: &str) -> Option<&str> {
    let at = svg.find("class=\"ordinates\"")?;
    let rest = &svg[at..];
    Some(&rest[..rest.find("</g>").expect("unterminated ordinates group")])
}

/// One witness line: (x1, y1, x2, y2) in SVG coordinates.
fn lines(group: &str) -> Vec<(f64, f64, f64, f64)> {
    let mut out = Vec::new();
    for chunk in group.split("<line ").skip(1) {
        let head = &chunk[..chunk.find("/>").expect("unterminated <line>")];
        let attr = |name: &str| -> f64 {
            let at = head
                .find(&format!("{name}=\""))
                .unwrap_or_else(|| panic!("{name} missing from <line {head}>"));
            let rest = &head[at + name.len() + 2..];
            rest[..rest.find('"').expect("unterminated attribute")]
                .parse()
                .expect("numeric line attribute")
        };
        out.push((attr("x1"), attr("y1"), attr("x2"), attr("y2")));
    }
    out
}

/// Every ordinate label, as written.
fn labels(group: &str) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in group.split("<text ").skip(1) {
        let body_at = chunk.find('>').expect("unterminated <text>") + 1;
        let body = &chunk[body_at..];
        out.push(body[..body.find('<').expect("unterminated text body")].to_string());
    }
    out
}

/// Ordinate labels as sorted numbers, for order-independent comparison.
fn label_values(group: &str) -> Vec<f64> {
    let mut v: Vec<f64> = labels(group)
        .iter()
        .map(|l| l.parse().expect("numeric ordinate label"))
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

// ---------------------------------------------------------------------------
// DXF parsing helpers
// ---------------------------------------------------------------------------

type Entity = (String, HashMap<String, String>);

fn parse_dxf(path: &Path) -> Vec<Entity> {
    let text = std::fs::read_to_string(path).expect("read dxf");
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i] == "0" && i + 1 < lines.len() {
            let kind = lines[i + 1].to_string();
            if matches!(kind.as_str(), "LINE" | "TEXT") {
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

fn on_ordinate_layer(entities: &[Entity], kind: &str) -> Vec<Entity> {
    entities
        .iter()
        .filter(|e| e.0 == kind && e.1.get("8").map(String::as_str) == Some("ORDINATE"))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// What gets measured
// ---------------------------------------------------------------------------

#[test]
fn every_hole_centre_gets_an_ordinate_on_both_axes() {
    let ws = Workspace::new("holes");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        "ord.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    // Holes at x = 12, 68 and y = 12, 38 — four distinct ordinates.
    assert_eq!(label_values(group), vec![12.0, 12.0, 38.0, 68.0]);
}

#[test]
fn features_sharing_a_coordinate_collapse_to_one_ordinate() {
    // Two holes in a row at the same Y: one Y ordinate, two X ordinates.
    let ws = Workspace::new("collapse");
    let svg = ws.svg(
        "plate = box(80, 50, 6)
         [[12, 25], [68, 25]].each do |x, y|
           plate = plate.cut(cylinder(2.6, 12).translate(x, y, -1))
         end
         plate.export(OUT, view: :top, ordinate: true)",
        "ord.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    assert_eq!(
        label_values(group),
        vec![12.0, 25.0, 68.0],
        "a shared Y should be dimensioned once, not once per hole"
    );
}

#[test]
fn corner_fillets_are_features_too() {
    // Fillet arcs are axis-aligned cylindrical faces, so they locate like holes
    // — the same set `center_marks:` and `callouts:` already act on. Their
    // ordinates give the corner radii's centres.
    let ws = Workspace::new("fillets");
    let svg = ws.svg(
        "plate = box(80, 50, 6).fillet(4, :vertical)
         plate.export(OUT, view: :top, ordinate: true)",
        "ord.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    assert_eq!(
        label_values(group),
        vec![4.0, 4.0, 46.0, 76.0],
        "expected the four fillet centres at 4 mm from each edge"
    );
}

#[test]
fn a_part_with_no_located_features_still_exports() {
    let ws = Workspace::new("nofeatures");
    let svg = ws.svg(
        "box(30, 20, 10).export(OUT, view: :top, ordinate: true)",
        "ord.svg",
    );
    assert!(
        ordinate_group(&svg).is_none(),
        "a plain box has nothing to dimension, so no group should be emitted"
    );
    assert!(
        svg.contains("<polyline"),
        "the drawing itself is still there"
    );
}

// ---------------------------------------------------------------------------
// What the numbers mean
// ---------------------------------------------------------------------------

#[test]
fn ordinates_are_measured_from_the_parts_own_corner() {
    // The datum is the lower-left of the projected geometry, not the model
    // origin. A part modelled far from the origin must still read 12 mm.
    let ws = Workspace::new("datum");
    let svg = ws.svg(
        "plate = box(80, 50, 6)
         [[12, 12]].each do |x, y|
           plate = plate.cut(cylinder(2.6, 12).translate(x, y, -1))
         end
         plate.translate(500, 300, 0).export(OUT, view: :top, ordinate: true)",
        "ord.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    assert_eq!(
        label_values(group),
        vec![12.0, 12.0],
        "ordinates should measure across the part, not from the model origin"
    );
}

#[test]
fn labels_stay_in_model_units_when_the_drawing_is_scaled() {
    // The geometry is drawn at 2:1 but the part is still 80 mm wide, so the
    // labels must not double even though the witness lines move.
    let ws = Workspace::new("scale");
    let plain = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        "one.svg",
    );
    let scaled = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, scale: 2, ordinate: true)"),
        "two.svg",
    );
    let (g1, g2) = (
        ordinate_group(&plain).expect("group"),
        ordinate_group(&scaled).expect("group"),
    );
    assert_eq!(
        label_values(g1),
        label_values(g2),
        "ordinate labels state the part's dimensions, not the page's"
    );
    // The witness lines themselves do move: the first X ordinate sits at 12
    // at 1:1 and at 24 at 2:1.
    let x_at = |g: &str| -> Vec<f64> {
        let mut v: Vec<f64> = lines(g)
            .iter()
            .filter(|(x1, _, x2, _)| (x1 - x2).abs() < 1e-9)
            .map(|(x1, _, _, _)| *x1)
            .collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v
    };
    let (a, b) = (x_at(g1), x_at(g2));
    assert!(
        a.len() == b.len() && a.iter().zip(&b).all(|(p, q)| (q - 2.0 * p).abs() < 1e-6),
        "witness lines should follow the drawing scale: {a:?} vs {b:?}"
    );
}

#[test]
fn the_front_view_measures_in_its_own_axes() {
    // A hole drilled along Y is a circular feature to the front view and not to
    // the top one, so the front view locates it in X/Z. The same hole seen from
    // the top is a pair of straight edges and is correctly not dimensioned.
    let block = "block = box(80, 20, 40).cut(
                   cylinder(4, 40).rotate(1, 0, 0, -90).translate(25, -5, 30))";
    let ws = Workspace::new("front");
    let svg = ws.svg(
        &format!("{block}\nblock.export(OUT, view: :front, ordinate: true)"),
        "front.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    assert_eq!(
        label_values(group),
        vec![25.0, 30.0],
        "front view should locate the cross-hole at X = 25, Z = 30"
    );

    let top = ws.svg(
        &format!("{block}\nblock.export(OUT, view: :top, ordinate: true)"),
        "top.svg",
    );
    assert!(
        ordinate_group(&top).is_none(),
        "seen from the top that hole is not a circular feature"
    );
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

#[test]
fn witness_lines_run_from_the_datum_edges_out_to_a_common_baseline() {
    let ws = Workspace::new("baseline");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        "ord.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    // Vertical witness lines all reach the same baseline below the part; SVG is
    // Y-down, so "below" is a positive y.
    let baselines: Vec<f64> = lines(group)
        .iter()
        .filter(|(x1, _, x2, _)| (x1 - x2).abs() < 1e-9)
        .map(|(_, _, _, y2)| *y2)
        .filter(|y| *y > 1.0)
        .collect();
    assert!(baselines.len() >= 2, "expected several X ordinates");
    assert!(
        baselines.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-9),
        "X ordinates should share one baseline: {baselines:?}"
    );
}

#[test]
fn the_datum_corner_is_marked() {
    let ws = Workspace::new("cross");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        "ord.svg",
    );
    let group = ordinate_group(&svg).expect("ordinates group");
    // A short cross centred on (0, 0): the plate's lower-left corner.
    let arms = lines(group)
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            x1.abs() <= 2.0 + 1e-9
                && x2.abs() <= 2.0 + 1e-9
                && y1.abs() <= 2.0 + 1e-9
                && y2.abs() <= 2.0 + 1e-9
        })
        .count();
    assert_eq!(arms, 2, "expected a two-armed datum cross at the corner");
}

#[test]
fn ordinates_compose_with_the_overall_dimensions() {
    let ws = Workspace::new("compose");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, dimensions: true, ordinate: true)"),
        "ord.svg",
    );
    assert!(svg.contains("class=\"dimensions\""), "overall dimensions");
    let group = ordinate_group(&svg).expect("ordinates group");
    // The ordinate baseline sits outside the overall dimension line (8 mm out),
    // so the two never overlap.
    let baseline = lines(group)
        .iter()
        .filter(|(x1, _, x2, _)| (x1 - x2).abs() < 1e-9)
        .map(|(_, _, _, y2)| *y2)
        .fold(f64::MIN, f64::max);
    assert!(
        baseline > 8.0,
        "ordinate baseline should clear the overall dimension line, at {baseline}"
    );
}

#[test]
fn the_canvas_grows_to_hold_the_ordinates() {
    let ws = Workspace::new("canvas");
    let plain = ws.svg(&format!("{PLATE}\nplate.export(OUT, view: :top)"), "a.svg");
    let with_ord = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        "b.svg",
    );
    let width = |svg: &str| -> f64 {
        let at = svg.find(" width=\"").expect("width") + 8;
        let rest = &svg[at..];
        rest[..rest.find('"').unwrap()].parse().expect("numeric")
    };
    assert!(
        width(&with_ord) > width(&plain) + 20.0,
        "expected room for the left-hand ordinates: {} vs {}",
        width(&with_ord),
        width(&plain)
    );
}

#[test]
fn locating_features_for_ordinates_does_not_draw_centre_marks() {
    // Feature centres are collected for both options, but asking for ordinates
    // alone must not start drawing crosshairs on the part.
    let ws = Workspace::new("nomarks");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        "ord.svg",
    );
    assert!(ordinate_group(&svg).is_some());
    assert!(
        !svg.contains("class=\"center-marks\""),
        "centre marks were not requested"
    );
}

#[test]
fn no_ordinates_appear_when_none_are_requested() {
    let ws = Workspace::new("absent");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :top, dimensions: true)"),
        "ord.svg",
    );
    assert!(ordinate_group(&svg).is_none(), "unexpected ordinates group");
}

#[test]
fn a_sheet_dimensions_each_view_from_its_own_corner() {
    let ws = Workspace::new("sheet");
    let svg = ws.svg(
        &format!("{PLATE}\nplate.export(OUT, view: :sheet, ordinate: true)"),
        "ord.svg",
    );
    // One group per view that has features; the top view certainly does.
    assert!(
        svg.matches("ordinates\"").count() >= 1,
        "expected ordinates on the sheet"
    );
    assert!(
        svg.contains("view-top ordinates\""),
        "the top view's ordinates should be tagged with their view"
    );
}

// ---------------------------------------------------------------------------
// DXF
// ---------------------------------------------------------------------------

#[test]
fn dxf_writes_ordinates_on_their_own_layer() {
    let ws = Workspace::new("dxflayer");
    let out = ws.path("ord.dxf");
    ws.run(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        &out,
    )
    .expect("export");
    let entities = parse_dxf(&out);
    let texts = on_ordinate_layer(&entities, "TEXT");
    let mut values: Vec<f64> = texts
        .iter()
        .map(|e| e.1["1"].parse().expect("numeric label"))
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(values, vec![12.0, 12.0, 38.0, 68.0]);
    // Two witness lines per ordinate label, plus the two-armed datum cross.
    assert_eq!(
        on_ordinate_layer(&entities, "LINE").len(),
        texts.len() + 2,
        "expected one witness line per ordinate plus the datum cross"
    );
}

#[test]
fn dxf_labels_are_right_aligned_so_they_grow_away_from_the_drawing() {
    // A rotated left-aligned label would run back over the geometry. Right
    // alignment (group code 72 = 2) requires the 11/21 alignment point too, and
    // a viewer ignores the justification without it.
    let ws = Workspace::new("dxfalign");
    let out = ws.path("ord.dxf");
    ws.run(
        &format!("{PLATE}\nplate.export(OUT, view: :top, ordinate: true)"),
        &out,
    )
    .expect("export");
    for text in on_ordinate_layer(&parse_dxf(&out), "TEXT") {
        assert_eq!(text.1.get("72").map(String::as_str), Some("2"));
        assert!(
            text.1.contains_key("11") && text.1.contains_key("21"),
            "right-aligned TEXT needs its second alignment point"
        );
        let rotation: f64 = text.1["50"].parse().expect("numeric rotation");
        assert!(
            rotation == 0.0 || rotation == 90.0,
            "ordinate labels read either across or up, got {rotation}"
        );
    }
}
