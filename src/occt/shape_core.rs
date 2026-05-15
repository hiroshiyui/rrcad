use super::{FeatureNode, FeatureOp, Shape, ffi};
use cxx::UniquePtr;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

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

pub(crate) static DEBUG_EXPORT_SEQ: AtomicUsize = AtomicUsize::new(1);

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
}
