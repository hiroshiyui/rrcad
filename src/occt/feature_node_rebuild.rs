use super::FeatureOp;
use crate::occt::{FeatureNode, Shape};

impl FeatureNode {
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
            FeatureOp::Profile2D {
                points,
                counts,
                kinds,
            } => Shape::make_profile_2d(points, counts, kinds),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn node(
        op: FeatureOp,
        parents: Vec<Arc<FeatureNode>>,
        history_entry: &str,
    ) -> Arc<FeatureNode> {
        FeatureNode::new(op, parents, history_entry.to_string())
    }

    fn leaf(op: FeatureOp, history_entry: &str) -> Arc<FeatureNode> {
        node(op, Vec::new(), history_entry)
    }

    #[test]
    fn rebuild_translates_feature_tree() {
        let base = node(
            FeatureOp::Box {
                dx: 10.0,
                dy: 20.0,
                dz: 30.0,
            },
            Vec::new(),
            "box(dx=10, dy=20, dz=30)",
        );
        let translated = node(
            FeatureOp::Translate {
                dx: 5.0,
                dy: 0.0,
                dz: 0.0,
            },
            vec![base],
            "translate(dx=5, dy=0, dz=0)",
        );

        let rebuilt = translated.rebuild().expect("translate rebuild failed");
        let bb = rebuilt.bounding_box().expect("bounding_box failed");
        assert!(
            (bb[0] - 5.0).abs() < 1.0e-6 && (bb[3] - 15.0).abs() < 1.0e-6,
            "expected translated box to shift by 5mm, got {bb:?}"
        );
    }

    #[test]
    fn rebuild_reports_missing_parent_and_opaque() {
        let missing_parent = FeatureNode {
            id: 1,
            op: FeatureOp::Translate {
                dx: 1.0,
                dy: 0.0,
                dz: 0.0,
            },
            parents: Vec::new(),
            history_entry: "translate(dx=1, dy=0, dz=0)".to_string(),
        };
        match missing_parent.rebuild() {
            Ok(_) => panic!("expected missing parent error"),
            Err(err) => assert_eq!(err, "translate feature missing parent"),
        }

        let opaque = FeatureNode {
            id: 2,
            op: FeatureOp::Opaque {
                label: "manual".to_string(),
            },
            parents: Vec::new(),
            history_entry: "manual".to_string(),
        };
        match opaque.rebuild() {
            Ok(_) => panic!("expected opaque rebuild error"),
            Err(err) => assert_eq!(
                err,
                "cannot rebuild opaque feature 'manual' from history entry: manual"
            ),
        }
    }

    #[test]
    fn rebuild_covers_common_feature_branches() {
        let box_a = leaf(
            FeatureOp::Box {
                dx: 10.0,
                dy: 20.0,
                dz: 30.0,
            },
            "box(dx=10, dy=20, dz=30)",
        );
        let box_b = leaf(
            FeatureOp::Box {
                dx: 6.0,
                dy: 8.0,
                dz: 10.0,
            },
            "box(dx=6, dy=8, dz=10)",
        );
        let face_a = leaf(FeatureOp::Rect { w: 10.0, h: 10.0 }, "rect(w=10, h=10)");
        let face_b = leaf(FeatureOp::Rect { w: 12.0, h: 12.0 }, "rect(w=12, h=12)");
        let face_c = leaf(FeatureOp::Circle { r: 5.0 }, "circle(r=5)");
        let path = leaf(
            FeatureOp::Spline3D {
                points: vec![
                    0.0, 0.0, 0.0, //
                    5.0, 0.0, 2.0, //
                    10.0, 0.0, 0.0,
                ],
                tangents: None,
            },
            "spline_3d",
        );
        let guide = leaf(
            FeatureOp::Spline3D {
                points: vec![
                    0.0, 0.0, 0.0, //
                    5.0, 2.0, 2.0, //
                    10.0, 0.0, 0.0,
                ],
                tangents: None,
            },
            "spline_3d_guide",
        );

        for node in [
            node(
                FeatureOp::Translate {
                    dx: 5.0,
                    dy: 0.0,
                    dz: 0.0,
                },
                vec![box_a.clone()],
                "translate(dx=5, dy=0, dz=0)",
            ),
            node(
                FeatureOp::Rotate {
                    axis_x: 0.0,
                    axis_y: 0.0,
                    axis_z: 1.0,
                    angle_deg: 45.0,
                },
                vec![box_a.clone()],
                "rotate(axis=1)",
            ),
            node(
                FeatureOp::Scale { factor: 2.0 },
                vec![box_a.clone()],
                "scale(factor=2)",
            ),
            node(
                FeatureOp::ScaleXyz {
                    sx: 2.0,
                    sy: 3.0,
                    sz: 4.0,
                },
                vec![box_a.clone()],
                "scale_xyz",
            ),
            node(
                FeatureOp::Mirror {
                    plane: "xy".to_string(),
                },
                vec![box_a.clone()],
                "mirror(plane=xy)",
            ),
            node(FeatureOp::Fuse, vec![box_a.clone(), box_b.clone()], "fuse"),
            node(FeatureOp::Cut, vec![box_a.clone(), box_b.clone()], "cut"),
            node(
                FeatureOp::Common,
                vec![box_a.clone(), box_b.clone()],
                "common",
            ),
            node(
                FeatureOp::Extrude {
                    height: 5.0,
                    twist_deg: 0.0,
                    scale: 1.0,
                },
                vec![face_c.clone()],
                "extrude",
            ),
            node(
                FeatureOp::Revolve { angle_deg: 180.0 },
                vec![face_a.clone()],
                "revolve",
            ),
            node(
                FeatureOp::Loft {
                    ruled: false,
                    profile_count: 2,
                },
                vec![face_a.clone(), face_b.clone()],
                "loft",
            ),
            node(
                FeatureOp::Shell { thickness: 2.0 },
                vec![box_a.clone()],
                "shell",
            ),
            node(
                FeatureOp::Offset { distance: 1.0 },
                vec![box_a.clone()],
                "offset",
            ),
            node(
                FeatureOp::Offset2D { distance: 1.0 },
                vec![face_a.clone()],
                "offset_2d",
            ),
            node(
                FeatureOp::Simplify {
                    min_feature_size: 0.5,
                },
                vec![box_a.clone()],
                "simplify",
            ),
            node(
                FeatureOp::Sweep,
                vec![face_a.clone(), path.clone()],
                "sweep",
            ),
            node(
                FeatureOp::PatternLinear {
                    n: 3,
                    dx: 5.0,
                    dy: 0.0,
                    dz: 0.0,
                },
                vec![box_a.clone()],
                "linear_pattern",
            ),
            node(
                FeatureOp::PatternPolar {
                    n: 3,
                    angle_deg: 360.0,
                },
                vec![box_a.clone()],
                "polar_pattern",
            ),
            node(
                FeatureOp::Slice {
                    plane: "xy".to_string(),
                    offset: 5.0,
                },
                vec![box_a.clone()],
                "slice",
            ),
            node(FeatureOp::ConvexHull, vec![box_a.clone()], "convex_hull"),
            node(
                FeatureOp::PathPattern { n: 3 },
                vec![face_a.clone(), path.clone()],
                "path_pattern",
            ),
            node(
                FeatureOp::Fillet { radius: 1.0 },
                vec![box_a.clone()],
                "fillet",
            ),
            node(
                FeatureOp::FilletSel {
                    radius: 1.0,
                    selector: "vertical".to_string(),
                },
                vec![box_a.clone()],
                "fillet_sel",
            ),
            node(
                FeatureOp::FilletVar { r1: 1.0, r2: 2.0 },
                vec![box_a.clone()],
                "fillet_var",
            ),
            node(
                FeatureOp::FilletVarSel {
                    r1: 1.0,
                    r2: 2.0,
                    selector: "vertical".to_string(),
                },
                vec![box_a.clone()],
                "fillet_var_sel",
            ),
            node(
                FeatureOp::Chamfer { dist: 1.0 },
                vec![box_a.clone()],
                "chamfer",
            ),
            node(
                FeatureOp::ChamferSel {
                    dist: 1.0,
                    selector: "vertical".to_string(),
                },
                vec![box_a.clone()],
                "chamfer_sel",
            ),
            node(
                FeatureOp::ChamferAsym { d1: 1.0, d2: 2.0 },
                vec![box_a.clone()],
                "chamfer_asym",
            ),
            node(
                FeatureOp::ChamferAsymSel {
                    d1: 1.0,
                    d2: 2.0,
                    selector: "vertical".to_string(),
                },
                vec![box_a.clone()],
                "chamfer_asym_sel",
            ),
            node(
                FeatureOp::FilletWire { radius: 1.0 },
                vec![face_a.clone()],
                "fillet_wire",
            ),
            node(
                FeatureOp::Pad { height: 5.0 },
                vec![box_a.clone(), face_a.clone(), face_c.clone()],
                "pad",
            ),
            node(
                FeatureOp::Pocket { depth: 3.0 },
                vec![box_a.clone(), face_a.clone(), face_c.clone()],
                "pocket",
            ),
            node(
                FeatureOp::DatumPlane {
                    ox: 0.0,
                    oy: 0.0,
                    oz: 0.0,
                    nx: 0.0,
                    ny: 0.0,
                    nz: 1.0,
                    xx: 1.0,
                    xy: 0.0,
                    xz: 0.0,
                },
                Vec::new(),
                "datum_plane",
            ),
            node(
                FeatureOp::ExtrudeDraft {
                    height: 5.0,
                    draft_deg: 3.0,
                },
                vec![face_a.clone()],
                "extrude_draft",
            ),
            node(
                FeatureOp::BezierPatch {
                    points: vec![
                        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, //
                        0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 2.0, 1.0, 0.0, 3.0, 1.0, 0.0, //
                        0.0, 2.0, 0.0, 1.0, 2.0, 0.0, 2.0, 2.0, 0.0, 3.0, 2.0, 0.0, //
                        0.0, 3.0, 0.0, 1.0, 3.0, 0.0, 2.0, 3.0, 0.0, 3.0, 3.0, 0.0,
                    ],
                },
                Vec::new(),
                "bezier_patch",
            ),
            node(
                FeatureOp::Sew {
                    face_count: 2,
                    tolerance: 0.01,
                },
                vec![face_a.clone(), face_b.clone()],
                "sew",
            ),
            node(
                FeatureOp::SweepSections { profile_count: 2 },
                vec![face_a.clone(), face_b.clone(), path.clone()],
                "sweep_sections",
            ),
            node(
                FeatureOp::RuledSurface,
                vec![path.clone(), guide.clone()],
                "ruled_surface",
            ),
            node(
                FeatureOp::FillSurface,
                vec![leaf(
                    FeatureOp::Arc {
                        r: 5.0,
                        start_deg: 0.0,
                        end_deg: 360.0,
                    },
                    "arc",
                )],
                "fill_surface",
            ),
        ] {
            let label = node.op.name();
            assert!(
                node.rebuild().is_ok(),
                "expected rebuild to succeed for {}",
                label
            );
        }
    }
}
