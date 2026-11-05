//! Sealed role bindings; caller inputs are snapshotted at the owning operation.

use std::collections::HashSet;
use std::ops::RangeFull;

use polars_core::prelude::{Column, DataFrame, DataType, IntoColumn, Series};

use crate::error::{binding, BindingResult};
use crate::{AdapterError, AxisValues, BooleanValues, CategoryValues, NumericValues, SourceRows};

pub(crate) mod sealed {
    pub trait Sealed {
        fn native_slots(&self) -> bool {
            false
        }
        fn cell_shape(
            &self,
            _: Option<&polars_core::prelude::DataFrame>,
        ) -> super::BindingResult<Option<(usize, usize)>> {
            Ok(None)
        }
    }
}

impl sealed::Sealed for &str {}
impl sealed::Sealed for &String {}
impl sealed::Sealed for &Column {}
impl sealed::Sealed for &Series {}
impl sealed::Sealed for NumericValues {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for CategoryValues {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for BooleanValues {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for SourceRows {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for AxisValues {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for RangeFull {}
impl sealed::Sealed for &DataFrame {
    fn cell_shape(&self, _: Option<&DataFrame>) -> BindingResult<Option<(usize, usize)>> {
        Ok(Some((self.height(), self.width())))
    }
}

pub(crate) fn lookup<'a>(
    source: Option<&'a DataFrame>,
    name: &str,
    role: &'static str,
) -> BindingResult<&'a Column> {
    source
        .ok_or_else(|| AdapterError::MissingSource {
            name: name.into(),
            role,
        })?
        .column(name)
        .map_err(|source| AdapterError::Lookup {
            name: name.into(),
            role,
            source,
        })
}

fn context(error: AdapterError, role: &'static str) -> AdapterError {
    match error {
        AdapterError::UnsupportedDtype { name, dtype, .. } => AdapterError::UnsupportedDtype {
            name,
            dtype,
            channel: role,
        },
        other => other,
    }
}

/// A foreign field or borrowed column, resolved only in the owning operation.
pub trait ColumnInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_column(self, source: Option<&DataFrame>, role: &'static str) -> BindingResult<Column>;
}
impl ColumnInput for &Column {
    fn bind_column(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<Column> {
        Ok(self.clone())
    }
}
impl ColumnInput for &Series {
    fn bind_column(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<Column> {
        Ok(self.clone().into_column())
    }
}
impl ColumnInput for &str {
    fn bind_column(self, source: Option<&DataFrame>, role: &'static str) -> BindingResult<Column> {
        Ok(lookup(source, self, role)?.clone())
    }
}
impl ColumnInput for &String {
    fn bind_column(self, source: Option<&DataFrame>, role: &'static str) -> BindingResult<Column> {
        self.as_str().bind_column(source, role)
    }
}

fn repeat_numeric(column: &Column, copies: usize) -> BindingResult<NumericValues> {
    let original = crate::numeric::values(column)?;
    let len = original
        .len()
        .checked_mul(copies)
        .ok_or_else(|| crate::error::binding("expansion", "slot count exceeds usize"))?;
    let mut values = crate::snapshot::allocate(len, "numeric expansion")?;
    values.extend(
        original
            .into_iter()
            .flat_map(|value| std::iter::repeat_n(value, copies)),
    );
    Ok(NumericValues::nullable(column.name().as_str(), values))
}
fn repeat_category(column: &Column, copies: usize) -> BindingResult<CategoryValues> {
    let original = crate::category::values(column)?;
    let len = original
        .len()
        .checked_mul(copies)
        .ok_or_else(|| crate::error::binding("expansion", "slot count exceeds usize"))?;
    let mut values = crate::snapshot::allocate(len, "category expansion")?;
    values.extend(
        original
            .into_iter()
            .flat_map(|value| std::iter::repeat_n(value, copies)),
    );
    Ok(CategoryValues::nullable(column.name().as_str(), values)?)
}

macro_rules! role_input {
    ($trait:ident, $method:ident, $repeated:ident, $repeat:path, $value:ty, $convert:path, $doc:literal) => {
        #[doc = $doc]
        /// This protocol is sealed; callers pass supported values without importing it.
        pub trait $trait: sealed::Sealed + Sized {
            #[doc(hidden)]
            fn $method(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<$value>;
            #[doc(hidden)]
            fn $repeated(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
                copies: usize,
            ) -> BindingResult<$value>;
        }
        impl $trait for &Column {
            fn $method(self, _: Option<&DataFrame>, role: &'static str) -> BindingResult<$value> {
                $convert(self).map_err(|error| context(error, role))
            }
            fn $repeated(
                self,
                _: Option<&DataFrame>,
                role: &'static str,
                copies: usize,
            ) -> BindingResult<$value> {
                $repeat(self, copies).map_err(|error| context(error, role))
            }
        }
        impl $trait for &Series {
            fn $method(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<$value> {
                (&self.clone().into_column()).$method(source, role)
            }
            fn $repeated(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
                copies: usize,
            ) -> BindingResult<$value> {
                (&self.clone().into_column()).$repeated(source, role, copies)
            }
        }
        impl $trait for &str {
            fn $method(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<$value> {
                lookup(source, self, role)?.$method(source, role)
            }
            fn $repeated(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
                copies: usize,
            ) -> BindingResult<$value> {
                lookup(source, self, role)?.$repeated(source, role, copies)
            }
        }
        impl $trait for &String {
            fn $method(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<$value> {
                self.as_str().$method(source, role)
            }
            fn $repeated(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
                copies: usize,
            ) -> BindingResult<$value> {
                self.as_str().$repeated(source, role, copies)
            }
        }
        impl $trait for $value {
            fn $method(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<$value> {
                Ok(self)
            }
            fn $repeated(
                self,
                _: Option<&DataFrame>,
                _: &'static str,
                _: usize,
            ) -> BindingResult<$value> {
                Ok(self)
            }
        }
    };
}

role_input!(
    NumericInput,
    bind_numeric,
    bind_numeric_repeated,
    repeat_numeric,
    NumericValues,
    crate::numeric::numeric,
    "A numeric field, borrowed Polars variable or owned native numeric value."
);
role_input!(
    CategoryInput,
    bind_category,
    bind_category_repeated,
    repeat_category,
    CategoryValues,
    crate::category::categories,
    "A categorical field, borrowed Polars variable or owned native category value."
);
role_input!(
    BooleanInput,
    bind_boolean,
    bind_boolean_repeated,
    repeat_boolean,
    BooleanValues,
    crate::boolean::booleans,
    "A Boolean field, borrowed Polars variable or owned native Boolean value."
);
role_input!(
    SourceRowsInput,
    bind_source_rows,
    bind_source_rows_repeated,
    repeat_source_rows,
    SourceRows,
    crate::provenance::source_rows,
    "Exact integer source IDs from a field, borrowed Polars variable or native IDs."
);

fn repeat_boolean(column: &Column, copies: usize) -> BindingResult<BooleanValues> {
    let values = crate::boolean::values(column)?;
    let len = values
        .len()
        .checked_mul(copies)
        .ok_or_else(|| binding("Boolean expansion", "slot count exceeds usize"))?;
    let mut expanded = crate::snapshot::allocate(len, "Boolean expansion")?;
    expanded.extend(
        values
            .into_iter()
            .flat_map(|value| std::iter::repeat_n(value, copies)),
    );
    Ok(BooleanValues::nullable(column.name().as_str(), expanded))
}
fn repeat_source_rows(column: &Column, copies: usize) -> BindingResult<SourceRows> {
    if copies != 1 {
        return Err(binding(
            "source rows",
            "provide unique per-cell IDs, not repeated row IDs",
        ));
    }
    crate::provenance::source_rows(column)
}

pub(crate) type SeriesBinding = (NumericValues, Option<(usize, Vec<String>, bool)>);

/// One numeric variable or an explicit ordered many-variable collection.
pub trait NumericSeriesInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_series(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding>;
}
macro_rules! single_series {
    ($($ty:ty),+) => {$ (
        impl NumericSeriesInput for $ty {
            fn bind_series(self, source: Option<&DataFrame>, role: &'static str) -> BindingResult<SeriesBinding> {
                Ok((self.bind_numeric(source, role)?, None))
            }
        }
    )+};
}
single_series!(&str, &String, &Column, &Series, NumericValues);

pub(crate) fn multiple(
    columns: &[Column],
    local: bool,
    role: &'static str,
) -> BindingResult<SeriesBinding> {
    let rows = columns[0].len();
    let cells = rows
        .checked_mul(columns.len())
        .ok_or_else(|| binding(role, "slot count exceeds usize"))?;
    let mut payloads = crate::snapshot::allocate(columns.len(), role)?;
    for column in columns {
        crate::error::length(role, column.name().as_str(), rows, column.len())?;
        payloads.push(crate::numeric::values(column).map_err(|error| context(error, role))?);
    }
    let mut values = crate::snapshot::allocate(cells, role)?;
    for row in 0..rows {
        for column in &payloads {
            values.push(column[row]);
        }
    }
    let labels = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            if column.name().is_empty() {
                format!("Column {}", index + 1)
            } else {
                column.name().to_string()
            }
        })
        .collect();
    Ok((
        NumericValues::nullable(role, values),
        Some((rows, labels, local)),
    ))
}
impl NumericSeriesInput for &[&str] {
    fn bind_series(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding> {
        multiple(&self.bind_columns(source)?, false, role)
    }
}
impl<const N: usize> NumericSeriesInput for [&str; N] {
    fn bind_series(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding> {
        multiple(&self.bind_columns(source)?, false, role)
    }
}
macro_rules! many_series {
    ($ty:ty) => {
        impl NumericSeriesInput for &[$ty] {
            fn bind_series(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<SeriesBinding> {
                multiple(&self.bind_columns(source)?, true, role)
            }
        }
        impl<const N: usize> NumericSeriesInput for [$ty; N] {
            fn bind_series(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<SeriesBinding> {
                multiple(&self.bind_columns(source)?, true, role)
            }
        }
    };
}
many_series!(&Column);
many_series!(&Series);

fn boolean_cells(columns: &[Column]) -> BindingResult<BooleanValues> {
    let rows = columns[0].len();
    let cells = rows
        .checked_mul(columns.len())
        .ok_or_else(|| binding("highlights", "cell count exceeds usize"))?;
    let mut payloads = crate::snapshot::allocate(columns.len(), "highlights")?;
    for column in columns {
        crate::error::length("highlights", column.name().as_str(), rows, column.len())?;
        payloads.push(crate::boolean::values(column)?);
    }
    let mut values = crate::snapshot::allocate(cells, "highlights")?;
    for row in 0..rows {
        for column in &payloads {
            values.push(column[row]);
        }
    }
    Ok(BooleanValues::nullable("Highlights", values))
}
fn source_cells(columns: &[Column]) -> BindingResult<SourceRows> {
    let rows = columns[0].len();
    let cells = rows
        .checked_mul(columns.len())
        .ok_or_else(|| binding("source rows", "cell count exceeds usize"))?;
    let mut payloads = crate::snapshot::allocate(columns.len(), "source rows")?;
    for column in columns {
        crate::error::length("source rows", column.name().as_str(), rows, column.len())?;
        payloads.push(crate::provenance::ids(column)?);
    }
    let mut values = crate::snapshot::allocate(cells, "source rows")?;
    for row in 0..rows {
        for column in &payloads {
            values.push(column[row]);
        }
    }
    Ok(SourceRows::new(values)?)
}
macro_rules! cell_inputs {
    ($ty:ty) => {
        impl BooleanInput for $ty {
            fn bind_boolean(
                self,
                source: Option<&DataFrame>,
                _: &'static str,
            ) -> BindingResult<BooleanValues> {
                boolean_cells(&self.bind_columns(source)?)
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
        impl SourceRowsInput for $ty {
            fn bind_source_rows(
                self,
                source: Option<&DataFrame>,
                _: &'static str,
            ) -> BindingResult<SourceRows> {
                source_cells(&self.bind_columns(source)?)
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
    };
}
cell_inputs!(&[&str]);
cell_inputs!(&[&Column]);
cell_inputs!(&[&Series]);
cell_inputs!(&DataFrame);
macro_rules! cell_arrays {
    ($ty:ty) => {
        impl<const N: usize> BooleanInput for [$ty; N] {
            fn bind_boolean(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<BooleanValues> {
                self.as_slice().bind_boolean(source, role)
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
        impl<const N: usize> SourceRowsInput for [$ty; N] {
            fn bind_source_rows(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<SourceRows> {
                self.as_slice().bind_source_rows(source, role)
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
    };
}
cell_arrays!(&str);
cell_arrays!(&Column);
cell_arrays!(&Series);

/// A numeric or categorical trajectory coordinate, inferred only from admitted dtypes.
/// Use the explicit numeric/categorical-x constructors for an ambiguous Null dtype.
pub trait AxisInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_axis(self, source: Option<&DataFrame>, role: &'static str)
        -> BindingResult<AxisValues>;
}

impl AxisInput for &Column {
    fn bind_axis(self, source: Option<&DataFrame>, _: &'static str) -> BindingResult<AxisValues> {
        match self.dtype() {
            DataType::String | DataType::Categorical(..) | DataType::Enum(..) => {
                self.bind_category(source, "x").map(AxisValues::Category)
            }
            DataType::Null => Err(binding(
                "x",
                "Null dtype is ambiguous; use an explicit numeric-x or categorical-x constructor",
            )),
            _ => self.bind_numeric(source, "x").map(AxisValues::Numeric),
        }
    }
}
impl AxisInput for &Series {
    fn bind_axis(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<AxisValues> {
        (&self.clone().into_column()).bind_axis(source, role)
    }
}
impl AxisInput for &str {
    fn bind_axis(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<AxisValues> {
        lookup(source, self, role)?.bind_axis(source, role)
    }
}
impl AxisInput for &String {
    fn bind_axis(
        self,
        source: Option<&DataFrame>,
        role: &'static str,
    ) -> BindingResult<AxisValues> {
        self.as_str().bind_axis(source, role)
    }
}
impl AxisInput for AxisValues {
    fn bind_axis(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<AxisValues> {
        Ok(self)
    }
}
impl AxisInput for NumericValues {
    fn bind_axis(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<AxisValues> {
        Ok(AxisValues::Numeric(self))
    }
}
impl AxisInput for CategoryValues {
    fn bind_axis(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<AxisValues> {
        Ok(AxisValues::Category(self))
    }
}

/// Ordered columns for a rectangular heatmap or scatter matrix.
/// Names select exact fields; `..` selects all logical fields, not only numeric ones.
pub trait ColumnsInput: sealed::Sealed {
    #[doc(hidden)]
    fn local_identities(&self) -> bool {
        false
    }
    #[doc(hidden)]
    fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>>;
}

fn selected(source: Option<&DataFrame>, names: &[&str]) -> BindingResult<Vec<Column>> {
    let mut columns = crate::snapshot::allocate(names.len(), "selected columns")?;
    let mut seen = HashSet::new();
    seen.try_reserve(names.len())
        .map_err(|source| AdapterError::Allocation {
            name: "selected columns".into(),
            channel: "selection identities",
            source,
        })?;
    for &name in names {
        if !seen.insert(name) {
            return Err(binding(
                "selected columns",
                format!("duplicate field {name:?}"),
            ));
        }
        columns.push(lookup(source, name, "selected columns")?.clone());
    }
    if columns.is_empty() {
        return Err(binding("selected columns", "select at least one variable"));
    }
    Ok(columns)
}

impl sealed::Sealed for &[&str] {
    fn cell_shape(&self, source: Option<&DataFrame>) -> BindingResult<Option<(usize, usize)>> {
        columns_shape(&self.bind_columns(source)?)
    }
}
impl ColumnsInput for &[&str] {
    fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
        selected(source, self)
    }
}
impl<const N: usize> sealed::Sealed for [&str; N] {
    fn cell_shape(&self, source: Option<&DataFrame>) -> BindingResult<Option<(usize, usize)>> {
        self.as_slice().cell_shape(source)
    }
}
impl<const N: usize> ColumnsInput for [&str; N] {
    fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
        selected(source, &self)
    }
}
impl ColumnsInput for RangeFull {
    fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
        let frame = source.ok_or_else(|| {
            binding(
                "selected columns",
                "all fields require a DataFrame lookup context",
            )
        })?;
        selected(
            source,
            &frame
                .columns()
                .iter()
                .map(|column| column.name().as_str())
                .collect::<Vec<_>>(),
        )
    }
}
impl ColumnsInput for &DataFrame {
    fn bind_columns(self, _: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
        (..).bind_columns(Some(self))
    }
}

macro_rules! direct_columns {
    ($ty:ty, $convert:expr) => {
        impl sealed::Sealed for &[$ty] {
            fn cell_shape(
                &self,
                source: Option<&DataFrame>,
            ) -> BindingResult<Option<(usize, usize)>> {
                columns_shape(&self.bind_columns(source)?)
            }
        }
        impl ColumnsInput for &[$ty] {
            fn local_identities(&self) -> bool {
                true
            }
            fn bind_columns(self, _: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
                if self.is_empty() {
                    return Err(binding("selected columns", "select at least one variable"));
                }
                let mut columns = crate::snapshot::allocate(self.len(), "direct columns")?;
                columns.extend(self.iter().map($convert));
                Ok(columns)
            }
        }
        impl<const N: usize> sealed::Sealed for [$ty; N] {
            fn cell_shape(
                &self,
                source: Option<&DataFrame>,
            ) -> BindingResult<Option<(usize, usize)>> {
                self.as_slice().cell_shape(source)
            }
        }
        impl<const N: usize> ColumnsInput for [$ty; N] {
            fn local_identities(&self) -> bool {
                true
            }
            fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
                self.as_slice().bind_columns(source)
            }
        }
    };
}
direct_columns!(&Column, |column: &&Column| (*column).clone());
direct_columns!(&Series, |series: &&Series| (*series).clone().into_column());

fn columns_shape(columns: &[Column]) -> BindingResult<Option<(usize, usize)>> {
    let rows = columns[0].len();
    for column in columns {
        crate::error::length("cell grid", column.name().as_str(), rows, column.len())?;
    }
    Ok(Some((rows, columns.len())))
}
