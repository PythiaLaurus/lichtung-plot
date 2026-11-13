use super::range_ticks::resolve_range;
use super::{Axis, AxisSide, AxisSpec, AxisTick, Layout, NumericTickPlan, Projection, TickSource};
use crate::layout::fit::PlanError;
use crate::layout::PixelRect;
#[cfg(test)]
use crate::render::backend::Backend;
use crate::Error;

impl Layout {
    /// Retain the actual assigned containing box of a self-decorating Grid.
    pub(crate) fn envelope(panel: PixelRect) -> Self {
        Self {
            x_axis: Axis {
                label: None,
                ticks: Vec::new(),
                tick_labels_visible: true,
                category_labels: crate::CategoryLabelSelection::all(),
                explicit_ticks: false,
                assigned_tick_width: None,
                assigned_cells: None,
                source: TickSource::SUPPLIED,
                numeric: None,
            },
            y_axis: Axis {
                label: None,
                ticks: Vec::new(),
                tick_labels_visible: true,
                category_labels: crate::CategoryLabelSelection::all(),
                explicit_ticks: false,
                assigned_tick_width: None,
                assigned_cells: None,
                source: TickSource::SUPPLIED,
                numeric: None,
            },
            projection: Projection::new((0.0, 1.0), (0.0, 1.0), panel.as_tuple()),
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
        x_label: Option<crate::Text>,
        y_label: Option<crate::Text>,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
        theme: &crate::text::TextRequestContext<'_, '_>,
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
        panel_box: PixelRect,
        x_label: Option<crate::Text>,
        y_label: Option<crate::Text>,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
        theme: &crate::text::TextRequestContext<'_, '_>,
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
            None,
            [None; 2],
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    /// Create a new layout with explicit ticks and axis specs inside an explicit panel box.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_ticks_and_axis_specs_in_panel_box(
        x_range: (f64, f64),
        y_range: (f64, f64),
        panel_box: PixelRect,
        x_label: Option<crate::Text>,
        y_label: Option<crate::Text>,
        x_ticks: Vec<AxisTick>,
        y_ticks: Vec<AxisTick>,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
        theme: &crate::text::TextRequestContext<'_, '_>,
        scope: &crate::layout::fit::FitScope,
        binding: Option<crate::numeric::source::NumericBinding<'_>>,
        raw_windows: [Option<&crate::layout::RawWindow>; 2],
    ) -> Result<Self, crate::layout::fit::PlanError> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::LegacyPanelConstructor,
        );
        let pixel_box = panel_box.as_tuple();

        // Don't expand categorical x ranges — the user picked
        // them deliberately.
        let x_range = if x_ticks.is_empty() || x_spec.exact_ticks.is_some() {
            resolve_range(x_range, x_spec)?
        } else if let Some(limits) = x_spec.window.map(crate::layout::RawWindow::pair) {
            limits
        } else {
            x_range
        };
        let y_range = if y_ticks.is_empty() || y_spec.exact_ticks.is_some() {
            resolve_range(y_range, y_spec)?
        } else if let Some(limits) = y_spec.window.map(crate::layout::RawWindow::pair) {
            limits
        } else {
            y_range
        };
        let projection = Projection::new(x_range, y_range, pixel_box);

        let x_ticks = x_spec.layout_ticks(
            x_ticks,
            &projection,
            AxisSide::Bottom,
            theme,
            scope,
            binding,
            raw_windows,
        );
        let y_ticks = y_spec.layout_ticks(
            y_ticks,
            &projection,
            AxisSide::Left,
            theme,
            scope,
            binding,
            raw_windows,
        );
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

        let categories = [&x_ticks, &y_ticks]
            .map(|ticks| (ticks.source == TickSource::SUPPLIED).then_some(ticks.ticks.len()));
        Ok(Self {
            projection: projection.with_categories(categories),
            x_axis: Axis {
                label: x_label,
                ticks: x_ticks.ticks,
                tick_labels_visible: true,
                category_labels: x_spec.category_labels.clone(),
                explicit_ticks: x_spec.exact_ticks.is_some(),
                assigned_tick_width: None,
                assigned_cells: None,
                source: x_ticks.source,
                numeric: x_ticks.numeric,
            },
            y_axis: Axis {
                label: y_label,
                ticks: y_ticks.ticks,
                tick_labels_visible: true,
                category_labels: y_spec.category_labels.clone(),
                explicit_ticks: y_spec.exact_ticks.is_some(),
                assigned_tick_width: None,
                assigned_cells: None,
                source: y_ticks.source,
                numeric: y_ticks.numeric,
            },
        })
    }

    /// Draw frame and axes ON TOP of data (called after data rendering).
    /// Grid is drawn separately from the retained panel plan BEFORE data.
    #[cfg(test)]
    pub fn draw_decorations(
        &self,
        theme: &crate::text::TextRequestContext<'_, '_>,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.plan_axes(theme)?.draw(backend)
    }
}

impl AxisSpec {
    pub(crate) fn numeric_request<'v>(
        &'v self,
        binding: Option<crate::numeric::source::NumericBinding<'v>>,
        raw_windows: [Option<&'v crate::layout::RawWindow>; 2],
    ) -> Result<Option<crate::numeric::source::AxisRequest<'v>>, Error> {
        let Some(binding) = binding else {
            if self.format.is_some() || self.generation.position_function().is_some() {
                return Err(Error::Render(
                    "numeric axis lacks its operation-bound owner".into(),
                ));
            }
            return Ok(None);
        };
        Ok(Some(crate::numeric::source::AxisRequest {
            format: self.format.as_deref(),
            binding,
            raw_windows,
            provenance: self.provenance,
            generation: self.generation.clone(),
            source_indices: None,
        }))
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "numeric sources borrow the operation and raw windows only for this selection"
    )]
    pub(crate) fn layout_ticks(
        &self,
        supplied: Vec<AxisTick>,
        projection: &Projection,
        side: AxisSide,
        theme: &crate::text::TextRequestContext<'_, '_>,
        scope: &crate::layout::fit::FitScope,
        binding: Option<crate::numeric::source::NumericBinding<'_>>,
        raw_windows: [Option<&crate::layout::RawWindow>; 2],
    ) -> Result<NumericTickPlan, crate::layout::fit::PlanError> {
        layout_axis_ticks(
            self.exact_ticks.as_deref().map(|raw| {
                crate::orchestrator::registration::ExactTicksRef {
                    raw,
                    projected: None,
                }
            }),
            &self.generation,
            supplied,
            projection,
            side,
            theme,
            scope,
            || self.numeric_request(binding, raw_windows),
        )
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "raw and prepared exact positions share the original tick selection program"
)]
pub(crate) fn layout_axis_ticks<'a>(
    exact: Option<crate::orchestrator::registration::ExactTicksRef<'_>>,
    generation: &crate::TickGeneration,
    supplied: Vec<AxisTick>,
    projection: &Projection,
    side: AxisSide,
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &crate::layout::fit::FitScope,
    numeric: impl FnOnce() -> Result<Option<crate::numeric::source::AxisRequest<'a>>, Error>,
) -> Result<NumericTickPlan, crate::layout::fit::PlanError> {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AxisSourcePreparation,
    );
    let (range, axis) = match side {
        AxisSide::Bottom => (projection.x_range(), "x-axis"),
        AxisSide::Left => (projection.y_range(), "y-axis"),
    };
    if let Some(ticks) = exact {
        Axis::validate_exact_ticks(&ticks, range, axis)?;

        let mut retained = Vec::new();
        crate::render::backend::reserve_text_records(&mut retained, ticks.raw.len())?;
        for index in 0..ticks.raw.len() {
            let (position, source) = ticks.tick(index)?;
            let mut label = String::new();
            label
                .try_reserve_exact(source.len())
                .map_err(|error| Error::Render(error.to_string()))?;
            label.push_str(source);
            retained.push(AxisTick::new(position, label));
        }
        return Ok(NumericTickPlan {
            ticks: retained,
            source: TickSource::EXACT,
            numeric: None,
        });
    }
    if supplied.is_empty() {
        super::numeric_tick_plan(
            range,
            generation.clone(),
            false,
            projection,
            side,
            theme,
            numeric()?.as_ref(),
            scope,
        )
    } else {
        Ok(NumericTickPlan {
            ticks: supplied,
            source: TickSource::SUPPLIED,
            numeric: None,
        })
    }
}

#[cfg(test)]
fn default_panel_box(canvas_width: u32, canvas_height: u32) -> PixelRect {
    PixelRect::new(
        80.0,
        f64::from(canvas_width) - 30.0,
        60.0,
        f64::from(canvas_height) - 30.0,
    )
}
