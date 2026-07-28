use std::sync::Arc;

use super::{GdtRenderSpec, NamedRef, Shape, ffi};

impl Shape {
    pub fn mate(&self, from_face: &Shape, to_face: &Shape, offset: f64) -> Result<Shape, String> {
        ffi::shape_mate(&self.inner, &from_face.inner, &to_face.inner, offset)
            .map(|p| {
                self.with_inner_and_history(
                    p,
                    format!(
                        "mate(from_face={}, to_face={}, offset={offset})",
                        super::shape_core::summarize(from_face),
                        super::shape_core::summarize(to_face)
                    ),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn set_color(&self, r: f64, g: f64, b: f64) -> Result<Shape, String> {
        ffi::shape_set_color(&self.inner, r, g, b)
            .map(|p| self.with_inner_and_history(p, format!("set_color(r={r}, g={g}, b={b})")))
            .map_err(|e| e.to_string())
    }

    pub fn name_face(&self, name: &str, selector: &str) -> Result<(), String> {
        self.faces(selector)?;
        self.set_named_ref(name, NamedRef::FaceSelector(selector.to_string()));
        Ok(())
    }

    pub fn name_edge(&self, name: &str, selector: &str) -> Result<(), String> {
        self.edges(selector)?;
        self.set_named_ref(name, NamedRef::EdgeSelector(selector.to_string()));
        Ok(())
    }

    // Allowed: Shape wraps OCCT handles that are not Send/Sync by design; the
    // crate never shares Shapes across threads, so Arc is used only for cheap
    // clones within one thread.
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn datum(&self, name: &str, shape: &Shape) -> Result<(), String> {
        self.set_named_ref(name, NamedRef::Datum(Arc::new(shape.clone())));
        Ok(())
    }

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

    pub(crate) fn gdt_apply(&self, spec: GdtRenderSpec) {
        self.set_gdt_render(Some(spec));
    }
}
