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
  s.scale(factor)           uniform scale
  s.mirror(:xy|:xz|:yz)     mirror about a plane

Modifiers
  s.fillet(r)               round all edges
  s.chamfer(d)              bevel all edges
  s.extrude(h)              extrude face/profile by height
  s.revolve(deg=360)        revolve around Z axis
  s.sweep(path)             sweep profile along 3D wire path

Boolean operations           (return a new Shape)
  a.fuse(b)                 union of a and b
  a.cut(b)                  subtract b from a
  a.common(b)               intersection of a and b

Export
  shape.export(\"out.step\")  write STEP file

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
    "preview", // REPL control
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
    // Phase 3+ — stubs
    "faces",
    "edges",
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        None | Some("--repl") => run_repl(),
        Some("--preview") => match args.get(2) {
            Some(path) => run_preview(path),
            None => {
                eprintln!("usage: rrcad --preview <script.rb>");
                std::process::exit(1);
            }
        },
        Some(path) => run_script(path),
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

fn run_script(path: &str) {
    let code = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{path}': {e}");
            std::process::exit(1);
        }
    };
    let mut vm = MrubyVm::new();
    if let Err(e) = vm.eval(&code) {
        eprintln!("{path}: {e}");
        std::process::exit(1);
    }
}

fn run_preview(script_path: &str) {
    use notify::{RecursiveMode, Watcher};
    use rrcad::preview;

    let glb_path = std::env::temp_dir().join("rrcad_preview.glb");
    let _rt = preview::start(glb_path, 3000);

    // Helper: read and eval the script, reporting errors to stderr.
    let eval_script = |path: &str| match std::fs::read_to_string(path) {
        Ok(code) => {
            let mut vm = MrubyVm::new();
            if let Err(e) = vm.eval(&code) {
                eprintln!("{path}: {e}");
            }
        }
        Err(e) => eprintln!("error: could not read '{path}': {e}"),
    };

    // Initial eval.
    eval_script(script_path);

    // Watch the script file; re-eval on every change.
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).ok();
    })
    .expect("failed to create file watcher");
    watcher
        .watch(script_path.as_ref(), RecursiveMode::NonRecursive)
        .expect("failed to watch script");

    println!("Watching {script_path} for changes…");

    loop {
        match rx.recv() {
            Ok(_) => {
                // Debounce: drain events that arrive within the next 50 ms.
                loop {
                    match rx.recv_timeout(std::time::Duration::from_millis(50)) {
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
                eval_script(script_path);
            }
            Err(_) => break,
        }
    }
}
