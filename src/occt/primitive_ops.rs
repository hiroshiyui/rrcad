use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    pub fn make_box(dx: f64, dy: f64, dz: f64) -> Result<Self, String> {
        ffi::make_box(dx, dy, dz)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Box { dx, dy, dz },
                    format!("box(dx={dx}, dy={dy}, dz={dz})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_cylinder(radius: f64, height: f64) -> Result<Self, String> {
        ffi::make_cylinder(radius, height)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Cylinder { radius, height },
                    format!("cylinder(radius={radius}, height={height})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_sphere(radius: f64) -> Result<Self, String> {
        ffi::make_sphere(radius)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Sphere { radius },
                    format!("sphere(radius={radius})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_cone(r1: f64, r2: f64, height: f64) -> Result<Self, String> {
        ffi::make_cone(r1, r2, height)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Cone { r1, r2, height },
                    format!("cone(r1={r1}, r2={r2}, height={height})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_torus(r1: f64, r2: f64) -> Result<Self, String> {
        ffi::make_torus(r1, r2)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Torus { r1, r2 },
                    format!("torus(r1={r1}, r2={r2})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64) -> Result<Self, String> {
        ffi::make_wedge(dx, dy, dz, ltx)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Wedge { dx, dy, dz, ltx },
                    format!("wedge(dx={dx}, dy={dy}, dz={dz}, ltx={ltx})"),
                )
            })
            .map_err(|e| e.to_string())
    }

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

    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Result<Shape, String> {
        ffi::shape_translate(&self.inner, dx, dy, dz)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Translate { dx, dy, dz },
                    format!("translate(dx={dx}, dy={dy}, dz={dz})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn rotate(&self, ax: f64, ay: f64, az: f64, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_rotate(&self.inner, ax, ay, az, angle_deg)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Rotate {
                        axis_x: ax,
                        axis_y: ay,
                        axis_z: az,
                        angle_deg,
                    },
                    format!("rotate(axis=({ax}, {ay}, {az}), angle_deg={angle_deg})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn scale(&self, factor: f64) -> Result<Shape, String> {
        ffi::shape_scale(&self.inner, factor)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Scale { factor },
                    format!("scale(factor={factor})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn scale_xyz(&self, sx: f64, sy: f64, sz: f64) -> Result<Shape, String> {
        ffi::shape_scale_xyz(&self.inner, sx, sy, sz)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::ScaleXyz { sx, sy, sz },
                    format!("scale_xyz(sx={sx}, sy={sy}, sz={sz})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn mirror(&self, plane: &str) -> Result<Shape, String> {
        ffi::shape_mirror(&self.inner, plane)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Mirror {
                        plane: plane.to_string(),
                    },
                    format!("mirror(plane={plane})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_rect(w: f64, h: f64) -> Result<Self, String> {
        ffi::make_rect(w, h)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Rect { w, h },
                    format!("rect(w={w}, h={h})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_circle_face(r: f64) -> Result<Self, String> {
        ffi::make_circle_face(r)
            .map(|p| {
                Shape::fresh_with_feature(p, FeatureOp::Circle { r }, format!("circle(r={r})"))
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_polygon(pts: &[f64]) -> Result<Self, String> {
        ffi::make_polygon(pts)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Polygon {
                        points: pts.to_vec(),
                    },
                    format!("polygon(points={})", pts.len() / 2),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_ellipse_face(rx: f64, ry: f64) -> Result<Self, String> {
        ffi::make_ellipse_face(rx, ry)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Ellipse { rx, ry },
                    format!("ellipse(rx={rx}, ry={ry})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_arc(r: f64, start_deg: f64, end_deg: f64) -> Result<Self, String> {
        ffi::make_arc(r, start_deg, end_deg)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Arc {
                        r,
                        start_deg,
                        end_deg,
                    },
                    format!("arc(r={r}, start_deg={start_deg}, end_deg={end_deg})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn extrude(&self, height: f64) -> Result<Shape, String> {
        ffi::shape_extrude(&self.inner, height)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Extrude {
                        height,
                        twist_deg: 0.0,
                        scale: 1.0,
                    },
                    format!("extrude(height={height})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "extrude(h={height}) on {} failed: {e}{}",
                        summarize(self),
                        hint(
                            "extrude requires a 2-D profile (Face or Wire); a Solid cannot be extruded"
                        )
                    ),
                    "extrude",
                    &[("input", self)],
                )
            })
    }

    pub fn revolve(&self, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_revolve(&self.inner, angle_deg)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Revolve { angle_deg },
                    format!("revolve(angle_deg={angle_deg})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "revolve(angle={angle_deg}°) on {} failed: {e}",
                        summarize(self)
                    ),
                    "revolve",
                    &[("input", self)],
                )
            })
    }
}
