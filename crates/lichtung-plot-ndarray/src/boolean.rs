use lichtung_plot::BooleanValues;
use ndarray::ArrayView1;

use crate::{snapshot, BooleanElement, Result};

/// Snapshot logical true/false/null slots, without numeric coercion.
pub fn booleans<T: BooleanElement>(name: &str, view: ArrayView1<'_, T>) -> Result<BooleanValues> {
    Ok(BooleanValues::nullable(
        name,
        snapshot::values(view, None, BooleanElement::as_boolean, "booleans")?,
    ))
}

/// Snapshot Boolean values with exact-shape validity: true means present.
pub fn booleans_with_validity<T: BooleanElement>(
    name: &str,
    view: ArrayView1<'_, T>,
    validity: ArrayView1<'_, bool>,
) -> Result<BooleanValues> {
    Ok(BooleanValues::nullable(
        name,
        snapshot::values(
            view,
            Some(validity),
            BooleanElement::as_boolean,
            "Boolean validity",
        )?,
    ))
}
