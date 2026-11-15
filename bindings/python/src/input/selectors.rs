//! Immutable descriptions only. Resolution, hashing and source access happen at bind time.

use std::collections::HashSet;

use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PySlice, PyTuple};
use pyo3::{PyTraverseError, PyVisit};

pub(super) enum Selection {
    One(Py<PyAny>),
    Keys(Vec<Py<PyAny>>),
    Slice(Py<PyAny>),
}

impl Selection {
    fn clone_ref(&self, py: Python<'_>) -> Self {
        match self {
            Self::One(key) => Self::One(key.clone_ref(py)),
            Self::Keys(keys) => Self::Keys(keys.iter().map(|key| key.clone_ref(py)).collect()),
            Self::Slice(slice) => Self::Slice(slice.clone_ref(py)),
        }
    }

    fn traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        match self {
            Self::One(key) | Self::Slice(key) => visit.call(key),
            Self::Keys(keys) => {
                for key in keys {
                    visit.call(key)?;
                }
                Ok(())
            }
        }
    }
}

#[pyclass(name = "ColumnSelector", module = "lichtung_plot._native")]
struct ColumnSelector {
    selection: Option<Selection>,
}

#[pyclass(name = "ColumnsSelector", module = "lichtung_plot._native")]
struct ColumnsSelector {
    selection: Option<Selection>,
}

macro_rules! selector_gc {
    ($class:ident) => {
        #[pymethods]
        impl $class {
            fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
                if let Some(selection) = &self.selection {
                    selection.traverse(&visit)?;
                }
                Ok(())
            }

            fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
                let retired = { slf.try_borrow_mut()?.selection.take() };
                drop(retired);
                Ok(())
            }
        }
    };
}
selector_gc!(ColumnSelector);
selector_gc!(ColumnsSelector);

#[pyclass(name = "_ColumnEntry", frozen, module = "lichtung_plot._native")]
struct SingleEntry;

#[pymethods]
impl SingleEntry {
    fn __getitem__(slf: &Bound<'_, Self>, key: &Bound<'_, PyAny>) -> PyResult<Py<ColumnSelector>> {
        Py::new(
            slf.py(),
            ColumnSelector {
                selection: Some(Selection::One(key.clone().unbind())),
            },
        )
    }
}

#[pyclass(name = "_ColumnsEntry", frozen, module = "lichtung_plot._native")]
struct CollectionEntry;

#[pymethods]
impl CollectionEntry {
    fn __getitem__(slf: &Bound<'_, Self>, key: &Bound<'_, PyAny>) -> PyResult<Py<ColumnsSelector>> {
        let selection = if key.is_instance_of::<PySlice>() {
            Selection::Slice(key.clone().unbind())
        } else if let Ok(keys) = key.cast::<PyTuple>() {
            Selection::Keys(keys.iter().map(Bound::unbind).collect())
        } else {
            Selection::Keys(vec![key.clone().unbind()])
        };
        Py::new(
            slf.py(),
            ColumnsSelector {
                selection: Some(selection),
            },
        )
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ColumnSelector>()?;
    module.add_class::<ColumnsSelector>()?;
    module.add_class::<SingleEntry>()?;
    module.add_class::<CollectionEntry>()?;
    module.add("col", Py::new(module.py(), SingleEntry)?)?;
    module.add("cols", Py::new(module.py(), CollectionEntry)?)
}

pub(super) fn description(value: &Bound<'_, PyAny>) -> PyResult<Option<Selection>> {
    // Only INCREF while a selector is borrowed. No key methods or formatting here.
    let snapshot = if let Ok(selector) = value.cast::<ColumnSelector>() {
        Some(
            selector
                .try_borrow()?
                .selection
                .as_ref()
                .map(|selection| selection.clone_ref(value.py())),
        )
    } else if let Ok(selector) = value.cast::<ColumnsSelector>() {
        Some(
            selector
                .try_borrow()?
                .selection
                .as_ref()
                .map(|selection| selection.clone_ref(value.py())),
        )
    } else {
        None
    };
    snapshot
        .map(|selection| {
            selection.ok_or_else(|| PyValueError::new_err("this selector has been cleared"))
        })
        .transpose()
}

fn integer<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if value.is_instance_of::<PyBool>() || super::instance(value, "numpy", "bool_")? {
        return Err(PyTypeError::new_err(
            "Boolean positions are not column indices",
        ));
    }
    PyModule::import(value.py(), "operator")?
        .getattr("index")?
        .call1((value,))
}

pub(super) fn position(value: &Bound<'_, PyAny>, columns: usize) -> PyResult<usize> {
    let value = integer(value)?;
    let n = i128::try_from(columns).map_err(|_| PyIndexError::new_err("too many columns"))?;
    let mut index = value
        .extract::<i128>()
        .map_err(|_| PyIndexError::new_err("column index is out of bounds"))?;
    if index < 0 {
        index += n;
    }
    if !(0..n).contains(&index) {
        return Err(PyIndexError::new_err("column index is out of bounds"));
    }
    usize::try_from(index).map_err(|_| PyIndexError::new_err("column index is out of bounds"))
}

fn endpoint(value: &Bound<'_, PyAny>, n: i128, reverse: bool, default: i128) -> PyResult<i128> {
    if value.is_none() {
        return Ok(default);
    }
    let value = integer(value)?;
    let mut index = value
        .extract::<i128>()
        .map_err(|_| PyIndexError::new_err("slice endpoint is out of bounds"))?;
    if index < 0 {
        index += n;
    }
    let valid = if reverse {
        (-1..n).contains(&index)
    } else {
        (0..=n).contains(&index)
    };
    if !valid {
        return Err(PyIndexError::new_err(
            "slice endpoint is out of bounds; selectors do not clip",
        ));
    }
    Ok(index)
}

pub(super) fn slice(value: &Bound<'_, PyAny>, columns: usize) -> PyResult<Vec<usize>> {
    let n = i128::try_from(columns).map_err(|_| PyIndexError::new_err("too many columns"))?;
    let step = value.getattr("step")?;
    let step = if step.is_none() {
        1
    } else {
        let step = integer(&step)?;
        // Comparing exact PyInts handles arbitrary magnitudes. The equivalent
        // bounded step cannot skip a second reachable position in this table.
        if step.eq(0)? {
            return Err(PyValueError::new_err("a column slice step cannot be zero"));
        }
        if step.gt(n + 1)? {
            n + 1
        } else if step.lt(-n - 1)? {
            -n - 1
        } else {
            step.extract::<i128>()?
        }
    };
    let reverse = step < 0;
    let start = endpoint(
        &value.getattr("start")?,
        n,
        reverse,
        if reverse { n - 1 } else { 0 },
    )?;
    let boundary = endpoint(
        &value.getattr("stop")?,
        n,
        reverse,
        if reverse { -1 } else { n },
    )?;
    let mut result = Vec::new();
    let mut index = start;
    while if reverse {
        index > boundary
    } else {
        index < boundary
    } {
        result.push(
            usize::try_from(index)
                .map_err(|_| PyIndexError::new_err("slice position is outside the source"))?,
        );
        index += step;
    }
    nonempty_unique(&result)?;
    Ok(result)
}

pub(super) fn nonempty_unique(indices: &[usize]) -> PyResult<()> {
    if indices.is_empty() {
        return Err(PyValueError::new_err("select at least one column"));
    }
    let mut seen = HashSet::new();
    if indices.iter().any(|index| !seen.insert(*index)) {
        return Err(PyValueError::new_err("duplicate resolved column selection"));
    }
    Ok(())
}
