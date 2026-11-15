//! Binding-time snapshots for the currently admitted native-data roles.

use lichtung_plot::{AxisTick, CategoryValues, NumericValues};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyTuple};

/// Retain the actual sequence entries before conversions can run Python code.
fn entries<'py>(value: &Bound<'py, PyAny>, role: &str) -> PyResult<Vec<Bound<'py, PyAny>>> {
    if let Ok(values) = value.cast::<PyList>() {
        Ok(values.iter().collect())
    } else if let Ok(values) = value.cast::<PyTuple>() {
        Ok(values.iter().collect())
    } else {
        Err(PyTypeError::new_err(format!(
            "{role} requires a Python list or tuple in this package's native-data interface"
        )))
    }
}

pub(crate) fn numeric(value: &Bound<'_, PyAny>, role: &str) -> PyResult<NumericValues> {
    let values = entries(value, role)?
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            if value.is_none() {
                return Ok(None);
            }
            if value.is_instance_of::<PyBool>()
                || !(value.is_instance_of::<PyInt>() || value.is_instance_of::<PyFloat>())
            {
                return Err(PyTypeError::new_err(format!(
                    "{role}[{index}] must be int, float, or None; bool is not numeric"
                )));
            }
            value.extract::<f64>().map(Some)
        })
        .collect::<PyResult<Vec<_>>>()?;
    Ok(NumericValues::nullable(role, values))
}

pub(crate) fn category(value: &Bound<'_, PyAny>, role: &str) -> PyResult<CategoryValues> {
    let values = entries(value, role)?
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            if value.is_none() {
                Ok(None)
            } else {
                value.extract::<String>().map(Some).map_err(|_| {
                    PyTypeError::new_err(format!("{role}[{index}] must be str or None"))
                })
            }
        })
        .collect::<PyResult<Vec<_>>>()?;
    CategoryValues::nullable(role, values).map_err(crate::native_error)
}

pub(crate) fn ticks(value: &Bound<'_, PyAny>) -> PyResult<Vec<AxisTick>> {
    entries(value, "ticks")?
        .into_iter()
        .map(|entry| {
            let pair = entry.cast::<PyTuple>()?;
            if pair.len() != 2 {
                return Err(PyTypeError::new_err(
                    "each exact tick must be a (position, label) tuple",
                ));
            }
            let position = pair.get_item(0)?;
            if position.is_instance_of::<PyBool>()
                || !(position.is_instance_of::<PyInt>() || position.is_instance_of::<PyFloat>())
            {
                return Err(PyTypeError::new_err(
                    "an exact tick position must be numeric",
                ));
            }
            Ok(AxisTick::new(
                position.extract::<f64>()?,
                pair.get_item(1)?.extract::<String>()?,
            ))
        })
        .collect()
}
