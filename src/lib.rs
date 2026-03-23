/// rrcad library — exposes the geometry, Ruby VM, preview, and MCP layers so
/// integration tests (and future embedders) can import them without going
/// through main.rs.
pub mod mcp;
pub mod occt;
pub mod preview;
pub mod ruby;
