use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
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
