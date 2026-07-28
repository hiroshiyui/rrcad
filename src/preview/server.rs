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
    extract::Request,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::net::TcpListener as StdTcpListener;
use std::path::Path;
use tokio::sync::broadcast;

const VIEWER_HTML: &str = include_str!("viewer.html");
const LOGO_PNG: &[u8] = include_bytes!("../../doc/images/rrcad-logo.png");

pub async fn serve(port: u16) {
    match bind_listener(port) {
        Ok(listener) => serve_with_listener(listener).await,
        Err(e) => eprintln!("rrcad preview: failed to bind 127.0.0.1:{port}: {e}"),
    }
}

/// Bind the preview server to localhost, optionally requesting a specific port.
///
/// Passing `0` asks the OS to choose a free port, which avoids conflicts when
/// the default preview port is already in use.
pub fn bind_listener(port: u16) -> Result<tokio::net::TcpListener, String> {
    let addr = format!("127.0.0.1:{port}");
    let listener = StdTcpListener::bind(&addr).map_err(|e| format!("{e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("failed to set nonblocking mode: {e}"))?;
    tokio::net::TcpListener::from_std(listener)
        .map_err(|e| format!("failed to convert listener: {e}"))
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
        .route("/ws", get(handler_ws))
        // DNS-rebinding defence: refuse requests whose Host / Origin headers
        // do not identify the local machine (see `is_local_request`).
        .layer(middleware::from_fn(require_local_origin));

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

/// Middleware guarding every preview route against DNS-rebinding attacks.
///
/// A malicious page at `attacker.example` can point its own DNS record at
/// 127.0.0.1 and issue same-origin requests (including WebSocket upgrades) to
/// the preview server, reading model data.  In that attack the browser sends
/// `Host: attacker.example`, so requiring a localhost Host header (and, when a
/// browser supplies one, a localhost Origin) blocks it without dependencies.
async fn require_local_origin(request: Request, next: Next) -> Response {
    let headers = request.headers();
    let host = headers.get(header::HOST).and_then(|v| v.to_str().ok());
    // Distinguish "no Origin header" (allowed: non-browser or same-origin
    // navigations) from "Origin present but not valid UTF-8" (rejected).
    let origin = match headers.get(header::ORIGIN) {
        None => None,
        Some(value) => match value.to_str() {
            Ok(s) => Some(s),
            Err(_) => return StatusCode::FORBIDDEN.into_response(),
        },
    };

    if is_local_request(host, origin) {
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

/// Return `true` when the request headers identify a local client.
///
/// Rules:
/// - The `Host` header must be present and be `localhost`, `127.0.0.1`, or
///   `[::1]`, each with an optional `:port` suffix.
/// - The `Origin` header is optional, but when present its host part must be
///   one of the same local names (any scheme/port).
fn is_local_request(host: Option<&str>, origin: Option<&str>) -> bool {
    match host {
        Some(h) if host_is_local(h) => {}
        _ => return false,
    }
    match origin {
        None => true,
        Some(o) => origin_is_local(o),
    }
}

/// Check a `host[:port]` string against the allowed local hostnames.
fn host_is_local(host: &str) -> bool {
    // Strip a trailing `:port` (digits only). For bracketed IPv6 (`[::1]:80`)
    // the split also isolates the port; a bare `[::1]` splits at an inner
    // colon whose "port" is not numeric, so it falls through unchanged.
    let name = match host.rsplit_once(':') {
        Some((h, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => h,
        _ => host,
    };
    name.eq_ignore_ascii_case("localhost") || name == "127.0.0.1" || name == "[::1]"
}

/// Check an `Origin` header value (`scheme://host[:port]`) for a local host.
fn origin_is_local(origin: &str) -> bool {
    let rest = match origin.split_once("://") {
        Some((_scheme, rest)) => rest,
        None => return false, // opaque origins like "null" are rejected
    };
    let authority = rest.split('/').next().unwrap_or(rest);
    host_is_local(authority)
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
                        if socket.send(Message::Text("reload".into())).await.is_err() {
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
    use crate::test_util::unique_test_dir;
    use axum::body::to_bytes;
    use std::fs;
    use tokio::sync::broadcast;

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

    #[test]
    fn is_local_request_accepts_local_hosts() {
        assert!(is_local_request(Some("localhost"), None));
        assert!(is_local_request(Some("localhost:8080"), None));
        assert!(is_local_request(Some("LOCALHOST:8080"), None));
        assert!(is_local_request(Some("127.0.0.1"), None));
        assert!(is_local_request(Some("127.0.0.1:3000"), None));
        assert!(is_local_request(Some("[::1]"), None));
        assert!(is_local_request(Some("[::1]:3000"), None));
    }

    #[test]
    fn is_local_request_rejects_missing_or_foreign_hosts() {
        assert!(!is_local_request(None, None));
        assert!(!is_local_request(Some("attacker.example"), None));
        assert!(!is_local_request(Some("attacker.example:8080"), None));
        // DNS rebinding: hostname resolves to 127.0.0.1 but the Host header
        // still carries the attacker's name — must be rejected.
        assert!(!is_local_request(Some("rebind.attacker.example"), None));
        // Lookalike names must not pass the suffix/port stripping.
        assert!(!is_local_request(Some("localhost.attacker.example"), None));
        assert!(!is_local_request(Some("127.0.0.1.attacker.example"), None));
    }

    #[test]
    fn is_local_request_checks_origin_when_present() {
        assert!(is_local_request(
            Some("localhost:8080"),
            Some("http://localhost:8080")
        ));
        assert!(is_local_request(
            Some("127.0.0.1:8080"),
            Some("http://127.0.0.1")
        ));
        assert!(is_local_request(
            Some("localhost"),
            Some("https://localhost:443")
        ));
        assert!(!is_local_request(
            Some("localhost:8080"),
            Some("http://attacker.example")
        ));
        assert!(!is_local_request(Some("localhost:8080"), Some("null")));
        assert!(!is_local_request(Some("localhost:8080"), Some("")));
        // Foreign host is rejected even with a local-looking origin.
        assert!(!is_local_request(
            Some("attacker.example"),
            Some("http://localhost")
        ));
    }

    #[tokio::test]
    async fn bind_listener_zero_assigns_free_port() {
        let listener = bind_listener(0).expect("bind port 0");
        let addr = listener.local_addr().expect("local addr");
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_ne!(addr.port(), 0, "OS must assign a concrete port");
    }

    #[tokio::test]
    async fn bind_listener_fails_on_taken_port() {
        let first = bind_listener(0).expect("bind port 0");
        let taken_port = first.local_addr().expect("local addr").port();

        let err = bind_listener(taken_port).expect_err("second bind must fail");
        assert!(!err.is_empty(), "error message should not be empty");
    }
}
