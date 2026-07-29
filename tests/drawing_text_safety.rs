// Drawing annotations must not be able to corrupt the file they are written
// into.
//
// Several annotations carry text the user chose — datum labels, feature
// control frames, parts-list cells, title-block fields. Two of those strings
// are ordinary things for an engineer to write and used to produce a broken
// file:
//
//   datum: "A&B"                → an unescaped `&` in SVG
//   feature_control: "<0.05> A" → an unescaped `<` in SVG
//   any label containing "\n"   → a shifted group-code stream in DXF
//
// Both failures are silent at export time. The SVG is written, weighs the
// right amount, and only fails when something tries to read it; the DXF stays
// syntactically plausible while every value after the break is read as the
// wrong thing. So these tests check the invariant the format actually
// requires, not that a particular string was escaped a particular way.

use rrcad::ruby::vm::MrubyVm;
use std::path::{Path, PathBuf};

/// A throwaway working directory, removed on drop. `safe_path` confines
/// exports to the process CWD, so these write into it and clean up.
struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/textsafety_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Run `script` with `OUT` substituted for the export path.
    fn run(&self, script: &str, out: &Path) {
        let literal = format!("{:?}", out.to_string_lossy());
        let mut vm = MrubyVm::new();
        vm.eval(&script.replace("OUT", &literal))
            .expect("export should succeed");
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

/// The contents of every `<text>` node in an SVG, still escaped.
fn text_nodes(path: &Path) -> Vec<String> {
    let svg = std::fs::read_to_string(path).expect("read svg");
    let mut out = Vec::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text") {
        rest = &rest[open..];
        let body = rest.find('>').expect("unterminated <text");
        let close = rest.find("</text>").expect("unclosed <text>");
        out.push(rest[body + 1..close].to_string());
        rest = &rest[close + 7..];
    }
    out
}

/// Resolve the five XML predefined entities, so a node can be compared with
/// what the user actually wrote.
fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Panic if any text node carries markup a parser would choke on: a raw angle
/// bracket, or an ampersand that does not open an entity.
fn assert_all_text_escaped(path: &Path) {
    for node in text_nodes(path) {
        assert!(
            !node.contains('<') && !node.contains('>'),
            "raw angle bracket in text node: {node:?}"
        );
        let mut rest = node.as_str();
        while let Some(i) = rest.find('&') {
            let after = &rest[i..];
            assert!(
                ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"]
                    .iter()
                    .any(|e| after.starts_with(e)),
                "bare ampersand in text node: {node:?}"
            );
            rest = &after[1..];
        }
    }
}

/// Panic unless the DXF's group-code stream stays in sync: values and codes
/// strictly alternate, so every even-indexed line must parse as an integer.
///
/// This is the property a stray newline breaks. Checking it directly catches
/// the corruption wherever it came from, rather than only the one field the
/// test happened to poison.
fn assert_dxf_stream_in_sync(path: &Path) {
    let text = std::fs::read_to_string(path).expect("read dxf");
    for (i, line) in text.lines().enumerate() {
        if i % 2 == 0 {
            assert!(
                line.trim().parse::<i32>().is_ok(),
                "line {} should be a group code, got {line:?} — the stream has shifted",
                i + 1
            );
        }
    }
}

/// A plate with every text-bearing annotation switched on at once, and user
/// text carrying both of the characters SVG cares about.
const ANNOTATED: &str = r#"
    b = box(40, 30, 10).cut(cylinder(3, 12).translate(20, 15, -1))
    b.export(OUT, view: :front, dimensions: true, callouts: true,
             title_block: true, datum: "A&B", feature_control: "<0.05> A|B")
"#;

// ---------------------------------------------------------------------------
// SVG
// ---------------------------------------------------------------------------

#[test]
fn markup_in_an_annotation_does_not_break_the_svg() {
    // The bug this exists for: `datum: "A&B"` produced a document no XML
    // parser would open, and nothing said so at export time.
    let ws = Workspace::new("svg_markup");
    let out = ws.path("plate.svg");
    ws.run(ANNOTATED, &out);
    assert_all_text_escaped(&out);
}

#[test]
fn the_escaped_text_still_says_what_the_user_wrote() {
    // The partner to the check above, not a second copy of it: escaping is
    // only correct if it round-trips. Dropping the offending characters, or
    // double-escaping them into `&amp;amp;`, would satisfy well-formedness and
    // still be wrong on the page. This one alone would pass against the
    // unescaped output, which is exactly why both are here.
    let ws = Workspace::new("svg_roundtrip");
    let out = ws.path("plate.svg");
    ws.run(ANNOTATED, &out);

    let decoded: Vec<String> = text_nodes(&out).iter().map(|n| unescape(n)).collect();
    assert!(
        decoded.iter().any(|n| n == "DATUM A&B"),
        "datum should read back exactly as written: {decoded:?}"
    );
    assert!(
        decoded.iter().any(|n| n == "<0.05> A|B"),
        "feature control should read back exactly as written: {decoded:?}"
    );
}

#[test]
fn every_text_node_is_escaped_not_just_the_annotated_ones() {
    // Dimension labels, diameter callouts, ordinates and the title block are
    // built from numbers today, so nothing user-chosen reaches them. They go
    // through the same escaper anyway — the alternative is asking every future
    // caller to work out whether its text can reach a user, which is the
    // question that produced this bug.
    let ws = Workspace::new("svg_all");
    let out = ws.path("sheet.svg");
    ws.run(
        r#"
        b = box(40, 30, 10).cut(cylinder(3, 12).translate(20, 15, -1))
        b.export(OUT, view: :sheet, dimensions: true, callouts: true,
                 ordinate: true, title_block: true, tolerance: 0.1,
                 datum: "A&B", feature_control: "<0.05>")
        "#,
        &out,
    );
    assert!(
        text_nodes(&out).len() > 5,
        "expected a well-annotated sheet"
    );
    assert_all_text_escaped(&out);
}

#[test]
fn a_parts_list_cell_cannot_inject_markup() {
    // Component names are free text and reach the drawing in bulk, which is
    // what made this path the first one to be escaped.
    let ws = Workspace::new("svg_bom");
    let out = ws.path("panel.svg");
    ws.run(
        r#"
        asm = assembly("panel") do |a|
          a.place box(60, 40, 3), name: :plate, component: :"<plate>", material: "steel & tin"
        end
        asm.export(OUT, view: :top, bom: true, balloons: true)
        "#,
        &out,
    );
    assert_all_text_escaped(&out);
    let decoded: Vec<String> = text_nodes(&out).iter().map(|n| unescape(n)).collect();
    assert!(
        decoded.iter().any(|n| n.contains("<plate>")),
        "the component name should survive escaping: {decoded:?}"
    );
}

// ---------------------------------------------------------------------------
// DXF
// ---------------------------------------------------------------------------

#[test]
fn a_newline_in_an_annotation_does_not_shift_the_dxf_stream() {
    // DXF ASCII is line-oriented: a value occupies exactly one line and the
    // next line is read as a group code. A newline inside a datum label does
    // not wrap the text, it desynchronises the rest of the file.
    let ws = Workspace::new("dxf_newline");
    let out = ws.path("plate.dxf");
    ws.run(
        r#"
        b = box(40, 30, 10)
        b.export(OUT, view: :front, title_block: true,
                 datum: "A\nB", feature_control: "0.05\r\nA")
        "#,
        &out,
    );
    assert_dxf_stream_in_sync(&out);
}

#[test]
fn a_flattened_annotation_keeps_its_words() {
    // The break becomes a space rather than being dropped — the drawing still
    // carries what the user wrote, on one line.
    let ws = Workspace::new("dxf_words");
    let out = ws.path("plate.dxf");
    ws.run(
        r#"
        b = box(40, 30, 10)
        b.export(OUT, view: :front, datum: "TOP\nLEFT")
        "#,
        &out,
    );
    let text = std::fs::read_to_string(ws.path("plate.dxf")).expect("read dxf");
    assert!(
        text.contains("DATUM TOP LEFT"),
        "the newline should flatten to a space, not drop the words"
    );
    assert_dxf_stream_in_sync(&out);
}

#[test]
fn a_clean_drawing_is_left_alone() {
    // The sanitisers must be transparent when there is nothing to fix.
    let ws = Workspace::new("dxf_clean");
    let out = ws.path("plate.dxf");
    ws.run(
        r#"
        b = box(40, 30, 10).cut(cylinder(3, 12).translate(20, 15, -1))
        b.export(OUT, view: :front, dimensions: true, callouts: true,
                 title_block: true, datum: "A", feature_control: "0.05 A")
        "#,
        &out,
    );
    assert_dxf_stream_in_sync(&out);
    let text = std::fs::read_to_string(&out).expect("read dxf");
    assert!(text.contains("DATUM A"), "plain text should pass through");
    assert!(text.contains("0.05 A"), "plain text should pass through");
}
