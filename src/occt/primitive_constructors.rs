use super::{FeatureOp, Shape, ffi};

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
}
