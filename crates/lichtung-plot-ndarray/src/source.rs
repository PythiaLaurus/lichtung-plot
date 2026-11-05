//! Safe, borrowed descriptors; selecting a column snapshots only that column.
use ndarray::{ArrayView2, Axis};

use crate::error::{unsupported_column, BindingResult};
use crate::input::sealed;
use crate::{ArrayElement, AxisValues, CategoryValues, NumericValues};

/// Sealed array-table inputs, including `(values, present)` two-dimensional views.
/// Rows are observations and selectors always address logical columns.
pub trait ArraySource: sealed::Sealed {
    #[doc(hidden)]
    fn validate(&self) -> BindingResult<()>;
    #[doc(hidden)]
    fn shape(&self) -> (usize, usize);
    #[doc(hidden)]
    fn numeric_column(&self, index: usize, role: &'static str) -> BindingResult<Vec<Option<f64>>>;
    #[doc(hidden)]
    fn category_column(
        &self,
        index: usize,
        role: &'static str,
    ) -> BindingResult<Vec<Option<String>>>;
    #[doc(hidden)]
    fn boolean_column(&self, index: usize, role: &'static str) -> BindingResult<Vec<Option<bool>>>;
    #[doc(hidden)]
    fn id_column(&self, index: usize, role: &'static str) -> BindingResult<Vec<usize>>;
    #[doc(hidden)]
    fn byte_column(&self, index: usize, role: &'static str) -> BindingResult<Vec<u8>>;
    #[doc(hidden)]
    fn axis_column(&self, index: usize, role: &'static str) -> BindingResult<AxisValues>;
}

pub(crate) fn name(index: usize) -> String {
    index.checked_add(1).map_or_else(
        || format!("Column index {index}"),
        |position| format!("Column {position}"),
    )
}
pub(crate) fn lookup<'a>(
    source: Option<&'a dyn ArraySource>,
    index: usize,
    role: &'static str,
) -> BindingResult<&'a dyn ArraySource> {
    let source = source.ok_or(crate::AdapterError::MissingSource { role, index })?;
    let (_, columns) = source.shape();
    if index >= columns {
        return Err(crate::AdapterError::Column {
            role,
            index,
            columns,
        });
    }
    Ok(source)
}

fn check<T>(
    values: ArrayView2<'_, T>,
    present: Option<ArrayView2<'_, bool>>,
    index: usize,
    role: &'static str,
) -> BindingResult<()> {
    if let Some(present) = present {
        if values.shape() != present.shape() {
            return Err(crate::AdapterError::Shape {
                operation: role,
                expected: values.shape().to_vec(),
                actual: present.shape().to_vec(),
            });
        }
    }
    if index >= values.ncols() {
        return Err(crate::AdapterError::Column {
            role,
            index,
            columns: values.ncols(),
        });
    }
    Ok(())
}

macro_rules! source_method {
    ($method:ident, $ty:ty, $supported:ident, $convert:expr) => {
        fn $method(&self, index: usize, role: &'static str) -> BindingResult<Vec<Option<$ty>>> {
            let (values, present) = self.parts();
            check(values, present, index, role)?;
            if !T::$supported {
                return Err(unsupported_column::<T>(role, index));
            }
            crate::snapshot::values(
                values.index_axis(Axis(1), index),
                present.as_ref().map(|mask| mask.index_axis(Axis(1), index)),
                $convert,
                role,
            )
        }
    };
}

trait Parts<T> {
    fn parts(&self) -> (ArrayView2<'_, T>, Option<ArrayView2<'_, bool>>);
}
impl<T> Parts<T> for ArrayView2<'_, T> {
    fn parts(&self) -> (ArrayView2<'_, T>, Option<ArrayView2<'_, bool>>) {
        (self.view(), None)
    }
}
impl<T> Parts<T> for (ArrayView2<'_, T>, ArrayView2<'_, bool>) {
    fn parts(&self) -> (ArrayView2<'_, T>, Option<ArrayView2<'_, bool>>) {
        (self.0.view(), Some(self.1.view()))
    }
}

macro_rules! array_source {
    ($ty:ty) => {
        impl<T: ArrayElement> ArraySource for $ty {
            fn validate(&self) -> BindingResult<()> {
                let (values, present) = self.parts();
                if let Some(present) = present {
                    if values.shape() != present.shape() {
                        return Err(crate::AdapterError::Shape {
                            operation: "source validity",
                            expected: values.shape().to_vec(),
                            actual: present.shape().to_vec(),
                        });
                    }
                }
                Ok(())
            }
            fn shape(&self) -> (usize, usize) {
                let (values, _) = self.parts();
                values.dim()
            }
            source_method!(numeric_column, f64, NUMERIC, ArrayElement::numeric);
            source_method!(category_column, String, CATEGORY, |value| value
                .category()
                .map(str::to_owned));
            source_method!(boolean_column, bool, BOOLEAN, ArrayElement::boolean);
            fn id_column(&self, index: usize, role: &'static str) -> BindingResult<Vec<usize>> {
                let (values, present) = self.parts();
                check(values, present, index, role)?;
                if !T::ID {
                    return Err(unsupported_column::<T>(role, index));
                }
                let values = values.index_axis(Axis(1), index);
                let present = present.as_ref().map(|mask| mask.index_axis(Axis(1), index));
                crate::input::ids(values, present, role).map_err(|error| match error {
                    crate::AdapterError::SourceRow { slot, value, .. } => {
                        crate::AdapterError::SourceRow {
                            slot,
                            value,
                            column: Some(index),
                        }
                    }
                    other => other,
                })
            }
            fn byte_column(&self, index: usize, role: &'static str) -> BindingResult<Vec<u8>> {
                let (values, present) = self.parts();
                check(values, present, index, role)?;
                if !T::BYTE {
                    return Err(unsupported_column::<T>(role, index));
                }
                crate::input::bytes(
                    values.index_axis(Axis(1), index),
                    present.as_ref().map(|mask| mask.index_axis(Axis(1), index)),
                    role,
                )
            }
            fn axis_column(&self, index: usize, role: &'static str) -> BindingResult<AxisValues> {
                if T::NUMERIC {
                    Ok(AxisValues::Numeric(NumericValues::nullable(
                        name(index),
                        self.numeric_column(index, role)?,
                    )))
                } else if T::CATEGORY {
                    Ok(AxisValues::Category(CategoryValues::nullable(
                        name(index),
                        self.category_column(index, role)?,
                    )?))
                } else {
                    Err(unsupported_column::<T>(role, index))
                }
            }
        }
    };
}
array_source!(ArrayView2<'_, T>);
array_source!((ArrayView2<'_, T>, ArrayView2<'_, bool>));
