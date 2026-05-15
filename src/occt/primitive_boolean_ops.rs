use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::summarize;

impl Shape {
    pub fn fuse(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_fuse(&self.inner, &other.inner)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Fuse,
                    format!("fuse(lhs={}, rhs={})", summarize(self), summarize(other)),
                    vec![self.feature.clone(), other.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fuse({}, {}) failed: {e}",
                        summarize(self),
                        summarize(other)
                    ),
                    "fuse",
                    &[("lhs", self), ("rhs", other)],
                )
            })
    }

    pub fn cut(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_cut(&self.inner, &other.inner)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Cut,
                    format!("cut(lhs={}, rhs={})", summarize(self), summarize(other)),
                    vec![self.feature.clone(), other.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!("cut({}, {}) failed: {e}", summarize(self), summarize(other)),
                    "cut",
                    &[("lhs", self), ("rhs", other)],
                )
            })
    }

    pub fn common(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_common(&self.inner, &other.inner)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Common,
                    format!("common(lhs={}, rhs={})", summarize(self), summarize(other)),
                    vec![self.feature.clone(), other.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "common({}, {}) failed: {e}",
                        summarize(self),
                        summarize(other)
                    ),
                    "common",
                    &[("lhs", self), ("rhs", other)],
                )
            })
    }
}
