use rrcad::ruby::vm::MrubyVm;
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

Modifiers
  s.fillet(r[, :sel])       round all (or selected) edges
  s.chamfer(d[, :sel])      bevel all (or selected) edges
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
  shape.export(\"out.step\")  write STEP / STL / GLB / OBJ (by extension)

Patterns
  linear_pattern(s,n,[dx,dy,dz]) n copies translated along vector
  polar_pattern(s, n, angle_deg) n copies rotated around Z axis

Parameters (Phase 5)
  param :name, default: val        declare a parameter (returns value)
  param :name, default: val,       same, with range validation
       range: lo..hi
  # Override at the command line:
  #   rrcad --param name=value script.rb

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
///   rrcad                                       # REPL
///   rrcad --repl                                # REPL (explicit)
///   rrcad [--param k=v]... <script.rb>          # run script
///   rrcad --preview [--param k=v]... <script.rb> # live preview
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
    let code = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{path}': {e}");
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

fn run_preview(script_path: &str, params: &[(String, String)]) {
    use notify::{RecursiveMode, Watcher};
    use rrcad::preview;

    let glb_path = std::env::temp_dir().join("rrcad_preview.glb");
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
}
