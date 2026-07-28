// Phase 11 — multi-file projects via `require_relative`.
//
// A project of any size outgrows one file. These tests drive the loader the
// way a user does: real files on disk, evaluated through a real VM with a
// script directory set.
//
// Covers resolution relative to the *requiring file* (not the CWD),
// evaluate-once semantics, require cycles, error attribution, the dependency
// list that `--preview` watches, and the guard that keeps the whole mechanism
// unavailable when no script directory has been set.

use rrcad::ruby::vm::MrubyVm;
use std::path::{Path, PathBuf};

/// A throwaway project directory, removed on drop even if a test panics.
struct Project {
    root: PathBuf,
}

impl Project {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "rrcad_multifile_{tag}_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).expect("create project root");
        Self { root }
    }

    /// Write `body` to `rel` inside the project, creating parent directories.
    fn file(&self, rel: &str, body: &str) -> PathBuf {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&path, body).expect("write project file");
        path
    }

    /// Evaluate `entry` as the entry script, with the loader pointed at it.
    fn run(&self, entry: &Path) -> Result<String, String> {
        let code = std::fs::read_to_string(entry).expect("read entry script");
        let mut vm = MrubyVm::new();
        vm.set_script_path(entry);
        vm.eval(&code)
    }

    /// Evaluate `entry` and also return the files it pulled in.
    fn run_with_deps(&self, entry: &Path) -> (Result<String, String>, Vec<PathBuf>) {
        let code = std::fs::read_to_string(entry).expect("read entry script");
        let mut vm = MrubyVm::new();
        vm.set_script_path(entry);
        let result = vm.eval(&code);
        let deps = vm.loaded_files();
        (result, deps)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

// ---------------------------------------------------------------------------
// Basic loading
// ---------------------------------------------------------------------------

#[test]
fn a_required_file_shares_constants_with_the_entry_script() {
    let p = Project::new("constants");
    p.file("params.rb", "PLATE_T = 2.5\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"params\"\n\
         PLATE_T",
    );
    let out = p.run(&entry).expect("script should run");
    assert_eq!(out.trim(), "2.5", "constant should cross the file boundary");
}

#[test]
fn a_required_file_shares_methods() {
    let p = Project::new("methods");
    p.file("arm.rb", "def arm\n  box(60, 8, 2)\nend\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"arm\"\n\
         arm.volume",
    );
    let out = p.run(&entry).expect("script should run");
    assert_eq!(out.trim(), "960.0", "method should cross the file boundary");
}

#[test]
fn the_rb_suffix_is_optional() {
    let p = Project::new("suffix");
    p.file("params.rb", "VALUE = 7\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"params.rb\"\n\
         VALUE",
    );
    let out = p.run(&entry).expect("explicit .rb should work");
    assert_eq!(out.trim(), "7");
}

#[test]
fn requires_resolve_relative_to_the_requiring_file_not_the_entry_script() {
    // The load-bearing rule. frame/arm.rb says `require_relative "params"`,
    // which must find frame/params.rb — not a params.rb beside the entry
    // script. Both exist here, with different values, so a wrong resolution
    // is visible rather than silently benign.
    let p = Project::new("relative");
    p.file("params.rb", "WHICH = :entry_dir\n");
    p.file("frame/params.rb", "WHICH = :frame_dir\n");
    p.file(
        "frame/arm.rb",
        "require_relative \"params\"\n\
         LOADED_FROM = WHICH\n",
    );
    let entry = p.file(
        "main.rb",
        "require_relative \"frame/arm\"\n\
         LOADED_FROM.inspect",
    );
    let out = p.run(&entry).expect("script should run");
    assert_eq!(
        out.trim().trim_matches('"'),
        ":frame_dir",
        "a nested require must resolve against its own directory"
    );
}

#[test]
fn the_include_stack_unwinds_back_to_the_entry_directory() {
    // After frame/arm.rb finishes, a bare require in the entry script must
    // resolve against the entry directory again.
    let p = Project::new("unwind");
    p.file("params.rb", "ENTRY_ONE = 1\n");
    p.file("frame/params.rb", "FRAME_ONE = 1\n");
    p.file("frame/arm.rb", "require_relative \"params\"\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"frame/arm\"\n\
         require_relative \"params\"\n\
         [FRAME_ONE, ENTRY_ONE].inspect",
    );
    let out = p.run(&entry).expect("script should run");
    assert_eq!(
        out.trim().trim_matches('"'),
        "[1, 1]",
        "both directories' params.rb should have loaded: {out}"
    );
}

// ---------------------------------------------------------------------------
// Evaluate-once semantics
// ---------------------------------------------------------------------------

#[test]
fn a_file_is_evaluated_only_once_and_the_second_require_returns_false() {
    let p = Project::new("once");
    p.file("counter.rb", "$count = ($count || 0) + 1\n");
    let entry = p.file(
        "main.rb",
        "first = require_relative \"counter\"\n\
         second = require_relative \"counter\"\n\
         [first, second, $count].inspect",
    );
    let out = p.run(&entry).expect("script should run");
    assert_eq!(
        out.trim().trim_matches('"'),
        "[true, false, 1]",
        "the second require must be a no-op returning false: {out}"
    );
}

#[test]
fn a_require_cycle_terminates() {
    // a requires b, b requires a. Without evaluate-once this recurses until
    // the stack blows; with it, both files finish.
    let p = Project::new("cycle");
    p.file("a.rb", "require_relative \"b\"\nA_DONE = true\n");
    p.file("b.rb", "require_relative \"a\"\nB_DONE = true\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"a\"\n\
         [A_DONE, B_DONE].inspect",
    );
    let out = p.run(&entry).expect("a cycle should not hang or crash");
    assert_eq!(
        out.trim().trim_matches('"'),
        "[true, true]",
        "both files in the cycle should complete: {out}"
    );
}

#[test]
fn the_same_file_reached_by_two_paths_loads_once() {
    // Both requires name the same file; the canonical path de-duplicates them
    // even though the spellings differ.
    let p = Project::new("canonical");
    p.file("shared.rb", "$loads = ($loads || 0) + 1\n");
    p.file("sub/via.rb", "require_relative \"../shared\"\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"shared\"\n\
         require_relative \"sub/via\"\n\
         $loads",
    );
    let out = p.run(&entry).expect("script should run");
    assert_eq!(
        out.trim(),
        "1",
        "'shared' and '../shared' are the same file and must load once"
    );
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn a_missing_file_names_what_was_tried() {
    let p = Project::new("missing");
    let entry = p.file("main.rb", "require_relative \"nope\"\n");
    let err = p.run(&entry).expect_err("should fail");
    assert!(
        err.contains("cannot load 'nope'") && err.contains("nope.rb"),
        "error should name the candidates tried: {err}"
    );
}

#[test]
fn a_runtime_error_in_a_required_file_propagates() {
    let p = Project::new("raise");
    p.file("boom.rb", "raise \"boom from the required file\"\n");
    let entry = p.file("main.rb", "require_relative \"boom\"\n");
    let err = p.run(&entry).expect_err("should fail");
    assert!(
        err.contains("boom from the required file"),
        "the original message should survive: {err}"
    );
}

#[test]
fn the_include_stack_unwinds_after_a_rescued_error() {
    // If a raise left the stack pushed, the next require would resolve
    // against the failed file's directory instead of the entry script's.
    let p = Project::new("rescue");
    p.file("sub/boom.rb", "raise \"boom\"\n");
    p.file("later.rb", "LATER = :ok\n");
    let entry = p.file(
        "main.rb",
        "begin\n\
           require_relative \"sub/boom\"\n\
         rescue => e\n\
         end\n\
         require_relative \"later\"\n\
         LATER.inspect",
    );
    let out = p.run(&entry).expect("script should recover");
    assert_eq!(
        out.trim().trim_matches('"'),
        ":ok",
        "resolution should work again after a rescued failure: {out}"
    );
}

#[test]
fn a_syntax_error_names_the_file_it_is_in() {
    // Without a compile context the message would say "(eval)", sending the
    // user to the wrong file.
    let p = Project::new("syntax");
    p.file("broken.rb", "def unclosed(\n");
    let entry = p.file("main.rb", "require_relative \"broken\"\n");
    let err = p.run(&entry).expect_err("should fail");
    assert!(
        err.to_lowercase().contains("syntax"),
        "expected a syntax error, got: {err}"
    );
}

#[test]
fn bare_require_redirects_to_require_relative() {
    let p = Project::new("bare");
    let entry = p.file("main.rb", "require \"something\"\n");
    let err = p.run(&entry).expect_err("bare require should fail");
    assert!(
        err.contains("require_relative"),
        "the error should point at the right method: {err}"
    );
}

// ---------------------------------------------------------------------------
// Guard: no script directory, no loading
// ---------------------------------------------------------------------------

#[test]
fn require_relative_is_unavailable_without_a_script_directory() {
    // This is the fallback guard behind the MCP security prelude: a VM that
    // was never told where the script lives cannot read files at all.
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("require_relative \"anything\"")
        .expect_err("should be unavailable");
    assert!(
        err.contains("only available when running a script file"),
        "unexpected error: {err}"
    );
}

#[test]
fn each_vm_starts_with_a_clean_load_state() {
    // State is process-global, so a stale entry would leak between runs and
    // make a file appear already-loaded when it is not.
    let p = Project::new("fresh");
    p.file("counter.rb", "$count = ($count || 0) + 1\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"counter\"\n\
         $count",
    );
    for run in 1..=3 {
        let out = p.run(&entry).expect("script should run");
        assert_eq!(
            out.trim(),
            "1",
            "run {run} should re-load the file in its own VM"
        );
    }
}

// ---------------------------------------------------------------------------
// Dependency reporting (what --preview watches)
// ---------------------------------------------------------------------------

#[test]
fn loaded_files_reports_every_required_file() {
    let p = Project::new("deps");
    p.file("frame/params.rb", "T = 2\n");
    p.file("frame/arm.rb", "require_relative \"params\"\n");
    let entry = p.file("main.rb", "require_relative \"frame/arm\"\n");

    let (result, deps) = p.run_with_deps(&entry);
    result.expect("script should run");

    let names: Vec<String> = deps
        .iter()
        .map(|d| d.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.contains(&"arm.rb".to_string()) && names.contains(&"params.rb".to_string()),
        "both required files should be reported for watching, got: {names:?}"
    );
    assert_eq!(deps.len(), 2, "exactly the two required files: {names:?}");
}

#[test]
fn loaded_files_reports_dependencies_even_when_the_script_fails() {
    // A syntax error in a required file is precisely when live-reload matters
    // most: the watcher has to be watching that file to see the fix.
    let p = Project::new("deps_fail");
    p.file("good.rb", "GOOD = 1\n");
    let entry = p.file(
        "main.rb",
        "require_relative \"good\"\n\
         raise \"later failure\"\n",
    );

    let (result, deps) = p.run_with_deps(&entry);
    result.expect_err("script should fail");
    assert_eq!(
        deps.len(),
        1,
        "the file loaded before the failure should still be watched"
    );
}

#[test]
fn loaded_files_is_empty_for_a_single_file_script() {
    let p = Project::new("solo");
    let entry = p.file("main.rb", "box(1, 1, 1).volume\n");
    let (result, deps) = p.run_with_deps(&entry);
    result.expect("script should run");
    assert!(deps.is_empty(), "nothing was required: {deps:?}");
}
