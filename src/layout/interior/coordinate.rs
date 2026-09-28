//! Native coordinate panels. Parents allocate the frame; this common producer
//! selects its axes once and hands the resulting projection to retained marks.

use super::{DeferredLegacyPanelMinimum, InteriorRoute, OneCategoricalAxis, StandardCoordinate};
use crate::layout::fit::{self, FitKind, PlanError};
use crate::layout::{Axis, AxisSide, Layout, PanelBox, Projection, StandardPanelInputs};
use crate::spec::{AxisDomain, PlotRenderContext, PlotRenderResult};
use crate::{Error, Plot};

/// Only foundational text contributes new hard requirements here. Mark density
/// and exact mark/annotation intrinsics remain T5c, including the named floor.
#[derive(Debug)]
pub(crate) struct Intrinsics {
    standard: StandardPanelInputs,
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
        let environment = theme.prepared_font_environment()?;
        environment.claim_measurement_call().map_err(|error| {
            Error::Render(format!("coordinate intrinsic call admission: {error:?}"))
        })?;
        if axes.intrinsics.get().is_none() {
            let summary = Self::analyze(axes, theme, scope)?;
            environment.claim_cache_entries(1).map_err(|error| {
                Error::Render(format!("coordinate intrinsic cache admission: {error:?}"))
            })?;
            let _ = axes.intrinsics.set(summary);
        }
        let summary = axes.intrinsics.get().expect("initialized panel intrinsics");
        if !summary.standard.matches(theme) {
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

    fn analyze(
        axes: &crate::plots::shared_x::PreparedAxes,
        theme: &crate::theme::ResolvedTheme,
        scope: Option<&str>,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        crate::orchestrator::measurements::intrinsic(
            crate::orchestrator::measurements::Counter::StandardPanelIntrinsic,
            scope,
        );
        let standard = StandardPanelInputs::new(theme)?;
        let mut minimum_body = DeferredLegacyPanelMinimum::SIZE;
        let mut without_y_categories = minimum_body.1;
        for side in 0..2 {
            if !axes.labels[side].is_empty() {
                let (hard, _) = fit::intrinsic_widths(
                    theme,
                    &axes.labels[side],
                    &standard.axis_title,
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
                theme
                    .prepared_font_environment()?
                    .claim_processing_visits(labels.len() as u128)
                    .map_err(|error| {
                        Error::Render(format!("coordinate intrinsic labels {scope:?}: {error:?}"))
                    })?;
                let mut category_pitch = 0.0_f64;
                for label in labels {
                    if !label.is_empty() {
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
                        category_pitch = category_pitch
                            .max(parallel + 2.0 * crate::layout::panel::LABEL_CLEARANCE);
                    }
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
            }
        }
        Ok(Self {
            standard,
            minimum_body,
            without_y_categories,
        })
    }
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
            Self::SpecializedLegacy(_) => false,
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
        let resources = context
            .theme
            .prepared_font_environment()?
            .generation_resources();
        let data = context.data.with_axis_resources(&resources);
        let mut keys = [None, None];
        let domains = inputs.categories.each_ref().map(|categories| {
            if categories.is_some() {
                AxisDomain::Categorical
            } else {
                AxisDomain::Continuous
            }
        });
        let mut axes = fit::collect(
            context.theme,
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
                    keys[side] = Some(
                        data.collect_source(
                            categories
                                .iter()
                                .map(|tick| data.copy_source_string(&tick.label)),
                        )?,
                    );
                    data.collect_source(categories.iter().zip(display).map(|(tick, label)| {
                        Ok(crate::layout::AxisTick {
                            position: tick.position,
                            label: if side == 1 && !plot.y_tick_labels_visible() {
                                String::new()
                            } else {
                                data.copy_source_string(label)?
                            },
                        })
                    }))?
                } else {
                    Vec::new()
                };
                let ticks = if side == 0
                    && inputs.logarithmic
                    && inputs.specs[side].exact_ticks.is_none()
                {
                    crate::plots::interval_estimate::log_tick_plan(
                        projection.x_range(),
                        inputs.specs[side].interval,
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
                    label: data.copy_source_string(&inputs.labels[side])?,
                    ticks: ticks.ticks,
                    explicit_ticks: inputs.specs[side].exact_ticks.is_some(),
                    assigned_tick_width: None,
                    source: ticks.source,
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
            Self::SpecializedLegacy(_) => {
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
    use crate::decorations::title_block::plan::{RootAlignments, RootTitleBlockPlan};
    use crate::layout::canvas::CanvasInsets;
    let minimum = plot.intrinsic_body_minimum(theme)?;
    // Only root-owned padding and text are hard reservations. Axis gutters are
    // measured from selected sources at the assigned span by the root round;
    // endpoint estimates must not reject a body before tick selection.
    let mut insets = CanvasInsets {
        left: 16.0,
        right: 16.0,
        bottom: 8.0,
        top: 12.0,
    };
    let (width, height) = (f64::from(plot.width()), f64::from(plot.height()));
    let text = RootTitleBlockPlan::measure(
        theme,
        [plot.title_text(), plot.subtitle_text(), plot.caption_text()],
        PanelBox::new(16.0, width - 16.0, 8.0, height - 12.0),
        RootAlignments::resolve(
            theme,
            [plot.title_alignment, plot.subtitle_alignment],
            [None; 2],
        ),
        minimum.1,
        &fit::FitScope::root(),
    )?;
    insets.top += text.top_reserve;
    insets.bottom += text.bottom_reserve;
    let panel_box = PanelBox::new(
        insets.left,
        width - insets.right,
        insets.bottom,
        height - insets.top,
    );
    fit::measured(
        crate::layout::kernel::FrameRequirements {
            minimum_body: minimum,
            natural_insets: [0.0; 4],
        }
        .assign_fit(panel_box, [0.0; 4], &fit::FitScope::root()),
    )?;
    Ok(crate::layout::RootFramePlan {
        panel_box,
        insets,
        root_text: text,
        native_preference: match super::InteriorRoute::classify(plot.kind()) {
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
        },
    })
}

#[cfg(test)]
mod tests;
