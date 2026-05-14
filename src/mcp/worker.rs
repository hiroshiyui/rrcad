use std::{io::Read, process::Stdio, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{io::AsyncWriteExt, process::Command, time::timeout};

use super::security::{
    MCP_EVAL_TIMEOUT_SECS, apply_memory_limit, create_mcp_vm, validate_code, validate_format,
};

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
pub(crate) async fn run_worker_process(
    kind: &str,
    code: String,
    format: Option<String>,
    filename: Option<String>,
) -> Result<Value, String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to locate rrcad: {e}"))?;
    let request = WorkerRequest {
        code,
        format,
        filename,
    };
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
