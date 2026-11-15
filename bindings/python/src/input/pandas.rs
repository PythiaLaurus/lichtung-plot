//! Public pandas interfaces, exact complete keys, and bound-only value imports.

use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyInt, PyList, PySlice, PyString, PyTuple};

use super::array::{self, Dtype, Role, Scratch};

pub(super) struct Table<'py> {
    object: Bound<'py, PyAny>,
    columns: Bound<'py, PyAny>,
    pub(super) keys: Vec<Bound<'py, PyAny>>,
    pub(super) shape: (usize, usize),
}

impl<'py> Table<'py> {
    pub(super) fn new(object: &Bound<'py, PyAny>) -> PyResult<Self> {
        let shape = object.getattr("shape")?.extract::<(usize, usize)>()?;
        let kwargs = PyDict::new(object.py());
        kwargs.set_item("deep", true)?;
        let columns = object
            .getattr("columns")?
            .call_method("copy", (), Some(&kwargs))?;
        let keys = columns
            .call_method0("to_list")?
            .cast_into::<PyList>()?
            .iter()
            .collect::<Vec<_>>();
        if keys.len() != shape.1 {
            return Err(crate::DataError::new_err(
                "pandas columns and shape disagree",
            ));
        }
        Ok(Self {
            object: object.clone(),
            columns,
            keys,
            shape,
        })
    }

    pub(super) fn verify(&self) -> PyResult<()> {
        if self.object.getattr("shape")?.extract::<(usize, usize)>()? != self.shape
            || !self
                .object
                .getattr("columns")?
                .call_method1("equals", (&self.columns,))?
                .extract::<bool>()?
        {
            return Err(crate::DataError::new_err(
                "pandas schema changed during input preparation",
            ));
        }
        Ok(())
    }

    /// None is a missing complete key, never a request for direct-data fallback.
    pub(super) fn exact(&self, query: &Bound<'_, PyAny>) -> PyResult<Option<usize>> {
        self.locate(query, true)
    }

    fn locate(&self, query: &Bound<'_, PyAny>, missing_ok: bool) -> PyResult<Option<usize>> {
        query.hash()?;
        let pd = PyModule::import(query.py(), "pandas")?;
        if self.columns.is_instance(&pd.getattr("MultiIndex")?)? {
            let Ok(tuple) = query.cast::<PyTuple>() else {
                return Ok(None);
            };
            if tuple.len() != self.columns.getattr("nlevels")?.extract::<usize>()? {
                return Ok(None);
            }
        }
        let mut complete = false;
        for actual in &self.keys {
            if complete_equal(actual, query)? {
                complete = true;
                break;
            }
        }
        if !complete {
            return Ok(None);
        }
        // A unique integer locator alone would admit partial date/Interval queries.
        let loc = match self.columns.call_method1("get_loc", (query,)) {
            Ok(loc) => loc,
            Err(error) if missing_ok && error.is_instance_of::<PyKeyError>(query.py()) => {
                return Ok(None)
            }
            Err(error) => return Err(error),
        };
        if loc.is_instance_of::<PyBool>()
            || !(loc.is_instance_of::<PyInt>() || super::instance(&loc, "numpy", "integer")?)
        {
            return Ok(None);
        }
        let index = loc.extract::<usize>()?;
        let Some(canonical) = self.keys.get(index) else {
            return Err(crate::DataError::new_err(
                "pandas returned an out-of-bounds column locator",
            ));
        };
        if !complete_equal(canonical, query)? {
            return Ok(None);
        }
        Ok(Some(index))
    }

    pub(super) fn require(&self, query: &Bound<'_, PyAny>) -> PyResult<usize> {
        // Preserve pandas/user exception context on the actual requested key.
        // Registry probes alone interpret get_loc's missing-key protocol.
        self.locate(query, false)?
            .ok_or_else(|| PyKeyError::new_err("expected one exact complete pandas column label"))
    }

    fn verify_column(&self, index: usize) -> PyResult<()> {
        self.verify()?;
        // Index.equals alone can equate missing labels which the actual locator
        // distinguishes. Reprove this bound slot against current original keys
        // before/after iloc; unused duplicate labels remain metadata-only.
        let current = Self::new(&self.object)?;
        if current.exact(&self.keys[index])? != Some(index) {
            return Err(crate::DataError::new_err(
                "pandas bound column identity changed during input preparation",
            ));
        }
        current.verify()?;
        self.verify()
    }

    pub(super) fn import(&self, index: usize, role: Role) -> PyResult<Scratch> {
        self.verify_column(index)?;
        let series = self
            .object
            .getattr("iloc")?
            .get_item((PySlice::full(self.object.py()), index))?;
        if !complete_equal(&series.getattr("name")?, &self.keys[index])? {
            return Err(crate::DataError::new_err(
                "pandas selected a different column during preparation",
            ));
        }
        let data = series_values(&series, role)?;
        self.verify_column(index)?;
        if data.shape() != (self.shape.0, 1) {
            return Err(crate::DataError::new_err(
                "pandas selected column has changed shape",
            ));
        }
        Ok(data)
    }

    pub(super) fn numeric_indices(&self) -> PyResult<Vec<usize>> {
        // dtypes is schema metadata, not a whole-table observation conversion.
        let dtypes = self.object.getattr("dtypes")?.call_method0("to_list")?;
        let dtypes = dtypes.cast::<PyList>()?;
        if dtypes.len() != self.shape.1 {
            return Err(crate::DataError::new_err(
                "pandas dtype metadata changed during preparation",
            ));
        }
        let mut result = Vec::new();
        for (index, dtype) in dtypes.iter().enumerate() {
            if logical_dtype(&dtype)?.is_some_and(Dtype::numeric) {
                self.require(&self.keys[index])?;
                result.push(index);
            }
        }
        self.verify()?;
        Ok(result)
    }
}

/// Object singleton equality proves complete values without invoking a query API.
/// Original get_loc remains the uniqueness/missing-equivalence authority.
pub(super) fn complete_equal(
    actual: &Bound<'_, PyAny>,
    query: &Bound<'_, PyAny>,
) -> PyResult<bool> {
    match (actual.cast::<PyTuple>(), query.cast::<PyTuple>()) {
        (Ok(left), Ok(right)) => {
            if left.len() != right.len() {
                return Ok(false);
            }
            for (left, right) in left.iter().zip(right.iter()) {
                if !complete_equal(&left, &right)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        (Ok(_), Err(_)) | (Err(_), Ok(_)) => return Ok(false),
        (Err(_), Err(_)) => {}
    }
    let left = singleton_index(actual)?;
    let right = singleton_index(query)?;
    left.call_method1("equals", (right,))?.extract()
}

fn singleton_index<'py>(key: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let pd = PyModule::import(key.py(), "pandas")?;
    let kwargs = PyDict::new(key.py());
    kwargs.set_item("dtype", "object")?;
    kwargs.set_item("tupleize_cols", false)?;
    pd.getattr("Index")?
        .call((PyList::new(key.py(), [key])?,), Some(&kwargs))
}

/// A direct Series name has no parent Index. Use an actual singleton object
/// Index's complete-key and locator semantics, including distinct missing keys.
pub(super) fn same_key(actual: &Bound<'_, PyAny>, query: &Bound<'_, PyAny>) -> PyResult<bool> {
    query.hash()?;
    if !complete_equal(actual, query)? {
        return Ok(false);
    }
    let index = singleton_index(actual)?;
    let loc = match index.call_method1("get_loc", (query,)) {
        Ok(loc) => loc,
        Err(error) if error.is_instance_of::<PyKeyError>(query.py()) => return Ok(false),
        Err(error) => return Err(error),
    };
    Ok(!loc.is_instance_of::<PyBool>()
        && (loc.is_instance_of::<PyInt>() || super::instance(&loc, "numpy", "integer")?)
        && loc.extract::<usize>()? == 0)
}

pub(super) fn logical_dtype(dtype: &Bound<'_, PyAny>) -> PyResult<Option<Dtype>> {
    let py = dtype.py();
    let pd = PyModule::import(py, "pandas")?;
    let np = PyModule::import(py, "numpy")?;
    if dtype.is_instance(&np.getattr("dtype")?)? {
        return array::dtype(dtype);
    }
    for (name, kind) in [
        ("Int8Dtype", Dtype::I8),
        ("Int16Dtype", Dtype::I16),
        ("Int32Dtype", Dtype::I32),
        ("Int64Dtype", Dtype::I64),
        ("UInt8Dtype", Dtype::U8),
        ("UInt16Dtype", Dtype::U16),
        ("UInt32Dtype", Dtype::U32),
        ("UInt64Dtype", Dtype::U64),
        ("Float32Dtype", Dtype::F32),
        ("Float64Dtype", Dtype::F64),
        ("BooleanDtype", Dtype::Bool),
        ("StringDtype", Dtype::String),
        ("CategoricalDtype", Dtype::String),
    ] {
        // A custom EA cannot be admitted by claiming a primitive kind/storage.
        if dtype.get_type().is(pd.getattr(name)?) {
            return Ok(Some(kind));
        }
    }
    if dtype.get_type().is(pd.getattr("ArrowDtype")?) {
        let arrow = dtype.getattr("pyarrow_dtype")?;
        let types = PyModule::import(py, "pyarrow.types")?;
        for (predicate, kind) in [
            ("is_int8", Dtype::I8),
            ("is_int16", Dtype::I16),
            ("is_int32", Dtype::I32),
            ("is_int64", Dtype::I64),
            ("is_uint8", Dtype::U8),
            ("is_uint16", Dtype::U16),
            ("is_uint32", Dtype::U32),
            ("is_uint64", Dtype::U64),
            ("is_float32", Dtype::F32),
            ("is_float64", Dtype::F64),
            ("is_boolean", Dtype::Bool),
            ("is_string", Dtype::String),
            ("is_large_string", Dtype::String),
        ] {
            if types
                .getattr(predicate)?
                .call1((&arrow,))?
                .extract::<bool>()?
            {
                return Ok(Some(kind));
            }
        }
    }
    Ok(None)
}

pub(super) fn series_values(series: &Bound<'_, PyAny>, role: Role) -> PyResult<Scratch> {
    let py = series.py();
    let pd = PyModule::import(py, "pandas")?;
    let np = PyModule::import(py, "numpy")?;
    let rows = series.len()?;
    let dtype = series.getattr("dtype")?;
    let numpy_dtype = dtype.is_instance(&np.getattr("dtype")?)?;
    let object_category =
        numpy_dtype && dtype.getattr("kind")?.extract::<String>()? == "O" && role == Role::Category;
    let kind = if object_category {
        Dtype::String
    } else {
        logical_dtype(&dtype)?
            .ok_or_else(|| PyTypeError::new_err("unsupported pandas logical dtype"))?
    };
    if numpy_dtype && !object_category {
        // Ordinary NumPy floats have no independent null bitmap. Never isna→null.
        let kwargs = PyDict::new(py);
        kwargs.set_item("copy", true)?;
        let carrier = series.call_method("to_numpy", (), Some(&kwargs))?;
        let values = array::numpy_series(&carrier, rows, &dtype)?;
        if series.len()? != rows {
            return Err(crate::DataError::new_err(
                "pandas Series length changed during input preparation",
            ));
        }
        if !series.getattr("dtype")?.eq(&dtype)? {
            return Err(PyTypeError::new_err(
                "pandas Series logical dtype changed during input preparation",
            ));
        }
        return Ok(values);
    }
    if dtype.get_type().is(pd.getattr("CategoricalDtype")?) {
        let categories = dtype.getattr("categories")?;
        let category_dtype = categories.getattr("dtype")?;
        let numpy_categories = category_dtype.is_instance(&np.getattr("dtype")?)?;
        if numpy_categories {
            let kind = category_dtype.getattr("kind")?.extract::<String>()?;
            if kind != "O" && kind != "U" {
                return Err(PyTypeError::new_err(
                    "categorical observations must have a string domain",
                ));
            }
        } else if logical_dtype(&category_dtype)? != Some(Dtype::String) {
            return Err(PyTypeError::new_err(
                "categorical observations must have a string domain",
            ));
        }
        for category in categories.call_method0("to_list")?.cast::<PyList>()?.iter() {
            if !category.is_instance_of::<PyString>() {
                return Err(PyTypeError::new_err(
                    "categorical observations must have a string domain",
                ));
            }
        }
    }
    let array = series.getattr("array")?;
    // Both Python calls complete before borrowing the bitmap or extracting cells.
    let mask = array.call_method0("isna")?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("dtype", np.getattr("object_")?)?;
    kwargs.set_item("copy", true)?;
    kwargs.set_item("na_value", pd.getattr("NA")?)?;
    let payload = array.call_method("to_numpy", (), Some(&kwargs))?;
    let payload = array::list_payload(&payload)?;
    let missing = array::missing(&mask, &[rows])?;
    array::boxed(&payload, &missing, (rows, 1), kind)
}
