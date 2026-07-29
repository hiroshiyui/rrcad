use super::{Shape, ffi};
use crate::occt::shape_core::summarize;
use crate::occt::{DrawingAnchor, DrawingSpec};

/// Which of the two 2-D drawing formats an export targets.
///
/// The SVG and DXF writers take an identical request and differ only in what
/// they emit, so the format is a parameter rather than two parallel entry
/// points that have to be kept in step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingFormat {
    Svg,
    Dxf,
}

impl DrawingFormat {
    /// The name this format reports itself by in errors.
    fn fn_name(self) -> &'static str {
        match self {
            DrawingFormat::Svg => "export_svg",
            DrawingFormat::Dxf => "export_dxf",
        }
    }
}

impl Shape {
    /// Write a 2-D drawing of this shape, using hidden-line removal
    /// (`HLRBRep_PolyAlgo`) to project its visible edges.
    ///
    /// The whole request arrives as a [`DrawingSpec`], the shared struct
    /// generated for both sides of the C++ boundary from one declaration in
    /// `mod.rs`. It used to be some thirty scalars repeated through four
    /// layers that had to stay in lockstep by hand.
    ///
    /// A `gdt()` spec stored on the shape overrides the datum and
    /// feature-control fields, so a part carries its own tolerancing wherever
    /// it is drawn from.
    pub(crate) fn export_drawing(
        &self,
        spec: &DrawingSpec,
        format: DrawingFormat,
    ) -> Result<(), String> {
        let (
            datum,
            datum_anchor_valid,
            datum_anchor,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor,
        ) = self.gdt_export_inputs(
            &spec.datum,
            spec.datum_anchor.valid,
            spec.datum_anchor.x,
            spec.datum_anchor.y,
            spec.datum_anchor.z,
            &spec.feature_control,
            spec.feature_control_anchor.valid,
            spec.feature_control_anchor.x,
            spec.feature_control_anchor.y,
            spec.feature_control_anchor.z,
        );
        let resolved = DrawingSpec {
            datum: datum.clone(),
            datum_anchor: DrawingAnchor {
                valid: datum_anchor_valid,
                x: datum_anchor[0],
                y: datum_anchor[1],
                z: datum_anchor[2],
            },
            feature_control: feature_control.clone(),
            feature_control_anchor: DrawingAnchor {
                valid: feature_control_anchor_valid,
                x: feature_control_anchor[0],
                y: feature_control_anchor[1],
                z: feature_control_anchor[2],
            },
            ..spec.clone()
        };

        let result = match format {
            DrawingFormat::Svg => ffi::export_svg(&self.inner, &resolved),
            DrawingFormat::Dxf => ffi::export_dxf(&self.inner, &resolved),
        };
        result.map_err(|e| {
            let name = format.fn_name();
            self.fail_with_debug(
                format!(
                    "{name}({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}, section_plane: {section_plane:?}, section_offset: {section_offset}) on {} failed: {e}",
                    summarize(self),
                    path = resolved.path,
                    view = resolved.view,
                    scale = resolved.scale,
                    hidden = resolved.hidden,
                    center_marks = resolved.center_marks,
                    dimensions = resolved.dimensions,
                    title_block = resolved.title_block,
                    callouts = resolved.callouts,
                    tolerance_plus = resolved.tolerance_plus,
                    tolerance_minus = resolved.tolerance_minus,
                    section_plane = resolved.section_plane,
                    section_offset = resolved.section_offset,
                ),
                name,
                &[("input", self)],
            )
        })
    }

    /// Export to SVG using hidden-line removal.
    ///
    /// The plain-parameter form, for the common drawing with no section,
    /// detail, ordinates, or assembly annotations. `view` is `"top"`
    /// (default), `"front"`, `"side"`, or `"sheet"`; `scale` multiplies
    /// drawing geometry, with `1.0` preserving model units.
    #[allow(clippy::too_many_arguments)] // the public shorthand; the full request is DrawingSpec
    pub fn export_svg(
        &self,
        path: &str,
        view: &str,
        scale: f64,
        hidden: bool,
        center_marks: bool,
        dimensions: bool,
        title_block: bool,
        callouts: bool,
        datum: &str,
        feature_control: &str,
        tolerance_plus: f64,
        tolerance_minus: f64,
    ) -> Result<(), String> {
        self.export_drawing(
            &DrawingSpec {
                path: path.to_owned(),
                view: view.to_owned(),
                scale,
                hidden,
                center_marks,
                dimensions,
                title_block,
                callouts,
                datum: datum.to_owned(),
                feature_control: feature_control.to_owned(),
                tolerance_plus,
                tolerance_minus,
                ..DrawingSpec::default()
            },
            DrawingFormat::Svg,
        )
    }

    /// Export to DXF R12 ASCII. The DXF counterpart of [`Shape::export_svg`],
    /// taking the same arguments and writing Y-up CAD coordinates.
    #[allow(clippy::too_many_arguments)] // the public shorthand; the full request is DrawingSpec
    pub fn export_dxf(
        &self,
        path: &str,
        view: &str,
        scale: f64,
        hidden: bool,
        center_marks: bool,
        dimensions: bool,
        title_block: bool,
        callouts: bool,
        datum: &str,
        feature_control: &str,
        tolerance_plus: f64,
        tolerance_minus: f64,
    ) -> Result<(), String> {
        self.export_drawing(
            &DrawingSpec {
                path: path.to_owned(),
                view: view.to_owned(),
                scale,
                hidden,
                center_marks,
                dimensions,
                title_block,
                callouts,
                datum: datum.to_owned(),
                feature_control: feature_control.to_owned(),
                tolerance_plus,
                tolerance_minus,
                ..DrawingSpec::default()
            },
            DrawingFormat::Dxf,
        )
    }
}

impl Shape {
    /// Export the closed loops of one planar face as a 1:1 cut file.
    ///
    /// This is a different deliverable from [`Shape::export_svg`] /
    /// [`Shape::export_dxf`], which draw an HLR *projection* of a 3-D shape —
    /// a drawing, carrying whatever else happens to be visible from that
    /// direction. A cut file carries only this face's outer profile and its
    /// holes, at true size, which is what a laser or CNC shop consumes.
    ///
    /// The receiver must be a planar `Face`, or a shape containing exactly one
    /// face. Circular edges become true `CIRCLE` / `ARC` entities rather than
    /// chord approximations, so a bolt hole stays a hole; only free-form
    /// curves are approximated, bounded by `deflection` (mm of chord error).
    ///
    /// The outline is shifted so its bounding box starts at the origin, ready
    /// to nest on a sheet. `format` is `"dxf"` or `"svg"`.
    pub fn export_face_outline(
        &self,
        path: &str,
        format: &str,
        deflection: f64,
    ) -> Result<(), String> {
        ffi::export_face_outline(&self.inner, path, format, deflection)
            .map_err(|e| format!("export_outline({path:?}) failed: {e} [{}]", summarize(self)))
    }
}

#[cfg(test)]
mod tests {
    use super::{DrawingFormat, DrawingSpec};
    use crate::occt::Shape;
    use crate::test_util::unique_test_dir;
    use std::fs;

    /// Export `shape` as an SVG section view and return the file contents.
    fn section_svg(shape: &Shape, name: &str, plane: &str, offset: f64) -> Result<String, String> {
        let dir = unique_test_dir(name);
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("section.svg");
        let path = path.to_string_lossy().into_owned();
        shape.export_drawing(
            &DrawingSpec {
                path: path.clone(),
                view: "top".to_owned(),
                scale: 1.0,
                section_plane: plane.to_owned(),
                section_offset: offset,
                ..DrawingSpec::default()
            },
            DrawingFormat::Svg,
        )?;
        Ok(fs::read_to_string(&path).expect("read section SVG"))
    }

    #[test]
    fn section_view_hatches_the_cut_face() {
        let cube = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        let svg = section_svg(&cube, "rrcad-drawing-section", "xy", 15.0).expect("section export");
        assert!(svg.contains("hatch\""), "missing hatch group: {svg:.400}");
        // A 10x20 rectangle at 2.5 mm hatch spacing yields many hatch lines.
        assert!(
            svg.matches("<line").count() > 4,
            "expected several hatch lines, got {}",
            svg.matches("<line").count()
        );
        assert!(svg.contains("section\""), "missing cut-outline group");
    }

    #[test]
    fn section_without_a_plane_produces_a_plain_projection() {
        let cube = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        let svg = section_svg(&cube, "rrcad-drawing-plain", "", 0.0).expect("plain export");
        assert!(!svg.contains("hatch\""), "plain view must not be hatched");
    }

    #[test]
    fn section_of_a_non_solid_is_rejected() {
        let cube = Shape::make_box(10.0, 20.0, 30.0).expect("make_box");
        // `slice` returns a compound of edges — there is no material to cut.
        let edges = cube.slice("xy", 15.0).expect("slice");
        let err = section_svg(&edges, "rrcad-drawing-nonsolid", "xy", 5.0)
            .expect_err("a non-solid section must fail");
        assert!(
            err.contains("requires a solid shape"),
            "expected a non-solid error, got: {err}"
        );
    }
}
