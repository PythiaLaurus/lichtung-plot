//! Explicit row, expanded-cell and generated-column origins.
use super::{context, Context, PlotAdapter};
use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, ArraySource, CategoryInput, CategoryValues, ColumnsInput, FacetSpec,
    NumericInput, NumericSeriesInput, NumericValues, Plot, RaincloudInput, SourceRowsInput,
};

#[derive(Clone)]
pub(super) struct Expansion {
    pub(super) rows: usize,
    pub(super) keys: Vec<String>,
    pub(super) field: String,
    categories: CategoryValues,
    local: bool,
    aliases: Vec<(String, String)>,
}
impl Expansion {
    fn new(rows: usize, indices: Vec<usize>, local: bool, other: &str) -> BindingResult<Self> {
        let field = if other == "Variable" {
            "Variable_"
        } else {
            "Variable"
        }
        .to_owned();
        let keys = indices
            .iter()
            .map(|index| {
                if local {
                    format!("__lichtung_variable_{index}")
                } else {
                    crate::source::name(*index)
                }
            })
            .collect::<Vec<_>>();
        let aliases = if local {
            keys.iter()
                .cloned()
                .zip(indices.into_iter().map(crate::source::name))
                .collect()
        } else {
            Vec::new()
        };
        let categories = CategoryValues::new(&field, (0..rows).flat_map(|_| keys.iter().cloned()))?;
        Ok(Self {
            rows,
            keys,
            field,
            categories,
            local,
            aliases,
        })
    }
    pub(super) fn column_key(
        &self,
        source: Option<&dyn ArraySource>,
        input: impl SourceRowsInput,
    ) -> BindingResult<CategoryValues> {
        if input.cell_shape().is_some() || input.selected_columns().is_some() {
            return Err(binding(
                "column alpha selector",
                "expected one integer variable, not a cell grid",
            ));
        }
        Ok(CategoryValues::new(
            &self.field,
            input
                .raw_source_rows(source, "column alpha selector")?
                .into_iter()
                .map(|index| {
                    if self.local {
                        format!("__lichtung_variable_{index}")
                    } else {
                        crate::source::name(index)
                    }
                }),
        )?)
    }
}
macro_rules! numeric_family {
    ($family:ident,$native:ident,$group:expr) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Bind actual x coordinates and one numeric variable or an explicit many-y matrix/selection.
            pub fn new(
                x: impl NumericInput,
                y: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::wide_input(None, x, y)
            }
            /// Bind logical source columns/direct variables and retain later column lookup.
            pub fn from_data(
                source: impl ArraySource + 'source,
                x: impl NumericInput,
                y: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::wide_input(context(source)?, x, y)
            }
            fn wide_input(
                source: Context<'source>,
                x: impl NumericInput,
                y: impl NumericSeriesInput,
            ) -> BindingResult<Self> {
                let (y, shape) = y.bind_series(source.as_deref(), "y")?;
                let copies = shape.as_ref().map_or(1, |(_, columns, _)| columns.len());
                let x = x.bind_numeric_repeated(source.as_deref(), "x", copies)?;
                length("y", y.name(), x.len(), y.len())?;
                let expansion = shape
                    .map(|(rows, indices, local)| Expansion::new(rows, indices, local, x.name()))
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
            /// Explicitly choose zero-based logical row position as x.
            pub fn indexed(
                variables: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::indexed_input(None, variables)
            }
            /// Use row position and an ordered logical-column selection, including `..`.
            pub fn indexed_from_data(
                source: impl ArraySource + 'source,
                columns: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::indexed_input(context(source)?, columns)
            }
            /// Explicit numeric discovery on a homogeneous source uses all columns.
            pub fn indexed_numeric_from_data(
                source: impl ArraySource + 'source,
            ) -> std::result::Result<Self, AdapterError> {
                Self::indexed_from_data(source, ..)
            }
            fn indexed_input(
                source: Context<'source>,
                columns: impl ColumnsInput,
            ) -> BindingResult<Self> {
                let local = columns.local_identities();
                let (values, shape) =
                    crate::input::multiple(columns.bind_columns(source.as_deref())?, local, "y")?;
                let (rows, indices, local) =
                    shape.ok_or_else(|| binding("indexed", "expected selected columns"))?;
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "row positions use ordinary f64 normalization"
                )]
                let x = NumericValues::new(
                    "Row",
                    (0..rows).flat_map(|row| std::iter::repeat_n(row as f64, indices.len())),
                );
                let expansion = Expansion::new(rows, indices, local, "Row")?;
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
    ($family:ident,$native:expr) => {
        impl<'source> PlotAdapter<'source, kind::$family> {
            /// Bind categories and one or explicitly multiple sample variables.
            pub fn new(
                category: impl CategoryInput,
                values: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::sample_input(None, category, values)
            }
            /// Bind source columns/direct samples; generated column identity supplies positioned series.
            pub fn from_data(
                source: impl ArraySource + 'source,
                category: impl CategoryInput,
                values: impl NumericSeriesInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::sample_input(context(source)?, category, values)
            }
            fn sample_input(
                source: Context<'source>,
                category: impl CategoryInput,
                values: impl NumericSeriesInput,
            ) -> BindingResult<Self> {
                let (values, shape) = values.bind_series(source.as_deref(), "value")?;
                let copies = shape.as_ref().map_or(1, |(_, columns, _)| columns.len());
                let category =
                    category.bind_category_repeated(source.as_deref(), "category", copies)?;
                length("samples", values.name(), category.len(), values.len())?;
                let expansion = shape
                    .map(|(rows, indices, local)| {
                        Expansion::new(rows, indices, local, category.name())
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
            /// Use logical column identities as categories, with cells as observations.
            pub fn columns(
                variables: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::columns_input(None, variables)
            }
            /// Select ordered logical source columns for column-category distributions.
            pub fn columns_from_data(
                source: impl ArraySource + 'source,
                columns: impl ColumnsInput,
            ) -> std::result::Result<Self, AdapterError> {
                Self::columns_input(context(source)?, columns)
            }
            /// Explicit numeric discovery on a homogeneous source uses all columns.
            pub fn numeric_columns_from_data(
                source: impl ArraySource + 'source,
            ) -> std::result::Result<Self, AdapterError> {
                Self::columns_from_data(source, ..)
            }
            fn columns_input(
                source: Context<'source>,
                columns: impl ColumnsInput,
            ) -> BindingResult<Self> {
                let local = columns.local_identities();
                let (values, shape) = crate::input::multiple(
                    columns.bind_columns(source.as_deref())?,
                    local,
                    "value",
                )?;
                let (rows, indices, local) =
                    shape.ok_or_else(|| binding("columns", "expected selected columns"))?;
                let expansion = Expansion::new(rows, indices, local, values.name())?;
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
            .map(|wide| wide.categories.clone())
            .ok_or_else(|| {
                binding(
                    "column identity",
                    "this plot has no generated wide-column dimension",
                )
            })
    }
    /// Change selected/generated column labels without changing logical column identity.
    pub fn column_labels<I, S>(mut self, labels: I) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let keys = self
            .expansion
            .as_ref()
            .map(|wide| &wide.keys)
            .or(self.matrix_keys.as_ref())
            .ok_or_else(|| binding("column labels", "no generated column dimension"))?;
        let labels = labels.into_iter().map(Into::into).collect::<Vec<_>>();
        length(
            "column labels",
            "selected variables",
            keys.len(),
            labels.len(),
        )?;
        self.labels = self.labels.aliases(keys.iter().cloned().zip(labels));
        self.plot = self.plot.labels(self.labels.clone());
        Ok(self)
    }
    /// Bind already-expanded unique IDs without repeating any source row identity.
    pub fn source_rows_cells(
        mut self,
        values: impl SourceRowsInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        K: kind::SourceRows,
    {
        self.cell_shape(&values, "source rows")?;
        let values = values.bind_source_rows(self.source.as_deref(), "source rows")?;
        self.aligned("source rows", "source rows", values.len())?;
        self.plot = self.plot.source_rows(values);
        Ok(self)
    }
}
macro_rules! column_role {
    ($method:ident,$cells:ident,$native:ident,$cap:ident) => {
        impl<K: kind::$cap> PlotAdapter<'_, K> {
            /// Use the generated logical-column dimension in this native role.
            pub fn $method(mut self) -> std::result::Result<Self, AdapterError> {
                let values = self.generated()?;
                self.plot = self.plot.$native(values);
                Ok(self)
            }
            /// Bind explicitly expanded categories without source-row repetition.
            pub fn $cells(
                mut self,
                values: impl CategoryInput,
            ) -> std::result::Result<Self, AdapterError> {
                let values = values.bind_category(self.source.as_deref(), stringify!($native))?;
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
    /// Facet rows by generated logical-column identity.
    pub fn facet_columns_row(self) -> std::result::Result<Self, AdapterError> {
        let values = self.generated()?;
        Ok(self.facet(FacetSpec::row(values)))
    }
    /// Facet columns by generated logical-column identity.
    pub fn facet_columns_column(self) -> std::result::Result<Self, AdapterError> {
        let values = self.generated()?;
        Ok(self.facet(FacetSpec::column(values)))
    }
    /// Wrap the generated logical-column dimension.
    pub fn facet_columns_wrap(self, columns: usize) -> std::result::Result<Self, AdapterError> {
        let values = self.generated()?;
        Ok(self.facet(FacetSpec::wrap(values, columns)?))
    }
    /// Facet rows by explicitly expanded categorical slots.
    pub fn facet_row_cells(
        self,
        values: impl CategoryInput,
    ) -> std::result::Result<Self, AdapterError> {
        let values = values.bind_category(self.source.as_deref(), "facet row")?;
        self.aligned("facet row", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::row(values)))
    }
    /// Facet columns by explicitly expanded categorical slots.
    pub fn facet_column_cells(
        self,
        values: impl CategoryInput,
    ) -> std::result::Result<Self, AdapterError> {
        let values = values.bind_category(self.source.as_deref(), "facet column")?;
        self.aligned("facet column", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::column(values)))
    }
    /// Wrap explicitly expanded categorical slots.
    pub fn facet_wrap_cells(
        self,
        values: impl CategoryInput,
        columns: usize,
    ) -> std::result::Result<Self, AdapterError> {
        let values = values.bind_category(self.source.as_deref(), "facet wrap")?;
        self.aligned("facet wrap", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::wrap(values, columns)?))
    }
}
macro_rules! grid_facet {
    ($name:ident,$row:expr,$column:expr) => {
        impl<K: kind::Facet> PlotAdapter<'_, K> {
            /// Cross explicitly row/cell-scoped facets, never inferring scope from length.
            pub fn $name(
                self,
                row: impl CategoryInput,
                column: impl CategoryInput,
            ) -> std::result::Result<Self, AdapterError> {
                let row = row.bind_category_repeated(
                    self.source.as_deref(),
                    "facet row",
                    if $row { self.row_copies() } else { 1 },
                )?;
                let column = column.bind_category_repeated(
                    self.source.as_deref(),
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
    ($name:ident,$generated_row:expr,$cells:expr) => {
        impl<K: kind::Facet> PlotAdapter<'_, K> {
            /// Cross generated column identity with an explicitly scoped external facet.
            pub fn $name(
                self,
                other: impl CategoryInput,
            ) -> std::result::Result<Self, AdapterError> {
                let generated = self.generated()?;
                let other = other.bind_category_repeated(
                    self.source.as_deref(),
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
