use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    // --- Phase 3: splines and sweep ---

    pub fn make_spline_2d(pts: &[f64]) -> Result<Self, String> {
        ffi::make_spline_2d(pts)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Spline2D {
                        points: pts.to_vec(),
                        tangents: None,
                    },
                    format!("spline_2d(points={})", pts.len() / 2),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_spline_3d(pts: &[f64]) -> Result<Self, String> {
        ffi::make_spline_3d(pts)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Spline3D {
                        points: pts.to_vec(),
                        tangents: None,
                    },
                    format!("spline_3d(points={})", pts.len() / 3),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Like `make_spline_2d` but with explicit start/end tangent vectors in
    /// the XZ plane - suppresses natural-boundary oscillation on short splines.
    pub fn make_spline_2d_tan(
        pts: &[f64],
        t0x: f64,
        t0z: f64,
        t1x: f64,
        t1z: f64,
    ) -> Result<Self, String> {
        ffi::make_spline_2d_tan(pts, t0x, t0z, t1x, t1z)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Spline2D {
                        points: pts.to_vec(),
                        tangents: Some([t0x, t0z, t1x, t1z]),
                    },
                    format!(
                        "spline_2d_tan(points={}, t0=({}, {}), t1=({}, {}))",
                        pts.len() / 2,
                        t0x,
                        t0z,
                        t1x,
                        t1z
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Like `make_spline_3d` but with explicit start/end tangent vectors -
    /// suppresses natural-boundary oscillation on short splines.
    pub fn make_spline_3d_tan(
        pts: &[f64],
        t0x: f64,
        t0y: f64,
        t0z: f64,
        t1x: f64,
        t1y: f64,
        t1z: f64,
    ) -> Result<Self, String> {
        ffi::make_spline_3d_tan(pts, t0x, t0y, t0z, t1x, t1y, t1z)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Spline3D {
                        points: pts.to_vec(),
                        tangents: Some([t0x, t0y, t0z, t1x, t1y, t1z]),
                    },
                    format!(
                        "spline_3d_tan(points={}, t0=({}, {}, {}), t1=({}, {}, {}))",
                        pts.len() / 3,
                        t0x,
                        t0y,
                        t0z,
                        t1x,
                        t1y,
                        t1z
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn sweep(&self, path: &Shape) -> Result<Shape, String> {
        ffi::shape_sweep(&self.inner, &path.inner)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Sweep,
                    format!("sweep(profile={}, path={})", summarize(self), summarize(path)),
                    vec![self.feature.clone(), path.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "sweep({}, path={}) failed: {e}{}",
                        summarize(self),
                        summarize(path),
                        hint("profile must be a Face or Wire and path must be a Wire; check the path doesn't kink sharply against the profile size")
                    ),
                    "sweep",
                    &[("profile", self), ("path", path)],
                )
            })
    }

    /// Variable-section pipe sweep using BRepOffsetAPI_MakePipeShell.
    /// `path` is a Wire (from `spline_3d`); `profiles` are Faces, Wires, or
    /// Vertices distributed along the spine (first at start, last at end).
    /// Requires at least 2 profiles.
    pub fn sweep_sections(profiles: &[&Shape], path: &Shape) -> Result<Shape, String> {
        if profiles.len() < 2 {
            return Err("sweep_sections requires at least 2 profiles".to_string());
        }
        let n = profiles.len();
        let ctx = || format!("sweep_sections(profiles={n}, path={})", summarize(path));
        let mut builder =
            ffi::pipe_shell_new(&path.inner).map_err(|e| format!("{} failed: {e}", ctx()))?;
        for (i, p) in profiles.iter().enumerate() {
            ffi::pipe_shell_add(builder.pin_mut(), &p.inner).map_err(|e| {
                format!(
                    "{} failed adding profile {} ({}): {e}",
                    ctx(),
                    i,
                    summarize(p)
                )
            })?;
        }
        ffi::pipe_shell_build(builder.pin_mut())
            .map(|p| {
                let profile_summary = profiles
                    .iter()
                    .map(|s| summarize(s))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut parents = profiles
                    .iter()
                    .map(|s| s.feature.clone())
                    .collect::<Vec<_>>();
                parents.push(path.feature.clone());
                Shape::fresh_with_feature_parents(
                    p,
                    FeatureOp::SweepSections {
                        profile_count: profiles.len(),
                    },
                    format!(
                        "sweep_sections(profiles=[{profile_summary}], path={})",
                        summarize(path)
                    ),
                    parents,
                )
            })
            .map_err(|e| format!("{} failed: {e}", ctx()))
    }
}
