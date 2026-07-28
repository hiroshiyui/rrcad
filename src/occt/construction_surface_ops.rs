use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    // --- Phase 4: loft (ThruSections builder pattern) ---

    /// Loft through a sequence of planar profiles (Faces or Wires).
    /// `ruled=true` produces a ruled surface (straight lines between sections).
    pub fn loft(profiles: &[&Shape], ruled: bool) -> Result<Shape, String> {
        let n = profiles.len();
        let ctx = || format!("loft(profiles={n}, ruled={ruled})");
        let mut builder =
            ffi::thru_sections_new(true, ruled).map_err(|e| format!("{} failed: {e}", ctx()))?;
        for (i, p) in profiles.iter().enumerate() {
            ffi::thru_sections_add(builder.pin_mut(), &p.inner).map_err(|e| {
                format!(
                    "{} failed adding profile {} ({}): {e}",
                    ctx(),
                    i,
                    summarize(p)
                )
            })?;
        }
        ffi::thru_sections_build(builder.pin_mut())
            .map(|p| {
                let profile_summary = profiles
                    .iter()
                    .map(|s| summarize(s))
                    .collect::<Vec<_>>()
                    .join(", ");
                Shape::fresh_with_feature_parents(
                    p,
                    FeatureOp::Loft {
                        ruled,
                        profile_count: profiles.len(),
                    },
                    format!("loft(profiles=[{profile_summary}], ruled={ruled})"),
                    profiles.iter().map(|s| s.feature.clone()).collect(),
                )
            })
            .map_err(|e| format!("{} failed: {e}", ctx()))
    }

    // --- Phase 4: 3-D operations ---

    /// Hollow out the solid, removing the topmost face and offsetting inward
    /// by `thickness`.  Wraps BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin.
    pub fn shell(&self, thickness: f64) -> Result<Shape, String> {
        ffi::shape_shell(&self.inner, thickness)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Shell { thickness },
                    format!("shell(thickness={thickness})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "shell(thickness={thickness}) on {} failed: {e}{}",
                        summarize(self),
                        hint("thickness must be smaller than the part's smallest dimension; reduce thickness or shell with a specific face removed")
                    ),
                    "shell",
                    &[("input", self)],
                )
            })
    }

    /// Inflate (positive) or deflate (negative) the solid uniformly.
    /// Wraps BRepOffsetAPI_MakeOffsetShape::PerformByJoin.
    pub fn offset(&self, distance: f64) -> Result<Shape, String> {
        ffi::shape_offset(&self.inner, distance)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Offset { distance },
                    format!("offset(distance={distance})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "offset(distance={distance}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "offset",
                    &[("input", self)],
                )
            })
    }

    /// Offset a 2D Wire or Face inward (negative) or outward (positive) in its own plane.
    ///
    /// A Face returns a Face — the offset wires are rebuilt into a planar
    /// profile, so the result still extrudes, pads, and pockets. Profiles with
    /// holes are supported; growing the material shrinks the holes. Errors if an
    /// inward offset consumes the whole profile.
    pub fn offset_2d(&self, distance: f64) -> Result<Shape, String> {
        ffi::shape_offset_2d(&self.inner, distance)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Offset2D { distance },
                    format!("offset_2d(distance={distance})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "offset_2d(distance={distance}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "offset_2d",
                    &[("input", self)],
                )
            })
    }

    /// Remove small holes and fillets.  Faces with area < min_feature_size²
    /// are passed to BRepAlgoAPI_Defeaturing.  Returns the original shape
    /// unchanged if no faces qualify.
    pub fn simplify(&self, min_feature_size: f64) -> Result<Shape, String> {
        ffi::shape_simplify(&self.inner, min_feature_size)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Simplify { min_feature_size },
                    format!("simplify(min_feature_size={min_feature_size})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "simplify(min_feature_size={min_feature_size}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "simplify",
                    &[("input", self)],
                )
            })
    }

    /// Extrude with optional end-twist (degrees) and end-scale.
    /// Falls back to MakePrism for the zero-twist/unity-scale case.
    pub fn extrude_ex(&self, height: f64, twist_deg: f64, scale: f64) -> Result<Shape, String> {
        ffi::shape_extrude_ex(&self.inner, height, twist_deg, scale)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Extrude {
                        height,
                        twist_deg,
                        scale,
                    },
                    format!("extrude_ex(height={height}, twist_deg={twist_deg}, scale={scale})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "extrude(h={height}, twist={twist_deg}°, scale={scale}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "extrude_ex",
                    &[("input", self)],
                )
            })
    }

    // --- Bézier surface patch ---

    /// Build a single bicubic Bézier face from 16 control points.
    /// `pts` must be a flat slice of 48 doubles: 16 points × (x, y, z) in row-major order.
    pub fn make_bezier_patch(pts: &[f64]) -> Result<Self, String> {
        ffi::make_bezier_patch(pts)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::BezierPatch {
                        points: pts.to_vec(),
                    },
                    format!("bezier_patch(points={})", pts.len() / 3),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Sew a collection of Faces (or Shells) into a closed Shell / Solid.
    /// `tolerance` controls how close shared edges need to be to be sewn together.
    /// Uses `BRepBuilderAPI_Sewing` followed by `BRepBuilderAPI_MakeSolid`.
    pub fn sew(faces: &[&Shape], tolerance: f64) -> Result<Self, String> {
        let mut builder = ffi::sewing_new(tolerance).map_err(|e| e.to_string())?;
        for face in faces {
            ffi::sewing_add(builder.pin_mut(), &face.inner).map_err(|e| e.to_string())?;
        }
        ffi::sewing_build(builder.pin_mut())
            .map(|p| {
                let face_summary = faces
                    .iter()
                    .map(|s| summarize(s))
                    .collect::<Vec<_>>()
                    .join(", ");
                let parents = faces.iter().map(|s| s.feature.clone()).collect();
                Shape::fresh_with_feature_parents(
                    p,
                    FeatureOp::Sew {
                        face_count: faces.len(),
                        tolerance,
                    },
                    format!("sew(faces=[{face_summary}], tolerance={tolerance})"),
                    parents,
                )
            })
            .map_err(|e| e.to_string())
    }
}
