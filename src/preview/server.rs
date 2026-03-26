//! axum HTTP server for the live preview.
//!
//! Routes:
//!   GET /           — Three.js viewer HTML
//!   GET /model.glb  — current tessellated shape (binary glTF)
//!   GET /ws         — WebSocket, pushes "reload" on model update

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
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

    match tokio::fs::read(&state.glb_path).await {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, "model/gltf-binary")],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn handler_logo() -> Response {
    ([(header::CONTENT_TYPE, "image/png")], LOGO_PNG).into_response()
}

async fn handler_ws(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
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
