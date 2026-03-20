/// Integration tests for the mRuby VM layer (`rrcad::ruby::vm::MrubyVm`).
///
/// Test groups:
///   lifecycle   — open, close, multiple instances
///   eval values — integers, strings, nil, booleans, arrays, arithmetic
///   state       — variables persist across eval calls within one VM instance
///   errors      — syntax errors and runtime exceptions surface as Err
use rrcad::ruby::vm::MrubyVm;

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn vm_opens_without_panic() {
    let _vm = MrubyVm::new();
}

#[test]
fn vm_drops_cleanly() {
    {
        let _vm = MrubyVm::new();
    }
    // If mrb_close double-frees or panics, this test will crash.
}

#[test]
fn vm_multiple_independent_instances() {
    let mut a = MrubyVm::new();
    let mut b = MrubyVm::new();
    // Global variables survive across mrb_load_string calls and are VM-local.
    a.eval("$x = 1").unwrap();
    b.eval("$x = 2").unwrap();
    assert_eq!(a.eval("$x").unwrap(), "1");
    assert_eq!(b.eval("$x").unwrap(), "2");
}

// ---------------------------------------------------------------------------
// Eval — value types
// ---------------------------------------------------------------------------

#[test]
fn eval_integer() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("42").unwrap(), "42");
}

#[test]
fn eval_negative_integer() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("-7").unwrap(), "-7");
}

#[test]
fn eval_float() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("3.14").unwrap(), "3.14");
}

#[test]
fn eval_string_literal() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("\"hello\"").unwrap(), "\"hello\"");
}

#[test]
fn eval_nil() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("nil").unwrap(), "nil");
}

#[test]
fn eval_true() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("true").unwrap(), "true");
}

#[test]
fn eval_false() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("false").unwrap(), "false");
}

#[test]
fn eval_array() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("[1, 2, 3]").unwrap(), "[1, 2, 3]");
}

#[test]
fn eval_symbol() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval(":hello").unwrap(), ":hello");
}

// ---------------------------------------------------------------------------
// Eval — arithmetic and expressions
// ---------------------------------------------------------------------------

#[test]
fn eval_arithmetic_add() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("3 + 4").unwrap(), "7");
}

#[test]
fn eval_arithmetic_multiply() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("6 * 7").unwrap(), "42");
}

#[test]
fn eval_string_concat() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("\"foo\" + \"bar\"").unwrap(), "\"foobar\"");
}

#[test]
fn eval_comparison() {
    let mut vm = MrubyVm::new();
    assert_eq!(vm.eval("10 > 5").unwrap(), "true");
    assert_eq!(vm.eval("10 < 5").unwrap(), "false");
}

// ---------------------------------------------------------------------------
// Eval — state persistence
// ---------------------------------------------------------------------------

#[test]
fn eval_global_persists_across_calls() {
    // Local variables are scoped to a single mrb_load_string call.
    // Global variables ($name) survive across calls.
    let mut vm = MrubyVm::new();
    vm.eval("$counter = 0").unwrap();
    vm.eval("$counter += 1").unwrap();
    vm.eval("$counter += 1").unwrap();
    assert_eq!(vm.eval("$counter").unwrap(), "2");
}

#[test]
fn eval_method_defined_in_earlier_call() {
    let mut vm = MrubyVm::new();
    vm.eval("def double(n); n * 2; end").unwrap();
    assert_eq!(vm.eval("double(21)").unwrap(), "42");
}

#[test]
fn eval_class_defined_in_earlier_call() {
    // Class definitions persist across mrb_load_string calls.
    // Object instances must be stored in globals to be visible in later calls.
    let mut vm = MrubyVm::new();
    vm.eval("class Counter; def initialize; @n = 0; end; def inc; @n += 1; end; def value; @n; end; end")
        .unwrap();
    vm.eval("$c = Counter.new").unwrap();
    vm.eval("$c.inc").unwrap();
    vm.eval("$c.inc").unwrap();
    assert_eq!(vm.eval("$c.value").unwrap(), "2");
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn eval_syntax_error_returns_err() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("def broken(");
    assert!(result.is_err(), "syntax error should return Err");
}

#[test]
fn eval_undefined_method_returns_err() {
    let mut vm = MrubyVm::new();
    let result = vm.eval("totally_undefined_method_xyz()");
    assert!(result.is_err());
}

#[test]
fn eval_runtime_error_message_non_empty() {
    let mut vm = MrubyVm::new();
    let err = vm.eval("raise \"boom\"").unwrap_err();
    assert!(!err.is_empty(), "error message should not be empty");
    assert!(err.contains("boom"), "error message should contain 'boom'");
}

#[test]
fn eval_error_does_not_poison_vm() {
    // A failed eval must not leave the VM in an unrecoverable state.
    let mut vm = MrubyVm::new();
    vm.eval("raise \"transient error\"").unwrap_err();
    assert_eq!(vm.eval("1 + 1").unwrap(), "2");
}
