use super::{Shape, ffi};
use crate::occt::shape_core::summarize;

impl Shape {
    /// Export to SVG using hidden-line removal (HLRBRep_PolyAlgo).
    /// `view` is `"top"` (default), `"front"`, or `"side"`.
    /// `scale` multiplies drawing geometry; `1.0` preserves model units.
    /// `hidden` includes hidden HLR edges as dashed secondary geometry.
    /// `center_marks` adds crosshair marks for cylindrical faces aligned to the view axis.
    /// `dimensions` adds overall width and height annotations.
    /// `callouts` adds diameter callouts for cylindrical faces aligned to the view axis.
    /// `datum` and `feature_control` add a simple framed GD&T annotation block.
    /// `section_plane` (`""` / `"xy"` / `"xz"` / `"yz"`) turns the drawing into a
    /// section view cut at `section_offset` along that plane's normal; the cut
    /// faces are drawn with 45 degree hatching.
    #[allow(clippy::too_many_arguments)] // mirrors the flat scalar parameter list of the C++ FFI export
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
        self.export_svg_with_anchor(
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            datum,
            false,
            0.0,
            0.0,
            0.0,
            feature_control,
            false,
            0.0,
            0.0,
            0.0,
            tolerance_plus,
            tolerance_minus,
            "",
            0.0,
        )
    }

    #[allow(clippy::too_many_arguments)] // mirrors the flat scalar parameter list of the C++ FFI export
    pub(crate) fn export_svg_with_anchor(
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
        datum_anchor_valid: bool,
        datum_anchor_x: f64,
        datum_anchor_y: f64,
        datum_anchor_z: f64,
        feature_control: &str,
        feature_control_anchor_valid: bool,
        feature_control_anchor_x: f64,
        feature_control_anchor_y: f64,
        feature_control_anchor_z: f64,
        tolerance_plus: f64,
        tolerance_minus: f64,
        section_plane: &str,
        section_offset: f64,
    ) -> Result<(), String> {
        let (
            datum,
            datum_anchor_valid,
            datum_anchor,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor,
        ) = self.gdt_export_inputs(
            datum,
            datum_anchor_valid,
            datum_anchor_x,
            datum_anchor_y,
            datum_anchor_z,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
        );
        ffi::export_svg(
            &self.inner,
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            &datum,
            datum_anchor_valid,
            datum_anchor[0],
            datum_anchor[1],
            datum_anchor[2],
            &feature_control,
            feature_control_anchor_valid,
            feature_control_anchor[0],
            feature_control_anchor[1],
            feature_control_anchor[2],
            tolerance_plus,
            tolerance_minus,
            section_plane,
            section_offset,
        )
        .map_err(|e| {
            self.fail_with_debug(
                format!(
                    "export_svg({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}, section_plane: {section_plane:?}, section_offset: {section_offset}) on {} failed: {e}",
                    summarize(self)
                ),
                "export_svg",
                &[("input", self)],
            )
        })
    }

    /// Export to DXF R12 using hidden-line removal (HLRBRep_PolyAlgo).
    /// `view` is `"top"` (default), `"front"`, or `"side"`.
    /// `scale` multiplies drawing geometry; `1.0` preserves model units.
    /// `hidden` includes hidden HLR edges on a `HIDDEN` layer.
    /// `center_marks` adds crosshair marks on a `CENTER` layer.
    /// `dimensions` adds overall width/height labels.
    /// `callouts` adds diameter callouts on a `CALLOUT` layer.
    /// `datum` and `feature_control` add a simple framed GD&T annotation block.
    /// `section_plane` (`""` / `"xy"` / `"xz"` / `"yz"`) turns the drawing into a
    /// section view cut at `section_offset` along that plane's normal; the cut
    /// faces are drawn with 45 degree hatching.
    #[allow(clippy::too_many_arguments)] // mirrors the flat scalar parameter list of the C++ FFI export
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
        self.export_dxf_with_anchor(
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            datum,
            false,
            0.0,
            0.0,
            0.0,
            feature_control,
            false,
            0.0,
            0.0,
            0.0,
            tolerance_plus,
            tolerance_minus,
            "",
            0.0,
        )
    }

    #[allow(clippy::too_many_arguments)] // mirrors the flat scalar parameter list of the C++ FFI export
    pub(crate) fn export_dxf_with_anchor(
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
        datum_anchor_valid: bool,
        datum_anchor_x: f64,
        datum_anchor_y: f64,
        datum_anchor_z: f64,
        feature_control: &str,
        feature_control_anchor_valid: bool,
        feature_control_anchor_x: f64,
        feature_control_anchor_y: f64,
        feature_control_anchor_z: f64,
        tolerance_plus: f64,
        tolerance_minus: f64,
        section_plane: &str,
        section_offset: f64,
    ) -> Result<(), String> {
        let (
            datum,
            datum_anchor_valid,
            datum_anchor,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor,
        ) = self.gdt_export_inputs(
            datum,
            datum_anchor_valid,
            datum_anchor_x,
            datum_anchor_y,
            datum_anchor_z,
            feature_control,
            feature_control_anchor_valid,
            feature_control_anchor_x,
            feature_control_anchor_y,
            feature_control_anchor_z,
        );
        ffi::export_dxf(
            &self.inner,
            path,
            view,
            scale,
            hidden,
            center_marks,
            dimensions,
            title_block,
            callouts,
            &datum,
            datum_anchor_valid,
            datum_anchor[0],
            datum_anchor[1],
            datum_anchor[2],
            &feature_control,
            feature_control_anchor_valid,
            feature_control_anchor[0],
            feature_control_anchor[1],
            feature_control_anchor[2],
            tolerance_plus,
            tolerance_minus,
            section_plane,
            section_offset,
        )
        .map_err(|e| {
            self.fail_with_debug(
                format!(
                    "export_dxf({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}, section_plane: {section_plane:?}, section_offset: {section_offset}) on {} failed: {e}",
                    summarize(self)
                ),
                "export_dxf",
                &[("input", self)],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::occt::Shape;
    use crate::test_util::unique_test_dir;
    use std::fs;

    /// Export `shape` as an SVG section view and return the file contents.
    fn section_svg(shape: &Shape, name: &str, plane: &str, offset: f64) -> Result<String, String> {
        let dir = unique_test_dir(name);
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("section.svg");
        let path = path.to_string_lossy().into_owned();
        shape.export_svg_with_anchor(
            &path, "top", 1.0, false, false, false, false, false, "", false, 0.0, 0.0, 0.0, "",
            false, 0.0, 0.0, 0.0, 0.0, 0.0, plane, offset,
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
