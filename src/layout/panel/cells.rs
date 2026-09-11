//! Assigned axis-parallel cells, shared by source fit and frozen axis geometry.

use super::{sources, AxisSide, AxisTick, Projection};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::theme::ResolvedTheme;
use crate::Error;

// Split the existing 4px label separation across the two neighboring cells.
pub(super) const LABEL_CLEARANCE: f64 = 2.0;

#[derive(Debug, Clone, Copy)]
pub(super) struct LabelCell {
    pub(super) center: f64,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) inward: bool,
}

impl LabelCell {
    pub(super) fn capacity(self) -> f64 {
        if self.inward {
            self.end - self.start
        } else {
            2.0 * (self.center - self.start).min(self.end - self.center)
        }
    }

    pub(super) fn label_center(self, extent: f64) -> f64 {
        if self.inward {
            self.center
                .max(self.start + extent / 2.0)
                .min(self.end - extent / 2.0)
        } else {
            self.center
        }
    }
}

pub(super) fn assign(
    ticks: &[AxisTick],
    categorical: bool,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<Vec<LabelCell>, PlanError> {
    let panel = projection.panel_box();
    let (origin, span, range) = match side {
        AxisSide::Bottom => (panel.x0, panel.width(), projection.x_range()),
        AxisSide::Left => (panel.y0, panel.height(), projection.y_range()),
    };
    let span = sources::span_quanta(span)?;
    let mut cells = Vec::new();
    crate::layout::fit::reserve(theme, &mut cells, ticks.len(), "axis label cells")?;
    let mut previous_key = None;
    for tick in ticks {
        let key = sources::projection_key(tick.position, range).ok_or_else(|| {
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
        let (center, start, end) = if categorical {
            let count = ticks.len() as u128;
            let index =
                (u128::from(key) * count / u128::from(u64::MAX)).min(count.saturating_sub(1));
            let rounded = |numerator: u128, denominator: u128| -> Result<f64, Error> {
                let product = u128::try_from(span)
                    .map_err(|_| Error::Render("negative category span".into()))?
                    .checked_mul(numerator)
                    .ok_or_else(|| Error::Render("category cell overflow".into()))?;
                let quotient = product / denominator;
                let remainder = product % denominator;
                let value = quotient
                    + u128::from(
                        2 * remainder > denominator
                            || (2 * remainder == denominator && !quotient.is_multiple_of(2)),
                    );
                Ok(origin + value as f64 / 1024.0)
            };
            (
                rounded(2 * index + 1, 2 * count)?,
                rounded(index, count)?,
                rounded(index + 1, count)?,
            )
        } else {
            let center = sources::projected_center(key, span)
                .ok_or_else(|| Error::Render("axis center overflow".into()))?;
            (
                origin + center as f64 / 1024.0,
                origin,
                origin + span as f64 / 1024.0,
            )
        };
        cells.push(LabelCell {
            center,
            start,
            end,
            inward: false,
        });
    }
    if !categorical {
        let mut previous: Option<usize> = None;
        for (index, _) in ticks
            .iter()
            .enumerate()
            .filter(|(_, tick)| !tick.label.is_empty())
        {
            if let Some(previous_index) = previous {
                let boundary = f64::midpoint(cells[previous_index].center, cells[index].center);
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
    for (cell, tick) in cells.iter_mut().zip(ticks) {
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
    Ok(cells)
}
