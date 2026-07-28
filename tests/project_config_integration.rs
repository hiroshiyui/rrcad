//! Integration tests for `rrcad.toml` project configuration loading.
//!
//! Exercises the public `rrcad::project_config` API end-to-end: nearest-file
//! resolution, the parent-directory walk-up (including from a bare relative
//! script path and from the CWD), and `[params]` scalar type handling.
//!
//! Note: cargo test runs single-threaded (`RUST_TEST_THREADS=1` in
//! .cargo/config.toml), so tests that change the process CWD are safe as long
//! as they restore it before returning.

use rrcad::project_config::{ProjectConfig, load_for_cwd, load_for_script};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

/// Create a unique temp directory path for one test.
fn unique_test_dir(prefix: &str) -> PathBuf {
    let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()))
}

/// Run `f` with the process CWD set to `dir`, restoring the original CWD
/// afterwards even though `f` may panic-free return early.
fn with_cwd<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
    let original = std::env::current_dir().expect("current dir");
    std::env::set_current_dir(dir).expect("enter temp dir");
    let result = f();
    std::env::set_current_dir(original).expect("restore cwd");
    result
}

#[test]
fn config_walks_up_from_nested_script_dir() {
    let dir = unique_test_dir("rrcad-it-config-walkup");
    let nested = dir.join("workspace/project/models");
    fs::create_dir_all(&nested).expect("create nested dirs");
    // Config sits three levels above the script.
    fs::write(dir.join("rrcad.toml"), "preview_port = 4100\n").expect("write config");
    let script = nested.join("part.rb");
    fs::write(&script, "box(1, 1, 1)").expect("write script");

    let config = load_for_script(&script).expect("load config");
    assert_eq!(config.preview_port, Some(4100));
    assert!(config.params.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn nearest_config_shadows_ancestor_config() {
    let dir = unique_test_dir("rrcad-it-config-nearest");
    let nested = dir.join("project");
    fs::create_dir_all(&nested).expect("create nested dirs");
    fs::write(dir.join("rrcad.toml"), "preview_port = 1000\n").expect("write outer config");
    fs::write(nested.join("rrcad.toml"), "preview_port = 2000\n").expect("write inner config");
    let script = nested.join("part.rb");
    fs::write(&script, "box(1, 1, 1)").expect("write script");

    let config = load_for_script(&script).expect("load config");
    assert_eq!(config.preview_port, Some(2000), "nearest rrcad.toml wins");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bare_relative_script_path_still_walks_up() {
    // Simulates `rrcad --preview part.rb` run from a nested directory: the
    // script path has no directory component, yet the walk-up must climb the
    // real filesystem from the CWD and find the ancestor config.
    let dir = unique_test_dir("rrcad-it-config-relative");
    let nested = dir.join("project/models");
    fs::create_dir_all(&nested).expect("create nested dirs");
    fs::write(dir.join("rrcad.toml"), "preview_port = 4200\n").expect("write config");
    fs::write(nested.join("part.rb"), "box(1, 1, 1)").expect("write script");

    let config = with_cwd(&nested, || load_for_script(Path::new("part.rb")))
        .expect("load config from relative path");
    assert_eq!(config.preview_port, Some(4200));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_for_cwd_finds_ancestor_config() {
    let dir = unique_test_dir("rrcad-it-config-cwd");
    let nested = dir.join("deeper/still");
    fs::create_dir_all(&nested).expect("create nested dirs");
    fs::write(
        dir.join("rrcad.toml"),
        r#"
preview_port = 4300

[params]
width = 12
"#,
    )
    .expect("write config");

    let config = with_cwd(&nested, load_for_cwd).expect("load config for cwd");
    assert_eq!(config.preview_port, Some(4300));
    assert_eq!(config.params, vec![("width".to_string(), "12".to_string())]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn params_scalar_types_convert_to_strings() {
    let dir = unique_test_dir("rrcad-it-config-params");
    fs::create_dir_all(&dir).expect("create dir");
    fs::write(
        dir.join("rrcad.toml"),
        r#"
[params]
width = 50
scale = 1.5
label = "bracket"
rounded = true
"#,
    )
    .expect("write config");
    let script = dir.join("part.rb");
    fs::write(&script, "box(1, 1, 1)").expect("write script");

    let config = load_for_script(&script).expect("load config");
    // BTreeMap ordering: alphabetical by key.
    assert_eq!(
        config.params,
        vec![
            ("label".to_string(), "bracket".to_string()),
            ("rounded".to_string(), "true".to_string()),
            ("scale".to_string(), "1.5".to_string()),
            ("width".to_string(), "50".to_string()),
        ]
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn params_reject_non_scalar_values() {
    let dir = unique_test_dir("rrcad-it-config-bad-params");
    fs::create_dir_all(&dir).expect("create dir");
    fs::write(
        dir.join("rrcad.toml"),
        r#"
[params]
sizes = [1, 2, 3]
"#,
    )
    .expect("write config");
    let script = dir.join("part.rb");
    fs::write(&script, "box(1, 1, 1)").expect("write script");

    let err = load_for_script(&script).expect_err("array param must fail");
    assert!(err.contains("scalar values"), "unexpected error: {err}");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_config_yields_defaults() {
    let dir = unique_test_dir("rrcad-it-config-none");
    // Use a nested dir with no rrcad.toml anywhere beneath temp_dir — an
    // ancestor outside the temp dir could theoretically hold one, so assert
    // only when nothing was picked up from the tree we created.
    let nested = dir.join("project");
    fs::create_dir_all(&nested).expect("create nested dirs");
    let script = nested.join("part.rb");
    fs::write(&script, "box(1, 1, 1)").expect("write script");

    let config = load_for_script(&script).expect("load config");
    // No rrcad.toml in the created tree; unless the system temp dir's
    // ancestors contain one (extremely unlikely), this is the default.
    assert_eq!(config, ProjectConfig::default());

    let _ = fs::remove_dir_all(&dir);
}
