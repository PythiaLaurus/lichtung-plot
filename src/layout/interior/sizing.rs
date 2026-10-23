//! Natural root composition at one chosen width. Public constructors stay exact.

use crate::decorations::legend::discrete::PreparedGuide;
use crate::layout::fit::{self, FitScope};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{ComponentAddress, PhysicalTextAlignment};
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind};

pub(crate) struct GuidedPreference {
    pub(crate) descriptor: std::sync::Arc<PreparedGuide>,
    pub(crate) position: crate::LegendPosition,
}

impl GuidedPreference {
    pub(crate) fn horizontal(&self) -> bool {
        matches!(
            self.position,
            crate::LegendPosition::OutsideTopLeft
                | crate::LegendPosition::OutsideTop
                | crate::LegendPosition::OutsideTopRight
                | crate::LegendPosition::OutsideBottomLeft
                | crate::LegendPosition::OutsideBottom
                | crate::LegendPosition::OutsideBottomRight
        )
    }

    pub(crate) fn preference(&self, theme: &ResolvedTheme) -> Result<(f64, f64), Error> {
        crate::decorations::legend::Legend::natural_preference(&self.descriptor, theme)
    }

    pub(crate) fn at_width(
        &self,
        width: f64,
        theme: &ResolvedTheme,
        scope: &FitScope,
    ) -> Result<(f64, f64), Error> {
        let guide = fit::measured(crate::decorations::legend::Legend::measure_assigned(
            std::sync::Arc::clone(&self.descriptor),
            (width, None),
            self.horizontal(),
            false,
            theme,
            &mut false,
            scope,
        ))
        .map_err(fit::PlanError::into_error)?;
        Ok((guide.compute_box_width(), guide.compute_box_height()))
    }
}

/// The prepared scope owns the guide. No panel assignment or edge trial occurs
/// while this intrinsic input is retained.
pub(crate) fn guided_preference(
    plot: &Plot,
    _inputs: &super::PreparedInteriorInputs,
    _theme: &ResolvedTheme,
) -> Result<Option<GuidedPreference>, Error> {
    let native = matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm);
    let position = if native {
        crate::LegendPosition::OutsideRight
    } else {
        plot.legend_position_spec()
    };
    if plot.legend_policy_spec() == crate::LegendPolicy::Hidden
        || (position != crate::LegendPosition::Auto
            && !crate::decorations::plan::is_outside(position))
        || plot.kind() == PlotKind::Matrix
        || (native && plot.continuous_guide_suppressed())
    {
        return Ok(None);
    }
    let Some(descriptor) = plot.prepared_discrete_guide()? else {
        return Ok(None);
    };
    Ok(Some(GuidedPreference {
        descriptor,
        position,
    }))
}

pub(crate) fn guided_body_height(
    plot: &Plot,
    theme: &ResolvedTheme,
    width: f64,
    guide: Option<&GuidedPreference>,
    scope: &FitScope,
) -> Result<f64, Error> {
    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let Some(guide) = guide else {
        return plot.natural_body_height(theme, width, preference.1);
    };
    let natural = guide.preference(theme)?;
    if guide.horizontal() {
        let measured = guide.at_width(width, theme, scope)?;
        return Ok(plot.natural_body_height(theme, width, preference.1)?
            + measured.1
            + theme.legend_outside_gap()?);
    }
    let available = width - hard.0 - theme.legend_outside_gap()?;
    if available <= 0.0
        || (available < natural.0 && !matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm))
    {
        // This proposal does not select an Auto edge. The existing containing
        // frame trials report the selected width-dependent content height.
        return Ok(plot
            .natural_body_height(theme, width, preference.1)?
            .max(natural.1));
    }
    let allocated = natural.0.min(available);
    let measured = guide.at_width(allocated, theme, scope)?;
    Ok(plot
        .natural_body_height(
            theme,
            width - measured.0 - theme.legend_outside_gap()?,
            preference.1,
        )?
        .max(measured.1))
}

pub(crate) fn integral(value: f64) -> Result<u32, Error> {
    let quanta = crate::layout::kernel::ceil_spatial_quanta(value)?;
    let pixels = (i128::from(quanta) + 1023) / 1024;
    u32::try_from(pixels)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            Error::Render(
                "natural root dimension is not representable as positive CSS pixels".into(),
            )
        })
}

#[expect(
    clippy::cast_precision_loss,
    reason = "root-bounded spatial quanta convert exactly to CSS pixels"
)]
fn pixels(value: i64) -> f64 {
    value as f64 / 1024.0
}

pub(crate) fn text_width(
    theme: &ResolvedTheme,
    source: &str,
    role: &crate::theme::TextTheme,
) -> Result<f64, Error> {
    let (hard, natural) =
        fit::intrinsic_widths(theme, source, role, "natural root text preference")?;
    if natural <= hard {
        return Ok(hard);
    }
    let ch = theme
        .measure_text(
            "0",
            role,
            PhysicalTextAlignment::Start,
            "natural root 66ch",
            false,
        )?
        .width();
    Ok(hard.max(natural.min(66.0 * ch)))
}

pub(crate) fn body_preference(
    plot: &Plot,
    theme: &ResolvedTheme,
    hard: (f64, f64),
) -> Result<(f64, f64), Error> {
    use super::{InteriorRoute, SpecializedGrid};
    match InteriorRoute::classify(plot.kind()) {
        InteriorRoute::StandardCoordinate(_) => {
            let preference = super::preference::mark_preference(plot, theme, hard)?;
            Ok((
                pixels(preference.body_quanta.0),
                pixels(preference.body_quanta.1),
            ))
        }
        InteriorRoute::OneCategoricalAxis(_) => {
            let preference = super::preference::categorical_preference(plot, theme, hard)?;
            Ok((
                pixels(preference.body_quanta.0),
                pixels(preference.body_quanta.1),
            ))
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Marginal) => {
            crate::plots::marginal::natural_body_preference(plot, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Matrix) => {
            crate::plots::matrix::natural_body_preference(plot, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Heatmap) => {
            crate::plots::heatmap::natural_body_preference(plot, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Beeswarm | SpecializedGrid::Waterfall) => {
            let ch = theme
                .measure_text(
                    "0",
                    &theme.typography.axes.tick_label,
                    PhysicalTextAlignment::Start,
                    "specialized numeric 66ch",
                    false,
                )?
                .width();
            let count = plot
                .prepared_axis_inputs()
                .and_then(|axes| axes.categories[1].as_ref())
                .map_or(0, Vec::len);
            let count = u32::try_from(count).map_err(|_| {
                Error::Render("specialized category preference count overflow".into())
            })?;
            let height = hard.1.max(
                2.0 * f64::from(theme.typography.axes.tick_label.font_size) * f64::from(count),
            );
            let width = hard.0.max(66.0 * ch).max(4.0 * height / 3.0);
            Ok((width, height.max(3.0 * width / 4.0)))
        }
    }
}

/// Intrinsic horizontal axis/chrome preference before a root chooses its width.
/// Actual selected-source gutters subsequently replace this estimate via feedback.
pub(crate) fn horizontal_gutter_preference(
    plot: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let lengths = theme.axes.spacing().lengths(
        theme.typography.axes.tick_label.get_font_size(),
        theme.typography.axes.axis_title.get_font_size(),
    )?;
    let chrome = if plot.kind() == PlotKind::Matrix {
        theme.axes.cell_axis_chrome()
    } else {
        theme.axes.coordinate_chrome()
    };
    let mut label_width = 0.0_f64;
    let mut include_label = |label: &str| -> Result<(), Error> {
        if !label.is_empty() {
            let plan = theme.measure_text(
                label,
                &theme.typography.axes.tick_label,
                PhysicalTextAlignment::End,
                "natural vertical gutter",
                false,
            )?;
            label_width = label_width.max(plan.width().max(plan.fit_width()));
        }
        Ok(())
    };
    if let Some(axes) = plot
        .prepared_axis_inputs()
        .filter(|_| plot.y_tick_labels_visible())
    {
        if let Some(labels) = &axes.display_categories[1] {
            for label in labels {
                include_label(label)?;
            }
        } else if let Some(ticks) = &axes.specs[1].exact_ticks {
            for tick in ticks {
                include_label(&tick.label)?;
            }
        } else {
            for value in [
                axes.ranges[1].0,
                f64::midpoint(axes.ranges[1].0, axes.ranges[1].1),
                axes.ranges[1].1,
            ] {
                if let Some(format) = &axes.specs[1].format {
                    let (min, max) = format.get_precision().bounds()?;
                    let mut formatter = crate::numeric::power::certified::PowerFormatter::default();
                    for digits in min..=max {
                        include_label(&format.format_selected(
                            value,
                            digits,
                            false,
                            &mut formatter,
                        )?)?;
                    }
                } else {
                    include_label(&crate::layout::format_guide_label(value, 0)?)?;
                }
            }
        }
    }
    let edge = theme.panel_edge_length()?;
    let left = lengths
        .y_tick_label_gap
        .add_length(&lengths.y_title_tick_label_gap)
        .add_length(&lengths.y_title_edge_padding)
        .add_pixels(theme.typography.axes.axis_title.line_pitch_pixels())?
        .add_length(&chrome.outward(crate::layout::AxisSide::Left, true)?)
        .add_length(&edge)
        .add_pixels(label_width)?;
    // Matrix's native variable Grid owns its trailing horizontal edge. Its
    // outer preference estimates only the leading axis column here.
    if plot.kind() == PlotKind::Matrix {
        return Ok((crate::theme::AxisLengths::hard(&left)?, 0.0));
    }
    let frame = chrome
        .frame
        .filter(|frame| frame.sides().right())
        .map_or(0.0, |frame| f64::from(frame.stroke().width()) / 2.0);
    let tick = theme
        .axes
        .ticks()
        .map_or(0.0, |ticks| f64::from(ticks.stroke().width()) / 2.0);
    let right = edge.add_pixels(frame.max(tick))?;
    Ok((
        crate::theme::AxisLengths::hard(&left)?,
        crate::theme::AxisLengths::hard(&right)?,
    ))
}

/// Measure natural outer height after width selection, under consumed root feedback.
pub(crate) fn plot_size(
    plot: &Plot,
    inputs: &super::PreparedInteriorInputs,
    theme: &ResolvedTheme,
    requested: RootSize,
    round: &FeedbackRound,
) -> Result<RootSize, Error> {
    let (requested_width, requested_height) = requested.requested_dimensions();
    if let (Some(width), Some(height)) = (requested_width, requested_height) {
        return Ok(RootSize::new(width, height));
    }

    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let lengths = theme.axes.spacing().lengths(
        theme.typography.axes.tick_label.get_font_size(),
        theme.typography.axes.axis_title.get_font_size(),
    )?;
    let common_chrome = if plot.kind() == PlotKind::Matrix {
        theme.axes.cell_axis_chrome()
    } else {
        theme.axes.coordinate_chrome()
    };
    let panel_edge = theme.panel_edge_length()?;
    let bottom_outward = common_chrome.outward(crate::layout::AxisSide::Bottom, true)?;
    let (initial_left, initial_right) = horizontal_gutter_preference(plot, theme)?;
    let guide = guided_preference(plot, inputs, theme)?;
    let (guide_width, guide_height) = if plot.kind() == PlotKind::Matrix {
        crate::plots::matrix::natural_guide_preference(plot, theme)?
    } else if let Some(guide) = &guide {
        guide.preference(theme)?
    } else {
        (0.0, 0.0)
    };
    let guide_width = guide_width
        + if guide_width > 0.0 {
            theme.legend_outside_gap()?
        } else {
            0.0
        };
    let horizontal = guide.as_ref().is_some_and(GuidedPreference::horizontal);
    let mut outer_width = theme.root_outer_width(
        preference.0 + initial_left + initial_right + if horizontal { 0.0 } else { guide_width },
    )?;
    if horizontal {
        outer_width = outer_width.max(theme.root_outer_width(guide_width)?);
    }
    for (source, role) in [plot.title_text(), plot.subtitle_text(), plot.caption_text()]
        .into_iter()
        .zip([
            &theme.typography.root.title,
            &theme.typography.root.subtitle,
            &theme.typography.root.caption,
        ])
    {
        if let Some(source) = source {
            outer_width =
                outer_width.max(theme.root_outer_width(text_width(theme, source, role)?)?);
        }
    }
    let width = requested_width.map_or_else(|| integral(outer_width), Ok)?;
    let owner = ComponentAddress::root().child(crate::layout::AddressRole::PlotBody, 0)?;
    let gutters = round.insets(&owner, FeedbackKind::AxisGutter);
    let ink = round.insets(&owner, FeedbackKind::OwnedInk);
    let left = if gutters[0] > 0.0 {
        gutters[0]
    } else {
        initial_left
    };
    let right = if gutters[1] > 0.0 {
        gutters[1]
    } else {
        initial_right
    };
    // This initial content proposal is not a guide placement. The owning panel
    // trial derives its height from the selected guide edge at this fixed width.
    let body_width =
        (theme.root_inner_width(f64::from(width))? - left - right - ink[0] - ink[1]).max(hard.0);
    let body_height = plot.natural_body_height(theme, body_width, preference.1)?;
    let sources = [plot.title_text(), plot.subtitle_text(), plot.caption_text()];
    let prose = crate::decorations::title_block::plan::RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        crate::decorations::title_block::plan::RootAlignments::resolve(
            theme,
            [plot.title_alignment, plot.subtitle_alignment],
            [None; 2],
        ),
        &FitScope::root(),
    )
    .map_err(fit::PlanError::into_error)?;
    let bottom = if gutters[2] > 0.0 {
        gutters[2]
    } else {
        crate::theme::AxisLengths::hard(
            &lengths
                .x_tick_label_gap
                .add_length(&lengths.x_title_tick_label_gap)
                .add_pixels(theme.typography.axes.tick_label.line_pitch_pixels())?
                .add_pixels(theme.typography.axes.axis_title.line_pitch_pixels())?
                .add_length(&bottom_outward)
                .add_length(&panel_edge),
        )?
    };
    let height = integral(theme.root_outer_height(&prose.height_with_body(
        body_height.max(guide_height) + bottom + gutters[3] + ink[2] + ink[3],
    )?)?)?;
    Ok(RootSize::new(width, height))
}

impl Plot {
    pub(crate) fn natural_body_height(
        &self,
        theme: &ResolvedTheme,
        width: f64,
        preference: f64,
    ) -> Result<f64, Error> {
        let hard = self.intrinsic_body_minimum(theme)?;
        let height = match self.kind() {
            PlotKind::Heatmap => crate::plots::heatmap::natural_body_height(self, theme, width),
            PlotKind::Beeswarm => crate::plots::beeswarm::natural_body_height(self, theme, width),
            PlotKind::Waterfall => crate::plots::waterfall::natural_body_height(self, theme, width),
            PlotKind::Matrix => Ok(width.max(hard.1)),
            PlotKind::Marginal => crate::plots::marginal::natural_body_height(self, theme, width),
            _ => {
                let density = match super::InteriorRoute::classify(self.kind()) {
                    super::InteriorRoute::StandardCoordinate(_) => {
                        super::preference::mark_preference(self, theme, hard)?.density_side
                    }
                    super::InteriorRoute::OneCategoricalAxis(_) => {
                        let categorical =
                            super::preference::categorical_preference(self, theme, hard)?;
                        if categorical.axis == crate::layout::AxisSide::Left {
                            pixels(categorical.body_quanta.1)
                        } else {
                            0.0
                        }
                    }
                    super::InteriorRoute::SpecializedGrid(_) => preference,
                };
                Ok(hard.1.max(3.0 * width / 4.0).max(density))
            }
        }?;
        crate::layout::kernel::required_length(height)
    }
}

#[cfg(test)]
mod tests {
    fn prepared_color(
        descriptor: crate::decorations::legend::descriptor::ContinuousDescriptor,
        theme: &crate::theme::ResolvedTheme,
    ) -> std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide> {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0]),
            crate::NumericValues::new("y", [0.0]),
        )
        .erase();
        crate::decorations::legend::discrete::PreparedGuide::prepare(
            &plot,
            theme,
            &crate::layout::ComponentAddress::root(),
            Vec::new(),
            Some(std::sync::Arc::new(descriptor)),
            Vec::new(),
            crate::LegendComposition::Combined,
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the one-quantum width oracle includes complete configured label and native-guide operands"
    )]
    fn continuous_natural_and_native_right_width_keep_half_quantum_reserves_exact() {
        use crate::decorations::legend::{descriptor::ContinuousDescriptor, Legend};
        use crate::layout::{
            fit::{FitOutcome, FitScope},
            PanelBox,
        };
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let half = crate::theme::RelativeSpacing::ratio(1, 22_528).unwrap();
        let spacing = theme
            .legend
            .spacing()
            .clone()
            .with_padding(half.clone())
            .unwrap()
            .with_key_label_gap(half)
            .unwrap();
        theme.legend = theme
            .legend
            .clone()
            .with_spacing(spacing)
            .unwrap()
            .with_frame(None)
            .unwrap();
        let descriptor = ContinuousDescriptor {
            field: "Value".into(),
            title: "Value".into(),
            selection: crate::theme::ContinuousColorScale::sequential(None, None).unwrap(),
            domain: (0.000_000_000_012_345, 0.000_000_000_098_765),
            center: None,
            exact: true,
            missing: None,
            stops: vec![
                crate::render::backend::GradientStop::new(0.0, (0, 0, 0)),
                crate::render::backend::GradientStop::new(1.0, (255, 255, 255)),
            ],
            format: Some(std::sync::Arc::new(
                crate::numeric::format::NumericFormat::new(
                    crate::numeric::decimal::NumericNotation::Decimal,
                    crate::numeric::format::NumericPrecision::Fractional(15),
                )
                .unwrap(),
            )),
            numeric: crate::numeric::set::FormattedSet {
                members: vec![
                    0.000_000_000_012_345_f64.to_bits(),
                    0.000_000_000_098_765_f64.to_bits(),
                ],
                labels: vec!["0.000000000012345".into(), "0.000000000098765".into()],
                precision: 15,
            },
        };
        let label_width = descriptor
            .numeric
            .labels
            .iter()
            .map(|label| {
                let plan = theme
                    .measure_text(
                        label,
                        &theme.typography.guides.legend_label,
                        crate::layout::PhysicalTextAlignment::Start,
                        "fractional continuous width fixture",
                        false,
                    )
                    .unwrap();
                plan.width().max(plan.fit_width())
            })
            .fold(0.0_f64, f64::max);
        let base = crate::layout::kernel::ceil_spatial_quanta(11.0 + label_width).unwrap();
        let full = f64::from(u32::try_from(base + 2).unwrap()) / 1024.0;
        let below = f64::from(u32::try_from(base + 1).unwrap()) / 1024.0;
        let scope = FitScope::root();
        let descriptor = prepared_color(descriptor, &theme);
        let (width, _) = Legend::natural_preference(&descriptor, &theme).unwrap();
        assert_eq!(width.to_bits(), full.to_bits());
        assert!(matches!(
            Legend::measure_assigned(
                std::sync::Arc::clone(&descriptor),
                (below, 500.0),
                false,
                false,
                &theme,
                &mut false,
                &scope
            )
            .unwrap(),
            FitOutcome::Miss(_)
        ));

        let gap = theme.legend_outside_gap().unwrap();
        let (_, guide) = crate::layout::interior::guide::native_right(
            std::sync::Arc::clone(&descriptor),
            PanelBox::new(0.0, 64.0 + gap + full + 128.0, 0.0, 500.0),
            64.0,
            &theme,
            &scope,
            None,
        )
        .unwrap();
        assert_eq!(guide.compute_box_width().to_bits(), full.to_bits());
        assert!(matches!(
            crate::layout::interior::guide::native_right(
                std::sync::Arc::clone(&descriptor),
                PanelBox::new(0.0, 64.0 + gap + below, 0.0, 500.0),
                64.0,
                &theme,
                &scope,
                None
            ),
            Err(crate::layout::fit::PlanError::Fit(_))
        ));
    }

    #[test]
    fn continuous_intrinsic_width_admits_broad_frame_and_missing_key_chrome() {
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let frame = crate::theme::LegendFrameTheme::new(
            theme.legend.frame().unwrap().fill(),
            Some(
                crate::theme::OutlineTheme::new(
                    theme.legend.frame().unwrap().outline().unwrap().color(),
                    30.0,
                )
                .unwrap(),
            ),
            crate::theme::RelativeSpacing::from_pixels(50.0, 11).unwrap(),
        )
        .unwrap();
        theme.legend = theme.legend.clone().with_frame(Some(frame)).unwrap();
        let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
            selection: crate::theme::ContinuousColorScale::sequential(
                Some(
                    crate::theme::scale_values::test_defaults()
                        .sequential()
                        .clone(),
                ),
                None,
            )
            .unwrap(),
            field: "signal".into(),
            title: "Signal".into(),
            stops: vec![
                crate::render::backend::GradientStop::new(0.0, (0, 0, 0)),
                crate::render::backend::GradientStop::new(1.0, (255, 255, 255)),
            ],
            center: None,
            missing: Some(crate::decorations::legend::descriptor::MissingKey {
                fill: crate::theme::Color::rgb(224, 224, 224),
                outline: Some(crate::theme::OutlineTheme {
                    paint: crate::theme::Color::rgb(120, 120, 120),
                    width: 40.0,
                }),
                label: "Missing".into(),
            }),
            domain: (0.0, 1.0),
            exact: false,
            format: None,
            numeric: crate::decorations::legend::descriptor::domain_presentation(
                (0.0, 1.0),
                None,
                None,
            )
            .unwrap(),
        };
        let descriptor = prepared_color(descriptor, &theme);
        let (width, height) =
            crate::decorations::legend::Legend::natural_preference(&descriptor, &theme).unwrap();
        assert!(width >= 130.0 && height >= 130.0);
        assert!(width < 300.0 && height < 500.0);
    }

    #[test]
    fn upward_css_conversion_preserves_extreme_representability() {
        assert_eq!(super::integral(1.0 / 1024.0).unwrap(), 1);
        assert_eq!(super::integral(300.0 + 1.0 / 1024.0).unwrap(), 301);
        assert_eq!(super::integral(f64::from(u32::MAX)).unwrap(), u32::MAX);
        for value in [
            0.0,
            -1.0,
            f64::INFINITY,
            f64::NAN,
            f64::from(u32::MAX) + 1.0 / 1024.0,
        ] {
            assert!(super::integral(value).is_err());
        }
    }
}
