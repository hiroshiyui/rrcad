//! Script loading for multi-file projects (`require_relative`).
//!
//! A project of any size outgrows one file: a drone frame is a dozen parts
//! that share a plate thickness, a bolt pattern, and a stack height, and
//! copy-pasted constants are how the motor mount and the arm end up
//! disagreeing. This module lets one script pull in another.
//!
//! # Semantics
//!
//! `require_relative "frame/arm"` resolves against the directory of the file
//! that is *currently executing*, exactly as in Ruby — not the process CWD, so
//! a required file can require its own neighbours without knowing where the
//! entry script was run from. A `.rb` suffix is optional. Each file is
//! evaluated at most once per VM; requiring it again returns `false` rather
//! than re-running it, which also makes a require cycle terminate instead of
//! recursing forever.
//!
//! # Why a process-global is sound here
//!
//! `MrubyVm` holds the global mRuby mutex for its entire lifetime (see
//! `vm.rs`), so at most one VM exists per process at a time. The load state is
//! therefore per-VM in practice, and `reset` is called from `MrubyVm::new` so
//! nothing leaks between VMs — which matters for the test suite, where many
//! VMs run in sequence on one thread.
//!
//! # Security
//!
//! `require_relative` reads arbitrary files, so it is a file-read primitive
//! and is **disabled unless a base directory has been set**. The CLI sets one
//! from the script path; MCP never does, and additionally undefines the method
//! in its security prelude (`src/mcp/security.rs`). Both guards are deliberate:
//! the prelude is the enforcement, and the unset base is the fallback if the
//! prelude is ever weakened.

use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// One entry on the include stack.
///
/// The two `CString`s are owned here so that the pointers handed to `glue.c`
/// stay valid for as long as the file is being evaluated — the frame is not
/// popped until evaluation finishes.
struct Frame {
    path: PathBuf,
    path_c: CString,
    code_c: CString,
}

#[derive(Default)]
struct LoadState {
    /// Directory that a bare `require_relative` resolves against when the
    /// include stack is empty — i.e. the entry script's directory. `None`
    /// disables loading entirely.
    base: Option<PathBuf>,
    /// Files currently being evaluated, outermost first.
    stack: Vec<Frame>,
    /// Canonical paths already evaluated, in load order.
    loaded: Vec<PathBuf>,
}

static STATE: Mutex<Option<LoadState>> = Mutex::new(None);

fn with_state<R>(f: impl FnOnce(&mut LoadState) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|p| p.into_inner());
    f(guard.get_or_insert_with(LoadState::default))
}

/// Clear all load state. Called from `MrubyVm::new` so each VM starts clean.
pub fn reset() {
    with_state(|state| {
        state.base = None;
        state.stack.clear();
        state.loaded.clear();
    });
}

/// Enable `require_relative`, resolving against `dir`.
///
/// `dir` is the directory of the entry script (or the CWD for the REPL).
pub fn set_base_dir(dir: PathBuf) {
    with_state(|state| state.base = Some(dir));
}

/// Every file loaded so far, in load order. Used by `--preview` to watch the
/// whole project rather than only the entry script.
pub fn loaded_files() -> Vec<PathBuf> {
    with_state(|state| state.loaded.clone())
}

/// Directory that the next `require_relative` resolves against: the file
/// currently being evaluated, or the entry script's directory at the top level.
fn current_dir(state: &LoadState) -> Option<PathBuf> {
    if let Some(frame) = state.stack.last() {
        return frame.path.parent().map(|p| p.to_path_buf());
    }
    state.base.clone()
}

/// Resolve a `require_relative` argument to an absolute path.
///
/// Tries the literal name first so an explicit `.rb` works, then appends `.rb`.
fn resolve(dir: &Path, arg: &str) -> Result<PathBuf, String> {
    let joined = dir.join(arg);
    let candidates = if joined.extension().is_some_and(|e| e == "rb") {
        vec![joined]
    } else {
        vec![joined.with_extension("rb"), joined]
    };
    for candidate in &candidates {
        if candidate.is_file() {
            return std::fs::canonicalize(candidate)
                .map_err(|e| format!("cannot resolve '{}': {e}", candidate.display()));
        }
    }
    Err(format!(
        "cannot load '{arg}': no such file {}",
        candidates
            .iter()
            .map(|c| format!("'{}'", c.display()))
            .collect::<Vec<_>>()
            .join(" or ")
    ))
}

/// Outcome of `begin_require`, consumed by `glue.c`.
#[derive(Debug)]
pub enum Begin {
    /// File was already loaded; nothing to evaluate.
    AlreadyLoaded,
    /// Evaluate this source, then call [`end_require`].
    Evaluate {
        code: *const std::ffi::c_char,
        filename: *const std::ffi::c_char,
    },
}

/// Resolve, read, and push a file onto the include stack.
///
/// On `Begin::Evaluate` the caller **must** call [`end_require`] once
/// evaluation finishes, including when it raises, or the stack will not unwind.
pub fn begin_require(arg: &str) -> Result<Begin, String> {
    with_state(|state| {
        let dir = current_dir(state).ok_or_else(|| {
            "require_relative is only available when running a script file".to_string()
        })?;
        let path = resolve(&dir, arg)?;

        // Mark as loaded *before* evaluating, so a cycle terminates: a file
        // that (transitively) requires itself sees itself as already loaded.
        if state.loaded.contains(&path) {
            return Ok(Begin::AlreadyLoaded);
        }

        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
        // mRuby's C API is NUL-terminated; a NUL byte would silently truncate
        // the file and evaluate only its prefix.
        let code_c = CString::new(source)
            .map_err(|_| format!("'{}' contains a null byte", path.display()))?;
        let path_c = CString::new(path.to_string_lossy().into_owned())
            .map_err(|_| format!("'{}' contains a null byte", path.display()))?;

        state.loaded.push(path.clone());
        state.stack.push(Frame {
            path,
            path_c,
            code_c,
        });

        // Safe to hand out: the frame owns both strings and is not popped
        // until `end_require`.
        let frame = state.stack.last().expect("just pushed");
        Ok(Begin::Evaluate {
            code: frame.code_c.as_ptr(),
            filename: frame.path_c.as_ptr(),
        })
    })
}

/// Pop the include stack after a required file finishes evaluating.
pub fn end_require() {
    with_state(|state| {
        state.stack.pop();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialise these tests against each other: they share the process-global
    /// load state. (`RUST_TEST_THREADS=1` already serialises the suite, but
    /// this keeps the module honest if that ever changes.)
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rrcad_loader_{tag}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn resolve_appends_rb_and_accepts_an_explicit_suffix() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let dir = temp_dir("resolve");
        std::fs::write(dir.join("part.rb"), "# part").expect("write");

        let bare = resolve(&dir, "part").expect("bare name should resolve");
        let explicit = resolve(&dir, "part.rb").expect("explicit .rb should resolve");
        assert_eq!(bare, explicit, "both spellings must reach the same file");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_reports_both_candidates_when_missing() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let dir = temp_dir("missing");
        let err = resolve(&dir, "nope").expect_err("should not resolve");
        assert!(
            err.contains("nope.rb") && err.contains("no such file"),
            "error should name what was tried: {err}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn require_is_disabled_until_a_base_directory_is_set() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        reset();
        let err = begin_require("anything").expect_err("should be disabled");
        assert!(
            err.contains("only available when running a script file"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn a_file_is_evaluated_once_and_then_reports_already_loaded() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let dir = temp_dir("once");
        std::fs::write(dir.join("a.rb"), "x = 1").expect("write");
        reset();
        set_base_dir(dir.clone());

        match begin_require("a").expect("first require") {
            Begin::Evaluate { .. } => {}
            Begin::AlreadyLoaded => panic!("first require should evaluate"),
        }
        end_require();

        match begin_require("a").expect("second require") {
            Begin::AlreadyLoaded => {}
            Begin::Evaluate { .. } => panic!("second require should be a no-op"),
        }

        assert_eq!(loaded_files().len(), 1, "file should be recorded once");
        reset();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn nested_requires_resolve_against_the_requiring_file() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let dir = temp_dir("nested");
        let sub = dir.join("frame");
        std::fs::create_dir_all(&sub).expect("create subdir");
        std::fs::write(sub.join("arm.rb"), "# arm").expect("write");
        std::fs::write(sub.join("bolt.rb"), "# bolt").expect("write");
        reset();
        set_base_dir(dir.clone());

        // Entry requires frame/arm; while arm is on the stack, a bare "bolt"
        // must resolve inside frame/, not the entry directory.
        begin_require("frame/arm").expect("outer require");
        let inner = with_state(|state| current_dir(state).expect("a current dir"));
        assert_eq!(inner, sub, "nested require should resolve against frame/");
        end_require();

        // Once unwound, the base directory applies again.
        let outer = with_state(|state| current_dir(state).expect("a current dir"));
        assert_eq!(
            outer, dir,
            "stack should unwind back to the entry directory"
        );

        reset();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reset_clears_state_between_vms() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let dir = temp_dir("reset");
        std::fs::write(dir.join("a.rb"), "x = 1").expect("write");
        reset();
        set_base_dir(dir.clone());
        begin_require("a").expect("require");
        end_require();
        assert_eq!(loaded_files().len(), 1);

        reset();
        assert!(loaded_files().is_empty(), "reset should clear loaded files");
        assert!(
            begin_require("a").is_err(),
            "reset should also clear the base directory"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
