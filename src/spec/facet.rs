//! Closed generic-facet layouts.

use crate::{CategoryValues, Error};

/// Numeric-axis sharing policy for generic facet panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FacetScales {
    /// Share both numeric axes where applicable.
    #[default]
    Fixed,
    /// Resolve the x axis independently per panel.
    FreeX,
    /// Resolve the y axis independently per panel.
    FreeY,
    /// Resolve both axes independently per panel.
    Free,
}

/// One complete generic-facet layout and numeric-axis sharing policy.
///
/// A facet is either a row, a column, a row-by-column grid, or one wrapped
/// categorical dimension. Mixed grid and wrap layouts are not representable,
/// and numeric axes are fixed unless [`Self::scales`] selects otherwise.
#[derive(Debug, Clone)]
pub struct FacetSpec {
    layout: FacetLayout,
    scales: FacetScales,
}

#[derive(Debug, Clone)]
enum FacetLayout {
    Row(CategoryValues),
    Column(CategoryValues),
    Grid {
        row: CategoryValues,
        column: CategoryValues,
    },
    Wrap {
        values: CategoryValues,
        columns: usize,
    },
}

impl FacetSpec {
    /// Split the plot into rows by one categorical dimension.
    #[must_use]
    pub fn row(values: CategoryValues) -> Self {
        Self {
            layout: FacetLayout::Row(values),
            scales: FacetScales::Fixed,
        }
    }

    /// Split the plot into columns by one categorical dimension.
    #[must_use]
    pub fn column(values: CategoryValues) -> Self {
        Self {
            layout: FacetLayout::Column(values),
            scales: FacetScales::Fixed,
        }
    }

    /// Split the plot into a row-by-column grid.
    pub fn grid(row: CategoryValues, column: CategoryValues) -> Result<Self, Error> {
        if row.name() == column.name() {
            return Err(Error::Data(
                "facet grid row and column must use different columns".into(),
            ));
        }
        Ok(Self {
            layout: FacetLayout::Grid { row, column },
            scales: FacetScales::Fixed,
        })
    }

    /// Wrap one categorical dimension into a positive number of columns.
    pub fn wrap(values: CategoryValues, columns: usize) -> Result<Self, Error> {
        if columns == 0 {
            return Err(Error::Data(
                "facet wrap columns must be greater than zero".into(),
            ));
        }
        Ok(Self {
            layout: FacetLayout::Wrap { values, columns },
            scales: FacetScales::Fixed,
        })
    }

    /// Configure whether facet panels share numeric axes.
    #[must_use]
    pub const fn scales(mut self, scales: FacetScales) -> Self {
        self.scales = scales;
        self
    }

    pub(crate) const fn scale_policy(&self) -> FacetScales {
        self.scales
    }

    pub(crate) const fn row_values(&self) -> Option<&CategoryValues> {
        match &self.layout {
            FacetLayout::Row(values) | FacetLayout::Grid { row: values, .. } => Some(values),
            FacetLayout::Column(_) | FacetLayout::Wrap { .. } => None,
        }
    }

    pub(crate) const fn column_values(&self) -> Option<&CategoryValues> {
        match &self.layout {
            FacetLayout::Column(values) | FacetLayout::Grid { column: values, .. } => Some(values),
            FacetLayout::Row(_) | FacetLayout::Wrap { .. } => None,
        }
    }

    pub(crate) const fn wrap_values(&self) -> Option<&CategoryValues> {
        match &self.layout {
            FacetLayout::Wrap { values, .. } => Some(values),
            FacetLayout::Row(_) | FacetLayout::Column(_) | FacetLayout::Grid { .. } => None,
        }
    }

    pub(crate) const fn wrap_columns(&self) -> Option<usize> {
        match &self.layout {
            FacetLayout::Wrap { columns, .. } => Some(*columns),
            FacetLayout::Row(_) | FacetLayout::Column(_) | FacetLayout::Grid { .. } => None,
        }
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &CategoryValues> {
        [self.row_values(), self.column_values(), self.wrap_values()]
            .into_iter()
            .flatten()
    }
}
