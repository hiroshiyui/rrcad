use super::{
    FeatureNode, FeatureOp, GdtFeatureControlSpec, GdtRenderSpec, NamedRef, NamedRefSnapshot,
    Shape, ffi,
};
use cxx::UniquePtr;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Best-effort one-word shape kind ("solid", "face", "wire", …) used to
/// enrich error messages. Falls back to `"shape"` if the type query
/// itself fails (e.g. the inner pointer is in a degraded state).
pub(crate) fn summarize(shape: &Shape) -> String {
    ffi::shape_type_str(&shape.inner)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "shape".to_string())
}

/// Format a one-line actionable hint for the error message. Returns the
/// empty string when `s` is empty so callers can unconditionally append.
pub(crate) fn hint(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        format!("\n  hint: {s}")
    }
}

static DEBUG_EXPORT_SEQ: AtomicUsize = AtomicUsize::new(1);

pub(crate) fn debug_exports_enabled() -> bool {
    matches!(
        env::var("RRCAD_DEBUG_EXPORTS")
            .ok()
            .as_deref()
            .map(|v| v.to_ascii_lowercase())
            .as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

pub(crate) fn debug_exports_root() -> PathBuf {
    env::var_os("RRCAD_DEBUG_EXPORTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("rrcad-debug"))
}

pub(crate) fn debug_export_component(s: &str) -> String {
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
    pub(crate) fn fresh(inner: UniquePtr<ffi::OcctShape>) -> Self {
        Self {
            inner,
            named_refs: RefCell::new(BTreeMap::new()),
            gdt_render: RefCell::new(None),
            history: RefCell::new(Vec::new()),
            feature: FeatureNode::new(
                FeatureOp::Opaque {
                    label: "shape".to_string(),
                },
                Vec::new(),
                "shape".to_string(),
            ),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn fresh_with_history(
        inner: UniquePtr<ffi::OcctShape>,
        entry: impl Into<String>,
    ) -> Self {
        let shape = Self::fresh(inner);
        let entry = entry.into();
        shape.history.borrow_mut().push(entry.clone());
        let feature = FeatureNode::new(
            FeatureOp::Opaque {
                label: entry.clone(),
            },
            Vec::new(),
            entry,
        );
        let mut shape = shape;
        shape.feature = feature;
        shape
    }

    pub(crate) fn with_inner(&self, inner: UniquePtr<ffi::OcctShape>) -> Self {
        Self {
            inner,
            named_refs: RefCell::new(self.named_refs.borrow().clone()),
            gdt_render: RefCell::new(self.gdt_render.borrow().clone()),
            history: RefCell::new(self.history.borrow().clone()),
            feature: self.feature.clone(),
        }
    }

    pub(crate) fn with_inner_and_history(
        &self,
        inner: UniquePtr<ffi::OcctShape>,
        entry: impl Into<String>,
    ) -> Self {
        let shape = self.with_inner(inner);
        shape.history.borrow_mut().push(entry.into());
        shape
    }

    pub(crate) fn with_feature(
        &self,
        inner: UniquePtr<ffi::OcctShape>,
        op: FeatureOp,
        entry: impl Into<String>,
        parents: Vec<Arc<FeatureNode>>,
    ) -> Self {
        let entry = entry.into();
        Self {
            inner,
            named_refs: RefCell::new(self.named_refs.borrow().clone()),
            gdt_render: RefCell::new(self.gdt_render.borrow().clone()),
            history: RefCell::new({
                let mut hist = self.history.borrow().clone();
                hist.push(entry.clone());
                hist
            }),
            feature: FeatureNode::new(op, parents, entry),
        }
    }

    pub(crate) fn fresh_with_feature(
        inner: UniquePtr<ffi::OcctShape>,
        op: FeatureOp,
        entry: impl Into<String>,
    ) -> Self {
        let entry = entry.into();
        let mut shape = Self::fresh(inner);
        shape.history.borrow_mut().push(entry.clone());
        shape.feature = FeatureNode::new(op, Vec::new(), entry);
        shape
    }

    pub(crate) fn fresh_with_feature_parents(
        inner: UniquePtr<ffi::OcctShape>,
        op: FeatureOp,
        entry: impl Into<String>,
        parents: Vec<Arc<FeatureNode>>,
    ) -> Self {
        let entry = entry.into();
        let mut shape = Self::fresh(inner);
        shape.history.borrow_mut().push(entry.clone());
        shape.feature = FeatureNode::new(op, parents, entry);
        shape
    }

    pub(crate) fn named_ref(&self, name: &str) -> Option<NamedRef> {
        self.named_refs.borrow().get(name).cloned()
    }

    pub(crate) fn set_named_ref(&self, name: impl Into<String>, named: NamedRef) {
        self.named_refs.borrow_mut().insert(name.into(), named);
    }

    pub(crate) fn resolve_named_selector(&self, name: &str) -> Option<NamedRef> {
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

    pub(crate) fn gdt_render(&self) -> Option<GdtRenderSpec> {
        self.gdt_render.borrow().clone()
    }

    pub(crate) fn set_gdt_render(&self, render: Option<GdtRenderSpec>) {
        *self.gdt_render.borrow_mut() = render;
    }

    pub fn history(&self) -> Vec<String> {
        self.history.borrow().clone()
    }

    pub fn feature_graph(&self) -> String {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        self.feature.snapshot_lines(&mut out, &mut seen);
        out.join("\n")
    }

    pub fn rebuild(&self) -> Result<Self, String> {
        self.feature.rebuild()
    }

    fn history_tail(&self, max_entries: usize) -> Option<String> {
        let history = self.history.borrow();
        if history.is_empty() {
            return None;
        }
        let start = history.len().saturating_sub(max_entries);
        Some(history[start..].join(" -> "))
    }

    pub(crate) fn history_note(&self, shapes: &[(&str, &Shape)]) -> Option<String> {
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

    pub(crate) fn debug_export_note(&self, op: &str, shapes: &[(&str, &Shape)]) -> Option<String> {
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

    pub(crate) fn fail_with_debug(
        &self,
        base: String,
        op: &str,
        shapes: &[(&str, &Shape)],
    ) -> String {
        let mut msg = base;
        if let Some(note) = self.history_note(shapes) {
            msg.push_str(&note);
        }
        if let Some(note) = self.debug_export_note(op, shapes) {
            msg.push_str(&note);
        }
        msg
    }

    pub(crate) fn format_gdt_feature_control(
        standard: &super::GdtStandard,
        fc: &GdtFeatureControlSpec,
    ) -> String {
        let mut parts = Vec::new();
        match standard {
            super::GdtStandard::Asme => {
                parts.push(fc.text.clone());
                if !fc.modifiers.is_empty() {
                    parts.push(fc.modifiers.join(" "));
                }
                parts.extend(fc.datums.iter().cloned());
            }
            super::GdtStandard::Iso => {
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
    pub(crate) fn gdt_export_inputs(
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
            feature: self.feature.clone(),
        }
    }
}
