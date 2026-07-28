//! mRuby binding layer: raw C FFI declarations (`ffi`), extern "C" entry
//! points called from `glue.c` (`native`), and the safe `MrubyVm` wrapper
//! (`vm`) that evaluates rrcad DSL scripts.

pub mod ffi;
pub mod native;
pub mod vm;
