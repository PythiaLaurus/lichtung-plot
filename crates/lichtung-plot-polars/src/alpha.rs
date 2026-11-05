//! Alpha bindings materialize native declarations exactly once at the binding call.
use crate::error::{binding, BindingResult};
use crate::input::sealed;
use crate::input::sealed::Sealed as _;
use crate::{AlphaEncoding, AlphaValues, ColumnInput};
use polars_core::prelude::{DataFrame, DataType};

impl sealed::Sealed for AlphaEncoding {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for AlphaValues {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for u8 {}

/// Exact non-null alpha bytes, or already-materialized native mapping values.
pub trait AlphaValuesInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_alpha_values(self, source: Option<&DataFrame>) -> BindingResult<AlphaValues>;
}
impl AlphaValuesInput for AlphaValues {
    fn bind_alpha_values(self, _: Option<&DataFrame>) -> BindingResult<AlphaValues> {
        Ok(self)
    }
}
impl AlphaValuesInput for u8 {
    fn bind_alpha_values(self, _: Option<&DataFrame>) -> BindingResult<AlphaValues> {
        Ok(AlphaValues::new("Alpha", [self])?)
    }
}
macro_rules! bytes_input {
    ($($ty:ty),+) => {$ (
        impl AlphaValuesInput for $ty {
            fn bind_alpha_values(self, source: Option<&DataFrame>) -> BindingResult<AlphaValues> {
                let column = self.bind_column(source, "alpha bytes")?;
                if column.dtype() != &DataType::UInt8 { return Err(crate::error::unsupported(&column, "exact u8 alpha")); }
                let bytes = crate::snapshot::collect(&column, "alpha bytes", column.as_materialized_series().u8().map_err(|source| crate::error::polars(&column, source))?.iter())?;
                let bytes = bytes.into_iter().enumerate().map(|(slot, byte)| byte.ok_or_else(|| binding("alpha bytes", format!("missing byte at slot {slot}")))).collect::<BindingResult<Vec<_>>>()?;
                let name = if column.name().is_empty() { "Alpha" } else { column.name().as_str() };
                Ok(AlphaValues::new(name, bytes)?)
            }
        }
    )+};
}
bytes_input!(
    &str,
    &String,
    &polars_core::prelude::Column,
    &polars_core::prelude::Series
);

/// Observation bytes or an already-native alpha encoding, with native unit validation retained.
pub trait AlphaInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_alpha(
        self,
        source: Option<&DataFrame>,
        expected: Option<usize>,
        copies: usize,
    ) -> BindingResult<AlphaEncoding>;
}
impl AlphaInput for AlphaEncoding {
    fn bind_alpha(
        self,
        _: Option<&DataFrame>,
        _: Option<usize>,
        _: usize,
    ) -> BindingResult<AlphaEncoding> {
        Ok(self)
    }
}
macro_rules! observations {
    ($($ty:ty),+) => {$ (
        impl AlphaInput for $ty {
            fn bind_alpha(self, source: Option<&DataFrame>, expected: Option<usize>, copies: usize) -> BindingResult<AlphaEncoding> {
                let copies = if self.native_slots() { 1 } else { copies };
                let values = expand(self.bind_alpha_values(source)?, expected, copies)?;
                Ok(AlphaEncoding::observations(values))
            }
        }
    )+};
}
observations!(
    &str,
    &String,
    &polars_core::prelude::Column,
    &polars_core::prelude::Series,
    AlphaValues
);

pub(crate) fn expand(
    values: AlphaValues,
    expected: Option<usize>,
    copies: usize,
) -> BindingResult<AlphaValues> {
    if let Some(expected) = expected {
        crate::error::length("alpha", values.name(), expected / copies, values.len())?;
    }
    if copies == 1 {
        Ok(values)
    } else {
        Ok(values.select((0..values.len()).flat_map(|row| std::iter::repeat_n(row, copies)))?)
    }
}

pub(crate) fn keys<I, S, B>(
    source: Option<&DataFrame>,
    keys: I,
) -> BindingResult<Vec<crate::CategoryValues>>
where
    I: IntoIterator<Item = (S, B)>,
    S: AsRef<str>,
    B: ColumnInput,
{
    keys.into_iter()
        .map(|(field, binding)| {
            let column = binding.bind_column(source, "alpha selector")?;
            Ok(crate::CategoryValues::nullable(
                field.as_ref(),
                crate::category::values(&column)?,
            )?)
        })
        .collect()
}
