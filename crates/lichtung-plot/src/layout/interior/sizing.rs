//! Natural root composition at one chosen width. Public constructors stay exact.

use crate::builder::PlotDescription;
use crate::decorations::legend::discrete::PreparedGuide;
use crate::layout::fit::{self, FitScope};
use crate::layout::kernel::{root_dimension, FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{ComponentAddress, PhysicalTextAlignment};
use crate::{Error, PlotKind};

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

    pub(crate) fn preference(
        &self,
        theme: &crate::text::TextRequestContext<'_, '_>,
    ) -> Result<(f64, f64), Error> {
        crate::decorations::legend::Legend::natural_preference(&self.descriptor, theme)
    }

    pub(crate) fn at_width(
        &self,
        width: f64,
        theme: &crate::text::TextRequestContext<'_, '_>,
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
    plot: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
) -> Result<Option<GuidedPreference>, Error> {
    let native = matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm);
    let position = if native {
        crate::LegendPosition::OutsideRight
    } else {
        plot.legend_position_spec()
    };
    if crate::orchestrator::registration::legend_policy(plot, registered)
        == crate::LegendPolicy::Hidden
        || (position != crate::LegendPosition::Auto
            && !crate::decorations::plan::is_outside(position))
        || plot.kind() == PlotKind::Matrix
        || (native && crate::orchestrator::registration::continuous_guide_suppressed(registered))
    {
        return Ok(None);
    }
    let Some(descriptor) = crate::orchestrator::registration::guide_descriptor(registered)? else {
        return Ok(None);
    };
    Ok(Some(GuidedPreference {
        descriptor,
        position,
    }))
}

#[expect(
    clippy::too_many_arguments,
    reason = "the guide shares the occurrence's explicit operation without loading fonts or numeric presentations early"
)]
pub(crate) fn guided_body_height(
    plot: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    intrinsic: &crate::orchestrator::root::PlotIntrinsics,
    operation: &crate::font::canonical::OperationContext,
    theme: &crate::text::TextRequestContext<'_, '_>,
    width: f64,
    guide: Option<&GuidedPreference>,
    scope: &FitScope,
    ranges: Option<[(f64, f64); 2]>,
) -> Result<f64, Error> {
    let hard = plot.intrinsic_body_minimum(registered, intrinsic, operation, theme, ranges)?;
    let preference = body_preference(plot, registered, intrinsic, operation, theme, hard)?;
    let Some(guide) = guide else {
        return plot.natural_body_height(
            registered,
            intrinsic,
            operation,
            theme,
            width,
            preference.1,
            ranges,
        );
    };
    let natural = guide.preference(theme)?;
    if guide.horizontal() {
        let measured = guide.at_width(width, theme, scope)?;
        return Ok(plot.natural_body_height(
            registered,
            intrinsic,
            operation,
            theme,
            width,
            preference.1,
            ranges,
        )? + measured.1
            + theme.legend_outside_gap()?);
    }
    let available = width - hard.0 - theme.legend_outside_gap()?;
    if available <= 0.0
        || (available < natural.0 && !matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm))
    {
        // This proposal does not select an Auto edge. The existing containing
        // frame trials report the selected width-dependent content height.
        return Ok(plot
            .natural_body_height(
                registered,
                intrinsic,
                operation,
                theme,
                width,
                preference.1,
                ranges,
            )?
            .max(natural.1));
    }
    let allocated = natural.0.min(available);
    let measured = guide.at_width(allocated, theme, scope)?;
    Ok(plot
        .natural_body_height(
            registered,
            intrinsic,
            operation,
            theme,
            width - measured.0 - theme.legend_outside_gap()?,
            preference.1,
            ranges,
        )?
        .max(measured.1))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "root-bounded spatial quanta convert exactly to CSS pixels"
)]
fn pixels(value: i64) -> f64 {
    value as f64 / 1024.0
}

pub(crate) fn text_width<'a>(
    theme: &crate::text::TextRequestContext<'_, '_>,
    source: &str,
    role: impl Into<crate::text::TextAppearance<'a>>,
) -> Result<f64, Error> {
    let role = role.into();
    let (hard, natural) =
        fit::intrinsic_widths(theme, source, role, "natural root text preference")?;
    if natural <= hard {
        return Ok(hard);
    }
    let ch = theme
        .measure_text(
            "0",
            role.theme,
            PhysicalTextAlignment::Start,
            "natural root 66ch",
            role.exact_override,
        )?
        .width();
    Ok(hard.max(natural.min(66.0 * ch)))
}

pub(crate) fn body_preference(
    plot: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    intrinsic: &crate::orchestrator::root::PlotIntrinsics,
    operation: &crate::font::canonical::OperationContext,
    theme: &crate::text::TextRequestContext<'_, '_>,
    hard: (f64, f64),
) -> Result<(f64, f64), Error> {
    use super::{InteriorRoute, SpecializedGrid};
    let preference = match InteriorRoute::classify(plot.kind()) {
        InteriorRoute::StandardCoordinate(_) => {
            let preference = super::preference::mark_preference(
                registered.expect("prepared standard preference"),
                crate::builder::RootFrameDomainInput::Prepared,
                theme,
                hard,
            )?;
            Ok((
                pixels(preference.body_quanta.0),
                pixels(preference.body_quanta.1),
            ))
        }
        InteriorRoute::OneCategoricalAxis(_) => {
            let preference = super::preference::categorical_preference(
                registered.expect("prepared categorical preference"),
                crate::builder::RootFrameDomainInput::Prepared,
                theme,
                hard,
            )?;
            Ok((
                pixels(preference.body_quanta.0),
                pixels(preference.body_quanta.1),
            ))
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Marginal) => {
            crate::plots::marginal::natural_body_preference(
                plot, registered, intrinsic, operation, theme,
            )
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Matrix) => {
            crate::plots::matrix::natural_body_preference(plot, registered, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Heatmap) => {
            crate::plots::heatmap::natural_body_preference(registered, intrinsic, operation, theme)
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
            let count = registered
                .and_then(|registered| registered.axes.coordinate())
                .and_then(|axes| axes.categories(1))
                .map_or(0, |categories| categories.raw.len());
            let count = u32::try_from(count).map_err(|_| {
                Error::Render("specialized category preference count overflow".into())
            })?;
            let height = hard
                .1
                .max(2.0 * theme.typography.axes.tick_label.font_size * f64::from(count));
            let width = hard.0.max(66.0 * ch).max(4.0 * height / 3.0);
            Ok((width, height.max(3.0 * width / 4.0)))
        }
    }?;
    Ok((
        preference.0.max(crate::numeric::values::preferred_width(
            plot, registered, operation, theme,
        )?),
        preference.1,
    ))
}

fn axis_title_style<'a>(
    registered: Option<&'a crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &'a crate::text::TextRequestContext<'_, '_>,
    side: usize,
) -> &'a crate::TextTheme {
    registered
        .and_then(|registered| registered.axes.coordinate())
        .and_then(|axes| {
            crate::orchestrator::registration::axis_label(
                registered.expect("prepared axis occurrence"),
                &axes.sources()[side],
                side,
            )
        })
        .map_or(&theme.typography.axes.axis_title, |text| {
            text.appearance(&theme.typography.axes.axis_title).theme
        })
}

/// Intrinsic horizontal axis/chrome preference before a root chooses its width.
/// Actual selected-source gutters subsequently replace this estimate via feedback.
#[expect(
    clippy::too_many_lines,
    reason = "category, exact and numeric gutter sources retain their distinct original admission order"
)]
pub(crate) fn horizontal_gutter_preference(
    plot: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    operation: &crate::font::canonical::OperationContext,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let title = axis_title_style(registered, theme, 1);
    let lengths = theme.axes.get_spacing().lengths(
        theme.typography.axes.tick_label.get_font_size(),
        title.get_font_size(),
    )?;
    let chrome = if plot.kind() == PlotKind::Matrix {
        theme.axes.cell_axis_chrome()
    } else {
        theme.axes.coordinate_chrome()
    };
    let mut label_width = 0.0_f64;
    let mut include_label = |label: &str| -> Result<(), Error> {
        if !label.is_empty() {
            let plan = theme.measure_appearance_text(
                label,
                theme.tick_text_appearance(crate::layout::AxisSide::Left),
                PhysicalTextAlignment::End,
                "natural vertical gutter",
            )?;
            label_width = label_width.max(plan.occupied_size().0.max(plan.fit_width()));
        }
        Ok(())
    };
    if let Some(axes) = registered
        .and_then(|registered| registered.axes.coordinate())
        .filter(|_| crate::orchestrator::registration::axis_tick_labels_visible(registered, 1))
    {
        let base = axes.axis(registered.expect("prepared axis occurrence").source, 1);
        if let Some(categories) = axes.categories(1) {
            let labels = &categories.ticks;
            let eligible = crate::data::source::collect_source(
                base.authored()
                    .expect("coordinate authored intent")
                    .category_labels
                    .indices(labels.len())
                    .map(Ok),
            )?;
            for &index in &eligible {
                if !theme
                    .tick_text_appearance(crate::layout::AxisSide::Left)
                    .includes(index, &eligible)
                {
                    continue;
                }
                include_label(&labels[index].label)?;
            }
        } else if let Some(ticks) = &base
            .authored()
            .expect("coordinate authored intent")
            .exact_ticks
        {
            let eligible = crate::data::source::collect_source((0..ticks.len()).map(Ok))?;
            for (index, tick) in ticks.iter().enumerate() {
                if !theme
                    .tick_text_appearance(crate::layout::AxisSide::Left)
                    .includes(index, &eligible)
                {
                    continue;
                }
                include_label(&tick.label)?;
            }
        } else if let Some(probe) = base.numeric_probe(
            registered
                .expect("prepared axis occurrence")
                .axis_numeric_binding(operation)?,
        )? {
            for label in &probe.labels {
                include_label(label)?;
            }
        } else if !base.empty_numeric_source(
            registered.expect("prepared axis occurrence").raw_windows()[1]
                .map(|window| window.pair()),
        )? {
            for value in [
                axes.ranges()[1].0,
                f64::midpoint(axes.ranges()[1].0, axes.ranges()[1].1),
                axes.ranges()[1].1,
            ] {
                include_label(&crate::layout::format_guide_label(value, 0)?)?;
            }
        }
    }
    let edge = theme.panel_edge_length()?;
    let left = lengths
        .y_tick_label_gap
        .add_length(&lengths.y_title_tick_label_gap)
        .add_length(&lengths.y_title_edge_padding)
        .add_pixels(title.line_pitch_pixels())?
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
        .filter(|frame| frame.get_sides().get_right())
        .map_or(0.0, |frame| f64::from(frame.get_stroke().get_width()) / 2.0);
    let tick = theme
        .axes
        .get_ticks()
        .map_or(0.0, |ticks| f64::from(ticks.get_stroke().get_width()) / 2.0);
    let right = edge.add_pixels(frame.max(tick))?;
    Ok((
        crate::theme::AxisLengths::hard(&left)?,
        crate::theme::AxisLengths::hard(&right)?,
    ))
}

/// Measure natural outer height after width selection, under consumed root feedback.
#[expect(
    clippy::too_many_lines,
    reason = "preserve the original root preference, feedback and prose ordering"
)]
pub(crate) fn plot_size(
    occurrence: &crate::orchestrator::registration::PlotRegistration<'_>,
    intrinsic: &crate::orchestrator::root::PlotIntrinsics,
    operation: &crate::font::canonical::OperationContext,
    theme: &crate::text::TextRequestContext<'_, '_>,
    requested: crate::layout::RootRequest,
    round: &FeedbackRound,
) -> Result<RootSize, Error> {
    let requested_width = requested.exact_width();
    if let crate::layout::RootRequest::ExactSize { width, height } = requested {
        return Ok(RootSize::new(width, height));
    }

    let plot = occurrence.source;
    let registered = Some(occurrence);
    let hard = plot.intrinsic_body_minimum(registered, intrinsic, operation, theme, None)?;
    let preference = body_preference(plot, registered, intrinsic, operation, theme, hard)?;
    let title = axis_title_style(registered, theme, 0);
    let lengths = theme.axes.get_spacing().lengths(
        theme.typography.axes.tick_label.get_font_size(),
        title.get_font_size(),
    )?;
    let common_chrome = if plot.kind() == PlotKind::Matrix {
        theme.axes.cell_axis_chrome()
    } else {
        theme.axes.coordinate_chrome()
    };
    let panel_edge = theme.panel_edge_length()?;
    let bottom_outward = common_chrome.outward(crate::layout::AxisSide::Bottom, true)?;
    let (initial_left, initial_right) =
        horizontal_gutter_preference(plot, registered, operation, theme)?;
    let guide = guided_preference(plot, registered)?;
    let (guide_width, guide_height) = if plot.kind() == PlotKind::Matrix {
        crate::plots::matrix::natural_guide_preference(registered, theme)?
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
    let width = requested.select_width(if requested_width.is_none() {
        outer_width(
            plot,
            theme,
            preference.0,
            (initial_left, initial_right),
            guide_width,
            horizontal,
        )?
    } else {
        0.0
    })?;
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
    let body_height = plot.natural_body_height(
        registered,
        intrinsic,
        operation,
        theme,
        body_width,
        preference.1,
        None,
    )?;
    let sources = plot.prose();
    let prose = crate::decorations::title_block::plan::RootTitleBlockPlan::content_reserves(
        theme,
        sources,
        theme.root_inner_width(f64::from(width))?,
        crate::decorations::title_block::plan::RootAlignments::resolve(
            theme,
            plot.prose_alignments(),
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
                .add_pixels(title.line_pitch_pixels())?
                .add_length(&bottom_outward)
                .add_length(&panel_edge),
        )?
    };
    let height = root_dimension(theme.root_outer_height(&prose.height_with_body(
        body_height.max(guide_height) + bottom + gutters[3] + ink[2] + ink[3],
    )?)?)?;
    Ok(RootSize::new(width, height))
}

fn outer_width(
    plot: &PlotDescription,
    theme: &crate::text::TextRequestContext<'_, '_>,
    body: f64,
    gutters: (f64, f64),
    guide: f64,
    horizontal: bool,
) -> Result<f64, Error> {
    let mut width = theme
        .root_outer_width(body + gutters.0 + gutters.1 + if horizontal { 0.0 } else { guide })?;
    if horizontal {
        width = width.max(theme.root_outer_width(guide)?);
    }
    for (source, role) in plot.prose().into_iter().zip(
        crate::decorations::title_block::plan::RootTitleBlockPlan::appearances(theme, plot.prose()),
    ) {
        if let Some(source) = source {
            width = width.max(theme.root_outer_width(text_width(theme, source.as_str(), role)?)?);
        }
    }
    Ok(width)
}

impl PlotDescription {
    #[expect(
        clippy::too_many_arguments,
        reason = "the existing height reader keeps registration, intrinsic, operation, request and independent allocation operands explicit"
    )]
    pub(crate) fn natural_body_height(
        &self,
        registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        intrinsic: &crate::orchestrator::root::PlotIntrinsics,
        operation: &crate::font::canonical::OperationContext,
        theme: &crate::text::TextRequestContext<'_, '_>,
        width: f64,
        preference: f64,
        ranges: Option<[(f64, f64); 2]>,
    ) -> Result<f64, Error> {
        let hard = self.intrinsic_body_minimum(registered, intrinsic, operation, theme, ranges)?;
        let height = match self.kind() {
            PlotKind::Heatmap => {
                crate::plots::heatmap::natural_body_height(self, registered, theme, width)
            }
            PlotKind::Beeswarm => crate::plots::beeswarm::natural_body_height(
                self,
                registered,
                intrinsic,
                operation,
                theme,
                width,
                ranges.map(|ranges| ranges[0]),
            ),
            PlotKind::Waterfall => crate::plots::waterfall::natural_body_height(
                self, registered, intrinsic, operation, theme, width, ranges,
            ),
            PlotKind::Matrix => Ok(width.max(hard.1)),
            PlotKind::Marginal => crate::plots::marginal::natural_body_height(
                self, registered, intrinsic, operation, theme, width,
            ),
            _ => {
                let density = match super::InteriorRoute::classify(self.kind()) {
                    super::InteriorRoute::StandardCoordinate(_) => {
                        super::preference::mark_preference(
                            registered.expect("prepared standard preference"),
                            crate::builder::RootFrameDomainInput::Prepared,
                            theme,
                            hard,
                        )?
                        .density_side
                    }
                    super::InteriorRoute::OneCategoricalAxis(_) => {
                        let categorical = super::preference::categorical_preference(
                            registered.expect("prepared categorical preference"),
                            crate::builder::RootFrameDomainInput::Prepared,
                            theme,
                            hard,
                        )?;
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
        crate::layout::kernel::required_length(
            height.max(crate::numeric::values::axis_preferences(registered, operation, theme)?.1)
                + crate::numeric::values::natural_height(self, registered, theme, width)?,
        )
    }
}
