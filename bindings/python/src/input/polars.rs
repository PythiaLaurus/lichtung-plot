//! Bound eager Series only: public Arrow C stream, never the runtime's Rust ABI.
//!
//! Imported arrays are call-local adapter scratch. The caller must finish its
//! native snapshot and drop these Columns outside any Python owner borrow.

use std::collections::{HashSet, TryReserveError};

use polars_arrow::array::{new_empty_array, ArrayRef};
use polars_arrow::datatypes::{
    ArrowDataType, Field, IntegerType, DTYPE_CATEGORICAL_LEGACY, DTYPE_CATEGORICAL_NEW,
    DTYPE_ENUM_VALUES_LEGACY, DTYPE_ENUM_VALUES_NEW,
};
use polars_arrow::ffi::{ArrowArrayStream, ArrowArrayStreamReader};
use polars_core::prelude::{
    CategoricalPhysical, Column, DataType, IntoColumn, PolarsError, Series,
};
use pyo3::exceptions::{PyMemoryError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyDict};

const PRODUCER_VERSION: &str = "1.41.2";
const PRIMITIVES: [(&str, DataType, ArrowDataType); 13] = [
    ("Int8", DataType::Int8, ArrowDataType::Int8),
    ("Int16", DataType::Int16, ArrowDataType::Int16),
    ("Int32", DataType::Int32, ArrowDataType::Int32),
    ("Int64", DataType::Int64, ArrowDataType::Int64),
    ("Int128", DataType::Int128, ArrowDataType::Int128),
    ("UInt8", DataType::UInt8, ArrowDataType::UInt8),
    ("UInt16", DataType::UInt16, ArrowDataType::UInt16),
    ("UInt32", DataType::UInt32, ArrowDataType::UInt32),
    ("UInt64", DataType::UInt64, ArrowDataType::UInt64),
    ("UInt128", DataType::UInt128, ArrowDataType::UInt128),
    ("Float16", DataType::Float16, ArrowDataType::Float16),
    ("Float32", DataType::Float32, ArrowDataType::Float32),
    ("Float64", DataType::Float64, ArrowDataType::Float64),
];

/// Metadata-only concrete identity mapping; subclasses and logical storage types
/// are deliberately not primitive candidates.
pub(crate) fn primitive_dtype(dtype: &Bound<'_, PyAny>) -> PyResult<Option<DataType>> {
    let classes = PyModule::import(dtype.py(), "polars.datatypes.classes")?;
    primitive_dtype_in(dtype, &classes)
}

fn primitive_dtype_in(
    dtype: &Bound<'_, PyAny>,
    classes: &Bound<'_, PyModule>,
) -> PyResult<Option<DataType>> {
    for (name, native, _) in PRIMITIVES {
        if exact_dtype(dtype, classes, name)? {
            return Ok(Some(native));
        }
    }
    Ok(None)
}

fn exact_dtype(
    dtype: &Bound<'_, PyAny>,
    classes: &Bound<'_, PyModule>,
    name: &str,
) -> PyResult<bool> {
    let class = classes.getattr(name)?;
    Ok(dtype.is(&class) || dtype.get_type().is(&class))
}

#[derive(Debug)]
enum LogicalType {
    Primitive(DataType),
    String,
    Boolean,
    Null,
    Categorical {
        name: String,
        namespace: String,
        physical: CategoricalPhysical,
    },
    Enum(Vec<String>),
}

struct ExpectedSeries {
    name: String,
    len: usize,
    logical: LogicalType,
}

/// Only the genuine, concrete eager Series family is admitted here. Type/version
/// checks narrow compatibility; they do NOT prove a capsule's memory safety.
pub(crate) fn import_series(series: &Bound<'_, PyAny>, role: &str) -> PyResult<Column> {
    let py = series.py();
    let series_class = PyModule::import(py, "polars.series.series")?.getattr("Series")?;
    if !series.get_type().is(&series_class) {
        return Err(PyTypeError::new_err(format!(
            "{role} requires a concrete eager polars.Series"
        )));
    }
    check_producer_version(py, role)?;
    let name = series.getattr("name")?.extract::<String>()?;
    let len = series.len()?;
    let dtype = series.getattr("dtype")?;
    let classes = PyModule::import(py, "polars.datatypes.classes")?;
    let logical = inspect_logical(&dtype, &classes, role, &name)?;
    let expected = ExpectedSeries { name, len, logical };

    // All Python calls and owned metadata extraction precede the pointer loan.
    // In particular, no Plot/Canvas/PyRef/lock is accepted by this module.
    let capsule = series
        .call_method0("__arrow_c_stream__")?
        .cast_into::<PyCapsule>()?;
    consume_capsule(capsule, &expected, role)
}

fn check_producer_version(py: Python<'_>, role: &str) -> PyResult<()> {
    let polars = PyModule::import(py, "polars")?;
    let version = polars.getattr("__version__")?.extract::<String>()?;
    // _plr is inspected for loaded-module identity/version only, never for data
    // or a private exporter. Do not accidentally accept runtime-64/compat.
    let runtime = PyModule::import(py, "polars._plr")?;
    let runtime_version = runtime.getattr("__version__")?.extract::<String>()?;
    let modules = PyModule::import(py, "sys")?
        .getattr("modules")?
        .cast_into::<PyDict>()?;
    let runtime32 = modules.get_item("_polars_runtime_32._polars_runtime")?;
    if version != PRODUCER_VERSION
        || runtime_version != PRODUCER_VERSION
        || !runtime32.is_some_and(|module| runtime.is(&module))
    {
        return Err(PyTypeError::new_err(format!(
            "{role}: Polars Arrow import requires polars/runtime-32 {PRODUCER_VERSION}; \
             found Python {version:?}, runtime {runtime_version:?}"
        )));
    }
    Ok(())
}

fn inspect_logical(
    dtype: &Bound<'_, PyAny>,
    classes: &Bound<'_, PyModule>,
    role: &str,
    name: &str,
) -> PyResult<LogicalType> {
    if let Some(native) = primitive_dtype_in(dtype, classes)? {
        return Ok(LogicalType::Primitive(native));
    }
    for (class, logical) in [
        ("String", LogicalType::String),
        ("Boolean", LogicalType::Boolean),
        ("Null", LogicalType::Null),
    ] {
        if exact_dtype(dtype, classes, class)? {
            return Ok(logical);
        }
    }
    // A Series' parameterized logical dtype must be an actual dtype instance.
    if dtype.get_type().is(&classes.getattr("Categorical")?) {
        let categories = dtype.getattr("categories")?;
        let physical = categories.call_method0("physical")?;
        let physical = match primitive_dtype_in(&physical, classes)? {
            Some(DataType::UInt8) => CategoricalPhysical::U8,
            Some(DataType::UInt16) => CategoricalPhysical::U16,
            Some(DataType::UInt32) => CategoricalPhysical::U32,
            _ => {
                return Err(PyTypeError::new_err(format!(
                    "{role} field {name:?}: unsupported Categorical physical dtype"
                )));
            }
        };
        return Ok(LogicalType::Categorical {
            name: categories.call_method0("name")?.extract()?,
            namespace: categories.call_method0("namespace")?.extract()?,
            physical,
        });
    }
    if dtype.get_type().is(&classes.getattr("Enum")?) {
        // Enum levels are logical metadata, including unused levels. Index the
        // public categories Series; never transport observation data as a list.
        let categories = dtype.getattr("categories")?;
        let count = categories.len()?;
        let mut levels = Vec::new();
        levels
            .try_reserve_exact(count)
            .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
        for index in 0..count {
            levels.push(categories.get_item(index)?.extract::<String>()?);
        }
        validate_expected_levels(&levels).map_err(|error| error.into_python(role, name))?;
        return Ok(LogicalType::Enum(levels));
    }
    // repr is diagnostic only; admission used concrete object/type identities.
    Err(PyTypeError::new_err(format!(
        "{role} field {name:?}: unsupported Polars logical dtype {}; \
         temporal, Decimal, Binary, nested, Object and Extension values are not admitted",
        dtype.repr()?
    )))
}

#[derive(Debug)]
enum ImportError {
    Incompatible(String),
    Polars(PolarsError),
    Allocation(TryReserveError),
}

type ImportResult<T> = Result<T, ImportError>;

impl From<PolarsError> for ImportError {
    fn from(error: PolarsError) -> Self {
        Self::Polars(error)
    }
}

impl From<TryReserveError> for ImportError {
    fn from(error: TryReserveError) -> Self {
        Self::Allocation(error)
    }
}

impl ImportError {
    fn into_python(self, role: &str, name: &str) -> PyErr {
        let context = format!("{role} field {name:?}: Polars Arrow import");
        match self {
            Self::Incompatible(message) => PyTypeError::new_err(format!("{context}: {message}")),
            Self::Polars(error) => crate::DataError::new_err(format!("{context}: {error}")),
            Self::Allocation(error) => PyMemoryError::new_err(format!("{context}: {error}")),
        }
    }
}

fn incompatible(message: impl Into<String>) -> ImportError {
    ImportError::Incompatible(message.into())
}

/// The fresh capsule remains the sole stream owner, including on failure.
#[allow(unsafe_code)]
fn consume_capsule(
    capsule: Bound<'_, PyCapsule>,
    expected: &ExpectedSeries,
    role: &str,
) -> PyResult<Column> {
    let pointer = capsule
        .pointer_checked(Some(c"arrow_array_stream"))?
        .cast::<ArrowArrayStream>();
    let imported: ImportResult<Column> = {
        // SAFETY: the supported, genuine producer must return a fresh, conforming
        // Arrow C stream with valid immutable data and independently releasable
        // schema/arrays. Type, version, GIL and capsule name are NOT that proof.
        // This is its sole short mutable loan: capsule is strongly held, nothing
        // calls Python/reenters/detaches, and no owner is borrowed in this block.
        // Do not move the stream out, invoke release, or reconstruct its Box.
        let stream = unsafe { &mut *pointer.as_ptr() };
        // SAFETY: the same producer contract and exclusive loan cover every
        // callback executed synchronously by the reader.
        unsafe { consume_stream(stream, expected) }
    };
    // Reader/reference are gone before DECREF/finalizers and PyErr conversion.
    // Imported arrays own separate C-data handles and may outlive this capsule.
    drop(capsule);
    imported.map_err(|error| error.into_python(role, &expected.name))
}

/// # Safety
/// `stream` must be an exclusively loaned conforming Arrow C stream. All callback
/// pointers, UTF-8 error messages, schemas, arrays, offsets, dictionaries and
/// validity/buffer lifetimes must satisfy that contract, also on error. Callbacks
/// must not reenter Python. The stream's owner must outlive this entire call.
#[allow(unsafe_code)]
unsafe fn consume_stream(
    stream: &mut ArrowArrayStream,
    expected: &ExpectedSeries,
) -> ImportResult<Column> {
    // SAFETY: caller supplies a valid exclusive stream and callback contract.
    let mut reader = unsafe { ArrowArrayStreamReader::try_new(stream)? };
    let field = reader.field().clone();
    // Guard BEFORE next()/infallible upstream logical-dtype/metadata parsers.
    validate_field(&field, &expected.logical)?;
    let mut chunks: Vec<ArrayRef> = Vec::new();
    let mut len = 0_usize;
    // SAFETY: the reader retains the same exclusive stream loan. A zero-length
    // owned array is not EOS; only the reader's None signals release=None.
    while let Some(chunk) = unsafe { reader.next() } {
        let chunk = chunk?;
        len = len
            .checked_add(chunk.len())
            .ok_or_else(|| incompatible("chunk lengths overflow usize"))?;
        if len > expected.len {
            return Err(incompatible("stream is longer than the expected Series"));
        }
        chunks.try_reserve(1)?;
        chunks.push(chunk);
    }
    if len != expected.len {
        return Err(incompatible(format!(
            "stream length {len} differs from Series length {}",
            expected.len
        )));
    }
    if chunks.is_empty() {
        // Only a genuine zero-row Series reaches this branch.
        chunks.try_reserve(1)?;
        chunks.push(new_empty_array(field.dtype.clone()));
    }
    // This safe, Field-aware conversion retains Categorical/Enum metadata.
    let mut series = Series::try_from((&field, chunks))?;
    validate_imported_dtype(series.dtype(), &expected.logical)?;
    if series.len() != expected.len {
        return Err(incompatible("Series conversion changed the stream length"));
    }
    series.rename(expected.name.as_str().into());
    Ok(series.into_column())
}

fn is_string(dtype: &ArrowDataType) -> bool {
    matches!(
        dtype,
        ArrowDataType::Utf8 | ArrowDataType::LargeUtf8 | ArrowDataType::Utf8View
    )
}

fn metadata<'a>(field: &'a Field, key: &str) -> Option<&'a str> {
    field
        .metadata
        .as_ref()?
        .get(key)
        .map(polars_core::datatypes::PlSmallStr::as_str)
}

fn validate_field(field: &Field, expected: &LogicalType) -> ImportResult<()> {
    let categorical_keys = [
        DTYPE_CATEGORICAL_NEW,
        DTYPE_CATEGORICAL_LEGACY,
        DTYPE_ENUM_VALUES_NEW,
        DTYPE_ENUM_VALUES_LEGACY,
    ];
    let shape_matches = match expected {
        LogicalType::Primitive(native) => PRIMITIVES
            .iter()
            .any(|(_, dtype, arrow)| dtype == native && arrow == &field.dtype),
        LogicalType::String => is_string(&field.dtype),
        LogicalType::Boolean => field.dtype == ArrowDataType::Boolean,
        LogicalType::Null => field.dtype == ArrowDataType::Null,
        LogicalType::Categorical { .. } | LogicalType::Enum(_) => {
            validate_dictionary(&field.dtype)?;
            return validate_category_metadata(field, expected);
        }
    };
    if !shape_matches {
        return Err(incompatible(format!(
            "Arrow logical dtype {:?} does not preserve {expected:?}",
            field.dtype
        )));
    }
    if categorical_keys
        .iter()
        .any(|key| metadata(field, key).is_some())
    {
        return Err(incompatible(
            "categorical metadata on a non-categorical Field",
        ));
    }
    Ok(())
}

fn validate_dictionary(dtype: &ArrowDataType) -> ImportResult<()> {
    if let ArrowDataType::Dictionary(key, values, _) = dtype {
        if matches!(
            key,
            IntegerType::Int8
                | IntegerType::Int16
                | IntegerType::Int32
                | IntegerType::Int64
                | IntegerType::UInt8
                | IntegerType::UInt16
                | IntegerType::UInt32
                | IntegerType::UInt64
        ) && is_string(values)
        {
            return Ok(());
        }
    }
    Err(incompatible(
        "Categorical/Enum requires a supported string dictionary",
    ))
}

/// Validate byte lengths and UTF-8 boundaries before Polars' split_at/unwrap
/// metadata parser. Names/levels may themselves contain semicolons or be empty.
fn take_metadata_string<'a>(remaining: &mut &'a str) -> ImportResult<&'a str> {
    let (length, rest) = remaining
        .split_once(';')
        .ok_or_else(|| incompatible("missing metadata byte-length separator"))?;
    if length.is_empty() || !length.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(incompatible("invalid metadata byte length"));
    }
    let length = length
        .parse::<usize>()
        .map_err(|_| incompatible("metadata byte length overflows usize"))?;
    let value = rest
        .get(..length)
        .ok_or_else(|| incompatible("metadata byte length exceeds a UTF-8 string boundary"))?;
    *remaining = rest
        .get(length..)
        .ok_or_else(|| incompatible("metadata byte length exceeds a UTF-8 string boundary"))?;
    Ok(value)
}

fn validate_expected_levels(levels: &[String]) -> ImportResult<()> {
    if u32::try_from(levels.len()).is_err() {
        return Err(incompatible("Enum exceeds native category capacity"));
    }
    let mut seen = HashSet::new();
    seen.try_reserve(levels.len())?;
    if levels.iter().any(|level| !seen.insert(level.as_str())) {
        return Err(incompatible("Enum metadata contains duplicate levels"));
    }
    Ok(())
}

fn validate_enum_metadata(mut encoded: &str, levels: &[String]) -> ImportResult<()> {
    // This also protects direct member-local consumers of consume_stream.
    validate_expected_levels(levels)?;
    for expected in levels {
        if take_metadata_string(&mut encoded)? != expected.as_str() {
            return Err(incompatible(
                "Enum metadata differs from the full Python level order",
            ));
        }
    }
    if !encoded.is_empty() {
        return Err(incompatible(
            "Enum metadata has extra levels or trailing bytes",
        ));
    }
    Ok(())
}

fn validate_category_metadata(field: &Field, expected: &LogicalType) -> ImportResult<()> {
    match expected {
        LogicalType::Categorical {
            name,
            namespace,
            physical,
        } => {
            if [DTYPE_ENUM_VALUES_NEW, DTYPE_ENUM_VALUES_LEGACY]
                .iter()
                .any(|key| metadata(field, key).is_some())
            {
                return Err(incompatible("Enum metadata on a Categorical Field"));
            }
            let mut encoded = metadata(field, DTYPE_CATEGORICAL_NEW)
                .ok_or_else(|| incompatible("missing full Categorical metadata"))?;
            let actual_name = take_metadata_string(&mut encoded)?;
            let actual_namespace = take_metadata_string(&mut encoded)?;
            let actual_physical = match encoded {
                "u8;" => CategoricalPhysical::U8,
                "u16;" => CategoricalPhysical::U16,
                "u32;" => CategoricalPhysical::U32,
                _ => return Err(incompatible("invalid Categorical physical metadata")),
            };
            if actual_name != name.as_str()
                || actual_namespace != namespace.as_str()
                || actual_physical != *physical
            {
                return Err(incompatible(
                    "Categorical name/namespace/physical metadata changed",
                ));
            }
        }
        LogicalType::Enum(levels) => {
            if [DTYPE_CATEGORICAL_NEW, DTYPE_CATEGORICAL_LEGACY]
                .iter()
                .any(|key| metadata(field, key).is_some())
            {
                return Err(incompatible("Categorical metadata on an Enum Field"));
            }
            let mut found = false;
            for key in [DTYPE_ENUM_VALUES_NEW, DTYPE_ENUM_VALUES_LEGACY] {
                if let Some(encoded) = metadata(field, key) {
                    validate_enum_metadata(encoded, levels)?;
                    found = true;
                }
            }
            if !found {
                return Err(incompatible("missing full Enum metadata"));
            }
        }
        _ => return Err(incompatible("unexpected categorical metadata validator")),
    }
    Ok(())
}

fn validate_imported_dtype(dtype: &DataType, expected: &LogicalType) -> ImportResult<()> {
    let matches = match (expected, dtype) {
        (LogicalType::Primitive(expected), actual) => expected == actual,
        (LogicalType::String, DataType::String)
        | (LogicalType::Boolean, DataType::Boolean)
        | (LogicalType::Null, DataType::Null) => true,
        (
            LogicalType::Categorical {
                name,
                namespace,
                physical,
            },
            DataType::Categorical(categories, _),
        ) => {
            categories.name().as_str() == name.as_str()
                && categories.namespace().as_str() == namespace.as_str()
                && categories.physical() == *physical
        }
        (LogicalType::Enum(expected), DataType::Enum(categories, _)) => categories
            .categories()
            .values_iter()
            .eq(expected.iter().map(String::as_str)),
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(incompatible(format!(
            "imported logical dtype {dtype:?} does not preserve {expected:?}"
        )))
    }
}

#[cfg(test)]
mod tests;
