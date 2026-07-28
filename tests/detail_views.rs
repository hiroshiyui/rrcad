// Detail views — `export(..., detail: { at:, radius:, scale: })`.
//
// A detail view is a magnified close-up of one circular region of a drawing:
// the region is clipped out of the projection, scaled up, and drawn beside the
// parent view inside a border circle, with the parent gaining a thin circle
// marking what was magnified.
//
// The tests parse the emitted SVG and DXF and check real geometry — where the
// marker lands, what the magnification did to a known feature's radius, and
// that the clip cuts exactly on the border rather than at the nearest vertex.
// The failure mode that gets an explicit assertion is geometry escaping the
// region: a clip that stopped at the nearest vertex, or joined two separate
// runs across a gap, still produces a well-formed file that simply shows edges
// the part does not have.

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
            .join(format!("target/detailview_{tag}"));
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

/// A 60x40x4 plate with a single Ø4 hole at (50, 30).
///
/// One isolated feature at a known position is what a detail view is for, and
/// it makes every magnified measurement checkable by hand.
const PLATE: &str = "plate = box(60, 40, 4).cut(cylinder(2, 10).translate(50, 30, -1))";

// ---------------------------------------------------------------------------
// SVG parsing helpers
// ---------------------------------------------------------------------------

/// One `<circle>`: centre and radius, in SVG coordinates (Y down).
type Circle = (f64, f64, f64);

fn svg_circles(svg: &str) -> Vec<Circle> {
    let mut out = Vec::new();
    for chunk in svg.split("<circle ").skip(1) {
        let head = &chunk[..chunk.find("/>").expect("unterminated <circle>")];
        let attr = |name: &str| -> f64 {
            let at = head
                .find(&format!("{name}=\""))
                .unwrap_or_else(|| panic!("{name} missing from <circle {head}>"));
            let rest = &head[at + name.len() + 2..];
            rest[..rest.find('"').expect("unterminated attribute")]
                .parse()
                .expect("numeric circle attribute")
        };
        out.push((attr("cx"), attr("cy"), attr("r")));
    }
    out
}

fn svg_polylines(svg: &str) -> Vec<Vec<(f64, f64)>> {
    let mut out = Vec::new();
    for chunk in svg.split("<polyline points=\"").skip(1) {
        let pts = &chunk[..chunk.find('"').expect("unterminated points")];
        out.push(
            pts.split_whitespace()
                .map(|t| {
                    let (x, y) = t.split_once(',').expect("malformed point");
                    (x.parse().expect("numeric x"), y.parse().expect("numeric y"))
                })
                .collect(),
        );
    }
    out
}

/// The parent's region marker and the detail view's border circle.
///
/// The parent view is written before the detail view, so the first two circles
/// in the document are the marker and the border in that order.
fn marker_and_border(svg: &str) -> (Circle, Circle) {
    let circles = svg_circles(svg);
    assert!(
        circles.len() >= 2,
        "expected a marker circle and a border circle, got {circles:?}"
    );
    (circles[0], circles[1])
}

/// SVG coordinates are written to four decimals, so a point the exporter placed
/// exactly on the border circle reads back a fraction off it.
const SVG_EPS: f64 = 2e-3;

/// The polylines belonging to the detail view: those lying wholly inside its
/// border circle. Parent-view geometry sits far to the left and never
/// qualifies, so this cleanly separates the two views.
fn detail_polylines(svg: &str) -> Vec<Vec<(f64, f64)>> {
    let (_, (bx, by, br)) = marker_and_border(svg);
    svg_polylines(svg)
        .into_iter()
        .filter(|pl| {
            pl.iter()
                .all(|(x, y)| ((x - bx).powi(2) + (y - by).powi(2)).sqrt() <= br + SVG_EPS)
        })
        .collect()
}

/// Every detail-view point as its radius from the border centre.
fn detail_radii(svg: &str) -> Vec<f64> {
    let (_, (bx, by, _)) = marker_and_border(svg);
    detail_polylines(svg)
        .into_iter()
        .flatten()
        .map(|(x, y)| ((x - bx).powi(2) + (y - by).powi(2)).sqrt())
        .collect()
}

// ---------------------------------------------------------------------------
// DXF parsing helpers
// ---------------------------------------------------------------------------

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
            if matches!(kind.as_str(), "LINE" | "CIRCLE" | "ARC" | "TEXT") {
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

fn layer(entity: &Entity) -> &str {
    entity.1.get("8").map(String::as_str).unwrap_or("")
}

// ---------------------------------------------------------------------------
// The region marker on the parent view
// ---------------------------------------------------------------------------

#[test]
fn the_marker_circle_lands_on_the_stated_region() {
    let ws = Workspace::new("marker");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let (marker, _) = marker_and_border(&svg);
    // SVG is Y-down, so a model Y of 30 is written as -30.
    assert!(
        (marker.0 - 50.0).abs() < 1e-6 && (marker.1 + 30.0).abs() < 1e-6,
        "marker should sit at the stated centre, got {marker:?}"
    );
    assert!(
        (marker.2 - 6.0).abs() < 1e-6,
        "marker radius should be the stated radius, got {}",
        marker.2
    );
}

#[test]
fn the_region_is_stated_in_the_views_own_axes() {
    // In the front view the drawing plane is X/Z, so the hole through Z is a
    // slot spanning the plate's full 4 mm thickness. Asking for a region at
    // Z = 2 must find it; the same numbers as the top view would not.
    let ws = Workspace::new("axes");
    let out = ws.path("front.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :front, \
             detail: {{ at: [50, 2], radius: 5, scale: 3 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let (marker, _) = marker_and_border(&svg);
    assert!(
        (marker.0 - 50.0).abs() < 1e-6 && (marker.1 + 2.0).abs() < 1e-6,
        "front-view marker should use X/Z, got {marker:?}"
    );
}

// ---------------------------------------------------------------------------
// The magnified view
// ---------------------------------------------------------------------------

#[test]
fn the_border_circle_is_the_region_scaled_up() {
    let ws = Workspace::new("border");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let (marker, border) = marker_and_border(&svg);
    assert!(
        (border.2 - 24.0).abs() < 1e-6,
        "border radius should be radius x scale = 24, got {}",
        border.2
    );
    // Placed clear of the parent view, to its right.
    assert!(
        border.0 - border.2 > marker.0 + marker.2,
        "detail view should not overlap the parent: border {border:?}, marker {marker:?}"
    );
}

#[test]
fn a_known_feature_is_magnified_by_exactly_the_scale() {
    // The Ø4 hole sits at the region centre, so at 4:1 its projected radius of
    // 2 mm must become 8 mm — measured from the border circle's own centre.
    let ws = Workspace::new("magnify");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let radii = detail_radii(&svg);
    assert!(!radii.is_empty(), "detail view has no geometry");
    let max = radii.iter().cloned().fold(f64::MIN, f64::max);
    // HLR approximates the circle with chords, so points fall just inside 8.
    assert!(
        (max - 8.0).abs() < 0.1,
        "magnified hole should reach r = 8, got {max}"
    );
    assert!(
        radii.iter().all(|r| *r > 7.8),
        "only the hole is inside this region, so every point should be near r = 8"
    );
}

#[test]
fn magnification_scales_with_the_requested_ratio() {
    let ws = Workspace::new("ratio");
    let mut measured = Vec::new();
    for scale in [2, 5] {
        let out = ws.path(&format!("detail{scale}.svg"));
        ws.run(
            &format!(
                "{PLATE}\nplate.export(OUT, view: :top, \
                 detail: {{ at: [50, 30], radius: 6, scale: {scale} }})"
            ),
            &out,
        )
        .expect("export");
        let svg = std::fs::read_to_string(&out).expect("read svg");
        let max = detail_radii(&svg).into_iter().fold(f64::MIN, f64::max);
        measured.push(max);
    }
    // The same Ø4 hole: 2 mm radius at 2:1 and at 5:1.
    assert!(
        (measured[0] - 4.0).abs() < 0.05 && (measured[1] - 10.0).abs() < 0.15,
        "expected radii near 4 and 10, got {measured:?}"
    );
}

// ---------------------------------------------------------------------------
// Clipping
// ---------------------------------------------------------------------------

#[test]
fn geometry_is_cut_exactly_on_the_border_not_at_the_nearest_vertex() {
    // A region straddling the plate's right-hand edge. The edge is one long
    // polyline of coarse points; clipping at the nearest vertex would stop
    // short of the border by up to a full point spacing.
    let ws = Workspace::new("clip");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [57, 30], radius: 6, scale: 3 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let (_, border) = marker_and_border(&svg);
    let on_border = detail_radii(&svg)
        .into_iter()
        .filter(|r| (r - border.2).abs() < SVG_EPS)
        .count();
    assert!(
        on_border >= 2,
        "the crossed edge should terminate exactly on the border circle \
         (both ends), found {on_border} such points"
    );
}

#[test]
fn no_polyline_straddles_the_border() {
    // Every emitted polyline belongs either to the parent view or to the detail
    // view. One with points on both sides of the border circle would mean the
    // clip let geometry escape the region it was supposed to bound.
    let ws = Workspace::new("leak");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [57, 30], radius: 6, scale: 3 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let (_, (bx, by, br)) = marker_and_border(&svg);
    for pl in svg_polylines(&svg) {
        let radii: Vec<f64> = pl
            .iter()
            .map(|(x, y)| ((x - bx).powi(2) + (y - by).powi(2)).sqrt())
            .collect();
        let inside = radii.iter().filter(|r| **r < br - SVG_EPS).count();
        let outside = radii.iter().filter(|r| **r > br + SVG_EPS).count();
        assert!(
            inside == 0 || outside == 0,
            "a polyline straddles the border circle: {inside} points in, {outside} out"
        );
    }
}

#[test]
fn each_clipped_run_is_its_own_polyline() {
    // A region straddling the plate's top-left corner crosses two outer edges.
    // Each contributes one run, and they must stay separate: joining them would
    // draw an edge across the corner that the part does not have.
    let ws = Workspace::new("runs");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [0, 40], radius: 8, scale: 2 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    let pls = detail_polylines(&svg);
    assert!(
        pls.len() >= 2,
        "expected one polyline per crossed edge, got {}",
        pls.len()
    );
    // Each run enters and leaves the region exactly once, so it carries two
    // border points — its own two ends — and no more.
    let (_, (bx, by, br)) = marker_and_border(&svg);
    for pl in &pls {
        let on_border = pl
            .iter()
            .filter(|(x, y)| (((x - bx).powi(2) + (y - by).powi(2)).sqrt() - br).abs() < SVG_EPS)
            .count();
        assert!(
            on_border <= 2,
            "a run touching the border {on_border} times means separate pieces were joined"
        );
    }
}

// ---------------------------------------------------------------------------
// Captions and labels
// ---------------------------------------------------------------------------

#[test]
fn the_caption_states_the_label_and_the_ratio() {
    let ws = Workspace::new("caption");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    assert!(
        svg.contains("DETAIL A (4:1)"),
        "missing caption in:\n{svg:.600}"
    );
}

#[test]
fn a_custom_label_is_used_for_both_the_marker_and_the_caption() {
    let ws = Workspace::new("label");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 2, label: \"C\" }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    assert!(svg.contains("DETAIL C (2:1)"), "missing custom caption");
    assert!(
        svg.contains(">C</text>"),
        "the marker should carry the same label"
    );
}

#[test]
fn a_fractional_magnification_reads_as_a_decimal_ratio() {
    let ws = Workspace::new("fractional");
    let out = ws.path("detail.svg");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 2.5 }})"
        ),
        &out,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    assert!(
        svg.contains("DETAIL A (2.5:1)"),
        "expected a decimal ratio in:\n{svg:.600}"
    );
}

// ---------------------------------------------------------------------------
// Canvas
// ---------------------------------------------------------------------------

#[test]
fn the_canvas_grows_to_hold_the_detail_view() {
    let ws = Workspace::new("canvas");
    let plain = ws.path("plain.svg");
    let with_detail = ws.path("detail.svg");
    ws.run(&format!("{PLATE}\nplate.export(OUT, view: :top)"), &plain)
        .expect("export");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &with_detail,
    )
    .expect("export");

    let width = |p: &Path| -> f64 {
        let svg = std::fs::read_to_string(p).expect("read svg");
        let at = svg.find(" width=\"").expect("width attribute") + 8;
        let rest = &svg[at..];
        rest[..rest.find('"').unwrap()].parse().expect("numeric")
    };
    assert!(
        width(&with_detail) > width(&plain) + 40.0,
        "the detail view should widen the canvas: {} vs {}",
        width(&with_detail),
        width(&plain)
    );
}

#[test]
fn no_detail_annotation_appears_when_none_is_requested() {
    let ws = Workspace::new("absent");
    let out = ws.path("plain.svg");
    ws.run(&format!("{PLATE}\nplate.export(OUT, view: :top)"), &out)
        .expect("export");
    let svg = std::fs::read_to_string(&out).expect("read svg");
    assert!(!svg.contains("class=\"detail\""), "unexpected detail group");
    assert!(!svg.contains("DETAIL"), "unexpected detail caption");
}

// ---------------------------------------------------------------------------
// DXF
// ---------------------------------------------------------------------------

#[test]
fn dxf_puts_the_bubble_on_its_own_layer() {
    let ws = Workspace::new("dxflayer");
    let out = ws.path("detail.dxf");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &out,
    )
    .expect("export");
    let entities = parse_dxf(&out);
    let detail_circles: Vec<&Entity> = entities
        .iter()
        .filter(|e| e.0 == "CIRCLE" && layer(e) == "DETAIL")
        .collect();
    assert_eq!(
        detail_circles.len(),
        2,
        "expected the marker and the border circle on layer DETAIL"
    );
    let mut radii: Vec<f64> = detail_circles.iter().map(|e| num(e, "40")).collect();
    radii.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        (radii[0] - 6.0).abs() < 1e-6 && (radii[1] - 24.0).abs() < 1e-6,
        "expected marker r = 6 and border r = 24, got {radii:?}"
    );
    let captions: Vec<&Entity> = entities
        .iter()
        .filter(|e| e.0 == "TEXT" && layer(e) == "DETAIL")
        .collect();
    assert!(
        captions
            .iter()
            .any(|e| e.1.get("1").map(String::as_str) == Some("DETAIL A (4:1)")),
        "missing DXF caption, got {:?}",
        captions.iter().map(|e| e.1.get("1")).collect::<Vec<_>>()
    );
}

#[test]
fn dxf_geometry_is_magnified_like_the_svg() {
    let ws = Workspace::new("dxfgeom");
    let out = ws.path("detail.dxf");
    ws.run(
        &format!(
            "{PLATE}\nplate.export(OUT, view: :top, \
             detail: {{ at: [50, 30], radius: 6, scale: 4 }})"
        ),
        &out,
    )
    .expect("export");
    let entities = parse_dxf(&out);
    let border = entities
        .iter()
        .filter(|e| e.0 == "CIRCLE" && layer(e) == "DETAIL")
        .max_by(|a, b| num(a, "40").partial_cmp(&num(b, "40")).unwrap())
        .expect("border circle");
    let (bx, by) = (num(border, "10"), num(border, "20"));
    // Magnified hole: every line endpoint near the border centre is at r ≈ 8.
    let mut max = f64::MIN;
    for e in entities.iter().filter(|e| e.0 == "LINE") {
        for (cx, cy) in [("10", "20"), ("11", "21")] {
            let r = ((num(e, cx) - bx).powi(2) + (num(e, cy) - by).powi(2)).sqrt();
            if r <= 24.0 + 1e-6 {
                max = max.max(r);
            }
        }
    }
    assert!(
        (max - 8.0).abs() < 0.1,
        "magnified hole should reach r = 8 in DXF too, got {max}"
    );
}

// ---------------------------------------------------------------------------
// Rejections
// ---------------------------------------------------------------------------

#[test]
fn an_empty_region_is_rejected_rather_than_drawn_blank() {
    let ws = Workspace::new("empty");
    let out = ws.path("detail.svg");
    let err = ws
        .run(
            &format!(
                "{PLATE}\nplate.export(OUT, view: :top, \
                 detail: {{ at: [20, 20], radius: 3, scale: 4 }})"
            ),
            &out,
        )
        .expect_err("an empty region must fail");
    assert!(
        err.contains("contains no drawing geometry"),
        "unhelpful error: {err}"
    );
}

#[test]
fn a_detail_on_the_three_view_sheet_is_rejected() {
    let ws = Workspace::new("sheet");
    let out = ws.path("sheet.svg");
    let err = ws
        .run(
            &format!(
                "{PLATE}\nplate.export(OUT, view: :sheet, \
                 detail: {{ at: [50, 30], radius: 6 }})"
            ),
            &out,
        )
        .expect_err("a sheet detail must fail");
    assert!(
        err.contains("need a single view"),
        "expected a sheet-mode refusal, got: {err}"
    );
}

#[test]
fn a_detail_without_a_radius_is_rejected() {
    let ws = Workspace::new("noradius");
    let out = ws.path("detail.svg");
    let err = ws
        .run(
            &format!("{PLATE}\nplate.export(OUT, view: :top, detail: {{ at: [50, 30] }})"),
            &out,
        )
        .expect_err("a detail without a radius must fail");
    assert!(err.contains("radius"), "unhelpful error: {err}");
}

#[test]
fn a_malformed_region_centre_is_rejected() {
    let ws = Workspace::new("badat");
    let out = ws.path("detail.svg");
    for at in ["[50]", "[50, 30, 4]", "50"] {
        let err = ws
            .run(
                &format!(
                    "{PLATE}\nplate.export(OUT, view: :top, \
                     detail: {{ at: {at}, radius: 6 }})"
                ),
                &out,
            )
            .unwrap_err();
        assert!(
            err.contains("2-element"),
            "at: {at} should be refused, got: {err}"
        );
    }
}

#[test]
fn a_non_positive_radius_or_scale_is_rejected() {
    let ws = Workspace::new("nonpositive");
    let out = ws.path("detail.svg");
    for (opts, word) in [
        ("radius: 0, scale: 4", "radius"),
        ("radius: -2, scale: 4", "radius"),
        ("radius: 6, scale: 0", "scale"),
    ] {
        let err = ws
            .run(
                &format!(
                    "{PLATE}\nplate.export(OUT, view: :top, \
                     detail: {{ at: [50, 30], {opts} }})"
                ),
                &out,
            )
            .unwrap_err();
        assert!(
            err.contains(word) && err.contains("positive"),
            "expected a {word} refusal, got: {err}"
        );
    }
}

#[test]
fn a_detail_that_is_not_a_hash_is_rejected() {
    let ws = Workspace::new("nothash");
    let out = ws.path("detail.svg");
    let err = ws
        .run(
            &format!("{PLATE}\nplate.export(OUT, view: :top, detail: 6)"),
            &out,
        )
        .expect_err("a bare detail value must fail");
    assert!(err.contains("Hash"), "unhelpful error: {err}");
}
