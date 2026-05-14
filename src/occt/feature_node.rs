use super::FeatureOp;
use crate::occt::Shape;
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Debug)]
pub(crate) struct FeatureNode {
    id: u64,
    op: FeatureOp,
    parents: Vec<Arc<FeatureNode>>,
    history_entry: String,
}

static FEATURE_NODE_SEQ: AtomicU64 = AtomicU64::new(1);

impl FeatureNode {
    pub(crate) fn new(
        op: FeatureOp,
        parents: Vec<Arc<FeatureNode>>,
        history_entry: String,
    ) -> Arc<Self> {
        debug_assert!(
            op.parents_are_valid(parents.len()),
            "feature node parent arity mismatch"
        );
        Arc::new(Self {
            id: FEATURE_NODE_SEQ.fetch_add(1, Ordering::Relaxed),
            op,
            parents,
            history_entry,
        })
    }

    fn label(&self) -> String {
        self.op.name()
    }

    pub(crate) fn snapshot_lines(&self, out: &mut Vec<String>, seen: &mut BTreeSet<u64>) {
        if !seen.insert(self.id) {
            return;
        }
        for parent in &self.parents {
            parent.snapshot_lines(out, seen);
        }
        let parent_ids = self
            .parents
            .iter()
            .map(|p| p.id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out.push(format!(
            "{}\t{}\t{}\t{}",
            self.id,
            parent_ids,
            self.label(),
            self.history_entry.replace('\t', " ")
        ));
    }

    pub(crate) fn rebuild(&self) -> Result<Shape, String> {
        match &self.op {
            FeatureOp::Box { dx, dy, dz } => Shape::make_box(*dx, *dy, *dz),
            FeatureOp::Cylinder { radius, height } => Shape::make_cylinder(*radius, *height),
            FeatureOp::Sphere { radius } => Shape::make_sphere(*radius),
            FeatureOp::Cone { r1, r2, height } => Shape::make_cone(*r1, *r2, *height),
            FeatureOp::Torus { r1, r2 } => Shape::make_torus(*r1, *r2),
            FeatureOp::Wedge { dx, dy, dz, ltx } => Shape::make_wedge(*dx, *dy, *dz, *ltx),
            FeatureOp::Rect { w, h } => Shape::make_rect(*w, *h),
            FeatureOp::Circle { r } => Shape::make_circle_face(*r),
            FeatureOp::Polygon { points } => Shape::make_polygon(points),
            FeatureOp::Ellipse { rx, ry } => Shape::make_ellipse_face(*rx, *ry),
            FeatureOp::Arc {
                r,
                start_deg,
                end_deg,
            } => Shape::make_arc(*r, *start_deg, *end_deg),
            FeatureOp::Translate { dx, dy, dz } => self
                .parents
                .first()
                .ok_or_else(|| "translate feature missing parent".to_string())?
                .rebuild()?
                .translate(*dx, *dy, *dz),
            FeatureOp::Rotate {
                axis_x,
                axis_y,
                axis_z,
                angle_deg,
            } => self
                .parents
                .first()
                .ok_or_else(|| "rotate feature missing parent".to_string())?
                .rebuild()?
                .rotate(*axis_x, *axis_y, *axis_z, *angle_deg),
            FeatureOp::Scale { factor } => self
                .parents
                .first()
                .ok_or_else(|| "scale feature missing parent".to_string())?
                .rebuild()?
                .scale(*factor),
            FeatureOp::ScaleXyz { sx, sy, sz } => self
                .parents
                .first()
                .ok_or_else(|| "scale_xyz feature missing parent".to_string())?
                .rebuild()?
                .scale_xyz(*sx, *sy, *sz),
            FeatureOp::Mirror { plane } => self
                .parents
                .first()
                .ok_or_else(|| "mirror feature missing parent".to_string())?
                .rebuild()?
                .mirror(plane),
            FeatureOp::Fuse => {
                let lhs = self
                    .parents
                    .first()
                    .ok_or_else(|| "fuse feature missing lhs parent".to_string())?
                    .rebuild()?;
                let rhs = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "fuse feature missing rhs parent".to_string())?
                    .rebuild()?;
                lhs.fuse(&rhs)
            }
            FeatureOp::Cut => {
                let lhs = self
                    .parents
                    .first()
                    .ok_or_else(|| "cut feature missing lhs parent".to_string())?
                    .rebuild()?;
                let rhs = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "cut feature missing rhs parent".to_string())?
                    .rebuild()?;
                lhs.cut(&rhs)
            }
            FeatureOp::Common => {
                let lhs = self
                    .parents
                    .first()
                    .ok_or_else(|| "common feature missing lhs parent".to_string())?
                    .rebuild()?;
                let rhs = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "common feature missing rhs parent".to_string())?
                    .rebuild()?;
                lhs.common(&rhs)
            }
            FeatureOp::Extrude {
                height,
                twist_deg,
                scale,
            } => self
                .parents
                .first()
                .ok_or_else(|| "extrude feature missing parent".to_string())?
                .rebuild()?
                .extrude_ex(*height, *twist_deg, *scale),
            FeatureOp::Revolve { angle_deg } => self
                .parents
                .first()
                .ok_or_else(|| "revolve feature missing parent".to_string())?
                .rebuild()?
                .revolve(*angle_deg),
            FeatureOp::Spline2D { points, tangents } => match tangents {
                Some([t0x, t0z, t1x, t1z]) => {
                    Shape::make_spline_2d_tan(points, *t0x, *t0z, *t1x, *t1z)
                }
                None => Shape::make_spline_2d(points),
            },
            FeatureOp::Spline3D { points, tangents } => match tangents {
                Some([t0x, t0y, t0z, t1x, t1y, t1z]) => {
                    Shape::make_spline_3d_tan(points, *t0x, *t0y, *t0z, *t1x, *t1y, *t1z)
                }
                None => Shape::make_spline_3d(points),
            },
            FeatureOp::Helix {
                radius,
                pitch,
                height,
            } => Shape::make_helix(*radius, *pitch, *height),
            FeatureOp::Loft { ruled, .. } => {
                let rebuilt = self
                    .parents
                    .iter()
                    .map(|parent| parent.rebuild())
                    .collect::<Result<Vec<_>, _>>()?;
                let refs = rebuilt.iter().collect::<Vec<_>>();
                Shape::loft(&refs, *ruled)
            }
            FeatureOp::Shell { thickness } => self
                .parents
                .first()
                .ok_or_else(|| "shell feature missing parent".to_string())?
                .rebuild()?
                .shell(*thickness),
            FeatureOp::Offset { distance } => self
                .parents
                .first()
                .ok_or_else(|| "offset feature missing parent".to_string())?
                .rebuild()?
                .offset(*distance),
            FeatureOp::Offset2D { distance } => self
                .parents
                .first()
                .ok_or_else(|| "offset_2d feature missing parent".to_string())?
                .rebuild()?
                .offset_2d(*distance),
            FeatureOp::Simplify { min_feature_size } => self
                .parents
                .first()
                .ok_or_else(|| "simplify feature missing parent".to_string())?
                .rebuild()?
                .simplify(*min_feature_size),
            FeatureOp::Sweep => {
                let profile = self
                    .parents
                    .first()
                    .ok_or_else(|| "sweep feature missing profile parent".to_string())?
                    .rebuild()?;
                let path = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "sweep feature missing path parent".to_string())?
                    .rebuild()?;
                profile.sweep(&path)
            }
            FeatureOp::SweepGuide => {
                let profile = self
                    .parents
                    .first()
                    .ok_or_else(|| "sweep_guide feature missing profile parent".to_string())?
                    .rebuild()?;
                let path = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "sweep_guide feature missing path parent".to_string())?
                    .rebuild()?;
                let guide = self
                    .parents
                    .get(2)
                    .ok_or_else(|| "sweep_guide feature missing guide parent".to_string())?
                    .rebuild()?;
                profile.sweep_guide(&path, &guide)
            }
            FeatureOp::ImportStep { path } => Shape::import_step(path),
            FeatureOp::ImportStl { path } => Shape::import_stl(path),
            FeatureOp::PatternLinear { n, dx, dy, dz } => self
                .parents
                .first()
                .ok_or_else(|| "linear_pattern feature missing parent".to_string())?
                .rebuild()?
                .linear_pattern(*n, *dx, *dy, *dz),
            FeatureOp::PatternPolar { n, angle_deg } => self
                .parents
                .first()
                .ok_or_else(|| "polar_pattern feature missing parent".to_string())?
                .rebuild()?
                .polar_pattern(*n, *angle_deg),
            FeatureOp::FragmentAll { .. } => {
                let rebuilt = self
                    .parents
                    .iter()
                    .map(|parent| parent.rebuild())
                    .collect::<Result<Vec<_>, _>>()?;
                let refs = rebuilt.iter().collect::<Vec<_>>();
                Shape::fragment_all(&refs)
            }
            FeatureOp::ConvexHull => self
                .parents
                .first()
                .ok_or_else(|| "convex_hull feature missing parent".to_string())?
                .rebuild()?
                .convex_hull(),
            FeatureOp::PathPattern { n } => {
                let profile = self
                    .parents
                    .first()
                    .ok_or_else(|| "path_pattern feature missing profile parent".to_string())?
                    .rebuild()?;
                let path = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "path_pattern feature missing path parent".to_string())?
                    .rebuild()?;
                profile.path_pattern(&path, *n)
            }
            FeatureOp::Slice { plane, offset } => self
                .parents
                .first()
                .ok_or_else(|| "slice feature missing parent".to_string())?
                .rebuild()?
                .slice(plane, *offset),
            FeatureOp::Fillet { radius } => self
                .parents
                .first()
                .ok_or_else(|| "fillet feature missing parent".to_string())?
                .rebuild()?
                .fillet(*radius),
            FeatureOp::FilletSel { radius, selector } => self
                .parents
                .first()
                .ok_or_else(|| "fillet_sel feature missing parent".to_string())?
                .rebuild()?
                .fillet_sel(*radius, selector),
            FeatureOp::FilletVar { r1, r2 } => self
                .parents
                .first()
                .ok_or_else(|| "fillet_var feature missing parent".to_string())?
                .rebuild()?
                .fillet_var(*r1, *r2),
            FeatureOp::FilletVarSel { r1, r2, selector } => self
                .parents
                .first()
                .ok_or_else(|| "fillet_var_sel feature missing parent".to_string())?
                .rebuild()?
                .fillet_var_sel(*r1, *r2, selector),
            FeatureOp::Chamfer { dist } => self
                .parents
                .first()
                .ok_or_else(|| "chamfer feature missing parent".to_string())?
                .rebuild()?
                .chamfer(*dist),
            FeatureOp::ChamferSel { dist, selector } => self
                .parents
                .first()
                .ok_or_else(|| "chamfer_sel feature missing parent".to_string())?
                .rebuild()?
                .chamfer_sel(*dist, selector),
            FeatureOp::ChamferAsym { d1, d2 } => self
                .parents
                .first()
                .ok_or_else(|| "chamfer_asym feature missing parent".to_string())?
                .rebuild()?
                .chamfer_asym(*d1, *d2),
            FeatureOp::ChamferAsymSel { d1, d2, selector } => self
                .parents
                .first()
                .ok_or_else(|| "chamfer_asym_sel feature missing parent".to_string())?
                .rebuild()?
                .chamfer_asym_sel(*d1, *d2, selector),
            FeatureOp::Pad { height } => {
                let body = self
                    .parents
                    .first()
                    .ok_or_else(|| "pad feature missing body parent".to_string())?
                    .rebuild()?;
                let face = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "pad feature missing face parent".to_string())?
                    .rebuild()?;
                let sketch = self
                    .parents
                    .get(2)
                    .ok_or_else(|| "pad feature missing sketch parent".to_string())?
                    .rebuild()?;
                body.pad(&face, &sketch, *height)
            }
            FeatureOp::Pocket { depth } => {
                let body = self
                    .parents
                    .first()
                    .ok_or_else(|| "pocket feature missing body parent".to_string())?
                    .rebuild()?;
                let face = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "pocket feature missing face parent".to_string())?
                    .rebuild()?;
                let sketch = self
                    .parents
                    .get(2)
                    .ok_or_else(|| "pocket feature missing sketch parent".to_string())?
                    .rebuild()?;
                body.pocket(&face, &sketch, *depth)
            }
            FeatureOp::FilletWire { radius } => self
                .parents
                .first()
                .ok_or_else(|| "fillet_wire feature missing parent".to_string())?
                .rebuild()?
                .fillet_wire(*radius),
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
            } => Shape::make_datum_plane(*ox, *oy, *oz, *nx, *ny, *nz, *xx, *xy, *xz),
            FeatureOp::ExtrudeDraft { height, draft_deg } => self
                .parents
                .first()
                .ok_or_else(|| "extrude_draft feature missing parent".to_string())?
                .rebuild()?
                .extrude_draft(*height, *draft_deg),
            FeatureOp::BezierPatch { points } => Shape::make_bezier_patch(points),
            FeatureOp::Sew { tolerance, .. } => {
                let rebuilt = self
                    .parents
                    .iter()
                    .map(|parent| parent.rebuild())
                    .collect::<Result<Vec<_>, _>>()?;
                let refs = rebuilt.iter().collect::<Vec<_>>();
                Shape::sew(&refs, *tolerance)
            }
            FeatureOp::SweepSections { .. } => {
                let rebuilt = self
                    .parents
                    .iter()
                    .map(|parent| parent.rebuild())
                    .collect::<Result<Vec<_>, _>>()?;
                let (path, profiles) = rebuilt
                    .split_last()
                    .ok_or_else(|| "sweep_sections feature missing parents".to_string())?;
                let refs: Vec<&Shape> = profiles.iter().collect();
                Shape::sweep_sections(&refs, path)
            }
            FeatureOp::RuledSurface => {
                let a = self
                    .parents
                    .first()
                    .ok_or_else(|| "ruled_surface feature missing first parent".to_string())?
                    .rebuild()?;
                let b = self
                    .parents
                    .get(1)
                    .ok_or_else(|| "ruled_surface feature missing second parent".to_string())?
                    .rebuild()?;
                Shape::ruled_surface(&a, &b)
            }
            FeatureOp::FillSurface => self
                .parents
                .first()
                .ok_or_else(|| "fill_surface feature missing parent".to_string())?
                .rebuild()
                .and_then(|boundary| Shape::fill_surface(&boundary)),
            FeatureOp::Opaque { label } => Err(format!(
                "cannot rebuild opaque feature '{label}' from history entry: {}",
                self.history_entry
            )),
        }
    }
}
