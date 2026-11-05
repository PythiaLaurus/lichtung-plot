//! Explicit variable expansion and row/cell/generated-column scopes.

use polars_core::prelude::DataFrame;

use super::PlotAdapter;
use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, CategoryInput, CategoryValues, ColumnsInput, FacetSpec, NumericInput,
    NumericSeriesInput, NumericValues, Plot, RaincloudInput, SourceRowsInput,
};

#[derive(Clone)]
pub(super) struct Expansion {
    pub(super) rows: usize,
    pub(super) keys: Vec<String>,
    pub(super) field: String,
    categories: CategoryValues,
    aliases: Vec<(String, String)>,
    local: bool,
}

impl Expansion {
    fn new(
        rows: usize,
        labels: Vec<String>,
        local: bool,
        source: Option<&DataFrame>,
        other: &str,
    ) -> BindingResult<Self> {
        let mut field = "Variable".to_owned();
        while field == other
            || source.is_some_and(|source| {
                source
                    .columns()
                    .iter()
                    .any(|column| column.name().as_str() == field)
            })
        {
            field.push('_');
        }
        let keys = if local {
            (0..labels.len())
                .map(|index| format!("__lichtung_variable_{index}"))
                .collect()
        } else {
            labels.clone()
        };
        let aliases = if local {
            keys.iter().cloned().zip(labels).collect()
        } else {
            Vec::new()
        };
        let categories =
            CategoryValues::new(field.clone(), (0..rows).flat_map(|_| keys.iter().cloned()))?;
        Ok(Self {
            rows,
            keys,
            field,
            categories,
            aliases,
            local,
        })
    }

    pub(super) fn column_key(
        &self,
        source: Option<&DataFrame>,
        input: impl crate::ColumnInput,
    ) -> BindingResult<CategoryValues> {
        let column = input.bind_column(source, "column alpha selector")?;
        if self.local {
            let indices = crate::provenance::ids(&column)?;
            Ok(CategoryValues::new(
                &self.field,
                indices
                    .into_iter()
                    .map(|index| format!("__lichtung_variable_{index}")),
            )?)
        } else {
            Ok(CategoryValues::nullable(
                &self.field,
                crate::category::values(&column)?,
            )?)
        }
    }
}

pub(super) fn numeric_names(source: &DataFrame) -> Vec<&str> {
    source
        .columns()
        .iter()
        .filter(|column| column.dtype().is_primitive_numeric())
        .map(|column| column.name().as_str())
        .collect()
}

macro_rules! numeric_family {
    ($family:ident, $native:ident, $group:expr) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Bind an actual x coordinate and one or an explicitly ordered collection of y variables.
            pub fn new(
                x: impl NumericInput,
                y: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::wide_input(None, x, y)
            }
            /// Bind actual x/y roles and retain the source for later row-scoped setters.
            pub fn from_data(
                source: &'source DataFrame,
                x: impl NumericInput,
                y: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::wide_input(Some(source), x, y)
            }
            fn wide_input(
                source: Option<&'source DataFrame>,
                x: impl NumericInput,
                y: impl NumericSeriesInput,
            ) -> BindingResult<Self> {
                let (y, shape) = y.bind_series(source, "y")?;
                let copies = shape.as_ref().map_or(1, |(_, columns, _)| columns.len());
                let x = x.bind_numeric_repeated(source, "x", copies)?;
                length("y", y.name(), x.len(), y.len())?;
                let expansion = shape
                    .map(|(rows, labels, local)| {
                        Expansion::new(rows, labels, local, source, x.name())
                    })
                    .transpose()?;
                let len = y.len();
                let mut plot = Plot::$native(x, y);
                if let Some(expansion) = &expansion {
                    plot = plot.hue(expansion.categories.clone());
                    plot = ($group)(plot, expansion.categories.clone());
                }
                let mut adapter = Self::bound(plot, source, Some(len));
                adapter.install_expansion(expansion);
                Ok(adapter)
            }
            /// Explicitly choose zero-based source row position as x.
            pub fn indexed(
                variables: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::indexed_input(None, variables)
            }
            /// Use row position and an ordered field selection; `..` means every field.
            pub fn indexed_from_data(
                source: &'source DataFrame,
                columns: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::indexed_input(Some(source), columns)
            }
            /// Discover supported numeric columns explicitly, then use row position as x.
            pub fn indexed_numeric_from_data(
                source: &'source DataFrame,
            ) -> std::result::Result<Self, AdapterError> {
                Self::indexed_input(Some(source), numeric_names(source).as_slice())
            }
            fn indexed_input(
                source: Option<&'source DataFrame>,
                columns: impl ColumnsInput,
            ) -> BindingResult<Self> {
                let local = columns.local_identities();
                let (values, shape) =
                    crate::input::multiple(&columns.bind_columns(source)?, local, "y")?;
                let (rows, labels, local) =
                    shape.ok_or_else(|| binding("indexed", "expected selected columns"))?;
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "row positions follow ordinary numeric f64 normalization"
                )]
                let x = NumericValues::new(
                    "Row",
                    (0..rows).flat_map(|row| std::iter::repeat_n(row as f64, labels.len())),
                );
                let expansion = Expansion::new(rows, labels, local, source, "Row")?;
                let len = values.len();
                let plot = Plot::$native(x, values).hue(expansion.categories.clone());
                let plot = ($group)(plot, expansion.categories.clone());
                let mut adapter = Self::bound(plot, source, Some(len));
                adapter.install_expansion(Some(expansion));
                Ok(adapter)
            }
        }
    };
}
numeric_family!(
    Scatter,
    scatter,
    |plot: Plot<kind::Scatter>, _: CategoryValues| plot
);
numeric_family!(Line, line, |plot: Plot<kind::Line>, groups| plot
    .group(groups));

macro_rules! distribution_family {
    ($family:ident, $native:expr) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Bind categories and one or multiple sample variables; column identity owns positioned series.
            pub fn new(
                category: impl CategoryInput,
                values: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::sample_input(None, category, values)
            }
            /// Bind category/sample roles with retained source lookup.
            pub fn from_data(
                source: &'source DataFrame,
                category: impl CategoryInput,
                values: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::sample_input(Some(source), category, values)
            }
            fn sample_input(
                source: Option<&'source DataFrame>,
                category: impl CategoryInput,
                values: impl NumericSeriesInput,
            ) -> BindingResult<Self> {
                let (values, shape) = values.bind_series(source, "value")?;
                let copies = shape.as_ref().map_or(1, |(_, columns, _)| columns.len());
                let category = category.bind_category_repeated(source, "category", copies)?;
                length("samples", values.name(), category.len(), values.len())?;
                let expansion = shape
                    .map(|(rows, labels, local)| {
                        Expansion::new(rows, labels, local, source, category.name())
                    })
                    .transpose()?;
                let len = values.len();
                let mut plot = ($native)(category, values);
                if let Some(expansion) = &expansion {
                    plot = plot.series(expansion.categories.clone());
                }
                let mut adapter = Self::bound(plot, source, Some(len));
                adapter.install_expansion(expansion);
                Ok(adapter)
            }
            /// Use selected column identities as categories, with cells as observations.
            pub fn columns(
                variables: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::columns_input(None, variables)
            }
            /// Bind an ordered field selection as column-category distributions.
            pub fn columns_from_data(
                source: &'source DataFrame,
                columns: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::columns_input(Some(source), columns)
            }
            /// Explicitly discover numeric fields for column-category distributions.
            pub fn numeric_columns_from_data(
                source: &'source DataFrame,
            ) -> std::result::Result<Self, AdapterError> {
                Self::columns_input(Some(source), numeric_names(source).as_slice())
            }
            fn columns_input(
                source: Option<&'source DataFrame>,
                columns: impl ColumnsInput,
            ) -> BindingResult<Self> {
                let local = columns.local_identities();
                let (values, shape) =
                    crate::input::multiple(&columns.bind_columns(source)?, local, "value")?;
                let (rows, labels, local) =
                    shape.ok_or_else(|| binding("columns", "expected selected columns"))?;
                let expansion = Expansion::new(rows, labels, local, source, values.name())?;
                let len = values.len();
                let plot = ($native)(expansion.categories.clone(), values);
                let mut adapter = Self::bound(plot, source, Some(len));
                adapter.install_expansion(Some(expansion));
                Ok(adapter)
            }
        }
    };
}
distribution_family!(Boxplot, Plot::boxplot);
distribution_family!(Violin, Plot::violin);
distribution_family!(Raincloud, |category, values| Plot::raincloud(
    RaincloudInput::samples(category, values)
));

impl<K: kind::Marker> PlotAdapter<'_, K> {
    fn install_expansion(&mut self, expansion: Option<Expansion>) {
        if let Some(expansion) = &expansion {
            if !expansion.aliases.is_empty() {
                self.labels = self.labels.clone().aliases(expansion.aliases.clone());
                self.plot = self.plot.clone().labels(self.labels.clone());
            }
        }
        self.expansion = expansion;
    }
    fn generated(&self) -> BindingResult<CategoryValues> {
        self.expansion
            .as_ref()
            .map(|expansion| expansion.categories.clone())
            .ok_or_else(|| {
                binding(
                    "column identity",
                    "this plot has no generated wide-column dimension",
                )
            })
    }
    /// Change selected/generated variable display labels, not source identities or indices.
    pub fn column_labels<I, S>(mut self, labels: I) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let keys = if let Some(expansion) = &self.expansion {
            expansion.keys.clone()
        } else if let Some(keys) = &self.matrix_keys {
            keys.clone()
        } else {
            return Err(binding(
                "column labels",
                "this plot has no generated column dimension",
            ));
        };
        let labels = labels.into_iter().map(Into::into).collect::<Vec<String>>();
        length(
            "column labels",
            "selected variables",
            keys.len(),
            labels.len(),
        )?;
        self.labels = self.labels.aliases(keys.into_iter().zip(labels));
        self.plot = self.plot.labels(self.labels.clone());
        Ok(self)
    }
    /// Bind unique already-expanded IDs, never repeating a source-row identity.
    pub fn source_rows_cells(
        mut self,
        values: impl SourceRowsInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        K: kind::SourceRows,
    {
        self.cell_shape(&values, "source rows")?;
        let values = values.bind_source_rows(self.source, "source rows")?;
        self.aligned("source rows", "source rows", values.len())?;
        self.plot = self.plot.source_rows(values);
        Ok(self)
    }
}

macro_rules! column_role {
    ($method:ident, $cells:ident, $native:ident, $cap:ident) => {
        impl<K: kind::$cap> PlotAdapter<'_, K> {
            /// Use the generated variable dimension in this native role.
            pub fn $method(mut self) -> std::result::Result<Self, AdapterError> {
                let values = self.generated()?;
                self.plot = self.plot.$native(values);
                Ok(self)
            }
            /// Bind explicitly expanded categorical slots, without source-row repetition.
            pub fn $cells(
                mut self,
                values: impl CategoryInput,
            ) -> std::result::Result<Self, AdapterError> {
                let values = values.bind_category(self.source, stringify!($native))?;
                self.aligned(stringify!($native), values.name(), values.len())?;
                self.plot = self.plot.$native(values);
                Ok(self)
            }
        }
    };
}
column_role!(hue_columns, hue_cells, hue, Hue);
column_role!(group_columns, group_cells, group, Group);
column_role!(series_columns, series_cells, series, Series);

impl<K: kind::Facet> PlotAdapter<'_, K> {
    /// Facet rows by generated wide-column identity.
    pub fn facet_columns_row(self) -> std::result::Result<Self, AdapterError> {
        let values = self.generated()?;
        Ok(self.facet(FacetSpec::row(values)))
    }
    /// Facet columns by generated wide-column identity.
    pub fn facet_columns_column(self) -> std::result::Result<Self, AdapterError> {
        let values = self.generated()?;
        Ok(self.facet(FacetSpec::column(values)))
    }
    /// Wrap generated wide-column identity.
    pub fn facet_columns_wrap(self, columns: usize) -> std::result::Result<Self, AdapterError> {
        let values = self.generated()?;
        Ok(self.facet(FacetSpec::wrap(values, columns)?))
    }
    /// Facet rows by explicitly expanded values.
    pub fn facet_row_cells(
        self,
        values: impl CategoryInput,
    ) -> std::result::Result<Self, AdapterError> {
        let values = values.bind_category(self.source, "facet row")?;
        self.aligned("facet row", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::row(values)))
    }
    /// Facet columns by explicitly expanded values.
    pub fn facet_column_cells(
        self,
        values: impl CategoryInput,
    ) -> std::result::Result<Self, AdapterError> {
        let values = values.bind_category(self.source, "facet column")?;
        self.aligned("facet column", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::column(values)))
    }
    /// Wrap explicitly expanded values.
    pub fn facet_wrap_cells(
        self,
        values: impl CategoryInput,
        columns: usize,
    ) -> std::result::Result<Self, AdapterError> {
        let values = values.bind_category(self.source, "facet wrap")?;
        self.aligned("facet wrap", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::wrap(values, columns)?))
    }
}

macro_rules! grid_facet {
    ($name:ident, $row:expr, $column:expr) => {
        impl<K: kind::Facet> PlotAdapter<'_, K> {
            /// Cross explicit row/cell-scoped facet variables without length-based scope inference.
            pub fn $name(
                self,
                row: impl CategoryInput,
                column: impl CategoryInput,
            ) -> std::result::Result<Self, AdapterError> {
                let row = row.bind_category_repeated(
                    self.source,
                    "facet row",
                    if $row { self.row_copies() } else { 1 },
                )?;
                let column = column.bind_category_repeated(
                    self.source,
                    "facet column",
                    if $column { self.row_copies() } else { 1 },
                )?;
                self.aligned("facet row", row.name(), row.len())?;
                self.aligned("facet column", column.name(), column.len())?;
                Ok(self.facet(FacetSpec::grid(row, column)?))
            }
        }
    };
}
grid_facet!(facet_grid_row_cells, false, true);
grid_facet!(facet_grid_column_cells, true, false);
grid_facet!(facet_grid_cells, false, false);

macro_rules! generated_grid {
    ($name:ident, $generated_row:expr, $cells:expr) => {
        impl<K: kind::Facet> PlotAdapter<'_, K> {
            /// Cross the generated column dimension with an explicitly scoped external facet.
            pub fn $name(
                self,
                other: impl CategoryInput,
            ) -> std::result::Result<Self, AdapterError> {
                let generated = self.generated()?;
                let other = other.bind_category_repeated(
                    self.source,
                    "facet",
                    if $cells { 1 } else { self.row_copies() },
                )?;
                self.aligned("facet", other.name(), other.len())?;
                let facet = if $generated_row {
                    FacetSpec::grid(generated, other)?
                } else {
                    FacetSpec::grid(other, generated)?
                };
                Ok(self.facet(facet))
            }
        }
    };
}
generated_grid!(facet_grid_columns_row, true, false);
generated_grid!(facet_grid_columns_column, false, false);
generated_grid!(facet_grid_columns_row_cells, true, true);
generated_grid!(facet_grid_columns_column_cells, false, true);
