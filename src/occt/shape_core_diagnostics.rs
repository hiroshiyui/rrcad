use super::{GdtFeatureControlSpec, GdtRenderSpec, Shape, ffi};
use crate::occt::shape_core::{
    DEBUG_EXPORT_SEQ, debug_export_component, debug_exports_enabled, debug_exports_root,
};
use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

impl Shape {
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
            named_refs: std::cell::RefCell::new(self.named_refs.borrow().clone()),
            gdt_render: std::cell::RefCell::new(self.gdt_render.borrow().clone()),
            history: std::cell::RefCell::new(self.history.borrow().clone()),
            feature: self.feature.clone(),
        }
    }
}
