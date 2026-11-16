use super::{Axis, AxisSide, AxisTick, Layout, NumericTickPlan, Projection, TickSource};
use crate::layout::PixelRect;
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
