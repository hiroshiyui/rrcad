use super::{FeatureOp, Shape, ffi};

impl Shape {
    /// Axis-aligned box with edge lengths `dx` x `dy` x `dz`, corner at the origin.
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

    /// Cylinder of the given `radius` and `height`, base centred on the origin, axis +Z.
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

    /// Sphere of the given `radius`, centred on the origin.
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

    /// Truncated cone from base radius `r1` to top radius `r2` over `height`, axis +Z.
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

    /// Torus with major (ring) radius `r1` and minor (tube) radius `r2`, axis +Z.
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

    /// Wedge (tapered box) of size `dx` x `dy` x `dz`; `ltx` is the top-face length along X.
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
