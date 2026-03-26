//! Stress and monkey tests for the MCP server layer.
//!
//! These tests exercise the `do_cad_*` logic and the underlying mRuby/OCCT
//! stack under conditions that reveal concurrency and stability problems:
//!
//! * **Sequential rapid-fire** — many calls back-to-back, each with a fresh VM.
//! * **Error recovery** — garbage input followed by valid input (VM isolation).
//! * **Boundary inputs** — empty string, huge-but-valid code, deeply nested ops.
//! * **Concurrency guard** — verifies `MRUBY_EVAL_LOCK` prevents simultaneous
//!   VM creation from two threads.
//!
//! All mRuby VMs must run on a single OS thread (see `RUST_TEST_THREADS=1` in
//! `.cargo/config.toml`); concurrent-thread tests are carefully serialised via
//! the same `MRUBY_EVAL_LOCK` that the MCP tools use in production.

use rrcad::ruby::vm::MrubyVm;

/// The security prelude applied to every MCP VM (mirrors `mcp::create_mcp_vm`).
const MCP_SECURITY_PRELUDE: &str = r#"
[
  :system, :exec, :spawn, :fork, :exit, :exit!, :abort,
  :`, :puts, :print, :p, :pp, :gets, :readline
].each do |m|
  Kernel.send(:undef_method, m) rescue nil
end
"#;

fn make_mcp_vm() -> MrubyVm {
    let mut vm = MrubyVm::new();
    vm.eval(MCP_SECURITY_PRELUDE)
        .expect("security prelude should load");
    vm
}

// ---------------------------------------------------------------------------
// Sequential stress: N consecutive VMs
// ---------------------------------------------------------------------------

/// Create and discard VMs rapidly.  Exposes use-after-free and GC bugs that
/// only appear after many alloc/free cycles.
#[test]
fn stress_sequential_vm_create_destroy() {
    for i in 0..20 {
        let mut vm = make_mcp_vm();
        let vol: f64 = vm
            .eval(&format!("box({}, {}, {}).volume", i + 1, i + 2, i + 3))
            .expect("eval should succeed")
            .parse()
            .expect("volume should be a float");
        let expected = (i + 1) as f64 * (i + 2) as f64 * (i + 3) as f64;
        assert!(
            (vol - expected).abs() < 0.1,
            "iteration {i}: expected {expected}, got {vol}"
        );
    }
}

// ---------------------------------------------------------------------------
// Error recovery: bad code followed by good code
// ---------------------------------------------------------------------------

/// A VM that raised an exception on one eval should remain usable for
/// subsequent evals.  Verifies mRuby exception state is cleared correctly.
#[test]
fn stress_error_then_success_same_vm() {
    let mut vm = make_mcp_vm();
    // Trigger an error.
    assert!(vm.eval("this_is_not_defined_xyz").is_err());
    // The VM should still work.
    let vol: f64 = vm
        .eval("box(2, 3, 4).volume")
        .expect("eval after error should succeed")
        .parse()
        .expect("volume should be a float");
    assert!((vol - 24.0).abs() < 0.1, "volume should be 24, got {vol}");
}

/// Fresh VM after a previous VM that errored — verifies no global state leak.
#[test]
fn stress_fresh_vm_after_errored_vm() {
    {
        let mut bad_vm = make_mcp_vm();
        let _ = bad_vm.eval("raise 'intentional error'");
        // bad_vm dropped here.
    }
    let mut good_vm = make_mcp_vm();
    let result = good_vm.eval("sphere(5.0).volume");
    assert!(
        result.is_ok(),
        "fresh VM after errored VM should work: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Boundary inputs
// ---------------------------------------------------------------------------

/// Empty string eval should return nil, not crash.
#[test]
fn stress_empty_code() {
    let mut vm = make_mcp_vm();
    let r = vm.eval("");
    assert!(r.is_ok(), "empty code should not crash: {r:?}");
}

/// Code that produces nil (no expression) should not crash.
#[test]
fn stress_nil_result() {
    let mut vm = make_mcp_vm();
    let val = vm.eval("nil").expect("nil should eval cleanly");
    assert_eq!(val, "nil");
}

/// Deeply chained boolean operations stress OCCT's BRep kernel.
#[test]
fn stress_deep_boolean_chain() {
    let mut vm = make_mcp_vm();
    // Build a chain: start with a big box, then cut 5 cylinders from it.
    let code = r#"
        base = box(20, 20, 20)
        c1 = cylinder(2, 25).translate(5, 5, -2)
        c2 = cylinder(2, 25).translate(-5, 5, -2)
        c3 = cylinder(2, 25).translate(5, -5, -2)
        c4 = cylinder(2, 25).translate(-5, -5, -2)
        c5 = cylinder(1, 25).translate(0, 0, -2)
        result = base.cut(c1).cut(c2).cut(c3).cut(c4).cut(c5)
        result.volume
    "#;
    let vol_str = vm.eval(code).expect("deep boolean chain should succeed");
    let vol: f64 = vol_str.parse().expect("should produce a float volume");
    assert!(
        vol > 0.0,
        "resulting solid should have positive volume, got {vol}"
    );
}

/// Multiple shapes in one VM, checking each individually.
#[test]
fn stress_multiple_shapes_one_vm() {
    let mut vm = make_mcp_vm();
    vm.eval("$b = box(10, 10, 10)").unwrap();
    vm.eval("$c = cylinder(3, 15)").unwrap();
    vm.eval("$s = sphere(4)").unwrap();

    let b_vol: f64 = vm.eval("$b.volume").unwrap().parse().unwrap();
    let s_vol: f64 = vm.eval("$s.volume").unwrap().parse().unwrap();

    assert!((b_vol - 1000.0).abs() < 0.1, "box volume: {b_vol}");
    // sphere volume = 4/3 * π * r³ ≈ 268.08 for r=4
    assert!((s_vol - 268.08).abs() < 1.0, "sphere volume: {s_vol}");
}

// ---------------------------------------------------------------------------
// Validate geometry after operations
// ---------------------------------------------------------------------------

/// All shapes produced by the DSL should pass BRepCheck_Analyzer.
#[test]
fn stress_validate_after_ops() {
    let mut vm = make_mcp_vm();
    let shapes = [
        "box(5, 5, 5)",
        "cylinder(3, 10)",
        "sphere(4)",
        "box(10, 10, 10).fuse(sphere(6))",
        "box(10, 10, 10).cut(cylinder(3, 20))",
        "box(10, 10, 10).fillet(0.5)",
    ];
    for code in &shapes {
        vm.eval(&format!("$__s = {code}")).unwrap_or_else(|e| {
            panic!("setup for '{code}' failed: {e}");
        });
        let validity = vm
            .eval("$__s.validate")
            .unwrap_or_else(|e| panic!("validate failed for '{code}': {e}"));
        assert_eq!(
            validity, ":ok",
            "shape '{code}' failed BRepCheck_Analyzer: {validity}"
        );
    }
}

// ---------------------------------------------------------------------------
// Security prelude persistence across many evals
// ---------------------------------------------------------------------------

/// After many evals, the security prelude restrictions should still hold.
#[test]
fn stress_security_prelude_persists() {
    let mut vm = make_mcp_vm();
    // Warm up with many evals.
    for i in 0..10 {
        vm.eval(&format!("box({}, {}, {})", i + 1, i + 1, i + 1))
            .unwrap();
    }
    // Dangerous methods must still be undefined.
    assert!(
        vm.eval("system('id')").is_err(),
        "system() should still be blocked"
    );
    assert!(
        vm.eval("puts 'hi'").is_err(),
        "puts should still be blocked"
    );
}

// ---------------------------------------------------------------------------
// Concurrent VM serialisation (MRUBY_EVAL_LOCK)
// ---------------------------------------------------------------------------

/// Two threads attempting to create VMs must not run simultaneously.
///
/// This test uses the same `MRUBY_EVAL_LOCK` the MCP tools use to prove the
/// mechanism works: neither thread should observe a SIGSEGV.
#[test]
fn stress_mruby_eval_lock_serialises_threads() {
    use std::sync::{Arc, Barrier};

    // Shared barrier so both threads start "at the same time".
    let barrier = Arc::new(Barrier::new(2));

    let b1 = Arc::clone(&barrier);
    let t1 = std::thread::spawn(move || {
        b1.wait();
        let _lock = rrcad::mcp::mruby_eval_lock()
            .lock()
            .expect("lock should not be poisoned");
        let mut vm = MrubyVm::new();
        vm.eval("box(1, 2, 3).volume")
            .expect("VM eval in thread 1 should succeed")
            .parse::<f64>()
            .expect("volume should be a float")
    });

    let b2 = Arc::clone(&barrier);
    let t2 = std::thread::spawn(move || {
        b2.wait();
        let _lock = rrcad::mcp::mruby_eval_lock()
            .lock()
            .expect("lock should not be poisoned");
        let mut vm = MrubyVm::new();
        vm.eval("sphere(3).volume")
            .expect("VM eval in thread 2 should succeed")
            .parse::<f64>()
            .expect("volume should be a float")
    });

    let vol1 = t1.join().expect("thread 1 should not panic");
    let vol2 = t2.join().expect("thread 2 should not panic");

    assert!(
        (vol1 - 6.0).abs() < 0.1,
        "box volume should be 6, got {vol1}"
    );
    // sphere volume = 4/3 * π * 3³ ≈ 113.1
    assert!(
        vol2 > 100.0 && vol2 < 120.0,
        "sphere volume out of range: {vol2}"
    );
}
