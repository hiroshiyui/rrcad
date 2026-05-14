use crate::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Security constants
// ---------------------------------------------------------------------------

/// Maximum size of AI-supplied code strings (Mitigation 7a).
/// Legitimate DSL scripts are never 64 KB; large inputs are likely DoS attempts.
pub(crate) const MCP_MAX_CODE_BYTES: usize = 64 * 1024;

/// Per-call evaluation time limit in seconds (Mitigation 3).
pub(crate) const MCP_EVAL_TIMEOUT_SECS: u64 = 30;

/// Export sandbox directory (Mitigation 5).  Created with mode 0700 at startup;
/// CWD is changed here so bare filenames resolve inside this directory and pass
/// `safe_path()` in the native layer.
pub(crate) const MCP_SANDBOX_DIR: &str = "/tmp/rrcad_mcp";

/// Address-space ceiling applied once at server startup (Mitigation 4).
///
/// `setrlimit(RLIMIT_AS)` is **process-wide** on Linux. We apply it once in
/// `start()` for the MCP server process and once in each one-shot worker before
/// user code runs.
///
/// 2 GB gives OCCT's BRep kernel enough virtual address space for complex
/// boolean operations and tessellation while still bounding runaway allocations.
#[cfg(target_os = "linux")]
pub(crate) const MCP_MEMORY_LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

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
pub(crate) const MCP_SECURITY_PRELUDE: &str = r#"
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

/// Mitigation 7a: reject inputs longer than 64 KB.
pub fn validate_code_length(code: &str) -> Result<(), String> {
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
pub fn validate_code_nulls(code: &str) -> Result<(), String> {
    if code.contains('\0') {
        return Err("Code must not contain null bytes (\\0).".to_string());
    }
    Ok(())
}

/// Run all Mitigation 7 input validation checks on a code string.
pub fn validate_code(code: &str) -> Result<(), String> {
    validate_code_length(code)?;
    validate_code_nulls(code)?;
    Ok(())
}

/// Mitigation 7c: validate the export format against the allowlist.
pub fn validate_format(format: &str) -> Result<(), String> {
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
pub(crate) fn apply_memory_limit() {
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
pub fn create_mcp_vm() -> Result<MrubyVm, String> {
    let mut vm = MrubyVm::new();
    vm.eval(MCP_SECURITY_PRELUDE)?;
    Ok(vm)
}
