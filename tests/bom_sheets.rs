// Parts lists and balloon callouts on assembly drawings.
//
// `asm.export("sheet.svg", bom: true, balloons: true)` draws the assembly's
// bill of materials as a table below the drawing, and marks each component with
// a numbered balloon whose leader lands on that part.
//
// Per-component data cannot travel as scalar export options — the row count is
// not known until the assembly is walked — so it crosses the FFI as delimited
// records. Two things therefore get explicit tests: that the delimiters survive
// a component name containing one, and that a balloon's number agrees with the
// table row of the same number. A balloon pointing at the wrong part is a
// drawing that looks perfectly correct and instructs the shop to build the
// wrong thing.

use rrcad::ruby::vm::MrubyVm;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A throwaway working directory, removed on drop.
struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/bomsheet_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn run(&self, script: &str, out: &Path) -> Result<String, String> {
        let literal = format!("{:?}", out.to_string_lossy());
        let mut vm = MrubyVm::new();
        vm.eval(&script.replace("OUT", &literal))
    }

    /// Export the standard panel assembly with `opts` and return the file.
    fn export(&self, name: &str, opts: &str) -> String {
        let path = self.path(name);
        self.run(&format!("{PANEL}\nasm.export(OUT, {opts})"), &path)
            .expect("export");
        std::fs::read_to_string(&path).expect("read export")
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// A panel with one plate, one post, and four identical screws — enough for the
/// quantity rollup to matter and for the balloons to have distinct anchors.
/// The post is deliberately off-centre so its balloon anchor is checkable.
const PANEL: &str = r#"
    plate = box(120, 80, 6)
    screw = cylinder(3, 12)
    post  = box(16, 16, 40)
    asm = assembly("panel") do |a|
      a.place plate, name: :plate, material: "aluminium"
      a.place post.translate(20, 50, 6), name: :post, material: "steel"
      4.times do |i|
        a.place screw.translate(15 + (i % 2) * 90, 15 + (i / 2) * 50, 6),
                name: :"screw_#{i}", component: :m6_screw, material: "stainless"
      end
    end
"#;

// ---------------------------------------------------------------------------
// SVG parsing helpers
// ---------------------------------------------------------------------------

/// The contents of an SVG group by class, or `None` when it was not emitted.
fn group<'a>(svg: &'a str, class: &str) -> Option<&'a str> {
    let at = svg.find(&format!("class=\"{class}\""))?;
    let rest = &svg[at..];
    Some(&rest[..rest.find("</g>").expect("unterminated group")])
}

/// Every `<text>` body in a fragment, in document order.
fn texts(fragment: &str) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in fragment.split("<text ").skip(1) {
        let body_at = chunk.find('>').expect("unterminated <text>") + 1;
        let body = &chunk[body_at..];
        out.push(body[..body.find('<').expect("unterminated text body")].to_string());
    }
    out
}

/// Every `<circle>` as (cx, cy, r).
fn circles(fragment: &str) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for chunk in fragment.split("<circle ").skip(1) {
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

/// Every `<line>` as (x1, y1, x2, y2).
fn lines(fragment: &str) -> Vec<(f64, f64, f64, f64)> {
    let mut out = Vec::new();
    for chunk in fragment.split("<line ").skip(1) {
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

/// The leader anchors: the small filled dots at the end of each leader, as
/// model coordinates (SVG is Y-down, so the sign of y is flipped back).
fn anchors(balloons: &str) -> Vec<(f64, f64)> {
    circles(balloons)
        .into_iter()
        .filter(|(_, _, r)| *r < 1.0)
        .map(|(x, y, _)| (x, -y))
        .collect()
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
            if matches!(kind.as_str(), "LINE" | "CIRCLE" | "TEXT") {
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

fn on_layer<'a>(entities: &'a [Entity], kind: &str, layer: &str) -> Vec<&'a Entity> {
    entities
        .iter()
        .filter(|e| e.0 == kind && e.1.get("8").map(String::as_str) == Some(layer))
        .collect()
}

/// Evaluate a script whose final value is a comma-joined String, and return the
/// parts. The VM renders the final value with `inspect`, so asking Ruby to join
/// first avoids having to unescape a nested Array of Strings.
fn eval_list(script: &str) -> Vec<String> {
    let mut vm = MrubyVm::new();
    let out = vm.eval(script).expect("eval");
    let out = out.trim();
    let inner = out
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or_else(|| panic!("expected a quoted String, got: {out}"));
    if inner.is_empty() {
        return Vec::new();
    }
    inner.split(',').map(str::to_owned).collect()
}

// ---------------------------------------------------------------------------
// The parts list
// ---------------------------------------------------------------------------

#[test]
fn the_table_carries_a_row_per_component_with_the_rolled_up_quantity() {
    let ws = Workspace::new("table");
    let svg = ws.export("panel.svg", "view: :top, bom: true");
    let table = group(&svg, "bom").expect("bom group");
    let cells = texts(table);
    assert_eq!(
        &cells[..5],
        &["Item", "Component", "Qty", "Material", "Mass (g)"],
        "unexpected header: {cells:?}"
    );
    // Four screws roll up into one row with a quantity of 4.
    let screw_at = cells
        .iter()
        .position(|c| c == "m6_screw")
        .expect("m6_screw row");
    assert_eq!(cells[screw_at + 1], "4", "quantity should be rolled up");
    assert_eq!(cells[screw_at + 2], "stainless");
    // Three components: header row plus three data rows, five cells each.
    assert_eq!(cells.len(), 20, "expected 4 rows of 5 cells: {cells:?}");
}

#[test]
fn the_table_masses_agree_with_bom_text() {
    let ws = Workspace::new("masses");
    let svg = ws.export("panel.svg", "view: :top, bom: true");
    let cells = texts(group(&svg, "bom").expect("bom group"));

    let masses: Vec<f64> = eval_list(&format!(
        "{PANEL}\nasm.bom.map {{ |r| r[:mass] }}.join(\",\")"
    ))
    .iter()
    .map(|v| v.parse().expect("numeric mass"))
    .collect();

    // Table masses are the same numbers, rounded to two decimals.
    for (row, mass) in masses.iter().enumerate() {
        let cell = &cells[5 + row * 5 + 4];
        let shown: f64 = cell
            .parse()
            .unwrap_or_else(|_| panic!("mass cell {cell:?}"));
        assert!(
            (shown - mass).abs() < 0.005,
            "row {row}: table says {shown}, bom says {mass}"
        );
    }
}

#[test]
fn the_table_is_ruled_and_sits_below_the_drawing() {
    let ws = Workspace::new("rules");
    let svg = ws.export("panel.svg", "view: :top, bom: true");
    let table = group(&svg, "bom").expect("bom group");
    // Top, under-header, and bottom rules.
    assert_eq!(lines(table).len(), 3, "expected three rules");
    // SVG is Y-down: below the drawing means a larger y than the geometry,
    // which spans 0..80 in model terms and so 0..-80 on the page.
    for (_, y1, _, _) in lines(table) {
        assert!(
            y1 > 0.0,
            "the table should sit below the drawing, at y={y1}"
        );
    }
}

#[test]
fn the_page_grows_to_hold_the_table() {
    let ws = Workspace::new("page");
    let plain = ws.export("plain.svg", "view: :top");
    let with_bom = ws.export("bom.svg", "view: :top, bom: true");
    let height = |svg: &str| -> f64 {
        let at = svg.find(" height=\"").expect("height") + 9;
        let rest = &svg[at..];
        rest[..rest.find('"').unwrap()].parse().expect("numeric")
    };
    assert!(
        height(&with_bom) > height(&plain) + 20.0,
        "expected room for four table rows: {} vs {}",
        height(&with_bom),
        height(&plain)
    );
}

// ---------------------------------------------------------------------------
// Balloons
// ---------------------------------------------------------------------------

#[test]
fn one_balloon_per_parts_list_row() {
    let ws = Workspace::new("count");
    let svg = ws.export("panel.svg", "view: :top, bom: true, balloons: true");
    let balloons = group(&svg, "balloons").expect("balloons group");
    let numbers = texts(balloons);
    assert_eq!(
        numbers,
        vec!["1", "2", "3"],
        "expected one balloon per component, not per part"
    );
}

#[test]
fn a_balloons_leader_lands_on_the_component_it_numbers() {
    // The whole point of a balloon: number 2 must point at the part the table's
    // row 2 names. The post sits at (20, 50) in plan, the plate's centroid at
    // (60, 40), and the first screw at (15, 15).
    let ws = Workspace::new("anchor");
    let svg = ws.export("panel.svg", "view: :top, bom: true, balloons: true");
    let balloons = group(&svg, "balloons").expect("balloons group");
    let found = anchors(balloons);

    let expected: Vec<(f64, f64)> = eval_list(&format!(
        "{PANEL}\nasm.bom.map {{ |r| r[:component] }}.join(\",\")"
    ))
    .iter()
    .map(|name| match name.as_str() {
        "m6_screw" => (15.0, 15.0),
        "plate" => (60.0, 40.0),
        "post" => (28.0, 58.0),
        other => panic!("unexpected component {other}"),
    })
    .collect();

    assert_eq!(found.len(), expected.len(), "anchor count");
    for (i, (want, got)) in expected.iter().zip(&found).enumerate() {
        assert!(
            (want.0 - got.0).abs() < 1e-3 && (want.1 - got.1).abs() < 1e-3,
            "balloon {} should anchor at {want:?}, got {got:?}",
            i + 1
        );
    }
}

#[test]
fn balloons_sit_clear_of_the_geometry() {
    let ws = Workspace::new("clear");
    let svg = ws.export("panel.svg", "view: :top, balloons: true, bom: true");
    let balloons = group(&svg, "balloons").expect("balloons group");
    // The bubbles themselves (r = 4) all sit outside the 120 x 80 plate.
    for (cx, cy, r) in circles(balloons) {
        if r < 1.0 {
            continue; // an anchor dot, which is meant to be on the part
        }
        let (x, y) = (cx, -cy);
        assert!(
            !(0.0..=120.0).contains(&x) || !(0.0..=80.0).contains(&y),
            "balloon at ({x}, {y}) overlaps the part"
        );
    }
}

#[test]
fn each_leader_starts_on_its_balloons_edge_not_its_centre() {
    // A leader drawn from the centre would strike through the number.
    let ws = Workspace::new("leader");
    let svg = ws.export("panel.svg", "view: :top, balloons: true, bom: true");
    let balloons = group(&svg, "balloons").expect("balloons group");
    let bubbles: Vec<(f64, f64)> = circles(balloons)
        .into_iter()
        .filter(|(_, _, r)| *r > 1.0)
        .map(|(x, y, _)| (x, y))
        .collect();
    for (x1, y1, _, _) in lines(balloons) {
        let nearest = bubbles
            .iter()
            .map(|(bx, by)| ((bx - x1).powi(2) + (by - y1).powi(2)).sqrt())
            .fold(f64::MAX, f64::min);
        assert!(
            (nearest - 4.0).abs() < 1e-3,
            "leader should start on the 4 mm bubble edge, found {nearest} away"
        );
    }
}

#[test]
fn balloons_ring_the_top_view_on_a_three_view_sheet() {
    let ws = Workspace::new("sheet");
    let svg = ws.export("sheet.svg", "view: :sheet, bom: true, balloons: true");
    let balloons = group(&svg, "balloons").expect("balloons group");
    assert_eq!(texts(balloons).len(), 3, "one balloon per component");
    assert!(group(&svg, "bom").is_some(), "the sheet still gets a table");
}

#[test]
fn the_front_view_anchors_balloons_in_its_own_axes() {
    let ws = Workspace::new("front");
    let svg = ws.export("front.svg", "view: :front, bom: true, balloons: true");
    let balloons = group(&svg, "balloons").expect("balloons group");
    // The post's centroid is (28, 58, 26); the front view keeps X and Z.
    let found = anchors(balloons);
    assert!(
        found
            .iter()
            .any(|(x, y)| (x - 28.0).abs() < 1e-3 && (y - 26.0).abs() < 1e-3),
        "expected the post at (28, 26) in X/Z, got {found:?}"
    );
}

// ---------------------------------------------------------------------------
// The delimited channel
// ---------------------------------------------------------------------------

#[test]
fn a_component_name_containing_a_delimiter_does_not_shift_the_columns() {
    // Tabs separate cells and newlines separate rows, so a name carrying one
    // would silently move every column after it.
    let ws = Workspace::new("delims");
    let path = ws.path("odd.svg");
    ws.run(
        "asm = assembly(\"odd\") do |a|
           a.place box(20, 20, 5), name: :a, component: :\"we\\tird\", material: \"steel\"
         end
         asm.export(OUT, view: :top, bom: true)",
        &path,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&path).expect("read svg");
    let cells = texts(group(&svg, "bom").expect("bom group"));
    assert_eq!(
        cells.len(),
        10,
        "header plus one row of five cells: {cells:?}"
    );
    assert_eq!(cells[6], "we ird", "the tab should become a space");
}

#[test]
fn a_component_name_with_markup_characters_stays_valid_xml() {
    let ws = Workspace::new("xml");
    let path = ws.path("xml.svg");
    ws.run(
        "asm = assembly(\"xml\") do |a|
           a.place box(20, 20, 5), name: :a, component: :\"M6 <A&B>\", material: \"steel\"
         end
         asm.export(OUT, view: :top, bom: true)",
        &path,
    )
    .expect("export");
    let svg = std::fs::read_to_string(&path).expect("read svg");
    assert!(
        svg.contains("M6 &lt;A&amp;B&gt;"),
        "expected escaped markup in the cell"
    );
    assert!(
        !svg.contains("<A&B>"),
        "raw markup would make the document unparseable"
    );
}

#[test]
fn neither_annotation_appears_unless_asked_for() {
    let ws = Workspace::new("absent");
    let svg = ws.export("plain.svg", "view: :top");
    assert!(group(&svg, "bom").is_none(), "unexpected parts list");
    assert!(group(&svg, "balloons").is_none(), "unexpected balloons");
}

#[test]
fn the_solid_formats_ignore_the_drawing_annotations() {
    let ws = Workspace::new("step");
    let step = ws.export("panel.step", "bom: true, balloons: true");
    assert!(step.contains("ISO-10303-21"), "expected a STEP file");
}

// ---------------------------------------------------------------------------
// DXF
// ---------------------------------------------------------------------------

#[test]
fn dxf_puts_the_table_and_balloons_on_their_own_layers() {
    let ws = Workspace::new("dxf");
    let path = ws.path("panel.dxf");
    ws.run(
        &format!("{PANEL}\nasm.export(OUT, view: :top, bom: true, balloons: true)"),
        &path,
    )
    .expect("export");
    let entities = parse_dxf(&path);

    assert_eq!(
        on_layer(&entities, "LINE", "BOM").len(),
        3,
        "three table rules on the BOM layer"
    );
    assert_eq!(
        on_layer(&entities, "TEXT", "BOM").len(),
        20,
        "four rows of five cells"
    );
    assert_eq!(
        on_layer(&entities, "CIRCLE", "BALLOON").len(),
        3,
        "one bubble per component"
    );
    // Balloon numbers are centred (group code 72 = 1), which needs the second
    // alignment point to take effect.
    for text in on_layer(&entities, "TEXT", "BALLOON") {
        assert_eq!(text.1.get("72").map(String::as_str), Some("1"));
        assert!(text.1.contains_key("11") && text.1.contains_key("21"));
    }
}
