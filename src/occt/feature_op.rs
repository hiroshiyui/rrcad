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
    /// Text glyph outlines (Phase 12). Rebuilds by re-rendering, so the
    /// font must still resolve at rebuild time.
    Text {
        text: String,
        size: f64,
        font: String,
    },
    Circle {
        r: f64,
    },
    Polygon {
        points: Vec<f64>,
    },
    Profile2D {
        points: Vec<f64>,
        counts: Vec<i32>,
        kinds: Vec<i32>,
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
    /// Shell with chosen opening faces. The single parent is the body; the
    /// removed faces are re-found on rebuild by centroid (flat x,y,z
    /// triples), because a face Shape carries its parent solid's feature
    /// node and cannot rebuild into a face.
    ShellOpen {
        thickness: f64,
        face_centroids: Vec<f64>,
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
    Thicken {
        thickness: f64,
    },
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

// FeatureOp implementations live in `feature_op_impl.rs`.
