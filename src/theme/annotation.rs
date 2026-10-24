//! Complete annotation geometry, separate from text, content and placement policy.

use super::component::validated_component;
use super::spacing::ExactLength;
use super::{
    LocalMetric, OpaqueColor, OutlineTheme, RelativeSpacing, ResolvedTheme, SolidStrokeTheme,
    TextTheme,
};
use crate::layout::kernel::SpatialRounding;
use crate::Result;

/// One selected significance appearance, including exact-override provenance.
pub(crate) struct EffectiveSignificance {
    pub(crate) label: TextTheme,
    pub(crate) exact_override: bool,
    pub(crate) gap: ExactLength,
    geometry: SignificanceGeometry,
}

impl ResolvedTheme {
    pub(crate) fn effective_significance(
        &self,
        exact: Option<crate::SignificanceMarkStyle>,
    ) -> Result<EffectiveSignificance> {
        let mut label = self
            .typography
            .get_annotations()
            .get_significance_label()
            .try_copy()?;
        if let Some(exact) = exact {
            label = label
                .font_size(exact.get_font_size())?
                .weight(exact.get_weight())?;
        }
        let geometry = self.annotations.get_significance();
        let em = LocalMetric::css_pixels(label.get_font_size())?;
        Ok(EffectiveSignificance {
            gap: exact.map_or_else(
                || Ok(geometry.get_label_gap().product(em)),
                |exact| ExactLength::pixels(exact.get_gap()),
            )?,
            geometry: geometry.clone(),
            label,
            exact_override: exact.is_some(),
        })
    }
}

impl EffectiveSignificance {
    fn length(&self, spacing: &RelativeSpacing) -> Result<ExactLength> {
        Ok(spacing.product(LocalMetric::css_pixels(self.label.get_font_size())?))
    }

    pub(crate) fn tick_height(&self) -> Result<ExactLength> {
        self.length(self.geometry.get_tick_height())
    }

    pub(crate) fn data_gap(&self) -> Result<ExactLength> {
        self.length(self.geometry.get_data_gap())
    }

    pub(crate) fn level_step(&self) -> Result<ExactLength> {
        self.length(self.geometry.get_level_step())
    }

    pub(crate) fn panel_padding(&self) -> Result<ExactLength> {
        let selected = self.length(self.geometry.get_panel_padding())?;
        let ink =
            ExactLength::pixels(f64::from(self.geometry.get_bracket_stroke().get_width()) / 2.0)?
                .add_pixels(1.0 / 1024.0)?;
        Ok(if selected.compare(&ink).is_lt() {
            ink
        } else {
            selected
        })
    }

    pub(crate) fn gap_pixels(&self) -> Result<f64> {
        self.gap.pixels_rounded(SpatialRounding::NearestEven)
    }

    pub(crate) fn attached_reserve(&self, extent: f64) -> Result<f64> {
        self.gap
            .add_length(&self.length(self.geometry.get_attached_extra_clearance())?)
            .add_pixels(extent)?
            .pixels_rounded(SpatialRounding::Ceil)
    }

    pub(crate) fn bracket_style(
        &self,
        label: &crate::layout::ResolvedTextPlan,
    ) -> Result<crate::render::backend::BracketStyle> {
        Ok(crate::render::backend::BracketStyle {
            color: self.geometry.get_bracket_stroke().get_color(),
            width: self.geometry.get_bracket_stroke().get_width(),
            label_color: label.style.paint,
            font_family: label
                .copy_stack()
                .map_err(|error| crate::Error::Render(error.to_string()))?,
            font_size: label.style.font_size,
            line_height: label.line_height(),
            weight: label.style.weight,
            label_gap: self.gap_pixels()?,
            tick_height: self
                .tick_height()?
                .pixels_rounded(SpatialRounding::NearestEven)?,
        })
    }
}

validated_component!(SignificanceGeometry {
    borrow bracket_stroke: SolidStrokeTheme => get_bracket_stroke,
    borrow label_gap: RelativeSpacing => get_label_gap,
    borrow tick_height: RelativeSpacing => get_tick_height,
    borrow data_gap: RelativeSpacing => get_data_gap,
    borrow level_step: RelativeSpacing => get_level_step,
    borrow panel_padding: RelativeSpacing => get_panel_padding,
    borrow attached_extra_clearance: RelativeSpacing => get_attached_extra_clearance,
} validate |value| { value.get_bracket_stroke().validate()?; });

validated_component!(CellAnnotationGeometry {
    borrow highlight_outline: OutlineTheme => get_highlight_outline,
    borrow value_horizontal_padding: RelativeSpacing => get_value_horizontal_padding,
    borrow value_vertical_padding: RelativeSpacing => get_value_vertical_padding,
    borrow null_row_horizontal_padding: RelativeSpacing => get_null_row_horizontal_padding,
    borrow null_row_vertical_padding: RelativeSpacing => get_null_row_vertical_padding,
} validate |value| { value.get_highlight_outline().validate()?; });

pub(crate) struct CellTextPadding {
    horizontal: ExactLength,
    vertical: ExactLength,
}

impl CellAnnotationGeometry {
    pub(crate) fn value_padding(&self, em: f64) -> Result<CellTextPadding> {
        let em = LocalMetric::css_pixels(em)?;
        Ok(CellTextPadding {
            horizontal: self.get_value_horizontal_padding().product(em),
            vertical: self.get_value_vertical_padding().product(em),
        })
    }

    pub(crate) fn null_row_padding(&self, em: f64) -> Result<CellTextPadding> {
        let em = LocalMetric::css_pixels(em)?;
        Ok(CellTextPadding {
            horizontal: self.get_null_row_horizontal_padding().product(em),
            vertical: self.get_null_row_vertical_padding().product(em),
        })
    }
}

impl CellTextPadding {
    pub(crate) fn width(&self, text_width: f64) -> Result<f64> {
        self.horizontal
            .add_pixels(text_width)?
            .pixels_rounded(SpatialRounding::Ceil)
    }

    pub(crate) fn height(&self, text_height: f64) -> Result<f64> {
        self.vertical
            .add_pixels(text_height)?
            .pixels_rounded(SpatialRounding::Ceil)
    }

    pub(crate) fn available_width(&self, assigned: f64) -> Result<f64> {
        ExactLength::pixels(assigned)?
            .subtract(&self.horizontal)
            .map_or(Ok(0.0), |remaining| {
                remaining.pixels_rounded(SpatialRounding::Floor)
            })
    }
}

validated_component!(EstimateAnnotationGeometry {
    borrow panel_inset: RelativeSpacing => get_panel_inset,
    borrow guide_clearance: RelativeSpacing => get_guide_clearance,
    borrow numeric_label_gap: RelativeSpacing => get_numeric_label_gap,
} validate |_value| {});

pub(crate) struct EstimateAnnotationLengths {
    pub(crate) panel_inset: ExactLength,
    pub(crate) guide_clearance: ExactLength,
}

impl EstimateAnnotationGeometry {
    pub(crate) fn lengths(&self, em: f64) -> Result<EstimateAnnotationLengths> {
        let em = LocalMetric::css_pixels(em)?;
        Ok(EstimateAnnotationLengths {
            panel_inset: self.get_panel_inset().product(em),
            guide_clearance: self.get_guide_clearance().product(em),
        })
    }
}

validated_component!(ValueShelfSpacing {
    borrow padding: RelativeSpacing => get_padding,
    borrow gap: RelativeSpacing => get_gap,
    borrow row_gap: RelativeSpacing => get_row_gap,
} validate |_value| {});

pub(crate) struct ValueShelfLengths {
    pub(crate) padding: ExactLength,
    pub(crate) gap: ExactLength,
    pub(crate) row_gap: ExactLength,
}

impl ValueShelfSpacing {
    pub(crate) fn lengths(&self, em: f64) -> Result<ValueShelfLengths> {
        let em = LocalMetric::css_pixels(em)?;
        Ok(ValueShelfLengths {
            padding: self.get_padding().product(em),
            gap: self.get_gap().product(em),
            row_gap: self.get_row_gap().product(em),
        })
    }
}

validated_component!(AnnotationTheme {
    borrow significance: SignificanceGeometry => get_significance,
    borrow cells: CellAnnotationGeometry => get_cells,
    borrow estimates: EstimateAnnotationGeometry => get_estimates,
    borrow point_label_gap: RelativeSpacing => get_point_label_gap,
    borrow panel_text_padding: RelativeSpacing => get_panel_text_padding,
    borrow waterfall_shelf: ValueShelfSpacing => get_waterfall_shelf,
    copy planning_backdrop: OpaqueColor => get_planning_backdrop,
} validate |value| {
    value.get_significance().validate()?;
    value.get_cells().validate()?;
    value.get_estimates().validate()?;
    value.get_waterfall_shelf().validate()?;
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

    pub(crate) fn point_label_gap_length(&self, em: f64) -> Result<ExactLength> {
        Ok(self
            .get_point_label_gap()
            .product(LocalMetric::css_pixels(em)?))
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
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let shelf = theme
            .annotations
            .get_waterfall_shelf()
            .clone()
            .padding(RelativeSpacing::ratio(0, 1).unwrap())
            .unwrap();
        theme.annotations = theme.annotations.clone().waterfall_shelf(shelf).unwrap();
        let unpadded = chart.intrinsic_body_minimum(&theme).unwrap();
        // Each edge contributes exactly 1/4 quantum at the selected 14px role.
        // The combined 1/2 quantum reserves one quantum, rather than two.
        let shelf = theme
            .annotations
            .get_waterfall_shelf()
            .clone()
            .padding(RelativeSpacing::ratio(1, 14 * 4096).unwrap())
            .unwrap();
        theme.annotations = theme.annotations.clone().waterfall_shelf(shelf).unwrap();
        let padded = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(padded.0.to_bits(), unpadded.0.to_bits());
        assert_eq!(
            (padded.1 - unpadded.1).to_bits(),
            (1.0_f64 / 1024.0).to_bits()
        );
    }
}
