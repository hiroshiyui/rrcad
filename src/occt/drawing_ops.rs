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
    #[allow(clippy::too_many_arguments)]
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
        )
    }

    #[allow(clippy::too_many_arguments)]
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
        )
        .map_err(|e| {
            self.fail_with_debug(
                format!(
                    "export_svg({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}) on {} failed: {e}",
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
    #[allow(clippy::too_many_arguments)]
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
        )
    }

    #[allow(clippy::too_many_arguments)]
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
        )
        .map_err(|e| {
            self.fail_with_debug(
                format!(
                    "export_dxf({path:?}, view: {view:?}, scale: {scale}, hidden: {hidden}, center_marks: {center_marks}, dimensions: {dimensions}, title_block: {title_block}, callouts: {callouts}, datum: {datum:?}, datum_anchor_valid: {datum_anchor_valid}, feature_control: {feature_control:?}, feature_control_anchor_valid: {feature_control_anchor_valid}, tolerance_plus: {tolerance_plus}, tolerance_minus: {tolerance_minus}) on {} failed: {e}",
                    summarize(self)
                ),
                "export_dxf",
                &[("input", self)],
            )
        })
    }
}
