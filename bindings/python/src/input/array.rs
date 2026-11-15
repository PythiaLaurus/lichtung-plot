//! Call-local typed scratch. NumPy loans end before any further Python call.

use lichtung_plot_ndarray as nd;
use ndarray::{Array2, Axis};
use numpy::{
    Element, PyArrayDescr, PyArrayDescrMethods, PyArrayDyn, PyArrayMethods, PyUntypedArray,
    PyUntypedArrayMethods,
};
use polars_core::prelude::{Column, DataType, IntoColumn, NamedFrom, Series};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    Numeric,
    Category,
    Ids,
}

macro_rules! scratch {
    ($($variant:ident: $ty:ty),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub(super) enum Dtype { $($variant),+ }

        pub(super) enum Scratch { $($variant(Array2<Option<$ty>>)),+ }

        impl Scratch {
            pub(super) fn view(&self) -> Box<dyn nd::ArraySource + '_> {
                match self { $(Self::$variant(values) => Box::new(values.view())),+ }
            }

            pub(super) fn shape(&self) -> (usize, usize) {
                match self { $(Self::$variant(values) => values.dim()),+ }
            }

            pub(super) fn columns(&self, names: &[String]) -> Vec<Column> {
                match self { $(Self::$variant(values) => names.iter().enumerate().map(|(j, name)| {
                    let column: Vec<Option<$ty>> = values.column(j).iter().cloned().collect();
                    Series::new(name.as_str().into(), column.as_slice()).into_column()
                }).collect()),+ }
            }
        }
    };
}
scratch!(
    I8: i8, I16: i16, I32: i32, I64: i64, I128: i128,
    U8: u8, U16: u16, U32: u32, U64: u64, U128: u128,
    F32: f32, F64: f64, Bool: bool, String: String,
);

impl Dtype {
    pub(super) fn numeric(self) -> bool {
        !matches!(self, Self::Bool | Self::String)
    }
}

fn shaped<T>(shape: (usize, usize), values: Vec<T>) -> PyResult<Array2<T>> {
    Array2::from_shape_vec(shape, values)
        .map_err(|error| crate::DataError::new_err(error.to_string()))
}

impl Scratch {
    /// Normalize numeric payloads only through the existing ndarray extractor.
    pub(super) fn numeric(&self) -> PyResult<Array2<Option<f64>>> {
        let source = self.view();
        let (rows, width) = self.shape();
        let columns = (0..width)
            .map(|j| source.numeric_column(j, "numeric"))
            .collect::<Result<Vec<_>, _>>()
            .map_err(super::nd_error)?;
        let mut values = Vec::new();
        for row in 0..rows {
            for column in &columns {
                values.push(column[row]);
            }
        }
        shaped((rows, width), values)
    }

    pub(super) fn category(&self) -> PyResult<Vec<Option<String>>> {
        self.view()
            .category_column(0, "hue")
            .map_err(super::nd_error)
    }

    pub(super) fn native(value: &Bound<'_, PyAny>, role: Role) -> PyResult<(Self, usize)> {
        let entries = super::entries(value, "input")?;
        let matrix = role != Role::Category && entries.first().is_some_and(super::native_sequence);
        let shape = if matrix {
            (
                entries.len(),
                super::entries(&entries[0], "matrix row")?.len(),
            )
        } else {
            (entries.len(), 1)
        };
        let mut cells = Vec::new();
        if matrix {
            for row in entries {
                let row = super::entries(&row, "matrix row")?;
                if row.len() != shape.1 {
                    return Err(crate::DataError::new_err(
                        "direct matrix rows must have equal widths",
                    ));
                }
                cells.extend(row);
            }
        } else {
            cells = entries;
        }
        let missing = cells.iter().map(Bound::is_none).collect::<Vec<_>>();
        let data = boxed(
            &cells,
            &missing,
            shape,
            match role {
                Role::Numeric => Dtype::F64,
                Role::Category => Dtype::String,
                Role::Ids => {
                    if cells
                        .iter()
                        .filter(|cell| !cell.is_none())
                        .all(|cell| cell.extract::<i128>().is_ok())
                    {
                        Dtype::I128
                    } else {
                        Dtype::U128
                    }
                }
            },
        )?;
        Ok((data, if matrix { 2 } else { 1 }))
    }
}

/// Public dtype metadata is inspected before a foreign array loan.
pub(super) fn dtype(dtype: &Bound<'_, PyAny>) -> PyResult<Option<Dtype>> {
    let np = PyModule::import(dtype.py(), "numpy")?;
    for field in ["fields", "subdtype", "metadata"] {
        if !dtype.getattr(field)?.is_none() {
            return Ok(None);
        }
    }
    let scalar = dtype.getattr("type")?;
    for (name, kind) in [
        ("int8", Dtype::I8),
        ("int16", Dtype::I16),
        ("int32", Dtype::I32),
        ("int64", Dtype::I64),
        ("longlong", Dtype::I64),
        ("uint8", Dtype::U8),
        ("uint16", Dtype::U16),
        ("uint32", Dtype::U32),
        ("uint64", Dtype::U64),
        ("ulonglong", Dtype::U64),
        ("float32", Dtype::F32),
        ("float64", Dtype::F64),
        ("bool_", Dtype::Bool),
        ("str_", Dtype::String),
    ] {
        let builtin = np.getattr(name)?;
        if scalar.is(&builtin) {
            let canonical = np.getattr("dtype")?.call1((builtin,))?;
            if !dtype.get_type().is(canonical.get_type())
                || (kind == Dtype::String && dtype.getattr("kind")?.extract::<String>()? != "U")
            {
                return Ok(None);
            }
            return Ok(Some(kind));
        }
    }
    Ok(None)
}

pub(super) fn shape(value: &Bound<'_, PyAny>) -> PyResult<(usize, usize, usize)> {
    let shape = value.getattr("shape")?.extract::<Vec<usize>>()?;
    match shape.as_slice() {
        [rows] => Ok((*rows, 1, 1)),
        [rows, columns] => Ok((*rows, *columns, 2)),
        _ => Err(PyTypeError::new_err(
            "input must be a vector or a two-dimensional rectangle",
        )),
    }
}

fn copy<T: Element + Copy>(
    data: &Bound<'_, PyAny>,
    missing: &[bool],
    shape: (usize, usize),
    expected_shape: &[usize],
) -> PyResult<Array2<Option<T>>> {
    let array = data.cast::<PyArrayDyn<T>>()?;
    let (actual_shape, values) = {
        let guard = array.try_readonly()?;
        let view = guard.as_array();
        (
            view.shape().to_vec(),
            view.iter().copied().collect::<Vec<_>>(),
        )
    };
    verify_shape(&actual_shape, expected_shape)?;
    if values.len() != missing.len() {
        return Err(crate::DataError::new_err(
            "data and validity shapes changed during binding",
        ));
    }
    shaped(
        shape,
        values
            .into_iter()
            .zip(missing)
            .map(|(value, missing)| (!missing).then_some(value))
            .collect(),
    )
}

fn verify_shape(actual: &[usize], expected: &[usize]) -> PyResult<()> {
    if actual != expected {
        return Err(crate::DataError::new_err(
            "NumPy payload/validity rank or shape changed during binding",
        ));
    }
    Ok(())
}

pub(super) fn missing(value: &Bound<'_, PyAny>, expected_shape: &[usize]) -> PyResult<Vec<bool>> {
    let array = value.cast::<PyArrayDyn<bool>>()?;
    let (actual_shape, result) = {
        let guard = array.try_readonly()?;
        let view = guard.as_array();
        (
            view.shape().to_vec(),
            view.iter().copied().collect::<Vec<_>>(),
        )
    };
    verify_shape(&actual_shape, expected_shape)?;
    Ok(result)
}

pub(super) fn numpy(value: &Bound<'_, PyAny>) -> PyResult<(Scratch, usize)> {
    numpy_with_dtype(value, None)
}

pub(super) fn numpy_series(
    value: &Bound<'_, PyAny>,
    rows: usize,
    declared_dtype: &Bound<'_, PyAny>,
) -> PyResult<Scratch> {
    verify_shape(value.cast::<PyUntypedArray>()?.shape(), &[rows])?;
    let (values, rank) = numpy_with_dtype(value, Some(declared_dtype))?;
    if rank != 1 || values.shape() != (rows, 1) {
        return Err(crate::DataError::new_err(
            "pandas Series carrier must retain its original one-dimensional shape",
        ));
    }
    Ok(values)
}

fn numpy_with_dtype(
    value: &Bound<'_, PyAny>,
    declared_dtype: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Scratch, usize)> {
    let py = value.py();
    let np = PyModule::import(py, "numpy")?;
    let ma = np.getattr("ma")?;
    // Do not cast the subclass to PyArray before this check.
    let masked = value.is_instance(&ma.getattr("MaskedArray")?)?;
    let (rows, columns, rank) = shape(value)?;
    let data = if masked {
        let kwargs = PyDict::new(py);
        kwargs.set_item("subok", false)?;
        ma.getattr("getdata")?.call((value,), Some(&kwargs))?
    } else {
        value.clone()
    };
    let untyped = data.cast::<PyUntypedArray>()?;
    let expected_shape = if rank == 1 {
        vec![rows]
    } else {
        vec![rows, columns]
    };
    if untyped.shape() != expected_shape.as_slice() {
        return Err(crate::DataError::new_err(
            "NumPy rank/shape metadata changed during preparation",
        ));
    }
    let original_dtype = untyped.dtype();
    let kind = dtype(original_dtype.as_any())?
        .ok_or_else(|| PyTypeError::new_err("unsupported NumPy logical dtype"))?;
    if let Some(declared) = declared_dtype {
        let declared = declared.cast::<PyArrayDescr>()?;
        let native = declared
            .call_method1("newbyteorder", ("=",))?
            .cast_into::<PyArrayDescr>()?;
        if !original_dtype.is_equiv_to(declared) && !original_dtype.is_equiv_to(&native) {
            return Err(PyTypeError::new_err(
                "pandas Series carrier differs from its declared logical dtype",
            ));
        }
    }
    let count = rows
        .checked_mul(columns)
        .ok_or_else(|| crate::DataError::new_err("array slot count is too large"))?;
    let mask = if masked {
        let mask = ma.getattr("getmaskarray")?.call1((value,))?;
        if mask.cast::<PyUntypedArray>()?.shape() != expected_shape.as_slice() {
            return Err(crate::DataError::new_err(
                "masked data and mask must have the same shape",
            ));
        }
        Some(mask)
    } else {
        None
    };
    let kwargs = PyDict::new(py);
    kwargs.set_item("copy", true)?;
    let target_dtype = if kind == Dtype::String {
        numpy::dtype::<Py<PyAny>>(py)
    } else {
        original_dtype
            .call_method1("newbyteorder", ("=",))?
            .cast_into::<PyArrayDescr>()?
    };
    let data = if kind == Dtype::String {
        // U-only admission above; object dtype is a carrier, not an input route.
        data.call_method("astype", (np.getattr("object_")?,), Some(&kwargs))?
    } else if !original_dtype.getattr("isnative")?.extract::<bool>()? {
        kwargs.set_item("casting", "equiv")?;
        data.call_method("astype", (&target_dtype,), Some(&kwargs))?
    } else {
        data
    };
    // All Python metadata/mask/carrier/endian calls above precede the first loan.
    if !data
        .cast::<PyUntypedArray>()?
        .dtype()
        .is_equiv_to(&target_dtype)
    {
        return Err(PyTypeError::new_err(
            "NumPy carrier differs from the expected normalized dtype",
        ));
    }
    let missing = mask.as_ref().map_or_else(
        || Ok(vec![false; count]),
        |mask| missing(mask, &expected_shape),
    )?;
    Ok((
        numpy_values(&data, &missing, (rows, columns), &expected_shape, kind)?,
        rank,
    ))
}

fn numpy_values(
    data: &Bound<'_, PyAny>,
    missing: &[bool],
    shape: (usize, usize),
    expected_shape: &[usize],
    kind: Dtype,
) -> PyResult<Scratch> {
    let py = data.py();
    macro_rules! primitive {
        ($variant:ident, $ty:ty) => {
            Scratch::$variant(copy::<$ty>(data, missing, shape, expected_shape)?)
        };
    }
    let values = match kind {
        Dtype::I8 => primitive!(I8, i8),
        Dtype::I16 => primitive!(I16, i16),
        Dtype::I32 => primitive!(I32, i32),
        Dtype::I64 => primitive!(I64, i64),
        Dtype::U8 => primitive!(U8, u8),
        Dtype::U16 => primitive!(U16, u16),
        Dtype::U32 => primitive!(U32, u32),
        Dtype::U64 => primitive!(U64, u64),
        Dtype::F32 => primitive!(F32, f32),
        Dtype::F64 => primitive!(F64, f64),
        Dtype::Bool => primitive!(Bool, bool),
        Dtype::String => {
            let array = data.cast::<PyArrayDyn<Py<PyAny>>>()?;
            let (actual_shape, refs) = {
                let guard = array.try_readonly()?;
                let view = guard.as_array();
                (
                    view.shape().to_vec(),
                    view.iter()
                        .map(|value| value.clone_ref(py))
                        .collect::<Vec<_>>(),
                )
            };
            verify_shape(&actual_shape, expected_shape)?;
            // No foreign view or guard survives String extraction / DECREF.
            let refs = refs
                .iter()
                .map(|value| value.bind(py).clone())
                .collect::<Vec<_>>();
            boxed(&refs, missing, shape, Dtype::String)?
        }
        Dtype::I128 | Dtype::U128 => unreachable!("NumPy has no admitted 128-bit integer dtype"),
    };
    Ok(values)
}

pub(super) fn boxed(
    values: &[Bound<'_, PyAny>],
    missing: &[bool],
    shape: (usize, usize),
    dtype: Dtype,
) -> PyResult<Scratch> {
    if values.len() != missing.len() {
        return Err(crate::DataError::new_err(
            "payload and validity lengths differ",
        ));
    }
    macro_rules! extract {
        ($variant:ident, $ty:ty, $allowed:expr) => {{
            let values = values
                .iter()
                .zip(missing)
                .map(|(value, missing)| {
                    if *missing {
                        return Ok(None);
                    }
                    if !($allowed)(value) {
                        return Err(PyTypeError::new_err(
                            "value does not match its declared logical dtype",
                        ));
                    }
                    value.extract::<$ty>().map(Some)
                })
                .collect::<PyResult<Vec<_>>>()?;
            Scratch::$variant(shaped(shape, values)?)
        }};
    }
    let integer = |value: &Bound<'_, PyAny>| {
        value.is_instance_of::<PyInt>() && !value.is_instance_of::<PyBool>()
    };
    let number = |value: &Bound<'_, PyAny>| {
        (value.is_instance_of::<PyInt>() || value.is_instance_of::<PyFloat>())
            && !value.is_instance_of::<PyBool>()
    };
    Ok(match dtype {
        Dtype::I8 => extract!(I8, i8, integer),
        Dtype::I16 => extract!(I16, i16, integer),
        Dtype::I32 => extract!(I32, i32, integer),
        Dtype::I64 => extract!(I64, i64, integer),
        Dtype::I128 => extract!(I128, i128, integer),
        Dtype::U8 => extract!(U8, u8, integer),
        Dtype::U16 => extract!(U16, u16, integer),
        Dtype::U32 => extract!(U32, u32, integer),
        Dtype::U64 => extract!(U64, u64, integer),
        Dtype::U128 => extract!(U128, u128, integer),
        Dtype::F32 => extract!(F32, f32, number),
        Dtype::F64 => extract!(F64, f64, number),
        Dtype::Bool => extract!(Bool, bool, |value: &Bound<'_, PyAny>| value
            .is_instance_of::<PyBool>()),
        Dtype::String => extract!(String, String, |value: &Bound<'_, PyAny>| value
            .is_instance_of::<PyString>()),
    })
}

/// Rehome an already admitted PL variable as typed Rust scratch, not native slots.
/// The construction origin, not this argument's container, owns repetition.
pub(super) fn from_polars(column: &Column, role: Role) -> PyResult<Scratch> {
    macro_rules! primitive {
        ($variant:ident, $accessor:ident) => {
            Scratch::$variant(shaped(
                (column.len(), 1),
                column
                    .$accessor()
                    .map_err(|error| crate::DataError::new_err(error.to_string()))?
                    .iter()
                    .collect(),
            )?)
        };
    }
    let series = column.as_materialized_series();
    let result = match column.dtype() {
        DataType::Int8 => primitive!(I8, i8),
        DataType::Int16 => primitive!(I16, i16),
        DataType::Int32 => primitive!(I32, i32),
        DataType::Int64 => primitive!(I64, i64),
        DataType::Int128 => primitive!(I128, i128),
        DataType::UInt8 => primitive!(U8, u8),
        DataType::UInt16 => primitive!(U16, u16),
        DataType::UInt32 => primitive!(U32, u32),
        DataType::UInt64 => primitive!(U64, u64),
        DataType::UInt128 => primitive!(U128, u128),
        DataType::Float32 => primitive!(F32, f32),
        DataType::Float64 => primitive!(F64, f64),
        DataType::Boolean => primitive!(Bool, bool),
        DataType::Float16 => Scratch::F64(shaped(
            (column.len(), 1),
            series
                .f16()
                .map_err(|error| crate::DataError::new_err(error.to_string()))?
                .iter()
                .map(|value| value.map(f64::from))
                .collect(),
        )?),
        DataType::String | DataType::Categorical(..) | DataType::Enum(..) => {
            // This native extractor admits only actual categorical logical types.
            lichtung_plot_polars::CategoryInput::bind_category(column, None, "hue")
                .map_err(super::pl_error)?;
            let decoded = column
                .cast(&DataType::String)
                .map_err(|error| crate::DataError::new_err(error.to_string()))?;
            Scratch::String(shaped(
                (column.len(), 1),
                decoded
                    .str()
                    .map_err(|error| crate::DataError::new_err(error.to_string()))?
                    .iter()
                    .map(|value| value.map(str::to_owned))
                    .collect(),
            )?)
        }
        DataType::Null if role != Role::Ids => {
            if role == Role::Category {
                Scratch::String(Array2::from_elem((column.len(), 1), None))
            } else {
                Scratch::F64(Array2::from_elem((column.len(), 1), None))
            }
        }
        _ => {
            return Err(PyTypeError::new_err(
                "unsupported Polars logical dtype for this role",
            ))
        }
    };
    Ok(result)
}

pub(super) fn single_column<'py>(
    value: &Bound<'py, PyAny>,
    index: usize,
) -> PyResult<Bound<'py, PyAny>> {
    let rows = pyo3::types::PySlice::full(value.py());
    let start = isize::try_from(index)
        .map_err(|_| crate::DataError::new_err("column index is too large"))?;
    let stop = start
        .checked_add(1)
        .ok_or_else(|| crate::DataError::new_err("column index is too large"))?;
    let column = pyo3::types::PySlice::new(value.py(), start, stop, 1);
    value.get_item((rows, column))
}

/// Only for the admitted pandas EA's boxed public payload, never a NumPy fallback.
pub(super) fn list_payload<'py>(value: &Bound<'py, PyAny>) -> PyResult<Vec<Bound<'py, PyAny>>> {
    Ok(value
        .call_method0("tolist")?
        .cast_into::<PyList>()?
        .iter()
        .collect())
}

pub(super) fn column_values(values: &Array2<Option<f64>>) -> ndarray::ArrayView1<'_, Option<f64>> {
    values.index_axis(Axis(1), 0)
}
