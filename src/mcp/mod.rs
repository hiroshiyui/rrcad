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
//! | 3 | Execution timeout | one-shot worker process killed after 30 s |
//! | 4 | Memory limit | `setrlimit(RLIMIT_AS, 2 GB)` applied in server and workers |
//! | 5 | Export path confinement | CWD → `/tmp/rrcad_mcp/` (mode 0700) |
//! | 6 | Per-call VM isolation | fresh `MrubyVm::new()` per tool call |
//! | 7 | Input validation | length cap, null-byte check, format allowlist |
//! | 8 | mRuby serialisation | `MRUBY_EVAL_LOCK` mutex prevents concurrent VMs (SIGSEGV) |

use std::{io::Read, path::PathBuf, process::Stdio, sync::Arc, time::Duration};

use rmcp::{
    RoleServer, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, ListResourcesResult,
        ListToolsResult, PaginatedRequestParam, ReadResourceRequestParam, ReadResourceResult,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::{
    io::AsyncWriteExt,
    io::{stdin, stdout},
    process::Command,
    sync::broadcast,
    time::timeout,
};

// ---------------------------------------------------------------------------
// Security constants
// ---------------------------------------------------------------------------

/// Maximum size of AI-supplied code strings (Mitigation 7a).
/// Legitimate DSL scripts are never 64 KB; large inputs are likely DoS attempts.
const MCP_MAX_CODE_BYTES: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// mRuby serialisation lock
// ---------------------------------------------------------------------------

/// Global mutex used by stress tests to ensure at most one mRuby VM executes at
/// a time inside one process.
///
/// # Why this is necessary
///
/// mRuby is not thread-safe: running two `mrb_state` instances concurrently in
/// the same process (even completely independent ones) causes SIGSEGV. MCP tool
/// calls run in one-shot child processes, so production isolation no longer
/// depends on this mutex. It remains public for tests that intentionally create
/// VMs on multiple Rust threads in one process.
static MRUBY_EVAL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Return a reference to the global mRuby serialisation mutex.
///
/// Exposed for stress tests that need to prove the lock works correctly when
/// two threads race to create mRuby VMs. Production MCP tool calls use
/// process-level isolation instead.
pub fn mruby_eval_lock() -> &'static std::sync::Mutex<()> {
    &MRUBY_EVAL_LOCK
}

/// Per-call evaluation time limit in seconds (Mitigation 3).
const MCP_EVAL_TIMEOUT_SECS: u64 = 30;

/// Export sandbox directory (Mitigation 5).  Created with mode 0700 at startup;
/// CWD is changed here so bare filenames resolve inside this directory and pass
/// `safe_path()` in the native layer.
const MCP_SANDBOX_DIR: &str = "/tmp/rrcad_mcp";

/// Address-space ceiling applied once at server startup (Mitigation 4).
///
/// `setrlimit(RLIMIT_AS)` is **process-wide** on Linux. We apply it once in
/// `start()` for the MCP server process and once in each one-shot worker before
/// user code runs.
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
///
/// Includes file-access methods (`open`, `require`, `load`) and metaprogramming
/// methods (`eval`, `send`, `instance_eval`, `define_method`, `binding`, …) even
/// though Mitigation 1's `mcp_safe.gembox` already excludes the gems that
/// provide them.  This ensures development builds (which may include the
/// default gembox) are equally restricted at runtime — otherwise a `send` or
/// `eval` would let user code reach methods we just undef'd by name.
///
/// Uses `Kernel.module_eval` (a Module instance method, not a Kernel one) so
/// that undef'ing `:send` / `:__send__` mid-loop does not break the loop
/// itself — `module_eval` was already invoked once before any undef occurred.
const MCP_SECURITY_PRELUDE: &str = r#"
BasicObject.module_eval do
  [
    :instance_eval, :instance_exec,
    :send, :__send__
  ].each do |m|
    undef_method(m) rescue nil
  end
end

Kernel.module_eval do
  [
    :system, :exec, :spawn, :fork, :exit, :exit!, :abort,
    :`, :puts, :print, :p, :pp, :gets, :readline,
    :open, :require, :require_relative, :load,
    :eval, :instance_eval, :class_eval, :module_eval,
    :send, :__send__, :public_send,
    :method, :define_method, :define_singleton_method, :binding
  ].each do |m|
    undef_method(m) rescue nil
  end
end

Object.module_eval do
  [
    :define_method, :define_singleton_method
  ].each do |m|
    undef_method(m) rescue nil
  end
end

class << self
  [
    :define_method, :define_singleton_method
  ].each do |m|
    undef_method(m) rescue nil
  end
end

Module.module_eval do
  [
    :define_method, :define_singleton_method,
    :module_eval, :class_eval
  ].each do |m|
    undef_method(m) rescue nil
  end
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

/// Mitigation 4: set the process-wide address-space limit.
///
/// Called once from `start()` for the MCP server process and once in every
/// worker process before user code runs. `setrlimit(RLIMIT_AS)` is process-wide
/// on Linux, which is exactly the boundary used for MCP tool isolation.
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
pub fn create_mcp_vm() -> Result<crate::ruby::vm::MrubyVm, String> {
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
// Killable MCP worker process
// ---------------------------------------------------------------------------

/// Request payload sent from the MCP server process to a one-shot worker.
#[derive(Serialize, Deserialize)]
struct WorkerRequest {
    code: String,
    format: Option<String>,
    filename: Option<String>,
}

/// Structured worker response. Worker-level errors are ordinary tool errors;
/// protocol failures such as invalid JSON make the worker exit non-zero.
#[derive(Serialize, Deserialize)]
struct WorkerResponse {
    ok: bool,
    value: Option<Value>,
    error: Option<String>,
}

impl WorkerResponse {
    fn ok(value: Value) -> Self {
        Self {
            ok: true,
            value: Some(value),
            error: None,
        }
    }

    fn err(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            value: None,
            error: Some(error.into()),
        }
    }
}

/// Run one MCP tool operation in a child process and kill it if the timeout
/// expires. This gives the timeout process-level enforcement: a runaway Ruby
/// loop or OCCT call cannot keep holding `MRUBY_EVAL_LOCK` in the server.
async fn run_worker_process(kind: &str, request: WorkerRequest) -> Result<Value, String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to locate rrcad: {e}"))?;
    let payload = serde_json::to_vec(&request)
        .map_err(|e| format!("Failed to encode worker request: {e}"))?;

    let mut child = Command::new(exe)
        .arg("--mcp-worker")
        .arg(kind)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to start MCP worker: {e}"))?;

    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open MCP worker stdin".to_string())?;
    child_stdin
        .write_all(&payload)
        .await
        .map_err(|e| format!("Failed to write MCP worker request: {e}"))?;
    drop(child_stdin);

    let output = match timeout(
        Duration::from_secs(MCP_EVAL_TIMEOUT_SECS),
        child.wait_with_output(),
    )
    .await
    {
        Ok(result) => result.map_err(|e| format!("MCP worker failed: {e}"))?,
        Err(_elapsed) => {
            return Err(format!(
                "Evaluation timed out ({} s limit).",
                MCP_EVAL_TIMEOUT_SECS
            ));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "MCP worker exited with {}: {stderr}",
            output.status
        ));
    }

    let response: WorkerResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("MCP worker returned invalid JSON: {e}"))?;
    if response.ok {
        Ok(response.value.unwrap_or(Value::Null))
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "MCP worker failed without an error message".to_string()))
    }
}

fn worker_eval_json(code: &str) -> Result<Value, String> {
    validate_code(code)?;

    let mut vm = create_mcp_vm()?;

    // Capture the last shape in a global so we can query it.
    vm.eval(&format!("$__s = begin\n{code}\nend"))?;

    // Query each property individually. `vm.eval` returns the Ruby
    // `inspect` string of the result (e.g. `":solid"` for a Symbol,
    // `"1234.5"` for a Float).
    let raw_type = vm.eval("$__s.shape_type.to_s")?;
    let volume = vm.eval("$__s.volume")?;
    let surface_area = vm.eval("$__s.surface_area")?;

    // Pack the bounding box as comma-separated floats to avoid parsing Ruby
    // Hash syntax (e.g. `{:x=>0.0, ...}`).
    let bb_str = vm.eval(concat!(
        "bb=$__s.bounding_box;",
        "\"#{bb[:x].to_f},#{bb[:y].to_f},#{bb[:z].to_f},",
        "#{bb[:dx].to_f},#{bb[:dy].to_f},#{bb[:dz].to_f}\""
    ))?;

    let valid = vm.eval("$__s.validate == :ok")?;

    // Symbols inspect as ":name"; strip the leading colon. Strings inspect
    // with surrounding quotes; trim them.
    let shape_type = raw_type
        .trim_matches('"')
        .trim_start_matches(':')
        .to_string();
    let bb_clean = bb_str.trim_matches('"');
    let bb: Vec<f64> = bb_clean
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f64>()
                .map_err(|e| format!("Failed to parse bounding box value '{s}': {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let volume = volume
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse volume '{volume}': {e}"))?;
    let surface_area = surface_area
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse surface area '{surface_area}': {e}"))?;

    Ok(json!({
        "shape_type":   shape_type,
        "volume":       volume,
        "surface_area": surface_area,
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
}

fn worker_export(code: &str, format: &str, filename: &str) -> Result<Value, String> {
    validate_code(code)?;
    validate_format(format)?;

    let mut vm = create_mcp_vm()?;
    vm.eval(&format!("$__s = begin\n{code}\nend"))?;
    // Relative filename resolves under CWD = /tmp/rrcad_mcp/ in MCP mode.
    vm.eval(&format!("$__s.export(\"{filename}\")"))?;
    Ok(Value::Null)
}

fn worker_validate_json(code: &str) -> Result<Value, String> {
    validate_code(code)?;

    let mut vm = create_mcp_vm()?;

    // Try to evaluate and assign the shape.
    if let Err(eval_err) = vm.eval(&format!("$__s = begin\n{code}\nend")) {
        // Syntax or runtime error in the DSL code.
        return Ok(json!({ "errors": [eval_err] }));
    }

    // Run OCCT's BRepCheck_Analyzer for geometric validity.
    match vm.eval("$__s.validate") {
        Err(e) => Ok(json!({ "errors": [e] })),
        Ok(v) if v == ":ok" => Ok(json!({ "status": "ok" })),
        Ok(issues) => Ok(json!({ "errors": [issues] })),
    }
}

/// Hidden CLI entry point used by `run_worker_process`.
///
/// Reads a `WorkerRequest` from stdin, writes a `WorkerResponse` to stdout, and
/// exits. The parent MCP server owns timeout enforcement and kills this process
/// when an operation exceeds `MCP_EVAL_TIMEOUT_SECS`.
pub fn run_worker(kind: &str) -> i32 {
    apply_memory_limit();

    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read worker request: {e}");
        return 1;
    }

    let request: WorkerRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("invalid worker request: {e}");
            return 1;
        }
    };

    let result = match kind {
        "eval" => worker_eval_json(&request.code),
        "export" => {
            let format = request.format.as_deref().unwrap_or("");
            let filename = request.filename.as_deref().unwrap_or("");
            worker_export(&request.code, format, filename)
        }
        "preview" => worker_export(&request.code, "glb", "preview.glb"),
        "validate" => worker_validate_json(&request.code),
        other => {
            eprintln!("unknown MCP worker kind: {other}");
            return 1;
        }
    };

    let response = match result {
        Ok(value) => WorkerResponse::ok(value),
        Err(error) => WorkerResponse::err(error),
    };

    match serde_json::to_string(&response) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("failed to encode worker response: {e}");
            1
        }
    }
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

/// `cad_eval` — evaluate DSL code and return shape properties as JSON.
async fn do_cad_eval(code: String) -> CallToolResult {
    if let Err(e) = validate_code(&code) {
        return err_result(e);
    }

    let request = WorkerRequest {
        code,
        format: None,
        filename: None,
    };
    match run_worker_process("eval", request).await {
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

    let request = WorkerRequest {
        code,
        format: Some(format),
        filename: Some(filename),
    };

    match run_worker_process("export", request).await {
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
            let glb_path = PathBuf::from(MCP_SANDBOX_DIR).join("preview.glb");
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
    let request = WorkerRequest {
        code,
        format: None,
        filename: None,
    };
    if let Err(e) = run_worker_process("preview", request).await {
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

    let request = WorkerRequest {
        code,
        format: None,
        filename: None,
    };
    match run_worker_process("validate", request).await {
        Ok(json_val) => ok_json(json_val),
        Err(e) => err_result(e),
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
        ("04_bracket.rb", include_str!("../../samples/04_bracket.rb")),
        (
            "05_export_formats.rb",
            include_str!("../../samples/05_export_formats.rb"),
        ),
        (
            "06_live_preview.rb",
            include_str!("../../samples/06_live_preview.rb"),
        ),
        ("07_teapot.rb", include_str!("../../samples/07_teapot.rb")),
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
    // Each one-shot worker applies the same cap before evaluating user code.
    apply_memory_limit();

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
    use super::*;

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
}
