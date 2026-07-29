// Binary STL output.
//
// `.stl` is the most-used export in the tool and was the only one still
// written as text. An ASCII facet costs roughly 250 bytes where binary spends
// a fixed 50, so a finely tessellated part arrived several times larger than
// it needed to be for no benefit — every slicer reads binary.
//
// The interesting failure here is not "no file appeared". A binary STL has a
// self-describing structure — an 80-byte header, a little-endian `uint32`
// triangle count, then exactly 50 bytes per triangle — so a writer that
// emitted a plausible-looking file with a wrong count would still open in some
// tools and silently truncate in others. These tests check that structure
// against itself, against the triangle count OCCT produced for the same shape
// at the same deflection (read out of the ASCII encoding, an independent
// route to the same number), and against the solid the mesh came from.

use rrcad::ruby::vm::MrubyVm;
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
            .join(format!("target/stlbin_{tag}"));
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
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// The triangle count a binary STL declares in its header.
fn declared_triangles(bytes: &[u8]) -> u32 {
    assert!(
        bytes.len() >= 84,
        "a binary STL is at least an 80-byte header plus a 4-byte count, got {}",
        bytes.len()
    );
    u32::from_le_bytes(bytes[80..84].try_into().expect("4 bytes"))
}

/// The triangle count implied by the file's length.
fn triangles_from_length(bytes: &[u8]) -> f64 {
    (bytes.len() as f64 - 84.0) / 50.0
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).expect("read the exported STL")
}

#[test]
fn the_default_encoding_is_binary_and_self_consistent() {
    // The structural claim: header, count, and file length must agree exactly.
    // A writer that emitted the right triangles with a stale count would pass a
    // "file exists" check and fail here.
    let ws = Workspace::new("default");
    let out = ws.path("part.stl");
    ws.run("cylinder(20, 40).fillet(2).export(OUT)", &out);

    let bytes = read(&out);
    let declared = declared_triangles(&bytes);
    assert!(declared > 0, "the mesh should not be empty");
    assert_eq!(
        declared as f64,
        triangles_from_length(&bytes),
        "the declared count must match the file length exactly \
         (declared {declared}, file is {} bytes)",
        bytes.len()
    );
}

#[test]
fn the_binary_header_does_not_begin_with_solid() {
    // The real interop hazard. The two encodings are told apart by sniffing the
    // first bytes for "solid", so a binary file whose header happened to start
    // that way would be parsed as text and rejected as corrupt. OCCT writes its
    // own banner; this pins that it stays out of the way.
    let ws = Workspace::new("sniff");
    let out = ws.path("part.stl");
    ws.run("box(10, 20, 30).export(OUT)", &out);

    let bytes = read(&out);
    assert!(
        !bytes.starts_with(b"solid"),
        "a binary STL must not open with the ASCII keyword: {:?}",
        String::from_utf8_lossy(&bytes[..16.min(bytes.len())])
    );
}

#[test]
fn ascii_true_opts_back_into_text() {
    // The escape hatch has to actually produce the other format, not merely be
    // accepted and ignored — which is exactly how `linear_deflection:` failed
    // before it was fixed.
    let ws = Workspace::new("ascii");
    let out = ws.path("part.stl");
    ws.run("box(10, 20, 30).export(OUT, ascii: true)", &out);

    let text = String::from_utf8(read(&out)).expect("ASCII STL should be valid UTF-8");
    assert!(
        text.starts_with("solid"),
        "an ASCII STL opens with `solid`: {:?}",
        &text[..20.min(text.len())]
    );
    assert!(
        text.contains("facet normal"),
        "an ASCII STL spells out each facet"
    );
}

#[test]
fn both_encodings_describe_the_same_mesh() {
    // The independent oracle: the number of triangles is a property of the
    // tessellation, not of how it is written down. Counting `facet normal` in
    // the text file reaches that number by a completely different route than
    // reading the binary header, so the two agreeing means the binary count is
    // right rather than merely self-consistent.
    let ws = Workspace::new("agree");
    let binary = ws.path("b.stl");
    let ascii = ws.path("a.stl");
    ws.run(
        "cylinder(20, 40).fillet(2).export(OUT, linear_deflection: 0.05)",
        &binary,
    );
    ws.run(
        "cylinder(20, 40).fillet(2).export(OUT, linear_deflection: 0.05, ascii: true)",
        &ascii,
    );

    let declared = declared_triangles(&read(&binary));
    let facets = String::from_utf8(read(&ascii))
        .expect("ASCII STL is UTF-8")
        .matches("facet normal")
        .count();
    assert_eq!(
        declared as usize, facets,
        "the binary header should declare the same triangle count the text spells out"
    );
}

#[test]
fn binary_is_several_times_smaller_than_ascii() {
    // The reason for the change. The exact ratio depends on how many digits
    // each coordinate needs, so this asserts the order of the win rather than a
    // brittle number — but a regression to text-by-default would collapse it
    // to 1.0 and fail.
    let ws = Workspace::new("size");
    let binary = ws.path("b.stl");
    let ascii = ws.path("a.stl");
    ws.run(
        "cylinder(20, 40).fillet(2).export(OUT, linear_deflection: 0.02)",
        &binary,
    );
    ws.run(
        "cylinder(20, 40).fillet(2).export(OUT, linear_deflection: 0.02, ascii: true)",
        &ascii,
    );

    let ratio = read(&ascii).len() as f64 / read(&binary).len() as f64;
    assert!(
        ratio > 4.0,
        "binary should be several times smaller than ASCII, got {ratio:.2}x"
    );
}

#[test]
fn a_binary_stl_reads_back_as_the_solid_it_came_from() {
    // Structure is not enough: the numbers inside have to be the geometry. A
    // box has flat faces, so its mesh is exact and the round-tripped volume
    // must match the source solid rather than merely come close.
    let ws = Workspace::new("roundtrip");
    let out = ws.path("part.stl");
    let literal = format!("{:?}", out.to_string_lossy());
    let mut vm = MrubyVm::new();
    let volume = vm
        .eval(&format!(
            "box(10, 20, 30).export({literal})\n\
             import_stl({literal}).volume"
        ))
        .expect("export and re-import should succeed");
    let volume: f64 = volume.trim().parse().expect("a number");
    assert!(
        (volume - 6000.0).abs() < 1e-6,
        "expected the box volume back, got {volume}"
    );
}

#[test]
fn deflection_still_controls_the_binary_mesh() {
    // The encoding change must not have cost the quality option: a finer
    // deflection has to produce more triangles in the binary header too.
    let ws = Workspace::new("deflection");
    let coarse = ws.path("coarse.stl");
    let fine = ws.path("fine.stl");
    ws.run(
        "cylinder(20, 40).export(OUT, linear_deflection: 0.5)",
        &coarse,
    );
    ws.run(
        "cylinder(20, 40).export(OUT, linear_deflection: 0.01)",
        &fine,
    );

    let coarse_n = declared_triangles(&read(&coarse));
    let fine_n = declared_triangles(&read(&fine));
    assert!(
        fine_n > coarse_n,
        "a finer deflection should mesh more finely: {coarse_n} -> {fine_n}"
    );
}
