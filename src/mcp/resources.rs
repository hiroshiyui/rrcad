use std::sync::Arc;

use serde_json::{Map, Value, json};

use super::security::MCP_EXPORT_FORMATS;

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
                    // Derived from the single allowlist shared with validate_format().
                    "enum": MCP_EXPORT_FORMATS
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

#[cfg(test)]
mod tests {
    use super::{api_doc, build_examples_content, code_format_schema, code_schema};

    #[test]
    fn api_doc_mentions_core_dsl_entrypoints() {
        let doc = api_doc();
        assert!(doc.contains("box("), "api doc should mention primitives");
        assert!(
            doc.contains("preview(shape)"),
            "api doc should mention preview"
        );
    }

    #[test]
    fn code_schema_defines_code_object() {
        let schema = code_schema();
        let props = schema
            .get("properties")
            .and_then(|v| v.as_object())
            .unwrap();
        assert!(props.contains_key("code"));
        assert_eq!(
            props
                .get("code")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str()),
            Some("string")
        );
        let required = schema.get("required").and_then(|v| v.as_array()).unwrap();
        assert_eq!(required, &vec![serde_json::Value::from("code")]);
    }

    #[test]
    fn code_format_schema_defines_export_formats() {
        let schema = code_format_schema();
        let props = schema
            .get("properties")
            .and_then(|v| v.as_object())
            .unwrap();
        let format = props.get("format").and_then(|v| v.as_object()).unwrap();
        assert_eq!(
            format.get("enum").and_then(|v| v.as_array()).unwrap(),
            &vec![
                serde_json::Value::from("step"),
                serde_json::Value::from("stl"),
                serde_json::Value::from("3mf"),
                serde_json::Value::from("glb"),
                serde_json::Value::from("gltf"),
                serde_json::Value::from("obj"),
            ]
        );
        let required = schema.get("required").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            required,
            &vec![
                serde_json::Value::from("code"),
                serde_json::Value::from("format"),
            ]
        );
    }

    #[test]
    fn build_examples_content_lists_all_sample_scripts() {
        let content = build_examples_content();
        let names = [
            "01_hello_box.rb",
            "02_boolean_ops.rb",
            "03_transforms.rb",
            "04_bracket.rb",
            "05_export_formats.rb",
            "06_live_preview.rb",
            "07_teapot.rb",
            "08_parametric_box.rb",
        ];

        let mut last = 0usize;
        for name in names {
            let needle = format!("# ===== {name} =====");
            let idx = content[last..]
                .find(&needle)
                .map(|offset| last + offset)
                .unwrap_or_else(|| panic!("missing sample header {needle}"));
            last = idx + needle.len();
        }
    }
}
