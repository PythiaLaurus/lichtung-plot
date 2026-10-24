//! Native coordinate panels. Parents allocate the frame; this common producer
//! selects its axes once and hands the resulting projection to retained marks.

use super::{InteriorRoute, OneCategoricalAxis, StandardCoordinate};
use crate::layout::fit::{self, FitKind, PlanError};
#[cfg(test)]
use crate::layout::PanelBox;
use crate::layout::{Axis, AxisSide, Layout, Projection, StandardPanelInputs};
use crate::spec::{AxisDomain, PlotRenderContext, PlotRenderResult};
use crate::{Error, Plot};

/// Foundational axis text contributes hard requirements independently from
/// the prepared mark and annotation intrinsics of each family.
#[derive(Debug)]
pub(crate) struct Intrinsics {
    standard: StandardPanelInputs,
    titles: [Option<crate::Text>; 2],
    formats: [Option<std::sync::Arc<crate::numeric::format::NumericFormat>>; 2],
    base_ranges: [(u64, u64); 2],
    logarithmic: bool,
    minimum_body: (f64, f64),
    without_y_categories: f64,
}

impl Intrinsics {
    /// The prepared axis scope is the owner identity. Parent widths, domain
    /// feedback, and hidden shared-axis labels cannot invalidate its analysis.
    pub(crate) fn prepared<'a>(
        plot: &'a Plot,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<&'a Self, Error> {
        let axes = plot.prepared_axis_inputs().ok_or_else(|| {
            Error::Render("native coordinate panel requires root-prepared axes".into())
        })?;
        Self::prepare(axes, theme, plot.axis_scope())
    }

    pub(crate) fn prepare<'a>(
        axes: &'a crate::plots::shared_x::PreparedAxes,
        theme: &crate::theme::ResolvedTheme,
        scope: Option<&str>,
    ) -> Result<&'a Self, Error> {
        if axes.intrinsics.get().is_none() {
            let summary = Self::analyze(axes, theme, scope)?;

            let _ = axes.intrinsics.set(summary);
        }
        let summary = axes.intrinsics.get().expect("initialized panel intrinsics");
        if summary.formats != axes.specs.each_ref().map(|spec| spec.format.clone())
            || summary.base_ranges != axes.ranges.map(|(lo, hi)| (lo.to_bits(), hi.to_bits()))
            || summary.logarithmic != axes.logarithmic
        {
            return Err(Error::Render(
                "prepared panel intrinsic numeric context changed".into(),
            ));
        }
        if summary.titles != axes.labels || !summary.standard.matches(theme)? {
            return Err(Error::Render(
                "prepared panel intrinsic typography changed".into(),
            ));
        }
        Ok(summary)
    }

    pub(crate) fn minimum(&self, plot: &Plot) -> (f64, f64) {
        (
            self.minimum_body.0,
            if plot.y_tick_labels_visible() {
                self.minimum_body.1
            } else {
                self.without_y_categories
            },
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one traversal owns title, category and formatted terminal intrinsics"
    )]
    fn analyze(
        axes: &crate::plots::shared_x::PreparedAxes,
        theme: &crate::theme::ResolvedTheme,
        diagnostic_scope: Option<&str>,
    ) -> Result<Self, Error> {
        #[cfg(not(test))]
        let _ = diagnostic_scope;
        #[cfg(test)]
        crate::orchestrator::measurements::intrinsic(
            crate::orchestrator::measurements::Counter::StandardPanelIntrinsic,
            diagnostic_scope,
        );
        let standard = StandardPanelInputs::new(theme)?;
        let mut minimum_body = (
            crate::layout::canvas::MINIMUM_PANEL_SPAN,
            crate::layout::canvas::MINIMUM_PANEL_SPAN,
        );
        let mut without_y_categories = minimum_body.1;
        for side in 0..2 {
            if let Some(label) = &axes.labels[side] {
                let (hard, _) = fit::intrinsic_widths(
                    theme,
                    label.as_str(),
                    label.appearance(&standard.axis_title),
                    "standard panel axis title intrinsic",
                )?;
                if side == 0 {
                    minimum_body.0 = minimum_body.0.max(hard);
                } else {
                    minimum_body.1 = minimum_body.1.max(hard);
                    without_y_categories = minimum_body.1;
                }
            }
            if let Some(labels) = &axes.display_categories[side] {
                let mut category_pitch = 0.0_f64;
                for label in labels {
                    let parallel = if side == 0 {
                        fit::intrinsic_widths(
                            theme,
                            label,
                            &standard.tick_label,
                            "coordinate panel tick label intrinsic",
                        )?
                        .0
                    } else {
                        // Mandatory lines and actual font metrics, not a
                        // nominal single-line height, constrain row slots.
                        theme
                            .measure_text(
                                label,
                                &standard.tick_label,
                                crate::layout::PhysicalTextAlignment::Center,
                                "coordinate panel tick label intrinsic",
                                false,
                            )?
                            .height()
                    };
                    category_pitch =
                        category_pitch.max(parallel + 2.0 * crate::layout::panel::LABEL_CLEARANCE);
                }
                let span = category_pitch
                    * f64::from(
                        u32::try_from(labels.len())
                            .map_err(|_| Error::Render("native category span overflow".into()))?,
                    );
                if side == 0 {
                    minimum_body.0 = minimum_body.0.max(span);
                } else {
                    minimum_body.1 = minimum_body.1.max(span);
                }
            } else if axes.specs[side].exact_ticks.is_none() {
                // The terminal source is derived from this scientific domain,
                // using the same indivisible numeric formatter as tick selection.
                let value = f64::midpoint(axes.ranges[side].0, axes.ranges[side].1);
                let power = axes.logarithmic && side == 0;
                let label = if let Some(probe) = axes.numeric_probe(side)? {
                    crate::data::source::copy_source_string(&probe.labels[1])?
                } else {
                    crate::layout::panel::format_label(value, power, false, 0)?
                };
                let plan = theme.measure_text(
                    &label,
                    &standard.tick_label,
                    crate::layout::PhysicalTextAlignment::Center,
                    "numeric terminal source intrinsic",
                    false,
                )?;
                let parallel = terminal_parallel_minimum(
                    &plan,
                    if side == 0 {
                        AxisSide::Bottom
                    } else {
                        AxisSide::Left
                    },
                )?;
                if side == 0 {
                    minimum_body.0 = minimum_body.0.max(parallel);
                } else {
                    minimum_body.1 = minimum_body.1.max(parallel);
                    without_y_categories = minimum_body.1;
                }
            }
        }
        Ok(Self {
            standard,
            titles: [axes.copy_label(0)?, axes.copy_label(1)?],
            formats: axes.specs.each_ref().map(|spec| spec.format.clone()),
            base_ranges: axes.ranges.map(|(lo, hi)| (lo.to_bits(), hi.to_bits())),
            logarithmic: axes.logarithmic,
            minimum_body,
            without_y_categories,
        })
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "text plans and their half extents retain canonical i64 CSS quanta"
)]
fn terminal_parallel_minimum(
    plan: &crate::layout::ResolvedTextPlan,
    side: AxisSide,
) -> Result<f64, Error> {
    use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
    let (length, half, anchor) = match side {
        AxisSide::Bottom => (plan.width(), plan.width_quanta, TextAnchor::Middle),
        AxisSide::Left => (plan.height(), plan.height_quanta, TextAnchor::End),
    };
    let half = crate::render::backend::nearest_even_text_half(half)
        .ok_or_else(|| Error::Render("numeric intrinsic half extent overflow".into()))?
        as f64
        / 1024.0;
    let (mut lower, mut upper) = match side {
        AxisSide::Bottom => (-half, length - half),
        AxisSide::Left => (half - length, half),
    };
    let placement = PlannedTextPlacement::middle(Point2D::new(0.0, 0.0), anchor, 0.0);
    if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
        plan,
        placement.position,
        placement.anchor,
        placement.rotation,
        placement.origin,
        placement.reverse_lines,
    ) {
        let (start, end) = match side {
            AxisSide::Bottom => (ink.left, ink.right),
            AxisSide::Left => (ink.bottom, ink.top),
        };
        lower = lower.min(start);
        upper = upper.max(end);
    }
    let parallel = match side {
        AxisSide::Bottom => (upper - lower).max(plan.fit_width()),
        AxisSide::Left => upper - lower,
    };
    crate::layout::kernel::required_length(parallel + 2.0 * crate::layout::panel::LABEL_CLEARANCE)
}

impl InteriorRoute {
    #[expect(
        clippy::too_many_lines,
        reason = "one native boundary freezes both sibling axes before projecting retained marks"
    )]
    pub(crate) fn measure(
        self,
        plot: &Plot,
        context: PlotRenderContext<'_>,
    ) -> Result<PlotRenderResult, PlanError> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(match self {
            Self::OneCategoricalAxis(_) => {
                crate::orchestrator::measurements::Counter::OneCategoricalAxisMeasure
            }
            _ => crate::orchestrator::measurements::Counter::StandardCoordinateMeasure,
        });
        self.validate(plot)?;
        let inputs = plot.prepared_axis_inputs().ok_or_else(|| {
            Error::Render("native coordinate panel requires root-prepared axes".into())
        })?;
        let valid = match self {
            Self::StandardCoordinate(family) => {
                !inputs.logarithmic
                    && inputs.categories[1].is_none()
                    && (inputs.categories[0].is_none() || family == StandardCoordinate::Trajectory)
            }
            Self::OneCategoricalAxis(family) => {
                // The prepared source owns orientation and midpoint order. New
                // categorical clusters reuse this boundary without rebuilding it.
                let category_side = match family {
                    OneCategoricalAxis::Bar
                    | OneCategoricalAxis::StackedBar
                    | OneCategoricalAxis::Boxplot
                    | OneCategoricalAxis::Violin
                    | OneCategoricalAxis::Raincloud
                    | OneCategoricalAxis::Interaction => 0,
                    OneCategoricalAxis::PointInterval
                    | OneCategoricalAxis::Coefficient
                    | OneCategoricalAxis::Forest
                    | OneCategoricalAxis::Dumbbell
                    | OneCategoricalAxis::HorizontalBar
                    | OneCategoricalAxis::StackedHorizontalBar => 1,
                };
                inputs.categories[category_side].is_some()
                    && inputs.categories[1 - category_side].is_none()
                    && (!inputs.logarithmic
                        || (family == OneCategoricalAxis::Forest && inputs.categories[0].is_none()))
            }
            Self::SpecializedGrid(_) => false,
        };
        if !valid {
            return Err(Error::Render("invalid native coordinate axis orientation".into()).into());
        }
        let minimum = plot.intrinsic_body_minimum(context.theme)?;
        let panel = context.panel;
        if ![panel.x0, panel.x1, panel.y0, panel.y1]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(Error::Render("native panel assignment must be finite".into()).into());
        }
        if panel.width() < minimum.0 || panel.height() < minimum.1 {
            return Err(context
                .fit_scope()
                .failure(
                    FitKind::TrackOrResidual,
                    "assigned coordinate panel is smaller than its foundational intrinsics",
                )
                .into());
        }
        let [x_range, y_range] = [AxisSide::Bottom, AxisSide::Left].map(|side| {
            let index = usize::from(side == AxisSide::Left);
            if inputs.categories[index].is_some() {
                Ok(inputs.ranges[index])
            } else {
                context.axis_range(side, || unreachable!("prepared native axes"))
            }
        });
        let projection = Projection::new(x_range?, y_range?, panel.as_tuple()).with_categories(
            inputs
                .categories
                .each_ref()
                .map(|ticks| ticks.as_ref().map(Vec::len)),
        );

        let mut keys = [None, None];
        let domains = inputs.categories.each_ref().map(|categories| {
            if categories.is_some() {
                AxisDomain::Categorical
            } else {
                AxisDomain::Continuous
            }
        });
        let mut axes = fit::collect(
            "native coordinate axes",
            (0..2).map(|side| {
                let supplied = if let Some(categories) = &inputs.categories[side] {
                    let display = inputs.display_categories[side].as_ref().ok_or_else(|| {
                        Error::Render("native category display source is missing".into())
                    })?;
                    if display.len() != categories.len() {
                        return Err(Error::Render(
                            "native category sources are not aligned".into(),
                        )
                        .into());
                    }
                    keys[side] =
                        Some(crate::data::source::collect_source(categories.iter().map(
                            |tick| crate::data::source::copy_source_string(&tick.label),
                        ))?);
                    crate::data::source::collect_source(categories.iter().zip(display).map(
                        |(tick, label)| {
                            Ok(crate::layout::AxisTick {
                                position: tick.position,
                                label: crate::data::source::copy_source_string(label)?,
                            })
                        },
                    ))?
                } else {
                    Vec::new()
                };
                let ticks = if side == 0
                    && inputs.logarithmic
                    && inputs.specs[side].exact_ticks.is_none()
                {
                    crate::plots::interval_estimate::log_tick_plan(
                        projection.x_range(),
                        inputs.specs[side].generation.clone(),
                        inputs.specs[side].numeric_request()?.as_ref(),
                        panel,
                        context.theme,
                        context.fit_scope(),
                    )?
                } else {
                    inputs.specs[side].layout_ticks(
                        supplied,
                        &projection,
                        if side == 0 {
                            AxisSide::Bottom
                        } else {
                            AxisSide::Left
                        },
                        context.theme,
                        context.fit_scope(),
                    )?
                };
                Ok(Axis {
                    label: inputs.copy_label(side)?,
                    ticks: ticks.ticks,
                    tick_labels_visible: side == 0 || plot.y_tick_labels_visible(),
                    explicit_ticks: inputs.specs[side].exact_ticks.is_some(),
                    assigned_tick_width: None,
                    assigned_cells: None,
                    source: ticks.source,
                    numeric: ticks.numeric,
                })
            }),
        )?;
        let y_axis = axes.pop().expect("two measured axes");
        let x_axis = axes.pop().expect("two measured axes");
        let layout = Layout {
            projection,
            x_axis,
            y_axis,
        };
        let frozen = layout.measure_axes_with_inputs(
            context.theme,
            context.fit_scope(),
            plot.axis_semantic_inputs(domains),
            || StandardPanelInputs::new(context.theme),
        )?;
        let mut result = match self {
            Self::StandardCoordinate(StandardCoordinate::Scatter) => {
                crate::plots::scatter::measure_marks(plot, context, layout)?
            }
            Self::StandardCoordinate(
                StandardCoordinate::Line
                | StandardCoordinate::Trajectory
                | StandardCoordinate::LinePath,
            ) => crate::plots::line::measure_marks(plot, context, layout)?,
            Self::StandardCoordinate(StandardCoordinate::Histogram) => {
                crate::plots::histogram::measure_marks(plot, context, layout)?
            }
            Self::StandardCoordinate(StandardCoordinate::Density) => {
                crate::plots::density::measure_marks(plot, context, layout)?
            }
            Self::StandardCoordinate(StandardCoordinate::Qq) => {
                crate::plots::qq::measure_marks(plot, context, layout)?
            }
            Self::StandardCoordinate(StandardCoordinate::Regression) => {
                crate::plots::regression::measure_marks(plot, context, layout)?
            }
            Self::OneCategoricalAxis(
                OneCategoricalAxis::PointInterval
                | OneCategoricalAxis::Coefficient
                | OneCategoricalAxis::Forest,
            ) => crate::plots::interval_estimate::measure_marks(plot, context, layout)?,
            Self::OneCategoricalAxis(OneCategoricalAxis::Dumbbell) => {
                crate::plots::dumbbell::measure_marks(plot, context, layout)?
            }
            Self::OneCategoricalAxis(OneCategoricalAxis::Bar | OneCategoricalAxis::StackedBar) => {
                crate::plots::bar::measure_marks(plot, context, layout)?
            }
            Self::OneCategoricalAxis(
                OneCategoricalAxis::HorizontalBar | OneCategoricalAxis::StackedHorizontalBar,
            ) => crate::plots::horizontal_bar::measure_marks(plot, context, layout)?,
            Self::OneCategoricalAxis(OneCategoricalAxis::Boxplot) => {
                crate::plots::boxplot::measure_marks(plot, context, layout)?
            }
            Self::OneCategoricalAxis(
                OneCategoricalAxis::Violin | OneCategoricalAxis::Raincloud,
            ) => crate::plots::violin::measure_marks(plot, context, layout)?,
            Self::OneCategoricalAxis(OneCategoricalAxis::Interaction) => {
                crate::plots::interaction::measure_marks(plot, context, layout)?
            }
            Self::SpecializedGrid(_) => {
                unreachable!("native route validated")
            }
        };
        result.axes = Some(frozen);
        [result.x_tick_keys, result.y_tick_keys] = keys;
        result.x_domain = domains[0];
        result.y_domain = domains[1];
        Ok(result)
    }
}

/// Exact standalone Plot frame, using the same carrier as constrained panels.
/// Facet/Canvas track allocation remains with the existing parent adapters.
pub(crate) fn canvas_layout(
    plot: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<crate::layout::RootFramePlan, PlanError> {
    use crate::decorations::title_block::plan::RootAlignments;
    let minimum = plot.intrinsic_body_minimum(theme)?;
    // Only root-owned padding and text are hard reservations. Axis gutters are
    // measured from selected sources at the assigned span by the root round;
    // endpoint estimates must not reject a body before tick selection.
    let mut root = crate::layout::RootFramePlan::measure(
        plot.canvas_dimensions()?,
        theme,
        plot.prose(),
        RootAlignments::resolve(theme, plot.prose_alignments(), [None; 2]),
        minimum,
    )?;
    root.native_preference = match super::InteriorRoute::classify(plot.kind()) {
        super::InteriorRoute::StandardCoordinate(_) => {
            Some(super::preference::NativePanelPreference::Standard(
                super::preference::mark_preference(plot, theme, minimum)?,
            ))
        }
        super::InteriorRoute::OneCategoricalAxis(
            super::OneCategoricalAxis::PointInterval
            | super::OneCategoricalAxis::Coefficient
            | super::OneCategoricalAxis::Forest
            | super::OneCategoricalAxis::Dumbbell
            | super::OneCategoricalAxis::Bar
            | super::OneCategoricalAxis::HorizontalBar
            | super::OneCategoricalAxis::StackedBar
            | super::OneCategoricalAxis::StackedHorizontalBar
            | super::OneCategoricalAxis::Boxplot
            | super::OneCategoricalAxis::Violin
            | super::OneCategoricalAxis::Raincloud
            | super::OneCategoricalAxis::Interaction,
        ) => Some(super::preference::NativePanelPreference::Categorical(
            super::preference::categorical_preference(plot, theme, minimum)?,
        )),
        _ => None,
    };
    Ok(root)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod intrinsic_tests {
    use super::*;

    #[test]
    fn terminal_sources_fit_the_traced_parallel_minimum_including_tight_vertical_ink() {
        for tight_lines in [false, true] {
            let source = Plot::scatter(
                crate::NumericValues::new("x", [-1.0, 1.0]),
                crate::NumericValues::new("y", [-1.0, 1.0]),
            )
            .x_limits(-1.0, 1.0)
            .y_limits(-1.0, 1.0)
            .x_label("")
            .y_label("")
            .erase();
            let resources = crate::font::canonical::OperationContext::new();
            let chart = crate::plots::shared_x::prepare_plot(&source, &resources, None)
                .unwrap()
                .attach(source)
                .unwrap();
            let mut theme = crate::Theme::publication().resolve().unwrap();
            if tight_lines {
                theme.typography.axes.tick_label.line_height_multiplier = 0.1;
            }
            let minimum = Intrinsics::prepared(&chart, &theme)
                .unwrap()
                .minimum(&chart);
            let projection = Projection::new(
                (-1.0, 1.0),
                (-1.0, 1.0),
                (0.25, 0.25 + minimum.0, 0.25, 0.25 + minimum.1),
            );
            for side in [AxisSide::Bottom, AxisSide::Left] {
                let selected = crate::layout::numeric_tick_plan(
                    (-1.0, 1.0),
                    crate::TickGeneration::automatic(),
                    false,
                    &projection,
                    side,
                    &theme,
                    false,
                    None,
                    &fit::ROOT_SCOPE,
                )
                .unwrap();
                assert_eq!(selected.ticks.len(), 1);
                assert_eq!(selected.ticks[0].label, "0");
            }
            assert_eq!(chart.user_x_limits(), Some((-1.0, 1.0)));
            assert_eq!(chart.user_y_limits(), Some((-1.0, 1.0)));
        }
    }

    #[test]
    fn numeric_intrinsic_uses_the_actual_indivisible_terminal_domain_source() {
        use crate::numeric::{
            decimal::NumericNotation,
            format::{NumericFormat, NumericPrecision},
        };
        for (range, label) in [((-1.0, 1.0), "[0.00e0]"), ((1e100, 3e100), "[2.00e100]")] {
            let source = Plot::scatter(
                crate::NumericValues::new("x", [range.0, range.1]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .x_limits(range.0, range.1)
            .x_label("")
            .y_label("")
            .x_tick_format(
                NumericFormat::new(
                    NumericNotation::Scientific,
                    NumericPrecision::AdaptiveFractional { min: 2, max: 4 },
                )
                .unwrap()
                .prefix("[")
                .unwrap()
                .suffix("]")
                .unwrap(),
            )
            .y_tick_format(
                NumericFormat::new(
                    NumericNotation::Decimal,
                    NumericPrecision::AdaptiveFractional { min: 0, max: 17 },
                )
                .unwrap(),
            )
            .erase();
            let resources = crate::font::canonical::OperationContext::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
            let chart = inputs.attach(source).unwrap();
            let theme = crate::Theme::publication().resolve().unwrap();
            let measured = theme
                .measure_text(
                    label,
                    &theme.typography.axes.tick_label,
                    crate::layout::PhysicalTextAlignment::Center,
                    "independent numeric terminal source",
                    false,
                )
                .unwrap();
            let minimum = Intrinsics::prepared(&chart, &theme)
                .unwrap()
                .minimum(&chart);
            assert_eq!(
                minimum.0.to_bits(),
                crate::layout::kernel::required_length(
                    measured.width().max(measured.fit_width())
                        + 2.0 * crate::layout::panel::LABEL_CLEARANCE,
                )
                .unwrap()
                .to_bits()
            );
            assert_eq!(chart.prepared_axis_inputs().unwrap().ranges[0], range);
            let widest_probe = theme
                .measure_text(
                    "0.50000000000000000",
                    &theme.typography.axes.tick_label,
                    crate::layout::PhysicalTextAlignment::End,
                    "independent maximum-precision gutter probe",
                    false,
                )
                .unwrap();
            assert!(
                crate::layout::interior::sizing::horizontal_gutter_preference(&chart, &theme)
                    .unwrap()
                    .0
                    >= widest_probe.width()
            );
        }
    }
}
