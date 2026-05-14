use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::{hint, summarize};

impl Shape {
    // --- Phase 8 Tier 1: Core Part Design ---

    /// Extrude a `sketch` (Face/Wire in XY plane at Z=0) along `face_ref`'s outward normal
    /// by `height`, then fuse the resulting prism with `self` (the body).
    pub fn pad(&self, face_ref: &Shape, sketch: &Shape, height: f64) -> Result<Shape, String> {
        ffi::shape_pad(&self.inner, &face_ref.inner, &sketch.inner, height)
            .map(|s| {
                self.with_feature(
                    s,
                    FeatureOp::Pad { height },
                    format!(
                        "pad(face={}, sketch={}, height={height})",
                        summarize(face_ref),
                        summarize(sketch)
                    ),
                    vec![
                        self.feature.clone(),
                        face_ref.feature.clone(),
                        sketch.feature.clone(),
                    ],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "pad(h={height}, face={}, sketch={}) on {} failed: {e}{}",
                        summarize(face_ref),
                        summarize(sketch),
                        summarize(self),
                        hint("face must be a planar face of the body; sketch must lie in (or be transformed into) that face's plane")
                    ),
                    "pad",
                    &[("body", self), ("face", face_ref), ("sketch", sketch)],
                )
            })
    }

    /// Pocket: place sketch on face_ref, extrude along -normal by depth, cut from body.
    pub fn pocket(&self, face_ref: &Shape, sketch: &Shape, depth: f64) -> Result<Shape, String> {
        ffi::shape_pocket(&self.inner, &face_ref.inner, &sketch.inner, depth)
            .map(|s| {
                self.with_feature(
                    s,
                    FeatureOp::Pocket { depth },
                    format!(
                        "pocket(face={}, sketch={}, depth={depth})",
                        summarize(face_ref),
                        summarize(sketch)
                    ),
                    vec![
                        self.feature.clone(),
                        face_ref.feature.clone(),
                        sketch.feature.clone(),
                    ],
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "pocket(d={depth}, face={}, sketch={}) on {} failed: {e}{}",
                        summarize(face_ref),
                        summarize(sketch),
                        summarize(self),
                        hint("face must be a planar face of the body; sketch must lie in that face's plane")
                    ),
                    "pocket",
                    &[("body", self), ("face", face_ref), ("sketch", sketch)],
                )
            })
    }

    /// Fillet all corners of a 2D Wire or Face profile with radius.
    pub fn fillet_wire(&self, radius: f64) -> Result<Shape, String> {
        ffi::shape_fillet_wire(&self.inner, radius)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::FilletWire { radius },
                    format!("fillet_wire(radius={radius})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Construct a reference plane (Face) from 9 scalars: origin, normal, x_dir.
    #[allow(clippy::too_many_arguments)] // 9 params mirror OCCT's gp_Ax3(origin, normal, x_dir) exactly
    pub fn make_datum_plane(
        ox: f64,
        oy: f64,
        oz: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        xx: f64,
        xy: f64,
        xz: f64,
    ) -> Result<Shape, String> {
        ffi::make_datum_plane(ox, oy, oz, nx, ny, nz, xx, xy, xz)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::DatumPlane {
                        ox,
                        oy,
                        oz,
                        nx,
                        ny,
                        nz,
                        xx,
                        xy,
                        xz,
                    },
                    format!(
                        "datum_plane(origin=({ox}, {oy}, {oz}), normal=({nx}, {ny}, {nz}), x_dir=({xx}, {xy}, {xz}))"
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    // --- Phase 8 Tier 2: Manufacturing features ---

    /// Extrude `profile` to `height` then apply a draft angle of `draft_deg` degrees
    /// to all lateral (non-Z-normal) planar faces.
    pub fn extrude_draft(&self, height: f64, draft_deg: f64) -> Result<Shape, String> {
        ffi::shape_extrude_draft(&self.inner, height, draft_deg)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::ExtrudeDraft { height, draft_deg },
                    format!("extrude_draft(height={height}, draft_deg={draft_deg})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Construct a helical Wire path.
    /// `radius`: distance from Z axis; `pitch`: axial rise per revolution;
    /// `height`: total Z extent.
    pub fn make_helix(radius: f64, pitch: f64, height: f64) -> Result<Shape, String> {
        ffi::make_helix(radius, pitch, height)
            .map(|p| {
                Shape::fresh_with_feature(
                    p,
                    FeatureOp::Helix {
                        radius,
                        pitch,
                        height,
                    },
                    format!("helix(radius={radius}, pitch={pitch}, height={height})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    // --- Phase 7 Tier 3: Surface modeling ---

    /// Create a ruled surface (shell) between two wires.
    pub fn ruled_surface(wire_a: &Shape, wire_b: &Shape) -> Result<Shape, String> {
        ffi::shape_ruled_surface(&wire_a.inner, &wire_b.inner)
            .map(|p| {
                Shape::fresh_with_feature_parents(
                    p,
                    FeatureOp::RuledSurface,
                    format!(
                        "ruled_surface(wire_a={}, wire_b={})",
                        summarize(wire_a),
                        summarize(wire_b)
                    ),
                    vec![wire_a.feature.clone(), wire_b.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Fill the interior of a closed boundary wire with a smooth surface.
    pub fn fill_surface(boundary_wire: &Shape) -> Result<Shape, String> {
        ffi::shape_fill_surface(&boundary_wire.inner)
            .map(|p| {
                Shape::fresh_with_feature_parents(
                    p,
                    FeatureOp::FillSurface,
                    format!("fill_surface(boundary_wire={})", summarize(boundary_wire)),
                    vec![boundary_wire.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Cross-section of a shape by an axis-aligned plane.
    /// `plane` is a NUL-terminated C string: "xy", "xz", or "yz".
    pub fn slice(&self, plane: &str, offset: f64) -> Result<Shape, String> {
        ffi::shape_slice(&self.inner, plane, offset)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::Slice {
                        plane: plane.to_string(),
                        offset,
                    },
                    format!("slice(plane={plane}, offset={offset})"),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }
}
