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

pub async fn serve(port: u16) {
    let app = Router::new()
        .route("/", get(handler_root))
        .route("/model.glb", get(handler_model))
        .route("/ws", get(handler_ws));

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {addr}: {e}"));

    axum::serve(listener, app)
        .await
        .expect("axum server error");
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
