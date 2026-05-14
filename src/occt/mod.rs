#[cxx::bridge(namespace = "rrcad")]
mod ffi {
    #![allow(clippy::too_many_arguments)] // OCCT bridge functions mirror C++ signatures with many scalar parameters

    unsafe extern "C++" {
        include!("bridge.h");

        type OcctShape;

        // --- Color ---
        fn shape_set_color(
            shape: &OcctShape,
            r: f64,
            g: f64,
            b: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_copy(shape: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // --- Assembly mating ---
        fn shape_mate(
            shape: &OcctShape,
            from_face: &OcctShape,
            to_face: &OcctShape,
            offset: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Primitives ---
        fn make_box(dx: f64, dy: f64, dz: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_cylinder(radius: f64, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_sphere(radius: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_cone(r1: f64, r2: f64, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_torus(r1: f64, r2: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Boolean operations ---
        fn shape_fuse(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_cut(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_common(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // --- Fillets and chamfers ---
        fn shape_fillet(shape: &OcctShape, radius: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_chamfer(shape: &OcctShape, dist: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_sel(
            shape: &OcctShape,
            radius: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_chamfer_sel(
            shape: &OcctShape,
            dist: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_var(shape: &OcctShape, r1: f64, r2: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_var_sel(
            shape: &OcctShape,
            r1: f64,
            r2: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;
        // Phase 7 Tier 1: asymmetric chamfer (.chamfer(d1, d2)).
        fn shape_chamfer_asym(shape: &OcctShape, d1: f64, d2: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_chamfer_asym_sel(
            shape: &OcctShape,
            d1: f64,
            d2: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Transforms (immutable; return new shapes) ---
        fn shape_translate(
            shape: &OcctShape,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_rotate(
            shape: &OcctShape,
            axis_x: f64,
            axis_y: f64,
            axis_z: f64,
            angle_deg: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_scale(shape: &OcctShape, factor: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_scale_xyz(
            shape: &OcctShape,
            sx: f64,
            sy: f64,
            sz: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        fn shape_mirror(shape: &OcctShape, plane: &str) -> Result<UniquePtr<OcctShape>>;

        fn make_rect(w: f64, h: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_circle_face(r: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_polygon(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        fn make_ellipse_face(rx: f64, ry: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_arc(r: f64, start_deg: f64, end_deg: f64) -> Result<UniquePtr<OcctShape>>;

        fn shape_extrude(shape: &OcctShape, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_revolve(shape: &OcctShape, angle_deg: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 4: ThruSections (loft) builder ---
        type ThruSectionsBuilder;
        fn thru_sections_new(solid: bool, ruled: bool) -> Result<UniquePtr<ThruSectionsBuilder>>;
        fn thru_sections_add(
            builder: Pin<&mut ThruSectionsBuilder>,
            profile: &OcctShape,
        ) -> Result<()>;
        fn thru_sections_build(
            builder: Pin<&mut ThruSectionsBuilder>,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: PipeShellBuilder (variable-section sweep) ---
        type PipeShellBuilder;
        fn pipe_shell_new(path: &OcctShape) -> Result<UniquePtr<PipeShellBuilder>>;
        fn pipe_shell_add(builder: Pin<&mut PipeShellBuilder>, profile: &OcctShape) -> Result<()>;
        fn pipe_shell_build(builder: Pin<&mut PipeShellBuilder>) -> Result<UniquePtr<OcctShape>>;

        // --- Bézier surface patch ---
        // pts: 48 doubles — 16 control points (4×4 row-major) each as (x, y, z).
        fn make_bezier_patch(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;

        // --- Sewing builder ---
        type SewingBuilder;
        fn sewing_new(tolerance: f64) -> Result<UniquePtr<SewingBuilder>>;
        fn sewing_add(builder: Pin<&mut SewingBuilder>, shape: &OcctShape) -> Result<()>;
        fn sewing_build(builder: Pin<&mut SewingBuilder>) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 4: 3-D operations ---
        fn shape_shell(shape: &OcctShape, thickness: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_offset(shape: &OcctShape, distance: f64) -> Result<UniquePtr<OcctShape>>;
        // Phase 7 Tier 1: 2D profile offset (Wire or Face in its own plane).
        fn shape_offset_2d(shape: &OcctShape, distance: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_simplify(shape: &OcctShape, min_feature_size: f64)
        -> Result<UniquePtr<OcctShape>>;
        fn shape_extrude_ex(
            shape: &OcctShape,
            height: f64,
            twist_deg: f64,
            scale: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: splines and sweep ---
        fn make_spline_2d(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        fn make_spline_3d(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        // Tangent-constrained variants (Phase 4 / Tier 4 quality improvement).
        fn make_spline_2d_tan(
            pts: &[f64],
            t0x: f64,
            t0z: f64,
            t1x: f64,
            t1z: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn make_spline_3d_tan(
            pts: &[f64],
            t0x: f64,
            t0y: f64,
            t0z: f64,
            t1x: f64,
            t1y: f64,
            t1z: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_sweep(profile: &OcctShape, path: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: sub-shape selectors ---
        fn shape_faces_count(shape: &OcctShape, selector: &str) -> Result<i32>;
        fn shape_faces_get(
            shape: &OcctShape,
            selector: &str,
            idx: i32,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_edges_count(shape: &OcctShape, selector: &str) -> Result<i32>;
        fn shape_edges_get(
            shape: &OcctShape,
            selector: &str,
            idx: i32,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_vertices_count(shape: &OcctShape, selector: &str) -> Result<i32>;
        fn shape_vertices_get(
            shape: &OcctShape,
            selector: &str,
            idx: i32,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Patterns ---
        fn shape_linear_pattern(
            shape: &OcctShape,
            n: i32,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_polar_pattern(
            shape: &OcctShape,
            n: i32,
            angle_deg: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Import ---
        fn import_step(path: &str) -> Result<UniquePtr<OcctShape>>;
        fn import_stl(path: &str) -> Result<UniquePtr<OcctShape>>;

        // --- Query / introspection ---
        fn shape_bounding_box(shape: &OcctShape, out: &mut [f64]) -> Result<()>;
        fn shape_volume(shape: &OcctShape) -> Result<f64>;
        fn shape_surface_area(shape: &OcctShape) -> Result<f64>;
        // Phase 7 Tier 2: validation & introspection.
        fn shape_type_str(shape: &OcctShape) -> Result<String>;
        fn shape_centroid(shape: &OcctShape, out: &mut [f64]) -> Result<()>;
        fn shape_face_normal(face: &OcctShape, out: &mut [f64]) -> Result<()>;
        fn shape_cylinder_axis(face: &OcctShape, out: &mut [f64]) -> Result<()>;
        fn shape_is_closed(shape: &OcctShape) -> Result<bool>;
        fn shape_is_manifold(shape: &OcctShape) -> Result<bool>;
        fn shape_validate_str(shape: &OcctShape) -> Result<String>;

        // --- Phase 8 Tier 1: Core Part Design ---
        fn shape_pad(
            body: &OcctShape,
            face_ref: &OcctShape,
            sketch: &OcctShape,
            height: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_pocket(
            body: &OcctShape,
            face_ref: &OcctShape,
            sketch: &OcctShape,
            depth: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_wire(profile: &OcctShape, radius: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_datum_plane(
            ox: f64,
            oy: f64,
            oz: f64,
            nx: f64,
            ny: f64,
            nz: f64,
            xx: f64,
            xy: f64,
            xz: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 8 Tier 3: Inspection & clearance ---
        fn shape_distance_to(a: &OcctShape, b: &OcctShape) -> Result<f64>;
        fn shape_inertia(shape: &OcctShape, out: &mut [f64]) -> Result<()>;
        fn shape_min_thickness(shape: &OcctShape) -> Result<f64>;

        // --- Phase 8 Tier 2: Manufacturing features ---
        fn shape_extrude_draft(
            profile: &OcctShape,
            height: f64,
            draft_deg: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn make_helix(radius: f64, pitch: f64, height: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 7 Tier 3: Surface modeling ---
        fn shape_ruled_surface(
            wire_a: &OcctShape,
            wire_b: &OcctShape,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_fill_surface(boundary_wire: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_slice(shape: &OcctShape, plane: &str, offset: f64)
        -> Result<UniquePtr<OcctShape>>;

        // --- Export ---
        fn export_step(shape: &OcctShape, path: &str) -> Result<()>;
        fn export_stl(shape: &OcctShape, path: &str) -> Result<()>;
        fn export_gltf(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
        fn export_glb(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
        fn export_obj(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;

        // Phase 8 Tier 4: 2-D drawing output.
        fn export_svg(
            shape: &OcctShape,
            path: &str,
            view: &str,
            scale: f64,
            hidden: bool,
            center_marks: bool,
            dimensions: bool,
            title_block: bool,
            callouts: bool,
            datum: &str,
            datum_anchor_valid: bool,
            datum_anchor_x: f64,
            datum_anchor_y: f64,
            datum_anchor_z: f64,
            feature_control: &str,
            feature_control_anchor_valid: bool,
            feature_control_anchor_x: f64,
            feature_control_anchor_y: f64,
            feature_control_anchor_z: f64,
            tolerance_plus: f64,
            tolerance_minus: f64,
        ) -> Result<()>;
        fn export_dxf(
            shape: &OcctShape,
            path: &str,
            view: &str,
            scale: f64,
            hidden: bool,
            center_marks: bool,
            dimensions: bool,
            title_block: bool,
            callouts: bool,
            datum: &str,
            datum_anchor_valid: bool,
            datum_anchor_x: f64,
            datum_anchor_y: f64,
            datum_anchor_z: f64,
            feature_control: &str,
            feature_control_anchor_valid: bool,
            feature_control_anchor_x: f64,
            feature_control_anchor_y: f64,
            feature_control_anchor_z: f64,
            tolerance_plus: f64,
            tolerance_minus: f64,
        ) -> Result<()>;

        // Phase 8 Tier 5: Advanced composition.

        // fragment builder — accumulate shapes then split at all intersections.
        type FragmentBuilder;
        fn fragment_new() -> Result<UniquePtr<FragmentBuilder>>;
        fn fragment_add(builder: Pin<&mut FragmentBuilder>, shape: &OcctShape) -> Result<()>;
        fn fragment_build(builder: Pin<&mut FragmentBuilder>) -> Result<UniquePtr<OcctShape>>;

        // convex hull of the shape's tessellated mesh vertices.
        fn shape_convex_hull(shape: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // n evenly-spaced (arc-length) copies of shape along path.
        fn shape_path_pattern(
            shape: &OcctShape,
            path: &OcctShape,
            n: i32,
        ) -> Result<UniquePtr<OcctShape>>;

        // guided sweep: profile swept along path, orientation locked to guide.
        fn shape_sweep_guide(
            profile: &OcctShape,
            path: &OcctShape,
            guide: &OcctShape,
        ) -> Result<UniquePtr<OcctShape>>;
    }
}

use cxx::UniquePtr;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
enum NamedRef {
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

/// Owned handle to a live OCCT shape.
pub struct Shape {
    inner: UniquePtr<ffi::OcctShape>,
    named_refs: RefCell<BTreeMap<String, NamedRef>>,
    gdt_render: RefCell<Option<GdtRenderSpec>>,
    history: RefCell<Vec<String>>,
}

/// Best-effort one-word shape kind ("solid", "face", "wire", …) used to
/// enrich error messages. Falls back to `"shape"` if the type query
/// itself fails (e.g. the inner pointer is in a degraded state).
fn summarize(shape: &Shape) -> String {
    ffi::shape_type_str(&shape.inner)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "shape".to_string())
}

/// Format a one-line actionable hint for the error message. Returns the
/// empty string when `s` is empty so callers can unconditionally append.
fn hint(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        format!("\n  hint: {s}")
    }
}

static DEBUG_EXPORT_SEQ: AtomicUsize = AtomicUsize::new(1);

fn debug_exports_enabled() -> bool {
    matches!(
        env::var("RRCAD_DEBUG_EXPORTS")
            .ok()
            .as_deref()
            .map(|v| v.to_ascii_lowercase())
            .as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

fn debug_exports_root() -> PathBuf {
    env::var_os("RRCAD_DEBUG_EXPORTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("rrcad-debug"))
}

fn debug_export_component(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "shape".to_string()
    } else {
        out
    }
}

impl Shape {
    fn fresh(inner: UniquePtr<ffi::OcctShape>) -> Self {
        Self {
            inner,
            named_refs: RefCell::new(BTreeMap::new()),
            gdt_render: RefCell::new(None),
            history: RefCell::new(Vec::new()),
        }
    }

    fn fresh_with_history(inner: UniquePtr<ffi::OcctShape>, entry: impl Into<String>) -> Self {
        let shape = Self::fresh(inner);
        shape.history.borrow_mut().push(entry.into());
        shape
    }

    fn with_inner(&self, inner: UniquePtr<ffi::OcctShape>) -> Self {
        Self {
            inner,
            named_refs: RefCell::new(self.named_refs.borrow().clone()),
            gdt_render: RefCell::new(self.gdt_render.borrow().clone()),
            history: RefCell::new(self.history.borrow().clone()),
        }
    }

    fn with_inner_and_history(
        &self,
        inner: UniquePtr<ffi::OcctShape>,
        entry: impl Into<String>,
    ) -> Self {
        let shape = self.with_inner(inner);
        shape.history.borrow_mut().push(entry.into());
        shape
    }

    fn named_ref(&self, name: &str) -> Option<NamedRef> {
        self.named_refs.borrow().get(name).cloned()
    }

    fn set_named_ref(&self, name: impl Into<String>, named: NamedRef) {
        self.named_refs.borrow_mut().insert(name.into(), named);
    }

    fn resolve_named_selector(&self, name: &str) -> Option<NamedRef> {
        self.named_ref(name)
    }

    pub(crate) fn named_ref_snapshots(&self) -> Vec<NamedRefSnapshot> {
        let entries: Vec<(String, NamedRef)> = self
            .named_refs
            .borrow()
            .iter()
            .map(|(name, named)| (name.clone(), named.clone()))
            .collect();
        entries
            .into_iter()
            .map(|(name, named)| match named {
                NamedRef::FaceSelector(selector) => {
                    let face = self
                        .faces(&selector)
                        .ok()
                        .and_then(|faces| faces.into_iter().next());
                    let centroid = face.as_ref().and_then(|f| f.centroid().ok());
                    let normal = face.as_ref().and_then(|f| f.face_normal().ok());
                    let shape_type = face
                        .as_ref()
                        .and_then(|f| f.shape_type_name().ok())
                        .unwrap_or_else(|| "face".to_string());
                    NamedRefSnapshot {
                        name: name.clone(),
                        kind: "face".to_string(),
                        selector: format!(":{selector}"),
                        shape_type,
                        centroid,
                        normal,
                    }
                }
                NamedRef::EdgeSelector(selector) => {
                    let edge = self
                        .edges(&selector)
                        .ok()
                        .and_then(|edges| edges.into_iter().next());
                    let centroid = edge.as_ref().and_then(|e| e.centroid().ok());
                    let shape_type = edge
                        .as_ref()
                        .and_then(|e| e.shape_type_name().ok())
                        .unwrap_or_else(|| "edge".to_string());
                    NamedRefSnapshot {
                        name: name.clone(),
                        kind: "edge".to_string(),
                        selector: format!(":{selector}"),
                        shape_type,
                        centroid,
                        normal: None,
                    }
                }
                NamedRef::Datum(shape) => {
                    let centroid = shape.centroid().ok();
                    let shape_type = shape
                        .shape_type_name()
                        .unwrap_or_else(|_| "shape".to_string());
                    NamedRefSnapshot {
                        name: name.clone(),
                        kind: "datum".to_string(),
                        selector: format!("ref(:{name})"),
                        shape_type,
                        centroid,
                        normal: None,
                    }
                }
            })
            .collect()
    }

    fn gdt_render(&self) -> Option<GdtRenderSpec> {
        self.gdt_render.borrow().clone()
    }

    fn set_gdt_render(&self, render: Option<GdtRenderSpec>) {
        *self.gdt_render.borrow_mut() = render;
    }

    pub fn history(&self) -> Vec<String> {
        self.history.borrow().clone()
    }

    fn history_tail(&self, max_entries: usize) -> Option<String> {
        let history = self.history.borrow();
        if history.is_empty() {
            return None;
        }
        let start = history.len().saturating_sub(max_entries);
        Some(history[start..].join(" -> "))
    }

    fn history_note(&self, shapes: &[(&str, &Shape)]) -> Option<String> {
        let mut parts = Vec::new();
        for (label, shape) in shapes {
            if let Some(tail) = shape.history_tail(4) {
                parts.push(format!("{label}: {tail}"));
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(format!("\n  history: {}", parts.join("; ")))
        }
    }

    fn debug_export_note(&self, op: &str, shapes: &[(&str, &Shape)]) -> Option<String> {
        if !debug_exports_enabled() {
            return None;
        }

        let seq = DEBUG_EXPORT_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = debug_exports_root().join(format!(
            "{}-{}-{}",
            debug_export_component(op),
            std::process::id(),
            seq
        ));

        if let Err(e) = std::fs::create_dir_all(&dir) {
            return Some(format!(
                "\n  debug export: could not create {}: {e}",
                dir.display()
            ));
        }

        for (label, shape) in shapes {
            let file = dir.join(format!("{}.step", debug_export_component(label)));
            if let Err(e) = ffi::export_step(&shape.inner, file.to_string_lossy().as_ref()) {
                return Some(format!(
                    "\n  debug export: {} (failed writing {}: {e})",
                    dir.display(),
                    file.display()
                ));
            }
        }

        Some(format!("\n  debug export: {}", dir.display()))
    }

    fn fail_with_debug(&self, base: String, op: &str, shapes: &[(&str, &Shape)]) -> String {
        let mut msg = base;
        if let Some(note) = self.history_note(shapes) {
            msg.push_str(&note);
        }
        if let Some(note) = self.debug_export_note(op, shapes) {
            msg.push_str(&note);
        }
        msg
    }

    fn format_gdt_feature_control(standard: &GdtStandard, fc: &GdtFeatureControlSpec) -> String {
        let mut parts = Vec::new();
        match standard {
            GdtStandard::Asme => {
                parts.push(fc.text.clone());
                if !fc.modifiers.is_empty() {
                    parts.push(fc.modifiers.join(" "));
                }
                parts.extend(fc.datums.iter().cloned());
            }
            GdtStandard::Iso => {
                parts.extend(fc.datums.iter().cloned());
                parts.push(fc.text.clone());
                if !fc.modifiers.is_empty() {
                    parts.push(fc.modifiers.join(" "));
                }
            }
        }
        parts.retain(|s| !s.is_empty());
        parts.join(" | ")
    }

    #[allow(clippy::too_many_arguments)]
    fn gdt_export_inputs(
        &self,
        datum: &str,
        datum_anchor_valid: bool,
        datum_anchor_x: f64,
        datum_anchor_y: f64,
        datum_anchor_z: f64,
        feature_control: &str,
        feature_control_anchor_valid: bool,
        feature_control_anchor_x: f64,
        feature_control_anchor_y: f64,
        feature_control_anchor_z: f64,
    ) -> (String, bool, [f64; 3], String, bool, [f64; 3]) {
        if let Some(render) = self.gdt_render() {
            let datum_text = render
                .datum
                .as_ref()
                .map(|datum| datum.label.clone())
                .unwrap_or_default();
            let datum_anchor = render
                .datum
                .as_ref()
                .and_then(|datum| datum.anchor)
                .unwrap_or([0.0, 0.0, 0.0]);
            let fc_text = render
                .feature_control
                .as_ref()
                .map(|fc| Self::format_gdt_feature_control(&render.standard, fc))
                .unwrap_or_default();
            let fc_anchor = render
                .feature_control
                .as_ref()
                .and_then(|fc| fc.anchor)
                .unwrap_or([0.0, 0.0, 0.0]);
            return (
                datum_text,
                render.datum.as_ref().and_then(|d| d.anchor).is_some(),
                datum_anchor,
                fc_text,
                render
                    .feature_control
                    .as_ref()
                    .and_then(|fc| fc.anchor)
                    .is_some(),
                fc_anchor,
            );
        }

        (
            datum.to_string(),
            datum_anchor_valid,
            [datum_anchor_x, datum_anchor_y, datum_anchor_z],
            feature_control.to_string(),
            feature_control_anchor_valid,
            [
                feature_control_anchor_x,
                feature_control_anchor_y,
                feature_control_anchor_z,
            ],
        )
    }
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::shape_copy(&self.inner).expect("shape_copy failed while cloning Shape"),
            named_refs: RefCell::new(self.named_refs.borrow().clone()),
            gdt_render: RefCell::new(self.gdt_render.borrow().clone()),
            history: RefCell::new(self.history.borrow().clone()),
        }
    }
}

impl Shape {
    // --- Constructors ---

    pub fn make_box(dx: f64, dy: f64, dz: f64) -> Result<Self, String> {
        ffi::make_box(dx, dy, dz)
            .map(|p| Shape::fresh_with_history(p, format!("box(dx={dx}, dy={dy}, dz={dz})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_cylinder(radius: f64, height: f64) -> Result<Self, String> {
        ffi::make_cylinder(radius, height)
            .map(|p| {
                Shape::fresh_with_history(p, format!("cylinder(radius={radius}, height={height})"))
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_sphere(radius: f64) -> Result<Self, String> {
        ffi::make_sphere(radius)
            .map(|p| Shape::fresh_with_history(p, format!("sphere(radius={radius})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_cone(r1: f64, r2: f64, height: f64) -> Result<Self, String> {
        ffi::make_cone(r1, r2, height)
            .map(|p| {
                Shape::fresh_with_history(p, format!("cone(r1={r1}, r2={r2}, height={height})"))
            })
            .map_err(|e| e.to_string())
    }

    pub fn make_torus(r1: f64, r2: f64) -> Result<Self, String> {
        ffi::make_torus(r1, r2)
            .map(|p| Shape::fresh_with_history(p, format!("torus(r1={r1}, r2={r2})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64) -> Result<Self, String> {
        ffi::make_wedge(dx, dy, dz, ltx)
            .map(|p| {
                Shape::fresh_with_history(p, format!("wedge(dx={dx}, dy={dy}, dz={dz}, ltx={ltx})"))
            })
            .map_err(|e| e.to_string())
    }

    // --- Boolean operations ---

    pub fn fuse(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_fuse(&self.inner, &other.inner)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("fuse(lhs={}, rhs={})", summarize(self), summarize(other)),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fuse({}, {}) failed: {e}",
                        summarize(self),
                        summarize(other)
                    ),
                    "fuse",
                    &[("lhs", self), ("rhs", other)],
                )
            })
    }

    pub fn cut(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_cut(&self.inner, &other.inner)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("cut(lhs={}, rhs={})", summarize(self), summarize(other)),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!("cut({}, {}) failed: {e}", summarize(self), summarize(other)),
                    "cut",
                    &[("lhs", self), ("rhs", other)],
                )
            })
    }

    pub fn common(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_common(&self.inner, &other.inner)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("common(lhs={}, rhs={})", summarize(self), summarize(other)),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "common({}, {}) failed: {e}",
                        summarize(self),
                        summarize(other)
                    ),
                    "common",
                    &[("lhs", self), ("rhs", other)],
                )
            })
    }

    // --- Fillets and chamfers ---

    pub fn fillet(&self, radius: f64) -> Result<Shape, String> {
        ffi::shape_fillet(&self.inner, radius)
            .map(|p| self.with_inner_and_history(p, format!("fillet(radius={radius})")))
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet(r={radius}) on {} failed: {e}{}",
                        summarize(self),
                        hint("radius likely exceeds the smallest adjacent face/edge; try a smaller value or use fillet_sel with an edge selector")
                    ),
                    "fillet",
                    &[("input", self)],
                )
            })
    }

    pub fn chamfer(&self, dist: f64) -> Result<Shape, String> {
        ffi::shape_chamfer(&self.inner, dist)
            .map(|p| self.with_inner_and_history(p, format!("chamfer(dist={dist})")))
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer(d={dist}) on {} failed: {e}{}",
                        summarize(self),
                        hint("distance likely exceeds an adjacent face dimension; try a smaller value or use chamfer_sel with an edge selector")
                    ),
                    "chamfer",
                    &[("input", self)],
                )
            })
    }

    /// Fillet only edges matching `selector` (`:all` / `:vertical` / `:horizontal`).
    pub fn fillet_sel(&self, radius: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_fillet_sel(&self.inner, radius, selector)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("fillet(radius={radius}, selector={selector})"),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet(r={radius}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "fillet_sel",
                    &[("input", self)],
                )
            })
    }

    /// Chamfer only edges matching `selector` (`:all` / `:vertical` / `:horizontal`).
    pub fn chamfer_sel(&self, dist: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_chamfer_sel(&self.inner, dist, selector)
            .map(|p| {
                self.with_inner_and_history(p, format!("chamfer(dist={dist}, selector={selector})"))
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer(d={dist}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "chamfer_sel",
                    &[("input", self)],
                )
            })
    }

    /// Variable-radius fillet on all edges: radius transitions from `r1` at one
    /// end-vertex of each edge to `r2` at the other.
    pub fn fillet_var(&self, r1: f64, r2: f64) -> Result<Shape, String> {
        ffi::shape_fillet_var(&self.inner, r1, r2)
            .map(|p| self.with_inner_and_history(p, format!("fillet_var(r1={r1}, r2={r2})")))
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet_var(r1={r1}, r2={r2}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "fillet_var",
                    &[("input", self)],
                )
            })
    }

    /// Variable-radius fillet on edges matching `selector`.
    pub fn fillet_var_sel(&self, r1: f64, r2: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_fillet_var_sel(&self.inner, r1, r2, selector)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("fillet_var(r1={r1}, r2={r2}, selector={selector})"),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "fillet_var(r1={r1}, r2={r2}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "fillet_var_sel",
                    &[("input", self)],
                )
            })
    }

    /// Asymmetric chamfer on all edges: `d1` and `d2` are the two bevel distances.
    pub fn chamfer_asym(&self, d1: f64, d2: f64) -> Result<Shape, String> {
        ffi::shape_chamfer_asym(&self.inner, d1, d2)
            .map(|p| self.with_inner_and_history(p, format!("chamfer_asym(d1={d1}, d2={d2})")))
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer_asym(d1={d1}, d2={d2}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "chamfer_asym",
                    &[("input", self)],
                )
            })
    }

    /// Asymmetric chamfer on edges matching `selector`.
    pub fn chamfer_asym_sel(&self, d1: f64, d2: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_chamfer_asym_sel(&self.inner, d1, d2, selector)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("chamfer_asym(d1={d1}, d2={d2}, selector={selector})"),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "chamfer_asym(d1={d1}, d2={d2}, edges: {selector:?}) on {} failed: {e}",
                        summarize(self)
                    ),
                    "chamfer_asym_sel",
                    &[("input", self)],
                )
            })
    }

    // --- Color ---

    /// Return a copy of `self` rigidly transformed so that `from_face` (a planar
    /// face of this shape) lies flush against `to_face` (a fixed reference face).
    ///
    /// The transform aligns face centroids and makes the outward normals antiparallel.
    /// `offset > 0` leaves a gap; `offset < 0` creates interference.
    pub fn mate(&self, from_face: &Shape, to_face: &Shape, offset: f64) -> Result<Shape, String> {
        ffi::shape_mate(&self.inner, &from_face.inner, &to_face.inner, offset)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!(
                        "mate(from_face={}, to_face={}, offset={offset})",
                        summarize(from_face),
                        summarize(to_face)
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Return a copy of this shape with an sRGB surface color attached.
    /// `r`, `g`, `b` are each in [0.0, 1.0].  The color is written into
    /// the XDE document during GLB / glTF / OBJ export.
    pub fn set_color(&self, r: f64, g: f64, b: f64) -> Result<Shape, String> {
        ffi::shape_set_color(&self.inner, r, g, b)
            .map(|p| self.with_inner_and_history(p, format!("set_color(r={r}, g={g}, b={b})")))
            .map_err(|e| e.to_string())
    }

    /// Register a persistent face name that resolves to a selector on this shape.
    pub fn name_face(&self, name: &str, selector: &str) -> Result<(), String> {
        self.faces(selector)?;
        self.set_named_ref(name, NamedRef::FaceSelector(selector.to_string()));
        Ok(())
    }

    /// Register a persistent edge name that resolves to a selector on this shape.
    pub fn name_edge(&self, name: &str, selector: &str) -> Result<(), String> {
        self.edges(selector)?;
        self.set_named_ref(name, NamedRef::EdgeSelector(selector.to_string()));
        Ok(())
    }

    /// Register a persistent datum/reference shape.
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn datum(&self, name: &str, shape: &Shape) -> Result<(), String> {
        self.set_named_ref(name, NamedRef::Datum(Arc::new(shape.clone())));
        Ok(())
    }

    /// Resolve a named face, edge, or datum reference.
    pub fn ref_named(&self, name: &str) -> Result<Shape, String> {
        match self.named_ref(name) {
            Some(NamedRef::FaceSelector(selector)) => self
                .faces(&selector)?
                .into_iter()
                .next()
                .ok_or_else(|| format!("unknown named reference: {name}")),
            Some(NamedRef::EdgeSelector(selector)) => self
                .edges(&selector)?
                .into_iter()
                .next()
                .ok_or_else(|| format!("unknown named reference: {name}")),
            Some(NamedRef::Datum(shape)) => Ok(shape.as_ref().clone()),
            None => Err(format!("unknown named reference: {name}")),
        }
    }

    /// Store a structured GD&T rendering spec on the shape.
    pub(crate) fn gdt_apply(&self, spec: GdtRenderSpec) {
        self.set_gdt_render(Some(spec));
    }

    /// Clear any stored GD&T rendering spec.
    #[allow(dead_code)]
    pub(crate) fn clear_gdt(&self) {
        self.set_gdt_render(None);
    }

    // --- Transforms ---

    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Result<Shape, String> {
        ffi::shape_translate(&self.inner, dx, dy, dz)
            .map(|p| {
                self.with_inner_and_history(p, format!("translate(dx={dx}, dy={dy}, dz={dz})"))
            })
            .map_err(|e| e.to_string())
    }

    pub fn rotate(&self, ax: f64, ay: f64, az: f64, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_rotate(&self.inner, ax, ay, az, angle_deg)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("rotate(axis=({ax}, {ay}, {az}), angle_deg={angle_deg})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn scale(&self, factor: f64) -> Result<Shape, String> {
        ffi::shape_scale(&self.inner, factor)
            .map(|p| self.with_inner_and_history(p, format!("scale(factor={factor})")))
            .map_err(|e| e.to_string())
    }

    /// Non-uniform scale with independent factors for each axis.
    pub fn scale_xyz(&self, sx: f64, sy: f64, sz: f64) -> Result<Shape, String> {
        ffi::shape_scale_xyz(&self.inner, sx, sy, sz)
            .map(|p| {
                self.with_inner_and_history(p, format!("scale_xyz(sx={sx}, sy={sy}, sz={sz})"))
            })
            .map_err(|e| e.to_string())
    }

    pub fn mirror(&self, plane: &str) -> Result<Shape, String> {
        ffi::shape_mirror(&self.inner, plane)
            .map(|p| self.with_inner_and_history(p, format!("mirror(plane={plane})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_rect(w: f64, h: f64) -> Result<Self, String> {
        ffi::make_rect(w, h)
            .map(|p| Shape::fresh_with_history(p, format!("rect(w={w}, h={h})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_circle_face(r: f64) -> Result<Self, String> {
        ffi::make_circle_face(r)
            .map(|p| Shape::fresh_with_history(p, format!("circle(r={r})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_polygon(pts: &[f64]) -> Result<Self, String> {
        ffi::make_polygon(pts)
            .map(|p| Shape::fresh_with_history(p, format!("polygon(points={})", pts.len() / 2)))
            .map_err(|e| e.to_string())
    }

    pub fn make_ellipse_face(rx: f64, ry: f64) -> Result<Self, String> {
        ffi::make_ellipse_face(rx, ry)
            .map(|p| Shape::fresh_with_history(p, format!("ellipse(rx={rx}, ry={ry})")))
            .map_err(|e| e.to_string())
    }

    pub fn make_arc(r: f64, start_deg: f64, end_deg: f64) -> Result<Self, String> {
        ffi::make_arc(r, start_deg, end_deg)
            .map(|p| {
                Shape::fresh_with_history(
                    p,
                    format!("arc(r={r}, start_deg={start_deg}, end_deg={end_deg})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn extrude(&self, height: f64) -> Result<Shape, String> {
        ffi::shape_extrude(&self.inner, height)
            .map(|p| self.with_inner_and_history(p, format!("extrude(height={height})")))
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "extrude(h={height}) on {} failed: {e}{}",
                        summarize(self),
                        hint(
                            "extrude requires a 2-D profile (Face or Wire); a Solid cannot be extruded"
                        )
                    ),
                    "extrude",
                    &[("input", self)],
                )
            })
    }

    pub fn revolve(&self, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_revolve(&self.inner, angle_deg)
            .map(|p| self.with_inner_and_history(p, format!("revolve(angle_deg={angle_deg})")))
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "revolve(angle={angle_deg}°) on {} failed: {e}",
                        summarize(self)
                    ),
                    "revolve",
                    &[("input", self)],
                )
            })
    }

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
                Shape::fresh_with_history(
                    p,
                    format!("loft(profiles=[{profile_summary}], ruled={ruled})"),
                )
            })
            .map_err(|e| format!("{} failed: {e}", ctx()))
    }

    // --- Phase 4: 3-D operations ---

    /// Hollow out the solid, removing the topmost face and offsetting inward
    /// by `thickness`.  Wraps BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin.
    pub fn shell(&self, thickness: f64) -> Result<Shape, String> {
        ffi::shape_shell(&self.inner, thickness)
            .map(|p| self.with_inner_and_history(p, format!("shell(thickness={thickness})")))
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
            .map(|p| self.with_inner_and_history(p, format!("offset(distance={distance})")))
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
    pub fn offset_2d(&self, distance: f64) -> Result<Shape, String> {
        ffi::shape_offset_2d(&self.inner, distance)
            .map(|p| self.with_inner_and_history(p, format!("offset_2d(distance={distance})")))
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
                self.with_inner_and_history(
                    p,
                    format!("simplify(min_feature_size={min_feature_size})"),
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
                self.with_inner_and_history(
                    p,
                    format!("extrude_ex(height={height}, twist_deg={twist_deg}, scale={scale})"),
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

    // --- Phase 3: splines and sweep ---

    pub fn make_spline_2d(pts: &[f64]) -> Result<Self, String> {
        ffi::make_spline_2d(pts)
            .map(|p| Shape::fresh_with_history(p, format!("spline_2d(points={})", pts.len() / 2)))
            .map_err(|e| e.to_string())
    }

    pub fn make_spline_3d(pts: &[f64]) -> Result<Self, String> {
        ffi::make_spline_3d(pts)
            .map(|p| Shape::fresh_with_history(p, format!("spline_3d(points={})", pts.len() / 3)))
            .map_err(|e| e.to_string())
    }

    /// Like `make_spline_2d` but with explicit start/end tangent vectors in
    /// the XZ plane — suppresses natural-boundary oscillation on short splines.
    pub fn make_spline_2d_tan(
        pts: &[f64],
        t0x: f64,
        t0z: f64,
        t1x: f64,
        t1z: f64,
    ) -> Result<Self, String> {
        ffi::make_spline_2d_tan(pts, t0x, t0z, t1x, t1z)
            .map(|p| {
                Shape::fresh_with_history(
                    p,
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

    /// Like `make_spline_3d` but with explicit start/end tangent vectors —
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
                Shape::fresh_with_history(
                    p,
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
                self.with_inner_and_history(p, format!("sweep(profile={}, path={})", summarize(self), summarize(path)))
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
                Shape::fresh_with_history(
                    p,
                    format!(
                        "sweep_sections(profiles=[{profile_summary}], path={})",
                        summarize(path)
                    ),
                )
            })
            .map_err(|e| {
                let mut note = String::new();
                if debug_exports_enabled() {
                    let seq = DEBUG_EXPORT_SEQ.fetch_add(1, Ordering::Relaxed);
                    let dir = debug_exports_root().join(format!(
                        "sweep_sections-{}-{}",
                        std::process::id(),
                        seq
                    ));
                    if let Err(dir_err) = std::fs::create_dir_all(&dir) {
                        note = format!(
                            "\n  debug export: could not create {}: {dir_err}",
                            dir.display()
                        );
                    } else {
                        for (i, p) in profiles.iter().enumerate() {
                            let file = dir.join(format!(
                                "profile-{}.step",
                                debug_export_component(&i.to_string())
                            ));
                            let _ = ffi::export_step(&p.inner, file.to_string_lossy().as_ref());
                        }
                        let file = dir.join("path.step");
                        if let Err(path_err) =
                            ffi::export_step(&path.inner, file.to_string_lossy().as_ref())
                        {
                            note = format!(
                                "\n  debug export: {} (failed writing {}: {path_err})",
                                dir.display(),
                                file.display()
                            );
                        } else {
                            note = format!("\n  debug export: {}", dir.display());
                        }
                    }
                }
                format!("{} failed: {e}{note}", ctx())
            })
    }

    // --- Bézier surface patch ---

    /// Build a single bicubic Bézier face from 16 control points.
    /// `pts` must be a flat slice of 48 doubles: 16 points × (x, y, z) in row-major order.
    pub fn make_bezier_patch(pts: &[f64]) -> Result<Self, String> {
        ffi::make_bezier_patch(pts)
            .map(|p| {
                Shape::fresh_with_history(p, format!("bezier_patch(points={})", pts.len() / 3))
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
                Shape::fresh_with_history(
                    p,
                    format!("sew(faces=[{face_summary}], tolerance={tolerance})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    // --- Phase 3: sub-shape selectors ---

    pub fn faces(&self, selector: &str) -> Result<Vec<Shape>, String> {
        if let Some(named) = self.resolve_named_selector(selector) {
            match named {
                NamedRef::FaceSelector(alias) => return self.faces(&alias),
                NamedRef::EdgeSelector(_) => {
                    return Err(format!("faces: named reference ':{selector}' is an edge"));
                }
                NamedRef::Datum(shape) => {
                    if shape.shape_type_name()? == "face" {
                        return Ok(vec![shape.as_ref().clone()]);
                    }
                    return Err(format!(
                        "faces: named reference ':{selector}' is not a face"
                    ));
                }
            }
        }
        let n = ffi::shape_faces_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_faces_get(&self.inner, selector, i)
                    .map(|p| {
                        self.with_inner_and_history(
                            p,
                            format!("faces(selector={selector}, idx={i})"),
                        )
                    })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    pub fn edges(&self, selector: &str) -> Result<Vec<Shape>, String> {
        if let Some(named) = self.resolve_named_selector(selector) {
            match named {
                NamedRef::EdgeSelector(alias) => return self.edges(&alias),
                NamedRef::FaceSelector(_) => {
                    return Err(format!("edges: named reference ':{selector}' is a face"));
                }
                NamedRef::Datum(shape) => {
                    if shape.shape_type_name()? == "edge" {
                        return Ok(vec![shape.as_ref().clone()]);
                    }
                    return Err(format!(
                        "edges: named reference ':{selector}' is not an edge"
                    ));
                }
            }
        }
        let n = ffi::shape_edges_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_edges_get(&self.inner, selector, i)
                    .map(|p| {
                        self.with_inner_and_history(
                            p,
                            format!("edges(selector={selector}, idx={i})"),
                        )
                    })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    /// Returns all unique vertices matching the selector (currently only `"all"`).
    pub fn vertices(&self, selector: &str) -> Result<Vec<Shape>, String> {
        let n = ffi::shape_vertices_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_vertices_get(&self.inner, selector, i)
                    .map(|p| {
                        self.with_inner_and_history(
                            p,
                            format!("vertices(selector={selector}, idx={i})"),
                        )
                    })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    // --- Query / introspection ---

    /// Returns `[xmin, ymin, zmin, xmax, ymax, zmax]`.
    pub fn bounding_box(&self) -> Result<[f64; 6], String> {
        let mut out = [0f64; 6];
        ffi::shape_bounding_box(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub fn volume(&self) -> Result<f64, String> {
        ffi::shape_volume(&self.inner).map_err(|e| e.to_string())
    }

    pub fn surface_area(&self) -> Result<f64, String> {
        ffi::shape_surface_area(&self.inner).map_err(|e| e.to_string())
    }

    /// Shape type as a lowercase string: `"solid"`, `"shell"`, `"face"`,
    /// `"wire"`, `"edge"`, `"vertex"`, `"compound"`, or `"other"`.
    pub fn shape_type_name(&self) -> Result<String, String> {
        ffi::shape_type_str(&self.inner).map_err(|e| e.to_string())
    }

    /// Centroid of the shape as `[x, y, z]`.
    pub fn centroid(&self) -> Result<[f64; 3], String> {
        let mut out = [0f64; 3];
        ffi::shape_centroid(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Outward unit normal of a face as `[nx, ny, nz]`.  Sampled at the
    /// middle of the face's parameter space; flipped when the face's
    /// orientation is REVERSED so it points out of the parent solid.
    pub fn face_normal(&self) -> Result<[f64; 3], String> {
        let mut out = [0f64; 3];
        ffi::shape_face_normal(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Cylindrical face axis as `[ox, oy, oz, ax, ay, az, radius]`.
    /// Errors if the shape is not a face or its surface is not a cylinder.
    pub fn cylinder_axis(&self) -> Result<[f64; 7], String> {
        let mut out = [0f64; 7];
        ffi::shape_cylinder_axis(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// `true` if every edge is shared by at least two faces (no free/boundary edges).
    pub fn is_closed(&self) -> Result<bool, String> {
        ffi::shape_is_closed(&self.inner).map_err(|e| e.to_string())
    }

    /// `true` if every edge is shared by exactly two faces (no T-junctions).
    pub fn is_manifold(&self) -> Result<bool, String> {
        ffi::shape_is_manifold(&self.inner).map_err(|e| e.to_string())
    }

    /// Run `BRepCheck_Analyzer`.  Returns `"ok"` if valid, or a
    /// newline-separated list of error names if not.
    pub fn validate(&self) -> Result<String, String> {
        ffi::shape_validate_str(&self.inner).map_err(|e| e.to_string())
    }

    // --- Phase 8 Tier 1: Core Part Design ---

    /// Extrude a `sketch` (Face/Wire in XY plane at Z=0) along `face_ref`'s outward normal
    /// by `height`, then fuse the resulting prism with `self` (the body).
    pub fn pad(&self, face_ref: &Shape, sketch: &Shape, height: f64) -> Result<Shape, String> {
        ffi::shape_pad(&self.inner, &face_ref.inner, &sketch.inner, height)
            .map(|s| {
                self.with_inner_and_history(
                    s,
                    format!(
                        "pad(face={}, sketch={}, height={height})",
                        summarize(face_ref),
                        summarize(sketch)
                    ),
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

    /// Extrude a `sketch` along `-normal` by `depth` and subtract the prism from `self`.
    pub fn pocket(&self, face_ref: &Shape, sketch: &Shape, depth: f64) -> Result<Shape, String> {
        ffi::shape_pocket(&self.inner, &face_ref.inner, &sketch.inner, depth)
            .map(|s| {
                self.with_inner_and_history(
                    s,
                    format!(
                        "pocket(face={}, sketch={}, depth={depth})",
                        summarize(face_ref),
                        summarize(sketch)
                    ),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "pocket(depth={depth}, face={}, sketch={}) on {} failed: {e}{}",
                        summarize(face_ref),
                        summarize(sketch),
                        summarize(self),
                        hint("face must be a planar face of the body; sketch must lie in (or be transformed into) that face's plane and fit within it")
                    ),
                    "pocket",
                    &[("body", self), ("face", face_ref), ("sketch", sketch)],
                )
            })
    }

    /// Fillet all corner vertices of a 2D Wire or Face profile with the given `radius`.
    /// Uses `BRepFilletAPI_MakeFillet2d`; non-corner vertices are silently skipped.
    pub fn fillet_wire(&self, radius: f64) -> Result<Shape, String> {
        ffi::shape_fillet_wire(&self.inner, radius)
            .map(|s| self.with_inner_and_history(s, format!("fillet_wire(radius={radius})")))
            .map_err(|e| e.to_string())
    }

    /// Construct a finite reference plane (Face) from origin, outward normal, and X direction.
    /// Returns a Face ±50 units wide, suitable for cross-sections and sketch placement.
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
                Shape::fresh_with_history(
                    p,
                    format!(
                        "datum_plane(origin=({ox}, {oy}, {oz}), normal=({nx}, {ny}, {nz}), x_dir=({xx}, {xy}, {xz}))"
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    // --- Phase 8 Tier 2: Manufacturing features ---

    /// Extrude a 2D profile and taper the lateral walls by `draft_deg` degrees.
    /// The neutral plane is Z=0 (the base), so base edges stay fixed.
    /// Positive `draft_deg` narrows the cross-section toward the top (standard mould taper).
    /// Falls through to a straight extrude when `draft_deg == 0`.
    pub fn extrude_draft(&self, height: f64, draft_deg: f64) -> Result<Shape, String> {
        ffi::shape_extrude_draft(&self.inner, height, draft_deg)
            .map(|s| {
                self.with_inner_and_history(
                    s,
                    format!("extrude_draft(height={height}, draft_deg={draft_deg})"),
                )
            })
            .map_err(|e| {
                self.fail_with_debug(
                    format!(
                        "extrude(h={height}, draft={draft_deg}°) on {} failed: {e}",
                        summarize(self)
                    ),
                    "extrude_draft",
                    &[("input", self)],
                )
            })
    }

    /// Helical Wire path — 32 sample points per turn via `GeomAPI_Interpolate`.
    /// `radius`: distance from Z axis; `pitch`: axial rise per revolution;
    /// `height`: total Z extent (= pitch × number of turns).
    pub fn make_helix(radius: f64, pitch: f64, height: f64) -> Result<Shape, String> {
        ffi::make_helix(radius, pitch, height)
            .map(|p| {
                Shape::fresh_with_history(
                    p,
                    format!("helix(radius={radius}, pitch={pitch}, height={height})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    // --- Phase 8 Tier 3: Inspection & clearance ---

    /// Minimum distance between `self` and `other`.  Returns `0.0` when the shapes
    /// intersect or touch.  Uses `BRepExtrema_DistShapeShape`.
    pub fn distance_to(&self, other: &Shape) -> Result<f64, String> {
        ffi::shape_distance_to(&self.inner, &other.inner).map_err(|e| e.to_string())
    }

    /// Inertia tensor about the centre of mass.
    /// Returns `[Ixx, Iyy, Izz, Ixy, Ixz, Iyz]` in the world frame via
    /// `BRepGProp::VolumeProperties` → `GProp_GProps::MatrixOfInertia`.
    pub fn inertia(&self) -> Result<[f64; 6], String> {
        let mut buf = [0f64; 6];
        ffi::shape_inertia(&self.inner, &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    /// Minimum wall thickness of a solid or shell.
    /// Offsets the outer shell inward by a small step and measures the resulting gap
    /// via `BRepExtrema_DistShapeShape`.
    pub fn min_thickness(&self) -> Result<f64, String> {
        ffi::shape_min_thickness(&self.inner).map_err(|e| e.to_string())
    }

    // --- Phase 7 Tier 3: Surface modeling ---

    /// Build a ruled surface (shell) between two wires via `BRepFill::Shell`.
    /// Both `wire_a` and `wire_b` must be Wire shapes.
    pub fn ruled_surface(wire_a: &Shape, wire_b: &Shape) -> Result<Shape, String> {
        ffi::shape_ruled_surface(&wire_a.inner, &wire_b.inner)
            .map(|p| {
                Shape::fresh_with_history(
                    p,
                    format!(
                        "ruled_surface(wire_a={}, wire_b={})",
                        summarize(wire_a),
                        summarize(wire_b)
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Fill the interior of a closed boundary wire with a smooth NURBS surface
    /// using `BRepFill_Filling`.  `boundary_wire` must be a Wire.
    pub fn fill_surface(boundary_wire: &Shape) -> Result<Shape, String> {
        ffi::shape_fill_surface(&boundary_wire.inner)
            .map(|p| {
                Shape::fresh_with_history(
                    p,
                    format!("fill_surface(boundary_wire={})", summarize(boundary_wire)),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Cross-section of `self` by an axis-aligned plane at `offset`.
    /// `plane` is `"xy"`, `"xz"`, or `"yz"`.
    /// Returns a compound of the section edges/wires via `BRepAlgoAPI_Section`.
    pub fn slice(&self, plane: &str, offset: f64) -> Result<Shape, String> {
        ffi::shape_slice(&self.inner, plane, offset)
            .map(|s| {
                self.with_inner_and_history(s, format!("slice(plane={plane}, offset={offset})"))
            })
            .map_err(|e| e.to_string())
    }

    // --- Patterns ---

    /// `n` translated copies of the shape at positions `i * [dx, dy, dz]` (i = 0..n-1).
    /// Returns a `TopoDS_Compound` — fuse explicitly if a merged solid is needed.
    pub fn linear_pattern(&self, n: i32, dx: f64, dy: f64, dz: f64) -> Result<Shape, String> {
        ffi::shape_linear_pattern(&self.inner, n, dx, dy, dz)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("linear_pattern(n={n}, dx={dx}, dy={dy}, dz={dz})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// `n` copies rotated around Z by `i * (angle_deg / n)` (i = 0..n-1).
    /// Returns a `TopoDS_Compound`.
    pub fn polar_pattern(&self, n: i32, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_polar_pattern(&self.inner, n, angle_deg)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("polar_pattern(n={n}, angle_deg={angle_deg})"),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// `nx * ny` translated copies arranged in a grid: copy `(i, j)` is at
    /// position `(i * dx, j * dy, 0)`.  Implemented as two nested
    /// `linear_pattern` calls — no new C++ needed.
    pub fn grid_pattern(&self, nx: i32, ny: i32, dx: f64, dy: f64) -> Result<Shape, String> {
        if nx < 1 || ny < 1 {
            return Err("grid_pattern: nx and ny must be >= 1".to_string());
        }
        // Build a row of nx copies along X, then replicate it ny times along Y.
        let row = self.linear_pattern(nx, dx, 0.0, 0.0)?;
        row.linear_pattern(ny, 0.0, dy, 0.0)
    }

    /// Fold-left fuse over a slice of shapes.  Requires at least two shapes.
    pub fn fuse_all(shapes: &[&Shape]) -> Result<Shape, String> {
        if shapes.len() < 2 {
            return Err("fuse_all: requires at least 2 shapes".to_string());
        }
        let mut iter = shapes.iter();
        let first = *iter
            .next()
            .expect("fuse_all: invariant: at least 2 shapes after len check");
        let second = *iter
            .next()
            .expect("fuse_all: invariant: at least 2 shapes after len check");
        let mut acc = first.fuse(second)?;
        for s in iter {
            acc = acc.fuse(s)?;
        }
        Ok(acc)
    }

    /// Subtract each tool from `self` in sequence (fold-left cut).  Requires at least one tool.
    pub fn cut_all(&self, tools: &[&Shape]) -> Result<Shape, String> {
        if tools.is_empty() {
            return Err("cut_all: requires at least 1 tool".to_string());
        }
        // Clone `self` via a no-op translate: Shape has no Clone impl because
        // cxx UniquePtr<OcctShape> is not automatically cloneable.  A zero-vector
        // BRepBuilderAPI_Transform is the lightest-weight way to get an owned copy.
        let mut acc = self.translate(0.0, 0.0, 0.0)?;
        for tool in tools {
            acc = acc.cut(tool)?;
        }
        Ok(acc)
    }

    // --- Phase 8 Tier 5: Advanced composition ---

    /// Fragment all shapes in the slice at their mutual intersection boundaries.
    /// Returns a Compound of all non-overlapping pieces.
    /// Uses `BRepAlgoAPI_BuilderAlgo` internally.
    pub fn fragment_all(shapes: &[&Shape]) -> Result<Shape, String> {
        if shapes.is_empty() {
            return Err("fragment: requires at least 1 shape".to_string());
        }
        let mut builder = ffi::fragment_new().map_err(|e| e.to_string())?;
        for s in shapes {
            ffi::fragment_add(builder.pin_mut(), &s.inner).map_err(|e| e.to_string())?;
        }
        ffi::fragment_build(builder.pin_mut())
            .map(|p| {
                let summary = shapes
                    .iter()
                    .map(|s| summarize(s))
                    .collect::<Vec<_>>()
                    .join(", ");
                Shape::fresh_with_history(p, format!("fragment_all(shapes=[{summary}])"))
            })
            .map_err(|e| e.to_string())
    }

    /// 3-D convex hull of the shape's tessellated mesh vertices.
    pub fn convex_hull(&self) -> Result<Shape, String> {
        ffi::shape_convex_hull(&self.inner)
            .map(|p| self.with_inner_and_history(p, "convex_hull()".to_string()))
            .map_err(|e| e.to_string())
    }

    /// Distribute `n` arc-length-evenly-spaced copies of `self` along `path`.
    /// Each copy is oriented so its local Z-axis aligns with the path tangent.
    pub fn path_pattern(&self, path: &Shape, n: i32) -> Result<Shape, String> {
        ffi::shape_path_pattern(&self.inner, &path.inner, n)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!("path_pattern(path={}, n={n})", summarize(path)),
                )
            })
            .map_err(|e| e.to_string())
    }

    /// Guided sweep: sweep `self` (a profile Wire/Face) along `path` while
    /// keeping the profile orientation locked to the `guide` auxiliary Wire.
    pub fn sweep_guide(&self, path: &Shape, guide: &Shape) -> Result<Shape, String> {
        ffi::shape_sweep_guide(&self.inner, &path.inner, &guide.inner)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!(
                        "sweep_guide(profile={}, path={}, guide={})",
                        summarize(self),
                        summarize(path),
                        summarize(guide)
                    ),
                )
            })
            .map_err(|e| {
                format!(
                    "sweep(profile={}, path={}, guide={}) failed: {e}{}",
                    summarize(self),
                    summarize(path),
                    summarize(guide),
                    hint("profile must be a Face or Wire; path and guide must both be Wires that don't kink sharply")
                )
            })
    }

    // --- Import ---

    pub fn import_step(path: &str) -> Result<Self, String> {
        ffi::import_step(path)
            .map(|p| Shape::fresh_with_history(p, format!("import_step(path={path:?})")))
            .map_err(|e| {
                format!(
                    "import_step({path:?}) failed: {e}{}",
                    hint("check that the path exists and is readable; STEP files end in .step or .stp")
                )
            })
    }

    pub fn import_stl(path: &str) -> Result<Self, String> {
        ffi::import_stl(path)
            .map(|p| Shape::fresh_with_history(p, format!("import_stl(path={path:?})")))
            .map_err(|e| {
                format!(
                    "import_stl({path:?}) failed: {e}{}",
                    hint("check that the path exists and is readable; STL files end in .stl")
                )
            })
    }

    // --- Export ---

    pub fn export_step(&self, path: &str) -> Result<(), String> {
        ffi::export_step(&self.inner, path).map_err(|e| {
            self.fail_with_debug(
                format!("export_step({path:?}) on {} failed: {e}", summarize(self)),
                "export_step",
                &[("input", self)],
            )
        })
    }

    pub fn export_stl(&self, path: &str) -> Result<(), String> {
        ffi::export_stl(&self.inner, path).map_err(|e| {
            self.fail_with_debug(
                format!("export_stl({path:?}) on {} failed: {e}", summarize(self)),
                "export_stl",
                &[("input", self)],
            )
        })
    }

    /// Export to glTF. `linear_deflection` controls tessellation quality (e.g. `0.1` for 0.1 mm).
    pub fn export_gltf(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_gltf(&self.inner, path, linear_deflection).map_err(|e| {
            self.fail_with_debug(
                format!("export_gltf({path:?}) on {} failed: {e}", summarize(self)),
                "export_gltf",
                &[("input", self)],
            )
        })
    }

    /// Export to binary glTF (GLB). Single-file format suitable for HTTP serving.
    pub fn export_glb(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_glb(&self.inner, path, linear_deflection).map_err(|e| {
            self.fail_with_debug(
                format!("export_glb({path:?}) on {} failed: {e}", summarize(self)),
                "export_glb",
                &[("input", self)],
            )
        })
    }

    /// Export to Wavefront OBJ. Tessellates with `linear_deflection` and writes
    /// the `.obj` file plus a companion `.mtl` material file in the same directory.
    pub fn export_obj(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_obj(&self.inner, path, linear_deflection).map_err(|e| {
            self.fail_with_debug(
                format!("export_obj({path:?}) on {} failed: {e}", summarize(self)),
                "export_obj",
                &[("input", self)],
            )
        })
    }

    /// Export to SVG using hidden-line removal (HLRBRep_PolyAlgo).
    /// `view` is `"top"` (default), `"front"`, or `"side"`.
    /// `scale` multiplies drawing geometry; `1.0` preserves model units.
    /// `hidden` includes hidden HLR edges as dashed secondary geometry.
    /// `center_marks` adds crosshair marks for cylindrical faces aligned to the view axis.
    /// `dimensions` adds overall width and height annotations.
    /// `callouts` adds diameter callouts for cylindrical faces aligned to the view axis.
    /// `datum` and `feature_control` add a simple framed GD&T annotation block.
    #[allow(clippy::too_many_arguments)]
    pub fn export_svg(
        &self,
        path: &str,
        view: &str,
        scale: f64,
        hidden: bool,
        center_marks: bool,
        dimensions: bool,
        title_block: bool,
        callouts: bool,
        datum: &str,
        feature_control: &str,
        tolerance_plus: f64,
        tolerance_minus: f64,
    ) -> Result<(), String> {
        self.export_svg_with_anchor(
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            datum,
            false,
            0.0,
            0.0,
            0.0,
            feature_control,
            false,
            0.0,
            0.0,
            0.0,
            tolerance_plus,
            tolerance_minus,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn export_svg_with_anchor(
        &self,
        path: &str,
        view: &str,
        scale: f64,
        hidden: bool,
        center_marks: bool,
        dimensions: bool,
        title_block: bool,
        callouts: bool,
        datum: &str,
        datum_anchor_valid: bool,
        datum_anchor_x: f64,
        datum_anchor_y: f64,
        datum_anchor_z: f64,
        feature_control: &str,
        feature_control_anchor_valid: bool,
        feature_control_anchor_x: f64,
        feature_control_anchor_y: f64,
        feature_control_anchor_z: f64,
        tolerance_plus: f64,
        tolerance_minus: f64,
    ) -> Result<(), String> {
        let (
            datum,
            datum_anchor_valid,
            datum_anchor,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor,
        ) = self.gdt_export_inputs(
            datum,
            datum_anchor_valid,
            datum_anchor_x,
            datum_anchor_y,
            datum_anchor_z,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
        );
        ffi::export_svg(
            &self.inner,
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            &datum,
            datum_anchor_valid,
            datum_anchor[0],
            datum_anchor[1],
            datum_anchor[2],
            &feature_control,
            feature_control_anchor_valid,
            feature_control_anchor[0],
            feature_control_anchor[1],
            feature_control_anchor[2],
            tolerance_plus,
            tolerance_minus,
        )
        .map_err(|e| {
            self.fail_with_debug(
                format!(
                    "export_svg({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}) on {} failed: {e}",
                    summarize(self)
                ),
                "export_svg",
                &[("input", self)],
            )
        })
    }

    /// Export to DXF R12 using hidden-line removal (HLRBRep_PolyAlgo).
    /// `view` is `"top"` (default), `"front"`, or `"side"`.
    /// `scale` multiplies drawing geometry; `1.0` preserves model units.
    /// `hidden` includes hidden HLR edges on a `HIDDEN` layer.
    /// `center_marks` adds crosshair marks on a `CENTER` layer.
    /// `dimensions` adds overall width/height labels.
    /// `callouts` adds diameter callouts on a `CALLOUT` layer.
    /// `datum` and `feature_control` add a simple framed GD&T annotation block.
    #[allow(clippy::too_many_arguments)]
    pub fn export_dxf(
        &self,
        path: &str,
        view: &str,
        scale: f64,
        hidden: bool,
        center_marks: bool,
        dimensions: bool,
        title_block: bool,
        callouts: bool,
        datum: &str,
        feature_control: &str,
        tolerance_plus: f64,
        tolerance_minus: f64,
    ) -> Result<(), String> {
        self.export_dxf_with_anchor(
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            datum,
            false,
            0.0,
            0.0,
            0.0,
            feature_control,
            false,
            0.0,
            0.0,
            0.0,
            tolerance_plus,
            tolerance_minus,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn export_dxf_with_anchor(
        &self,
        path: &str,
        view: &str,
        scale: f64,
        hidden: bool,
        center_marks: bool,
        dimensions: bool,
        title_block: bool,
        callouts: bool,
        datum: &str,
        datum_anchor_valid: bool,
        datum_anchor_x: f64,
        datum_anchor_y: f64,
        datum_anchor_z: f64,
        feature_control: &str,
        feature_control_anchor_valid: bool,
        feature_control_anchor_x: f64,
        feature_control_anchor_y: f64,
        feature_control_anchor_z: f64,
        tolerance_plus: f64,
        tolerance_minus: f64,
    ) -> Result<(), String> {
        let (
            datum,
            datum_anchor_valid,
            datum_anchor,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor,
        ) = self.gdt_export_inputs(
            datum,
            datum_anchor_valid,
            datum_anchor_x,
            datum_anchor_y,
            datum_anchor_z,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
        );
        ffi::export_dxf(
            &self.inner,
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            &datum,
            datum_anchor_valid,
            datum_anchor[0],
            datum_anchor[1],
            datum_anchor[2],
            &feature_control,
            feature_control_anchor_valid,
            feature_control_anchor[0],
            feature_control_anchor[1],
            feature_control_anchor[2],
            tolerance_plus,
            tolerance_minus,
        )
        .map_err(|e| {
            self.fail_with_debug(
                format!(
                    "export_dxf({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}) on {} failed: {e}",
                    summarize(self)
                ),
                "export_dxf",
                &[("input", self)],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Shape;

    #[test]
    fn smoke_filleted_box_to_step() {
        let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box failed");
        let filleted = shape.fillet(2.0).expect("fillet failed");

        let out = std::env::temp_dir().join("rrcad_smoke_filleted_box.step");
        filleted
            .export_step(out.to_str().unwrap())
            .expect("export_step failed");

        assert!(out.exists(), "STEP file was not created");
        assert!(
            std::fs::metadata(&out).unwrap().len() > 0,
            "STEP file is empty"
        );
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(
            content.contains("ISO-10303-21"),
            "output does not look like a STEP file"
        );
    }

    #[test]
    fn smoke_boolean_cut() {
        let base = Shape::make_box(20.0, 20.0, 20.0).unwrap();
        let cyl = Shape::make_cylinder(5.0, 25.0).unwrap();
        let result = base.cut(&cyl).expect("boolean cut failed");

        let out = std::env::temp_dir().join("rrcad_smoke_cut.step");
        result.export_step(out.to_str().unwrap()).unwrap();
        assert!(out.exists());
    }

    #[test]
    fn fillet_sel_vertical_only() {
        // A box has 4 vertical + 8 horizontal edges.
        // Filleting only :vertical edges should succeed and produce more edges
        // than the unfilleted box (each rounded edge becomes an arc).
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let filleted = b
            .fillet_sel(1.0, "vertical")
            .expect("fillet_sel vertical failed");
        let new_edge_count = filleted.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after selective fillet, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn chamfer_sel_horizontal_only() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let chamfered = b
            .chamfer_sel(1.0, "horizontal")
            .expect("chamfer_sel horizontal failed");
        let new_edge_count = chamfered.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after selective chamfer, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn fillet_var_all_edges_produces_valid_shape() {
        // Variable-radius fillet: r=0.5 at one vertex, r=2.0 at the other.
        // A box(10,10,10) has 12 edges; after filleting all, the edge count rises.
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let filleted = b.fillet_var(0.5, 2.0).expect("fillet_var failed");
        let new_edge_count = filleted.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after variable-radius fillet, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn fillet_var_sel_vertical_only() {
        // Variable-radius fillet on just the 4 vertical edges.
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let filleted = b
            .fillet_var_sel(0.5, 2.0, "vertical")
            .expect("fillet_var_sel vertical failed");
        let new_edge_count = filleted.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after variable-radius selective fillet, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn scale_xyz_stretches_bounding_box() {
        // box(1,1,1) scaled by (2,3,4) should produce extents (2,3,4).
        let unit = Shape::make_box(1.0, 1.0, 1.0).unwrap();
        let scaled = unit.scale_xyz(2.0, 3.0, 4.0).expect("scale_xyz failed");
        let bb = scaled.bounding_box().expect("bounding_box failed");
        // bb = [xmin, ymin, zmin, xmax, ymax, zmax]
        let (dx, dy, dz) = (bb[3] - bb[0], bb[4] - bb[1], bb[5] - bb[2]);
        assert!((dx - 2.0).abs() < 1e-6, "expected dx=2, got {dx}");
        assert!((dy - 3.0).abs() < 1e-6, "expected dy=3, got {dy}");
        assert!((dz - 4.0).abs() < 1e-6, "expected dz=4, got {dz}");
    }

    #[test]
    fn linear_pattern_copies_along_axis() {
        // 3 copies of a 2×2×2 box spaced 5 units apart along X should have
        // a bounding box that spans [0, 0+5+5+2] = [0, 12] in X.
        let b = Shape::make_box(2.0, 2.0, 2.0).unwrap();
        let pattern = b
            .linear_pattern(3, 5.0, 0.0, 0.0)
            .expect("linear_pattern failed");
        let bb = pattern.bounding_box().expect("bounding_box failed");
        let dx = bb[3] - bb[0]; // xmax - xmin
        assert!(
            (dx - 12.0).abs() < 1e-4,
            "expected x-extent of 12, got {dx}"
        );
    }

    #[test]
    fn polar_pattern_fills_circle() {
        // 4 copies at 360° — each rotated 90° further — should span roughly
        // the same extents in X and Y.
        let b = Shape::make_box(1.0, 1.0, 5.0)
            .unwrap()
            .translate(3.0, 0.0, 0.0)
            .unwrap();
        let pattern = b.polar_pattern(4, 360.0).expect("polar_pattern failed");
        let bb = pattern.bounding_box().expect("bounding_box failed");
        let dx = bb[3] - bb[0];
        let dy = bb[4] - bb[1];
        // With 4 copies at 90° intervals, the compound should be roughly symmetric.
        assert!(
            (dx - dy).abs() < 0.5,
            "expected symmetric extents, got dx={dx}, dy={dy}"
        );
    }

    #[test]
    fn linear_pattern_n1_returns_original_shape() {
        // n=1 should produce a single-copy compound with the same bounding box.
        let b = Shape::make_box(3.0, 4.0, 5.0).unwrap();
        let bb_orig = b.bounding_box().unwrap();
        let pattern = b
            .linear_pattern(1, 10.0, 0.0, 0.0)
            .expect("linear_pattern n=1 failed");
        let bb_pat = pattern.bounding_box().unwrap();
        let orig_dx = bb_orig[3] - bb_orig[0];
        let pat_dx = bb_pat[3] - bb_pat[0];
        assert!(
            (orig_dx - pat_dx).abs() < 1e-4,
            "n=1 pattern should match original x-extent"
        );
    }

    #[test]
    fn vertices_count_box() {
        // A box has exactly 8 corners.
        let b = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let verts = b.vertices("all").expect("vertices failed");
        assert_eq!(
            verts.len(),
            8,
            "expected 8 vertices on a box, got {}",
            verts.len()
        );
    }

    #[test]
    fn vertices_bad_selector_returns_error() {
        let b = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let result = b.vertices("top");
        assert!(
            result.is_err(),
            "expected error for unsupported selector 'top'"
        );
    }

    #[test]
    fn faces_direction_selector_top() {
        // A box has exactly 1 top face (normal pointing in +Z).
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let top_faces = b.faces(">Z").expect("faces(>Z) failed");
        assert_eq!(
            top_faces.len(),
            1,
            "expected 1 top face, got {}",
            top_faces.len()
        );
    }

    #[test]
    fn faces_direction_selector_bottom() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let bottom_faces = b.faces("<Z").expect("faces(<Z) failed");
        assert_eq!(
            bottom_faces.len(),
            1,
            "expected 1 bottom face, got {}",
            bottom_faces.len()
        );
    }

    #[test]
    fn faces_direction_selector_x_sides() {
        // A box has 2 faces with normals along the X axis.
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let pos_x = b.faces(">X").expect("faces(>X) failed");
        let neg_x = b.faces("<X").expect("faces(<X) failed");
        assert_eq!(pos_x.len(), 1, "expected 1 +X face");
        assert_eq!(neg_x.len(), 1, "expected 1 -X face");
    }

    #[test]
    fn export_obj_creates_file() {
        let b = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let out = std::env::temp_dir().join("rrcad_test_export.obj");
        b.export_obj(out.to_str().unwrap(), 0.1)
            .expect("export_obj failed");
        assert!(out.exists(), "OBJ file was not created");
        assert!(
            std::fs::metadata(&out).unwrap().len() > 0,
            "OBJ file is empty"
        );
    }

    // --- Color ---

    #[test]
    fn set_color_returns_new_shape() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        // set_color must succeed and produce a usable shape.
        let colored = b.set_color(1.0, 0.0, 0.0).expect("set_color failed");
        // The colored shape should export cleanly to GLB.
        let out = std::env::temp_dir().join("rrcad_test_colored.glb");
        colored
            .export_glb(out.to_str().unwrap(), 0.1)
            .expect("export_glb on colored shape failed");
        assert!(out.exists(), "GLB file was not created");
        assert!(
            std::fs::metadata(&out).unwrap().len() > 0,
            "GLB file is empty"
        );
    }

    // --- Assembly mating ---

    #[test]
    fn mate_stacks_box_on_box_z() {
        // A 5×5×5 post mated (bottom → top) onto a 10×10×10 base.
        // The base occupies Z = 0..10; the post should end up at Z = 10..15.
        let base = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let post = Shape::make_box(5.0, 5.0, 5.0).unwrap();

        let from_faces = post.faces("bottom").unwrap();
        let to_faces = base.faces("top").unwrap();

        let mated = post.mate(&from_faces[0], &to_faces[0], 0.0).unwrap();
        let bb = mated.bounding_box().unwrap();
        // bb = [xmin, ymin, zmin, xmax, ymax, zmax]
        assert!(
            (bb[2] - 10.0).abs() < 0.01,
            "Zmin should be ≈10, got {}",
            bb[2]
        );
        assert!(
            (bb[5] - 15.0).abs() < 0.01,
            "Zmax should be ≈15, got {}",
            bb[5]
        );
    }

    #[test]
    fn mate_with_offset_leaves_gap() {
        let base = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let post = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let from_faces = post.faces("bottom").unwrap();
        let to_faces = base.faces("top").unwrap();
        let mated = post.mate(&from_faces[0], &to_faces[0], 3.0).unwrap();
        let bb = mated.bounding_box().unwrap();
        // With offset=3, post bottom should be at Z=13 (10 + 3 gap).
        assert!(
            (bb[2] - 13.0).abs() < 0.01,
            "Zmin should be ≈13, got {}",
            bb[2]
        );
    }

    #[test]
    fn mate_non_planar_face_returns_error() {
        let cyl = Shape::make_cylinder(5.0, 10.0).unwrap();
        let base = Shape::make_box(20.0, 20.0, 5.0).unwrap();
        // Side face of a cylinder is non-planar — mate should error.
        let side_faces = cyl.faces("side").unwrap();
        let to_faces = base.faces("top").unwrap();
        let result = cyl.mate(&side_faces[0], &to_faces[0], 0.0);
        match result {
            Ok(_) => panic!("expected error for non-planar from-face"),
            Err(err) => assert!(err.contains("planar"), "unexpected error: {err}"),
        }
    }

    #[test]
    fn set_color_does_not_modify_original() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let _colored = b.set_color(0.0, 1.0, 0.0).expect("set_color failed");
        // Original shape must still export without error.
        let out = std::env::temp_dir().join("rrcad_test_uncolored.glb");
        b.export_glb(out.to_str().unwrap(), 0.1)
            .expect("original shape export failed after set_color");
    }
}
