use super::{FeatureNode, FeatureOp, Shape, ffi};
use cxx::UniquePtr;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

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
