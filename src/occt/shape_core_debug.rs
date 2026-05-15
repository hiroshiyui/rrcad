use std::{env, path::PathBuf, sync::atomic::AtomicUsize};

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
