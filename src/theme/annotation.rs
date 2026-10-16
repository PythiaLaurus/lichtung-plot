//! Complete annotation geometry, separate from text, content and placement policy.

use super::component::validated_component;
use super::spacing::ExactLength;
use super::{LocalMetric, OpaqueColor, OutlineTheme, RelativeSpacing, SolidStrokeTheme};
use crate::Result;

validated_component!(SignificanceGeometry {
    borrow bracket_stroke: SolidStrokeTheme => with_bracket_stroke,
    borrow label_gap: RelativeSpacing => with_label_gap,
    borrow tick_height: RelativeSpacing => with_tick_height,
    borrow data_gap: RelativeSpacing => with_data_gap,
    borrow level_step: RelativeSpacing => with_level_step,
    borrow panel_padding: RelativeSpacing => with_panel_padding,
    borrow attached_extra_clearance: RelativeSpacing => with_attached_extra_clearance,
} validate |value| { value.bracket_stroke().validate()?; });

validated_component!(CellAnnotationGeometry {
    borrow highlight_outline: OutlineTheme => with_highlight_outline,
    borrow value_horizontal_padding: RelativeSpacing => with_value_horizontal_padding,
    borrow value_vertical_padding: RelativeSpacing => with_value_vertical_padding,
    borrow null_row_horizontal_padding: RelativeSpacing => with_null_row_horizontal_padding,
    borrow null_row_vertical_padding: RelativeSpacing => with_null_row_vertical_padding,
} validate |value| { value.highlight_outline().validate()?; });

validated_component!(EstimateAnnotationGeometry {
    borrow panel_inset: RelativeSpacing => with_panel_inset,
    borrow guide_clearance: RelativeSpacing => with_guide_clearance,
    borrow numeric_label_gap: RelativeSpacing => with_numeric_label_gap,
} validate |_value| {});

validated_component!(ValueShelfSpacing {
    borrow padding: RelativeSpacing => with_padding,
    borrow gap: RelativeSpacing => with_gap,
    borrow row_gap: RelativeSpacing => with_row_gap,
} validate |_value| {});

pub(crate) struct ValueShelfLengths {
    pub(crate) padding: ExactLength,
    pub(crate) gap: ExactLength,
    pub(crate) row_gap: ExactLength,
}

impl ValueShelfSpacing {
    pub(crate) fn lengths(&self, em: u32) -> Result<ValueShelfLengths> {
        let em = LocalMetric::css_pixels(em)?;
        Ok(ValueShelfLengths {
            padding: self.padding().product(em),
            gap: self.gap().product(em),
            row_gap: self.row_gap().product(em),
        })
    }
}

validated_component!(AnnotationTheme {
    borrow significance: SignificanceGeometry => with_significance,
    borrow cells: CellAnnotationGeometry => with_cells,
    borrow estimates: EstimateAnnotationGeometry => with_estimates,
    borrow point_label_gap: RelativeSpacing => with_point_label_gap,
    borrow panel_text_padding: RelativeSpacing => with_panel_text_padding,
    borrow waterfall_shelf: ValueShelfSpacing => with_waterfall_shelf,
    copy planning_backdrop: OpaqueColor => with_planning_backdrop,
} validate |value| {
    value.significance().validate()?;
    value.cells().validate()?;
    value.estimates().validate()?;
    value.waterfall_shelf().validate()?;
});

impl AnnotationTheme {
    pub(crate) fn builtin(foreground: super::Color, bracket_width: f32) -> Result<Self> {
        let ratio = RelativeSpacing::ratio;
        Self::new(
            SignificanceGeometry::new(
                SolidStrokeTheme::new(foreground, bracket_width)?,
                ratio(8, 16)?,
                ratio(7, 16)?,
                ratio(6, 16)?,
                ratio(26, 16)?,
                ratio(2, 16)?,
                ratio(2, 16)?,
            )?,
            CellAnnotationGeometry::new(
                OutlineTheme::new(foreground, 2.5)?,
                ratio(6, 11)?,
                ratio(5, 11)?,
                ratio(12, 11)?,
                ratio(4, 11)?,
            )?,
            EstimateAnnotationGeometry::new(ratio(6, 14)?, ratio(6, 14)?, ratio(7, 14)?)?,
            ratio(2, 10)?,
            ratio(8, 14)?,
            ValueShelfSpacing::new(ratio(4, 14)?, ratio(4, 14)?, ratio(4, 14)?)?,
            OpaqueColor::rgb(255, 255, 255),
        )
    }

    pub(crate) fn point_label_gap_length(&self, em: u32) -> Result<ExactLength> {
        Ok(self.point_label_gap().product(LocalMetric::css_pixels(em)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waterfall_intrinsics_round_the_complete_fractional_shelf_padding_once() {
        let source = crate::Plot::waterfall(
            crate::CategoryValues::new("category", ["first", "second"]).unwrap(),
            crate::NumericValues::new("delta", [2.0, -1.0]),
            10.0,
        )
        .erase();
        let operation = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let shelf = theme
            .annotations
            .waterfall_shelf()
            .clone()
            .with_padding(RelativeSpacing::ratio(0, 1).unwrap())
            .unwrap();
        theme.annotations = theme
            .annotations
            .clone()
            .with_waterfall_shelf(shelf)
            .unwrap();
        let unpadded = chart.intrinsic_body_minimum(&theme).unwrap();
        // Each edge contributes exactly 1/4 quantum at the selected 14px role.
        // The combined 1/2 quantum reserves one quantum, rather than two.
        let shelf = theme
            .annotations
            .waterfall_shelf()
            .clone()
            .with_padding(RelativeSpacing::ratio(1, 14 * 4096).unwrap())
            .unwrap();
        theme.annotations = theme
            .annotations
            .clone()
            .with_waterfall_shelf(shelf)
            .unwrap();
        let padded = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(padded.0.to_bits(), unpadded.0.to_bits());
        assert_eq!(
            (padded.1 - unpadded.1).to_bits(),
            (1.0_f64 / 1024.0).to_bits()
        );
    }
}
