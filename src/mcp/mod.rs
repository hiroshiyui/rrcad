//! MCP (Model Context Protocol) server for rrcad.
//!
//! Activated with `rrcad --mcp`. Communicates over stdio using the standard
//! MCP JSON-RPC protocol. All security mitigations described in Phase 9 of
//! `doc/TODOs.md` are implemented here.
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
//! See `doc/TODOs.md § Phase 9 → Security` for the full threat model.
//!
//! | # | Mitigation | Implementation |
//! |---|-----------|----------------|
//! | 1 | MCP-safe mRuby gembox | `vendor/mruby/build_config/mcp_safe.gembox` |
//! | 2 | Runtime prelude hardening | `MCP_SECURITY_PRELUDE` evaluated in every VM |
//! | 3 | Execution timeout | `tokio::time::timeout(30 s, spawn_blocking(...))` |
//! | 4 | Memory limit | `setrlimit(RLIMIT_AS, 2 GB)` applied once in `start()` |
//! | 5 | Export path confinement | CWD → `/tmp/rrcad_mcp/` (mode 0700) |
//! | 6 | Per-call VM isolation | fresh `MrubyVm::new()` per tool call |
//! | 7 | Input validation | length cap, null-byte check, format allowlist |

use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};

use rmcp::{
    RoleServer,
    ServerHandler,
    ServiceExt,
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, ListResourcesResult,
        ListToolsResult, PaginatedRequestParam, ReadResourceRequestParam, ReadResourceResult,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::{
    io::{stdin, stdout},
    sync::broadcast,
    task::spawn_blocking,
    time::timeout,
};

// ---------------------------------------------------------------------------
// Security constants
// ---------------------------------------------------------------------------

/// Maximum size of AI-supplied code strings (Mitigation 7a).
/// Legitimate DSL scripts are never 64 KB; large inputs are likely DoS attempts.
const MCP_MAX_CODE_BYTES: usize = 64 * 1024;

/// Per-call evaluation time limit in seconds (Mitigation 3).
const MCP_EVAL_TIMEOUT_SECS: u64 = 30;

/// Address-space ceiling applied once at server startup (Mitigation 4).
///
/// `setrlimit(RLIMIT_AS)` is **process-wide** on Linux — calling it from a
/// `spawn_blocking` thread would permanently cap the entire server's VAS after
/// the first tool call, causing tokio, OCCT, and mRuby to fail on subsequent
/// calls.  We apply the limit once in `start()` instead.
///
/// 2 GB gives OCCT's BRep kernel enough virtual address space for complex
/// boolean operations and tessellation while still bounding runaway allocations.
#[cfg(target_os = "linux")]
const MCP_MEMORY_LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// MCP security prelude — evaluated once per VM before user code (Mitigation 2).
///
/// Undefs dangerous Kernel methods as a second line of defence after the
/// compile-time gem restrictions (Mitigation 1). Works whether or not the
/// binary was compiled with `mcp_safe.gembox`.
const MCP_SECURITY_PRELUDE: &str = r#"
[
  :system, :exec, :spawn, :fork, :exit, :exit!, :abort,
  :`, :puts, :print, :p, :pp, :gets, :readline
].each do |m|
  Kernel.send(:undef_method, m) rescue nil
end
"#;

// ---------------------------------------------------------------------------
// Compile-time resources
// ---------------------------------------------------------------------------

/// Full DSL API reference (doc/api.md), embedded at compile time.
const API_DOC: &str = include_str!("../../doc/api.md");

// ---------------------------------------------------------------------------
// Preview state for MCP mode
// ---------------------------------------------------------------------------

/// Port the MCP preview axum server is listening on (set on first cad_preview).
static MCP_PREVIEW_PORT: OnceLock<u16> = OnceLock::new();

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
// Security helpers
// ---------------------------------------------------------------------------

/// Mitigation 7a: reject inputs longer than 64 KB.
fn validate_code_length(code: &str) -> Result<(), String> {
    if code.len() > MCP_MAX_CODE_BYTES {
        return Err(format!(
            "Code exceeds the 64 KB size limit ({} bytes). Legitimate DSL scripts are never this large.",
            code.len()
        ));
    }
    Ok(())
}

/// Mitigation 7b: reject code containing null bytes.
///
/// mRuby's C string API truncates at `\0`, so a null byte could smuggle
/// additional Ruby code after the security prelude by hiding it from the
/// C-level string length check.
fn validate_code_nulls(code: &str) -> Result<(), String> {
    if code.contains('\0') {
        return Err("Code must not contain null bytes (\\0).".to_string());
    }
    Ok(())
}

/// Run all Mitigation 7 input validation checks on a code string.
fn validate_code(code: &str) -> Result<(), String> {
    validate_code_length(code)?;
    validate_code_nulls(code)?;
    Ok(())
}

/// Mitigation 7c: validate the export format against the allowlist.
fn validate_format(format: &str) -> Result<(), String> {
    match format {
        "step" | "stl" | "glb" | "gltf" | "obj" => Ok(()),
        other => Err(format!(
            "Unsupported export format '{other}'. Allowed: step, stl, glb, gltf, obj."
        )),
    }
}

/// Mitigation 4: set the process-wide address-space limit at server startup.
///
/// Must be called **once** from `start()`, not per-call.  `setrlimit(RLIMIT_AS)`
/// is process-wide on Linux — applying it inside a `spawn_blocking` thread would
/// permanently cap the server's VAS after the first eval, causing all subsequent
/// tokio/OCCT/mRuby allocations to fail.
fn apply_memory_limit() {
    #[cfg(target_os = "linux")]
    // SAFETY: setrlimit is async-signal-safe and idempotent.
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: MCP_MEMORY_LIMIT_BYTES,
            rlim_max: MCP_MEMORY_LIMIT_BYTES,
        };
        // Ignore errors: RLIMIT_AS may be unsupported in some container
        // environments (e.g. gVisor), but is available on bare-metal Linux.
        let _ = libc::setrlimit(libc::RLIMIT_AS, &limit);
    }
}

/// Mitigation 6 + 2: create a fresh, security-hardened mRuby VM.
///
/// Each tool call gets its own interpreter so no state leaks between calls.
/// The MCP security prelude is evaluated immediately after startup to strip
/// dangerous Kernel methods before user code runs.
fn create_mcp_vm() -> Result<crate::ruby::vm::MrubyVm, String> {
    let mut vm = crate::ruby::vm::MrubyVm::new();
    vm.eval(MCP_SECURITY_PRELUDE)?;
    Ok(vm)
}

// ---------------------------------------------------------------------------
// Tool input schema builders
// ---------------------------------------------------------------------------

/// JSON Schema for tools that accept a single `code` parameter.
fn code_schema() -> Arc<Map<String, Value>> {
    Arc::new(
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Ruby DSL CAD code to evaluate. Uses the rrcad DSL (box, cylinder, sphere, fuse, cut, extrude, etc.)."
                }
            },
            "required": ["code"]
        })
        .as_object()
        .unwrap()
        .clone(),
    )
}

/// JSON Schema for `cad_export` which takes `code` plus a `format` enum.
fn code_format_schema() -> Arc<Map<String, Value>> {
    Arc::new(
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Ruby DSL CAD code to evaluate and export."
                },
                "format": {
                    "type": "string",
                    "description": "Export file format.",
                    "enum": ["step", "stl", "glb", "gltf", "obj"]
                }
            },
            "required": ["code", "format"]
        })
        .as_object()
        .unwrap()
        .clone(),
    )
}

// ---------------------------------------------------------------------------
// Tool result helpers
// ---------------------------------------------------------------------------

/// Build a tool-level error result (`isError: true`).
///
/// Per MCP spec §4.3.5, `isError: true` signals that the tool itself failed
/// (e.g. invalid DSL, OCCT error) rather than a protocol or server error.
fn err_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg.into())])
}

/// Build a successful tool result with a single JSON text payload.
fn ok_json(value: Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(value.to_string())])
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

/// `cad_eval` — evaluate DSL code and return shape properties as JSON.
async fn do_cad_eval(code: String) -> CallToolResult {
    if let Err(e) = validate_code(&code) {
        return err_result(e);
    }

    let result = timeout(
        Duration::from_secs(MCP_EVAL_TIMEOUT_SECS),
        spawn_blocking(move || -> Result<Value, String> {
            // Mitigations 6, 2.  Mitigation 4 (memory limit) is applied once
            // at startup in start() — setrlimit is process-wide, not per-thread.
            let mut vm = create_mcp_vm()?;

            // Capture the last shape in a global so we can query it.
            vm.eval(&format!("$__s = begin\n{code}\nend"))?;

            // Query each property individually. `vm.eval` returns the Ruby
            // `inspect` string of the result (e.g. `":solid"` for a Symbol,
            // `"1234.5"` for a Float).
            let raw_type = vm.eval("$__s.shape_type.to_s")?;
            let volume = vm.eval("$__s.volume")?;
            let surface_area = vm.eval("$__s.surface_area")?;

            // Pack the bounding box as comma-separated floats to avoid
            // parsing Ruby Hash syntax (e.g. `{:x=>0.0, …}`).
            let bb_str = vm.eval(concat!(
                "bb=$__s.bounding_box;",
                "\"#{bb[:x].to_f},#{bb[:y].to_f},#{bb[:z].to_f},",
                "#{bb[:dx].to_f},#{bb[:dy].to_f},#{bb[:dz].to_f}\""
            ))?;

            let valid = vm.eval("$__s.validate == :ok")?;

            // Symbols inspect as ":name"; strip the leading colon.
            // Strings inspect with surrounding quotes; trim them.
            let shape_type = raw_type.trim_matches('"').trim_start_matches(':').to_string();
            let bb_clean = bb_str.trim_matches('"');
            let bb: Vec<f64> = bb_clean
                .split(',')
                .map(|s| s.trim().parse().unwrap_or(0.0))
                .collect();

            Ok(json!({
                "shape_type":   shape_type,
                "volume":       volume.parse::<f64>().unwrap_or(0.0),
                "surface_area": surface_area.parse::<f64>().unwrap_or(0.0),
                "bounding_box": {
                    "x":  bb.first()    .copied().unwrap_or(0.0),
                    "y":  bb.get(1)     .copied().unwrap_or(0.0),
                    "z":  bb.get(2)     .copied().unwrap_or(0.0),
                    "dx": bb.get(3)     .copied().unwrap_or(0.0),
                    "dy": bb.get(4)     .copied().unwrap_or(0.0),
                    "dz": bb.get(5)     .copied().unwrap_or(0.0),
                },
                "valid": valid == "true"
            }))
        }),
    )
    .await;

    match result {
        Err(_elapsed) => err_result("Evaluation timed out (30 s limit)."),
        Ok(Err(join_err)) => err_result(format!("Internal error: {join_err}")),
        Ok(Ok(Err(ruby_err))) => err_result(ruby_err),
        Ok(Ok(Ok(json_val))) => ok_json(json_val),
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
    let abs_path = PathBuf::from("/tmp/rrcad_mcp").join(&filename);

    let result = timeout(
        Duration::from_secs(MCP_EVAL_TIMEOUT_SECS),
        spawn_blocking(move || -> Result<(), String> {
            let mut vm = create_mcp_vm()?;
            vm.eval(&format!("$__s = begin\n{code}\nend"))?;
            // Relative filename → resolves under CWD = /tmp/rrcad_mcp/.
            vm.eval(&format!("$__s.export(\"{filename}\")"))?;
            Ok(())
        }),
    )
    .await;

    match result {
        Err(_elapsed) => err_result("Export timed out (30 s limit)."),
        Ok(Err(join_err)) => err_result(format!("Internal error: {join_err}")),
        Ok(Ok(Err(ruby_err))) => err_result(ruby_err),
        Ok(Ok(Ok(()))) => ok_json(json!({
            "path": abs_path.to_string_lossy()
        })),
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

    // Lazily start the preview server on first call.
    let port = if let Some(&p) = MCP_PREVIEW_PORT.get() {
        p
    } else {
        // Bind to port 0 so the OS assigns a free port.
        let new_port = match std::net::TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l.local_addr().unwrap().port(),
            Err(e) => return err_result(format!("Failed to find free port: {e}")),
        };

        // Wire up the preview state used by the existing axum route handlers.
        let (reload_tx, _) = broadcast::channel::<()>(16);
        let glb_path = PathBuf::from("/tmp/rrcad_mcp/preview.glb");
        // OnceLock::set returns Err(val) if already set; that is fine — another
        // concurrent call beat us here, just use whatever port was stored.
        let _ = crate::preview::PREVIEW.set(crate::preview::PreviewState {
            glb_path,
            reload_tx,
        });

        // Spawn the axum server in the current (rmcp's) tokio runtime.
        tokio::spawn(crate::preview::server::serve(new_port));

        // Give the server a moment to bind before returning the URL.
        tokio::time::sleep(Duration::from_millis(200)).await;

        let _ = MCP_PREVIEW_PORT.set(new_port);
        new_port
    };

    // Export the shape to `preview.glb` (relative → /tmp/rrcad_mcp/preview.glb).
    let result = timeout(
        Duration::from_secs(MCP_EVAL_TIMEOUT_SECS),
        spawn_blocking(move || -> Result<(), String> {
            let mut vm = create_mcp_vm()?;
            vm.eval(&format!("$__s = begin\n{code}\nend"))?;
            vm.eval("$__s.export(\"preview.glb\")")?;
            Ok(())
        }),
    )
    .await;

    match result {
        Err(_elapsed) => return err_result("Preview export timed out (30 s limit)."),
        Ok(Err(join_err)) => return err_result(format!("Internal error: {join_err}")),
        Ok(Ok(Err(ruby_err))) => return err_result(ruby_err),
        Ok(Ok(Ok(()))) => {}
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

    let result = timeout(
        Duration::from_secs(MCP_EVAL_TIMEOUT_SECS),
        spawn_blocking(move || -> Result<Value, String> {
            let mut vm = create_mcp_vm()?;

            // Try to evaluate and assign the shape.
            match vm.eval(&format!("$__s = begin\n{code}\nend")) {
                Err(eval_err) => {
                    // Syntax or runtime error in the DSL code.
                    return Ok(json!({ "errors": [eval_err] }));
                }
                Ok(_) => {}
            }

            // Run OCCT's BRepCheck_Analyzer for geometric validity.
            match vm.eval("$__s.validate") {
                Err(e) => Ok(json!({ "errors": [e] })),
                Ok(v) if v == ":ok" => Ok(json!({ "status": "ok" })),
                Ok(issues) => Ok(json!({ "errors": [issues] })),
            }
        }),
    )
    .await;

    match result {
        Err(_elapsed) => err_result("Validation timed out (30 s limit)."),
        Ok(Err(join_err)) => err_result(format!("Internal error: {join_err}")),
        Ok(Ok(Err(ruby_err))) => err_result(ruby_err),
        Ok(Ok(Ok(json_val))) => ok_json(json_val),
    }
}

// ---------------------------------------------------------------------------
// Examples resource builder
// ---------------------------------------------------------------------------

/// Concatenate all sample scripts into one text block for `rrcad://examples`.
fn build_examples_content() -> String {
    let samples: &[(&str, &str)] = &[
        (
            "01_hello_box.rb",
            include_str!("../../samples/01_hello_box.rb"),
        ),
        (
            "02_boolean_ops.rb",
            include_str!("../../samples/02_boolean_ops.rb"),
        ),
        (
            "03_transforms.rb",
            include_str!("../../samples/03_transforms.rb"),
        ),
        (
            "04_bracket.rb",
            include_str!("../../samples/04_bracket.rb"),
        ),
        (
            "05_export_formats.rb",
            include_str!("../../samples/05_export_formats.rb"),
        ),
        (
            "06_live_preview.rb",
            include_str!("../../samples/06_live_preview.rb"),
        ),
        (
            "07_teapot.rb",
            include_str!("../../samples/07_teapot.rb"),
        ),
        (
            "08_parametric_box.rb",
            include_str!("../../samples/08_parametric_box.rb"),
        ),
    ];

    let mut buf = String::new();
    for (name, content) in samples {
        buf.push_str(&format!("# ===== {name} =====\n\n{content}\n\n"));
    }
    buf
}

// ---------------------------------------------------------------------------
// ServerHandler implementation
// ---------------------------------------------------------------------------

impl ServerHandler for McpServer {
    /// Advertise server identity and capabilities.
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    /// List the four CAD tools exposed by this server.
    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: vec![
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
            ],
            ..Default::default()
        })
    }

    /// Dispatch an incoming tool call to the appropriate handler.
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
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
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        use rmcp::model::{Annotated, RawResource};

        Ok(ListResourcesResult {
            resources: vec![
                Annotated {
                    raw: RawResource {
                        uri: "rrcad://api".to_string(),
                        name: "rrcad DSL API reference".to_string(),
                        description: Some(
                            "Full API reference for the rrcad Ruby DSL — all methods, \
                             parameters, and examples."
                                .to_string(),
                        ),
                        mime_type: Some("text/markdown".to_string()),
                        size: None,
                    },
                    annotations: None,
                },
                Annotated {
                    raw: RawResource {
                        uri: "rrcad://examples".to_string(),
                        name: "rrcad example scripts".to_string(),
                        description: Some(
                            "Sample rrcad Ruby DSL scripts demonstrating common CAD workflows."
                                .to_string(),
                        ),
                        mime_type: Some("text/plain".to_string()),
                        size: None,
                    },
                    annotations: None,
                },
            ],
            ..Default::default()
        })
    }

    /// Return the content of a requested resource URI.
    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        use rmcp::model::ResourceContents;

        match request.uri.as_str() {
            "rrcad://api" => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(API_DOC, "rrcad://api")],
            }),
            "rrcad://examples" => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(
                    build_examples_content(),
                    "rrcad://examples",
                )],
            }),
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
    // Must be done here (once, before any threads are spawned) because
    // setrlimit(RLIMIT_AS) is process-wide — applying it per spawn_blocking
    // thread would re-apply after every eval and starve tokio/OCCT.
    apply_memory_limit();

    // Mitigation 5: create export sandbox with restricted permissions.
    let sandbox = PathBuf::from("/tmp/rrcad_mcp");
    std::fs::create_dir_all(&sandbox)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&sandbox, std::fs::Permissions::from_mode(0o700))?;
    }

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
    use super::*;

    // --- Input validation ---------------------------------------------------

    #[test]
    fn test_code_length_limit() {
        let code = "a".repeat(MCP_MAX_CODE_BYTES + 1);
        let err = validate_code(&code).unwrap_err();
        assert!(err.contains("64 KB"), "error should mention the limit: {err}");
    }

    #[test]
    fn test_code_null_byte_rejected() {
        let err = validate_code("box(10, 20, 30)\0evil").unwrap_err();
        assert!(err.contains("null"), "error should mention null bytes: {err}");
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
            assert!(
                err.contains("Unsupported"),
                "should reject '{bad}': {err}"
            );
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

    /// Verify that File.read is absent when compiled with the mcp_safe gembox.
    ///
    /// This test is **ignored** when the binary was built with the default
    /// gembox (which includes mruby-io).  Delete `vendor/mruby/build/host/lib/
    /// libmruby.a` and rebuild with `MRUBY_CONFIG=build_config/rrcad` to make
    /// it pass (Mitigation 1).
    #[test]
    #[ignore = "requires mcp_safe gembox rebuild (rm vendor/mruby/build/host/lib/libmruby.a && cargo build)"]
    fn test_mcp_vm_no_file_read() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        let err = vm.eval("File.read('/etc/passwd')").unwrap_err();
        assert!(
            err.contains("uninitialized") || err.contains("constant") || err.contains("File"),
            "File.read should not be available in MCP VM: {err}"
        );
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
}
