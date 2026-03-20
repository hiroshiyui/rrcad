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

/// Initialise the preview state, spawn the axum server on a background
/// Tokio runtime, and open the browser.
///
/// Returns the runtime so the caller can keep it alive for the process
/// lifetime (drop it to shut down the server).
pub fn start(glb_path: PathBuf, port: u16) -> tokio::runtime::Runtime {
    let (reload_tx, _) = broadcast::channel(16);
    PREVIEW
        .set(PreviewState { glb_path, reload_tx })
        .expect("preview::start called more than once");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.spawn(server::serve(port));

    // Give the server a moment to bind before opening the browser.
    std::thread::sleep(std::time::Duration::from_millis(300));

    let url = format!("http://localhost:{port}");
    println!("Preview server: {url}  (Ctrl-C to quit)");
    open::that(&url).ok();

    rt
}
