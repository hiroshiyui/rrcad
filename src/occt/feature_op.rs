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
    pub(crate) fn name(&self) -> String {
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

    pub(crate) fn parents_are_valid(&self, count: usize) -> bool {
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
