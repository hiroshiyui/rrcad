//! Live preview orchestration.
//!
//! `PREVIEW` is a process-wide singleton initialised by `start()` when the
//! CLI is invoked with `--preview`.  The Ruby `preview(shape)` method writes
//! the GLB file and fires the reload broadcast; the axum server picks it up.

pub mod server;

use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct PreviewState {
    /// Path to the temporary GLB file the server streams to the browser.
    pub glb_path: PathBuf,
    /// Send `()` here to push a "reload" WebSocket message to all clients.
    pub reload_tx: broadcast::Sender<()>,
}

/// Initialised once by `start()`; accessed from both the axum handlers and
/// `rrcad_preview_shape` (called on the main thread from mRuby).
pub static PREVIEW: OnceLock<PreviewState> = OnceLock::new();

/// Return the JSON sidecar path used by the browser properties panel.
pub fn metadata_path_for_glb(glb_path: &std::path::Path) -> PathBuf {
    glb_path.with_extension("json")
}

/// Collect lightweight shape properties for the live preview inspector.
pub fn metadata_json_for_shape(shape: &crate::occt::Shape) -> serde_json::Value {
    metadata_json_for_shape_with_error(shape, None)
}

pub fn metadata_json_for_shape_with_error(
    shape: &crate::occt::Shape,
    error: Option<&str>,
) -> serde_json::Value {
    let bounding_box = match shape.bounding_box() {
        Ok([xmin, ymin, zmin, xmax, ymax, zmax]) => serde_json::json!({
            "min": [xmin, ymin, zmin],
            "max": [xmax, ymax, zmax],
            "size": [xmax - xmin, ymax - ymin, zmax - zmin],
        }),
        Err(e) => serde_json::json!({ "error": e }),
    };

    serde_json::json!({
        "shape_type": query_or_error(shape.shape_type_name()),
        "validation": query_or_error(shape.validate()),
        "volume": query_or_error(shape.volume()),
        "surface_area": query_or_error(shape.surface_area()),
        "bounding_box": bounding_box,
        "named_refs": shape.named_ref_snapshots(),
        "error": error,
    })
}

fn query_or_error<T: serde::Serialize>(result: Result<T, String>) -> serde_json::Value {
    match result {
        Ok(value) => serde_json::json!({ "ok": true, "value": value }),
        Err(error) => serde_json::json!({ "ok": false, "error": error }),
    }
}

/// Initialise the preview state, spawn the axum server on a background
/// Tokio runtime, and open the browser.
///
/// Returns the runtime so the caller can keep it alive for the process
/// lifetime (drop it to shut down the server). Returns `Err` if the preview
/// state was already initialised or the tokio runtime could not be created.
pub fn start(glb_path: PathBuf, port: u16) -> Result<tokio::runtime::Runtime, String> {
    let listener = server::bind_listener(port)?;
    let actual_port = listener
        .local_addr()
        .map_err(|e| format!("failed to get preview port address: {e}"))?
        .port();
    let (reload_tx, _) = broadcast::channel(16);
    PREVIEW
        .set(PreviewState {
            glb_path,
            reload_tx,
        })
        .map_err(|_| "preview::start called more than once".to_string())?;
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("failed to create tokio runtime: {e}"))?;
    rt.spawn(server::serve_with_listener(listener));

    let url = format!("http://localhost:{actual_port}");
    println!("Preview server: {url}  (Ctrl-C to quit)");
    open::that(&url).ok();

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occt::Shape;
    use std::{fs, io::ErrorKind, process::Command, time::SystemTime};

    #[test]
    fn metadata_for_box_includes_core_properties() {
        let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        let metadata = metadata_json_for_shape(&shape);

        assert_eq!(metadata["shape_type"]["value"], "solid");
        assert_eq!(metadata["validation"]["value"], "ok");
        assert_eq!(metadata["volume"]["value"], 6000.0);
        assert_eq!(metadata["surface_area"]["value"], 2200.0);
        assert_eq!(metadata["bounding_box"]["size"][0], 10.0);
        assert_eq!(metadata["bounding_box"]["size"][1], 20.0);
        assert_eq!(metadata["bounding_box"]["size"][2], 30.0);
        assert!(metadata["named_refs"].as_array().is_some());
    }

    #[test]
    fn metadata_for_named_refs_includes_selectors() {
        let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        shape.name_face("mounting_face", "top").expect("name_face");
        shape
            .name_edge("vertical_edges", "vertical")
            .expect("name_edge");
        let metadata = metadata_json_for_shape(&shape);

        let refs = metadata["named_refs"].as_array().expect("named refs array");
        assert!(refs.iter().any(|entry| entry["name"] == "mounting_face"
            && entry["kind"] == "face"
            && entry["selector"] == ":top"));
        assert!(refs.iter().any(|entry| entry["name"] == "vertical_edges"
            && entry["kind"] == "edge"
            && entry["selector"] == ":vertical"));
    }

    #[test]
    fn viewer_html_rejects_stale_preview_loads() {
        let html = include_str!("viewer.html");
        let start = html
            .find(r#"<script type="module">"#)
            .expect("viewer script start");
        let end = html[start..]
            .find("</script>")
            .map(|offset| start + offset)
            .expect("viewer script end");
        let script = &html[start + r#"<script type="module">"#.len()..end];

        let mut path = std::env::temp_dir();
        path.push(format!(
            "rrcad-viewer-{}-{}.mjs",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock before unix epoch")
                .as_nanos()
        ));
        fs::write(&path, script).expect("write viewer script");

        assert!(
            script.contains("let loadGeneration = 0;"),
            "viewer must track preview load generations"
        );
        assert!(
            script.contains("if (loadToken !== loadGeneration) return;"),
            "stale preview responses must be ignored"
        );
        assert!(
            script.contains("const loadToken = ++loadGeneration;"),
            "each preview load must mint a new generation token"
        );
        assert!(
            script.contains("loadMetadata(loadToken);"),
            "metadata fetches must be tied to the same load generation"
        );

        if let Some(node) = find_node() {
            let status = Command::new(node)
                .arg("--check")
                .arg(&path)
                .status()
                .expect("run node --check");
            assert!(status.success(), "viewer script must be valid JavaScript");
        }

        let _ = fs::remove_file(&path);
    }

    fn find_node() -> Option<&'static str> {
        match Command::new("node").arg("--version").status() {
            Ok(status) if status.success() => Some("node"),
            Err(err) if err.kind() == ErrorKind::NotFound => None,
            Ok(status) => panic!("node --version failed: {status}"),
            Err(err) => panic!("failed to probe node: {err}"),
        }
    }
}
