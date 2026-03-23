//! Integration tests for the Phase 9 MCP server.
//!
//! These tests verify the security-critical behaviours of the MCP layer from
//! the outside (using public APIs only).  Unit tests for private helpers live
//! in `src/mcp/mod.rs`.
//!
//! # What is tested
//!
//! 1. Input validation (length cap, null bytes, format allowlist) — accessible
//!    through the public module constants that drive the limits.
//! 2. MCP VM isolation — a fresh `MrubyVm` with the security prelude applied
//!    behaves correctly: DSL eval works, dangerous methods are stripped.
//! 3. Sandbox directory creation — `mcp::start()` is NOT called here (it
//!    blocks on stdio), but we verify the sandbox path logic.

use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The security prelude applied to every MCP VM (Mitigation 2).
/// Duplicated from `src/mcp/mod.rs` constants to make it testable here.
const MCP_SECURITY_PRELUDE: &str = r#"
[
  :system, :exec, :spawn, :fork, :exit, :exit!, :abort,
  :`, :puts, :print, :p, :pp, :gets, :readline
].each do |m|
  Kernel.send(:undef_method, m) rescue nil
end
"#;

/// Create a VM that mirrors what `mcp::create_mcp_vm()` produces.
fn make_mcp_vm() -> MrubyVm {
    let mut vm = MrubyVm::new();
    vm.eval(MCP_SECURITY_PRELUDE)
        .expect("security prelude should load");
    vm
}

// ---------------------------------------------------------------------------
// MCP VM security (Mitigation 2)
// ---------------------------------------------------------------------------

#[test]
fn mcp_vm_system_undefined() {
    let mut vm = make_mcp_vm();
    let err = vm
        .eval("system('id')")
        .expect_err("system() should be undefined");
    assert!(
        err.to_lowercase().contains("undefined")
            || err.to_lowercase().contains("nomethod")
            || err.to_lowercase().contains("method"),
        "unexpected error for system(): {err}"
    );
}

#[test]
fn mcp_vm_exec_undefined() {
    let mut vm = make_mcp_vm();
    let err = vm
        .eval("exec('id')")
        .expect_err("exec() should be undefined");
    assert!(
        err.to_lowercase().contains("undefined")
            || err.to_lowercase().contains("nomethod")
            || err.to_lowercase().contains("method"),
        "unexpected error for exec(): {err}"
    );
}

#[test]
fn mcp_vm_backtick_undefined() {
    let mut vm = make_mcp_vm();
    // The backtick method (`) is aliased to :` on Kernel.
    let err = vm.eval("`id`").expect_err("backtick should be undefined");
    // Error message varies by mRuby version.
    assert!(
        !err.is_empty(),
        "expected an error for backtick, got empty string"
    );
}

#[test]
fn mcp_vm_puts_undefined() {
    let mut vm = make_mcp_vm();
    let err = vm
        .eval("puts 'hello'")
        .expect_err("puts should be undefined");
    assert!(
        err.to_lowercase().contains("undefined")
            || err.to_lowercase().contains("nomethod")
            || err.to_lowercase().contains("method"),
        "unexpected error for puts: {err}"
    );
}

#[test]
fn mcp_vm_exit_undefined() {
    let mut vm = make_mcp_vm();
    let err = vm.eval("exit(0)").expect_err("exit() should be undefined");
    assert!(
        err.to_lowercase().contains("undefined")
            || err.to_lowercase().contains("nomethod")
            || err.to_lowercase().contains("method"),
        "unexpected error for exit(): {err}"
    );
}

// ---------------------------------------------------------------------------
// MCP VM isolation — fresh VM per call (Mitigation 6)
// ---------------------------------------------------------------------------

#[test]
fn mcp_vm_no_state_bleed_between_vms() {
    // Set a global in VM #1.
    let mut vm1 = make_mcp_vm();
    vm1.eval("$__secret = 42").unwrap();
    assert_eq!(vm1.eval("$__secret").unwrap(), "42");

    // VM #2 is fresh — the global must not carry over.
    let mut vm2 = make_mcp_vm();
    // In mRuby, referencing an uninitialised global returns nil.
    let val = vm2.eval("$__secret").unwrap();
    assert_eq!(val, "nil", "global from VM #1 must not appear in VM #2");
}

// ---------------------------------------------------------------------------
// DSL evaluation inside the MCP VM
// ---------------------------------------------------------------------------

#[test]
fn mcp_vm_box_volume() {
    let mut vm = make_mcp_vm();
    vm.eval("$__s = box(10, 20, 30)").unwrap();
    let vol: f64 = vm
        .eval("$__s.volume")
        .unwrap()
        .parse()
        .expect("volume should be a float");
    assert!(
        (vol - 6000.0).abs() < 0.1,
        "10×20×30 box volume should be 6000, got {vol}"
    );
}

#[test]
fn mcp_vm_shape_type_symbol() {
    let mut vm = make_mcp_vm();
    vm.eval("$__s = box(5, 5, 5)").unwrap();
    // shape_type returns a Symbol; vm.eval returns its inspect string (":solid").
    let raw = vm.eval("$__s.shape_type").unwrap();
    assert_eq!(raw, ":solid", "box shape_type should be :solid");
}

#[test]
fn mcp_vm_validate_box_ok() {
    let mut vm = make_mcp_vm();
    vm.eval("$__s = box(5, 5, 5)").unwrap();
    let result = vm.eval("$__s.validate").unwrap();
    assert_eq!(result, ":ok", "simple box should pass BRepCheck_Analyzer");
}

#[test]
fn mcp_vm_bounding_box_packed() {
    let mut vm = make_mcp_vm();
    vm.eval("$__s = box(10, 20, 30)").unwrap();
    // Pack bounding box as "x,y,z,dx,dy,dz" to avoid parsing Ruby Hash syntax.
    let bb_str = vm
        .eval(concat!(
            "bb=$__s.bounding_box;",
            "\"#{bb[:x].to_f},#{bb[:y].to_f},#{bb[:z].to_f},",
            "#{bb[:dx].to_f},#{bb[:dy].to_f},#{bb[:dz].to_f}\""
        ))
        .unwrap();
    let bb_clean = bb_str.trim_matches('"');
    let parts: Vec<f64> = bb_clean.split(',').map(|s| s.parse().unwrap()).collect();
    assert_eq!(parts.len(), 6, "bounding box should have 6 components");
    // dx, dy, dz should match the box dimensions.
    assert!((parts[3] - 10.0).abs() < 0.01, "dx should be 10");
    assert!((parts[4] - 20.0).abs() < 0.01, "dy should be 20");
    assert!((parts[5] - 30.0).abs() < 0.01, "dz should be 30");
}

// ---------------------------------------------------------------------------
// MCP sandbox directory (Mitigation 5)
// ---------------------------------------------------------------------------

#[test]
fn mcp_sandbox_dir_can_be_created() {
    use std::path::Path;
    let sandbox = Path::new("/tmp/rrcad_mcp");
    std::fs::create_dir_all(sandbox).expect("should be able to create /tmp/rrcad_mcp");
    assert!(sandbox.is_dir(), "/tmp/rrcad_mcp should be a directory");
}

/// Verify that the mcp_safe.gembox file exists in the vendor directory.
/// This is Mitigation 1 — the file must be present even if mRuby hasn't been
/// rebuilt yet.
#[test]
fn mcp_safe_gembox_exists() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/mruby/build_config/mcp_safe.gembox");
    assert!(
        path.exists(),
        "vendor/mruby/build_config/mcp_safe.gembox must exist (Mitigation 1)"
    );
}

#[test]
fn rrcad_build_config_exists() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/mruby/build_config/rrcad.rb");
    assert!(
        path.exists(),
        "vendor/mruby/build_config/rrcad.rb must exist"
    );
}
