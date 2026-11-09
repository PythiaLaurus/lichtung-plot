//! Caller-computed columnar geometry accepted by distribution renderers.

use crate::{CategoryValues, Error, NumericValues};

mod histogram;
mod raincloud;

pub(crate) use histogram::validate_kde;

/// Complete supplied raincloud geometry in three independently aligned domains.
///
/// Summary row order determines group order. Density and point rows name their
/// summary group by raw category/hue strings and retain their relative order.
#[derive(Debug, Clone)]
pub struct RaincloudGeometry {
    /// Scientific category dimension, also used for the inferred x-axis title.
    pub category_dimension: String,
    /// Scientific value dimension, also used for the inferred y-axis title.
    pub value_dimension: String,
    /// Scientific hue dimension; present together with all three hue columns.
    pub hue_dimension: Option<String>,
    /// One category per summary row.
    pub category: CategoryValues,
    /// One hue per summary row when hue is present.
    pub hue: Option<CategoryValues>,
    /// First quartiles aligned with summary categories.
    pub q1: NumericValues,
    /// Medians aligned with summary categories.
    pub median: NumericValues,
    /// Third quartiles aligned with summary categories.
    pub q3: NumericValues,
    /// Category keys for density rows.
    pub density_category: CategoryValues,
    /// Hue keys for density rows when hue is present.
    pub density_hue: Option<CategoryValues>,
    /// Density support coordinates, strictly increasing within each group.
    pub density_y: NumericValues,
    /// Density widths in the inclusive range `[0, 1]`.
    pub normalized_width: NumericValues,
    /// Category keys for raw-point rows.
    pub point_category: CategoryValues,
    /// Hue keys for raw-point rows when hue is present.
    pub point_hue: Option<CategoryValues>,
    /// Raw-point data-space coordinates.
    pub point_y: NumericValues,
    /// Raw-point strip coordinates in the inclusive range `[0, 1]`.
    pub normalized_strip: NumericValues,
}

/// Complete supplied density-axis histogram geometry.
///
/// Bins, KDE and rugs are independent alignment domains. All components are
/// required; the renderer never estimates missing supplied components.
#[derive(Debug, Clone)]
pub struct HistogramGeometry {
    /// Scientific value dimension, also used for the inferred x-axis title.
    pub value_dimension: String,
    /// Scientific density dimension, also used for the inferred y-axis title.
    pub density_dimension: String,
    /// Lower bin edges in increasing, contiguous bin order.
    pub bin_lower: NumericValues,
    /// Upper bin edges aligned with lower edges.
    pub bin_upper: NumericValues,
    /// Nonnegative bin densities aligned with lower/upper edges.
    pub bin_density: NumericValues,
    /// Strictly increasing KDE coordinates within the bin support.
    pub kde_x: NumericValues,
    /// Nonnegative KDE densities aligned with KDE x.
    pub kde_y: NumericValues,
    /// Rug x coordinates in caller-supplied order, including duplicates.
    pub rug_x: NumericValues,
    /// Nonnegative rug baselines aligned with rug x.
    pub rug_baseline: NumericValues,
    /// Rug tops, strictly above their aligned baselines.
    pub rug_top: NumericValues,
}

impl PartialEq for HistogramGeometry {
    fn eq(&self, other: &Self) -> bool {
        self.value_dimension == other.value_dimension
            && self.density_dimension == other.density_dimension
            && [
                (&self.bin_lower, &other.bin_lower),
                (&self.bin_upper, &other.bin_upper),
                (&self.bin_density, &other.bin_density),
                (&self.kde_x, &other.kde_x),
                (&self.kde_y, &other.kde_y),
                (&self.rug_x, &other.rug_x),
                (&self.rug_baseline, &other.rug_baseline),
                (&self.rug_top, &other.rug_top),
            ]
            .into_iter()
            .all(|(left, right)| numeric_equal(left, right))
    }
}

impl PartialEq for RaincloudGeometry {
    fn eq(&self, other: &Self) -> bool {
        self.category_dimension == other.category_dimension
            && self.value_dimension == other.value_dimension
            && self.hue_dimension == other.hue_dimension
            && [
                (&self.category, &other.category),
                (&self.density_category, &other.density_category),
                (&self.point_category, &other.point_category),
            ]
            .into_iter()
            .all(|(left, right)| category_equal(left, right))
            && [
                (&self.hue, &other.hue),
                (&self.density_hue, &other.density_hue),
                (&self.point_hue, &other.point_hue),
            ]
            .into_iter()
            .all(|(left, right)| match (left, right) {
                (Some(left), Some(right)) => category_equal(left, right),
                (None, None) => true,
                _ => false,
            })
            && [
                (&self.q1, &other.q1),
                (&self.median, &other.median),
                (&self.q3, &other.q3),
                (&self.density_y, &other.density_y),
                (&self.normalized_width, &other.normalized_width),
                (&self.point_y, &other.point_y),
                (&self.normalized_strip, &other.normalized_strip),
            ]
            .into_iter()
            .all(|(left, right)| numeric_equal(left, right))
    }
}

pub(crate) fn numeric_equal(left: &NumericValues, right: &NumericValues) -> bool {
    left.name() == right.name()
        && left.len() == right.len()
        && (0..left.len()).all(|row| left.get(row) == right.get(row))
}

fn category_equal(left: &CategoryValues, right: &CategoryValues) -> bool {
    left.name() == right.name()
        && left.len() == right.len()
        && (0..left.len()).all(|row| left.get(row) == right.get(row))
}

fn dimension(value: &str, role: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        Err(Error::Data(format!("{role} must be nonempty")))
    } else {
        Ok(())
    }
}

fn aligned(domain: &str, expected: usize, columns: &[(&str, usize)]) -> Result<(), Error> {
    for &(name, actual) in columns {
        if actual != expected {
            return Err(Error::Data(format!(
                "{domain} column {name:?} has {actual} rows; expected {expected}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn finite(values: &NumericValues, row: usize, role: &str) -> Result<f64, Error> {
    values
        .get(row)
        .flatten()
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            Error::Data(format!(
                "{role} column {:?} requires a present finite value at row {row}",
                values.name()
            ))
        })
}

pub(crate) fn category_value<'a>(
    values: &'a CategoryValues,
    row: usize,
    role: &str,
) -> Result<&'a str, Error> {
    let value = values.get(row).flatten().ok_or_else(|| {
        Error::Data(format!(
            "{role} column {:?} has a missing key at row {row}",
            values.name()
        ))
    })?;
    dimension(value, role)?;
    Ok(value)
}

#[cfg(test)]
mod validation_tests;
