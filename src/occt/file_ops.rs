use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    // --- Import ---

    pub fn import_step(path: &str) -> Result<Self, String> {
        ffi::import_step(path)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::ImportStep {
                        path: path.to_string(),
                    },
                    format!("import_step(path={path:?})"),
                )
            })
            .map_err(|e| {
                format!(
                    "import_step({path:?}) failed: {e}{}",
                    hint("check that the path exists and is readable; STEP files end in .step or .stp")
                )
            })
    }

    pub fn import_stl(path: &str) -> Result<Self, String> {
        ffi::import_stl(path)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::ImportStl {
                        path: path.to_string(),
                    },
                    format!("import_stl(path={path:?})"),
                )
            })
            .map_err(|e| {
                format!(
                    "import_stl({path:?}) failed: {e}{}",
                    hint("check that the path exists and is readable; STL files end in .stl")
                )
            })
    }

    // --- Export ---

    pub fn export_step(&self, path: &str) -> Result<(), String> {
        ffi::export_step(&self.inner, path).map_err(|e| {
            self.fail_with_debug(
                format!("export_step({path:?}) on {} failed: {e}", summarize(self)),
                "export_step",
                &[("input", self)],
            )
        })
    }

    pub fn export_stl(&self, path: &str) -> Result<(), String> {
        ffi::export_stl(&self.inner, path).map_err(|e| {
            self.fail_with_debug(
                format!("export_stl({path:?}) on {} failed: {e}", summarize(self)),
                "export_stl",
                &[("input", self)],
            )
        })
    }

    /// Export to glTF. `linear_deflection` controls tessellation quality (e.g. `0.1` for 0.1 mm).
    pub fn export_gltf(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_gltf(&self.inner, path, linear_deflection).map_err(|e| {
            self.fail_with_debug(
                format!("export_gltf({path:?}) on {} failed: {e}", summarize(self)),
                "export_gltf",
                &[("input", self)],
            )
        })
    }

    /// Export to binary glTF (GLB). Single-file format suitable for HTTP serving.
    pub fn export_glb(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_glb(&self.inner, path, linear_deflection).map_err(|e| {
            self.fail_with_debug(
                format!("export_glb({path:?}) on {} failed: {e}", summarize(self)),
                "export_glb",
                &[("input", self)],
            )
        })
    }

    /// Export to Wavefront OBJ. Tessellates with `linear_deflection` and writes
    /// the `.obj` file plus a companion `.mtl` material file in the same directory.
    pub fn export_obj(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_obj(&self.inner, path, linear_deflection).map_err(|e| {
            self.fail_with_debug(
                format!("export_obj({path:?}) on {} failed: {e}", summarize(self)),
                "export_obj",
                &[("input", self)],
            )
        })
    }
}
