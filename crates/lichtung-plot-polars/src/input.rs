//! Sealed role bindings; caller inputs are snapshotted at the owning operation.

use std::collections::HashSet;
use std::ops::RangeFull;

use polars_core::prelude::{Column, DataFrame, DataType, IntoColumn, Series};

use crate::error::{binding, BindingResult};
use crate::{AdapterError, AxisValues, BooleanValues, CategoryValues, NumericValues, SourceRows};

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for &str {}
impl sealed::Sealed for &String {}
impl sealed::Sealed for &Column {}
impl sealed::Sealed for &Series {}
impl sealed::Sealed for NumericValues {}
impl sealed::Sealed for CategoryValues {}
impl sealed::Sealed for BooleanValues {}
impl sealed::Sealed for SourceRows {}
impl sealed::Sealed for AxisValues {}
impl sealed::Sealed for RangeFull {}
impl sealed::Sealed for &DataFrame {}

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

macro_rules! role_input {
    ($trait:ident, $method:ident, $value:ty, $convert:path, $doc:literal) => {
        #[doc = $doc]
        /// This protocol is sealed; callers pass supported values without importing it.
        pub trait $trait: sealed::Sealed {
            #[doc(hidden)]
            fn $method(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<$value>;
        }
        impl $trait for &Column {
            fn $method(self, _: Option<&DataFrame>, role: &'static str) -> BindingResult<$value> {
                $convert(self).map_err(|error| context(error, role))
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
        }
        impl $trait for &str {
            fn $method(
                self,
                source: Option<&DataFrame>,
                role: &'static str,
            ) -> BindingResult<$value> {
                lookup(source, self, role)?.$method(source, role)
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
        }
        impl $trait for $value {
            fn $method(self, _: Option<&DataFrame>, _: &'static str) -> BindingResult<$value> {
                Ok(self)
            }
        }
    };
}

role_input!(
    NumericInput,
    bind_numeric,
    NumericValues,
    crate::numeric::numeric,
    "A numeric field, borrowed Polars variable or owned native numeric value."
);
role_input!(
    CategoryInput,
    bind_category,
    CategoryValues,
    crate::category::categories,
    "A categorical field, borrowed Polars variable or owned native category value."
);
role_input!(
    BooleanInput,
    bind_boolean,
    BooleanValues,
    crate::boolean::booleans,
    "A Boolean field, borrowed Polars variable or owned native Boolean value."
);
role_input!(
    SourceRowsInput,
    bind_source_rows,
    SourceRows,
    crate::provenance::source_rows,
    "Exact integer source IDs from a field, borrowed Polars variable or native IDs."
);

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

impl sealed::Sealed for &[&str] {}
impl ColumnsInput for &[&str] {
    fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
        selected(source, self)
    }
}
impl<const N: usize> sealed::Sealed for [&str; N] {}
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
        impl sealed::Sealed for &[$ty] {}
        impl ColumnsInput for &[$ty] {
            fn bind_columns(self, _: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
                if self.is_empty() {
                    return Err(binding("selected columns", "select at least one variable"));
                }
                let mut columns = crate::snapshot::allocate(self.len(), "direct columns")?;
                columns.extend(self.iter().map($convert));
                Ok(columns)
            }
        }
        impl<const N: usize> sealed::Sealed for [$ty; N] {}
        impl<const N: usize> ColumnsInput for [$ty; N] {
            fn bind_columns(self, source: Option<&DataFrame>) -> BindingResult<Vec<Column>> {
                self.as_slice().bind_columns(source)
            }
        }
    };
}
direct_columns!(&Column, |column: &&Column| (*column).clone());
direct_columns!(&Series, |series: &&Series| (*series).clone().into_column());
