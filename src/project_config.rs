use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

/// Project-local configuration loaded from `rrcad.toml`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    /// Default preview port for `--preview`; `None` means auto-select a free port.
    pub preview_port: Option<u16>,
    /// Default `param` overrides applied before CLI `--param` flags.
    pub params: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
struct RawProjectConfig {
    #[serde(default)]
    preview_port: Option<u16>,
    #[serde(default)]
    params: BTreeMap<String, toml::Value>,
}

/// Load the nearest `rrcad.toml` above the current working directory.
pub fn load_for_cwd() -> Result<ProjectConfig, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("error: could not get cwd: {e}"))?;
    load_from_start_dir(&cwd)
}

/// Load the nearest `rrcad.toml` above `script_path`.
pub fn load_for_script(script_path: &Path) -> Result<ProjectConfig, String> {
    let start_dir = script_path.parent().unwrap_or_else(|| Path::new("."));
    load_from_start_dir(start_dir)
}

fn load_from_start_dir(start_dir: &Path) -> Result<ProjectConfig, String> {
    // A bare relative script path like `part.rb` yields a parent of "" whose
    // `ancestors()` iterator is just [""] — the documented walk up parent
    // directories would silently never happen.  Anchor the start directory to
    // the current working directory so `ancestors()` climbs the real
    // filesystem hierarchy.
    let start_dir = if start_dir.is_absolute() {
        start_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("error: could not get cwd: {e}"))?
            .join(start_dir)
    };
    for dir in start_dir.ancestors() {
        let config_path = dir.join("rrcad.toml");
        if config_path.is_file() {
            return load_file(&config_path);
        }
    }
    Ok(ProjectConfig::default())
}

fn load_file(path: &Path) -> Result<ProjectConfig, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("error: could not read '{}': {e}", path.display()))?;
    let raw: RawProjectConfig =
        toml::from_str(&src).map_err(|e| format!("error: invalid '{}': {e}", path.display()))?;

    let mut params = Vec::with_capacity(raw.params.len());
    for (name, value) in raw.params {
        let value = toml_value_to_param_string(&value)
            .map_err(|e| format!("error in '{}': {e}", path.display()))?;
        params.push((name, value));
    }

    Ok(ProjectConfig {
        preview_port: raw.preview_port,
        params,
    })
}

fn toml_value_to_param_string(value: &toml::Value) -> Result<String, String> {
    match value {
        toml::Value::String(s) => Ok(s.clone()),
        toml::Value::Integer(i) => Ok(i.to_string()),
        toml::Value::Float(f) => Ok(f.to_string()),
        toml::Value::Boolean(b) => Ok(b.to_string()),
        toml::Value::Datetime(dt) => Ok(dt.to_string()),
        _ => Err(
            "params entries must be scalar values (string, number, boolean, or datetime)"
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::unique_test_dir;
    use std::fs;

    #[test]
    fn load_for_script_finds_nearest_config() {
        let dir = unique_test_dir("rrcad-project-config");
        let nested = dir.join("project/models");
        fs::create_dir_all(&nested).expect("create nested dirs");
        fs::write(
            dir.join("rrcad.toml"),
            r#"
preview_port = 4321

[params]
width = 50
label = "bracket"
"#,
        )
        .expect("write config");

        let script = nested.join("part.rb");
        fs::write(&script, "box(1, 1, 1)").expect("write script");

        let config = load_for_script(&script).expect("load config");
        assert_eq!(config.preview_port, Some(4321));
        assert_eq!(
            config.params,
            vec![
                ("label".to_string(), "bracket".to_string()),
                ("width".to_string(), "50".to_string()),
            ]
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_for_script_returns_default_when_missing() {
        let dir = unique_test_dir("rrcad-project-config-missing");
        let nested = dir.join("project/models");
        fs::create_dir_all(&nested).expect("create nested dirs");
        let script = nested.join("part.rb");
        fs::write(&script, "box(1, 1, 1)").expect("write script");

        let config = load_for_script(&script).expect("load config");
        assert_eq!(config.preview_port, None);
        assert!(config.params.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_for_script_walks_up_from_bare_relative_path() {
        // Regression test: `rrcad --preview part.rb` gives a script path with
        // parent "" — the walk-up must still find rrcad.toml in an ancestor of
        // the current working directory.
        let dir = unique_test_dir("rrcad-project-config-relative");
        let nested = dir.join("project/models");
        fs::create_dir_all(&nested).expect("create nested dirs");
        fs::write(dir.join("rrcad.toml"), "preview_port = 5678\n").expect("write config");
        fs::write(nested.join("part.rb"), "box(1, 1, 1)").expect("write script");

        // cargo test runs single-threaded (RUST_TEST_THREADS=1), so changing
        // the process CWD is safe as long as we restore it afterwards.
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(&nested).expect("enter nested dir");
        let result = load_for_script(Path::new("part.rb"));
        std::env::set_current_dir(original).expect("restore cwd");

        let config = result.expect("load config");
        assert_eq!(config.preview_port, Some(5678));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_for_script_rejects_complex_param_values() {
        let dir = unique_test_dir("rrcad-project-config-invalid");
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(
            dir.join("rrcad.toml"),
            r#"
[params]
width = [1, 2, 3]
"#,
        )
        .expect("write config");

        let script = dir.join("part.rb");
        fs::write(&script, "box(1, 1, 1)").expect("write script");

        let err = load_for_script(&script).expect_err("complex param values should fail");
        assert!(err.contains("scalar values"));

        let _ = fs::remove_dir_all(&dir);
    }
}
