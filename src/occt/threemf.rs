//! The 3MF package container.
//!
//! A 3MF file is an OPC package — a ZIP with a fixed skeleton of three parts:
//!
//! ```text
//! [Content_Types].xml     what each extension in the package means
//! _rels/.rels             which part is the model, and how it is reached
//! 3D/3dmodel.model        the geometry, written by src/occt/bridge.cpp
//! ```
//!
//! Only the last one has anything to do with the shape. The other two are the
//! same bytes in every 3MF ever written, which is why this lives on the Rust
//! side: it is packaging, not geometry, and there is a ZIP library here.

use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

/// Declares what the two extensions in the package hold. A reader that cannot
/// resolve a part's content type is entitled to reject the file, so this is
/// not optional boilerplate.
const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>
"#;

/// Points the package root at the model part. Without this a reader finds the
/// model file but has no reason to believe it is the one to open.
const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>
"#;

/// The path the model part is required to live at.
const MODEL_PART: &str = "3D/3dmodel.model";

/// Write `model_xml` as a complete 3MF package at `path`.
///
/// The file appears atomically: everything is written to a sibling temp file
/// and renamed on success, matching the C++ exporters. A half-written 3MF is
/// still a readable ZIP right up to the point it isn't, so a slicer watching
/// the directory would happily open a truncated one.
pub(crate) fn write_package(path: &str, model_xml: &str) -> Result<(), String> {
    let final_path = Path::new(path);
    let temp_path = temp_sibling(final_path);

    // Scoped so the writer is flushed and the file closed before the rename.
    let result = (|| -> std::io::Result<()> {
        let file = std::fs::File::create(&temp_path)?;
        let mut zip = zip::ZipWriter::new(file);

        // Deflate: the model part is verbose XML and compresses about ten to
        // one, which is the difference between a mailable file and not.
        //
        // The timestamp is pinned to the ZIP epoch (1980-01-01) rather than
        // taken from the clock, so exporting the same shape twice produces the
        // same bytes. That makes the output diffable and the tests exact.
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::default());

        // `[Content_Types].xml` must come first: OPC readers are permitted to
        // stop looking for it after the first entry.
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(CONTENT_TYPES.as_bytes())?;

        zip.start_file("_rels/.rels", options)?;
        zip.write_all(ROOT_RELS.as_bytes())?;

        zip.start_file(MODEL_PART, options)?;
        zip.write_all(model_xml.as_bytes())?;

        zip.finish()?;
        Ok(())
    })();

    if let Err(e) = result {
        // Do not leave the temp file behind for a failure the caller will
        // report as "nothing was written".
        std::fs::remove_file(&temp_path).ok();
        return Err(format!("writing {path:?} failed: {e}"));
    }

    std::fs::rename(&temp_path, final_path).map_err(|e| {
        std::fs::remove_file(&temp_path).ok();
        format!("writing {path:?} failed: {e}")
    })
}

/// A temp path next to the target, so the rename stays within one filesystem
/// and is therefore atomic. Mirrors `atomic_export_temp_path` in bridge.cpp,
/// including the pid in the name so two processes exporting to the same
/// directory cannot collide.
fn temp_sibling(final_path: &Path) -> PathBuf {
    let stem = final_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = final_path
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    let name = format!("{stem}.rrcad-tmp.{}{ext}", std::process::id());
    match final_path.parent() {
        Some(dir) => dir.join(name),
        None => PathBuf::from(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    /// A scratch directory that removes itself, so these tests leave nothing in
    /// the working tree.
    struct Dir(PathBuf);

    impl Dir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("rrcad_3mf_{tag}_{}", std::process::id()));
            std::fs::remove_dir_all(&dir).ok();
            std::fs::create_dir_all(&dir).expect("create scratch dir");
            Self(dir)
        }

        fn path(&self, name: &str) -> String {
            self.0.join(name).to_string_lossy().into_owned()
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    fn entry(path: &str, name: &str) -> String {
        let file = std::fs::File::open(path).expect("open package");
        let mut zip = zip::ZipArchive::new(file).expect("the package should be a readable ZIP");
        let mut part = zip
            .by_name(name)
            .unwrap_or_else(|_| panic!("package should contain {name}"));
        let mut body = String::new();
        part.read_to_string(&mut body).expect("read part");
        body
    }

    #[test]
    fn the_package_carries_the_three_parts_a_reader_looks_for() {
        let dir = Dir::new("skeleton");
        let out = dir.path("cube.3mf");
        write_package(&out, "<model/>").expect("write");

        let file = std::fs::File::open(&out).expect("open");
        let zip = zip::ZipArchive::new(file).expect("readable ZIP");
        let names: Vec<&str> = zip.file_names().collect();
        for required in ["[Content_Types].xml", "_rels/.rels", MODEL_PART] {
            assert!(names.contains(&required), "missing {required} in {names:?}");
        }
        // The content-types part is the one an OPC reader may give up on if it
        // is not first, so its position is part of the contract.
        assert_eq!(zip.name_for_index(0), Some("[Content_Types].xml"));
    }

    #[test]
    fn the_model_xml_arrives_unchanged() {
        // Compression is the only thing between the writer and the reader, so
        // this is really checking that the round trip is lossless — including
        // for the non-ASCII a metadata field could carry.
        let dir = Dir::new("roundtrip");
        let out = dir.path("part.3mf");
        let xml = "<model unit=\"millimeter\">…</model>\n";
        write_package(&out, xml).expect("write");
        assert_eq!(entry(&out, MODEL_PART), xml);
    }

    #[test]
    fn the_relationship_names_the_part_that_is_actually_there() {
        // These two constants are written independently and a reader follows
        // one to find the other; a typo in either produces a package that
        // opens as a ZIP and fails as a 3MF.
        let dir = Dir::new("rels");
        let out = dir.path("part.3mf");
        write_package(&out, "<model/>").expect("write");
        let rels = entry(&out, "_rels/.rels");
        assert!(
            rels.contains(&format!("Target=\"/{MODEL_PART}\"")),
            "the root relationship should point at the model part: {rels}"
        );
    }

    #[test]
    fn exporting_the_same_model_twice_produces_the_same_bytes() {
        // The pinned timestamp is what makes this true. Without it the two
        // files differ in the entry headers alone, which makes any diff of
        // exported output useless.
        let dir = Dir::new("stable");
        let a = dir.path("a.3mf");
        let b = dir.path("b.3mf");
        write_package(&a, "<model/>").expect("write a");
        write_package(&b, "<model/>").expect("write b");
        assert_eq!(
            std::fs::read(&a).expect("read a"),
            std::fs::read(&b).expect("read b")
        );
    }

    #[test]
    fn a_failed_write_leaves_no_temp_file_behind() {
        let dir = Dir::new("cleanup");
        // A directory that does not exist: the temp file cannot be created.
        let out = dir.path("nope/part.3mf");
        assert!(write_package(&out, "<model/>").is_err());
        let leftovers: Vec<_> = std::fs::read_dir(&dir.0)
            .expect("read scratch dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect();
        assert!(leftovers.is_empty(), "left behind: {leftovers:?}");
    }
}
