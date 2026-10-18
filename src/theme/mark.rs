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
    borrow appearance: PointTheme => with_appearance,
    copy radius: u32 => with_radius,
} validate |value| {
    value.appearance().validate()?;
    positive_radius(value.radius())?;
});

impl SizedPointTheme {
    pub(crate) fn backend_style(&self, color: (u8, u8, u8), outline_visible: bool) -> PointStyle {
        self.appearance()
            .backend_style(color, self.radius(), outline_visible)
    }
}

validated_component!(SupplementalPointTheme {
    copy radius: u32 => with_radius,
    copy fill_alpha: u8 => with_fill_alpha,
} validate |value| { positive_radius(value.radius())?; });

validated_component!(PointRadii {
    copy scatter: u32 => with_scatter,
    copy path: u32 => with_path,
    copy trajectory: u32 => with_trajectory,
    copy regression: u32 => with_regression,
    copy qq: u32 => with_qq,
    copy marginal: u32 => with_marginal,
    copy matrix: u32 => with_matrix,
    copy interval_estimate: u32 => with_interval_estimate,
    copy dumbbell: u32 => with_dumbbell,
    copy interaction: u32 => with_interaction,
    copy beeswarm: u32 => with_beeswarm,
    copy box_flier: u32 => with_box_flier,
} validate |value| {
    for radius in [value.scatter(), value.path(), value.trajectory(), value.regression(),
        value.qq(), value.marginal(), value.matrix(), value.interval_estimate(),
        value.dumbbell(), value.interaction(), value.beeswarm(), value.box_flier()] {
        positive_radius(radius)?;
    }
});

validated_component!(PointRoles {
    borrow primary: PointTheme => with_primary,
    borrow raw_overlay: SizedPointTheme => with_raw_overlay,
    borrow mandatory_strip: SizedPointTheme => with_mandatory_strip,
    borrow radii: PointRadii => with_radii,
    borrow supplemental: SupplementalPointTheme => with_supplemental,
} validate |value| {
    value.primary().validate()?;
    value.raw_overlay().validate()?;
    value.mandatory_strip().validate()?;
    value.radii().validate()?;
    value.supplemental().validate()?;
});

validated_component!(BodyFillRoles {
    borrow bar_body: DataFillTheme => with_bar_body,
    borrow box_body: DataFillTheme => with_box_body,
    borrow violin_body: DataFillTheme => with_violin_body,
    borrow inner_summary: FixedFillTheme => with_inner_summary,
    borrow histogram_bin: DataFillTheme => with_histogram_bin,
    borrow waterfall_bar: DataFillTheme => with_waterfall_bar,
} validate |value| {
    for fill in [value.bar_body(), value.box_body(), value.violin_body(),
        value.histogram_bin(), value.waterfall_bar()] { fill.validate()?; }
    value.inner_summary().validate()?;
});

validated_component!(BandFillRoles {
    borrow line_band: DataFillTheme => with_line_band,
    borrow regression_band: DataFillTheme => with_regression_band,
} validate |value| {
    value.line_band().validate()?;
    value.regression_band().validate()?;
});

validated_component!(DensityFillRoles {
    borrow density_area: DataFillTheme => with_density_area,
    borrow marginal_density: DataFillTheme => with_marginal_density,
    borrow matrix_density: DataFillTheme => with_matrix_density,
} validate |value| {
    for fill in [value.density_area(), value.marginal_density(), value.matrix_density()] {
        fill.validate()?;
    }
});

validated_component!(FillRoles {
    borrow bodies: BodyFillRoles => with_bodies,
    borrow bands: BandFillRoles => with_bands,
    borrow densities: DensityFillRoles => with_densities,
} validate |value| {
    value.bodies().validate()?;
    value.bands().validate()?;
    value.densities().validate()?;
});

validated_component!(DataLineRoles {
    borrow series_line: DataLineTheme => with_series_line,
    borrow path_line: DataLineTheme => with_path_line,
    borrow density_line: DataLineTheme => with_density_line,
    borrow interaction_interval: DataLineTheme => with_interaction_interval,
    borrow interaction_connector: DataLineTheme => with_interaction_connector,
} validate |value| {
    for line in [value.series_line(), value.path_line(), value.density_line(),
        value.interaction_interval(), value.interaction_connector()] { line.validate()?; }
});

validated_component!(NeutralPaintRoles {
    copy path: OpaqueColor => with_path,
    copy box_flier: OpaqueColor => with_box_flier,
} validate |_value| {});

validated_component!(ReferenceStrokeRoles {
    borrow bar_zero: StrokeTheme => with_bar_zero,
    borrow waterfall_reference: StrokeTheme => with_waterfall_reference,
    borrow beeswarm_reference: StrokeTheme => with_beeswarm_reference,
    borrow point_interval_reference: StrokeTheme => with_point_interval_reference,
    borrow estimate_reference: StrokeTheme => with_estimate_reference,
    borrow qq_reference: StrokeTheme => with_qq_reference,
} validate |value| {
    for stroke in [value.bar_zero(), value.waterfall_reference(), value.beeswarm_reference(),
        value.point_interval_reference(), value.estimate_reference(), value.qq_reference()] {
        stroke.validate()?;
    }
});

validated_component!(ConnectorStrokeRoles {
    borrow waterfall_connector: StrokeTheme => with_waterfall_connector,
    borrow dumbbell_connector: StrokeTheme => with_dumbbell_connector,
} validate |value| {
    value.waterfall_connector().validate()?;
    value.dumbbell_connector().validate()?;
});

validated_component!(SummaryStrokeRoles {
    borrow uncertainty: StrokeTheme => with_uncertainty,
    borrow summary: StrokeTheme => with_summary,
    borrow estimate_interval: StrokeTheme => with_estimate_interval,
    borrow median: StrokeTheme => with_median,
} validate |value| {
    for stroke in [value.uncertainty(), value.summary(), value.estimate_interval(), value.median()] {
        stroke.validate()?;
    }
});

validated_component!(CurveStrokeRoles {
    borrow histogram_curve: StrokeTheme => with_histogram_curve,
    borrow histogram_rug: StrokeTheme => with_histogram_rug,
    borrow regression_fit: StrokeTheme => with_regression_fit,
} validate |value| {
    for stroke in [value.histogram_curve(), value.histogram_rug(), value.regression_fit()] {
        stroke.validate()?;
    }
});

validated_component!(StrokeRoles {
    borrow references: ReferenceStrokeRoles => with_references,
    borrow connectors: ConnectorStrokeRoles => with_connectors,
    borrow summaries: SummaryStrokeRoles => with_summaries,
    borrow curves: CurveStrokeRoles => with_curves,
} validate |value| {
    value.references().validate()?;
    value.connectors().validate()?;
    value.summaries().validate()?;
    value.curves().validate()?;
});

validated_component!(CellMarkRoles {
    copy fill_alpha: u8 => with_fill_alpha,
    optional cell_outline: OutlineTheme => with_cell_outline,
    copy missing_fill: Color => with_missing_fill,
    optional missing_row_outline: OutlineTheme => with_missing_row_outline,
    optional missing_key_outline: OutlineTheme => with_missing_key_outline,
    borrow missing_cross: StrokeTheme => with_missing_cross,
    borrow missing_cross_inset: RelativeSpacing => with_missing_cross_inset,
} validate |value| {
    for outline in [value.cell_outline(), value.missing_row_outline(), value.missing_key_outline()]
        .into_iter().flatten() { outline.validate()?; }
    value.missing_cross().validate()?;
    if value.missing_cross_inset().compare(&RelativeSpacing::ratio(1, 2)?).is_ge() {
        return Err(Error::Data("missing-cell cross inset must be smaller than one half".into()));
    }
});

validated_component!(CategoryMarkGeometry {
    borrow vertical_body_breadth: RelativeSpacing => with_vertical_body_breadth,
    borrow horizontal_body_breadth: RelativeSpacing => with_horizontal_body_breadth,
    borrow vertical_group_span: RelativeSpacing => with_vertical_group_span,
    borrow horizontal_group_span: RelativeSpacing => with_horizontal_group_span,
    borrow uncertainty_cap_ratio: RelativeSpacing => with_uncertainty_cap_ratio,
} validate |value| {
    let one = RelativeSpacing::ratio(1, 1)?;
    for extent in [value.vertical_body_breadth(), value.horizontal_body_breadth(),
        value.vertical_group_span(), value.horizontal_group_span()] {
        fraction(extent, &one, true)?;
    }
    fraction(value.uncertainty_cap_ratio(), &RelativeSpacing::ratio(1, 2)?, true)?;
});

validated_component!(MarkExtentTheme {
    borrow categories: CategoryMarkGeometry => with_categories,
    borrow waterfall_body_category_breadth: RelativeSpacing => with_waterfall_body_category_breadth,
    copy point_interval_cap_half_height: u32 => with_point_interval_cap_half_height,
    borrow sampled_rug_height: RelativeSpacing => with_sampled_rug_height,
    borrow marginal_density_fraction: RelativeSpacing => with_marginal_density_fraction,
    borrow matrix_density_fraction: RelativeSpacing => with_matrix_density_fraction,
} validate |value| {
    value.categories().validate()?;
    positive_radius(value.point_interval_cap_half_height())?;
    let one = RelativeSpacing::ratio(1, 1)?;
    for extent in [value.waterfall_body_category_breadth(), value.marginal_density_fraction(),
        value.matrix_density_fraction()] { fraction(extent, &one, true)?; }
});

impl CategoryMarkGeometry {
    pub(crate) fn group_span(&self, horizontal: bool) -> f64 {
        if horizontal {
            self.horizontal_group_span()
        } else {
            self.vertical_group_span()
        }
        .as_f64()
    }

    pub(crate) fn body_breadth(&self, horizontal: bool) -> f64 {
        if horizontal {
            self.horizontal_body_breadth()
        } else {
            self.vertical_body_breadth()
        }
        .as_f64()
    }
}

validated_component!(MarkTheme {
    borrow accents: AccentPalette => with_accents,
    borrow points: PointRoles => with_points,
    borrow fills: FillRoles => with_fills,
    borrow data_lines: DataLineRoles => with_data_lines,
    borrow neutral_paints: NeutralPaintRoles => with_neutral_paints,
    borrow strokes: StrokeRoles => with_strokes,
    borrow cells: CellMarkRoles => with_cells,
    borrow extents: MarkExtentTheme => with_extents,
} validate |value| {
    value.points().validate()?;
    value.fills().validate()?;
    value.data_lines().validate()?;
    value.neutral_paints().validate()?;
    value.strokes().validate()?;
    value.cells().validate()?;
    value.extents().validate()?;
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
            .fills()
            .bodies()
            .inner_summary()
            .clone()
            .with_color(Color::rgba(11, 23, 37, 53))
            .unwrap()
            .with_outline(None)
            .unwrap();
        let bodies = original
            .fills()
            .bodies()
            .clone()
            .with_inner_summary(summary)
            .unwrap();
        let fills = original.fills().clone().with_bodies(bodies).unwrap();
        let mut poisoned = original.clone();
        poisoned.points.mandatory_strip.radius = 0;
        assert!(poisoned.with_fills(fills).is_err());
        assert_eq!(original.points().mandatory_strip().radius(), 2);
        assert!(original
            .cells()
            .clone()
            .with_missing_cross_inset(RelativeSpacing::ratio(1, 2).unwrap())
            .is_err());
        assert!(original
            .extents()
            .categories()
            .clone()
            .with_vertical_body_breadth(RelativeSpacing::new(1.0_f64.next_up()).unwrap())
            .is_err());
        assert!(original
            .extents()
            .categories()
            .clone()
            .with_uncertainty_cap_ratio(RelativeSpacing::ratio(1, 2).unwrap())
            .is_ok());
        assert!(original
            .extents()
            .clone()
            .with_waterfall_body_category_breadth(RelativeSpacing::ratio(1, 1).unwrap())
            .is_ok());
    }

    #[test]
    fn sampled_and_supplied_strips_keep_their_own_style_and_guide_when_raw_roles_change() {
        use crate::plots::categorical_distribution::Mark;
        use crate::{CategoryValues, NumericValues, Plot, RaincloudInput};

        let sampled = Plot::raincloud(RaincloudInput::samples(
            CategoryValues::new("category", ["one"; 4]).unwrap(),
            NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        ))
        .series(CategoryValues::new("hue", ["series"; 4]).unwrap())
        .hide_point_outline()
        .erase();
        let supplied = crate::plots::categorical_distribution::extraction::supplied();
        for (source, count, outlined) in [(sampled, 4, false), (supplied, 2, true)] {
            let operation = crate::font::canonical::OperationContext::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
            let chart = inputs.attach(source).unwrap();
            let mut theme = crate::Theme::publication().resolve().unwrap();
            let appearance = theme
                .marks
                .points()
                .primary()
                .clone()
                .with_fill_alpha(117)
                .unwrap();
            let strip = SizedPointTheme::new(appearance, 5).unwrap();
            let points = theme
                .marks
                .points()
                .clone()
                .with_mandatory_strip(strip)
                .unwrap();
            theme.marks = theme.marks.clone().with_points(points).unwrap();
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
                .points()
                .raw_overlay()
                .clone()
                .with_radius(80)
                .unwrap();
            let points = theme.marks.points().clone().with_raw_overlay(raw).unwrap();
            theme.marks = theme.marks.clone().with_points(points).unwrap();
            assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), minimum);
            let after = measure(&theme);
            assert!(before.discrete_guide.is_some());
            assert_eq!(before.discrete_guide, after.discrete_guide);
            let crate::layout::interior::MarkPayload::Distribution(payload) =
                &after.marks.as_ref().unwrap().payload
            else {
                panic!("distribution payload")
            };
            let mut observed = 0;
            for mark in &payload.marks {
                if let Mark::Scatter(points, style) = mark {
                    observed += points.len();
                    assert_eq!(style.radius, 5);
                    assert_eq!(style.color.alpha(), 117);
                    assert_eq!(style.outline.is_some(), outlined);
                }
            }
            assert_eq!(observed, count);
        }
    }
}
