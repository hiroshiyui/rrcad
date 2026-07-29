// 3MF export.
//
// 3MF exists in this project because STL does not carry the three things a
// slicer needs and rrcad knows: the unit the numbers are in, the colour, and
// where one body ends and the next begins. So these tests check those three
// properties rather than that a file appeared.
//
// The mesh checks are the load-bearing ones. A triangle list can be the right
// size, parse cleanly, and still print inside out or with holes in it, and
// neither a file-size assertion nor a triangle count would notice. Both are
// checked against the geometry OCCT reports for the same shape, so the test
// would fail if the writer and the kernel ever disagreed:
//
//   * signed volume — sums to the true volume only when every triangle winds
//     counter-clockwise seen from outside
//   * edge parity — every directed edge appears exactly once, and so does its
//     reverse, which holds only for a closed, consistently oriented surface

use rrcad::ruby::vm::MrubyVm;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// A throwaway working directory, removed on drop. `safe_path` confines
/// exports to the process CWD, so these write into it and clean up.
struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/threemf_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Run `script` with `OUT` substituted for the export path.
    fn run(&self, script: &str, out: &Path) {
        let literal = format!("{:?}", out.to_string_lossy());
        let mut vm = MrubyVm::new();
        vm.eval(&script.replace("OUT", &literal))
            .expect("export should succeed");
    }

    /// Run `script`, expecting it to fail, and return the error text.
    fn run_err(&self, script: &str, out: &Path) -> String {
        let literal = format!("{:?}", out.to_string_lossy());
        let mut vm = MrubyVm::new();
        match vm.eval(&script.replace("OUT", &literal)) {
            Ok(v) => panic!("expected the export to be refused, got {v:?}"),
            Err(e) => e.to_string(),
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// Read one part out of the package.
fn part(path: &Path, name: &str) -> String {
    let file = std::fs::File::open(path).expect("open package");
    let mut zip = zip::ZipArchive::new(file).expect("a 3MF should be a readable ZIP");
    let mut entry = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("package should contain {name}"));
    let mut body = String::new();
    entry.read_to_string(&mut body).expect("read part");
    body
}

fn model(path: &Path) -> String {
    part(path, "3D/3dmodel.model")
}

/// The value of `name="…"` in `tag`, if present.
fn attr(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Every `<tag …>` opening tag in `xml`, as raw text.
fn tags<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag} ");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find(&open) {
        rest = &rest[i..];
        let end = rest.find('>').expect("unterminated tag");
        out.push(&rest[..end]);
        rest = &rest[end..];
    }
    out
}

/// One mesh, as vertices and index triples.
struct Mesh {
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
}

impl Mesh {
    /// Six times the signed volume, by the divergence theorem: for a closed
    /// surface whose triangles wind counter-clockwise from outside, this is
    /// positive and equals the enclosed volume. Inverted winding makes it
    /// negative; a hole makes it wrong.
    fn signed_volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|t| {
                let (p, q, r) = (
                    self.vertices[t[0]],
                    self.vertices[t[1]],
                    self.vertices[t[2]],
                );
                let cross = [
                    q[1] * r[2] - q[2] * r[1],
                    q[2] * r[0] - q[0] * r[2],
                    q[0] * r[1] - q[1] * r[0],
                ];
                (p[0] * cross[0] + p[1] * cross[1] + p[2] * cross[2]) / 6.0
            })
            .sum()
    }

    /// Directed edges that break the closed-surface rule: used other than
    /// exactly once, or without their reverse. A welded, closed, consistently
    /// oriented mesh has none.
    fn broken_edges(&self) -> usize {
        let mut count: HashMap<(usize, usize), usize> = HashMap::new();
        for t in &self.triangles {
            for e in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                *count.entry(e).or_insert(0) += 1;
            }
        }
        count
            .iter()
            .filter(|&(&(a, b), &n)| n != 1 || count.get(&(b, a)).copied().unwrap_or(0) != 1)
            .count()
    }
}

/// Split the model XML into one Mesh per `<object>`.
fn meshes(xml: &str) -> Vec<Mesh> {
    xml.split("<object ")
        .skip(1)
        .map(|body| {
            let vertices = tags(body, "vertex")
                .iter()
                .map(|t| {
                    let get = |n: &str| {
                        attr(t, n)
                            .unwrap_or_else(|| panic!("vertex missing {n}"))
                            .parse::<f64>()
                            .expect("vertex coordinate should be a number")
                    };
                    [get("x"), get("y"), get("z")]
                })
                .collect();
            let triangles = tags(body, "triangle")
                .iter()
                .map(|t| {
                    let get = |n: &str| {
                        attr(t, n)
                            .unwrap_or_else(|| panic!("triangle missing {n}"))
                            .parse::<usize>()
                            .expect("triangle index should be an integer")
                    };
                    [get("v1"), get("v2"), get("v3")]
                })
                .collect();
            Mesh {
                vertices,
                triangles,
            }
        })
        .collect()
}

/// Ask the kernel for the same shape's volume, so the mesh is checked against
/// geometry rather than against a number typed into this file.
fn volume_of(script: &str) -> f64 {
    let mut vm = MrubyVm::new();
    vm.eval(script)
        .expect("volume query should succeed")
        .trim()
        .parse::<f64>()
        .expect("volume should be a number")
}

// ---------------------------------------------------------------------------
// The package
// ---------------------------------------------------------------------------

#[test]
fn the_export_is_a_well_formed_3mf_package() {
    // A 3MF is an OPC package, not just a ZIP with a model in it: a reader
    // that cannot find the content types or the root relationship is entitled
    // to reject the file even though the geometry is right there.
    let ws = Workspace::new("package");
    let out = ws.path("cube.3mf");
    ws.run("box(10, 20, 30).export(OUT)", &out);

    let types = part(&out, "[Content_Types].xml");
    assert!(
        types.contains("3dmanufacturing-3dmodel+xml"),
        "the model's content type should be declared: {types}"
    );
    let rels = part(&out, "_rels/.rels");
    assert!(
        rels.contains("Target=\"/3D/3dmodel.model\""),
        "the root relationship should point at the model: {rels}"
    );
    assert!(model(&out).contains("<model "), "model part should be XML");
}

#[test]
fn the_model_says_what_unit_the_numbers_are_in() {
    // This is the whole reason to prefer 3MF over STL for a printable part.
    // An STL says "10"; the receiving end guesses whether that is millimetres.
    let ws = Workspace::new("unit");
    let out = ws.path("cube.3mf");
    ws.run("box(10, 20, 30).export(OUT)", &out);
    assert!(
        model(&out).contains("unit=\"millimeter\""),
        "the model should declare millimetres"
    );
}

// ---------------------------------------------------------------------------
// The mesh
// ---------------------------------------------------------------------------

#[test]
fn the_mesh_is_closed_and_wound_outward() {
    // A curved shape with a through hole, so the tessellator is doing real
    // work and the mesh has both an outer and an inner surface — the inner one
    // winds the other way, which is what the face-orientation handling is for.
    const SHAPE: &str = "cylinder(10, 20).cut(cylinder(4, 30).translate(0, 0, -5))";
    let ws = Workspace::new("closed");
    let out = ws.path("tube.3mf");
    ws.run(&format!("{SHAPE}.export(OUT)"), &out);

    let mesh = meshes(&model(&out)).pop().expect("one object");
    assert_eq!(
        mesh.broken_edges(),
        0,
        "every directed edge should appear once with its reverse — the mesh has holes or flipped faces"
    );

    // The faceted mesh under-fills a curved solid slightly, so it is compared
    // against the true volume with a tolerance, not for equality. The sign and
    // the magnitude are what matter: an inverted mesh gives the negative, and
    // a hole gives something unrelated.
    let exact = volume_of(&format!("{SHAPE}.volume"));
    let meshed = mesh.signed_volume();
    assert!(
        meshed > 0.0,
        "signed volume should be positive — the mesh is inside out ({meshed})"
    );
    assert!(
        (meshed - exact).abs() / exact < 0.02,
        "meshed volume {meshed} should be within 2% of the true volume {exact}"
    );
}

#[test]
fn a_box_welds_to_its_eight_corners() {
    // OCCT tessellates face by face, so a box arrives as six independent quads
    // with 24 nodes between them. Welding is what turns that into a closed
    // solid; without it the file is a pile of loose faces that happens to look
    // right in a viewer.
    let ws = Workspace::new("weld");
    let out = ws.path("cube.3mf");
    ws.run("box(10, 20, 30).export(OUT)", &out);

    let mesh = meshes(&model(&out)).pop().expect("one object");
    assert_eq!(mesh.vertices.len(), 8, "a box has eight distinct corners");
    assert_eq!(mesh.triangles.len(), 12, "six quads, two triangles each");
    assert!((mesh.signed_volume() - 6000.0).abs() < 1e-6);
}

#[test]
fn each_solid_becomes_its_own_object() {
    // The other thing STL cannot do: two disjoint bodies go into an STL as one
    // undifferentiated triangle soup, and the slicer has to guess them apart.
    let ws = Workspace::new("bodies");
    let out = ws.path("pair.3mf");
    ws.run(
        "box(10, 10, 10).fuse(box(10, 10, 10).translate(30, 0, 0)).export(OUT)",
        &out,
    );

    let xml = model(&out);
    let mesh_list = meshes(&xml);
    assert_eq!(mesh_list.len(), 2, "two disjoint boxes are two objects");
    for mesh in &mesh_list {
        assert!((mesh.signed_volume() - 1000.0).abs() < 1e-6);
        assert_eq!(mesh.broken_edges(), 0);
    }
    // Every object has to be placed, or it is a resource nothing prints.
    assert_eq!(
        tags(&xml, "item").len(),
        2,
        "both objects should appear in the build"
    );
}

#[test]
fn a_shape_with_no_solid_still_exports() {
    // A sheet-metal blank is a face: no volume, but a real outline someone may
    // want to hand to a cutter as a mesh. Exporting nothing would be worse
    // than exporting a surface.
    let ws = Workspace::new("face");
    let out = ws.path("blank.3mf");
    ws.run("rect(40, 25).export(OUT)", &out);

    let mesh = meshes(&model(&out)).pop().expect("one object");
    assert!(
        !mesh.triangles.is_empty(),
        "the face should tessellate to triangles"
    );
}

#[test]
fn a_shape_with_no_surface_is_refused() {
    // A wire has nothing to print. Writing an empty package would produce a
    // file a slicer opens and silently shows nothing for, so this fails at
    // export instead, and says what is wrong with the shape.
    let ws = Workspace::new("empty");
    let out = ws.path("nothing.3mf");
    let err = ws.run_err("helix(radius: 5, pitch: 2, height: 10).export(OUT)", &out);
    assert!(
        err.contains("no triangles") || err.contains("no surface"),
        "the error should say the shape has nothing to export: {err}"
    );
    assert!(
        !out.exists(),
        "a refused export should not leave a file behind"
    );
}

// ---------------------------------------------------------------------------
// Colour
// ---------------------------------------------------------------------------

#[test]
fn the_shapes_colour_reaches_the_package() {
    // The third thing STL drops. 0.8/0.5/0.2 in sRGB is CC/80/33.
    let ws = Workspace::new("color");
    let out = ws.path("cube.3mf");
    ws.run("box(10, 10, 10).color(0.8, 0.5, 0.2).export(OUT)", &out);

    let xml = model(&out);
    assert!(
        xml.contains("displaycolor=\"#CC8033FF\""),
        "the colour should be written as sRGB hex with alpha: {xml}"
    );
    let object = tags(&xml, "object").first().copied().expect("an object");
    assert_eq!(
        attr(object, "pid"),
        Some("1".to_string()),
        "the object should reference the material group"
    );
}

#[test]
fn an_uncoloured_shape_carries_no_material() {
    // Inventing a default colour would override the slicer's own, which is the
    // one the user actually chose at the printer.
    let ws = Workspace::new("nocolor");
    let out = ws.path("cube.3mf");
    ws.run("box(10, 10, 10).export(OUT)", &out);

    let xml = model(&out);
    assert!(
        !xml.contains("basematerials"),
        "no colour means no material group: {xml}"
    );
    let object = tags(&xml, "object").first().copied().expect("an object");
    assert_eq!(attr(object, "pid"), None, "and no material reference");
}
