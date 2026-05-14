use std::sync::Arc;

use serde_json::{Map, Value, json};

/// Full DSL API reference (doc/api.md), embedded at compile time.
pub(crate) fn api_doc() -> &'static str {
    include_str!("../../doc/api.md")
}

/// JSON Schema for tools that accept a single `code` parameter.
pub(crate) fn code_schema() -> Arc<Map<String, Value>> {
    Arc::new(
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Ruby DSL CAD code to evaluate. Uses the rrcad DSL (box, cylinder, sphere, fuse, cut, extrude, etc.)."
                }
            },
            "required": ["code"]
        })
        .as_object()
        .unwrap()
        .clone(),
    )
}

/// JSON Schema for `cad_export` which takes `code` plus a `format` enum.
pub(crate) fn code_format_schema() -> Arc<Map<String, Value>> {
    Arc::new(
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Ruby DSL CAD code to evaluate and export."
                },
                "format": {
                    "type": "string",
                    "description": "Export file format.",
                    "enum": ["step", "stl", "glb", "gltf", "obj"]
                }
            },
            "required": ["code", "format"]
        })
        .as_object()
        .unwrap()
        .clone(),
    )
}

/// Concatenate all sample scripts into one text block for `rrcad://examples`.
pub(crate) fn build_examples_content() -> String {
    let samples: &[(&str, &str)] = &[
        (
            "01_hello_box.rb",
            include_str!("../../samples/01_hello_box.rb"),
        ),
        (
            "02_boolean_ops.rb",
            include_str!("../../samples/02_boolean_ops.rb"),
        ),
        (
            "03_transforms.rb",
            include_str!("../../samples/03_transforms.rb"),
        ),
        ("04_bracket.rb", include_str!("../../samples/04_bracket.rb")),
        (
            "05_export_formats.rb",
            include_str!("../../samples/05_export_formats.rb"),
        ),
        (
            "06_live_preview.rb",
            include_str!("../../samples/06_live_preview.rb"),
        ),
        ("07_teapot.rb", include_str!("../../samples/07_teapot.rb")),
        (
            "08_parametric_box.rb",
            include_str!("../../samples/08_parametric_box.rb"),
        ),
    ];

    let mut buf = String::new();
    for (name, content) in samples {
        buf.push_str(&format!("# ===== {name} =====\n\n{content}\n\n"));
    }
    buf
}
