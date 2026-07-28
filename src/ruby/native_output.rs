// NOTE: the module-level safety contract in `native.rs` applies to every `extern "C"` function in this file.

//! Script output (`puts` / `print` / `p`).
//!
//! The vendored mRuby build deliberately omits the IO gems, so `Kernel#puts`
//! and friends do not exist in the interpreter.  The DSL provides them in the
//! prelude on top of the one primitive here, which is the only path from a
//! script to the process's output streams.
//!
//! Routing matters for correctness, not just taste: in MCP mode the worker
//! child's **stdout carries the JSON-RPC response** (`src/mcp/worker.rs`), so
//! anything a script printed there would corrupt the reply.  MCP therefore
//! switches this sink to stderr.  (The MCP security prelude also undefines
//! `puts`/`print`/`p` outright; the sink switch is defence in depth in case
//! that ever regresses.)

use std::ffi::c_char;
use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};

/// Where script output is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputSink {
    /// Normal CLI and REPL behaviour.
    Stdout,
    /// MCP mode: stdout is reserved for the JSON-RPC response.
    Stderr,
    /// Tests: accumulate into a buffer instead of a stream.
    Capture,
}

const SINK_STDOUT: u8 = 0;
const SINK_STDERR: u8 = 1;
const SINK_CAPTURE: u8 = 2;

static SINK: AtomicU8 = AtomicU8::new(SINK_STDOUT);

thread_local! {
    static CAPTURED: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Route subsequent script output to `sink`.
pub fn set_output_sink(sink: OutputSink) {
    let value = match sink {
        OutputSink::Stdout => SINK_STDOUT,
        OutputSink::Stderr => SINK_STDERR,
        OutputSink::Capture => SINK_CAPTURE,
    };
    SINK.store(value, Ordering::Relaxed);
}

/// The sink script output is currently routed to.
pub fn current_output_sink() -> OutputSink {
    match SINK.load(Ordering::Relaxed) {
        SINK_STDERR => OutputSink::Stderr,
        SINK_CAPTURE => OutputSink::Capture,
        _ => OutputSink::Stdout,
    }
}

/// Return and clear everything captured on this thread while the sink was
/// [`OutputSink::Capture`].
pub fn take_captured_output() -> String {
    CAPTURED.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// Write `text` to the active sink.
fn write_output(text: &str) {
    match SINK.load(Ordering::Relaxed) {
        SINK_CAPTURE => CAPTURED.with(|cell| cell.borrow_mut().push_str(text)),
        SINK_STDERR => {
            let mut err = std::io::stderr();
            // A closed or full stream must not abort a CAD script mid-model;
            // dropping the diagnostic text is the lesser failure.
            let _ = err.write_all(text.as_bytes());
            let _ = err.flush();
        }
        _ => {
            let mut out = std::io::stdout();
            let _ = out.write_all(text.as_bytes());
            let _ = out.flush();
        }
    }
}

/// Write an already-formatted string from a script.
///
/// The prelude's `puts` / `print` / `p` do all formatting in Ruby and call
/// this with the exact bytes to emit, so no newline handling happens here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rrcad_script_write(text: *const c_char, error_out: *mut *const c_char) {
    unsafe { *error_out = std::ptr::null() };
    if text.is_null() {
        return;
    }

    // On invalid UTF-8, utf8_arg has already reported through error_out.
    if let Some(s) = unsafe { super::native_helpers::utf8_arg(text, "output text", error_out) } {
        write_output(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Run `f` with output captured, returning what it wrote.
    fn captured(f: impl FnOnce()) -> String {
        set_output_sink(OutputSink::Capture);
        take_captured_output();
        f();
        let out = take_captured_output();
        set_output_sink(OutputSink::Stdout);
        out
    }

    #[test]
    fn write_appends_text_verbatim() {
        let out = captured(|| {
            let a = CString::new("hello\n").expect("cstring");
            let b = CString::new("world").expect("cstring");
            let mut err: *const c_char = std::ptr::null();
            unsafe { rrcad_script_write(a.as_ptr(), &mut err) };
            unsafe { rrcad_script_write(b.as_ptr(), &mut err) };
        });
        assert_eq!(out, "hello\nworld");
    }

    #[test]
    fn null_text_is_ignored() {
        let out = captured(|| {
            let mut err: *const c_char = std::ptr::null();
            unsafe { rrcad_script_write(std::ptr::null(), &mut err) };
        });
        assert!(out.is_empty(), "null pointer should write nothing");
    }

    #[test]
    fn take_captured_output_clears_the_buffer() {
        set_output_sink(OutputSink::Capture);
        take_captured_output();
        let text = CString::new("once").expect("cstring");
        let mut err: *const c_char = std::ptr::null();
        unsafe { rrcad_script_write(text.as_ptr(), &mut err) };

        assert_eq!(take_captured_output(), "once");
        assert_eq!(take_captured_output(), "", "buffer should be drained");
        set_output_sink(OutputSink::Stdout);
    }
}
