// Mesh tessellation quality on `export`.
//
// `linear_deflection:` is the largest gap allowed between the triangle mesh and
// the true curved surface. It was documented for a long time before it worked:
// `Shape#export` never read the option, so every mesh came out at a fixed
// 0.1 mm and a script asking for `0.01` got the same file as one asking for
// `2.0`, with nothing said about it.
//
// That history is why these tests count triangles rather than assert that a
// file was written. A test that only checks the export succeeded would have
// passed throughout the years the option did nothing.

use rrcad::ruby::vm::MrubyVm;
use std::io::Read;
use std::path::{Path, PathBuf};

struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/deflection_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Run `script` with `DIR` substituted for the workspace directory, so a
    /// script can write several files.
    fn run(&self, script: &str) {
        let mut vm = MrubyVm::new();
        vm.eval(&self.substitute(script))
            .expect("export should succeed");
    }

    fn run_err(&self, script: &str) -> String {
        let mut vm = MrubyVm::new();
        match vm.eval(&self.substitute(script)) {
            Ok(v) => panic!("expected the export to be refused, got {v:?}"),
            Err(e) => e.to_string(),
        }
    }

    fn substitute(&self, script: &str) -> String {
        script.replace("DIR", &self.dir.to_string_lossy())
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// How many triangles the exported mesh has.
///
/// 3MF is the format read here because its mesh is plain XML — the count is
/// the thing under test, and no mesh parser sits between the writer and the
/// assertion to be wrong in its own way.
fn triangles(path: &Path) -> usize {
    let file = std::fs::File::open(path).expect("open package");
    let mut zip = zip::ZipArchive::new(file).expect("readable ZIP");
    let mut entry = zip.by_name("3D/3dmodel.model").expect("model part");
    let mut body = String::new();
    entry.read_to_string(&mut body).expect("read model");
    body.matches("<triangle ").count()
}

#[test]
fn a_finer_deflection_produces_a_finer_mesh() {
    // The property that makes the option worth having: it trades file size
    // against how closely the mesh follows the curve.
    let ws = Workspace::new("finer");
    ws.run(
        r#"
        c = cylinder(10, 20)
        c.export("DIR/coarse.3mf", linear_deflection: 2.0)
        c.export("DIR/mid.3mf",    linear_deflection: 0.1)
        c.export("DIR/fine.3mf",   linear_deflection: 0.01)
        "#,
    );

    let coarse = triangles(&ws.path("coarse.3mf"));
    let mid = triangles(&ws.path("mid.3mf"));
    let fine = triangles(&ws.path("fine.3mf"));
    assert!(
        coarse < mid && mid < fine,
        "triangle count should rise as deflection falls: {coarse} / {mid} / {fine}"
    );
}

#[test]
fn the_default_is_what_it_says_it_is() {
    // Omitting the option must give the documented 0.1 mm, not some other
    // value that merely happens to sit between the extremes.
    let ws = Workspace::new("default");
    ws.run(
        r#"
        c = cylinder(10, 20)
        c.export("DIR/implicit.3mf")
        c.export("DIR/explicit.3mf", linear_deflection: 0.1)
        "#,
    );
    assert_eq!(
        triangles(&ws.path("implicit.3mf")),
        triangles(&ws.path("explicit.3mf")),
        "no option should mean exactly linear_deflection: 0.1"
    );
}

#[test]
fn re_exporting_the_same_shape_re_meshes_it() {
    // OCCT keeps the triangulation on the shape and reuses one it considers
    // good enough, so this used to hand back the first mesh whatever was asked
    // for afterwards — the same silent no-op in a second place. The third
    // export returning to the coarse count is the real assertion: it proves
    // the mesh was rebuilt rather than merely refined once and kept.
    let ws = Workspace::new("remesh");
    ws.run(
        r#"
        c = cylinder(10, 20)
        c.export("DIR/first_coarse.3mf",  linear_deflection: 2.0)
        c.export("DIR/then_fine.3mf",     linear_deflection: 0.01)
        c.export("DIR/coarse_again.3mf",  linear_deflection: 2.0)
        "#,
    );

    let first = triangles(&ws.path("first_coarse.3mf"));
    let fine = triangles(&ws.path("then_fine.3mf"));
    let again = triangles(&ws.path("coarse_again.3mf"));
    assert!(
        fine > first,
        "the second export should refine: {first} → {fine}"
    );
    assert_eq!(
        again, first,
        "going back to the coarse deflection should give the coarse mesh again"
    );
}

#[test]
fn every_mesh_format_honours_the_option() {
    // STL, glTF and OBJ go through a different writer from 3MF, and STL
    // through a different one again. All four tessellate, so all four have to
    // read the option — a fix in one exporter is not a fix.
    let ws = Workspace::new("formats");
    ws.run(
        r#"
        c = cylinder(10, 20)
        %w[stl glb obj].each do |ext|
          c.export("DIR/coarse.#{ext}", linear_deflection: 2.0)
          c.export("DIR/fine.#{ext}",   linear_deflection: 0.01)
        end
        "#,
    );

    for ext in ["stl", "glb", "obj"] {
        let coarse = std::fs::metadata(ws.path(&format!("coarse.{ext}")))
            .expect("coarse file")
            .len();
        let fine = std::fs::metadata(ws.path(&format!("fine.{ext}")))
            .expect("fine file")
            .len();
        assert!(
            fine > coarse,
            ".{ext} should grow with a finer mesh: {coarse} → {fine}"
        );
    }
}

#[test]
fn a_flat_shape_meshes_the_same_at_any_quality() {
    // Deflection bounds the gap between the mesh and the true surface, and a
    // plane has no gap at any subdivision. A box coming out finer at a tighter
    // deflection would mean the mesher was subdividing for no reason and
    // charging the file size for it.
    let ws = Workspace::new("flat");
    ws.run(
        r#"
        b = box(10, 20, 30)
        b.export("DIR/coarse.3mf", linear_deflection: 2.0)
        b.export("DIR/fine.3mf",   linear_deflection: 0.001)
        "#,
    );
    assert_eq!(triangles(&ws.path("coarse.3mf")), 12);
    assert_eq!(triangles(&ws.path("fine.3mf")), 12);
}

#[test]
fn the_short_spelling_works_too() {
    // `export_outline` already spells it `deflection:`, and an unrecognised
    // option here is silently dropped — which is the failure being fixed. Both
    // spellings are accepted so a reasonable guess cannot produce a file that
    // quietly ignores it.
    let ws = Workspace::new("alias");
    ws.run(
        r#"
        c = cylinder(10, 20)
        c.export("DIR/long.3mf",  linear_deflection: 0.01)
        c.export("DIR/short.3mf", deflection: 0.01)
        "#,
    );
    assert_eq!(
        triangles(&ws.path("long.3mf")),
        triangles(&ws.path("short.3mf"))
    );
}

#[test]
fn a_deflection_that_cannot_mesh_is_refused() {
    // Zero or negative asks for infinite detail. OCCT does not report that as
    // an error, so catching it here is the difference between a message naming
    // the option and an unexplained failure much further down.
    let ws = Workspace::new("refuse");
    for bad in ["0", "-1.5"] {
        let err = ws.run_err(&format!(
            "cylinder(10, 20).export(\"DIR/x.3mf\", linear_deflection: {bad})"
        ));
        assert!(
            err.contains("linear_deflection") && err.contains("greater than 0"),
            "the error should name the option and the rule, got: {err}"
        );
    }
}

#[test]
fn the_option_is_harmless_on_a_format_that_does_not_mesh() {
    // STEP carries exact geometry and has no mesh to control. Refusing the
    // option there would break the common case of one deflection applied to a
    // whole batch of exports.
    let ws = Workspace::new("step");
    ws.run(r#"box(10, 10, 10).export("DIR/part.step", linear_deflection: 0.01)"#);
    assert!(ws.path("part.step").exists());
}
