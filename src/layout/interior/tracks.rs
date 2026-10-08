//! Shared quantized track assignment for the closed scientific Grid producers.

use crate::layout::fit::{self, FitScope, PlanError};
use crate::layout::kernel::{
    allocate_grid_axis, nearest_spatial_quanta, GridRequirement, TrackSpec,
};
use crate::Error;

#[expect(
    clippy::cast_precision_loss,
    reason = "root-bounded spatial quanta convert exactly to CSS pixels"
)]
fn pixels(value: i64) -> f64 {
    value as f64 / 1024.0
}

pub(crate) fn assign(
    base: &[TrackSpec],
    edges: (f64, f64),
    gap: f64,
    reverse: bool,

    scope: &FitScope,
) -> Result<Vec<(f64, f64)>, PlanError> {
    let lower = nearest_spatial_quanta(edges.0)?;
    let upper = nearest_spatial_quanta(edges.1)?;
    let span = upper
        .checked_sub(lower)
        .ok_or_else(|| Error::Render("scientific track span overflow".into()))?;
    let gap = crate::layout::kernel::ceil_spatial_quanta(gap)?;
    let children: [GridRequirement<'_>; 0] = [];
    let allocated = fit::measured(allocate_grid_axis(
        base,
        &children,
        span,
        [0, 0],
        gap,
        None,
        true,
        scope,
    ))?;
    let mut bounds = Vec::new();
    crate::data::source::reserve_source(&mut bounds, base.len())?;
    let mut position = if reverse { upper } else { lower };
    for (index, size) in allocated.sizes.into_iter().enumerate() {
        let next = if reverse {
            position.checked_sub(size)
        } else {
            position.checked_add(size)
        }
        .ok_or_else(|| Error::Render("scientific track edge overflow".into()))?;
        bounds.push(if reverse {
            (pixels(next), pixels(position))
        } else {
            (pixels(position), pixels(next))
        });
        if index + 1 != base.len() {
            position = if reverse {
                next.checked_sub(gap)
            } else {
                next.checked_add(gap)
            }
            .ok_or_else(|| Error::Render("scientific track gap overflow".into()))?;
        }
    }
    Ok(bounds)
}
