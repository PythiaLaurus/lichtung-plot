use lichtung_plot::CategoryValues;
use ndarray::ArrayView1;

use crate::{error, snapshot, CategoryElement, Result};

/// Snapshot exact raw keys in logical order; unobserved levels are not invented.
pub fn categories<T: CategoryElement>(
    name: &str,
    view: ArrayView1<'_, T>,
) -> Result<CategoryValues> {
    let values = snapshot::values(
        view,
        None,
        |value| value.as_key().map(str::to_owned),
        "categories",
    )?;
    CategoryValues::nullable(name, values).map_err(|source| error::native("categories", source))
}

/// Snapshot raw keys with exact-shape validity: true means present.
pub fn categories_with_validity<T: CategoryElement>(
    name: &str,
    view: ArrayView1<'_, T>,
    validity: ArrayView1<'_, bool>,
) -> Result<CategoryValues> {
    let values = snapshot::values(
        view,
        Some(validity),
        |value| value.as_key().map(str::to_owned),
        "category validity",
    )?;
    CategoryValues::nullable(name, values).map_err(|source| error::native("categories", source))
}
