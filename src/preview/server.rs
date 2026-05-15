//! axum HTTP server for the live preview.
//!
//! Routes:
//!   GET /           — Three.js viewer HTML
//!   GET /model.glb  — current tessellated shape (binary glTF)
//!   GET /metadata.json — lightweight shape properties for the inspector
//!   GET /logo.png   — rrcad logo served from doc/images/
//!   GET /ws         — WebSocket, pushes "reload" on model update

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::path::Path;
use tokio::sync::broadcast;

const VIEWER_HTML: &str = include_str!("viewer.html");
const LOGO_PNG: &[u8] = include_bytes!("../../doc/images/rrcad-logo.png");

pub async fn serve(port: u16) {
    let addr = format!("127.0.0.1:{port}");
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => serve_with_listener(listener).await,
        Err(e) => eprintln!("rrcad preview: failed to bind {addr}: {e}"),
    }
}

/// Start the axum preview server on a pre-bound listener.
///
/// Used by the MCP `cad_preview` tool to eliminate the TOCTOU race between
/// port discovery and axum binding: the caller binds the port first (getting an
/// OS-assigned free port), then passes the live listener here.
pub async fn serve_with_listener(listener: tokio::net::TcpListener) {
    let app = Router::new()
        .route("/", get(handler_root))
        .route("/model.glb", get(handler_model))
        .route("/metadata.json", get(handler_metadata))
        .route("/logo.png", get(handler_logo))
        .route("/ws", get(handler_ws));

    // Errors are swallowed — the server exits cleanly when the runtime shuts
    // down or the listener is closed, both of which are normal shutdown paths.
    axum::serve(listener, app).await.ok();
}

async fn handler_root() -> Html<&'static str> {
    Html(VIEWER_HTML)
}

async fn handler_model() -> Response {
    let state = match crate::preview::PREVIEW.get() {
        Some(s) => s,
        None => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    preview_file_response(&state.glb_path, "model/gltf-binary").await
}

async fn handler_metadata() -> Response {
    let state = match crate::preview::PREVIEW.get() {
        Some(s) => s,
        None => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let path = crate::preview::metadata_path_for_glb(&state.glb_path);
    preview_file_response(&path, "application/json").await
}

async fn handler_logo() -> Response {
    ([(header::CONTENT_TYPE, "image/png")], LOGO_PNG).into_response()
}

async fn handler_ws(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn preview_file_response(path: &Path, content_type: &'static str) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, content_type)], bytes).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn handle_socket(mut socket: WebSocket) {
    let mut rx = match crate::preview::PREVIEW.get() {
        Some(s) => s.reload_tx.subscribe(),
        None => return,
    };

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(()) | Err(broadcast::error::RecvError::Lagged(_)) => {
                        if socket.send(Message::Text("reload".to_string())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                if msg.is_none() { break; }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::{PreviewState, metadata_path_for_glb};
    use axum::body::to_bytes;
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use tokio::sync::broadcast;

    static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

    fn unique_test_dir(prefix: &str) -> std::path::PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()))
    }

    fn preview_state() -> &'static PreviewState {
        crate::preview::PREVIEW.get_or_init(|| {
            let dir = unique_test_dir("rrcad-preview-server");
            fs::create_dir_all(&dir).expect("create preview temp dir");

            let glb_path = dir.join("preview.glb");
            fs::write(&glb_path, b"glb-bytes").expect("write preview glb");
            fs::write(metadata_path_for_glb(&glb_path), br#"{"hello":"world"}"#)
                .expect("write preview metadata");

            let (reload_tx, _) = broadcast::channel(4);
            PreviewState {
                glb_path,
                reload_tx,
            }
        })
    }

    async fn body_bytes(response: Response) -> Vec<u8> {
        to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body")
            .to_vec()
    }

    #[tokio::test]
    async fn handler_root_serves_viewer_html() {
        let response = handler_root().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(body_bytes(response).await, VIEWER_HTML.as_bytes());
    }

    #[tokio::test]
    async fn handler_logo_serves_png() {
        let response = handler_logo().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/png"
        );
        assert_eq!(body_bytes(response).await, LOGO_PNG);
    }

    #[tokio::test]
    async fn preview_file_response_reads_existing_file() {
        let dir = unique_test_dir("rrcad-preview-file");
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("model.glb");
        fs::write(&path, b"preview-bytes").expect("write preview file");

        let response = preview_file_response(&path, "model/gltf-binary").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "model/gltf-binary"
        );
        assert_eq!(body_bytes(response).await, b"preview-bytes");

        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn preview_file_response_returns_not_found_for_missing_file() {
        let dir = unique_test_dir("rrcad-preview-missing");
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("missing.glb");

        let response = preview_file_response(&path, "model/gltf-binary").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn handler_model_serves_preview_glb() {
        let _ = preview_state();
        let response = handler_model().await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "model/gltf-binary"
        );
        assert_eq!(body_bytes(response).await, b"glb-bytes");
    }

    #[tokio::test]
    async fn handler_metadata_serves_preview_metadata() {
        let _ = preview_state();
        let response = handler_metadata().await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        assert_eq!(body_bytes(response).await, br#"{"hello":"world"}"#);
    }
}
