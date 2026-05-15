use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    pub fn fillet(&self, radius: f64) -> Result<Shape, String> {
        ffi::shape_fillet(&self.inner, radius)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Fillet { radius },
                    format!("fillet(radius={radius})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet(r={radius}) on {} failed: {e}{}",
                        summarize(self),
                        hint("radius likely exceeds the smallest adjacent face/edge; try a smaller value or use fillet_sel with an edge selector")
                    ),
                    "fillet",
                    &[("input", self)],
                )
            })
    }

    pub fn chamfer(&self, dist: f64) -> Result<Shape, String> {
        ffi::shape_chamfer(&self.inner, dist)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Chamfer { dist },
                    format!("chamfer(dist={dist})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer(d={dist}) on {} failed: {e}{}",
                        summarize(self),
                        hint("distance likely exceeds an adjacent face dimension; try a smaller value or use chamfer_sel with an edge selector")
                    ),
                    "chamfer",
                    &[("input", self)],
                )
            })
    }

    pub fn fillet_sel(&self, radius: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_fillet_sel(&self.inner, radius, selector)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::FilletSel {
                        radius,
                        selector: selector.to_string(),
                    },
                    format!("fillet(radius={radius}, selector={selector})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet(r={radius}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "fillet_sel",
                    &[("input", self)],
                )
            })
    }

    pub fn chamfer_sel(&self, dist: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_chamfer_sel(&self.inner, dist, selector)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::ChamferSel {
                        dist,
                        selector: selector.to_string(),
                    },
                    format!("chamfer(dist={dist}, selector={selector})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer(d={dist}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "chamfer_sel",
                    &[("input", self)],
                )
            })
    }

    pub fn fillet_var(&self, r1: f64, r2: f64) -> Result<Shape, String> {
        ffi::shape_fillet_var(&self.inner, r1, r2)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::FilletVar { r1, r2 },
                    format!("fillet_var(r1={r1}, r2={r2})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet_var(r1={r1}, r2={r2}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "fillet_var",
                    &[("input", self)],
                )
            })
    }

    pub fn fillet_var_sel(&self, r1: f64, r2: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_fillet_var_sel(&self.inner, r1, r2, selector)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::FilletVarSel {
                        r1,
                        r2,
                        selector: selector.to_string(),
                    },
                    format!("fillet_var(r1={r1}, r2={r2}, selector={selector})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet_var(r1={r1}, r2={r2}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "fillet_var_sel",
                    &[("input", self)],
                )
            })
    }

    pub fn chamfer_asym(&self, d1: f64, d2: f64) -> Result<Shape, String> {
        ffi::shape_chamfer_asym(&self.inner, d1, d2)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::ChamferAsym { d1, d2 },
                    format!("chamfer_asym(d1={d1}, d2={d2})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer_asym(d1={d1}, d2={d2}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "chamfer_asym",
                    &[("input", self)],
                )
            })
    }

    pub fn chamfer_asym_sel(&self, d1: f64, d2: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_chamfer_asym_sel(&self.inner, d1, d2, selector)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::ChamferAsymSel {
                        d1,
                        d2,
                        selector: selector.to_string(),
                    },
                    format!("chamfer_asym(d1={d1}, d2={d2}, selector={selector})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer_asym(d1={d1}, d2={d2}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "chamfer_asym_sel",
                    &[("input", self)],
                )
            })
    }
}
