// Sheet metal — `sheet_metal`, edge flanges, bend relief, flat patterns.
//
// A sheet-metal part has two deliverables that have to agree: the folded solid
// and the flat blank the laser cuts. Most of what can go wrong here produces
// two individually plausible results that disagree — a bend allowance applied
// to the wrong run, a relief notch cut on one and not the other, a flange
// folded off the wrong side. So the tests check both against hand-computed
// geometry rather than against each other, and the numbers below are worked
// out from the bend formula and plain trigonometry, not read back from the
// implementation.
//
// Bend allowance is `angle_rad * (radius + k * thickness)` — the arc length of
// the neutral axis. For the common fixture (t = 2, r = 2, k = 0.44, 90°) that
// is `PI/2 * 2.88 = 4.5238934211693`.

use rrcad::ruby::vm::MrubyVm;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Bend allowance for the standard fixture: t = 2, r = 2, k = 0.44, 90°.
const BA90: f64 = std::f64::consts::FRAC_PI_2 * (2.0 + 0.44 * 2.0);

/// OCCT's booleans leave sub-nanometre noise on coordinates; assertions on
/// derived lengths and volumes are held to this.
const EPS: f64 = 1e-6;

/// Evaluate `script` and return its final value, with the quotes `inspect`
/// wraps a String in stripped off.
fn eval(script: &str) -> String {
    let mut vm = MrubyVm::new();
    let out = vm.eval(script).expect("script should evaluate");
    out.trim().trim_matches('"').to_string()
}

/// Evaluate a script whose last expression is a comma-joined list of numbers.
fn nums(script: &str) -> Vec<f64> {
    eval(script)
        .split(',')
        .map(|s| s.trim().parse().expect("numeric field"))
        .collect()
}

/// Evaluate a script expected to raise, returning the error message.
fn err(script: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(script).expect_err("script should raise")
}

/// Bounding box and volume of a folded part, as `x,y,z,dx,dy,dz,volume`.
fn folded(builder: &str) -> Vec<f64> {
    nums(&format!(
        r#"
        part = {builder}
        s = part.to_shape
        b = s.bounding_box
        [b[:x], b[:y], b[:z], b[:dx], b[:dy], b[:dz], s.volume].join(",")
        "#
    ))
}

/// Bounding box and area of a blank, as `x,y,dx,dy,area`.
fn blank(builder: &str) -> Vec<f64> {
    nums(&format!(
        r#"
        part = {builder}
        f = part.flat
        b = f.bounding_box
        [b[:x], b[:y], b[:dx], b[:dy], f.surface_area].join(",")
        "#
    ))
}

fn close(got: f64, want: f64, what: &str) {
    assert!(
        (got - want).abs() < EPS,
        "{what}: got {got}, expected {want}"
    );
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A 100 × 60 × 2 plate with a 25 mm wall folded square off its +x side.
const L_BRACKET: &str = r#"sheet_metal(thickness: 2, radius: 2) do |s|
        s.base 100, 60
        s.flange :xmax, length: 25
      end"#;

/// The same plate with the flange narrowed, so both bend ends get relief.
const RELIEVED: &str = r#"sheet_metal(thickness: 2, radius: 2) do |s|
        s.base 100, 60
        s.flange :ymin, length: 15, from: 10, to: 50
      end"#;

// ---------------------------------------------------------------------------
// The folded solid
// ---------------------------------------------------------------------------

#[test]
fn a_square_flange_lands_one_radius_plus_one_thickness_out() {
    // The bend's centre sits a radius above the plate's top face, so the wall
    // ends up r + t beyond the bend line and t + r + length tall. Getting the
    // centre wrong by a thickness is the classic sheet-metal off-by-one and
    // would show here as a 2 mm error in both.
    let g = folded(L_BRACKET);
    close(g[3], 104.0, "overall length"); // 100 + r + t
    close(g[4], 60.0, "width");
    close(g[5], 29.0, "height"); // t + r + 25

    // Plate + quarter tube + wall, each computed independently.
    let plate = 100.0 * 60.0 * 2.0;
    let bend = std::f64::consts::FRAC_PI_4 * (4.0 * 4.0 - 2.0 * 2.0) * 60.0;
    let wall = 25.0 * 60.0 * 2.0;
    close(g[6], plate + bend + wall, "volume");
}

#[test]
fn the_leg_is_measured_past_the_bend_not_from_the_bend_line() {
    // `length:` is the straight run beyond the arc. If it were measured from
    // the bend line instead, opening the radius from 2 to 5 would leave the
    // height alone; measured past the bend, the height grows with it.
    let tight = folded(
        r#"sheet_metal(thickness: 2, radius: 2) { |s| s.base(50, 40); s.flange(:xmax, length: 20) }"#,
    );
    let loose = folded(
        r#"sheet_metal(thickness: 2, radius: 5) { |s| s.base(50, 40); s.flange(:xmax, length: 20) }"#,
    );
    close(tight[5], 2.0 + 2.0 + 20.0, "height at r = 2");
    close(loose[5], 2.0 + 5.0 + 20.0, "height at r = 5");
}

#[test]
fn a_shallow_bend_lands_where_trigonometry_says() {
    // 45°, checked against the closed form rather than against a 90° case,
    // because a sign error in the sweep direction survives the square bend.
    let g = folded(
        r#"sheet_metal(thickness: 2, radius: 2) { |s| s.base(100, 60); s.flange(:xmax, length: 25, angle: 45) }"#,
    );
    let (c, s) = (
        std::f64::consts::FRAC_PI_4.cos(),
        std::f64::consts::FRAC_PI_4.sin(),
    );
    // Furthest point in x is the outer surface at the wall's far end; highest
    // point in z is the inner surface there.
    let x_max = 100.0 + 4.0 * s + 25.0 * s;
    let z_max = 2.0 + 2.0 - 2.0 * c + 25.0 * c;
    close(g[3], x_max, "reach in x");
    close(g[5], z_max, "reach in z");
}

#[test]
fn a_hem_folds_the_wall_back_over_the_base() {
    // 180° is the limit case: the wall ends up parallel to the plate, its
    // underside one bend diameter above the plate's top.
    let g = folded(
        r#"sheet_metal(thickness: 1, radius: 1) do |s|
             s.base 50, 40
             s.flange :ymax, length: 10, angle: 180
           end"#,
    );
    close(g[4], 40.0 + 2.0, "reach in y"); // r + t past the bend line
    close(g[5], 4.0, "height"); // t + r + r + t
}

#[test]
fn the_folded_part_is_closed_manifold_and_counted_once() {
    // Each flange is a bend and a wall built separately and fused onto the
    // plate. A gap between any two would still export and still look right in
    // a viewer; overlapping material would look right too and weigh more. The
    // volume is summed from the parts, so both show up.
    let out = eval(
        r#"
        part = sheet_metal(thickness: 1.5, radius: 1.5) do |s|
          s.base 80, 60
          s.flange :xmin, length: 20, from: 5, to: 55
          s.flange :xmax, length: 20, from: 5, to: 55
          s.flange :ymin, length: 20, from: 5, to: 75
          s.flange :ymax, length: 20, from: 5, to: 75
        end
        s = part.to_shape
        [s.closed?, s.manifold?, s.volume].join(",")
        "#,
    );
    let fields: Vec<&str> = out.split(',').collect();
    assert_eq!(
        &fields[..2],
        ["true", "true"],
        "tray should be a closed solid"
    );

    // Eight relief notches (1.5 wide, 3 deep) out of the plate, four quarter
    // tubes, four walls — 240 mm of bend line in total.
    let plate = (80.0 * 60.0 - 8.0 * 1.5 * 3.0) * 1.5;
    let bend = std::f64::consts::FRAC_PI_4 * (3.0 * 3.0 - 1.5 * 1.5) * 240.0;
    let wall = 20.0 * 240.0 * 1.5;
    close(
        fields[2].parse().expect("volume"),
        plate + bend + wall,
        "tray volume",
    );
}

// ---------------------------------------------------------------------------
// Which way each side folds
// ---------------------------------------------------------------------------

#[test]
fn every_side_folds_outward_from_the_plate() {
    // Four sides, four local frames. A wrong rotation or origin on any one of
    // them puts the flange through the plate instead of off it, so each case
    // asserts the plate is untouched in the three directions it did not grow.
    for (side, want) in [
        (":xmax", (0.0, 0.0, 54.0, 40.0)),
        (":xmin", (-4.0, 0.0, 54.0, 40.0)),
        (":ymax", (0.0, 0.0, 50.0, 44.0)),
        (":ymin", (0.0, -4.0, 50.0, 44.0)),
    ] {
        let g = folded(&format!(
            "sheet_metal(thickness: 2, radius: 2) {{ |s| s.base(50, 40); s.flange({side}, length: 10) }}"
        ));
        close(g[0], want.0, &format!("{side} x origin"));
        close(g[1], want.1, &format!("{side} y origin"));
        close(g[3], want.2, &format!("{side} dx"));
        close(g[4], want.3, &format!("{side} dy"));
        close(g[5], 14.0, &format!("{side} height"));
    }
}

#[test]
fn a_tray_folds_symmetrically() {
    // A square plate with four identical flanges. Any per-side sign error in
    // the frame — a span reversed, an origin on the wrong corner — breaks the
    // symmetry of the blank while leaving each individual flange plausible.
    let b = blank(
        r#"sheet_metal(thickness: 1.5, radius: 1.5) do |s|
             s.base 60, 60
             [:xmin, :xmax, :ymin, :ymax].each { |e| s.flange e, length: 20, from: 8, to: 52 }
           end"#,
    );
    close(b[0], b[1], "blank origin should be symmetric in x and y");
    close(b[2], b[3], "blank should be square");
    // Centred on the plate: the overhang is the same on both sides.
    close(
        b[0] + b[2],
        -b[0] + 60.0,
        "blank should straddle the plate evenly",
    );
}

// ---------------------------------------------------------------------------
// Bend allowance
// ---------------------------------------------------------------------------

#[test]
fn the_blank_grows_by_the_bend_allowance_plus_the_leg() {
    // The whole point of the flat pattern. The blank is longer than the plate
    // by the arc length of the neutral axis plus the straight leg — not by
    // the leg alone, and not by the outside dimension.
    let b = blank(L_BRACKET);
    close(b[2], 100.0 + BA90 + 25.0, "blank length");
    close(b[3], 60.0, "blank width");
    close(b[4], 60.0 * (100.0 + BA90 + 25.0), "blank area");

    // Stated plainly: it is neither of the two tempting wrong answers.
    assert!(
        b[2] > 125.0,
        "the bend must consume more than the leg alone"
    );
    assert!(
        b[2] < 100.0 + 4.0 + 25.0 + 4.0,
        "the blank must be shorter than the outside girth"
    );
}

#[test]
fn the_k_factor_moves_the_neutral_axis_and_the_blank_with_it() {
    // A higher k puts the neutral axis further from the inside of the bend,
    // so more material is needed. The two blanks differ by exactly the
    // difference in allowance.
    let low = blank(
        r#"sheet_metal(thickness: 2, radius: 2, k_factor: 0.3) { |s| s.base(100, 60); s.flange(:xmax, length: 25) }"#,
    );
    let high = blank(
        r#"sheet_metal(thickness: 2, radius: 2, k_factor: 0.5) { |s| s.base(100, 60); s.flange(:xmax, length: 25) }"#,
    );
    let quarter = std::f64::consts::FRAC_PI_2;
    close(low[2], 125.0 + quarter * (2.0 + 0.3 * 2.0), "k = 0.3 blank");
    close(
        high[2],
        125.0 + quarter * (2.0 + 0.5 * 2.0),
        "k = 0.5 blank",
    );
    close(
        high[2] - low[2],
        quarter * (0.5 - 0.3) * 2.0,
        "difference in allowance",
    );
}

#[test]
fn a_tighter_radius_needs_less_blank() {
    let tight = blank(
        r#"sheet_metal(thickness: 2, radius: 1) { |s| s.base(100, 60); s.flange(:xmax, length: 25) }"#,
    );
    let loose = blank(
        r#"sheet_metal(thickness: 2, radius: 6) { |s| s.base(100, 60); s.flange(:xmax, length: 25) }"#,
    );
    let quarter = std::f64::consts::FRAC_PI_2;
    close(tight[2], 125.0 + quarter * (1.0 + 0.88), "r = 1 blank");
    close(loose[2], 125.0 + quarter * (6.0 + 0.88), "r = 6 blank");
}

#[test]
fn a_shallower_bend_consumes_proportionally_less() {
    // Allowance is linear in the angle: 45° takes exactly half of 90°.
    let b45 = blank(
        r#"sheet_metal(thickness: 2, radius: 2) { |s| s.base(100, 60); s.flange(:xmax, length: 25, angle: 45) }"#,
    );
    close(b45[2], 125.0 + BA90 / 2.0, "45° blank");
}

#[test]
fn the_bend_table_reports_what_was_used() {
    let out = eval(&format!(
        r#"
        part = {L_BRACKET}
        b = part.bends[0]
        [b[:side], b[:angle], b[:radius], b[:length], b[:from], b[:to],
         b[:allowance], b[:relief]].join(",")
        "#
    ));
    let fields: Vec<&str> = out.split(',').collect();
    assert_eq!(fields[0], "xmax");
    assert_eq!(fields[1], "90.0");
    assert_eq!(fields[7], "none");
    close(
        fields[6].parse().expect("allowance"),
        BA90,
        "reported allowance",
    );
}

// ---------------------------------------------------------------------------
// Bend relief
// ---------------------------------------------------------------------------

#[test]
fn a_narrowed_flange_notches_the_plate_at_both_bend_ends() {
    // Relief is not decoration: without it the plate tears where the fold
    // stops. Default notch is one thickness wide and radius + thickness deep.
    let g = folded(RELIEVED);
    let notch = 2.0 * 4.0; // relief_width t=2 × relief_depth r+t=4
    let plate = 100.0 * 60.0 * 2.0 - 2.0 * notch * 2.0;
    let bend = std::f64::consts::FRAC_PI_4 * (16.0 - 4.0) * 40.0;
    let wall = 15.0 * 40.0 * 2.0;
    close(g[6], plate + bend + wall, "folded volume with two notches");
}

#[test]
fn the_notch_in_the_blank_is_the_notch_in_the_solid() {
    // The two deliverables have to agree. Cutting relief into the folded part
    // but not the blank — or the reverse — produces two files that each look
    // right and cannot both be built.
    let g = folded(RELIEVED);
    let b = blank(RELIEVED);

    // Blank area, with each notch removed and the developed flange added.
    let notch = 2.0 * 4.0;
    let ext = BA90 + 15.0;
    close(
        b[4],
        100.0 * 60.0 - 2.0 * notch + 40.0 * ext,
        "blank area with relief",
    );
    // And the same notch shows up in the solid, one thickness deep.
    let unrelieved = folded(
        r#"sheet_metal(thickness: 2, radius: 2) do |s|
             s.base 100, 60
             s.flange :ymin, length: 15, from: 10, to: 50, relief: :none
           end"#,
    );
    close(
        unrelieved[6] - g[6],
        2.0 * notch * 2.0,
        "the solid loses exactly the two notches",
    );
}

#[test]
fn relief_is_skipped_where_the_flange_already_reaches_the_corner() {
    // There is no material beside the bend end to relieve, and cutting there
    // would notch the outside edge of the part.
    let g = folded(
        r#"sheet_metal(thickness: 2, radius: 2) do |s|
             s.base 100, 60
             s.flange :ymin, length: 15, to: 50
           end"#,
    );
    let plate = 100.0 * 60.0 * 2.0 - 2.0 * 4.0 * 2.0; // one notch only
    let bend = std::f64::consts::FRAC_PI_4 * (16.0 - 4.0) * 50.0;
    let wall = 15.0 * 50.0 * 2.0;
    close(g[6], plate + bend + wall, "one-notch volume");
}

#[test]
fn relief_none_leaves_the_plate_whole() {
    let g = folded(
        r#"sheet_metal(thickness: 2, radius: 2) do |s|
             s.base 100, 60
             s.flange :ymin, length: 15, from: 10, to: 50, relief: :none
           end"#,
    );
    let bend = std::f64::consts::FRAC_PI_4 * (16.0 - 4.0) * 40.0;
    close(
        g[6],
        100.0 * 60.0 * 2.0 + bend + 15.0 * 40.0 * 2.0,
        "unrelieved volume",
    );
}

#[test]
fn an_obround_notch_keeps_its_stated_depth() {
    // The round end is part of the depth, not added to it, so switching relief
    // style does not quietly move the bend line's clearance.
    let b = blank(
        r#"sheet_metal(thickness: 2, radius: 2) do |s|
             s.base 100, 60
             s.flange :ymin, length: 15, from: 10, to: 50, relief: :obround
           end"#,
    );
    // Each notch: a 2 × 3 rectangle plus a half-circle of radius 1.
    let notch = 2.0 * 3.0 + std::f64::consts::FRAC_PI_2;
    close(
        b[4],
        100.0 * 60.0 - 2.0 * notch + 40.0 * (BA90 + 15.0),
        "blank area with obround relief",
    );
    close(
        b[1],
        -(BA90 + 15.0),
        "blank should still reach past the flange",
    );
}

// ---------------------------------------------------------------------------
// The blank as a cut file
// ---------------------------------------------------------------------------

/// A throwaway working directory, removed on drop. `safe_path` confines
/// exports to the process CWD, so these write into it and clean up.
struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/sheetmetal_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

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

fn code(entity: &Entity, c: &str) -> f64 {
    entity.1[c].parse().expect("numeric group code")
}

/// Export a blank and return its parsed DXF entities.
fn blank_dxf(ws: &Workspace, name: &str, builder: &str) -> Vec<Entity> {
    let out = ws.path(name);
    let literal = format!("{:?}", out.to_string_lossy());
    let mut vm = MrubyVm::new();
    vm.eval(&format!(
        "part = {builder}\npart.export_flat({literal})\n\"ok\""
    ))
    .expect("export_flat should succeed");
    parse_dxf(&out)
}

#[test]
fn the_blank_outline_closes_into_a_single_ring() {
    // A traced outline is easy to get subtly wrong — a duplicated vertex, a
    // side walked in the wrong direction, a notch left open. In a closed ring
    // every vertex is shared by exactly two segments, which is a property no
    // amount of coincidence produces.
    let ws = Workspace::new("ring");
    let entities = blank_dxf(&ws, "blank.dxf", RELIEVED);
    let lines: Vec<&Entity> = entities.iter().filter(|(k, _)| k == "LINE").collect();
    assert_eq!(lines.len(), 12, "expected 12 segments in the outline");

    let mut seen: HashMap<(i64, i64), usize> = HashMap::new();
    for l in &lines {
        for (x, y) in [
            (code(l, "10"), code(l, "20")),
            (code(l, "11"), code(l, "21")),
        ] {
            *seen
                .entry(((x * 1e6) as i64, (y * 1e6) as i64))
                .or_insert(0) += 1;
        }
    }
    assert_eq!(seen.len(), 12, "expected 12 distinct vertices");
    for (pt, n) in &seen {
        assert_eq!(*n, 2, "vertex {pt:?} is used {n} times, not twice");
    }
}

#[test]
fn the_notch_is_cut_where_the_bend_stops() {
    // Both notch corners, at true position on the blank. A notch on the wrong
    // side of the flange, or measured from the wrong end, still yields a
    // closed ring of the right length.
    let ws = Workspace::new("notch");
    let entities = blank_dxf(&ws, "blank.dxf", RELIEVED);

    // export_outline shifts the blank so its bounding box starts at the
    // origin; the plate's own edge is therefore at y = BA + 15, and the notch
    // floors sit 4 mm further in, since relief cuts back into the plate.
    let edge = BA90 + 15.0;
    let mut corners: Vec<(f64, f64)> = entities
        .iter()
        .filter(|(k, _)| k == "LINE")
        .flat_map(|l| {
            [
                (code(l, "10"), code(l, "20")),
                (code(l, "11"), code(l, "21")),
            ]
        })
        .filter(|(_, y)| (*y - (edge + 4.0)).abs() < EPS)
        .collect();
    corners.sort_by(|a, b| a.partial_cmp(b).unwrap());
    corners.dedup_by(|a, b| (a.0 - b.0).abs() < EPS && (a.1 - b.1).abs() < EPS);

    // Notch floors sit 4 mm into the plate, spanning x 8–10 and 50–52.
    let want = [8.0, 10.0, 50.0, 52.0];
    assert_eq!(corners.len(), 4, "expected four notch-floor corners");
    for (got, expect) in corners.iter().zip(want.iter()) {
        close(got.0, *expect, "notch corner x");
    }
}

#[test]
fn an_obround_relief_reaches_the_cut_file_as_a_true_arc() {
    // The round end exists so the notch does not start a crack. Approximated
    // as chords it is no longer a radius, and the whole point is lost.
    let ws = Workspace::new("obround");
    let entities = blank_dxf(
        &ws,
        "blank.dxf",
        r#"sheet_metal(thickness: 2, radius: 2) do |s|
             s.base 100, 60
             s.flange :ymin, length: 15, from: 10, to: 50, relief: :obround
           end"#,
    );
    let arcs: Vec<&Entity> = entities.iter().filter(|(k, _)| k == "ARC").collect();
    assert!(
        !arcs.is_empty(),
        "an obround notch must emit ARC entities, not chords"
    );
    for a in &arcs {
        close(
            code(a, "40"),
            1.0,
            "arc radius should be half the notch width",
        );
    }
}

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

#[test]
fn flanges_that_would_meet_at_a_corner_are_refused() {
    // They touch at a single point with nothing joining them, and the blank
    // pinches to zero width there. The folded solid looks fine, so this has to
    // be caught at the call.
    let msg = err(r#"sheet_metal(thickness: 2) do |s|
             s.base 50, 50
             s.flange :xmax, length: 5
             s.flange :ymax, length: 5
           end"#);
    assert!(
        msg.contains("shared corner") && msg.contains("from:/to:"),
        "unhelpful corner message: {msg}"
    );
}

#[test]
fn a_full_width_flange_cannot_be_relieved() {
    let msg = err(
        r#"sheet_metal(thickness: 2) { |s| s.base(50, 50); s.flange(:xmax, length: 5, relief: :rectangular) }"#,
    );
    assert!(msg.contains("no"), "unhelpful message: {msg}");
    assert!(
        msg.contains("relief: :none"),
        "should say what to do: {msg}"
    );
}

#[test]
fn a_notch_with_no_room_beside_it_is_refused() {
    // The flange starts 1 mm from the corner but the notch is 2 mm wide, so
    // it would run off the end of the plate.
    let msg = err(
        r#"sheet_metal(thickness: 2) { |s| s.base(50, 50); s.flange(:xmax, length: 5, from: 1, to: 40) }"#,
    );
    assert!(msg.contains("wide"), "unhelpful message: {msg}");
}

#[test]
fn a_notch_deeper_than_the_plate_is_refused() {
    let msg = err(r#"sheet_metal(thickness: 2, radius: 2) do |s|
             s.base 20, 50
             s.flange :xmax, length: 5, from: 5, to: 40, relief_depth: 25
           end"#);
    assert!(msg.contains("clean through"), "unhelpful message: {msg}");
}

#[test]
fn an_impossible_bend_angle_is_refused() {
    for angle in ["0", "-30", "200"] {
        let msg = err(&format!(
            "sheet_metal(thickness: 2) {{ |s| s.base(50, 50); s.flange(:xmax, length: 5, angle: {angle}) }}"
        ));
        assert!(msg.contains("angle"), "angle {angle}: {msg}");
    }
}

#[test]
fn a_second_flange_on_one_side_is_refused() {
    let msg = err(r#"sheet_metal(thickness: 2) do |s|
             s.base 50, 50
             s.flange :xmax, length: 5, to: 20
             s.flange :xmax, length: 5, from: 30
           end"#);
    assert!(msg.contains("already carries"), "unhelpful message: {msg}");
}

#[test]
fn a_flange_span_off_the_end_of_the_side_is_refused() {
    let msg = err(
        r#"sheet_metal(thickness: 2) { |s| s.base(50, 40); s.flange(:xmax, length: 5, from: 10, to: 90) }"#,
    );
    assert!(msg.contains("does not fit"), "unhelpful message: {msg}");
}

#[test]
fn a_flange_without_a_base_is_refused() {
    let msg = err(r#"sheet_metal(thickness: 2) { |s| s.flange(:xmax, length: 5) }"#);
    assert!(msg.contains("base"), "unhelpful message: {msg}");
}

#[test]
fn the_k_factor_must_lie_inside_the_thickness() {
    // k is a fraction of the thickness, so 0 and 1 put the neutral axis on a
    // surface, which is not a bend model anyone uses.
    for k in ["0", "1", "1.4"] {
        let msg = err(&format!("sheet_metal(thickness: 2, k_factor: {k})"));
        assert!(msg.contains("k_factor"), "k = {k}: {msg}");
    }
}

#[test]
fn an_unknown_side_is_refused_by_name() {
    let msg = err(r#"sheet_metal(thickness: 2) { |s| s.base(50, 50); s.flange(:top, length: 5) }"#);
    assert!(msg.contains(":top"), "should name the bad side: {msg}");
}
