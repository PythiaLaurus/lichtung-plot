//! Retained scientific membership, final component replacement and ordered point
//! batches shared by actual mark producers. Drawing consumes only final styles.

use crate::data::source::reserve_source;
use crate::data::PlotData;
use crate::render::backend::{Point2D, PointStyle};
use crate::render::geometry::GeometryBox;
use crate::{Error, Result};

use super::{PreparedAlpha, ResolvedAlpha};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObservationAlpha {
    pub(crate) slot: usize,
    pub(crate) source_row: usize,
    pub(crate) value: Option<ResolvedAlpha>,
}

impl ObservationAlpha {
    pub(crate) fn resolve(data: PlotData<'_>, alpha: &PreparedAlpha, row: usize) -> Result<Self> {
        Ok(Self {
            slot: data.row_index(row),
            source_row: data.source_row(row),
            value: data.observation_alpha(alpha, row)?,
        })
    }

    pub(crate) fn point(self, mut style: PointStyle) -> PointStyle {
        if let Some(value) = self.value {
            style.color = style.color.with_alpha(value.alpha);
        }
        style
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PointInk {
    pub(crate) ranges: [(f64, f64); 2],
    pub(crate) bounds: GeometryBox,
}

impl PointInk {
    pub(crate) fn dimensions(self) -> (f64, f64) {
        (
            self.bounds.right - self.bounds.left,
            self.bounds.top - self.bounds.bottom,
        )
    }
}

/// Only actual painted observations contribute ink. All scientific observations
/// still belong to the producer's separate scientific domains and statistics.
pub(crate) fn point_ink(
    style: &PointStyle,
    points: impl IntoIterator<Item = (f64, f64, ObservationAlpha)>,
) -> Result<Option<PointInk>> {
    let mut selected: Option<PointInk> = None;
    // With fixed geometry and an independent outline, only zero/nonzero fill
    // visibility changes the canonical bounds. Cache an actual style for each.
    let mut visibility = [None, None];
    for (x, y, alpha) in points {
        let visible =
            usize::from(alpha.value.map_or(style.color.alpha(), |value| value.alpha) != 0);
        let bounds = if let Some(bounds) = visibility[visible] {
            bounds
        } else {
            let bounds = crate::render::geometry::point_ink_bounds(&alpha.point(style.clone()))?;
            visibility[visible] = Some(bounds);
            bounds
        };
        let Some(bounds) = bounds else { continue };
        selected = Some(if let Some(current) = selected {
            PointInk {
                ranges: [
                    (current.ranges[0].0.min(x), current.ranges[0].1.max(x)),
                    (current.ranges[1].0.min(y), current.ranges[1].1.max(y)),
                ],
                bounds: current.bounds.union(bounds),
            }
        } else {
            PointInk {
                ranges: [(x, x), (y, y)],
                bounds,
            }
        });
    }
    Ok(selected)
}

/// Append in paint order. Equal styles only coalesce when contiguous; separated
/// equal-alpha points, including coincident points, never move or disappear.
pub(crate) fn push_point(
    batches: &mut Vec<(Vec<Point2D>, PointStyle)>,
    point: Point2D,
    style: PointStyle,
) -> Result<()> {
    if let Some((points, previous)) = batches.last_mut() {
        if *previous == style {
            push(points, point)?;
            return Ok(());
        }
    }
    let mut points = Vec::new();
    push(&mut points, point)?;
    push(batches, (points, style))?;
    Ok(())
}

fn push<T>(values: &mut Vec<T>, value: T) -> Result<()> {
    if values.len() == values.capacity() {
        let additional = values.capacity().max(1);
        reserve_source(values, additional)?;
    }
    values.push(value);
    Ok(())
}

pub(crate) fn require_aligned_points(points: usize, alphas: usize) -> Result<()> {
    if points != alphas {
        return Err(Error::Render(
            "prepared observation alpha membership is not aligned".into(),
        ));
    }
    Ok(())
}
