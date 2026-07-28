//! mRuby binding layer: raw C FFI declarations (`ffi`), extern "C" entry
//! points called from `glue.c` (`native`), and the safe `MrubyVm` wrapper
//! (`vm`) that evaluates rrcad DSL scripts.

pub mod ffi;
pub mod native;
pub mod vm;

pub use native::native_output::{
    OutputSink, current_output_sink, set_output_sink, take_captured_output,
};
