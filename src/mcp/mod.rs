//! MCP (Model Context Protocol) server for rrcad.
//!
//! Activated with `rrcad --mcp`. Communicates over stdio using the standard
//! MCP JSON-RPC protocol. All security mitigations described in Phase 9 of
//! `doc/ROADMAP.md` are implemented here.
//!
//! # Tools exposed
//!
//! | Tool | Input | Output |
//! |------|-------|--------|
//! | `cad_eval` | `{ code }` | shape_type, volume, surface_area, bounding_box, valid |
//! | `cad_export` | `{ code, format }` | absolute path of exported file |
//! | `cad_preview` | `{ code }` | localhost URL of Three.js live preview |
//! | `cad_validate` | `{ code }` | `{ status: "ok" }` or `{ errors: [...] }` |
//!
//! # Resources exposed
//!
//! | URI | Content |
//! |-----|---------|
//! | `rrcad://api` | `doc/api.md` — full DSL reference |
//! | `rrcad://examples` | `samples/*.rb` — concrete DSL scripts |
//!
//! # Security mitigations
//!
//! See `doc/ROADMAP.md § Phase 9 → Security` for the full threat model.
//!
//! | # | Mitigation | Implementation |
//! |---|-----------|----------------|
//! | 1 | MCP-safe mRuby gembox | `vendor/mruby/build_config/mcp_safe.gembox` |
//! | 2 | Runtime prelude hardening | `MCP_SECURITY_PRELUDE` evaluated in every VM |
//! | 3 | Execution timeout | one-shot worker process killed after 30 s |
//! | 4 | Memory limit | `setrlimit(RLIMIT_AS, 2 GB)` applied in server and workers |
//! | 5 | Export path confinement | CWD → `/tmp/rrcad_mcp/` (mode 0700) |
//! | 6 | Per-call VM isolation | fresh `MrubyVm::new()` per tool call |
//! | 7 | Input validation | length cap, null-byte check, format allowlist |
//! | 8 | mRuby serialisation | `MRUBY_EVAL_LOCK` mutex prevents concurrent VMs (SIGSEGV) |

use std::path::PathBuf;

use rmcp::{
    RoleServer, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorData, ListResourcesResult,
        ListToolsResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::{
    io::{stdin, stdout},
    sync::broadcast,
};

mod resources;
mod security;
mod worker;

use resources::{api_doc, build_examples_content, code_format_schema, code_schema};
use security::{MCP_PREVIEW_GLB, MCP_SANDBOX_DIR};
pub use security::{create_mcp_vm, mruby_eval_lock, validate_code, validate_format};
pub use worker::run_worker;

// ---------------------------------------------------------------------------
// Preview state for MCP mode
// ---------------------------------------------------------------------------

/// Port the MCP preview axum server is listening on (set on first cad_preview).
///
/// `tokio::sync::OnceCell` (rather than `std::sync::OnceLock`) is used so the
/// initialiser is awaited under a single mutex: concurrent first-time
/// `cad_preview` calls cannot each bind their own listener and leak axum
/// tasks — only one initialiser runs, the rest await its result.
static MCP_PREVIEW_PORT: tokio::sync::OnceCell<u16> = tokio::sync::OnceCell::const_new();

// ---------------------------------------------------------------------------
// MCP server struct
// ---------------------------------------------------------------------------

/// The rrcad MCP server.  Stateless — all per-call state lives in closures.
#[derive(Clone)]
pub struct McpServer;

// ---------------------------------------------------------------------------
// Tool argument types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CadEvalArgs {
    code: String,
}

#[derive(Deserialize)]
struct CadExportArgs {
    code: String,
    format: String,
}

#[derive(Deserialize)]
struct CadPreviewArgs {
    code: String,
}

#[derive(Deserialize)]
struct CadValidateArgs {
    code: String,
}

// ---------------------------------------------------------------------------
// Tool result helpers
// ---------------------------------------------------------------------------

/// Build a tool-level error result (`isError: true`).
///
/// Per MCP spec §4.3.5, `isError: true` signals that the tool itself failed
/// (e.g. invalid DSL, OCCT error) rather than a protocol or server error.
fn err_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(msg.into())])
}

/// Build a successful tool result with a single JSON text payload.
fn ok_json(value: Value) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(value.to_string())])
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

/// `cad_eval` — evaluate DSL code and return shape properties as JSON.
async fn do_cad_eval(code: String) -> CallToolResult {
    if let Err(e) = validate_code(&code) {
        return err_result(e);
    }

    match worker::run_worker_process("eval", code, None, None).await {
        Ok(json_val) => ok_json(json_val),
        Err(e) => err_result(e),
    }
}

/// `cad_export` — evaluate DSL code and write the shape to a sandboxed file.
///
/// **Mitigation 5** — output path confinement:
/// The CWD is `/tmp/rrcad_mcp/` in MCP mode (set by `start()`).
/// Exporting with a bare UUID filename (e.g. `"abc123.step"`) resolves to
/// `/tmp/rrcad_mcp/abc123.step`, which passes `safe_path()` inside the native
/// export handler without requiring changes to the Rust path guard.
async fn do_cad_export(code: String, format: String) -> CallToolResult {
    if let Err(e) = validate_code(&code) {
        return err_result(e);
    }
    if let Err(e) = validate_format(&format) {
        return err_result(e);
    }

    // Generate a UUID filename that is unique per call.
    let uuid = uuid::Uuid::new_v4().simple().to_string();
    let filename = format!("{uuid}.{format}");
    let abs_path = PathBuf::from(MCP_SANDBOX_DIR).join(&filename);

    match worker::run_worker_process("export", code, Some(format), Some(filename)).await {
        Ok(_) => ok_json(json!({
            "path": abs_path.to_string_lossy()
        })),
        Err(e) => err_result(e),
    }
}

/// `cad_preview` — evaluate DSL code and push the shape to the live preview.
///
/// The first call starts an axum HTTP + WebSocket server in the existing tokio
/// runtime (no nested runtime created). Subsequent calls overwrite `preview.glb`
/// in the sandbox and send a WebSocket reload signal.
async fn do_cad_preview(code: String) -> CallToolResult {
    if let Err(e) = validate_code(&code) {
        return err_result(e);
    }

    // Lazily start the preview server on first call.  `OnceCell::get_or_try_init`
    // serialises concurrent first-time callers: only one binds the listener and
    // spawns axum; the rest await the same result.  This prevents the leak
    // where two concurrent callers each bind a listener and only one wins the
    // OnceLock::set, orphaning the loser's listener + axum task.
    let port_result = MCP_PREVIEW_PORT
        .get_or_try_init(|| async {
            // Bind to port 0 **and keep the listener alive** so that the OS-assigned
            // port is guaranteed free when axum starts.  Passing the bound listener
            // directly to axum eliminates the TOCTOU race that existed when we
            // extracted the port number, dropped the listener, and then had axum try
            // to rebind — a window where another process could steal the port.
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .map_err(|e| format!("Failed to bind preview port: {e}"))?;
            let new_port = listener
                .local_addr()
                .map_err(|e| format!("Failed to get preview port address: {e}"))?
                .port();

            // Wire up the preview state used by the existing axum route handlers.
            let (reload_tx, _) = broadcast::channel::<()>(16);
            let glb_path = PathBuf::from(MCP_SANDBOX_DIR).join(MCP_PREVIEW_GLB);
            // OnceLock::set returns Err(val) if already set; that is fine —
            // a previous CLI `--preview` startup may have already initialised it.
            let _ = crate::preview::PREVIEW.set(crate::preview::PreviewState {
                glb_path,
                reload_tx,
            });

            // Spawn the axum server with the already-bound listener.  No sleep
            // needed — the port is bound now; clients can connect immediately.
            tokio::spawn(crate::preview::server::serve_with_listener(listener));

            Ok::<u16, String>(new_port)
        })
        .await;
    let port = match port_result {
        Ok(&p) => p,
        Err(e) => return err_result(e),
    };

    // Export the shape to `preview.glb` (relative → /tmp/rrcad_mcp/preview.glb).
    if let Err(e) = worker::run_worker_process("preview", code, None, None).await {
        return err_result(e);
    }

    // Push a reload signal to all connected browser clients.
    if let Some(state) = crate::preview::PREVIEW.get() {
        state.reload_tx.send(()).ok();
    }

    ok_json(json!({
        "url": format!("http://localhost:{port}"),
        "message": "Open the URL in a browser to view the 3D preview. Call cad_preview again to update the model."
    }))
}

/// `cad_validate` — check DSL code for syntax / runtime / geometry errors.
async fn do_cad_validate(code: String) -> CallToolResult {
    if let Err(e) = validate_code(&code) {
        return err_result(e);
    }

    match worker::run_worker_process("validate", code, None, None).await {
        Ok(json_val) => ok_json(json_val),
        Err(e) => err_result(e),
    }
}

// ---------------------------------------------------------------------------
// ServerHandler implementation
// ---------------------------------------------------------------------------

impl ServerHandler for McpServer {
    /// Advertise server identity and capabilities.
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
    }

    /// List the four CAD tools exposed by this server.
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(vec![
            Tool::new(
                "cad_eval",
                "Evaluate Ruby DSL CAD code and return shape properties \
                     (shape_type, volume, surface_area, bounding_box, valid).",
                code_schema(),
            ),
            Tool::new(
                "cad_export",
                "Evaluate Ruby DSL CAD code and export the resulting shape \
                     to a file. Returns the absolute path of the exported file.",
                code_format_schema(),
            ),
            Tool::new(
                "cad_preview",
                "Evaluate Ruby DSL CAD code and open a live Three.js browser \
                     preview. Returns the localhost URL to open.",
                code_schema(),
            ),
            Tool::new(
                "cad_validate",
                "Check Ruby DSL CAD code for syntax errors and geometric validity. \
                     Returns {status: 'ok'} or {errors: ['...']}.",
                code_schema(),
            ),
        ]))
    }

    /// Dispatch an incoming tool call to the appropriate handler.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = request.arguments.unwrap_or_default();

        match request.name.as_ref() {
            "cad_eval" => {
                let a: CadEvalArgs = serde_json::from_value(Value::Object(args))
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                Ok(do_cad_eval(a.code).await)
            }
            "cad_export" => {
                let a: CadExportArgs = serde_json::from_value(Value::Object(args))
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                Ok(do_cad_export(a.code, a.format).await)
            }
            "cad_preview" => {
                let a: CadPreviewArgs = serde_json::from_value(Value::Object(args))
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                Ok(do_cad_preview(a.code).await)
            }
            "cad_validate" => {
                let a: CadValidateArgs = serde_json::from_value(Value::Object(args))
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                Ok(do_cad_validate(a.code).await)
            }
            name => Err(ErrorData::invalid_params(
                format!("Unknown tool: '{name}'"),
                None,
            )),
        }
    }

    /// List the two static resources: the API reference and example scripts.
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        use rmcp::model::Resource;

        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new("rrcad://api", "rrcad DSL API reference")
                .with_description(
                    "Full API reference for the rrcad Ruby DSL — all methods, \
                     parameters, and examples.",
                )
                .with_mime_type("text/markdown"),
            Resource::new("rrcad://examples", "rrcad example scripts")
                .with_description(
                    "Sample rrcad Ruby DSL scripts demonstrating common CAD workflows.",
                )
                .with_mime_type("text/plain"),
        ]))
    }

    /// Return the content of a requested resource URI.
    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        use rmcp::model::ResourceContents;

        match request.uri.as_str() {
            "rrcad://api" => Ok(ReadResourceResult::new(vec![ResourceContents::text(
                api_doc(),
                "rrcad://api",
            )])),
            "rrcad://examples" => Ok(ReadResourceResult::new(vec![ResourceContents::text(
                build_examples_content(),
                "rrcad://examples",
            )])),
            uri => Err(ErrorData::invalid_params(
                format!("Unknown resource URI: '{uri}'"),
                None,
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Start the rrcad MCP server on stdio.
///
/// This function blocks until the MCP client disconnects.
///
/// # Security setup
///
/// Before accepting any tool calls:
/// 1. Creates `/tmp/rrcad_mcp/` with mode `0700` (Mitigation 5 — sandbox).
/// 2. Changes process CWD to that sandbox so `shape.export("uuid.ext")` in
///    the Ruby DSL resolves inside it, satisfying `safe_path()` validation
///    without modifying the native export handler.
///
/// Per-call mitigations (2, 3, 4, 6, 7) are applied inside each `do_*` helper.
pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    // Mitigation 4: cap virtual address space for the whole server process.
    // Each one-shot worker applies the same cap before evaluating user code.
    security::apply_memory_limit();

    // Mitigation 5: create export sandbox with restricted permissions.
    // Use DirBuilder with an explicit mode so the directory is never
    // world-readable, even transiently — create_dir_all + set_permissions
    // has a TOCTOU window where another process could observe the directory
    // before the permissions are narrowed.
    let sandbox = PathBuf::from(MCP_SANDBOX_DIR);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&sandbox)?;
    }
    #[cfg(not(unix))]
    std::fs::create_dir_all(&sandbox)?;

    // Change CWD → sandbox. Now bare filenames (e.g. "uuid.step") resolve to
    // /tmp/rrcad_mcp/uuid.step and pass safe_path() in the native layer.
    std::env::set_current_dir(&sandbox)?;

    // Use a single-threaded runtime; stdio MCP transport is inherently serial.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        eprintln!("rrcad MCP server ready (stdio transport).");
        let service = McpServer.serve((stdin(), stdout())).await?;
        service.waiting().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::security::MCP_MAX_CODE_BYTES;
    use super::*;
    use rmcp::model::{NumberOrString, ReadResourceRequestParams};
    use rmcp::service::{RequestContext, serve_directly};

    fn test_context() -> RequestContext<rmcp::RoleServer> {
        // Peer has no public constructor in rmcp 2.x; serve a throwaway server
        // over an in-memory duplex (skipping the init handshake) just to obtain
        // a Peer handle. The handlers under test never talk to the peer, so the
        // service being dropped immediately afterwards is harmless.
        let (transport, _client_side) = tokio::io::duplex(64);
        let running = serve_directly::<rmcp::RoleServer, _, _, _, _>(McpServer, transport, None);
        let peer = running.peer().clone();
        drop(running);
        RequestContext::new(NumberOrString::Number(1), peer)
    }

    // --- Input validation ---------------------------------------------------

    #[test]
    fn test_code_length_limit() {
        let code = "a".repeat(MCP_MAX_CODE_BYTES + 1);
        let err = validate_code(&code).unwrap_err();
        assert!(
            err.contains("64 KB"),
            "error should mention the limit: {err}"
        );
    }

    #[test]
    fn test_code_null_byte_rejected() {
        let err = validate_code("box(10, 20, 30)\0evil").unwrap_err();
        assert!(
            err.contains("null"),
            "error should mention null bytes: {err}"
        );
    }

    #[test]
    fn test_valid_code_passes_validation() {
        validate_code("box(10, 20, 30)").expect("should pass");
    }

    #[test]
    fn test_format_allowlist_accepts_valid() {
        for fmt in &["step", "stl", "glb", "gltf", "obj"] {
            validate_format(fmt).unwrap_or_else(|e| panic!("format '{fmt}' should be valid: {e}"));
        }
    }

    #[test]
    fn test_format_allowlist_rejects_unknown() {
        for bad in &["exe", "rb", "../etc/passwd", "step;rm -rf /", ""] {
            let err = validate_format(bad).unwrap_err();
            assert!(err.contains("Unsupported"), "should reject '{bad}': {err}");
        }
    }

    // --- MCP VM security prelude (Mitigation 2) ----------------------------

    #[test]
    fn test_mcp_vm_exec_undeffed() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        let err = vm.eval("system('id')").unwrap_err();
        // system() must not be defined after the security prelude.
        assert!(
            err.contains("undefined") || err.contains("NoMethod") || err.contains("method"),
            "system() should be undefined in MCP VM: {err}"
        );
    }

    #[test]
    fn test_mcp_vm_puts_undeffed() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        let err = vm.eval("puts 'hello'").unwrap_err();
        assert!(
            err.contains("undefined") || err.contains("NoMethod") || err.contains("method"),
            "puts should be undefined in MCP VM: {err}"
        );
    }

    #[test]
    fn test_mcp_vm_no_file_read() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        for constant in ["File", "IO", "Dir", "FileTest", "Process"] {
            let err = vm.eval(constant).unwrap_err();
            assert!(
                err.contains("uninitialized constant") || err.contains("constant"),
                "{constant} should be removed from MCP VM, got: {err}"
            );
        }
        let err = vm.eval("File.read('/etc/passwd')").unwrap_err();
        assert!(
            err.contains("uninitialized") || err.contains("constant") || err.contains("File"),
            "File.read should not be available in MCP VM: {err}"
        );
    }

    /// Verify that `open`, `require`, `require_relative`, and `load` are removed
    /// by the runtime security prelude (Mitigation 2) regardless of which gembox
    /// the binary was compiled with.  Unlike `test_mcp_vm_no_file_read` this test
    /// never needs to be ignored — the prelude runs in every build.
    #[test]
    fn test_mcp_prelude_blocks_file_access_methods() {
        let methods = [
            ("open(\"/etc/passwd\")", "open"),
            ("require \"json\"", "require"),
            ("require_relative \"../etc/passwd\"", "require_relative"),
            ("load \"/etc/passwd\"", "load"),
        ];
        for (code, label) in methods {
            let mut vm = create_mcp_vm().expect("VM should initialise");
            let err = vm.eval(code).unwrap_err();
            assert!(
                err.contains("undefined method") || err.contains("NoMethodError"),
                "{label} should be undefined after security prelude, got: {err}"
            );
        }
    }

    // --- Basic DSL evaluation -----------------------------------------------

    #[test]
    fn test_mcp_vm_dsl_box() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        let result = vm.eval("b = box(10, 20, 30); b.volume");
        assert!(result.is_ok(), "box.volume should succeed: {result:?}");
        let vol: f64 = result.unwrap().parse().expect("volume should be a number");
        assert!(
            (vol - 6000.0).abs() < 0.1,
            "10×20×30 box volume should be 6000, got {vol}"
        );
    }

    #[test]
    fn test_mcp_vm_validate_valid_shape() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        vm.eval("$__s = box(5, 5, 5)").unwrap();
        let result = vm.eval("$__s.validate").unwrap();
        assert_eq!(result, ":ok", "a simple box should be valid");
    }

    #[tokio::test]
    async fn server_info_advertises_tools_and_resources() {
        let info = McpServer.get_info();
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.resources.is_some());
        assert_eq!(info.server_info.name, "rmcp");
    }

    #[test]
    fn ok_json_wraps_payload_as_text_content() {
        let result = ok_json(json!({"answer": 42}));
        assert_eq!(result.is_error, Some(false));
        let text = result.content[0].as_text().expect("text content");
        assert_eq!(text.text, r#"{"answer":42}"#);
    }

    #[test]
    fn err_result_marks_tool_failure() {
        let result = err_result("bad input");
        assert_eq!(result.is_error, Some(true));
        let text = result.content[0].as_text().expect("text content");
        assert_eq!(text.text, "bad input");
    }

    #[tokio::test]
    async fn list_tools_and_resources_expose_expected_entries() {
        let server = McpServer;
        let tools = server
            .list_tools(None, test_context())
            .await
            .expect("list_tools");
        let tool_names: Vec<_> = tools.tools.iter().map(|tool| tool.name.as_ref()).collect();
        assert_eq!(
            tool_names,
            vec!["cad_eval", "cad_export", "cad_preview", "cad_validate"]
        );

        let resources = server
            .list_resources(None, test_context())
            .await
            .expect("list_resources");
        let uris: Vec<_> = resources
            .resources
            .iter()
            .map(|resource| resource.uri.as_str())
            .collect();
        assert_eq!(uris, vec!["rrcad://api", "rrcad://examples"]);
    }

    #[tokio::test]
    async fn read_resource_and_call_tool_cover_dispatch_paths() {
        let server = McpServer;

        let api = server
            .read_resource(
                ReadResourceRequestParams::new("rrcad://api"),
                test_context(),
            )
            .await
            .expect("read api resource");
        assert_eq!(api.contents.len(), 1);

        let examples = server
            .read_resource(
                ReadResourceRequestParams::new("rrcad://examples"),
                test_context(),
            )
            .await
            .expect("read examples resource");
        assert_eq!(examples.contents.len(), 1);

        let err = server
            .read_resource(
                ReadResourceRequestParams::new("rrcad://missing"),
                test_context(),
            )
            .await
            .expect_err("unknown resource should fail");
        assert_eq!(err.code.0, -32602);

        let err = server
            .call_tool(
                CallToolRequestParams::new("cad_eval")
                    .with_arguments(serde_json::json!({}).as_object().cloned().unwrap()),
                test_context(),
            )
            .await
            .expect_err("cad_eval should reject missing args");
        assert_eq!(err.code.0, -32602);

        let err = server
            .call_tool(CallToolRequestParams::new("unknown_tool"), test_context())
            .await
            .expect_err("unknown tool should fail");
        assert_eq!(err.code.0, -32602);
    }
}
