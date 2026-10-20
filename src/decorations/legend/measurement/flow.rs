//! Stable rows or equal-width columns within one already assigned section.

use super::{add, hard, length, MeasuredSection};
use crate::decorations::legend::{LegendMetrics, LegendSlot};
use crate::layout::fit::{self, FitKind, FitScope, PlanError};
use crate::theme::spacing::ExactLength;
use crate::Error;

pub(super) struct SectionFlow {
    pub(super) slots: Vec<LegendSlot>,
    pub(super) width: ExactLength,
    pub(super) height: ExactLength,
    pub(super) columns: usize,
}

fn baseline(
    top: &ExactLength,
    title: &ExactLength,
    consumed: &ExactLength,
    ascent: i64,
) -> Result<i64, Error> {
    Ok(top
        .add_length(title)
        .add_length(consumed)
        .add_length(&length(ascent)?)
        .before(0.0)?
        .quanta())
}

fn body_fits(
    section: &MeasuredSection,
    body: &ExactLength,
    height: Option<i64>,
) -> Result<bool, Error> {
    match height {
        Some(height) => Ok(hard(&section.title_occupancy.add_length(body))? <= height),
        None => Ok(true),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one deterministic pass finalizes baseline rows or equal-width columns without revisiting entries"
)]
pub(super) fn assigned_entries(
    section: &MeasuredSection,
    width: i64,
    height: Option<i64>,
    top: &ExactLength,
    metrics: &LegendMetrics,
    horizontal: bool,
    scope: &FitScope,
) -> Result<SectionFlow, PlanError> {
    let mut slots: Vec<LegendSlot> = Vec::new();
    fit::reserve(&mut slots, section.entries.len(), "measured guide slots")?;
    let mut x = length(0)?;
    let mut y = length(0)?;
    let mut used_width = length(0)?;
    let mut used_height = length(0)?;
    let mut columns = 1;
    let mut row_entries = 0;
    let mut row_start = 0;
    let mut row_ascent = 0;
    let mut row_descent = 0;
    let column_width = &section
        .entries
        .first()
        .ok_or_else(|| Error::Render("a guide flow section is empty".into()))?
        .width;
    for entry in &section.entries {
        let entry_height = entry.height()?;
        if horizontal {
            if row_entries > 0 && hard(&x.add_length(&entry.width))? > width {
                let baseline = baseline(top, &section.title_occupancy, &y, row_ascent)?;
                for slot in &mut slots[row_start..] {
                    slot.key[1] = baseline;
                    slot.label[1] = baseline;
                }
                y = y
                    .add_length(&length(add(row_ascent, row_descent)?)?)
                    .add_length(&metrics.exact.row_gap);
                row_start = slots.len();
                row_entries = 0;
                row_ascent = 0;
                row_descent = 0;
                x = length(0)?;
            }
            row_entries += 1;
            columns = columns.max(row_entries);
            row_ascent = row_ascent.max(entry.ascent);
            row_descent = row_descent.max(entry.descent);
        } else if row_entries > 0
            && !body_fits(section, &y.add_length(&length(entry_height)?), height)?
        {
            x = x
                .add_length(column_width)
                .add_length(&metrics.exact.column_gap);
            y = length(0)?;
            row_entries = 0;
            columns += 1;
        }
        let entry_right = x.add_length(&entry.width);
        let entry_bottom = y.add_length(&length(entry_height)?);
        if hard(&entry_right)? > width
            || (!horizontal && !body_fits(section, &entry_bottom, height)?)
        {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "guide section flow exceeds its assigned capacity or width",
                )
                .into());
        }
        let baseline = if horizontal {
            0
        } else {
            baseline(top, &section.title_occupancy, &y, entry.ascent)?
        };
        slots.push(LegendSlot {
            key: [
                x.add_length(&length(entry.key_origin)?)
                    .round(crate::layout::kernel::SpatialRounding::NearestEven)?
                    .quanta(),
                baseline,
            ],
            label: [
                x.add_length(&entry.chrome)
                    .round(crate::layout::kernel::SpatialRounding::NearestEven)?
                    .quanta(),
                baseline,
            ],
            block: entry.block.clone(),
            width: hard(&entry.width)?,
            height: entry_height,
        });
        if used_width.compare(&entry_right) == std::cmp::Ordering::Less {
            used_width = entry_right;
        }
        if horizontal {
            x = x
                .add_length(&entry.width)
                .add_length(&metrics.exact.column_gap);
        } else {
            row_entries += 1;
            y = entry_bottom.add_length(&metrics.exact.row_gap);
            if used_height.compare(&entry_bottom) == std::cmp::Ordering::Less {
                used_height = entry_bottom;
            }
        }
    }
    if horizontal {
        let baseline = baseline(top, &section.title_occupancy, &y, row_ascent)?;
        for slot in &mut slots[row_start..] {
            slot.key[1] = baseline;
            slot.label[1] = baseline;
        }
        used_height = y.add_length(&length(add(row_ascent, row_descent)?)?);
        if !body_fits(section, &used_height, height)? {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "guide section rows exceed their assigned height",
                )
                .into());
        }
    }
    Ok(SectionFlow {
        slots,
        width: used_width,
        height: used_height,
        columns,
    })
}
