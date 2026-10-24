//! Assigned axis-parallel cells, shared by source fit and frozen axis geometry.

use super::projection::{axis_quanta, projection_key, ratio_position};
use super::{AxisSide, AxisTick, Projection};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::kernel::nearest_spatial_quanta;
use crate::layout::ResolvedTextPlan;
use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
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

    /// Source selection and frozen labels use the same canonical anchor and ink.
    pub(super) fn placement(
        self,
        plan: &ResolvedTextPlan,
        side: AxisSide,
        perpendicular: f64,
    ) -> Result<Option<PlannedTextPlacement>, Error> {
        let anchor = match side {
            AxisSide::Bottom => TextAnchor::Middle,
            AxisSide::Left => TextAnchor::End,
        };
        let mut placement = PlannedTextPlacement::middle(Point2D::new(0.0, 0.0), anchor, 0.0);
        let length = match side {
            AxisSide::Bottom => plan.width_quanta,
            AxisSide::Left => plan.height_quanta,
        };
        let half = crate::render::backend::nearest_even_text_half(length)
            .ok_or_else(|| Error::Render("axis label half extent overflow".into()))?;
        let (mut lower, mut upper) = match side {
            AxisSide::Bottom => (-half, length - half),
            AxisSide::Left => (half - length, half),
        };
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
            lower = lower.min(nearest_spatial_quanta(start)?);
            upper = upper.max(nearest_spatial_quanta(end)?);
        }
        let minimum = i128::from(nearest_spatial_quanta(self.start)?) - i128::from(lower);
        let maximum = i128::from(nearest_spatial_quanta(self.end)?) - i128::from(upper);
        if minimum > maximum {
            return Ok(None);
        }
        let mut center = i128::from(nearest_spatial_quanta(self.center)?);
        if self.inward {
            center = center.clamp(minimum, maximum);
        }
        if center < minimum || center > maximum {
            return Ok(None);
        }
        let center = i64::try_from(center)
            .map_err(|_| Error::Render("axis label anchor overflow".into()))?
            as f64
            / 1024.0;
        placement.position = match side {
            AxisSide::Bottom => Point2D::new(center, perpendicular),
            AxisSide::Left => Point2D::new(perpendicular, center),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigned_cells_keep_positions_including_empty_labels() {
        let ticks = [
            AxisTick::new(0.5, "left"),
            AxisTick::new(1.5, ""),
            AxisTick::new(2.5, "right"),
        ];
        let projection = Projection::new((0.0, 3.0), (0.0, 3.0), (10.0, 310.0, 20.0, 320.0));
        for categorical in [false, true] {
            {
                let result = assign(
                    &ticks,
                    categorical,
                    true,
                    None,
                    &crate::CategoryLabelSelection::all(),
                    &projection,
                    AxisSide::Bottom,
                    &FitScope::root(),
                );
                let cells = result.unwrap();
                assert_eq!(cells.len(), ticks.len());
                assert_eq!(
                    cells
                        .iter()
                        .map(|cell| cell.center.to_bits())
                        .collect::<Vec<_>>(),
                    [60.0, 160.0, 260.0].map(f64::to_bits)
                );
            }
        }
    }

    #[test]
    fn geometric_order_preserves_source_order_and_coincident_drawable_cells() {
        let ticks = [
            AxisTick::new(1.5, "a"),
            AxisTick::new(0.5, "b"),
            AxisTick::new(0.5, "b"),
        ];
        let projection = Projection::new((0.0, 3.0), (0.0, 3.0), (0.0, 300.0, 0.0, 300.0));
        {
            let cells = assign(
                &ticks,
                false,
                true,
                None,
                &crate::CategoryLabelSelection::all(),
                &projection,
                AxisSide::Bottom,
                &FitScope::root(),
            )
            .unwrap();
            assert!(cells[0].center > cells[1].center);
            assert_eq!(cells[1].center.to_bits(), cells[2].center.to_bits());
            assert_eq!(cells[1].capacity().to_bits(), cells[2].capacity().to_bits());
        }
    }

    #[test]
    fn sparse_labels_use_selected_neighbors_without_compacting_assigned_tracks() {
        let ticks: Vec<_> = (0..6)
            .map(|index| AxisTick::new(f64::from(index) + 0.5, format!("label {index}")))
            .collect();
        let projection = Projection::new((0.0, 6.0), (0.0, 6.0), (0.0, 600.0, 0.0, 600.0));
        let bounds = [
            (0.0, 40.0),
            (40.0, 160.0),
            (160.0, 300.0),
            (300.0, 360.0),
            (360.0, 460.0),
            (460.0, 600.0),
        ];
        let all = assign(
            &ticks,
            true,
            true,
            Some(&bounds),
            &crate::CategoryLabelSelection::all(),
            &projection,
            AxisSide::Bottom,
            &FitScope::root(),
        )
        .unwrap();
        let selection = crate::CategoryLabelSelection::every(3)
            .unwrap()
            .offset(1)
            .unwrap();
        let sparse = assign(
            &ticks,
            true,
            true,
            Some(&bounds),
            &selection,
            &projection,
            AxisSide::Bottom,
            &FitScope::root(),
        )
        .unwrap();
        assert_eq!(all.len(), 6);
        assert_eq!(sparse.len(), 6);
        for (a, b) in all.iter().zip(&sparse) {
            assert_eq!(a.center.to_bits(), b.center.to_bits());
        }
        for index in [1, 4] {
            assert!(sparse[index].capacity() > all[index].capacity());
        }
        assert_eq!(
            (sparse[4].start - sparse[1].end).to_bits(),
            (2.0 * LABEL_CLEARANCE).to_bits()
        );
    }
}
