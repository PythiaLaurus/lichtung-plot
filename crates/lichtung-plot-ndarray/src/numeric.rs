use lichtung_plot::NumericValues;
use ndarray::ArrayView1;

use crate::{snapshot, NumericElement, Result};

/// Snapshot a logical one-dimensional numeric view, retaining nulls and NaNs.
pub fn numeric<T: NumericElement>(name: &str, view: ArrayView1<'_, T>) -> Result<NumericValues> {
    Ok(NumericValues::nullable(
        name,
        snapshot::values(view, None, NumericElement::to_f64, "numeric")?,
    ))
}

/// Snapshot numeric values with exact-shape validity: true means present.
pub fn numeric_with_validity<T: NumericElement>(
    name: &str,
    view: ArrayView1<'_, T>,
    validity: ArrayView1<'_, bool>,
) -> Result<NumericValues> {
    Ok(NumericValues::nullable(
        name,
        snapshot::values(
            view,
            Some(validity),
            NumericElement::to_f64,
            "numeric validity",
        )?,
    ))
}
