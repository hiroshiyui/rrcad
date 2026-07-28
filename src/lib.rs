/// rrcad library — exposes the geometry, Ruby VM, preview, and MCP layers so
/// integration tests (and future embedders) can import them without going
/// through main.rs.
pub mod mcp;
pub mod occt;
pub mod preview;
pub mod project_config;
pub mod ruby;

/// Helpers shared by unit tests across modules.
#[cfg(test)]
pub(crate) mod test_util {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_SEQ: AtomicUsize = AtomicUsize::new(1);

    /// Return a unique, not-yet-created temp-dir path for a test.
    ///
    /// Combines the process id with a process-wide counter so repeated runs
    /// and multiple tests in one binary never collide.
    pub(crate) fn unique_test_dir(prefix: &str) -> PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()))
    }
}
