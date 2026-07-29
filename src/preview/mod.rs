//! Live preview orchestration.
//!
//! `PREVIEW` is a process-wide singleton initialised by `start()` when the
//! CLI is invoked with `--preview`.  The Ruby `preview(shape)` method writes
//! the GLB file and fires the reload broadcast; the axum server picks it up.

pub mod server;

use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct PreviewState {
    /// Path to the temporary GLB file the server streams to the browser.
    pub glb_path: PathBuf,
    /// Send `()` here to push a "reload" WebSocket message to all clients.
    pub reload_tx: broadcast::Sender<()>,
}

/// Initialised once by `start()`; accessed from both the axum handlers and
/// `rrcad_preview_shape` (called on the main thread from mRuby).
pub static PREVIEW: OnceLock<PreviewState> = OnceLock::new();

/// Return the JSON sidecar path used by the browser properties panel.
pub fn metadata_path_for_glb(glb_path: &std::path::Path) -> PathBuf {
    glb_path.with_extension("json")
}

/// Collect lightweight shape properties for the live preview inspector.
pub fn metadata_json_for_shape(shape: &crate::occt::Shape) -> serde_json::Value {
    metadata_json_for_shape_with_error(shape, None)
}

pub fn metadata_json_for_shape_with_error(
    shape: &crate::occt::Shape,
    error: Option<&str>,
) -> serde_json::Value {
    let bounding_box = match shape.bounding_box() {
        Ok([xmin, ymin, zmin, xmax, ymax, zmax]) => serde_json::json!({
            "min": [xmin, ymin, zmin],
            "max": [xmax, ymax, zmax],
            "size": [xmax - xmin, ymax - ymin, zmax - zmin],
        }),
        Err(e) => serde_json::json!({ "error": e }),
    };

    serde_json::json!({
        "shape_type": query_or_error(shape.shape_type_name()),
        "validation": query_or_error(shape.validate()),
        "volume": query_or_error(shape.volume()),
        "surface_area": query_or_error(shape.surface_area()),
        "bounding_box": bounding_box,
        "named_refs": shape.named_ref_snapshots(),
        "feature_graph": feature_graph_json(shape),
        "error": error,
    })
}

/// Parse `Shape::feature_graph()`'s tab-separated snapshot into JSON nodes.
///
/// The snapshot is one line per node — `id \t parent_ids \t label \t entry` —
/// emitted parents-first, so the array is already in topological order and the
/// viewer can lay out the tree in a single pass. `parents` is a comma-separated
/// list that is empty for the roots (primitives and imports).
///
/// A line that does not have the expected shape is skipped rather than
/// poisoning the whole panel: the feature graph is a browsing aid, and losing
/// the model's properties over one malformed row would be a poor trade.
fn feature_graph_json(shape: &crate::occt::Shape) -> serde_json::Value {
    let snapshot = shape.feature_graph();
    let nodes: Vec<serde_json::Value> = snapshot
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            // `splitn(4, …)` keeps any tab inside the entry text with the entry.
            let mut fields = line.splitn(4, '\t');
            let id: u64 = fields.next()?.trim().parse().ok()?;
            let parents: Vec<u64> = fields
                .next()?
                .split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect();
            let label = fields.next()?;
            // The last field is absent when a node recorded no history entry.
            let entry = fields.next().unwrap_or(label);
            Some(serde_json::json!({
                "id": id,
                "parents": parents,
                "label": label,
                "entry": entry,
            }))
        })
        .collect();
    serde_json::Value::Array(nodes)
}

fn query_or_error<T: serde::Serialize>(result: Result<T, String>) -> serde_json::Value {
    match result {
        Ok(value) => serde_json::json!({ "ok": true, "value": value }),
        Err(error) => serde_json::json!({ "ok": false, "error": error }),
    }
}

/// Create the Tokio runtime and bind the preview port *inside* it.
///
/// The order matters. `bind_listener` ends in `tokio::net::TcpListener::from_std`,
/// which registers the socket with the current thread's reactor and panics
/// outright if there is none — so binding before the runtime exists takes the
/// whole process down. Holding the `enter()` guard across the bind is what
/// supplies that context on a plain (non-`#[tokio::main]`) thread.
fn create_runtime_and_bind(
    port: u16,
) -> Result<(tokio::runtime::Runtime, tokio::net::TcpListener), String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("failed to create tokio runtime: {e}"))?;
    let listener = {
        let _guard = rt.enter();
        server::bind_listener(port)?
    };
    Ok((rt, listener))
}

/// Initialise the preview state, spawn the axum server on a background
/// Tokio runtime, and open the browser.
///
/// Returns the runtime so the caller can keep it alive for the process
/// lifetime (drop it to shut down the server). Returns `Err` if the preview
/// state was already initialised or the tokio runtime could not be created.
pub fn start(glb_path: PathBuf, port: u16) -> Result<tokio::runtime::Runtime, String> {
    let (rt, listener) = create_runtime_and_bind(port)?;
    let actual_port = listener
        .local_addr()
        .map_err(|e| format!("failed to get preview port address: {e}"))?
        .port();
    let (reload_tx, _) = broadcast::channel(16);
    PREVIEW
        .set(PreviewState {
            glb_path,
            reload_tx,
        })
        .map_err(|_| "preview::start called more than once".to_string())?;
    rt.spawn(server::serve_with_listener(listener));

    let url = format!("http://localhost:{actual_port}");
    println!("Preview server: {url}  (Ctrl-C to quit)");
    open::that(&url).ok();

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occt::Shape;
    use std::{fs, io::ErrorKind, process::Command, time::SystemTime};

    #[test]
    fn metadata_for_box_includes_core_properties() {
        let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        let metadata = metadata_json_for_shape(&shape);

        assert_eq!(metadata["shape_type"]["value"], "solid");
        assert_eq!(metadata["validation"]["value"], "ok");
        assert_eq!(metadata["volume"]["value"], 6000.0);
        assert_eq!(metadata["surface_area"]["value"], 2200.0);
        assert_eq!(metadata["bounding_box"]["size"][0], 10.0);
        assert_eq!(metadata["bounding_box"]["size"][1], 20.0);
        assert_eq!(metadata["bounding_box"]["size"][2], 30.0);
        assert!(metadata["named_refs"].as_array().is_some());
    }

    #[test]
    fn metadata_for_named_refs_includes_selectors() {
        let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        shape.name_face("mounting_face", "top").expect("name_face");
        shape
            .name_edge("vertical_edges", "vertical")
            .expect("name_edge");
        let metadata = metadata_json_for_shape(&shape);

        let refs = metadata["named_refs"].as_array().expect("named refs array");
        assert!(refs.iter().any(|entry| entry["name"] == "mounting_face"
            && entry["kind"] == "face"
            && entry["selector"] == ":top"));
        assert!(refs.iter().any(|entry| entry["name"] == "vertical_edges"
            && entry["kind"] == "edge"
            && entry["selector"] == ":vertical"));
    }

    #[test]
    fn the_preview_port_binds_from_an_ordinary_thread() {
        // `--preview` calls this from `main`, which is not a Tokio context.
        // Binding before the runtime exists panics inside `TcpListener::from_std`
        // ("there is no reactor running"), killing the process before the server
        // ever starts — and no `#[tokio::test]` can catch it, because those
        // already supply the reactor that is missing in the real caller. Hence a
        // plain `#[test]`: the absence of a runtime here is the point.
        let (rt, listener) = create_runtime_and_bind(0).expect("bind preview port");
        assert!(
            listener.local_addr().expect("listener address").port() > 0,
            "port 0 should be replaced by an OS-assigned port"
        );
        drop(listener);
        drop(rt);
    }

    #[test]
    fn metadata_carries_the_feature_graph_in_dependency_order() {
        // The viewer lays the tree out in one pass, so it relies on parents
        // arriving before the nodes that use them. A box that is filleted and
        // then cut gives both a chain and a merge node to check.
        let body = Shape::make_box(20.0, 10.0, 5.0)
            .expect("make_box")
            .fillet(1.0)
            .expect("fillet");
        let tool = Shape::make_cylinder(2.0, 20.0)
            .expect("make_cylinder")
            .translate(10.0, 5.0, -1.0)
            .expect("translate");
        let part = body.cut(&tool).expect("cut");

        let metadata = metadata_json_for_shape(&part);
        let nodes = metadata["feature_graph"]
            .as_array()
            .expect("feature graph array");
        assert!(nodes.len() >= 5, "expected every step to appear: {nodes:?}");

        // Every parent must already have been seen by the time it is referenced.
        let mut seen = std::collections::BTreeSet::new();
        for node in nodes {
            for parent in node["parents"].as_array().expect("parents array") {
                let parent = parent.as_u64().expect("parent id");
                assert!(
                    seen.contains(&parent),
                    "parent {parent} came after its child"
                );
            }
            seen.insert(node["id"].as_u64().expect("node id"));
        }

        // The primitives are roots and the boolean merges two branches.
        assert!(
            nodes
                .iter()
                .any(|n| n["label"].as_str().unwrap_or("").starts_with("box(")
                    && n["parents"].as_array().expect("parents").is_empty()),
            "the box should be a root: {nodes:?}"
        );
        let merge = nodes
            .iter()
            .find(|n| n["label"].as_str().unwrap_or("").starts_with("cut("))
            .expect("the cut should appear in the graph");
        assert_eq!(
            merge["parents"].as_array().expect("parents").len(),
            2,
            "a boolean joins two branches: {merge:?}"
        );
        // `entry` carries the detail the short label drops — here, the operands.
        assert!(
            merge["entry"].as_str().unwrap_or("").contains("rhs="),
            "the entry should name the operands: {merge:?}"
        );
    }

    #[test]
    fn a_primitive_has_a_single_rootless_feature_node() {
        // The smallest possible graph — proof the panel has something to show
        // even before any modifier is applied.
        let metadata = metadata_json_for_shape(&Shape::make_box(1.0, 1.0, 1.0).expect("make_box"));
        let nodes = metadata["feature_graph"]
            .as_array()
            .expect("feature graph array");
        assert_eq!(nodes.len(), 1, "expected one node: {nodes:?}");
        assert!(nodes[0]["parents"].as_array().expect("parents").is_empty());
        assert!(
            nodes[0]["label"]
                .as_str()
                .expect("label")
                .starts_with("box(")
        );
    }

    /// Extract one top-level `function <name>(…) { … }` from the viewer script
    /// by brace matching, so its logic can be exercised directly under node.
    fn viewer_function(name: &str) -> String {
        let html = include_str!("viewer.html");
        let start = html
            .find(&format!("function {name}("))
            .unwrap_or_else(|| panic!("viewer.html must define {name}"));
        let open = start + html[start..].find('{').expect("function body");
        let mut depth = 0usize;
        for (offset, ch) in html[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return html[start..open + offset + 1].to_string();
                    }
                }
                _ => {}
            }
        }
        panic!("unbalanced braces in {name}");
    }

    #[test]
    fn the_feature_tree_indents_side_branches_under_the_main_chain() {
        // The panel's whole claim is that it reads like a CAD tree: the chain
        // leading to the previewed shape flush left, the tool bodies feeding a
        // boolean indented beneath it. This is the fixture that distinguishes
        // the two — a filleted box cut by a moved cylinder.
        //
        // Indenting by raw graph depth instead would push every step of a
        // linear model one level further right, which is exactly the layout
        // this function exists to avoid, so the linear case is checked too.
        let Some(node) = find_node() else { return };

        let graph = r#"[
          { id: 2, parents: [],     label: "box(...)" },
          { id: 3, parents: [2],    label: "fillet(...)" },
          { id: 5, parents: [],     label: "cylinder(...)" },
          { id: 6, parents: [5],    label: "translate(...)" },
          { id: 7, parents: [3, 6], label: "cut()" }
        ]"#;
        let linear = r#"[
          { id: 1, parents: [],  label: "box(...)" },
          { id: 2, parents: [1], label: "fillet(...)" },
          { id: 3, parents: [2], label: "chamfer(...)" }
        ]"#;

        let script = format!(
            "{}\n\
             const depths = g => {{ const d = featureDepths(g); \
                 return g.map(n => d.get(n.id)); }};\n\
             const eq = (got, want, what) => {{ \
                 const a = JSON.stringify(got), b = JSON.stringify(want); \
                 if (a !== b) {{ console.error(what + ': got ' + a + ', want ' + b); \
                                 process.exit(1); }} }};\n\
             // box, fillet flush left; the cylinder branch indented; cut on the spine.\n\
             eq(depths({graph}), [0, 0, 1, 1, 0], 'branching model');\n\
             // A model with no branches must not indent at all.\n\
             eq(depths({linear}), [0, 0, 0], 'linear model');\n\
             // An empty graph must not throw — a shape may carry no features.\n\
             eq(depths([]), [], 'empty graph');\n",
            viewer_function("featureDepths"),
        );

        let mut path = std::env::temp_dir();
        path.push(format!("rrcad-feature-depths-{}.mjs", std::process::id()));
        fs::write(&path, script).expect("write feature depth harness");
        let status = Command::new(node)
            .arg(&path)
            .status()
            .expect("run feature depth harness");
        let _ = fs::remove_file(&path);
        assert!(
            status.success(),
            "feature tree layout is wrong (see stderr)"
        );
    }

    #[test]
    fn the_feature_tree_renders_rows_markers_and_a_selection() {
        // `featureDepths` decides the layout, but `renderFeatureTree` is what
        // the user actually sees: the indent in pixels, the marker naming the
        // branch a boolean merges in, the count, and which row opens selected.
        // A stub DOM is enough to drive all of it — none of it touches Three.js.
        let Some(node) = find_node() else { return };

        let harness = format!(
            r#"
// Minimal stand-in for the handful of DOM behaviours the renderer relies on.
const makeClassList = () => {{
  const set = new Set();
  return {{
    add: c => set.add(c),
    contains: c => set.has(c),
    toggle: (c, on) => {{ on ? set.add(c) : set.delete(c); return on; }},
    all: () => [...set],
  }};
}};
const makeEl = () => {{
  const el = {{
    _text: '', className: '', dataset: {{}}, style: {{}}, title: '',
    children: [], classList: makeClassList(),
    appendChild(child) {{ this.children.push(child); return child; }},
    addEventListener() {{}},
  }};
  // Assigning textContent clears children, exactly as the real DOM does —
  // this is how the renderer empties the list between reloads.
  Object.defineProperty(el, 'textContent', {{
    get() {{ return el._text; }},
    set(v) {{ el._text = v; el.children.length = 0; }},
  }});
  return el;
}};
const document = {{ createElement: makeEl }};
const featureList = makeEl(), featureDetail = makeEl(), featureCount = makeEl();
let featureNodes = [];

{depths}
{by_id}
{detail}
{select}
{render}

const fail = m => {{ console.error(m); process.exit(1); }};
const eq = (got, want, what) => {{
  const a = JSON.stringify(got), b = JSON.stringify(want);
  if (a !== b) fail(what + ': got ' + a + ', want ' + b);
}};

renderFeatureTree([
  {{ id: 2, parents: [],     label: 'box(dx=80, dy=50, dz=30)', entry: 'box(dx=80, dy=50, dz=30)' }},
  {{ id: 3, parents: [2],    label: 'fillet(radius=4)',         entry: 'fillet(radius=4)' }},
  {{ id: 5, parents: [],     label: 'cylinder(radius=3)',       entry: 'cylinder(radius=3)' }},
  {{ id: 6, parents: [5],    label: 'translate(dx=20)',         entry: 'translate(dx=20)' }},
  {{ id: 7, parents: [3, 6], label: 'cut()',                    entry: 'cut(lhs=compound, rhs=solid)' }},
]);

const rows = featureList.children;
eq(rows.length, 5, 'row count');
eq(rows.map(r => r.dataset.id), ['2', '3', '5', '6', '7'], 'rows keep build order');
// Main chain flush left, the cylinder branch one level in.
eq(rows.map(r => r.style.paddingLeft), ['4px', '4px', '14px', '14px', '4px'], 'indent');
eq(featureCount.textContent, '5', 'count');

// Only the boolean names a merged-in branch, and it names the right one.
const markers = rows.map(r => (r.children.find(c => c.className === 'feat-merge') || {{}}).textContent);
eq(markers, [undefined, undefined, undefined, undefined, '+#6'], 'merge markers');

// The panel opens on the shape that was previewed, showing its full entry.
eq(rows.map(r => r.classList.contains('selected')), [false, false, false, false, true], 'selection');
eq(rows[4].classList.contains('result'), true, 'result row');
eq(featureDetail.textContent, '#7 cut(lhs=compound, rhs=solid) — from #3, #6', 'detail');
// The label stays short; the detail line is where the operands show up.
eq(rows[4].children[0].textContent, 'cut()', 'label');

// Selecting another row moves the highlight and the detail with it.
selectFeature(3);
eq(rows.map(r => r.classList.contains('selected')), [false, true, false, false, false], 'reselection');
eq(featureDetail.textContent, '#3 fillet(radius=4) — from #2', 'reselected detail');

// A shape with no recorded features must say so rather than render nothing.
renderFeatureTree([]);
eq(featureList.children.length, 1, 'empty placeholder row');
eq(featureList.children[0].textContent, 'no features recorded', 'empty text');
eq(featureCount.textContent, '', 'empty count');
"#,
            depths = viewer_function("featureDepths"),
            by_id = viewer_function("featureNodeById"),
            detail = viewer_function("setFeatureDetail"),
            select = viewer_function("selectFeature"),
            render = viewer_function("renderFeatureTree"),
        );

        let mut path = std::env::temp_dir();
        path.push(format!("rrcad-feature-render-{}.mjs", std::process::id()));
        fs::write(&path, harness).expect("write feature render harness");
        let status = Command::new(node)
            .arg(&path)
            .status()
            .expect("run feature render harness");
        let _ = fs::remove_file(&path);
        assert!(
            status.success(),
            "feature panel renders wrongly (see stderr)"
        );
    }

    #[test]
    fn viewer_html_rejects_stale_preview_loads() {
        let html = include_str!("viewer.html");
        let start = html
            .find(r#"<script type="module">"#)
            .expect("viewer script start");
        let end = html[start..]
            .find("</script>")
            .map(|offset| start + offset)
            .expect("viewer script end");
        let script = &html[start + r#"<script type="module">"#.len()..end];

        let mut path = std::env::temp_dir();
        path.push(format!(
            "rrcad-viewer-{}-{}.mjs",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock before unix epoch")
                .as_nanos()
        ));
        fs::write(&path, script).expect("write viewer script");

        assert!(
            script.contains("let loadGeneration = 0;"),
            "viewer must track preview load generations"
        );
        assert!(
            script.contains("if (loadToken !== loadGeneration) return;"),
            "stale preview responses must be ignored"
        );
        assert!(
            script.contains("const loadToken = ++loadGeneration;"),
            "each preview load must mint a new generation token"
        );
        assert!(
            script.contains("loadMetadata(loadToken);"),
            "metadata fetches must be tied to the same load generation"
        );

        if let Some(node) = find_node() {
            let status = Command::new(node)
                .arg("--check")
                .arg(&path)
                .status()
                .expect("run node --check");
            assert!(status.success(), "viewer script must be valid JavaScript");
        }

        let _ = fs::remove_file(&path);
    }

    fn find_node() -> Option<&'static str> {
        match Command::new("node").arg("--version").status() {
            Ok(status) if status.success() => Some("node"),
            Err(err) if err.kind() == ErrorKind::NotFound => None,
            Ok(status) => panic!("node --version failed: {status}"),
            Err(err) => panic!("failed to probe node: {err}"),
        }
    }
}
