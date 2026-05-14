use super::Shape;
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone)]
pub(crate) enum NamedRef {
    FaceSelector(String),
    EdgeSelector(String),
    Datum(Arc<Shape>),
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct NamedRefSnapshot {
    pub name: String,
    pub kind: String,
    pub selector: String,
    pub shape_type: String,
    pub centroid: Option<[f64; 3]>,
    pub normal: Option<[f64; 3]>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) enum GdtStandard {
    Asme,
    Iso,
}

impl GdtStandard {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "asme" => Ok(GdtStandard::Asme),
            "iso" => Ok(GdtStandard::Iso),
            other => Err(format!("unsupported GD&T standard: {other}")),
        }
    }

    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            GdtStandard::Asme => "asme",
            GdtStandard::Iso => "iso",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct GdtDatumSpec {
    pub label: String,
    pub selector: Option<String>,
    pub anchor: Option<[f64; 3]>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct GdtFeatureControlSpec {
    pub text: String,
    pub selector: Option<String>,
    pub anchor: Option<[f64; 3]>,
    pub datums: Vec<String>,
    pub modifiers: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct GdtRenderSpec {
    pub standard: GdtStandard,
    pub datum: Option<GdtDatumSpec>,
    pub feature_control: Option<GdtFeatureControlSpec>,
}

#[derive(Clone, Debug)]
pub(crate) enum FeatureOp {
    Box {
        dx: f64,
        dy: f64,
        dz: f64,
    },
    Cylinder {
        radius: f64,
        height: f64,
    },
    Sphere {
        radius: f64,
    },
    Cone {
        r1: f64,
        r2: f64,
        height: f64,
    },
    Torus {
        r1: f64,
        r2: f64,
    },
    Wedge {
        dx: f64,
        dy: f64,
        dz: f64,
        ltx: f64,
    },
    Rect {
        w: f64,
        h: f64,
    },
    Circle {
        r: f64,
    },
    Polygon {
        points: Vec<f64>,
    },
    Ellipse {
        rx: f64,
        ry: f64,
    },
    Arc {
        r: f64,
        start_deg: f64,
        end_deg: f64,
    },
    Translate {
        dx: f64,
        dy: f64,
        dz: f64,
    },
    Rotate {
        axis_x: f64,
        axis_y: f64,
        axis_z: f64,
        angle_deg: f64,
    },
    Scale {
        factor: f64,
    },
    ScaleXyz {
        sx: f64,
        sy: f64,
        sz: f64,
    },
    Mirror {
        plane: String,
    },
    Fuse,
    Cut,
    Common,
    Extrude {
        height: f64,
        twist_deg: f64,
        scale: f64,
    },
    Revolve {
        angle_deg: f64,
    },
    Spline2D {
        points: Vec<f64>,
        tangents: Option<[f64; 4]>,
    },
    Spline3D {
        points: Vec<f64>,
        tangents: Option<[f64; 6]>,
    },
    Helix {
        radius: f64,
        pitch: f64,
        height: f64,
    },
    Loft {
        ruled: bool,
        profile_count: usize,
    },
    Shell {
        thickness: f64,
    },
    Offset {
        distance: f64,
    },
    Offset2D {
        distance: f64,
    },
    Simplify {
        min_feature_size: f64,
    },
    Sweep,
    SweepGuide,
    ImportStep {
        path: String,
    },
    ImportStl {
        path: String,
    },
    PatternLinear {
        n: i32,
        dx: f64,
        dy: f64,
        dz: f64,
    },
    PatternPolar {
        n: i32,
        angle_deg: f64,
    },
    FragmentAll {
        count: usize,
    },
    ConvexHull,
    PathPattern {
        n: i32,
    },
    Slice {
        plane: String,
        offset: f64,
    },
    Fillet {
        radius: f64,
    },
    FilletSel {
        radius: f64,
        selector: String,
    },
    FilletVar {
        r1: f64,
        r2: f64,
    },
    FilletVarSel {
        r1: f64,
        r2: f64,
        selector: String,
    },
    Chamfer {
        dist: f64,
    },
    ChamferSel {
        dist: f64,
        selector: String,
    },
    ChamferAsym {
        d1: f64,
        d2: f64,
    },
    ChamferAsymSel {
        d1: f64,
        d2: f64,
        selector: String,
    },
    Pad {
        height: f64,
    },
    Pocket {
        depth: f64,
    },
    FilletWire {
        radius: f64,
    },
    DatumPlane {
        ox: f64,
        oy: f64,
        oz: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        xx: f64,
        xy: f64,
        xz: f64,
    },
    ExtrudeDraft {
        height: f64,
        draft_deg: f64,
    },
    BezierPatch {
        points: Vec<f64>,
    },
    Sew {
        face_count: usize,
        tolerance: f64,
    },
    SweepSections {
        profile_count: usize,
    },
    RuledSurface,
    FillSurface,
    Opaque {
        label: String,
    },
}

impl FeatureOp {
    fn name(&self) -> String {
        match self {
            FeatureOp::Box { dx, dy, dz } => format!("box(dx={dx}, dy={dy}, dz={dz})"),
            FeatureOp::Cylinder { radius, height } => {
                format!("cylinder(radius={radius}, height={height})")
            }
            FeatureOp::Sphere { radius } => format!("sphere(radius={radius})"),
            FeatureOp::Cone { r1, r2, height } => {
                format!("cone(r1={r1}, r2={r2}, height={height})")
            }
            FeatureOp::Torus { r1, r2 } => format!("torus(r1={r1}, r2={r2})"),
            FeatureOp::Wedge { dx, dy, dz, ltx } => {
                format!("wedge(dx={dx}, dy={dy}, dz={dz}, ltx={ltx})")
            }
            FeatureOp::Rect { w, h } => format!("rect(w={w}, h={h})"),
            FeatureOp::Circle { r } => format!("circle(r={r})"),
            FeatureOp::Polygon { points } => format!("polygon(points={})", points.len() / 2),
            FeatureOp::Ellipse { rx, ry } => format!("ellipse(rx={rx}, ry={ry})"),
            FeatureOp::Arc {
                r,
                start_deg,
                end_deg,
            } => {
                format!("arc(r={r}, start_deg={start_deg}, end_deg={end_deg})")
            }
            FeatureOp::Translate { dx, dy, dz } => {
                format!("translate(dx={dx}, dy={dy}, dz={dz})")
            }
            FeatureOp::Rotate {
                axis_x,
                axis_y,
                axis_z,
                angle_deg,
            } => format!("rotate(axis=({axis_x}, {axis_y}, {axis_z}), angle_deg={angle_deg})"),
            FeatureOp::Scale { factor } => format!("scale(factor={factor})"),
            FeatureOp::ScaleXyz { sx, sy, sz } => {
                format!("scale_xyz(sx={sx}, sy={sy}, sz={sz})")
            }
            FeatureOp::Mirror { plane } => format!("mirror(plane={plane})"),
            FeatureOp::Fuse => "fuse()".to_string(),
            FeatureOp::Cut => "cut()".to_string(),
            FeatureOp::Common => "common()".to_string(),
            FeatureOp::Extrude {
                height,
                twist_deg,
                scale,
            } => format!("extrude(height={height}, twist_deg={twist_deg}, scale={scale})"),
            FeatureOp::Revolve { angle_deg } => format!("revolve(angle_deg={angle_deg})"),
            FeatureOp::Spline2D { points, tangents } => match tangents {
                Some([t0x, t0z, t1x, t1z]) => format!(
                    "spline_2d_tan(points={}, t0=({}, {}), t1=({}, {}))",
                    points.len() / 2,
                    t0x,
                    t0z,
                    t1x,
                    t1z
                ),
                None => format!("spline_2d(points={})", points.len() / 2),
            },
            FeatureOp::Spline3D { points, tangents } => match tangents {
                Some([t0x, t0y, t0z, t1x, t1y, t1z]) => format!(
                    "spline_3d_tan(points={}, t0=({}, {}, {}), t1=({}, {}, {}))",
                    points.len() / 3,
                    t0x,
                    t0y,
                    t0z,
                    t1x,
                    t1y,
                    t1z
                ),
                None => format!("spline_3d(points={})", points.len() / 3),
            },
            FeatureOp::Helix {
                radius,
                pitch,
                height,
            } => {
                format!("helix(radius={radius}, pitch={pitch}, height={height})")
            }
            FeatureOp::Loft {
                ruled,
                profile_count,
            } => format!("loft(profiles={profile_count}, ruled={ruled})"),
            FeatureOp::Shell { thickness } => format!("shell(thickness={thickness})"),
            FeatureOp::Offset { distance } => format!("offset(distance={distance})"),
            FeatureOp::Offset2D { distance } => format!("offset_2d(distance={distance})"),
            FeatureOp::Simplify { min_feature_size } => {
                format!("simplify(min_feature_size={min_feature_size})")
            }
            FeatureOp::Sweep => "sweep()".to_string(),
            FeatureOp::SweepGuide => "sweep_guide()".to_string(),
            FeatureOp::ImportStep { path } => format!("import_step(path={path:?})"),
            FeatureOp::ImportStl { path } => format!("import_stl(path={path:?})"),
            FeatureOp::PatternLinear { n, dx, dy, dz } => {
                format!("linear_pattern(n={n}, dx={dx}, dy={dy}, dz={dz})")
            }
            FeatureOp::PatternPolar { n, angle_deg } => {
                format!("polar_pattern(n={n}, angle_deg={angle_deg})")
            }
            FeatureOp::FragmentAll { count } => format!("fragment_all(count={count})"),
            FeatureOp::ConvexHull => "convex_hull()".to_string(),
            FeatureOp::PathPattern { n } => format!("path_pattern(n={n})"),
            FeatureOp::Slice { plane, offset } => format!("slice(plane={plane}, offset={offset})"),
            FeatureOp::Fillet { radius } => format!("fillet(radius={radius})"),
            FeatureOp::FilletSel { radius, selector } => {
                format!("fillet(radius={radius}, selector={selector})")
            }
            FeatureOp::FilletVar { r1, r2 } => format!("fillet_var(r1={r1}, r2={r2})"),
            FeatureOp::FilletVarSel { r1, r2, selector } => {
                format!("fillet_var(r1={r1}, r2={r2}, selector={selector})")
            }
            FeatureOp::Chamfer { dist } => format!("chamfer(dist={dist})"),
            FeatureOp::ChamferSel { dist, selector } => {
                format!("chamfer(dist={dist}, selector={selector})")
            }
            FeatureOp::ChamferAsym { d1, d2 } => {
                format!("chamfer_asym(d1={d1}, d2={d2})")
            }
            FeatureOp::ChamferAsymSel { d1, d2, selector } => {
                format!("chamfer_asym(d1={d1}, d2={d2}, selector={selector})")
            }
            FeatureOp::Pad { height } => format!("pad(height={height})"),
            FeatureOp::Pocket { depth } => format!("pocket(depth={depth})"),
            FeatureOp::FilletWire { radius } => format!("fillet_wire(radius={radius})"),
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
            } => format!(
                "datum_plane(origin=({ox}, {oy}, {oz}), normal=({nx}, {ny}, {nz}), x_dir=({xx}, {xy}, {xz}))"
            ),
            FeatureOp::ExtrudeDraft { height, draft_deg } => {
                format!("extrude_draft(height={height}, draft_deg={draft_deg})")
            }
            FeatureOp::BezierPatch { points } => {
                format!("bezier_patch(points={})", points.len() / 3)
            }
            FeatureOp::Sew {
                face_count,
                tolerance,
            } => format!("sew(faces={face_count}, tolerance={tolerance})"),
            FeatureOp::SweepSections { profile_count } => {
                format!("sweep_sections(profiles={profile_count})")
            }
            FeatureOp::RuledSurface => "ruled_surface()".to_string(),
            FeatureOp::FillSurface => "fill_surface()".to_string(),
            FeatureOp::Opaque { label } => label.clone(),
        }
    }

    fn parents_are_valid(&self, count: usize) -> bool {
        match self {
            FeatureOp::Box { .. }
            | FeatureOp::Cylinder { .. }
            | FeatureOp::Sphere { .. }
            | FeatureOp::Cone { .. }
            | FeatureOp::Torus { .. }
            | FeatureOp::Wedge { .. }
            | FeatureOp::Rect { .. }
            | FeatureOp::Circle { .. }
            | FeatureOp::Polygon { .. }
            | FeatureOp::Ellipse { .. }
            | FeatureOp::Arc { .. }
            | FeatureOp::Spline2D { .. }
            | FeatureOp::Spline3D { .. }
            | FeatureOp::Helix { .. }
            | FeatureOp::ImportStep { .. }
            | FeatureOp::ImportStl { .. }
            | FeatureOp::Opaque { .. }
            | FeatureOp::DatumPlane { .. }
            | FeatureOp::BezierPatch { .. } => count == 0,
            FeatureOp::RuledSurface => count == 2,
            FeatureOp::FillSurface => count == 1,
            FeatureOp::Translate { .. }
            | FeatureOp::Rotate { .. }
            | FeatureOp::Scale { .. }
            | FeatureOp::ScaleXyz { .. }
            | FeatureOp::Mirror { .. }
            | FeatureOp::Extrude { .. }
            | FeatureOp::Revolve { .. }
            | FeatureOp::Shell { .. }
            | FeatureOp::Offset { .. }
            | FeatureOp::Offset2D { .. }
            | FeatureOp::Simplify { .. }
            | FeatureOp::ConvexHull
            | FeatureOp::Slice { .. }
            | FeatureOp::Fillet { .. }
            | FeatureOp::FilletSel { .. }
            | FeatureOp::FilletVar { .. }
            | FeatureOp::FilletVarSel { .. }
            | FeatureOp::Chamfer { .. }
            | FeatureOp::ChamferSel { .. }
            | FeatureOp::ChamferAsym { .. }
            | FeatureOp::ChamferAsymSel { .. }
            | FeatureOp::FilletWire { .. }
            | FeatureOp::ExtrudeDraft { .. }
            | FeatureOp::PatternLinear { .. }
            | FeatureOp::PatternPolar { .. } => count == 1,
            FeatureOp::Fuse
            | FeatureOp::Cut
            | FeatureOp::Common
            | FeatureOp::PathPattern { .. } => count == 2,
            FeatureOp::Sweep => count == 2,
            FeatureOp::SweepGuide | FeatureOp::Pad { .. } | FeatureOp::Pocket { .. } => count == 3,
            FeatureOp::Loft { .. } => count >= 2,
            FeatureOp::FragmentAll { .. } | FeatureOp::Sew { .. } => count >= 1,
            FeatureOp::SweepSections { .. } => count >= 3,
        }
    }
}

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
                let refs = profiles.iter().collect::<Vec<_>>();
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
