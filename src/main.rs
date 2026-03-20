use rustyline::{error::ReadlineError, DefaultEditor};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        None | Some("--repl") => run_repl(),
        Some(path) => run_script(path),
    }
}

fn run_repl() {
    println!("rrcad {}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or press Ctrl-D to quit.\n");

    let mut rl = DefaultEditor::new().expect("failed to initialise readline");

    loop {
        match rl.readline("rrcad> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);
                if line == "exit" || line == "quit" {
                    break;
                }
                // TODO(Phase 1): evaluate `line` via mRuby and print result
                println!("=> (mRuby not yet wired up)");
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
    // TODO(Phase 1): load `path`, execute via mRuby
    eprintln!("error: script execution not yet implemented (got: {path})");
    std::process::exit(1);
}
