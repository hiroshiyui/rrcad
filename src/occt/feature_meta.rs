use crate::occt::Shape;
use std::sync::Arc;

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
