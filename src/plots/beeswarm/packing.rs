//! Source-ordered greedy tangent packing in exact spatial quanta.

use crate::layout::Projection;
use crate::Error;

#[derive(Debug, Clone, Copy)]
pub(super) struct Point {
    pub(super) category: usize,
    pub(super) source_row: usize,
    pub(super) x: i64,
    pub(super) offset: i64,
    radius: i64,
}

pub(super) struct Swarm {
    pub(super) points: Vec<Point>,
    pub(super) hard_pitch: i64,
}

fn squared(value: i128) -> Result<u128, Error> {
    let value = value.unsigned_abs();
    value
        .checked_mul(value)
        .ok_or_else(|| Error::Render("Beeswarm squared distance overflow".into()))
}

fn ceil_sqrt(value: u128) -> Result<i64, Error> {
    let root = value.isqrt();
    let root = if root
        .checked_mul(root)
        .ok_or_else(|| Error::Render("Beeswarm square-root comparison overflow".into()))?
        == value
    {
        root
    } else {
        root.checked_add(1)
            .ok_or_else(|| Error::Render("Beeswarm tangent radius overflow".into()))?
    };
    i64::try_from(root).map_err(|_| Error::Render("Beeswarm tangent exceeds spatial range".into()))
}

pub(super) fn radius(style: &crate::render::backend::PointStyle) -> Result<i64, Error> {
    let Some(ink) = crate::render::geometry::point_ink_bounds(style)? else {
        return Ok(1);
    };
    let x = crate::layout::kernel::ceil_spatial_quanta(ink.left.abs().max(ink.right.abs()))?;
    let y = crate::layout::kernel::ceil_spatial_quanta(ink.bottom.abs().max(ink.top.abs()))?;
    ceil_sqrt(
        squared(i128::from(x))?
            .checked_add(squared(i128::from(y))?)
            .ok_or_else(|| Error::Render("Beeswarm complete glyph radius overflow".into()))?,
    )
}

pub(super) fn pack(
    rows: impl ExactSizeIterator<Item = (usize, f64, f64, usize)>,
    projection: &Projection,
    radius: i64,
) -> Result<Swarm, Error> {
    if radius <= 0 {
        return Err(Error::Render(
            "Beeswarm conservative glyph radius must be positive".into(),
        ));
    }
    let mut points: Vec<Point> = Vec::new();
    crate::data::source::reserve_source(&mut points, rows.len())?;
    let mut first = 0;
    let mut previous_category = None;
    let mut hard_pitch = 1_i64;
    for (category, value, _, source_row) in rows {
        if previous_category != Some(category) {
            if previous_category.is_some_and(|previous| previous > category) {
                return Err(Error::Render(
                    "Beeswarm category source order is not canonical".into(),
                ));
            }
            first = points.len();
            previous_category = Some(category);
        }
        let x = crate::layout::kernel::nearest_spatial_quanta(projection.project(value, 0.0).0)?;
        let previous = &points[first..];
        let capacity = previous
            .len()
            .checked_mul(2)
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| Error::Render("Beeswarm candidate count overflow".into()))?;
        let mut candidates = Vec::new();
        crate::data::source::reserve_source(&mut candidates, capacity)?;

        candidates.push((0_i64, source_row));
        for point in previous {
            let sum = radius
                .checked_add(point.radius)
                .ok_or_else(|| Error::Render("Beeswarm radius sum overflow".into()))?;
            let dx = i128::from(x) - i128::from(point.x);
            if dx.unsigned_abs() < sum as u128 {
                let distance = ceil_sqrt(
                    squared(i128::from(sum))?
                        .checked_sub(squared(dx)?)
                        .ok_or_else(|| {
                            Error::Render("Beeswarm tangent radicand underflow".into())
                        })?,
                )?;
                for direction in [-1_i64, 1] {
                    let delta = distance
                        .checked_mul(direction)
                        .ok_or_else(|| Error::Render("Beeswarm tangent sign overflow".into()))?;
                    let offset = point
                        .offset
                        .checked_add(delta)
                        .ok_or_else(|| Error::Render("Beeswarm tangent offset overflow".into()))?;
                    candidates.push((offset, point.source_row));
                }
            }
        }
        crate::data::source::sort_source_by(&mut candidates, |a, b| {
            a.0.unsigned_abs()
                .cmp(&b.0.unsigned_abs())
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });
        let mut last = None;
        let mut selected = None;
        for (offset, _) in candidates {
            if last == Some(offset) {
                continue;
            }
            last = Some(offset);
            let mut fits = true;
            for point in previous {
                let sum = radius
                    .checked_add(point.radius)
                    .ok_or_else(|| Error::Render("Beeswarm collision radius overflow".into()))?;
                let dx = i128::from(x) - i128::from(point.x);
                let dy = i128::from(offset) - i128::from(point.offset);
                let distance = squared(dx)?
                    .checked_add(squared(dy)?)
                    .ok_or_else(|| Error::Render("Beeswarm collision distance overflow".into()))?;
                if distance < squared(i128::from(sum))? {
                    fits = false;
                    break;
                }
            }
            if fits {
                selected = Some(offset);
                break;
            }
        }
        let offset = selected.ok_or_else(|| {
            Error::Render(
                "Beeswarm admitted tangent candidates have no collision-free placement".into(),
            )
        })?;
        let occupied = i128::from(offset)
            .unsigned_abs()
            .checked_add(radius as u128)
            .and_then(|extent| extent.checked_mul(2))
            .ok_or_else(|| Error::Render("Beeswarm occupied category span overflow".into()))?;
        hard_pitch =
            hard_pitch.max(i64::try_from(occupied).map_err(|_| {
                Error::Render("Beeswarm category span exceeds spatial range".into())
            })?);
        points.push(Point {
            category,
            source_row,
            x,
            offset,
            radius,
        });
    }
    Ok(Swarm { points, hard_pitch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coincident_points_take_zero_then_negative_tangencies_in_source_order() {
        let projection = Projection::new((-1.0, 1.0), (0.0, 1.0), (0.0, 2.0, 0.0, 1.0));
        let rows = [
            (0, 0.0, 0.0, 0),
            (0, 0.0, 1.0, 1),
            (0, 0.0, -1.0, 2),
            (0, 0.0, 2.0, 3),
        ];
        let result = pack(rows.into_iter(), &projection, 2).unwrap();
        assert_eq!(
            result
                .points
                .iter()
                .map(|point| point.offset)
                .collect::<Vec<_>>(),
            [0, -4, 4, -8]
        );
        assert_eq!(
            result
                .points
                .iter()
                .map(|point| point.source_row)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        assert_eq!(result.hard_pitch, 20);
        let separated = [(0, 0.0, 0.0, 0), (0, 4.0 / 1024.0, 0.0, 1)];
        let result = pack(separated.into_iter(), &projection, 2).unwrap();
        assert_eq!(
            result
                .points
                .iter()
                .map(|point| point.offset)
                .collect::<Vec<_>>(),
            [0, 0]
        );
    }

    #[test]
    fn nonintegral_tangent_distances_round_outward_once() {
        let projection = Projection::new((-1.0, 1.0), (0.0, 1.0), (0.0, 2.0, 0.0, 1.0));
        let rows = [(0, 0.0, 0.0, 0), (0, 1.0 / 1024.0, 0.0, 1)];
        let result = pack(rows.into_iter(), &projection, 2).unwrap();
        assert_eq!(result.points[1].x - result.points[0].x, 1);
        assert_eq!(result.points[1].offset, -4);
        assert_eq!(ceil_sqrt(0).unwrap(), 0);
        assert_eq!(ceil_sqrt(15).unwrap(), 4);
        assert_eq!(ceil_sqrt(16).unwrap(), 4);
        assert_eq!(ceil_sqrt(17).unwrap(), 5);
    }

    #[test]
    fn stroked_circle_and_triangle_glyphs_have_complete_conservative_disks() {
        let rows = (0..12)
            .map(|row| (0, 0.0, row as f64, row))
            .collect::<Vec<_>>();
        for shape in [crate::PointShape::Circle, crate::PointShape::TriangleUp] {
            let mut style = crate::render::backend::PointStyle::new((0, 0, 0), 3).with_shape(shape);
            style.outline.as_mut().unwrap().width_ratio =
                crate::theme::RelativeSpacing::ratio(4, 1).unwrap();
            let radius = radius(&style).unwrap();
            let ink = crate::render::geometry::point_ink_bounds(&style)
                .unwrap()
                .unwrap();
            for x in [ink.left, ink.right] {
                for y in [ink.bottom, ink.top] {
                    let x = crate::layout::kernel::ceil_spatial_quanta(x.abs()).unwrap();
                    let y = crate::layout::kernel::ceil_spatial_quanta(y.abs()).unwrap();
                    assert!(
                        squared(i128::from(x)).unwrap() + squared(i128::from(y)).unwrap()
                            <= squared(i128::from(radius)).unwrap()
                    );
                }
            }

            let projection = Projection::new((-1.0, 1.0), (0.0, 1.0), (0.0, 200.0, 0.0, 100.0));
            let result = pack(rows.iter().copied(), &projection, radius).unwrap();
            for (index, point) in result.points.iter().enumerate() {
                for previous in &result.points[..index] {
                    let dx = i128::from(point.x) - i128::from(previous.x);
                    let dy = i128::from(point.offset) - i128::from(previous.offset);
                    assert!(
                        squared(dx).unwrap() + squared(dy).unwrap()
                            >= squared(i128::from(2 * radius)).unwrap()
                    );
                }
            }
        }
    }
}
