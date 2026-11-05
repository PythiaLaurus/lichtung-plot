//! Sealed role protocols distinguish integer selectors, vectors, matrices and masks.
use std::collections::HashSet;
use std::ops::RangeFull;

use ndarray::{ArrayView, ArrayView1, ArrayView2, Dimension, Ix1, Ix2};

use crate::error::{binding, length, unsupported, BindingResult};
use crate::{
    ArrayElement, ArraySource, AxisValues, BooleanValues, CategoryValues, NumericValues, SourceRows,
};

pub(crate) mod sealed {
    pub trait Sealed {
        fn native_slots(&self) -> bool {
            false
        }
        fn cell_shape(&self) -> Option<(usize, usize)> {
            None
        }
        fn selected_columns(&self) -> Option<&[usize]> {
            None
        }
    }
}
impl sealed::Sealed for usize {}
impl sealed::Sealed for RangeFull {}
impl<T> sealed::Sealed for ArrayView1<'_, T> {}
impl<T> sealed::Sealed for ArrayView2<'_, T> {
    fn cell_shape(&self) -> Option<(usize, usize)> {
        Some(self.dim())
    }
}
impl<T> sealed::Sealed for (ArrayView1<'_, T>, ArrayView1<'_, bool>) {}
impl<T> sealed::Sealed for (ArrayView2<'_, T>, ArrayView2<'_, bool>) {
    fn cell_shape(&self) -> Option<(usize, usize)> {
        Some(self.0.dim())
    }
}
macro_rules! native {
    ($($ty:ty),+) => {$ (impl sealed::Sealed for $ty { fn native_slots(&self) -> bool { true } })+};
}
native!(
    NumericValues,
    CategoryValues,
    BooleanValues,
    SourceRows,
    AxisValues
);

type Raw<T> = (String, Vec<Option<T>>);

pub(crate) fn repeat<T: Clone>(
    values: Vec<Option<T>>,
    copies: usize,
    role: &'static str,
) -> BindingResult<Vec<Option<T>>> {
    let len = values
        .len()
        .checked_mul(copies)
        .ok_or_else(|| binding(role, "expanded slot count exceeds usize"))?;
    let mut expanded = crate::snapshot::allocate(len, role)?;
    expanded.extend(
        values
            .into_iter()
            .flat_map(|value| std::iter::repeat_n(value, copies)),
    );
    Ok(expanded)
}

macro_rules! role_input {
    ($trait:ident, $bind:ident, $raw:ident, $repeated:ident, $ty:ty, $native:ident, $bound:path, $supported:ident, $convert:expr, $column:ident, $construct:expr) => {
        #[doc = concat!("Sealed ", stringify!($trait), " role: logical column index, foreign vector, validity pair or native values.")]
        pub trait $trait: sealed::Sealed + Sized {
            #[doc(hidden)] fn $raw(self, source: Option<&dyn ArraySource>, role: &'static str) -> BindingResult<Raw<$ty>>;
            #[doc(hidden)] fn $bind(self, source: Option<&dyn ArraySource>, role: &'static str) -> BindingResult<$native> {
                let (name, values) = self.$raw(source, role)?;
                ($construct)(name, values)
            }
            #[doc(hidden)] fn $repeated(self, source: Option<&dyn ArraySource>, role: &'static str, copies: usize) -> BindingResult<$native> {
                let (name, values) = self.$raw(source, role)?;
                ($construct)(name, repeat(values, copies, role)?)
            }
        }
        impl $trait for usize {
            fn $raw(self, source: Option<&dyn ArraySource>, role: &'static str) -> BindingResult<Raw<$ty>> {
                Ok((crate::source::name(self), crate::source::lookup(source, self, role)?.$column(self, role)?))
            }
        }
        impl<T: ArrayElement + $bound> $trait for ArrayView1<'_, T> {
            fn $raw(self, _: Option<&dyn ArraySource>, role: &'static str) -> BindingResult<Raw<$ty>> {
                if !T::$supported { return Err(unsupported::<T>(role)); }
                Ok((role.into(), crate::snapshot::values(self, None, $convert, role)?))
            }
        }
        impl<T: ArrayElement + $bound> $trait for (ArrayView1<'_, T>, ArrayView1<'_, bool>) {
            fn $raw(self, _: Option<&dyn ArraySource>, role: &'static str) -> BindingResult<Raw<$ty>> {
                if !T::$supported { return Err(unsupported::<T>(role)); }
                Ok((role.into(), crate::snapshot::values(self.0, Some(self.1), $convert, role)?))
            }
        }
        impl $trait for $native {
            fn $raw(self, _: Option<&dyn ArraySource>, role: &'static str) -> BindingResult<Raw<$ty>> { Err(binding(role, "use a foreign variable in this mapping role")) }
            fn $bind(self, _: Option<&dyn ArraySource>, _: &'static str) -> BindingResult<$native> { Ok(self) }
            fn $repeated(self, _: Option<&dyn ArraySource>, _: &'static str, _: usize) -> BindingResult<$native> { Ok(self) }
        }
    };
}
role_input!(
    NumericInput,
    bind_numeric,
    raw_numeric,
    bind_numeric_repeated,
    f64,
    NumericValues,
    crate::NumericElement,
    NUMERIC,
    ArrayElement::numeric,
    numeric_column,
    |name, values| Ok(NumericValues::nullable(name, values))
);
role_input!(
    CategoryInput,
    bind_category,
    raw_category,
    bind_category_repeated,
    String,
    CategoryValues,
    crate::CategoryElement,
    CATEGORY,
    |value| value.category().map(str::to_owned),
    category_column,
    |name, values| Ok(CategoryValues::nullable(name, values)?)
);
role_input!(
    BooleanInput,
    bind_boolean,
    raw_boolean,
    bind_boolean_repeated,
    bool,
    BooleanValues,
    crate::BooleanElement,
    BOOLEAN,
    ArrayElement::boolean,
    boolean_column,
    |name, values| Ok(BooleanValues::nullable(name, values))
);

/// Exact source IDs: integer vectors, same-shape cell grids or native IDs.
pub trait SourceRowsInput: sealed::Sealed + Sized {
    #[doc(hidden)]
    fn raw_source_rows(
        self,
        _: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Vec<usize>> {
        Err(binding(
            role,
            "use an integer foreign variable for a generated-column selector",
        ))
    }
    #[doc(hidden)]
    fn bind_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SourceRows>;
    #[doc(hidden)]
    fn bind_source_rows_repeated(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
        copies: usize,
    ) -> BindingResult<SourceRows> {
        if copies != 1
            && !self.native_slots()
            && self.cell_shape().is_none()
            && self.selected_columns().is_none()
        {
            return Err(binding(
                role,
                "provide unique per-cell IDs, not repeated row IDs",
            ));
        }
        self.bind_source_rows(source, role)
    }
}
impl SourceRowsInput for SourceRows {
    fn bind_source_rows(
        self,
        _: Option<&dyn ArraySource>,
        _: &'static str,
    ) -> BindingResult<SourceRows> {
        Ok(self)
    }
}
impl SourceRowsInput for usize {
    fn raw_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Vec<usize>> {
        crate::source::lookup(source, self, role)?.id_column(self, role)
    }
    fn bind_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SourceRows> {
        Ok(SourceRows::new(
            crate::source::lookup(source, self, role)?.id_column(self, role)?,
        )?)
    }
}

pub(crate) fn ids<T: ArrayElement, D: Dimension>(
    values: ArrayView<'_, T, D>,
    present: Option<ArrayView<'_, bool, D>>,
    role: &'static str,
) -> BindingResult<Vec<usize>> {
    if !T::ID {
        return Err(unsupported::<T>(role));
    }
    let converted = crate::snapshot::values(values.view(), present, ArrayElement::id, role)?;
    let mut result = crate::snapshot::allocate(values.len(), role)?;
    for (slot, (value, id)) in values.iter().zip(converted).enumerate() {
        result.push(id.ok_or_else(|| crate::AdapterError::SourceRow {
            slot,
            value: format!("{value:?}"),
            column: None,
        })?);
    }
    Ok(result)
}
pub(crate) fn bytes<T: ArrayElement, D: Dimension>(
    values: ArrayView<'_, T, D>,
    present: Option<ArrayView<'_, bool, D>>,
    role: &'static str,
) -> BindingResult<Vec<u8>> {
    if !T::BYTE {
        return Err(unsupported::<T>(role));
    }
    crate::snapshot::values(values, present, ArrayElement::byte, role)?
        .into_iter()
        .enumerate()
        .map(|(slot, value)| {
            value.ok_or_else(|| binding(role, format!("missing byte at logical slot {slot}")))
        })
        .collect()
}
macro_rules! source_ids {
    ($dim:ty) => {
        impl<T: ArrayElement> SourceRowsInput for ArrayView<'_, T, $dim> {
            fn raw_source_rows(
                self,
                _: Option<&dyn ArraySource>,
                role: &'static str,
            ) -> BindingResult<Vec<usize>> {
                ids(self, None, role)
            }
            fn bind_source_rows(
                self,
                _: Option<&dyn ArraySource>,
                role: &'static str,
            ) -> BindingResult<SourceRows> {
                Ok(SourceRows::new(ids(self, None, role)?)?)
            }
        }
        impl<T: ArrayElement> SourceRowsInput
            for (ArrayView<'_, T, $dim>, ArrayView<'_, bool, $dim>)
        {
            fn raw_source_rows(
                self,
                _: Option<&dyn ArraySource>,
                role: &'static str,
            ) -> BindingResult<Vec<usize>> {
                ids(self.0, Some(self.1), role)
            }
            fn bind_source_rows(
                self,
                _: Option<&dyn ArraySource>,
                role: &'static str,
            ) -> BindingResult<SourceRows> {
                Ok(SourceRows::new(ids(self.0, Some(self.1), role)?)?)
            }
        }
    };
}
source_ids!(Ix1);
source_ids!(Ix2);
impl<T: ArrayElement> BooleanInput for ArrayView2<'_, T> {
    fn raw_boolean(
        self,
        _: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Raw<bool>> {
        if !T::BOOLEAN {
            return Err(unsupported::<T>(role));
        }
        Ok((
            role.into(),
            crate::snapshot::values(self, None, ArrayElement::boolean, role)?,
        ))
    }
}
impl<T: ArrayElement> BooleanInput for (ArrayView2<'_, T>, ArrayView2<'_, bool>) {
    fn raw_boolean(
        self,
        _: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Raw<bool>> {
        if !T::BOOLEAN {
            return Err(unsupported::<T>(role));
        }
        Ok((
            role.into(),
            crate::snapshot::values(self.0, Some(self.1), ArrayElement::boolean, role)?,
        ))
    }
}

/// Numeric or categorical trajectory x; Boolean input is never a numeric coordinate.
pub trait AxisInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_axis(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<AxisValues>;
}
impl AxisInput for usize {
    fn bind_axis(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<AxisValues> {
        crate::source::lookup(source, self, role)?.axis_column(self, role)
    }
}
fn axis<T: ArrayElement>(
    values: ArrayView1<'_, T>,
    present: Option<ArrayView1<'_, bool>>,
    role: &'static str,
) -> BindingResult<AxisValues> {
    if T::NUMERIC {
        Ok(AxisValues::Numeric(NumericValues::nullable(
            role,
            crate::snapshot::values(values, present, ArrayElement::numeric, role)?,
        )))
    } else if T::CATEGORY {
        Ok(AxisValues::Category(CategoryValues::nullable(
            role,
            crate::snapshot::values(
                values,
                present,
                |value| value.category().map(str::to_owned),
                role,
            )?,
        )?))
    } else {
        Err(unsupported::<T>(role))
    }
}
impl<T: ArrayElement> AxisInput for ArrayView1<'_, T> {
    fn bind_axis(
        self,
        _: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<AxisValues> {
        axis(self, None, role)
    }
}
impl<T: ArrayElement> AxisInput for (ArrayView1<'_, T>, ArrayView1<'_, bool>) {
    fn bind_axis(
        self,
        _: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<AxisValues> {
        axis(self.0, Some(self.1), role)
    }
}
impl AxisInput for AxisValues {
    fn bind_axis(self, _: Option<&dyn ArraySource>, _: &'static str) -> BindingResult<AxisValues> {
        Ok(self)
    }
}
impl AxisInput for NumericValues {
    fn bind_axis(self, _: Option<&dyn ArraySource>, _: &'static str) -> BindingResult<AxisValues> {
        Ok(AxisValues::Numeric(self))
    }
}
impl AxisInput for CategoryValues {
    fn bind_axis(self, _: Option<&dyn ArraySource>, _: &'static str) -> BindingResult<AxisValues> {
        Ok(AxisValues::Category(self))
    }
}

pub(crate) type Columns = (usize, Vec<(usize, Vec<Option<f64>>)>);
pub(crate) type SeriesBinding = (NumericValues, Option<(usize, Vec<usize>, bool)>);

/// Ordered logical source columns, `..`, or a direct matrix/validity pair.
pub trait ColumnsInput: sealed::Sealed {
    #[doc(hidden)]
    fn local_identities(&self) -> bool {
        false
    }
    #[doc(hidden)]
    fn bind_columns(self, source: Option<&dyn ArraySource>) -> BindingResult<Columns>;
}
pub(crate) fn selected(
    source: Option<&dyn ArraySource>,
    indices: &[usize],
) -> BindingResult<Columns> {
    if indices.is_empty() {
        return Err(binding("selected columns", "select at least one variable"));
    }
    let mut seen = HashSet::new();
    let mut columns = crate::snapshot::allocate(indices.len(), "selected columns")?;
    seen.try_reserve(indices.len())
        .map_err(|source| crate::AdapterError::Allocation {
            operation: "selection identities",
            source,
        })?;
    let mut rows = 0;
    for &index in indices {
        if !seen.insert(index) {
            return Err(binding(
                "selected columns",
                format!("duplicate column {index}"),
            ));
        }
        let source = crate::source::lookup(source, index, "selected columns")?;
        rows = source.shape().0;
        columns.push((index, source.numeric_column(index, "selected columns")?));
    }
    Ok((rows, columns))
}
impl sealed::Sealed for &[usize] {
    fn selected_columns(&self) -> Option<&[usize]> {
        Some(self)
    }
}
impl<const N: usize> sealed::Sealed for [usize; N] {
    fn selected_columns(&self) -> Option<&[usize]> {
        Some(self)
    }
}
impl ColumnsInput for &[usize] {
    fn bind_columns(self, source: Option<&dyn ArraySource>) -> BindingResult<Columns> {
        selected(source, self)
    }
}
impl<const N: usize> ColumnsInput for [usize; N] {
    fn bind_columns(self, source: Option<&dyn ArraySource>) -> BindingResult<Columns> {
        selected(source, &self)
    }
}
impl ColumnsInput for RangeFull {
    fn bind_columns(self, source: Option<&dyn ArraySource>) -> BindingResult<Columns> {
        let source = source
            .ok_or_else(|| binding("selected columns", "all columns require an array source"))?;
        let mut indices = crate::snapshot::allocate(source.shape().1, "selected columns")?;
        indices.extend(0..source.shape().1);
        selected(Some(source), &indices)
    }
}
impl<T: ArrayElement> ColumnsInput for ArrayView2<'_, T> {
    fn local_identities(&self) -> bool {
        true
    }
    fn bind_columns(self, _: Option<&dyn ArraySource>) -> BindingResult<Columns> {
        (..).bind_columns(Some(&self))
    }
}
impl<T: ArrayElement> ColumnsInput for (ArrayView2<'_, T>, ArrayView2<'_, bool>) {
    fn local_identities(&self) -> bool {
        true
    }
    fn bind_columns(self, _: Option<&dyn ArraySource>) -> BindingResult<Columns> {
        (..).bind_columns(Some(&self))
    }
}

pub(crate) fn multiple(
    (rows, columns): Columns,
    local: bool,
    role: &'static str,
) -> BindingResult<SeriesBinding> {
    let cells = rows
        .checked_mul(columns.len())
        .ok_or_else(|| binding(role, "expanded slot count exceeds usize"))?;
    let mut values = crate::snapshot::allocate(cells, role)?;
    for row in 0..rows {
        for (_, column) in &columns {
            length(role, role, rows, column.len())?;
            values.push(column[row]);
        }
    }
    Ok((
        NumericValues::nullable(role, values),
        Some((
            rows,
            columns.into_iter().map(|(index, _)| index).collect(),
            local,
        )),
    ))
}
/// One numeric variable or explicit logical-column/matrix expansion.
pub trait NumericSeriesInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_series(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding>;
}
impl NumericSeriesInput for usize {
    fn bind_series(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding> {
        Ok((self.bind_numeric(source, role)?, None))
    }
}
impl NumericSeriesInput for NumericValues {
    fn bind_series(
        self,
        _: Option<&dyn ArraySource>,
        _: &'static str,
    ) -> BindingResult<SeriesBinding> {
        Ok((self, None))
    }
}
macro_rules! one_series {
    ($ty:ty) => {
        impl<T: ArrayElement + crate::NumericElement> NumericSeriesInput for $ty {
            fn bind_series(
                self,
                source: Option<&dyn ArraySource>,
                role: &'static str,
            ) -> BindingResult<SeriesBinding> {
                Ok((self.bind_numeric(source, role)?, None))
            }
        }
    };
}
one_series!(ArrayView1<'_, T>);
one_series!((ArrayView1<'_, T>, ArrayView1<'_, bool>));
macro_rules! matrix_series {
    ($ty:ty) => {
        impl<T: ArrayElement> NumericSeriesInput for $ty {
            fn bind_series(
                self,
                source: Option<&dyn ArraySource>,
                role: &'static str,
            ) -> BindingResult<SeriesBinding> {
                multiple(self.bind_columns(source)?, true, role)
            }
        }
    };
}
matrix_series!(ArrayView2<'_, T>);
matrix_series!((ArrayView2<'_, T>, ArrayView2<'_, bool>));
impl NumericSeriesInput for &[usize] {
    fn bind_series(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding> {
        multiple(self.bind_columns(source)?, false, role)
    }
}
impl<const N: usize> NumericSeriesInput for [usize; N] {
    fn bind_series(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SeriesBinding> {
        multiple(self.bind_columns(source)?, false, role)
    }
}

fn cell_columns<T: Clone>(
    source: Option<&dyn ArraySource>,
    indices: &[usize],
    role: &'static str,
    read: impl Fn(&dyn ArraySource, usize) -> BindingResult<Vec<T>>,
) -> BindingResult<Vec<T>> {
    let first = *indices
        .first()
        .ok_or_else(|| binding(role, "select at least one column"))?;
    let source = crate::source::lookup(source, first, role)?;
    let rows = source.shape().0;
    let cells = rows
        .checked_mul(indices.len())
        .ok_or_else(|| binding(role, "cell count exceeds usize"))?;
    let mut columns = crate::snapshot::allocate(indices.len(), role)?;
    let mut seen = HashSet::new();
    seen.try_reserve(indices.len())
        .map_err(|source| crate::AdapterError::Allocation {
            operation: role,
            source,
        })?;
    for &index in indices {
        if !seen.insert(index) {
            return Err(binding(role, format!("duplicate column {index}")));
        }
        crate::source::lookup(Some(source), index, role)?;
        columns.push(read(source, index)?);
    }
    let mut values = crate::snapshot::allocate(cells, role)?;
    for row in 0..rows {
        for column in &columns {
            values.push(column[row].clone());
        }
    }
    Ok(values)
}
impl BooleanInput for &[usize] {
    fn raw_boolean(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Raw<bool>> {
        Ok((
            role.into(),
            cell_columns(source, self, role, |source, index| {
                source.boolean_column(index, role)
            })?,
        ))
    }
    fn bind_boolean_repeated(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
        _: usize,
    ) -> BindingResult<BooleanValues> {
        self.bind_boolean(source, role)
    }
}
impl<const N: usize> BooleanInput for [usize; N] {
    fn raw_boolean(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Raw<bool>> {
        self.as_slice().raw_boolean(source, role)
    }
    fn bind_boolean_repeated(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
        _: usize,
    ) -> BindingResult<BooleanValues> {
        self.bind_boolean(source, role)
    }
}
impl SourceRowsInput for &[usize] {
    fn raw_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Vec<usize>> {
        cell_columns(source, self, role, |source, index| {
            source.id_column(index, role)
        })
    }
    fn bind_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SourceRows> {
        Ok(SourceRows::new(self.raw_source_rows(source, role)?)?)
    }
}
impl<const N: usize> SourceRowsInput for [usize; N] {
    fn raw_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<Vec<usize>> {
        self.as_slice().raw_source_rows(source, role)
    }
    fn bind_source_rows(
        self,
        source: Option<&dyn ArraySource>,
        role: &'static str,
    ) -> BindingResult<SourceRows> {
        self.as_slice().bind_source_rows(source, role)
    }
}
