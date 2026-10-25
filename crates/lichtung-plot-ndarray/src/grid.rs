use std::collections::HashSet;

use lichtung_plot::{CategoryValues, NumericValues};
use ndarray::ArrayView2;

use crate::{error, snapshot, Error, NumericElement, Result};

/// Owned native channels from a labeled grid, ready for `Plot::heatmap`.
#[derive(Debug, Clone)]
pub struct HeatmapValues {
    pub x: CategoryValues,
    pub y: CategoryValues,
    pub fill: NumericValues,
}

#[derive(Debug)]
pub(crate) struct Cells<'a> {
    pub x: Vec<&'a str>,
    pub y: Vec<&'a str>,
    pub fill: Vec<Option<f64>>,
}

fn axis(keys: &[&str], length: usize, axis: &'static str) -> Result<()> {
    if keys.len() != length {
        return Err(Error::Shape {
            operation: axis,
            expected: vec![length],
            actual: vec![keys.len()],
        });
    }
    let mut seen = HashSet::new();
    seen.try_reserve(keys.len())
        .map_err(|source| Error::Allocation {
            operation: axis,
            source,
        })?;
    for key in keys {
        if !seen.insert(*key) {
            return Err(Error::DuplicateAxisKey {
                axis,
                key: (*key).into(),
            });
        }
    }
    Ok(())
}

pub(crate) fn cells<'a, T: NumericElement>(
    view: ArrayView2<'_, T>,
    x_keys: &[&'a str],
    y_keys: &[&'a str],
    validity: Option<ArrayView2<'_, bool>>,
) -> Result<Cells<'a>> {
    axis(x_keys, view.ncols(), "x")?;
    axis(y_keys, view.nrows(), "y")?;
    let count = view
        .nrows()
        .checked_mul(view.ncols())
        .ok_or(Error::SizeOverflow)?;
    let fill = snapshot::values(view, validity, NumericElement::to_f64, "heatmap validity")?;
    let mut x = snapshot::buffer(count, "heatmap x")?;
    let mut y = snapshot::buffer(count, "heatmap y")?;
    for row in y_keys {
        for column in x_keys {
            x.push(*column);
            y.push(*row);
        }
    }
    Ok(Cells { x, y, fill })
}

fn native(name: &str, cells: Cells<'_>, x_name: &str, y_name: &str) -> Result<HeatmapValues> {
    Ok(HeatmapValues {
        x: CategoryValues::new(x_name, cells.x)
            .map_err(|source| error::native("heatmap x", source))?,
        y: CategoryValues::new(y_name, cells.y)
            .map_err(|source| error::native("heatmap y", source))?,
        fill: NumericValues::nullable(name, cells.fill),
    })
}

/// Expand cell `(row, column)` to `(x_keys[column], y_keys[row], value)`.
///
/// Each axis is `(channel_name, raw_keys)` with one unique key per logical index.
/// Slots follow row-major logical order, regardless of memory layout. A later
/// provenance map needs one unique ID per cell, not per original matrix row.
pub fn heatmap_values<T: NumericElement>(
    value_name: &str,
    view: ArrayView2<'_, T>,
    x_keys: (&str, &[&str]),
    y_keys: (&str, &[&str]),
) -> Result<HeatmapValues> {
    native(
        value_name,
        cells(view, x_keys.1, y_keys.1, None)?,
        x_keys.0,
        y_keys.0,
    )
}

/// Expand a grid with exact-shape validity, retaining every cell's x/y identity.
pub fn heatmap_values_with_validity<T: NumericElement>(
    value_name: &str,
    view: ArrayView2<'_, T>,
    x_keys: (&str, &[&str]),
    y_keys: (&str, &[&str]),
    validity: ArrayView2<'_, bool>,
) -> Result<HeatmapValues> {
    native(
        value_name,
        cells(view, x_keys.1, y_keys.1, Some(validity))?,
        x_keys.0,
        y_keys.0,
    )
}
