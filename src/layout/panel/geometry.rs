use super::range_ticks::resolve_range;
use super::{Axis, AxisSide, AxisSpec, AxisTick, Layout, NumericTickPlan, Projection, TickSource};
use crate::layout::fit::PlanError;
use crate::layout::PanelBox;
#[cfg(test)]
use crate::render::backend::Backend;
use crate::theme::ResolvedTheme;
use crate::Error;

impl Layout {
    /// Build the non-rendered carrier returned by a self-decorating plot.
    pub(crate) fn self_managed(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            x_axis: Axis {
                label: String::new(),
                ticks: Vec::new(),
                explicit_ticks: false,
                assigned_tick_width: None,
                source: TickSource::SUPPLIED,
            },
            y_axis: Axis {
                label: String::new(),
                ticks: Vec::new(),
                explicit_ticks: false,
                assigned_tick_width: None,
                source: TickSource::SUPPLIED,
            },
            projection: Projection {
                x_range: (0.0, 1.0),
                y_range: (0.0, 1.0),
                pixel_box: default_panel_box(canvas_width, canvas_height).as_tuple(),
            },
        }
    }

    /// Create a new layout using explicit axis specs for limits.
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_axis_specs(
        x_range: (f64, f64),
        y_range: (f64, f64),
        canvas_width: u32,
        canvas_height: u32,
        x_label: String,
        y_label: String,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
        theme: &ResolvedTheme,
    ) -> Result<Self, Error> {
        // Hardcoded academic-style margins for now.
        let pixel_box = default_panel_box(canvas_width, canvas_height);

        Self::new_with_axis_specs_in_panel_box(
            x_range, y_range, pixel_box, x_label, y_label, x_spec, y_spec, theme,
        )
    }

    /// Create a new numeric layout inside an explicit panel box.
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_axis_specs_in_panel_box(
        x_range: (f64, f64),
        y_range: (f64, f64),
        panel_box: PanelBox,
        x_label: String,
        y_label: String,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
        theme: &ResolvedTheme,
    ) -> Result<Self, Error> {
        Self::new_with_ticks_and_axis_specs_in_panel_box(
            x_range,
            y_range,
            panel_box,
            x_label,
            y_label,
            Vec::new(),
            Vec::new(),
            x_spec,
            y_spec,
            theme,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    /// Create a new layout with explicit ticks and axis specs inside an explicit panel box.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_ticks_and_axis_specs_in_panel_box(
        x_range: (f64, f64),
        y_range: (f64, f64),
        panel_box: PanelBox,
        x_label: String,
        y_label: String,
        x_ticks: Vec<AxisTick>,
        y_ticks: Vec<AxisTick>,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::LegacyPanelConstructor,
        );
        let pixel_box = panel_box.as_tuple();

        // Don't expand categorical x ranges — the user picked
        // them deliberately.
        let x_range = if x_ticks.is_empty() || x_spec.exact_ticks.is_some() {
            resolve_range(x_range, x_spec)
        } else if let Some(limits) = x_spec.limits {
            limits
        } else {
            x_range
        };
        let y_range = if y_ticks.is_empty() || y_spec.exact_ticks.is_some() {
            resolve_range(y_range, y_spec)
        } else if let Some(limits) = y_spec.limits {
            limits
        } else {
            y_range
        };
        let projection = Projection {
            x_range,
            y_range,
            pixel_box,
        };

        let x_ticks = x_spec.layout_ticks(x_ticks, &projection, AxisSide::Bottom, theme, scope);
        let y_ticks = y_spec.layout_ticks(y_ticks, &projection, AxisSide::Left, theme, scope);
        // A later fatal axis error must not be hidden by an earlier geometric miss.
        let (x_ticks, y_ticks) = match (x_ticks, y_ticks) {
            (Err(error @ PlanError::Fatal(_)), _)
            | (_, Err(error @ PlanError::Fatal(_)))
            | (Err(error), Ok(_))
            | (Ok(_), Err(error)) => return Err(error),
            (Err(PlanError::Fit(x)), Err(PlanError::Fit(y))) => {
                return Err(PlanError::Fit(if x.reason <= y.reason { x } else { y }));
            }
            (Ok(x), Ok(y)) => (x, y),
        };

        Ok(Self {
            projection,
            x_axis: Axis {
                label: x_label,
                ticks: x_ticks.ticks,
                explicit_ticks: x_spec.exact_ticks.is_some(),
                assigned_tick_width: None,
                source: x_ticks.source,
            },
            y_axis: Axis {
                label: y_label,
                ticks: y_ticks.ticks,
                explicit_ticks: y_spec.exact_ticks.is_some(),
                assigned_tick_width: None,
                source: y_ticks.source,
            },
        })
    }

    /// Draw frame and axes ON TOP of data (called after data rendering).
    /// Grid is drawn separately from the retained panel plan BEFORE data.
    #[cfg(test)]
    pub fn draw_decorations(
        &self,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.plan_axes(theme)?.draw(backend)
    }
}

impl AxisSpec {
    pub(crate) fn layout_ticks(
        &self,
        supplied: Vec<AxisTick>,
        projection: &Projection,
        side: AxisSide,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<NumericTickPlan, crate::layout::fit::PlanError> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::AxisSourcePreparation,
        );
        let (range, axis) = match side {
            AxisSide::Bottom => (projection.x_range(), "x-axis"),
            AxisSide::Left => (projection.y_range(), "y-axis"),
        };
        if let Some(ticks) = &self.exact_ticks {
            Axis::validate_ticks(ticks, range, axis)?;
            let environment = theme.prepared_font_environment()?;
            let mut retained = Vec::new();
            crate::render::backend::reserve_text_records(&mut retained, ticks.len(), environment)?;
            for tick in ticks {
                environment
                    .claim_unicode_bytes(tick.label.len() as u64)
                    .map_err(|error| Error::Render(error.to_string()))?;
                let mut label = String::new();
                label
                    .try_reserve_exact(tick.label.len())
                    .map_err(|error| Error::Render(error.to_string()))?;
                label.push_str(&tick.label);
                retained.push(AxisTick::new(tick.position, label));
            }
            return Ok(NumericTickPlan {
                ticks: retained,
                source: TickSource::EXACT,
            });
        }
        if supplied.is_empty() {
            super::numeric_tick_plan(
                range,
                self.interval,
                false,
                projection,
                side,
                theme,
                false,
                scope,
            )
        } else {
            Ok(NumericTickPlan {
                ticks: supplied,
                source: TickSource::SUPPLIED,
            })
        }
    }
}

fn default_panel_box(canvas_width: u32, canvas_height: u32) -> PanelBox {
    PanelBox::new(
        80.0,
        f64::from(canvas_width) - 30.0,
        60.0,
        f64::from(canvas_height) - 30.0,
    )
}

pub(crate) fn y_axis_left_inset_for_tick_labels(
    tick_label_width: f64,
    tick_length: u32,
    label_font_size: u32,
    spacing: &crate::theme::AxisSpacing,
) -> f64 {
    spacing.y_title_edge_padding
        + f64::from(label_font_size)
        + spacing.y_title_tick_label_gap
        + tick_label_width
        + f64::from(tick_length)
        + spacing.y_tick_label_gap
}
