use super::{NamedRef, NamedRefSnapshot, Shape};

impl Shape {
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
}
