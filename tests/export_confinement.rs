/// Explicit coverage for rrcad's file-path confinement rules.
///
/// Every import and export path that a Ruby script supplies is resolved by
/// `safe_path()` in `src/ruby/native_helpers.rs`, which canonicalises the path
/// and rejects anything that does not land inside the current working
/// directory.  The behaviour is intentionally strict — it is what keeps a
/// hostile or careless script (and, in MCP mode, a model-authored one) from
/// reading or writing arbitrary files on the host.
///
/// These tests pin that contract down so a future refactor cannot quietly
/// loosen it.  The user-facing description lives in
/// `doc/user-guide/10-import-export.md`.
use rrcad::ruby::vm::MrubyVm;

/// Directory inside the project tree that confined writes may target.
fn out_dir() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from("target/export_confinement");
    std::fs::create_dir_all(&dir).expect("could not create test output directory");
    dir
}

/// Evaluate `code` and return the error message, failing if it succeeded.
fn expect_eval_error(code: &str, what: &str) -> String {
    let mut vm = MrubyVm::new();
    match vm.eval(code) {
        Ok(_) => panic!("{what} should have been rejected, but the script succeeded"),
        Err(e) => e.to_string(),
    }
}

/// Assert that a rejection came from the path guard rather than some other
/// failure (a syntax error would otherwise make these tests vacuously pass).
fn assert_confinement_error(err: &str, what: &str) {
    assert!(
        err.contains("outside the working directory")
            || err.contains("cannot resolve")
            || err.contains("path traversal"),
        "{what} was rejected, but not by the path guard: {err}"
    );
}

// ---------------------------------------------------------------------------
// Allowed: paths inside the working directory
// ---------------------------------------------------------------------------

#[test]
fn export_to_relative_path_inside_cwd_is_allowed() {
    let out = out_dir().join("inside.step");
    let _ = std::fs::remove_file(&out);

    let mut vm = MrubyVm::new();
    vm.eval(&format!("box(5.0, 5.0, 5.0).export('{}')", out.display()))
        .expect("export inside the working directory should succeed");

    assert!(out.exists(), "expected {} to be written", out.display());
}

// ---------------------------------------------------------------------------
// Rejected: escapes from the working directory
// ---------------------------------------------------------------------------

#[test]
fn export_to_absolute_path_outside_cwd_is_rejected() {
    let target = std::env::temp_dir().join("rrcad_confinement_absolute.step");
    let _ = std::fs::remove_file(&target);

    let err = expect_eval_error(
        &format!("box(5.0, 5.0, 5.0).export('{}')", target.display()),
        "absolute path outside the working directory",
    );
    assert_confinement_error(&err, "absolute path outside the working directory");
    assert!(
        !target.exists(),
        "rejected export must not create {}",
        target.display()
    );
}

#[test]
fn export_with_parent_traversal_is_rejected() {
    // `..` climbs out of the project tree even though the prefix looks local.
    let err = expect_eval_error(
        "box(5.0, 5.0, 5.0).export('../../rrcad_confinement_traversal.step')",
        "parent-directory traversal",
    );
    assert_confinement_error(&err, "parent-directory traversal");
}

#[test]
fn export_into_missing_directory_is_rejected() {
    // The parent directory must already exist and canonicalise; rrcad never
    // creates directories on a script's behalf.
    let err = expect_eval_error(
        "box(5.0, 5.0, 5.0).export('target/export_confinement/no_such_dir/part.step')",
        "export into a non-existent directory",
    );
    assert_confinement_error(&err, "export into a non-existent directory");
}

#[test]
#[cfg(unix)]
fn export_through_symlink_escaping_cwd_is_rejected() {
    use std::os::unix::fs::symlink;

    // A symlink that lives inside the working directory but resolves outside
    // it must not become an escape hatch: safe_path canonicalises first.
    let link = out_dir().join("escape_link.step");
    let target = std::env::temp_dir().join("rrcad_confinement_symlink.step");
    let _ = std::fs::remove_file(&link);
    let _ = std::fs::remove_file(&target);
    std::fs::write(&target, b"placeholder").expect("could not seed symlink target");
    symlink(&target, &link).expect("could not create test symlink");

    let err = expect_eval_error(
        &format!("box(5.0, 5.0, 5.0).export('{}')", link.display()),
        "symlink escaping the working directory",
    );
    assert_confinement_error(&err, "symlink escaping the working directory");

    // The link must not have been followed and overwritten.
    let content = std::fs::read(&target).expect("symlink target should still exist");
    assert_eq!(
        content, b"placeholder",
        "rejected export wrote through the symlink"
    );

    let _ = std::fs::remove_file(&link);
    let _ = std::fs::remove_file(&target);
}

#[test]
fn import_from_outside_cwd_is_rejected() {
    // Confinement covers reads as well as writes, so a script cannot exfiltrate
    // geometry from elsewhere on the host.
    let outside = std::env::temp_dir().join("rrcad_confinement_import.step");
    std::fs::write(&outside, b"not really a step file").expect("could not seed import source");

    let err = expect_eval_error(
        &format!("import_step('{}')", outside.display()),
        "import from outside the working directory",
    );
    assert_confinement_error(&err, "import from outside the working directory");

    let _ = std::fs::remove_file(&outside);
}

// ---------------------------------------------------------------------------
// The guard applies to every export format, not just STEP
// ---------------------------------------------------------------------------

#[test]
fn all_export_formats_enforce_confinement() {
    for ext in ["step", "stl", "gltf", "glb", "obj"] {
        let target = std::env::temp_dir().join(format!("rrcad_confinement_all_formats.{ext}"));
        let _ = std::fs::remove_file(&target);

        let err = expect_eval_error(
            &format!("box(5.0, 5.0, 5.0).export('{}')", target.display()),
            &format!("{ext} export outside the working directory"),
        );
        assert_confinement_error(&err, &format!("{ext} export"));
        assert!(
            !target.exists(),
            "rejected {ext} export must not create {}",
            target.display()
        );
    }
}
