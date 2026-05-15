use super::{FeatureOp, Shape, ffi};

impl Shape {
    pub fn linear_pattern(&self, n: i32, dx: f64, dy: f64, dz: f64) -> Result<Shape, String> {
        ffi::shape_linear_pattern(&self.inner, n, dx, dy, dz)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::PatternLinear { n, dx, dy, dz },
                    format!("linear_pattern(n={n}, dx={dx}, dy={dy}, dz={dz})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn polar_pattern(&self, n: i32, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_polar_pattern(&self.inner, n, angle_deg)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::PatternPolar { n, angle_deg },
                    format!("polar_pattern(n={n}, angle_deg={angle_deg})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn grid_pattern(&self, nx: i32, ny: i32, dx: f64, dy: f64) -> Result<Shape, String> {
        if nx < 1 || ny < 1 {
            return Err("grid_pattern: nx and ny must be >= 1".to_string());
        }
        let row = self.linear_pattern(nx, dx, 0.0, 0.0)?;
        row.linear_pattern(ny, 0.0, dy, 0.0)
    }
}
