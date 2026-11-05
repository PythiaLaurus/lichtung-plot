use ndarray::{ArrayView, Dimension};

use crate::error::BindingResult as Result;
use crate::AdapterError as Error;

pub(crate) fn buffer<T>(length: usize, operation: &'static str) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|source| Error::Allocation { operation, source })?;
    Ok(values)
}

pub(crate) use buffer as allocate;

pub(crate) fn values<T, U, D: Dimension>(
    view: ArrayView<'_, T, D>,
    validity: Option<ArrayView<'_, bool, D>>,
    mut convert: impl FnMut(&T) -> Option<U>,
    operation: &'static str,
) -> Result<Vec<Option<U>>> {
    if let Some(validity) = &validity {
        if view.shape() != validity.shape() {
            return Err(Error::Shape {
                operation,
                expected: view.shape().to_vec(),
                actual: validity.shape().to_vec(),
            });
        }
    }
    let mut values = buffer(view.len(), operation)?;
    if let Some(validity) = validity {
        values.extend(view.iter().zip(validity.iter()).map(|(value, present)| {
            if *present {
                convert(value)
            } else {
                None
            }
        }));
    } else {
        values.extend(view.iter().map(convert));
    }
    Ok(values)
}
