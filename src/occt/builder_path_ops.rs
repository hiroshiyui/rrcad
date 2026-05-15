use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    pub fn path_pattern(&self, path: &Shape, n: i32) -> Result<Shape, String> {
        ffi::shape_path_pattern(&self.inner, &path.inner, n)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::PathPattern { n },
                    format!("path_pattern(path={}, n={n})", summarize(path)),
                    vec![self.feature.clone(), path.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "path_pattern(shape={}, path={}, n={n}) failed: {e}",
                        summarize(self),
                        summarize(path)
                    ),
                    "path_pattern",
                    &[("input", self), ("path", path)],
                )
            })
    }

    pub fn sweep_guide(&self, path: &Shape, guide: &Shape) -> Result<Shape, String> {
        ffi::shape_sweep_guide(&self.inner, &path.inner, &guide.inner)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::SweepGuide,
                    format!(
                        "sweep_guide(profile={}, path={}, guide={})",
                        summarize(self),
                        summarize(path),
                        summarize(guide)
                    ),
                    vec![
                        self.feature.clone(),
                        path.feature.clone(),
                        guide.feature.clone(),
                    ],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "sweep(profile={}, path={}, guide={}) failed: {e}{}",
                        summarize(self),
                        summarize(path),
                        summarize(guide),
                        hint("profile must be a Face or Wire; path and guide must both be Wires that don't kink sharply")
                    ),
                    "sweep_guide",
                    &[("profile", self), ("path", path), ("guide", guide)],
                )
            })
    }
}
