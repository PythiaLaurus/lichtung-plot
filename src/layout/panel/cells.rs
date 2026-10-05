//! Assigned axis-parallel cells, shared by source fit and frozen axis geometry.

use super::projection::{axis_quanta, projection_key, ratio_position};
use super::{AxisSide, AxisTick, Projection};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::kernel::nearest_spatial_quanta;
use crate::layout::ResolvedTextPlan;
use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
use crate::render::coordinates::{canonical_text, CoordinatePrecision};
use crate::theme::ResolvedTheme;
use crate::Error;

// Split the existing 4px label separation across the two neighboring cells.
pub(crate) const LABEL_CLEARANCE: f64 = 2.0;

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

pub(super) fn assign(
    ticks: &[AxisTick],
    categorical: bool,
    assigned: Option<&[(f64, f64)]>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<Vec<LabelCell>, PlanError> {
    let panel = projection.panel_box();
    let (origin, end, range) = match side {
        AxisSide::Bottom => (panel.x0, panel.x1, projection.x_range()),
        AxisSide::Left => (panel.y0, panel.y1, projection.y_range()),
    };
    let (origin, span) = axis_quanta(origin, end)?;
    let mut cells = Vec::new();
    crate::layout::fit::reserve(theme, &mut cells, ticks.len(), "axis label cells")?;
    let mut previous_key = None;
    let environment = theme.prepared_font_environment()?;
    for tick in ticks {
        environment
            .claim_processing_visits(1)
            .map_err(|error| Error::Render(format!("axis position admission: {error:?}")))?;
        let key = projection_key(tick.position, range).ok_or_else(|| {
            scope.failure(
                FitKind::Tick,
                "axis position is outside its finite display domain",
            )
        })?;
        if !categorical && previous_key.is_some_and(|previous| key <= previous) {
            return Err(scope
                .failure(FitKind::Tick, "axis projection keys must strictly ascend")
                .into());
        }
        previous_key = Some(key);
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
    if !categorical {
        let mut previous: Option<usize> = None;
        for (index, tick) in ticks.iter().enumerate() {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("axis boundary admission: {error:?}")))?;
            if tick.label.is_empty() {
                continue;
            }
            if let Some(previous_index) = previous {
                let (start, span) = axis_quanta(cells[previous_index].center, cells[index].center)?;
                let boundary = ratio_position(start, span, 1, 2)
                    .ok_or_else(|| Error::Render("axis cell midpoint overflow".into()))?
                    as f64
                    / 1024.0;
                cells[previous_index].end = boundary;
                cells[index].start = boundary;
            } else {
                cells[index].inward = true;
            }
            previous = Some(index);
        }
        if let Some(last) = previous {
            cells[last].inward = true;
        }
    }
    apply_clearance(&mut cells, ticks, theme, scope)?;
    Ok(cells)
}

fn apply_clearance(
    cells: &mut [LabelCell],
    ticks: &[AxisTick],
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<(), PlanError> {
    let environment = theme.prepared_font_environment()?;
    for (cell, tick) in cells.iter_mut().zip(ticks) {
        environment
            .claim_processing_visits(1)
            .map_err(|error| Error::Render(format!("axis clearance admission: {error:?}")))?;
        if !tick.label.is_empty() {
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
    fn p2_cells_charge_each_semantic_pass_including_empty_labels() {
        let ticks = [
            AxisTick::new(0.5, "left"),
            AxisTick::new(1.5, ""),
            AxisTick::new(2.5, "right"),
        ];
        let projection = Projection::new((0.0, 3.0), (0.0, 3.0), (10.0, 310.0, 20.0, 320.0));
        for (categorical, visits) in [(false, 9), (true, 6)] {
            for available in [0, visits - 1, visits, visits + 1] {
                let theme = crate::Theme::Publication
                    .resolve(&crate::font::canonical::OperationGenerationResources::new())
                    .unwrap();
                let resources = theme
                    .prepared_font_environment()
                    .unwrap()
                    .generation_resources();
                let before = resources.usage().unwrap().1;
                resources.set_test_limits(before + available, u64::MAX);
                let result = assign(
                    &ticks,
                    categorical,
                    None,
                    &projection,
                    AxisSide::Bottom,
                    &theme,
                    &FitScope::root(),
                );
                assert_eq!(result.is_ok(), available >= visits);
                assert_eq!(resources.usage().unwrap().1 - before, available.min(visits));
                if let Err(error) = result {
                    assert!(matches!(error, PlanError::Fatal(_)));
                }
            }
        }
    }

    #[test]
    fn p2_cells_stop_visiting_after_position_failure() {
        let ticks = [
            AxisTick::new(1.5, "a"),
            AxisTick::new(0.5, "b"),
            AxisTick::new(2.5, "c"),
        ];
        let projection = Projection::new((0.0, 3.0), (0.0, 3.0), (0.0, 300.0, 0.0, 300.0));
        for available in [1, 2, 3] {
            let theme = crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let before = resources.usage().unwrap().1;
            resources.set_test_limits(before + available, u64::MAX);
            let error = assign(
                &ticks,
                false,
                None,
                &projection,
                AxisSide::Bottom,
                &theme,
                &FitScope::root(),
            )
            .unwrap_err();
            assert_eq!(matches!(error, PlanError::Fatal(_)), available < 2);
            assert_eq!(resources.usage().unwrap().1 - before, available.min(2));
        }
    }
}
