//! Complete validated mark roles, independent of axes and scientific presence.

use super::component::validated_component;
use super::{
    AccentPalette, Color, DataFillTheme, DataLineTheme, DataOutlineTheme, FixedFillTheme,
    OpaqueColor, OutlineTheme, PointOutlineTheme, PointTheme, RelativeSpacing, StrokeTheme,
};
use crate::render::backend::{PointStyle, StrokePattern};
use crate::{Error, PointShape, Result};

fn positive_radius(radius: u32) -> Result<()> {
    if radius == 0 {
        Err(Error::Data(
            "point radius must be positive CSS pixels".into(),
        ))
    } else {
        Ok(())
    }
}

fn fraction(value: &RelativeSpacing, limit: &RelativeSpacing, positive: bool) -> Result<()> {
    if (positive && value.is_zero()) || value.compare(limit).is_gt() {
        Err(Error::Data(
            "mark relationship is outside its admitted fraction".into(),
        ))
    } else {
        Ok(())
    }
}

validated_component!(SizedPointTheme {
    borrow appearance: PointTheme => get_appearance,
    copy radius: u32 => get_radius,
} validate |value| {
    value.get_appearance().validate()?;
    positive_radius(value.get_radius())?;
});

impl SizedPointTheme {
    pub(crate) fn backend_style(&self, color: (u8, u8, u8), outline_visible: bool) -> PointStyle {
        self.get_appearance()
            .backend_style(color, self.get_radius(), outline_visible)
    }
}

validated_component!(SupplementalPointTheme {
    copy radius: u32 => get_radius,
    copy fill_alpha: u8 => get_fill_alpha,
} validate |value| { positive_radius(value.get_radius())?; });

validated_component!(PointRadii {
    copy scatter: u32 => get_scatter,
    copy path: u32 => get_path,
    copy trajectory: u32 => get_trajectory,
    copy regression: u32 => get_regression,
    copy qq: u32 => get_qq,
    copy marginal: u32 => get_marginal,
    copy matrix: u32 => get_matrix,
    copy interval_estimate: u32 => get_interval_estimate,
    copy dumbbell: u32 => get_dumbbell,
    copy interaction: u32 => get_interaction,
    copy beeswarm: u32 => get_beeswarm,
    copy box_flier: u32 => get_box_flier,
} validate |value| {
    for radius in [value.get_scatter(), value.get_path(), value.get_trajectory(), value.get_regression(),
        value.get_qq(), value.get_marginal(), value.get_matrix(), value.get_interval_estimate(),
        value.get_dumbbell(), value.get_interaction(), value.get_beeswarm(), value.get_box_flier()] {
        positive_radius(radius)?;
    }
});

validated_component!(PointRoles {
    borrow primary: PointTheme => get_primary,
    borrow raw_overlay: SizedPointTheme => get_raw_overlay,
    borrow mandatory_strip: SizedPointTheme => get_mandatory_strip,
    borrow radii: PointRadii => get_radii,
    borrow supplemental: SupplementalPointTheme => get_supplemental,
} validate |value| {
    value.get_primary().validate()?;
    value.get_raw_overlay().validate()?;
    value.get_mandatory_strip().validate()?;
    value.get_radii().validate()?;
    value.get_supplemental().validate()?;
});

validated_component!(BodyFillRoles {
    borrow bar_body: DataFillTheme => get_bar_body,
    borrow box_body: DataFillTheme => get_box_body,
    borrow violin_body: DataFillTheme => get_violin_body,
    borrow inner_summary: FixedFillTheme => get_inner_summary,
    borrow histogram_bin: DataFillTheme => get_histogram_bin,
    borrow waterfall_bar: DataFillTheme => get_waterfall_bar,
} validate |value| {
    for fill in [value.get_bar_body(), value.get_box_body(), value.get_violin_body(),
        value.get_histogram_bin(), value.get_waterfall_bar()] { fill.validate()?; }
    value.get_inner_summary().validate()?;
});

validated_component!(BandFillRoles {
    borrow line_band: DataFillTheme => get_line_band,
    borrow regression_band: DataFillTheme => get_regression_band,
} validate |value| {
    value.get_line_band().validate()?;
    value.get_regression_band().validate()?;
});

validated_component!(DensityFillRoles {
    borrow density_area: DataFillTheme => get_density_area,
    borrow marginal_density: DataFillTheme => get_marginal_density,
    borrow matrix_density: DataFillTheme => get_matrix_density,
} validate |value| {
    for fill in [value.get_density_area(), value.get_marginal_density(), value.get_matrix_density()] {
        fill.validate()?;
    }
});

validated_component!(FillRoles {
    borrow bodies: BodyFillRoles => get_bodies,
    borrow bands: BandFillRoles => get_bands,
    borrow densities: DensityFillRoles => get_densities,
} validate |value| {
    value.get_bodies().validate()?;
    value.get_bands().validate()?;
    value.get_densities().validate()?;
});

validated_component!(DataLineRoles {
    borrow series_line: DataLineTheme => get_series_line,
    borrow path_line: DataLineTheme => get_path_line,
    borrow density_line: DataLineTheme => get_density_line,
    borrow interaction_interval: DataLineTheme => get_interaction_interval,
    borrow interaction_connector: DataLineTheme => get_interaction_connector,
} validate |value| {
    for line in [value.get_series_line(), value.get_path_line(), value.get_density_line(),
        value.get_interaction_interval(), value.get_interaction_connector()] { line.validate()?; }
});

validated_component!(NeutralPaintRoles {
    copy path: OpaqueColor => get_path,
    copy box_flier: OpaqueColor => get_box_flier,
} validate |_value| {});

validated_component!(ReferenceStrokeRoles {
    borrow bar_zero: StrokeTheme => get_bar_zero,
    borrow waterfall_reference: StrokeTheme => get_waterfall_reference,
    borrow beeswarm_reference: StrokeTheme => get_beeswarm_reference,
    borrow point_interval_reference: StrokeTheme => get_point_interval_reference,
    borrow estimate_reference: StrokeTheme => get_estimate_reference,
    borrow qq_reference: StrokeTheme => get_qq_reference,
} validate |value| {
    for stroke in [value.get_bar_zero(), value.get_waterfall_reference(), value.get_beeswarm_reference(),
        value.get_point_interval_reference(), value.get_estimate_reference(), value.get_qq_reference()] {
        stroke.validate()?;
    }
});

validated_component!(ConnectorStrokeRoles {
    borrow waterfall_connector: StrokeTheme => get_waterfall_connector,
    borrow dumbbell_connector: StrokeTheme => get_dumbbell_connector,
} validate |value| {
    value.get_waterfall_connector().validate()?;
    value.get_dumbbell_connector().validate()?;
});

validated_component!(SummaryStrokeRoles {
    borrow uncertainty: StrokeTheme => get_uncertainty,
    borrow summary: StrokeTheme => get_summary,
    borrow estimate_interval: StrokeTheme => get_estimate_interval,
    borrow median: StrokeTheme => get_median,
} validate |value| {
    for stroke in [value.get_uncertainty(), value.get_summary(), value.get_estimate_interval(), value.get_median()] {
        stroke.validate()?;
    }
});

validated_component!(CurveStrokeRoles {
    borrow histogram_curve: StrokeTheme => get_histogram_curve,
    borrow histogram_rug: StrokeTheme => get_histogram_rug,
    borrow regression_fit: StrokeTheme => get_regression_fit,
} validate |value| {
    for stroke in [value.get_histogram_curve(), value.get_histogram_rug(), value.get_regression_fit()] {
        stroke.validate()?;
    }
});

validated_component!(StrokeRoles {
    borrow references: ReferenceStrokeRoles => get_references,
    borrow connectors: ConnectorStrokeRoles => get_connectors,
    borrow summaries: SummaryStrokeRoles => get_summaries,
    borrow curves: CurveStrokeRoles => get_curves,
} validate |value| {
    value.get_references().validate()?;
    value.get_connectors().validate()?;
    value.get_summaries().validate()?;
    value.get_curves().validate()?;
});

validated_component!(CellMarkRoles {
    copy fill_alpha: u8 => get_fill_alpha,
    optional cell_outline: OutlineTheme => get_cell_outline,
    copy missing_fill: Color => get_missing_fill,
    optional missing_row_outline: OutlineTheme => get_missing_row_outline,
    optional missing_key_outline: OutlineTheme => get_missing_key_outline,
    borrow missing_cross: StrokeTheme => get_missing_cross,
    borrow missing_cross_inset: RelativeSpacing => get_missing_cross_inset,
} validate |value| {
    for outline in [value.get_cell_outline(), value.get_missing_row_outline(), value.get_missing_key_outline()]
        .into_iter().flatten() { outline.validate()?; }
    value.get_missing_cross().validate()?;
    if value.get_missing_cross_inset().compare(&RelativeSpacing::ratio(1, 2)?).is_ge() {
        return Err(Error::Data("missing-cell cross inset must be smaller than one half".into()));
    }
});

validated_component!(CategoryMarkGeometry {
    borrow vertical_body_breadth: RelativeSpacing => get_vertical_body_breadth,
    borrow horizontal_body_breadth: RelativeSpacing => get_horizontal_body_breadth,
    borrow vertical_group_span: RelativeSpacing => get_vertical_group_span,
    borrow horizontal_group_span: RelativeSpacing => get_horizontal_group_span,
    borrow uncertainty_cap_ratio: RelativeSpacing => get_uncertainty_cap_ratio,
} validate |value| {
    let one = RelativeSpacing::ratio(1, 1)?;
    for extent in [value.get_vertical_body_breadth(), value.get_horizontal_body_breadth(),
        value.get_vertical_group_span(), value.get_horizontal_group_span()] {
        fraction(extent, &one, true)?;
    }
    fraction(value.get_uncertainty_cap_ratio(), &RelativeSpacing::ratio(1, 2)?, true)?;
});

validated_component!(MarkExtentTheme {
    borrow categories: CategoryMarkGeometry => get_categories,
    borrow waterfall_body_category_breadth: RelativeSpacing => get_waterfall_body_category_breadth,
    copy point_interval_cap_half_height: u32 => get_point_interval_cap_half_height,
    borrow sampled_rug_height: RelativeSpacing => get_sampled_rug_height,
    borrow marginal_density_fraction: RelativeSpacing => get_marginal_density_fraction,
    borrow matrix_density_fraction: RelativeSpacing => get_matrix_density_fraction,
} validate |value| {
    value.get_categories().validate()?;
    positive_radius(value.get_point_interval_cap_half_height())?;
    let one = RelativeSpacing::ratio(1, 1)?;
    for extent in [value.get_waterfall_body_category_breadth(), value.get_marginal_density_fraction(),
        value.get_matrix_density_fraction()] { fraction(extent, &one, true)?; }
});

impl CategoryMarkGeometry {
    pub(crate) fn group_span(&self, horizontal: bool) -> f64 {
        if horizontal {
            self.get_horizontal_group_span()
        } else {
            self.get_vertical_group_span()
        }
        .as_f64()
    }

    pub(crate) fn body_breadth(&self, horizontal: bool) -> f64 {
        if horizontal {
            self.get_horizontal_body_breadth()
        } else {
            self.get_vertical_body_breadth()
        }
        .as_f64()
    }
}

validated_component!(MarkTheme {
    borrow accents: AccentPalette => get_accents,
    borrow points: PointRoles => get_points,
    borrow fills: FillRoles => get_fills,
    borrow data_lines: DataLineRoles => get_data_lines,
    borrow neutral_paints: NeutralPaintRoles => get_neutral_paints,
    borrow strokes: StrokeRoles => get_strokes,
    borrow cells: CellMarkRoles => get_cells,
    borrow extents: MarkExtentTheme => get_extents,
} validate |value| {
    value.get_points().validate()?;
    value.get_fills().validate()?;
    value.get_data_lines().validate()?;
    value.get_neutral_paints().validate()?;
    value.get_strokes().validate()?;
    value.get_cells().validate()?;
    value.get_extents().validate()?;
});

impl MarkTheme {
    #[expect(
        clippy::too_many_lines,
        reason = "one construction table retains the approved complete built-in mark defaults"
    )]
    pub(crate) fn builtin(
        accents: AccentPalette,
        foreground: OpaqueColor,
        axis_width: f32,
    ) -> Result<Self> {
        let paint = Color::from(foreground);
        let stroke = |width, pattern| StrokeTheme::new(paint, width, pattern);
        let filled = |alpha, width| {
            DataFillTheme::new(
                alpha,
                Some(DataOutlineTheme::fixed(OutlineTheme::new(paint, width)?)),
            )
        };
        let point_appearance = || {
            PointTheme::new(
                209,
                PointShape::Circle,
                Some(PointOutlineTheme::publication()?),
            )
        };
        let relative = RelativeSpacing::new;
        Self::new(
            accents,
            PointRoles::new(
                point_appearance()?,
                SizedPointTheme::new(point_appearance()?, 2)?,
                SizedPointTheme::new(point_appearance()?, 2)?,
                PointRadii::new(3, 3, 4, 3, 3, 3, 2, 4, 4, 4, 3, 2)?,
                SupplementalPointTheme::new(4, 204)?,
            )?,
            FillRoles::new(
                BodyFillRoles::new(
                    filled(255, 1.0)?,
                    filled(255, 1.0)?,
                    filled(178, 1.0)?,
                    FixedFillTheme::new(
                        Color::rgb(255, 255, 255),
                        Some(OutlineTheme::new(paint, 1.0)?),
                    )?,
                    filled(209, 0.8)?,
                    filled(255, 0.8)?,
                )?,
                BandFillRoles::new(DataFillTheme::new(46, None)?, DataFillTheme::new(64, None)?)?,
                DensityFillRoles::new(
                    DataFillTheme::new(56, None)?,
                    DataFillTheme::new(115, Some(DataOutlineTheme::mapped(1.0, 255)?))?,
                    DataFillTheme::new(102, Some(DataOutlineTheme::mapped(1.0, 255)?))?,
                )?,
            )?,
            DataLineRoles::new(
                DataLineTheme::new(1.8, 255, StrokePattern::Solid)?,
                DataLineTheme::new(1.4, 255, StrokePattern::Solid)?,
                DataLineTheme::new(1.6, 255, StrokePattern::Solid)?,
                DataLineTheme::new(1.2, 255, StrokePattern::Solid)?,
                DataLineTheme::new(1.8, 255, StrokePattern::Solid)?,
            )?,
            NeutralPaintRoles::new(foreground, foreground)?,
            StrokeRoles::new(
                ReferenceStrokeRoles::new(
                    stroke(0.8, StrokePattern::Solid)?,
                    stroke(0.7, StrokePattern::MediumDash)?,
                    stroke(0.8, StrokePattern::Solid)?,
                    stroke(0.8, StrokePattern::Solid)?,
                    stroke(1.0, StrokePattern::MediumDash)?,
                    stroke(1.2, StrokePattern::Solid)?,
                )?,
                ConnectorStrokeRoles::new(
                    stroke(0.7, StrokePattern::ShortDash)?,
                    stroke(1.2, StrokePattern::Solid)?,
                )?,
                SummaryStrokeRoles::new(
                    stroke(axis_width, StrokePattern::Solid)?,
                    stroke(axis_width, StrokePattern::Solid)?,
                    stroke(1.4, StrokePattern::Solid)?,
                    stroke(1.5, StrokePattern::Solid)?,
                )?,
                CurveStrokeRoles::new(
                    stroke(1.0, StrokePattern::Solid)?,
                    stroke(0.7, StrokePattern::Solid)?,
                    stroke(1.5, StrokePattern::Solid)?,
                )?,
            )?,
            CellMarkRoles::new(
                255,
                Some(OutlineTheme::new(Color::rgb(255, 255, 255), 0.5)?),
                Color::rgb(224, 224, 224),
                Some(OutlineTheme::new(Color::rgb(255, 255, 255), 0.5)?),
                Some(OutlineTheme::new(Color::rgb(120, 120, 120), 0.8)?),
                StrokeTheme::new(Color::rgb(120, 120, 120), 1.0, StrokePattern::Solid)?,
                relative(0.16)?,
            )?,
            MarkExtentTheme::new(
                CategoryMarkGeometry::new(
                    relative(0.6)?,
                    relative(0.55)?,
                    relative(0.76)?,
                    relative(0.74)?,
                    relative(0.2)?,
                )?,
                relative(0.6)?,
                4,
                relative(0.035)?,
                relative(0.90)?,
                relative(0.85)?,
            )?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_mark_edits_validate_nested_siblings_and_fraction_bounds() {
        let original = crate::theme::Theme::publication().marks;
        let summary = original
            .get_fills()
            .get_bodies()
            .get_inner_summary()
            .clone()
            .color(Color::rgba(11, 23, 37, 53))
            .unwrap()
            .outline(None)
            .unwrap();
        let bodies = original
            .get_fills()
            .get_bodies()
            .clone()
            .inner_summary(summary)
            .unwrap();
        let fills = original.get_fills().clone().bodies(bodies).unwrap();
        let mut poisoned = original.clone();
        poisoned.points.mandatory_strip.radius = 0;
        assert!(poisoned.fills(fills).is_err());
        assert_eq!(original.get_points().get_mandatory_strip().get_radius(), 2);
        assert!(original
            .get_cells()
            .clone()
            .missing_cross_inset(RelativeSpacing::ratio(1, 2).unwrap())
            .is_err());
        assert!(original
            .get_extents()
            .get_categories()
            .clone()
            .vertical_body_breadth(RelativeSpacing::new(1.0_f64.next_up()).unwrap())
            .is_err());
        assert!(original
            .get_extents()
            .get_categories()
            .clone()
            .uncertainty_cap_ratio(RelativeSpacing::ratio(1, 2).unwrap())
            .is_ok());
        assert!(original
            .get_extents()
            .clone()
            .waterfall_body_category_breadth(RelativeSpacing::ratio(1, 1).unwrap())
            .is_ok());
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "sampled and supplied strip ownership must remain independent of raw point roles and summary alpha under the same selection"
    )]
    fn sampled_and_supplied_strips_keep_their_own_style_and_guide_when_raw_roles_change() {
        use crate::plots::categorical_distribution::Mark;
        use crate::{CategoryValues, NumericValues, Plot, RaincloudInput};

        let sampled = Plot::raincloud(RaincloudInput::samples(
            CategoryValues::new("category", ["one"; 4]).unwrap(),
            NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        ))
        .series(CategoryValues::new("hue", ["series"; 4]).unwrap())
        .hide_point_outline()
        .erase()
        .with_global_hue_levels(vec!["\u{10FFFF}".into(), "series".into()]);
        let supplied = crate::plots::categorical_distribution::extraction::supplied();
        for (source, count, outlined) in [(sampled, 4, false), (supplied, 2, true)] {
            let source = source
                .with_alpha(
                    crate::alpha::AlphaTarget::Point,
                    crate::alpha::AlphaEncoding::observations(
                        crate::alpha::AlphaValues::new(
                            "confidence",
                            (0..count).map(|index| if index % 2 == 0 { 0 } else { 230 }),
                        )
                        .unwrap(),
                    ),
                )
                .with_alpha(
                    crate::alpha::AlphaTarget::Line,
                    crate::alpha::AlphaEncoding::single(
                        crate::alpha::AlphaValues::new("summary confidence", [64]).unwrap(),
                    )
                    .unwrap(),
                )
                .with_alpha(
                    crate::alpha::AlphaTarget::Fill,
                    crate::alpha::AlphaEncoding::single(
                        crate::alpha::AlphaValues::new("body confidence", [128]).unwrap(),
                    )
                    .unwrap(),
                );
            let operation = crate::font::canonical::OperationContext::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
            let chart = inputs.attach(source).unwrap();
            let mut theme = crate::Theme::publication().resolve().unwrap();
            let appearance = theme
                .marks
                .get_points()
                .get_primary()
                .clone()
                .fill_alpha(117)
                .unwrap();
            let strip = SizedPointTheme::new(appearance, 5).unwrap();
            let points = theme
                .marks
                .get_points()
                .clone()
                .mandatory_strip(strip)
                .unwrap();
            theme.marks = theme.marks.clone().points(points).unwrap();
            let minimum = chart.intrinsic_body_minimum(&theme).unwrap();
            let panel = crate::layout::PanelBox::new(10.0, 610.0, 20.0, 420.0);
            let measure = |theme: &super::super::ResolvedTheme| {
                let chart = crate::plots::shared_x::prepare_panel_for_test(&chart, theme).unwrap();
                chart
                    .measure_selected_interior(crate::spec::PlotRenderContext::new(
                        chart.data(),
                        panel,
                        theme,
                    ))
                    .unwrap()
            };
            let before = measure(&theme);
            let raw = theme
                .marks
                .get_points()
                .get_raw_overlay()
                .clone()
                .radius(80)
                .unwrap();
            let points = theme.marks.get_points().clone().raw_overlay(raw).unwrap();
            theme.marks = theme.marks.clone().points(points).unwrap();
            assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), minimum);
            let after = measure(&theme);
            assert!(before.discrete_guide.is_some());
            assert_eq!(before.discrete_guide.as_ref().unwrap().counts.entries, 1);
            assert_eq!(before.discrete_guide, after.discrete_guide);
            let crate::layout::interior::MarkPayload::Distribution(payload) =
                &after.marks.as_ref().unwrap().payload
            else {
                panic!("distribution payload")
            };
            let mut observed = 0;
            let mut alphas = Vec::new();
            for mark in &payload.marks {
                if let Mark::Scatter(points, style) = mark {
                    observed += points.len();
                    assert_eq!(style.radius, 5);
                    alphas.extend(std::iter::repeat_n(style.color.get_alpha(), points.len()));
                    assert_eq!(style.outline.is_some(), outlined);
                }
            }
            assert_eq!(observed, count);
            assert_eq!(
                payload
                    .marks
                    .iter()
                    .filter_map(|mark| match mark {
                        Mark::Polygon(_, style) => Some(style.color.get_alpha()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                [
                    128,
                    theme
                        .marks
                        .get_fills()
                        .get_bodies()
                        .get_inner_summary()
                        .backend_style()
                        .color
                        .get_alpha()
                ]
            );
            assert_eq!(
                payload
                    .marks
                    .iter()
                    .filter_map(|mark| match mark {
                        Mark::Line(_, style) => Some(style.color.get_alpha()),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                [64]
            );
            assert_eq!(
                alphas,
                (0..count)
                    .map(|index| if index % 2 == 0 { 0 } else { 230 })
                    .collect::<Vec<_>>()
            );
        }
    }
}
