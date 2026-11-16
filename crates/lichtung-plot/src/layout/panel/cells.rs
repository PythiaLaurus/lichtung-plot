//! Assigned axis-parallel cells, shared by source fit and frozen axis geometry.

use super::projection::{axis_quanta, projection_key, ratio_position};
use super::{AxisSide, AxisTick, Projection};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::kernel::nearest_spatial_quanta;
use crate::layout::ResolvedTextPlan;
use crate::render::backend::{PlannedTextPlacement, Point2D};
use crate::render::coordinates::{canonical_text, CoordinatePrecision};
use crate::Error;

// Split the existing 4px label separation across the two neighboring cells.
pub(crate) const LABEL_CLEARANCE: f64 = 2.0;

/// Uniform-category hard span from selected neighbors; full scientific count stays intact.
pub(crate) fn category_label_span(
    selection: &crate::CategoryLabelSelection,
    count: usize,
    mut extent: impl FnMut(usize) -> Result<f64, Error>,
) -> Result<f64, Error> {
    let count_pixels = f64::from(u32::try_from(count).map_err(|_| {
        Error::Render("category label span count exceeds spatial representation".into())
    })?);
    let indices = crate::data::source::collect_source(selection.indices(count).map(Ok))?;
    let mut span = 0.0_f64;
    for (ordinal, &index) in indices.iter().enumerate() {
        let index_pixels = f64::from(u32::try_from(index).expect("index below checked count"));
        let left = if ordinal == 0 {
            index_pixels + 0.5
        } else {
            (index - indices[ordinal - 1]) as f64 / 2.0
        };
        let right = if ordinal + 1 == indices.len() {
            count_pixels - index_pixels - 0.5
        } else {
            (indices[ordinal + 1] - index) as f64 / 2.0
        };
        span = span.max(extent(index)? * count_pixels / (2.0 * left.min(right)));
    }
    crate::layout::kernel::required_length(span)
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LabelCell {
    pub(super) center: f64,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) inward: bool,
}

impl LabelCell {
    pub(super) fn capacity(self) -> f64 {
        self.end - self.start
    }

    pub(super) fn local(
        plan: &ResolvedTextPlan,
        side: AxisSide,
    ) -> Result<PlannedTextPlacement, Error> {
        let requested = plan.request.as_ref().map(crate::TextLayout::instructions);
        let anchor = requested
            .and_then(|request| request.anchor)
            .unwrap_or_else(|| match side {
                AxisSide::Bottom => crate::TextAnchor::center(),
                AxisSide::Left => crate::TextAnchor::center_right(),
            });
        crate::layout::text::placement::at(
            plan,
            Point2D::new(0.0, 0.0),
            anchor,
            0.0,
            false,
            requested.and_then(|request| request.offset),
        )
    }

    /// Source selection and frozen labels use the same canonical anchor and ink.
    pub(super) fn placement(
        self,
        plan: &ResolvedTextPlan,
        side: AxisSide,
        perpendicular: f64,
    ) -> Result<Option<PlannedTextPlacement>, Error> {
        let requested = plan.request.as_ref().map(crate::TextLayout::instructions);
        let mut placement = Self::local(plan, side)?;
        let allow = requested.is_some_and(|request| {
            request.overlap == Some(crate::TextOverlap::Allow)
                || request
                    .visibility
                    .as_ref()
                    .is_some_and(crate::TextVisibility::reduces)
                || request
                    .overflow
                    .as_ref()
                    .is_some_and(|overflow| overflow.failure != crate::TextFailure::Error)
        });
        let bounds = crate::layout::text::placement::footprint(placement, plan)?;
        let (lower, upper) = match side {
            AxisSide::Bottom => (bounds.x0, bounds.x1),
            AxisSide::Left => (bounds.y0, bounds.y1),
        };
        let lower = crate::layout::kernel::ceil_spatial_quanta(-lower)?
            .checked_neg()
            .ok_or_else(|| Error::Render("axis label lower bound overflow".into()))?;
        let upper = crate::layout::kernel::ceil_spatial_quanta(upper)?;
        let minimum = i128::from(nearest_spatial_quanta(self.start)?) - i128::from(lower);
        let maximum = i128::from(nearest_spatial_quanta(self.end)?) - i128::from(upper);
        if !allow && minimum > maximum {
            return Ok(None);
        }
        let mut center = i128::from(nearest_spatial_quanta(self.center)?);
        if self.inward && !allow {
            center = center.clamp(minimum, maximum);
        }
        if !allow && (center < minimum || center > maximum) {
            return Ok(None);
        }
        let center = i64::try_from(center)
            .map_err(|_| Error::Render("axis label anchor overflow".into()))?
            as f64
            / 1024.0;
        placement.position = match side {
            AxisSide::Bottom => Point2D::new(
                placement.position.x + center,
                placement.position.y + perpendicular,
            ),
            AxisSide::Left => Point2D::new(
                placement.position.x + perpendicular,
                placement.position.y + center,
            ),
        };
        placement.position = canonical_text(
            placement.position,
            placement.rotation,
            CoordinatePrecision::CssQuantum,
        )?
        .position
        .value();
        Ok(Some(placement))
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "label eligibility is independent of full tick geometry and assigned tracks"
)]
pub(super) fn assign(
    ticks: &[AxisTick],
    categorical: bool,
    labels_visible: bool,
    assigned: Option<&[(f64, f64)]>,
    selection: &crate::CategoryLabelSelection,
    projection: &Projection,
    side: AxisSide,

    scope: &FitScope,
) -> Result<Vec<LabelCell>, PlanError> {
    let panel = projection.panel_box();
    let (origin, end, range) = match side {
        AxisSide::Bottom => (panel.x0, panel.x1, projection.x_range()),
        AxisSide::Left => (panel.y0, panel.y1, projection.y_range()),
    };
    let (origin, span) = axis_quanta(origin, end)?;
    let mut cells = Vec::new();
    crate::layout::fit::reserve(&mut cells, ticks.len(), "axis label cells")?;
    for tick in ticks {
        let key = projection_key(tick.position, range).ok_or_else(|| {
            scope.failure(
                FitKind::Tick,
                "axis position is outside its finite display domain",
            )
        })?;
        let (center, start, end) = if let Some(bounds) = assigned {
            assigned_bounds(bounds, cells.len(), categorical)?
        } else if categorical {
            let count = ticks.len() as u128;
            let index =
                (u128::from(key) * count / u128::from(u64::MAX)).min(count.saturating_sub(1));
            let rounded = |numerator: u128, denominator: u128| -> Result<f64, Error> {
                let value = ratio_position(origin, span, numerator, denominator)
                    .ok_or_else(|| Error::Render("category cell overflow".into()))?;
                Ok(value as f64 / 1024.0)
            };
            (
                rounded(2 * index + 1, 2 * count)?,
                rounded(index, count)?,
                rounded(index + 1, count)?,
            )
        } else {
            let center = ratio_position(origin, span, u128::from(key), u128::from(u64::MAX))
                .ok_or_else(|| Error::Render("axis center overflow".into()))?;
            (
                center as f64 / 1024.0,
                origin as f64 / 1024.0,
                (i128::from(origin) + i128::from(span)) as f64 / 1024.0,
            )
        };
        cells.push(LabelCell {
            center,
            start,
            end,
            inward: false,
        });
    }
    if assigned.is_some_and(|bounds| bounds.len() != ticks.len()) {
        return Err(
            Error::Render("assigned categorical cell count differs from ticks".into()).into(),
        );
    }
    let sparse = categorical && !selection.is_all();
    if !categorical || sparse {
        if sparse {
            for cell in &mut cells {
                cell.start = origin as f64 / 1024.0;
                cell.end = (i128::from(origin) + i128::from(span)) as f64 / 1024.0;
            }
        }
        neighbor_boundaries(
            &mut cells,
            ticks,
            categorical,
            selection,
            labels_visible,
            scope,
        )?;
    }
    if labels_visible {
        apply_clearance(&mut cells, ticks, categorical, selection, scope)?;
    }
    Ok(cells)
}

fn neighbor_boundaries(
    cells: &mut [LabelCell],
    ticks: &[AxisTick],
    categorical: bool,
    selection: &crate::CategoryLabelSelection,
    labels_visible: bool,
    scope: &FitScope,
) -> Result<(), PlanError> {
    let mut order = crate::data::source::collect_source((0..ticks.len()).map(Ok))?;
    order.retain(|&index| {
        if categorical {
            selection.includes(index, ticks.len())
        } else {
            !ticks[index].label.is_empty()
        }
    });
    crate::data::source::sort_source_by(&mut order, |a, b| {
        cells[*a]
            .center
            .total_cmp(&cells[*b].center)
            .then_with(|| a.cmp(b))
    });
    let midpoint = |a, b| -> Result<f64, Error> {
        let (start, span) = axis_quanta(a, b)?;
        Ok(ratio_position(start, span, 1, 2)
            .ok_or_else(|| Error::Render("axis cell midpoint overflow".into()))? as f64
            / 1024.0)
    };
    let mut ordinal = 0;
    let mut previous = None;
    while ordinal < order.len() {
        let first = order[ordinal];
        let center = cells[first].center;
        let end = ordinal
            + order[ordinal..]
                .partition_point(|&index| cells[index].center.to_bits() == center.to_bits());
        let start_bound = if let Some(previous) = previous {
            midpoint(previous, center)?
        } else {
            cells[first].start
        };
        let end_bound = if end < order.len() {
            midpoint(center, cells[order[end]].center)?
        } else {
            cells[first].end
        };
        for &index in &order[ordinal..end] {
            if labels_visible && ticks[index].label != ticks[first].label {
                return Err(scope
                    .failure(
                        FitKind::Tick,
                        "coincident painted axis labels overlap incompatibly",
                    )
                    .into());
            }
            cells[index].start = start_bound;
            cells[index].end = end_bound;
            cells[index].inward = !categorical && (ordinal == 0 || end == order.len());
        }
        previous = Some(center);
        ordinal = end;
    }
    Ok(())
}

fn apply_clearance(
    cells: &mut [LabelCell],
    ticks: &[AxisTick],
    categorical: bool,
    selection: &crate::CategoryLabelSelection,
    scope: &FitScope,
) -> Result<(), PlanError> {
    for (index, (cell, tick)) in cells.iter_mut().zip(ticks).enumerate() {
        if (categorical && selection.includes(index, ticks.len()))
            || (!categorical && !tick.label.is_empty())
        {
            cell.start += LABEL_CLEARANCE;
            cell.end -= LABEL_CLEARANCE;
            if cell.capacity() <= 0.0 {
                return Err(scope
                    .failure(
                        FitKind::Tick,
                        "axis label has no assigned cell after clearance",
                    )
                    .into());
            }
        }
    }
    Ok(())
}

fn assigned_bounds(
    cells: &[(f64, f64)],
    index: usize,
    categorical: bool,
) -> Result<(f64, f64, f64), Error> {
    let (start, end) = cells.get(index).copied().ok_or_else(|| {
        Error::Render("assigned categorical cell count differs from ticks".into())
    })?;
    if !categorical || !start.is_finite() || !end.is_finite() || start >= end {
        return Err(Error::Render(
            "invalid assigned categorical cell bounds".into(),
        ));
    }
    Ok((f64::midpoint(start, end), start, end))
}
