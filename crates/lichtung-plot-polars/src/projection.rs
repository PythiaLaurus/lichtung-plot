//! Selected columns with their complete, borrowed original names namespace.

use std::collections::HashSet;

use polars_core::prelude::{Column, DataFrame};

use crate::error::{binding, length, BindingResult};
use crate::input::{self, sealed, SeriesBinding};
use crate::{
    AdapterError, BooleanInput, BooleanValues, ColumnsInput, NumericSeriesInput, SourceRows,
    SourceRowsInput,
};

/// A short-lived selected-column collection with original raw field identities.
///
/// `source_names` must contain the complete ordered names of the original source,
/// including unselected fields whose dtypes or values have not been imported.
/// Only the selected `columns` are read by the existing role extractors. The
/// descriptor neither stores a schema nor installs a DataFrame lookup context.
/// Even one selected column remains a collection, not a scalar variable.
pub struct ColumnProjection<'columns> {
    source_rows: usize,
    source_names: &'columns [&'columns str],
    columns: &'columns [Column],
}

impl<'columns> ColumnProjection<'columns> {
    /// Check selection structure without reading observations or admitting dtypes.
    ///
    /// Names in each namespace/selection must be unique, selected names must
    /// belong to the complete namespace, and column lengths must match
    /// `source_rows`. Empty raw names are legal and are not renamed.
    pub fn new(
        source_rows: usize,
        source_names: &'columns [&'columns str],
        columns: &'columns [Column],
    ) -> std::result::Result<Self, AdapterError> {
        if columns.is_empty() {
            return Err(binding("column projection", "select at least one variable"));
        }
        let mut namespace = HashSet::new();
        namespace
            .try_reserve(source_names.len())
            .map_err(|source| AdapterError::Allocation {
                name: "column projection".into(),
                channel: "source names",
                source,
            })?;
        for &name in source_names {
            if !namespace.insert(name) {
                return Err(binding(
                    "column projection",
                    format!("duplicate source field {name:?}"),
                ));
            }
        }
        let mut selected = HashSet::new();
        selected
            .try_reserve(columns.len())
            .map_err(|source| AdapterError::Allocation {
                name: "column projection".into(),
                channel: "selection identities",
                source,
            })?;
        for column in columns {
            let name = column.name().as_str();
            if !selected.insert(name) {
                return Err(binding(
                    "column projection",
                    format!("duplicate selected field {name:?}"),
                ));
            }
            if !namespace.contains(name) {
                return Err(binding(
                    "column projection",
                    format!("selected field {name:?} is absent from the source namespace"),
                ));
            }
            length("column projection", name, source_rows, column.len())?;
        }
        Ok(Self {
            source_rows,
            source_names,
            columns,
        })
    }

    /// Discover primitive numeric fields in source order using the same native
    /// predicate as the existing discovery constructors, without reading values.
    ///
    /// This does not import unrelated fields or check an external bridge's dtype
    /// fidelity. Explicit selections do not use this discovery filter.
    #[must_use]
    pub fn numeric_names(source: &DataFrame) -> Vec<&str> {
        crate::frontend::numeric_names(source)
    }
}

impl sealed::Sealed for &ColumnProjection<'_> {
    fn source_names_snapshot(&self) -> Option<Vec<String>> {
        Some(
            self.source_names
                .iter()
                .map(|name| (*name).into())
                .collect(),
        )
    }

    fn cell_shape(&self, _: Option<&DataFrame>) -> BindingResult<Option<(usize, usize)>> {
        Ok(Some((self.source_rows, self.columns.len())))
    }
}

impl ColumnsInput for &ColumnProjection<'_> {
    fn bind_columns(self, _: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
        let mut columns = crate::snapshot::allocate(self.columns.len(), "selected columns")?;
        columns.extend_from_slice(self.columns);
        Ok(columns)
    }
}

impl NumericSeriesInput for &ColumnProjection<'_> {
    fn bind_series(
        self,
        _: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding> {
        input::multiple(self.columns, false, role)
    }
}

impl BooleanInput for &ColumnProjection<'_> {
    fn bind_boolean(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<BooleanValues> {
        input::boolean_cells(self.columns)
    }

    fn bind_boolean_repeated(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
        _: usize,
    ) -> BindingResult<BooleanValues> {
        self.bind_boolean(source, role)
    }
}

impl SourceRowsInput for &ColumnProjection<'_> {
    fn bind_source_rows(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<SourceRows> {
        input::source_cells(self.columns)
    }

    fn bind_source_rows_repeated(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
        _: usize,
    ) -> BindingResult<SourceRows> {
        self.bind_source_rows(source, role)
    }
}

#[cfg(test)]
mod tests;
