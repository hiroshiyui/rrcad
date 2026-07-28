/// Script output: `puts`, `print`, `p`, and `pp`.
///
/// The vendored mRuby build omits the IO gems, so these are defined in the
/// prelude on top of the native `__rrcad_write` primitive. These tests pin the
/// Kernel-compatible formatting rules people expect.
use rrcad::ruby::vm::MrubyVm;
use rrcad::ruby::{OutputSink, set_output_sink, take_captured_output};

/// Evaluate `code` with output captured, returning everything it printed.
fn printed(code: &str) -> String {
    set_output_sink(OutputSink::Capture);
    take_captured_output(); // drop anything left by a previous test
    let mut vm = MrubyVm::new();
    let result = vm.eval(code);
    let out = take_captured_output();
    set_output_sink(OutputSink::Stdout);
    result.unwrap_or_else(|e| panic!("script failed: {e}\n--- script ---\n{code}"));
    out
}

#[test]
fn puts_writes_a_line() {
    assert_eq!(printed(r#"puts "hello""#), "hello\n");
}

#[test]
fn puts_with_no_arguments_writes_a_blank_line() {
    assert_eq!(printed("puts"), "\n");
}

#[test]
fn puts_does_not_double_an_existing_newline() {
    assert_eq!(printed(r#"puts "trailing\n""#), "trailing\n");
}

#[test]
fn puts_writes_each_argument_on_its_own_line() {
    assert_eq!(printed("puts 42, 3.5, :sym"), "42\n3.5\nsym\n");
}

#[test]
fn puts_renders_nil_as_a_blank_line() {
    assert_eq!(printed("puts nil"), "\n");
}

#[test]
fn puts_flattens_arrays() {
    assert_eq!(printed(r#"puts ["a", ["b", "c"]]"#), "a\nb\nc\n");
}

#[test]
fn print_adds_no_separator_or_newline() {
    assert_eq!(printed(r#"print "a", "b", 1"#), "ab1");
}

#[test]
fn p_writes_the_inspect_form() {
    assert_eq!(printed(r#"p "quoted""#), "\"quoted\"\n");
}

#[test]
fn p_returns_its_argument_so_it_can_wrap_an_expression() {
    // `p` returning the value is what makes `x = p(compute)` a usable probe.
    assert_eq!(printed(r#"x = p(7); puts x + 1"#), "7\n8\n");
}

#[test]
fn pp_behaves_like_p() {
    assert_eq!(printed("pp [1, 2]"), "[1, 2]\n");
}

#[test]
fn output_interleaves_with_cad_work() {
    // The realistic use: printing a computed property mid-script.
    let out = printed(
        r#"
        part = box(10.0, 20.0, 30.0)
        puts "volume: #{part.volume}"
        part
        "#,
    );
    assert_eq!(out, "volume: 6000.0\n");
}

#[test]
fn a_nul_byte_in_output_is_rejected_rather_than_truncated() {
    set_output_sink(OutputSink::Capture);
    take_captured_output();
    let mut vm = MrubyVm::new();
    let err = vm
        .eval("puts \"a\\0b\"")
        .expect_err("embedded NUL should be rejected");
    set_output_sink(OutputSink::Stdout);
    assert!(
        err.to_string().contains("NUL"),
        "expected a NUL-byte error, got: {err}"
    );
}
