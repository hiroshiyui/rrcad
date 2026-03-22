use std::path::{Path, PathBuf};

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Path-traversal guard (CLI paths)
// ---------------------------------------------------------------------------

/// Resolve `raw` to a canonical absolute path and verify it is inside the
/// current working directory.
///
/// Security rationale: the CLI accepts a script filename from the command
/// line.  An attacker (or misconfigured invocation) could pass a path like
/// `"../../secret.rb"`.  This helper rejects any path that resolves outside
/// the process working directory, blocking directory-traversal attacks.
///
/// For files that do not yet exist (e.g. export targets inside a design-table
/// script), the parent directory is canonicalized and the filename re-joined.
fn safe_path(raw: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(raw);
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let canon_cwd = cwd
        .canonicalize()
        .map_err(|e| format!("cannot resolve cwd: {e}"))?;

    let canonical = if p.exists() {
        p.canonicalize()
            .map_err(|e| format!("cannot resolve path '{raw}': {e}"))?
    } else {
        let parent = p.parent().unwrap_or(Path::new(""));
        let canon_parent = if parent == Path::new("") {
            canon_cwd.clone()
        } else {
            parent
                .canonicalize()
                .map_err(|e| format!("cannot resolve directory for '{raw}': {e}"))?
        };
        canon_parent.join(
            p.file_name()
                .ok_or_else(|| format!("invalid path (no filename component): '{raw}'"))?,
        )
    };

    if !canonical.starts_with(&canon_cwd) {
        return Err(format!(
            "path '{raw}' is outside the working directory (path traversal rejected)"
        ));
    }
    Ok(canonical)
}
use rustyline::{
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    Context, Editor, Helper,
};

// ---------------------------------------------------------------------------
// Help text
// ---------------------------------------------------------------------------

const HELP_TEXT: &str = "\
rrcad DSL — quick reference
═══════════════════════════════════════════════════════════
Primitives (3D solids)
  box(dx, dy, dz)           rectangular solid
  cylinder(r, h)            cylinder (Z-axis)
  sphere(r)                 sphere

Sketch faces (2D, for extrude/revolve)
  rect(w, h)                rectangular face in XY plane
  circle(r)                 circular face in XY plane
  spline_2d([[r,z], ...])   closed profile in XZ plane (for revolve)
  spline_3d([[x,y,z], ...]) 3D wire path (for sweep)

Transforms                   (return a new Shape)
  s.translate(x, y, z)      move
  s.rotate(ax, ay, az, deg) rotate around axis by degrees
  s.scale(factor)           uniform scale (all axes)
  s.scale(sx, sy, sz)       non-uniform scale
  s.mirror(:xy|:xz|:yz)     mirror about a plane

Color
  s.color(r, g, b)          tag shape with sRGB color (each 0.0–1.0)
                            written into GLB / glTF / OBJ on export

Assembly mating
  s.mate(from_face, to_face)          reposition s flush against to_face
  s.mate(from_face, to_face, offset)  same with a gap (offset > 0)
  a.mate s, from: f1, to: f2         mate + add to assembly (keyword form)

Modifiers
  s.fillet(r[, :sel])       round all (or selected) edges
  s.chamfer(d[, :sel])      bevel all (or selected) edges (symmetric)
  s.chamfer_asym(d1,d2[,:sel]) asymmetric chamfer (two bevel distances)
                            sel: :all, :vertical, :horizontal
  s.extrude(h)              extrude face/profile by height
  s.revolve(deg=360)        revolve around Z axis
  s.sweep(path)             sweep profile along 3D wire path

Boolean operations           (return a new Shape)
  a.fuse(b)                 union of a and b
  a.cut(b)                  subtract b from a
  a.common(b)               intersection of a and b

Sub-shape selectors          (return an Array of Shapes)
  s.faces(:top|:bottom|:side|:all)
  s.faces(\">Z\"|\"<X\"|...)   direction-based (CadQuery style)
  s.edges(:vertical|:horizontal|:all)
  s.vertices(:all)          all unique vertices

Export
  shape.export(\"out.step\")             write STEP / STL / GLB / OBJ (by extension)
  shape.export(\"out.svg\")              2-D SVG drawing, top view (HLR projection)
  shape.export(\"out.svg\", view: :front) front or side view
  shape.export(\"out.dxf\")              DXF R12 drawing (same view options)

Patterns
  linear_pattern(s,n,[dx,dy,dz]) n copies translated along vector
  polar_pattern(s, n, angle_deg) n copies rotated around Z axis
  grid_pattern(s,nx,ny,dx,dy)    nx×ny copies in a 2-D grid

Boolean multi-shape
  fuse_all([a, b, c])       fold-left union of 2+ shapes
  cut_all(base, [t1, t2])   subtract each tool from base in sequence

2D profile
  s.offset_2d(d)            offset a Wire/Face inward (<0) or outward (>0)

Validation & introspection
  s.shape_type              → :solid/:shell/:face/:wire/:edge/:vertex (Symbol)
  s.centroid                → [x, y, z] centre of mass
  s.closed?                 → true if all edges have ≥2 adjacent faces
  s.manifold?               → true if all edges have exactly 2 adjacent faces
  s.validate                → :ok  or  [\"error1\", ...]

Surface modeling
  ruled_surface(a, b)       ruled surface (shell) between two wires
  fill_surface(wire)        smooth NURBS surface filling a closed wire
  s.slice(plane: :xy, z: d) cross-section by XY plane at z=d (also :xz/:yz)

Part Design
  s.pad(face_sel, height: h) { sk }   extrude sketch on face, fuse with s
  s.pocket(face_sel, depth: d) { sk } cut sketch pocket from s
  s.fillet_wire(r)           round corners of a 2D Wire/Face profile
  s.extrude(h, draft: a)     extrude with draft angle a (degrees, tapers top)
  datum_plane(origin: [x,y,z], normal: [nx,ny,nz], x_dir: [xx,xy,xz])
                             finite reference plane (Face) for design ops
  helix(radius: r, pitch: p, height: h)   helical Wire path (for thread sweep)
  thread(solid, :side, pitch: p, depth: d) cut helical thread groove into solid
  cbore(d:, cbore_d:, cbore_h:, depth:)  counterbore hole tool (use with .cut)
  csink(d:, csink_d:, csink_angle:, depth:) countersink tool (use with .cut)
  s.distance_to(other)       minimum distance between shapes (0 if overlapping)
  s.inertia                  {ixx:,iyy:,izz:,ixy:,ixz:,iyz:} inertia tensor
  s.min_thickness            minimum wall thickness of a solid/shell

Parameters
  param :name, default: val        declare a parameter (returns value)
  param :name, default: val,       same, with range validation
       range: lo..hi
  # Override at the command line:
  #   rrcad --param name=value script.rb

Design table (batch export)
  rrcad --design-table table.csv script.rb
  # CSV first row = column headers (param names).
  # Optional 'name' column → used as output filename stem.
  # Remaining columns map to param() declarations in the script.
  # Each data row evals the script once with those param values.

Builders
  solid do ... end          block returning last shape
  assembly(\"name\") do |a|
    a.place shape           add shape to assembly
  end

REPL controls
  help                      show this message
  exit  /  quit  /  Ctrl-D  leave the REPL
═══════════════════════════════════════════════════════════";

// ---------------------------------------------------------------------------
// Tab-completion helper
// ---------------------------------------------------------------------------

/// Top-level identifiers available in the rrcad DSL REPL.
const TOP_LEVEL: &[&str] = &[
    // DSL primitives
    "box",
    "cylinder",
    "sphere", // DSL sketch faces
    "rect",
    "circle",
    "spline_2d",
    "spline_3d", // DSL builders
    "solid",
    "assembly",
    "preview",
    "linear_pattern",
    "polar_pattern",
    "grid_pattern",
    "fuse_all",
    "cut_all",
    "ruled_surface",
    "fill_surface",
    "datum_plane",
    "helix",
    "thread",
    "cbore",
    "csink",
    "param", // REPL control
    "help",
    "exit",
    "quit", // Ruby keywords
    "do",
    "end",
    "if",
    "else",
    "elsif",
    "unless",
    "while",
    "until",
    "for",
    "def",
    "class",
    "module",
    "return",
    "nil",
    "true",
    "false",
    "puts",
    "p",
    "pp",
    "raise",
    "begin",
    "rescue",
];

/// Methods available on Shape objects.
const SHAPE_METHODS: &[&str] = &[
    // Phase 1 — native
    "export",
    "fuse",
    "cut",
    "common",
    // Phase 2 — native
    "translate",
    "rotate",
    "scale",
    "fillet",
    "chamfer",
    "mirror",
    "extrude",
    "revolve",
    // Phase 3 — native
    "sweep",
    // Phase 3+ — sub-shape selectors
    "faces",
    "edges",
    "vertices",
    // Phase 4 — 3-D ops and Tier 4 quality
    "shell",
    "offset",
    "offset_2d",
    "simplify",
    // Phase 7 Tier 1
    "chamfer_asym",
    // Phase 7 Tier 2 — validation & introspection
    "shape_type",
    "centroid",
    "closed?",
    "manifold?",
    "validate",
    // Phase 7 Tier 3 — surface modeling
    "slice",
    // Phase 8 Tier 1 — Core Part Design
    "pad",
    "pocket",
    "fillet_wire",
    // Phase 5 — color and mating
    "color",
    "mate",
    // Ruby built-ins
    "class",
    "inspect",
    "to_s",
    "nil?",
    "is_a?",
    "respond_to?",
];

struct DslHelper;

impl Completer for DslHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find the start of the word being typed (letters, digits, _, ?, !).
        let word_start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '?' && c != '!')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[word_start..pos];

        // Decide candidate set: method names after '.', top-level otherwise.
        let is_method = word_start > 0 && line[..word_start].trim_end().ends_with('.');
        let candidates: &[&str] = if is_method { SHAPE_METHODS } else { TOP_LEVEL };

        let matches = candidates
            .iter()
            .filter(|&&w| w.starts_with(prefix))
            .map(|&w| Pair {
                display: w.to_owned(),
                replacement: w.to_owned(),
            })
            .collect();

        Ok((word_start, matches))
    }
}

// No-op implementations for the remaining Helper sub-traits.
impl Hinter for DslHelper {
    type Hint = String;
}
impl Highlighter for DslHelper {}
impl Validator for DslHelper {}
impl Helper for DslHelper {}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

enum Mode {
    Repl,
    Script(String),
    Preview(String),
    DesignTable { table: String, script: String },
}

struct CliArgs {
    mode: Mode,
    /// Key-value pairs from --param key=value flags.
    params: Vec<(String, String)>,
}

/// Parse command-line arguments, extracting any number of `--param key=value`
/// flags (which may appear in any position) and the run mode.
///
/// Usage:
///   rrcad                                            # REPL
///   rrcad --repl                                     # REPL (explicit)
///   rrcad [--param k=v]... <script.rb>               # run script
///   rrcad --preview [--param k=v]... <script.rb>     # live preview
///   rrcad --design-table table.csv <script.rb>       # batch export
fn parse_args() -> CliArgs {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut params: Vec<(String, String)> = Vec::new();
    // Non-param args that determine the run mode.
    let mut rest: Vec<String> = Vec::new();

    let mut i = 0;
    while i < raw.len() {
        if raw[i] == "--param" {
            i += 1;
            if i >= raw.len() {
                eprintln!("error: --param requires a key=value argument");
                std::process::exit(1);
            }
            match raw[i].split_once('=') {
                Some((k, v)) => params.push((k.to_string(), v.to_string())),
                None => {
                    eprintln!("error: --param requires key=value format, got: {}", raw[i]);
                    std::process::exit(1);
                }
            }
        } else {
            rest.push(raw[i].clone());
        }
        i += 1;
    }

    let mode = match rest.first().map(String::as_str) {
        None | Some("--repl") => Mode::Repl,
        Some("--preview") => match rest.get(1) {
            Some(path) => Mode::Preview(path.clone()),
            None => {
                eprintln!("usage: rrcad --preview [--param key=val]... <script.rb>");
                std::process::exit(1);
            }
        },
        Some("--design-table") => match (rest.get(1), rest.get(2)) {
            (Some(table), Some(script)) => Mode::DesignTable {
                table: table.clone(),
                script: script.clone(),
            },
            _ => {
                eprintln!("usage: rrcad --design-table <table.csv> [--param k=v]... <script.rb>");
                std::process::exit(1);
            }
        },
        Some(path) => Mode::Script(path.to_string()),
    };

    CliArgs { mode, params }
}

fn main() {
    let CliArgs { mode, params } = parse_args();

    match mode {
        Mode::Repl => run_repl(),
        Mode::Script(path) => run_script(&path, &params),
        Mode::Preview(path) => run_preview(&path, &params),
        Mode::DesignTable { table, script } => {
            if let Err(e) = run_design_table(&table, &script, &params) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}

fn run_repl() {
    println!("rrcad {} — mRuby interpreter", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or press Ctrl-D to quit.\n");

    let mut vm = MrubyVm::new();
    let mut rl = Editor::<DslHelper, _>::with_history(
        rustyline::Config::default(),
        rustyline::history::DefaultHistory::new(),
    )
    .expect("failed to initialise readline");
    rl.set_helper(Some(DslHelper));

    loop {
        match rl.readline("rrcad> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);
                if line == "help" {
                    println!("{HELP_TEXT}");
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }
                match vm.eval(line) {
                    Ok(result) => println!("=> {result}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("readline error: {e}");
                break;
            }
        }
    }
}

fn run_script(path: &str, params: &[(String, String)]) {
    // Reject script paths that escape the working directory.
    let safe = match safe_path(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let code = match std::fs::read_to_string(&safe) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", safe.display());
            std::process::exit(1);
        }
    };
    let mut vm = MrubyVm::new();
    if let Err(e) = vm.set_params(params) {
        eprintln!("{path}: error setting params: {e}");
        std::process::exit(1);
    }
    if let Err(e) = vm.eval(&code) {
        eprintln!("{path}: {e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Design table
// ---------------------------------------------------------------------------

/// Parse a CSV or TSV file into a `Vec` of rows, each row a `Vec<String>`.
///
/// Rules:
/// - Lines that are empty or start with `#` are skipped (comments).
/// - Delimiter is auto-detected: tab if the first non-comment line contains
///   a tab character, otherwise comma.
/// - Fields are trimmed of surrounding whitespace.
/// - The first row is the header; subsequent rows are data.
///
/// Returns `Err` if the file has no header row or data rows.
fn parse_csv(content: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut lines = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));

    let header_line = lines
        .next()
        .ok_or("design table is empty (no header row)")?;

    let delim = if header_line.contains('\t') {
        '\t'
    } else {
        ','
    };
    let split =
        |line: &str| -> Vec<String> { line.split(delim).map(|f| f.trim().to_string()).collect() };

    let headers = split(header_line);
    let rows: Vec<Vec<String>> = lines.map(split).collect();

    if rows.is_empty() {
        return Err("design table has a header but no data rows".to_string());
    }

    Ok((headers, rows))
}

/// Run `script_path` once for every data row in `table_path`.
///
/// For each row the columns are merged with `base_params` (row values win on
/// conflict) and injected into a fresh `MrubyVm` via `set_params`.  The
/// optional `name` column determines the label used in progress output; the
/// script itself decides what to export and where.
///
/// Prints a per-row status line and a final summary.  Returns `Err` if any
/// row fails; all rows are always attempted regardless.
fn run_design_table(
    table_path: &str,
    script_path: &str,
    base_params: &[(String, String)],
) -> Result<(), String> {
    // Reject table and script paths that escape the working directory.
    let safe_table = safe_path(table_path)?;
    let safe_script = safe_path(script_path)?;

    let table_src = std::fs::read_to_string(&safe_table)
        .map_err(|e| format!("error: could not read '{table_path}': {e}"))?;
    let code = std::fs::read_to_string(&safe_script)
        .map_err(|e| format!("error: could not read '{script_path}': {e}"))?;

    let (headers, rows) = parse_csv(&table_src)?;
    let total = rows.len();
    println!(
        "Design table: {table_path} → {total} row{}",
        if total == 1 { "" } else { "s" }
    );

    let mut errors: usize = 0;

    for (i, row) in rows.iter().enumerate() {
        // Start with base_params then let row columns override.
        let mut params: Vec<(String, String)> = base_params.to_vec();
        for (col, val) in headers.iter().zip(row.iter()) {
            if let Some(entry) = params.iter_mut().find(|(k, _)| k == col) {
                entry.1 = val.clone();
            } else {
                params.push((col.clone(), val.clone()));
            }
        }

        // Use the `name` column as a human-readable label if present.
        let label = params
            .iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| format!("row_{:03}", i + 1));

        let mut vm = MrubyVm::new();
        match vm.set_params(&params).and_then(|_| vm.eval(&code)) {
            Ok(_) => println!("  [{}/{}] {} → ok", i + 1, total, label),
            Err(e) => {
                eprintln!("  [{}/{}] {} → error: {}", i + 1, total, label, e);
                errors += 1;
            }
        }
    }

    let ok = total - errors;
    println!("\n{ok} succeeded, {errors} failed");

    if errors > 0 {
        Err(format!("{errors} row(s) failed"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod design_table_tests {
    use super::parse_csv;

    #[test]
    fn parse_csv_basic() {
        let (headers, rows) = parse_csv("name,width,height\nsmall,50,20\nlarge,100,40").unwrap();
        assert_eq!(headers, vec!["name", "width", "height"]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["small", "50", "20"]);
        assert_eq!(rows[1], vec!["large", "100", "40"]);
    }

    #[test]
    fn parse_csv_skips_comments_and_blank_lines() {
        let src = "# generated\nname,w\n\n# skip\npart_a,10\npart_b,20\n";
        let (headers, rows) = parse_csv(src).unwrap();
        assert_eq!(headers, vec!["name", "w"]);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn parse_csv_trims_whitespace() {
        let (headers, rows) = parse_csv(" name , w \n part_a , 10 ").unwrap();
        assert_eq!(headers, vec!["name", "w"]);
        assert_eq!(rows[0], vec!["part_a", "10"]);
    }

    #[test]
    fn parse_tsv_auto_detected() {
        let (headers, rows) = parse_csv("name\tw\npart_a\t10").unwrap();
        assert_eq!(headers, vec!["name", "w"]);
        assert_eq!(rows[0], vec!["part_a", "10"]);
    }

    #[test]
    fn parse_csv_empty_returns_error() {
        assert!(parse_csv("").is_err());
        assert!(parse_csv("# only a comment\n").is_err());
    }

    #[test]
    fn parse_csv_header_only_returns_error() {
        assert!(parse_csv("name,width\n").is_err());
    }
}

// ---------------------------------------------------------------------------
// safe_path security tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod safe_path_tests {
    use super::safe_path;

    #[test]
    fn safe_path_accepts_simple_filename() {
        // A bare filename resolves relative to cwd — always safe.
        assert!(safe_path("output.step").is_ok());
    }

    #[test]
    fn safe_path_accepts_subdirectory() {
        // A path whose parent exists and is inside cwd is accepted.
        // Use "src/dummy.rb" — src/ exists in every build tree.
        assert!(safe_path("src/dummy.rb").is_ok());
    }

    #[test]
    fn safe_path_rejects_dotdot_traversal() {
        // ../../etc/passwd must be rejected: the resolved path escapes cwd.
        let err = safe_path("../../etc/passwd");
        assert!(err.is_err(), "path traversal via ../../ should be rejected");
    }

    #[test]
    fn safe_path_rejects_single_dotdot() {
        // ../escape.rb resolves one level above cwd — must be rejected.
        let err = safe_path("../escape.rb");
        assert!(err.is_err(), "path traversal via ../ should be rejected");
    }

    #[test]
    fn safe_path_rejects_absolute_path_outside_cwd() {
        // An absolute path pointing outside cwd must always be rejected.
        let err = safe_path("/etc/passwd");
        assert!(err.is_err(), "absolute path outside cwd should be rejected");
    }
}

/// Generate a randomised path for the temporary preview GLB file.
///
/// Security rationale: a hardcoded, predictable path like `/tmp/rrcad_preview.glb`
/// is vulnerable to symlink attacks — a local attacker can create the file (or a
/// symlink pointing at a sensitive target) before the process does, causing rrcad
/// to overwrite arbitrary files.  Mixing the process ID with a hash of the current
/// time makes the filename hard to predict even if the attacker can observe the PID.
fn make_preview_glb_path() -> std::path::PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    std::process::id().hash(&mut h);
    let token = h.finish();
    std::env::temp_dir().join(format!("rrcad_preview_{token:016x}.glb"))
}

fn run_preview(script_path: &str, params: &[(String, String)]) {
    use notify::{RecursiveMode, Watcher};
    use rrcad::preview;

    // Reject script paths that escape the working directory.
    if let Err(e) = safe_path(script_path) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    // Use a randomised temp-file name to prevent symlink attacks (Fix 3).
    let glb_path = make_preview_glb_path();
    // Keep a copy so we can delete the file when the process exits.
    let glb_path_for_cleanup = glb_path.clone();
    let _rt = preview::start(glb_path, 3000);

    // Helper: read and eval the script, reporting errors to stderr.
    // Each eval creates a fresh VM; params are re-injected every time so that
    // live-reload picks up the same overrides as the initial run.
    let eval_script = |path: &str| match std::fs::read_to_string(path) {
        Ok(code) => {
            let mut vm = MrubyVm::new();
            if let Err(e) = vm.set_params(params) {
                eprintln!("{path}: error setting params: {e}");
            } else if let Err(e) = vm.eval(&code) {
                eprintln!("{path}: {e}");
            }
        }
        Err(e) => eprintln!("error: could not read '{path}': {e}"),
    };

    // Initial eval.
    eval_script(script_path);

    // Watch the script file; re-eval on every change.
    //
    // We watch the *parent directory* rather than the file itself to handle
    // atomic-write editors (write temp → rename into place).  inotify tracks
    // inodes: a rename replaces the inode and the file-level watch goes silent.
    // A directory-level watch fires Create/Rename events for any file in the
    // directory, so we filter by the canonical script path.
    let canonical_script = std::fs::canonicalize(script_path)
        .unwrap_or_else(|_| std::path::PathBuf::from(script_path));
    let watch_dir = canonical_script
        .parent()
        .expect("script path has no parent directory")
        .to_path_buf();

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).ok();
    })
    .expect("failed to create file watcher");
    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .expect("failed to watch script directory");

    println!("Watching {script_path} for changes…");

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Filter: only react when the event involves our script file.
                let affects_script = event.paths.iter().any(|p| {
                    std::fs::canonicalize(p)
                        .map(|c| c == canonical_script)
                        .unwrap_or(false)
                });
                if !affects_script {
                    continue;
                }
                // Debounce: drain any further events that arrive within 50 ms.
                while rx
                    .recv_timeout(std::time::Duration::from_millis(50))
                    .is_ok()
                {}
                eval_script(script_path);
            }
            Ok(Err(e)) => eprintln!("watch error: {e}"),
            Err(_) => break,
        }
    }

    // Best-effort cleanup: remove the randomised temp GLB file so it does not
    // accumulate in /tmp across restarts.  Errors are silently ignored.
    std::fs::remove_file(&glb_path_for_cleanup).ok();
}
