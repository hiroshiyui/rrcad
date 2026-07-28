use rrcad::{project_config, ruby::vm::MrubyVm};
use rustyline::{
    Context, Editor, Helper,
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
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
  csink(d:, csink_d:, csink_angle:, depth:) countersink hole tool (use with .cut)
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
    "fillet_var",
    // Phase 7 Tier 2 — validation & introspection
    "shape_type",
    "centroid",
    "closed?",
    "manifold?",
    "validate",
    "bounding_box",
    "volume",
    "surface_area",
    "distance_to",
    "inertia",
    "min_thickness",
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

#[derive(Debug)]
enum Mode {
    Repl,
    Script(String),
    Preview {
        path: String,
        port: Option<u16>,
    },
    DesignTable {
        table: String,
        script: String,
    },
    /// MCP server mode: serve CAD tools over stdio to an AI client.
    Mcp,
    /// Hidden helper used by MCP mode to run one killable tool operation.
    McpWorker(String),
}

#[derive(Debug)]
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
///   rrcad --preview [--preview-port N] [--param k=v]... <script.rb>  # live preview
///   rrcad --design-table table.csv <script.rb>       # batch export
fn parse_args() -> CliArgs {
    match parse_args_from(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

fn parse_args_from<I>(raw: I) -> Result<CliArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let raw: Vec<String> = raw.into_iter().collect();
    let mut params: Vec<(String, String)> = Vec::new();
    let mut preview_port: Option<u16> = None;
    // Non-param args that determine the run mode.
    let mut rest: Vec<String> = Vec::new();

    let mut i = 0;
    while i < raw.len() {
        if raw[i] == "--param" {
            i += 1;
            if i >= raw.len() {
                return Err("error: --param requires a key=value argument".to_string());
            }
            match raw[i].split_once('=') {
                Some((k, v)) => params.push((k.to_string(), v.to_string())),
                None => {
                    return Err(format!(
                        "error: --param requires key=value format, got: {}",
                        raw[i]
                    ));
                }
            }
        } else if raw[i] == "--preview-port" {
            i += 1;
            if i >= raw.len() {
                return Err("error: --preview-port requires a port number".to_string());
            }
            preview_port = Some(
                raw[i]
                    .parse::<u16>()
                    .map_err(|_| format!("error: invalid --preview-port value: {}", raw[i]))?,
            );
        } else {
            rest.push(raw[i].clone());
        }
        i += 1;
    }

    let mode = match rest.first().map(String::as_str) {
        None | Some("--repl") => Mode::Repl,
        Some("--mcp") => Mode::Mcp,
        Some("--mcp-worker") => match rest.get(1) {
            Some(kind) => Mode::McpWorker(kind.clone()),
            None => {
                return Err("usage: rrcad --mcp-worker <eval|export|preview|validate>".to_string());
            }
        },
        Some("--preview") => match rest.get(1) {
            Some(path) => Mode::Preview {
                path: path.clone(),
                port: preview_port,
            },
            None => {
                return Err(
                    "usage: rrcad --preview [--preview-port N] [--param key=val]... <script.rb>"
                        .to_string(),
                );
            }
        },
        Some("--design-table") => match (rest.get(1), rest.get(2)) {
            (Some(table), Some(script)) => Mode::DesignTable {
                table: table.clone(),
                script: script.clone(),
            },
            _ => {
                return Err(
                    "usage: rrcad --design-table <table.csv> [--param k=v]... <script.rb>"
                        .to_string(),
                );
            }
        },
        Some(path) => Mode::Script(path.to_string()),
    };

    // --preview-port only makes sense with --preview; silently ignoring it in
    // other modes would hide user mistakes, so make it a hard error instead.
    if preview_port.is_some() && !matches!(mode, Mode::Preview { .. }) {
        return Err("error: --preview-port requires --preview".to_string());
    }

    Ok(CliArgs { mode, params })
}

pub fn run() {
    let CliArgs { mode, params } = parse_args();

    match mode {
        Mode::Repl => run_repl(),
        Mode::Script(path) => run_script(&path, &params),
        Mode::Preview { path, port } => run_preview(&path, &params, port),
        Mode::DesignTable { table, script } => {
            if let Err(e) = run_design_table(&table, &script, &params) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        Mode::Mcp => {
            if let Err(e) = rrcad::mcp::start() {
                eprintln!("rrcad MCP server error: {e}");
                std::process::exit(1);
            }
        }
        Mode::McpWorker(kind) => std::process::exit(rrcad::mcp::run_worker(&kind)),
    }
}

fn run_repl() {
    println!("rrcad {} — mRuby interpreter", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or press Ctrl-D to quit.\n");

    let project = load_project_config_for_cwd().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let mut vm = MrubyVm::new();
    if let Err(e) = vm.set_params(&project.params) {
        eprintln!("error loading project config params: {e}");
        std::process::exit(1);
    }
    let mut rl = Editor::<DslHelper, _>::with_history(
        rustyline::Config::default(),
        rustyline::history::DefaultHistory::new(),
    )
    .unwrap_or_else(|e| {
        eprintln!("error: failed to initialise readline: {e}");
        std::process::exit(1);
    });
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
    // The CLI input script may live anywhere the user can read — no CWD restriction.
    // Export paths produced *inside* the script are still guarded by safe_path in native.rs.
    let project = load_project_config_for_script(path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(1);
    });
    let code = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{path}': {e}");
            std::process::exit(1);
        }
    };
    let mut vm = MrubyVm::new();
    let effective_params = merge_params(&project.params, params);
    if let Err(e) = vm.set_params(&effective_params) {
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
    // CLI input files may live anywhere the user can read — no CWD restriction.
    // Export paths produced *inside* the script are still guarded by safe_path in native.rs.
    let project = load_project_config_for_script(script_path)?;
    let table_src = std::fs::read_to_string(table_path)
        .map_err(|e| format!("error: could not read '{table_path}': {e}"))?;
    let code = std::fs::read_to_string(script_path)
        .map_err(|e| format!("error: could not read '{script_path}': {e}"))?;

    let (headers, rows) = parse_csv(&table_src)?;
    let total = rows.len();
    println!(
        "Design table: {table_path} → {total} row{}",
        if total == 1 { "" } else { "s" }
    );

    let mut errors: usize = 0;
    let effective_base_params = merge_params(&project.params, base_params);

    for (i, row) in rows.iter().enumerate() {
        // Start with base_params then let row columns override.
        let mut params: Vec<(String, String)> = effective_base_params.clone();
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

/// Generate a hard-to-guess path for the temporary preview GLB file.
///
/// Security rationale: a hardcoded, predictable path like `/tmp/rrcad_preview.glb`
/// is vulnerable to symlink attacks — a local attacker can create the file (or a
/// symlink pointing at a sensitive target) before the process does, causing rrcad
/// to overwrite arbitrary files.  Uses a v4 UUID (122 bits of OS-CSPRNG entropy)
/// so the filename cannot be predicted from PID + approximate launch time.
fn make_preview_glb_path() -> std::path::PathBuf {
    let token = uuid::Uuid::new_v4().simple().to_string();
    std::env::temp_dir().join(format!("rrcad_preview_{token}.glb"))
}

/// Resolve the preview port from CLI flag and project config.
///
/// Precedence: `--preview-port` flag > `preview_port` in `rrcad.toml` > `0`
/// (which asks the OS to auto-select a free port).
fn effective_preview_port(cli_port: Option<u16>, config_port: Option<u16>) -> u16 {
    cli_port.or(config_port).unwrap_or(0)
}

fn run_preview(script_path: &str, params: &[(String, String)], port: Option<u16>) {
    use rrcad::preview;

    // The CLI input script may live anywhere the user can read — no CWD restriction.
    // Export paths produced *inside* the script are still guarded by safe_path in native.rs.
    let project = load_project_config_for_script(script_path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}", path = script_path);
        std::process::exit(1);
    });

    // Use a randomised temp-file name to prevent symlink attacks (Fix 3).
    let glb_path = make_preview_glb_path();
    // Keep a copy so we can delete the file when the process exits.
    let glb_path_for_cleanup = glb_path.clone();
    let effective_params = merge_params(&project.params, params);
    let preview_port = effective_preview_port(port, project.preview_port);
    let _rt = match preview::start(glb_path, preview_port) {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to start preview server: {e}");
            std::process::exit(1);
        }
    };

    // Helper: read and eval the script, reporting errors to stderr.
    // Each eval creates a fresh VM; params are re-injected every time so that
    // live-reload picks up the same overrides as the initial run.
    let eval_script = |path: &str| match std::fs::read_to_string(path) {
        Ok(code) => {
            let mut vm = MrubyVm::new();
            if let Err(e) = vm.set_params(&effective_params) {
                eprintln!("{path}: error setting params: {e}");
            } else if let Err(e) = vm.eval(&code) {
                eprintln!("{path}: {e}");
            }
        }
        Err(e) => eprintln!("error: could not read '{path}': {e}"),
    };

    // Initial eval.
    eval_script(script_path);

    // Delegate to the file-watcher loop; it returns when the watcher channel
    // closes (normally never — the process exits via Ctrl-C).
    watch_script_loop(script_path, &eval_script);

    // Best-effort cleanup: remove the randomised temp GLB file so it does not
    // accumulate in /tmp across restarts.  Errors are silently ignored.
    std::fs::remove_file(&glb_path_for_cleanup).ok();
}

/// Watch `script_path` for changes and call `eval_script` on every change.
///
/// Blocks until the watcher channel closes.  Extracted from `run_preview` so
/// that function reads as: load config → start server → initial eval →
/// delegate here.
fn watch_script_loop(script_path: &str, eval_script: &dyn Fn(&str)) {
    use notify::{RecursiveMode, Watcher};

    // Watch the script file; re-eval on every change.
    //
    // We watch the *parent directory* rather than the file itself to handle
    // atomic-write editors (write temp → rename into place).  inotify tracks
    // inodes: a rename replaces the inode and the file-level watch goes silent.
    // A directory-level watch fires Create/Rename events for any file in the
    // directory, so we filter by the canonical script path.
    let canonical_script = std::fs::canonicalize(script_path)
        .unwrap_or_else(|_| std::path::PathBuf::from(script_path));
    let watch_dir = match canonical_script.parent() {
        Some(d) => d.to_path_buf(),
        None => {
            eprintln!("error: script path '{script_path}' has no parent directory");
            std::process::exit(1);
        }
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).ok();
    })
    .unwrap_or_else(|e| {
        eprintln!("error: failed to create file watcher: {e}");
        std::process::exit(1);
    });
    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .unwrap_or_else(|e| {
            eprintln!("error: failed to watch '{}': {e}", watch_dir.display());
            std::process::exit(1);
        });

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
}

fn load_project_config_for_cwd() -> Result<project_config::ProjectConfig, String> {
    project_config::load_for_cwd()
}

fn load_project_config_for_script(
    script_path: &str,
) -> Result<project_config::ProjectConfig, String> {
    project_config::load_for_script(std::path::Path::new(script_path))
}

fn merge_params(
    base: &[(String, String)],
    overrides: &[(String, String)],
) -> Vec<(String, String)> {
    let mut merged = base.to_vec();
    for (key, value) in overrides {
        match merged.iter_mut().find(|(existing, _)| existing == key) {
            Some(entry) => entry.1 = value.clone(),
            None => merged.push((key.clone(), value.clone())),
        }
    }
    merged
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

#[cfg(test)]
mod parse_args_tests {
    use super::{Mode, parse_args_from};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn parse_args_defaults_to_repl() {
        let parsed = parse_args_from(Vec::<String>::new()).expect("parse args");
        assert!(matches!(parsed.mode, Mode::Repl));
        assert!(parsed.params.is_empty());
    }

    #[test]
    fn parse_args_accepts_explicit_repl() {
        let parsed = parse_args_from(args(&["--repl"])).expect("parse args");
        assert!(matches!(parsed.mode, Mode::Repl));
    }

    #[test]
    fn parse_args_collects_params_and_script_mode() {
        let parsed = parse_args_from(args(&[
            "--param",
            "width=42",
            "--param",
            "height=30",
            "script.rb",
        ]))
        .expect("parse args");

        assert!(matches!(parsed.mode, Mode::Script(ref path) if path == "script.rb"));
        assert_eq!(
            parsed.params,
            vec![
                ("width".to_string(), "42".to_string()),
                ("height".to_string(), "30".to_string()),
            ]
        );
    }

    #[test]
    fn parse_args_supports_preview_and_design_table_modes() {
        let preview = parse_args_from(args(&["--preview", "script.rb"])).expect("preview args");
        assert!(
            matches!(preview.mode, Mode::Preview { ref path, port: None } if path == "script.rb")
        );

        let preview = parse_args_from(args(&["--preview-port", "4321", "--preview", "script.rb"]))
            .expect("preview args with port");
        assert!(matches!(
            preview.mode,
            Mode::Preview {
                ref path,
                port: Some(4321)
            } if path == "script.rb"
        ));

        let table = parse_args_from(args(&["--design-table", "table.csv", "script.rb"]))
            .expect("design table args");
        assert!(matches!(
            table.mode,
            Mode::DesignTable {
                ref table,
                ref script
            } if table == "table.csv" && script == "script.rb"
        ));
    }

    #[test]
    fn parse_args_supports_mcp_modes() {
        let mcp = parse_args_from(args(&["--mcp"])).expect("mcp args");
        assert!(matches!(mcp.mode, Mode::Mcp));

        let worker = parse_args_from(args(&["--mcp-worker", "validate"])).expect("worker args");
        assert!(matches!(worker.mode, Mode::McpWorker(ref kind) if kind == "validate"));
    }

    #[test]
    fn parse_args_rejects_bad_param_flags() {
        let err = parse_args_from(args(&["--param"])).expect_err("bad param should fail");
        assert!(err.contains("requires a key=value argument"));

        let err = parse_args_from(args(&["--param", "width"])).expect_err("bad param should fail");
        assert!(err.contains("key=value format"));
    }

    #[test]
    fn parse_args_rejects_missing_mode_args() {
        let err = parse_args_from(args(&["--preview"])).expect_err("missing preview path");
        assert!(err.contains("usage: rrcad --preview"));

        let err = parse_args_from(args(&["--preview-port"])).expect_err("missing preview port");
        assert!(err.contains("requires a port number"));

        let err = parse_args_from(args(&["--preview-port", "oops", "--preview", "script.rb"]))
            .expect_err("invalid preview port");
        assert!(err.contains("invalid --preview-port value"));

        let err = parse_args_from(args(&["--design-table", "table.csv"]))
            .expect_err("missing design-table script");
        assert!(err.contains("usage: rrcad --design-table"));

        let err = parse_args_from(args(&["--mcp-worker"])).expect_err("missing worker kind");
        assert!(err.contains("usage: rrcad --mcp-worker"));
    }

    #[test]
    fn parse_args_rejects_preview_port_without_preview() {
        // Script mode.
        let err = parse_args_from(args(&["--preview-port", "4321", "script.rb"]))
            .expect_err("preview-port without preview must fail");
        assert!(err.contains("--preview-port requires --preview"));

        // REPL mode.
        let err = parse_args_from(args(&["--preview-port", "4321"]))
            .expect_err("preview-port without preview must fail");
        assert!(err.contains("--preview-port requires --preview"));

        // Design-table mode.
        let err = parse_args_from(args(&[
            "--preview-port",
            "4321",
            "--design-table",
            "table.csv",
            "script.rb",
        ]))
        .expect_err("preview-port without preview must fail");
        assert!(err.contains("--preview-port requires --preview"));
    }
}

#[cfg(test)]
mod preview_port_tests {
    use super::effective_preview_port;

    #[test]
    fn cli_flag_wins_over_config() {
        assert_eq!(effective_preview_port(Some(1111), Some(2222)), 1111);
    }

    #[test]
    fn config_used_when_no_cli_flag() {
        assert_eq!(effective_preview_port(None, Some(2222)), 2222);
    }

    #[test]
    fn defaults_to_zero_for_auto_port() {
        assert_eq!(effective_preview_port(None, None), 0);
    }
}

#[cfg(test)]
mod completion_tests {
    use super::DslHelper;
    use rustyline::{Context, completion::Completer, history::DefaultHistory};

    fn ctx() -> Context<'static> {
        let history = Box::leak(Box::new(DefaultHistory::new()));
        Context::new(history)
    }

    fn words(pairs: Vec<rustyline::completion::Pair>) -> Vec<String> {
        pairs.into_iter().map(|p| p.replacement).collect()
    }

    #[test]
    fn top_level_completion_filters_by_prefix() {
        let helper = DslHelper;
        let (_, pairs) = helper.complete("bo", 2, &ctx()).expect("completion");
        assert!(words(pairs).contains(&"box".to_string()));
    }

    #[test]
    fn method_completion_uses_shape_methods_after_dot() {
        let helper = DslHelper;
        let line = "box(1, 2, 3).sc";
        let (_, pairs) = helper
            .complete(line, line.len(), &ctx())
            .expect("completion");
        assert_eq!(words(pairs), vec!["scale".to_string()]);
    }

    #[test]
    fn method_completion_handles_trimmed_dot_prefix() {
        let helper = DslHelper;
        let (_, pairs) = helper
            .complete("shape.  re", 10, &ctx())
            .expect("completion");
        let words = words(pairs);
        assert!(words.contains(&"revolve".to_string()));
        assert!(words.contains(&"respond_to?".to_string()));
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::{make_preview_glb_path, run_design_table, run_script};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

    // Local duplicate of the lib crate's `test_util::unique_test_dir`: cli.rs is
    // part of the binary crate, which cannot see the lib's #[cfg(test)] modules.
    fn unique_test_dir(prefix: &str) -> PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()))
    }

    fn with_cwd<T>(dir: &PathBuf, f: impl FnOnce() -> T) -> T {
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(dir).expect("enter temp dir");
        let result = f();
        std::env::set_current_dir(original).expect("restore cwd");
        result
    }

    #[test]
    fn run_script_executes_successfully() {
        let dir = unique_test_dir("rrcad-cli-script");
        fs::create_dir_all(&dir).expect("create temp dir");
        let script_path = dir.join("script.rb");
        fs::write(&script_path, "box(1, 2, 3).export('out.step')").expect("write script");

        with_cwd(&dir, || run_script(script_path.to_str().unwrap(), &[]));
        assert!(dir.join("out.step").exists());
    }

    #[test]
    fn run_design_table_exports_one_file_per_row() {
        let dir = unique_test_dir("rrcad-cli-design-table");
        fs::create_dir_all(&dir).expect("create temp dir");
        let table = dir.join("table.csv");
        let script = dir.join("script.rb");
        fs::write(
            &table,
            "name,width,depth,height,fillet_r\nsmall,30,20,15,1.5\nlarge,50,30,20,2.0\n",
        )
        .expect("write table");
        fs::write(
            &script,
            "name = param :name, default: \"box_part\"\nwidth = param :width, default: 50, range: 1..500\ndepth = param :depth, default: 30, range: 1..500\nheight = param :height, default: 20, range: 1..500\nfillet_r = param :fillet_r, default: 2.0, range: 0.0..10.0\n\npart = box(width, depth, height)\npart = part.fillet(fillet_r) if fillet_r > 0.0\npart.export(\"#{name}.step\")",
        )
        .expect("write script");

        with_cwd(&dir, || {
            run_design_table(table.to_str().unwrap(), script.to_str().unwrap(), &[])
                .expect("design table run")
        });

        assert!(dir.join("small.step").exists());
        assert!(dir.join("large.step").exists());
    }

    #[test]
    fn preview_glb_paths_are_randomized() {
        let a = make_preview_glb_path();
        let b = make_preview_glb_path();
        assert_ne!(a, b);
        assert!(a.starts_with(std::env::temp_dir()));
        assert!(b.starts_with(std::env::temp_dir()));
    }
}
