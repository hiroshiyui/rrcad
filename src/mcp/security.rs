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

/// Filename of the GLB written inside `MCP_SANDBOX_DIR` by the `cad_preview`
/// tool and served back by the preview HTTP server.
pub(crate) const MCP_PREVIEW_GLB: &str = "preview.glb";

/// Export formats accepted by the `cad_export` tool (Mitigation 7c).
///
/// Single source of truth: `validate_format()` checks membership and the
/// `cad_export` JSON schema enum in `resources.rs` is built from this list.
pub(crate) const MCP_EXPORT_FORMATS: &[&str] = &["step", "stl", "glb", "gltf", "obj"];

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
///
/// **Defense-in-depth only.** The real sandbox is the compile-time gem
/// restriction in `mruby_configs/mcp_safe.gembox`: no mruby-io, socket, dir,
/// eval, or metaprogramming gems are linked, so the dangerous methods listed
/// below mostly do not exist in the first place. This prelude does *not* undef
/// `const_get`, `ObjectSpace`, `Marshal`, or `instance_variable_get` — if the
/// gembox ever gains metaprogramming or IO gems, this prelude must be
/// revisited and extended accordingly.
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
    :define_method, :define_singleton_method,
    # rrcad defines puts/print/p/pp in its own prelude with a top-level `def`,
    # which lands on Object rather than Kernel — undefining them on Kernel
    # alone leaves them reachable.  __rrcad_write is the primitive beneath
    # them and must go too, or it can be called directly.
    :puts, :print, :p, :pp, :__rrcad_write
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

class Object
  [
    :File, :IO, :Dir, :FileTest, :Process, :Tempfile, :Open3
  ].each do |const|
    remove_const(const) rescue nil
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
    if MCP_EXPORT_FORMATS.contains(&format) {
        Ok(())
    } else {
        Err(format!(
            "Unsupported export format '{format}'. Allowed: {}.",
            MCP_EXPORT_FORMATS.join(", ")
        ))
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
    // Defence in depth for the worker's JSON-RPC stream: the security prelude
    // below undefines puts/print/p, but if that ever regresses, script output
    // must still not land on stdout and corrupt the response.
    crate::ruby::set_output_sink(crate::ruby::OutputSink::Stderr);

    let mut vm = MrubyVm::new();
    vm.eval(MCP_SECURITY_PRELUDE)?;
    Ok(vm)
}

#[cfg(test)]
mod tests {
    use super::{
        MCP_MAX_CODE_BYTES, create_mcp_vm, validate_code_length, validate_code_nulls,
        validate_format,
    };

    #[test]
    fn validate_code_length_accepts_limit_and_rejects_over_limit() {
        let ok = "a".repeat(MCP_MAX_CODE_BYTES);
        validate_code_length(&ok).expect("exact limit should pass");

        let too_long = "a".repeat(MCP_MAX_CODE_BYTES + 1);
        let err = validate_code_length(&too_long).expect_err("over limit should fail");
        assert!(err.contains("64 KB size limit"));
    }

    #[test]
    fn validate_code_nulls_rejects_nul_bytes() {
        let err = validate_code_nulls("abc\0def").expect_err("nul bytes should fail");
        assert!(err.contains("null bytes"));
    }

    #[test]
    fn validate_format_accepts_known_exports_and_rejects_unknown() {
        for format in ["step", "stl", "glb", "gltf", "obj"] {
            validate_format(format).expect("known export format should pass");
        }
        let err = validate_format("exe").expect_err("unknown format should fail");
        assert!(err.contains("Unsupported export format"));
    }

    #[test]
    fn create_mcp_vm_loads_security_prelude() {
        let mut vm = create_mcp_vm().expect("VM should initialise");
        let err = vm
            .eval("system('id')")
            .expect_err("system should be undefined");
        assert!(err.contains("undefined method"));
    }

    #[test]
    fn mcp_vm_cannot_print_and_routes_output_off_stdout() {
        // The worker child's stdout carries the JSON-RPC response, so a script
        // must not be able to write there.  Two independent guards: the
        // prelude undefines the printing methods, and the output sink is moved
        // off stdout in case that ever regresses.
        let mut vm = create_mcp_vm().expect("VM should initialise");
        for call in ["puts 'x'", "print 'x'", "p 'x'", "pp 'x'"] {
            let err = vm
                .eval(call)
                .expect_err("printing methods should be undefined in MCP mode");
            assert!(
                err.contains("undefined method"),
                "{call} should be undefined, got: {err}"
            );
        }

        assert_ne!(
            crate::ruby::current_output_sink(),
            crate::ruby::OutputSink::Stdout,
            "MCP mode must not leave script output pointed at stdout"
        );

        // Leave the process default alone for any test that runs afterwards.
        crate::ruby::set_output_sink(crate::ruby::OutputSink::Stdout);
    }
}
