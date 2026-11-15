//! Exact Scatter entry points and short-lived preparation over the four routes.

use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{kind, CategoryValues, NumericValues, Plot, SourceRows};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use ndarray::Array2;
use polars_core::prelude::{Column, DataFrame, DataType};
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyInt, PyString};

use super::array::{self, Role, Scratch};
use super::pandas;
use super::selectors::{self, Selection};
use super::{Config, InputRoots, Origin, Prepared};

type ScatterConfig = Config<kind::Scatter>;

// No Python objects, key equality or identity recycling in this allocator.
// A copy can independently bind different new keys without reusing a raw ID.
static NEXT_KEY: AtomicU64 = AtomicU64::new(0);

pub(super) struct Selected {
    pub(super) indices: Vec<usize>,
    pub(super) collection: bool,
    discovered: Vec<(usize, DataType)>,
}

/// This finite enum is call-local container dispatch, not a retained source model.
pub(super) enum Source<'py> {
    Numpy {
        object: Bound<'py, PyAny>,
        shape: (usize, usize),
    },
    Pandas(pandas::Table<'py>),
    Polars {
        object: Bound<'py, PyAny>,
        shape: (usize, usize),
        names: Vec<String>,
    },
}

impl<'py> Source<'py> {
    pub(super) fn new(object: &Bound<'py, PyAny>) -> PyResult<Self> {
        if super::instance(object, "numpy", "ndarray")? {
            let (rows, columns, rank) = array::shape(object)?;
            if rank != 2 {
                return Err(PyTypeError::new_err("data must be a two-dimensional table"));
            }
            Ok(Self::Numpy {
                object: object.clone(),
                shape: (rows, columns),
            })
        } else if super::instance(object, "pandas", "DataFrame")? {
            Ok(Self::Pandas(pandas::Table::new(object)?))
        } else if super::instance(object, "polars", "DataFrame")? {
            let shape = object.getattr("shape")?.extract::<(usize, usize)>()?;
            let names = object.getattr("columns")?.extract::<Vec<String>>()?;
            if shape.1 != names.len() {
                return Err(crate::DataError::new_err(
                    "Polars columns and shape disagree",
                ));
            }
            Ok(Self::Polars {
                object: object.clone(),
                shape,
                names,
            })
        } else {
            Err(PyTypeError::new_err(
                "data must be a NumPy, pandas or eager Polars table",
            ))
        }
    }

    pub(super) fn shape(&self) -> (usize, usize) {
        match self {
            Self::Numpy { shape, .. } | Self::Polars { shape, .. } => *shape,
            Self::Pandas(table) => table.shape,
        }
    }

    pub(super) fn verify(&self) -> PyResult<()> {
        match self {
            Self::Numpy { object, shape } => {
                if array::shape(object)? != (shape.0, shape.1, 2) {
                    return Err(crate::DataError::new_err(
                        "NumPy shape changed during preparation",
                    ));
                }
            }
            Self::Pandas(table) => table.verify()?,
            Self::Polars {
                object,
                shape,
                names,
            } => {
                if object.getattr("shape")?.extract::<(usize, usize)>()? != *shape
                    || object.getattr("columns")?.extract::<Vec<String>>()? != *names
                {
                    return Err(crate::DataError::new_err(
                        "Polars schema changed during preparation",
                    ));
                }
            }
        }
        Ok(())
    }

    fn position(&self, key: &Bound<'_, PyAny>) -> PyResult<usize> {
        match self {
            Self::Numpy { shape, .. } => selectors::position(key, shape.1),
            Self::Pandas(table) => table.require(key),
            Self::Polars { names, .. } => {
                let name = key
                    .extract::<String>()
                    .map_err(|_| PyTypeError::new_err("Polars column labels must be strings"))?;
                names
                    .iter()
                    .position(|candidate| *candidate == name)
                    .ok_or_else(|| PyKeyError::new_err(name))
            }
        }
    }

    pub(super) fn resolve(&self, value: &Bound<'_, PyAny>) -> PyResult<Option<Selected>> {
        let description = selectors::description(value)?;
        let description = if let Some(description) = description {
            description
        } else if direct_container(value)? {
            return Ok(None);
        } else if value.is_instance_of::<PyString>()
            || value.is_instance_of::<PyInt>()
            || super::instance(value, "numpy", "integer")?
            || (matches!(self, Self::Numpy { .. }) && value.hasattr("__index__")?)
        {
            Selection::One(value.clone().unbind())
        } else {
            return Ok(None);
        };
        let py = value.py();
        let (indices, collection) = match description {
            Selection::One(key) => (vec![self.position(key.bind(py))?], false),
            Selection::Keys(keys) => (
                keys.iter()
                    .map(|key| self.position(key.bind(py)))
                    .collect::<PyResult<Vec<_>>>()?,
                true,
            ),
            Selection::Slice(slice) => {
                let indices = selectors::slice(slice.bind(py), self.shape().1)?;
                if let Self::Pandas(table) = self {
                    for &index in &indices {
                        if table.require(&table.keys[index])? != index {
                            return Err(crate::DataError::new_err(
                                "pandas positional key is ambiguous",
                            ));
                        }
                    }
                }
                (indices, true)
            }
        };
        selectors::nonempty_unique(&indices)?;
        self.verify()?;
        Ok(Some(Selected {
            indices,
            collection,
            discovered: Vec::new(),
        }))
    }

    pub(super) fn numpy_column(&self, index: usize, role: Role) -> PyResult<Scratch> {
        self.verify()?;
        let column = match self {
            Self::Numpy { object, .. } => array::numpy(&array::single_column(object, index)?)?.0,
            Self::Pandas(table) => table.import(index, role)?,
            Self::Polars { .. } => array::from_polars(&self.polars_column(index, role)?, role)?,
        };
        if column.shape() != (self.shape().0, 1) {
            return Err(crate::DataError::new_err(
                "selected column does not retain its original shape",
            ));
        }
        self.verify()?;
        Ok(column)
    }

    pub(super) fn polars_column(&self, index: usize, role: Role) -> PyResult<Column> {
        let Self::Polars {
            object,
            shape,
            names,
        } = self
        else {
            return Err(PyTypeError::new_err(
                "expected the retained eager Polars source",
            ));
        };
        self.verify()?;
        let series = object.call_method1("get_column", (&names[index],))?;
        let column = super::polars::import_series(&series, role_name(role))?;
        self.verify()?;
        if column.len() != shape.0 || column.name().as_str() != names[index] {
            return Err(crate::DataError::new_err(
                "Polars imported a different column or height",
            ));
        }
        Ok(column)
    }

    fn discover(&self) -> PyResult<Selected> {
        let mut discovered = Vec::new();
        let indices = match self {
            Self::Numpy { object, shape } => {
                if array::dtype(&object.getattr("dtype")?)?.is_some_and(array::Dtype::numeric) {
                    (0..shape.1).collect()
                } else {
                    Vec::new()
                }
            }
            Self::Pandas(table) => table.numeric_indices()?,
            Self::Polars { object, names, .. } => {
                let schema = object.getattr("schema")?;
                let mut result = Vec::new();
                for (index, name) in names.iter().enumerate() {
                    let dtype = schema.get_item(name)?;
                    if let Some(dtype) = super::polars::primitive_dtype(&dtype)? {
                        if dtype.is_primitive_numeric() {
                            discovered.push((index, dtype));
                            result.push(index);
                        }
                    }
                }
                result
            }
        };
        selectors::nonempty_unique(&indices)?;
        self.verify()?;
        Ok(Selected {
            indices,
            collection: true,
            discovered,
        })
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Numeric => "numeric",
        Role::Category => "hue",
        Role::Ids => "source rows",
    }
}

fn direct_container(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    Ok(super::native_sequence(value)
        || super::instance(value, "numpy", "ndarray")?
        || super::instance(value, "pandas", "Series")?
        || super::instance(value, "pandas", "DataFrame")?
        || super::instance(value, "polars", "Series")?
        || super::instance(value, "polars", "DataFrame")?)
}

pub(super) enum Data {
    Nd(Scratch),
    NdColumns(Vec<Scratch>),
    Polars(Vec<Column>),
}

pub(super) struct Direct {
    pub(super) data: Data,
    pub(super) rank: usize,
    pub(super) names: Vec<String>,
    // Call-local original key only; published edges belong to InputRoots.keys.
    pub(super) pandas_name: Option<Py<PyAny>>,
    // Only an actual independent Polars DataFrame has named collection origin.
    // Other direct representations stay local after conversion to PL columns.
    polars_frame_rows: Option<usize>,
}

impl Direct {
    pub(super) fn new(value: &Bound<'_, PyAny>, role: Role, name: &str) -> PyResult<Self> {
        let (data, rank, names) = if super::native_sequence(value) {
            let (data, rank) = Scratch::native(value, role)?;
            let names = local_names(data.shape().1, rank, name);
            (Data::Nd(data), rank, names)
        } else if super::instance(value, "numpy", "ndarray")? {
            let (data, rank) = array::numpy(value)?;
            let names = local_names(data.shape().1, rank, name);
            (Data::Nd(data), rank, names)
        } else if super::instance(value, "pandas", "Series")? {
            let data = pandas::series_values(value, role)?;
            let label = value.getattr("name")?;
            return Ok(Self {
                data: Data::Nd(data),
                rank: 1,
                names: vec![name.to_owned()],
                pandas_name: (!label.is_none()).then(|| label.unbind()),
                polars_frame_rows: None,
            });
        } else if super::instance(value, "pandas", "DataFrame")? {
            return Self::pandas_frame(value, role);
        } else if super::instance(value, "polars", "Series")? {
            let column = super::polars::import_series(value, role_name(role))?;
            let names = vec![column.name().to_string()];
            (Data::Polars(vec![column]), 1, names)
        } else if super::instance(value, "polars", "DataFrame")? {
            if role == Role::Category {
                return Err(PyTypeError::new_err(
                    "this role requires one direct variable",
                ));
            }
            let source = Source::new(value)?;
            let Source::Polars { names, shape, .. } = &source else {
                unreachable!()
            };
            let columns = (0..names.len())
                .map(|index| source.polars_column(index, role))
                .collect::<PyResult<Vec<_>>>()?;
            return Ok(Self {
                data: Data::Polars(columns),
                rank: 2,
                names: names.clone(),
                pandas_name: None,
                polars_frame_rows: Some(shape.0),
            });
        } else {
            if selectors::description(value)?.is_some() || value.is_instance_of::<PyString>() {
                return Err(PyTypeError::new_err(
                    "a column selector requires a data source",
                ));
            }
            return Err(PyTypeError::new_err(
                "expected native data, a NumPy array or a supported Series/rectangle",
            ));
        };
        if rank == 2 && role == Role::Category {
            return Err(PyTypeError::new_err(
                "this role requires a one-dimensional variable",
            ));
        }
        Ok(Self {
            data,
            rank,
            names,
            pandas_name: None,
            polars_frame_rows: None,
        })
    }

    fn pandas_frame(value: &Bound<'_, PyAny>, role: Role) -> PyResult<Self> {
        if role == Role::Category {
            return Err(PyTypeError::new_err(
                "this role requires one direct variable",
            ));
        }
        let table = pandas::Table::new(value)?;
        let mut columns = Vec::new();
        let mut names = Vec::new();
        for (index, key) in table.keys.iter().enumerate() {
            table.require(key)?;
            columns.push(table.import(index, role)?);
            names.push(key.str()?.extract()?);
        }
        table.verify()?;
        let data = if role == Role::Ids {
            Data::NdColumns(columns)
        } else {
            let matrices = columns
                .iter()
                .map(Scratch::numeric)
                .collect::<PyResult<Vec<_>>>()?;
            Data::Nd(Scratch::F64(combine(&matrices, table.shape.0)?))
        };
        Ok(Self {
            data,
            rank: 2,
            names,
            pandas_name: None,
            polars_frame_rows: None,
        })
    }

    pub(super) fn bind_name(
        &mut self,
        py: Python<'_>,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<()> {
        if let Some(name) = &self.pandas_name {
            let name = name.bind(py);
            name.hash()?;
            self.names[0] = registered_key(
                name,
                roots,
                aliases,
                |known| pandas::same_key(known, name),
                || Ok(()),
            )?;
        }
        Ok(())
    }

    fn numeric(&self) -> PyResult<Array2<Option<f64>>> {
        match &self.data {
            Data::Nd(values) => values.numeric(),
            Data::NdColumns(columns) => {
                let rows = columns.first().map_or(0, |column| column.shape().0);
                let matrices = columns
                    .iter()
                    .map(Scratch::numeric)
                    .collect::<PyResult<Vec<_>>>()?;
                combine(&matrices, rows)
            }
            Data::Polars(columns) => {
                let rows = columns.first().map_or(0, Column::len);
                let matrices = columns
                    .iter()
                    .map(|column| array::from_polars(column, Role::Numeric)?.numeric())
                    .collect::<PyResult<Vec<_>>>()?;
                combine(&matrices, rows)
            }
        }
    }

    fn columns(&self) -> Vec<Column> {
        match &self.data {
            Data::Nd(values) => values.columns(&self.names),
            Data::NdColumns(columns) => columns
                .iter()
                .zip(&self.names)
                .flat_map(|(column, name)| column.columns(std::slice::from_ref(name)))
                .collect(),
            Data::Polars(columns) => columns.clone(),
        }
    }

    fn category(&self) -> PyResult<Vec<Option<String>>> {
        match &self.data {
            Data::Nd(values) => values.category(),
            Data::Polars(columns) => array::from_polars(&columns[0], Role::Category)?.category(),
            Data::NdColumns(_) => Err(PyTypeError::new_err("hue requires one variable")),
        }
    }
}

fn local_names(width: usize, rank: usize, role: &str) -> Vec<String> {
    if rank == 1 {
        vec![role.to_owned()]
    } else {
        (1..=width).map(|index| format!("Column {index}")).collect()
    }
}

fn combine<T: Clone>(columns: &[Array2<T>], rows: usize) -> PyResult<Array2<T>> {
    let width = columns.iter().map(Array2::ncols).sum();
    if columns.iter().any(|column| column.nrows() != rows) {
        return Err(crate::DataError::new_err(
            "direct columns have different row counts",
        ));
    }
    let mut values = Vec::new();
    for row in 0..rows {
        for column in columns {
            values.extend(column.row(row).iter().cloned());
        }
    }
    Array2::from_shape_vec((rows, width), values)
        .map_err(|error| crate::DataError::new_err(error.to_string()))
}

pub(super) fn key_identity(
    table: &pandas::Table<'_>,
    index: usize,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<String> {
    registered_key(
        &table.keys[index],
        roots,
        aliases,
        |known| Ok(table.exact(known)? == Some(index)),
        || table.verify(),
    )
}

fn registered_key(
    key: &Bound<'_, PyAny>,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
    matches: impl Fn(&Bound<'_, PyAny>) -> PyResult<bool>,
    verify: impl Fn() -> PyResult<()>,
) -> PyResult<String> {
    let py = key.py();
    if roots.keys.is_none() {
        roots.keys = Some(PyDict::new(py).unbind());
    }
    let registry = roots.keys.as_ref().expect("initialized above").bind(py);
    // Pin entries before running arbitrary hash/equality/str methods.
    let known = registry
        .iter()
        .map(|(raw, key)| Ok((raw.extract::<String>()?, key)))
        .collect::<PyResult<Vec<_>>>()?;
    for (raw, key) in known {
        if matches(&key)? {
            return Ok(raw);
        }
    }
    let id = NEXT_KEY
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| crate::DataError::new_err("Python key identity space exhausted"))?;
    let raw = format!("__lichtung_python_key_{id}");
    let display = key.str()?.extract::<String>()?;
    verify()?;
    registry.set_item(&raw, key)?;
    aliases.push((raw.clone(), display));
    Ok(raw)
}

pub(super) struct NdColumns {
    shape: (usize, usize),
    columns: Vec<(usize, Scratch)>,
    identities: Vec<(usize, String)>,
}

impl NdColumns {
    pub(super) fn names(&self, indices: &[usize]) -> Vec<String> {
        indices
            .iter()
            .map(|index| {
                self.identities
                    .iter()
                    .find(|(known, _)| known == index)
                    .map_or_else(|| format!("Column {}", index + 1), |(_, name)| name.clone())
            })
            .collect()
    }

    pub(super) fn prepare(
        source: &Source<'_>,
        indices: &[usize],
        role: Role,
        roots: &mut InputRoots,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        let mut columns = Vec::new();
        let mut identities = Vec::new();
        for &index in indices {
            if columns.iter().any(|(known, _)| *known == index) {
                continue;
            }
            columns.push((index, source.numpy_column(index, role)?));
            if let Source::Pandas(table) = source {
                identities.push((index, key_identity(table, index, roots, aliases)?));
            }
        }
        source.verify()?;
        Ok(Self {
            shape: source.shape(),
            columns,
            identities,
        })
    }

    pub(super) fn with_projection<R>(
        &self,
        apply: impl FnOnce(nd::ColumnProjection<'_>) -> PyResult<R>,
    ) -> PyResult<R> {
        let views = self
            .columns
            .iter()
            .map(|(index, data)| (*index, data.view()))
            .collect::<Vec<_>>();
        let entries = views
            .iter()
            .map(|(index, data)| (*index, data.as_ref()))
            .collect::<Vec<_>>();
        let identities = self
            .identities
            .iter()
            .map(|(index, name)| (*index, name.as_str()))
            .collect::<Vec<_>>();
        let projection =
            nd::ColumnProjection::new(self.shape, &entries).map_err(super::nd_error)?;
        let projection = if identities.is_empty() {
            projection
        } else {
            projection
                .with_identities(&identities)
                .map_err(super::nd_error)?
        };
        apply(projection)
    }
}

/// Owned call-local y transport, retaining its actual selected/direct origin.
pub(super) enum Y {
    NdSelected {
        selected: Selected,
        columns: NdColumns,
    },
    NdDirect {
        values: Array2<Option<f64>>,
        rank: usize,
        names: Vec<String>,
        identity: Option<String>,
    },
    PlSelected {
        selected: Selected,
        columns: Vec<Column>,
        rows: usize,
        names: Vec<String>,
    },
    PlDirect {
        columns: Vec<Column>,
        rank: usize,
        names: Vec<String>,
        polars_frame_rows: Option<usize>,
    },
}

impl Y {
    pub(super) fn collection(&self) -> bool {
        match self {
            Self::NdSelected { selected, .. } | Self::PlSelected { selected, .. } => {
                selected.collection
            }
            Self::NdDirect { rank, .. } | Self::PlDirect { rank, .. } => *rank == 2,
        }
    }

    /// Run exactly the original pre-x binding protocol, not Expansion/preflight.
    /// A many-y input must also retain its typed columns for the real constructor;
    /// rebinding that immutable Rust scratch later incurs another payload copy,
    /// but never another foreign read or Python call.
    fn copies(&mut self) -> PyResult<usize> {
        self.copies_for("y")
    }

    pub(super) fn copies_for(&mut self, role: &'static str) -> PyResult<usize> {
        match self {
            Self::NdSelected { selected, columns } => columns.with_projection(|projection| {
                let (_, shape) = if selected.collection {
                    nd::NumericSeriesInput::bind_series(
                        selected.indices.as_slice(),
                        Some(&projection),
                        role,
                    )
                } else {
                    nd::NumericSeriesInput::bind_series(
                        selected.indices[0],
                        Some(&projection),
                        role,
                    )
                }
                .map_err(super::nd_error)?;
                Ok(shape.map_or(1, |(_, columns, _)| columns.len()))
            }),
            Self::NdDirect { values, rank, .. } => {
                let (_, shape) = if *rank == 1 {
                    nd::NumericSeriesInput::bind_series(array::column_values(values), None, role)
                } else {
                    nd::NumericSeriesInput::bind_series(values.view(), None, role)
                }
                .map_err(super::nd_error)?;
                Ok(shape.map_or(1, |(_, columns, _)| columns.len()))
            }
            Self::PlSelected {
                selected,
                columns,
                rows,
                names,
            } => {
                let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                let projection =
                    pl::ColumnProjection::new(*rows, &names, columns).map_err(super::pl_error)?;
                let (_, shape) = if selected.collection {
                    pl::NumericSeriesInput::bind_series(&projection, None, role)
                } else {
                    pl::NumericSeriesInput::bind_series(&columns[0], None, role)
                }
                .map_err(super::pl_error)?;
                own_numeric_columns(columns)?;
                Ok(shape.map_or(1, |(_, columns, _)| columns.len()))
            }
            Self::PlDirect {
                columns,
                rank,
                names,
                polars_frame_rows,
            } => {
                let (_, shape) = if *rank == 1 {
                    pl::NumericSeriesInput::bind_series(&columns[0], None, role)
                } else {
                    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                    let projection = pl_direct_projection(*polars_frame_rows, &names, columns)?;
                    pl::NumericSeriesInput::bind_series(&projection, None, role)
                }
                .map_err(super::pl_error)?;
                own_numeric_columns(columns)?;
                Ok(shape.map_or(1, |(_, columns, _)| columns.len()))
            }
        }
    }

    fn construct(&self, x: Option<NumericValues>) -> PyResult<ScatterConfig> {
        match self {
            Self::NdSelected { selected, columns } => columns.with_projection(|projection| {
                let adapter = if let Some(x) = x {
                    if selected.collection {
                        nd::Scatter::from_data(projection, x, selected.indices.as_slice())
                    } else {
                        nd::Scatter::from_data(projection, x, selected.indices[0])
                    }
                } else {
                    nd::Scatter::indexed_from_data(projection, selected.indices.as_slice())
                }
                .map_err(super::nd_error)?;
                let (native, metadata) = adapter.into_parts();
                Ok(Config {
                    native,
                    origin: Origin::Nd(metadata),
                })
            }),
            Self::NdDirect {
                values,
                rank,
                names,
                identity,
            } => nd_direct_construct(values, *rank, names, identity.as_deref(), x),
            Self::PlSelected {
                selected,
                columns,
                rows,
                names,
            } => {
                let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                let projection =
                    pl::ColumnProjection::new(*rows, &names, columns).map_err(super::pl_error)?;
                let adapter = if let Some(x) = x {
                    if selected.collection {
                        pl::Scatter::new(x, &projection)
                    } else {
                        pl::Scatter::new(x, &columns[0])
                    }
                } else {
                    pl::Scatter::indexed(&projection)
                }
                .map_err(super::pl_error)?;
                let (native, metadata) = adapter.into_parts();
                Ok(Config {
                    native,
                    origin: Origin::Polars(metadata),
                })
            }
            Self::PlDirect {
                columns,
                rank,
                names,
                polars_frame_rows,
            } => {
                let adapter = if let Some(x) = x {
                    if *rank == 1 {
                        pl::Scatter::new(x, &columns[0])
                    } else {
                        let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                        let projection = pl_direct_projection(*polars_frame_rows, &names, columns)?;
                        pl::Scatter::new(x, &projection)
                    }
                } else {
                    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                    let projection = pl_direct_projection(*polars_frame_rows, &names, columns)?;
                    pl::Scatter::indexed(&projection)
                }
                .map_err(super::pl_error)?;
                let (native, metadata) = adapter.into_parts();
                Ok(Config {
                    native,
                    origin: Origin::Polars(metadata),
                })
            }
        }
    }

    pub(super) fn numeric_matrix(&self) -> PyResult<Array2<Option<f64>>> {
        match self {
            Self::NdDirect { values, .. } => Ok(values.clone()),
            Self::NdSelected { selected, columns } => {
                let values = selected
                    .indices
                    .iter()
                    .map(|index| {
                        columns
                            .columns
                            .iter()
                            .find(|(known, _)| known == index)
                            .expect("selected column was prepared")
                            .1
                            .numeric()
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                combine(&values, columns.shape.0)
            }
            Self::PlSelected { columns, rows, .. } => {
                let values = columns
                    .iter()
                    .map(|column| array::from_polars(column, Role::Numeric)?.numeric())
                    .collect::<PyResult<Vec<_>>>()?;
                combine(&values, *rows)
            }
            Self::PlDirect { columns, .. } => {
                let rows = columns.first().map_or(0, Column::len);
                let values = columns
                    .iter()
                    .map(|column| array::from_polars(column, Role::Numeric)?.numeric())
                    .collect::<PyResult<Vec<_>>>()?;
                combine(&values, rows)
            }
        }
    }
}

pub(super) fn with_nd_identity<R>(
    values: &dyn nd::ArraySource,
    name: &str,
    apply: impl FnOnce(nd::ColumnProjection<'_>) -> PyResult<R>,
) -> PyResult<R> {
    let entries = [(0, values)];
    let identities = [(0, name)];
    let projection = nd::ColumnProjection::new(values.shape(), &entries)
        .map_err(super::nd_error)?
        .with_identities(&identities)
        .map_err(super::nd_error)?;
    apply(projection)
}

fn nd_direct_construct(
    values: &Array2<Option<f64>>,
    rank: usize,
    names: &[String],
    identity: Option<&str>,
    x: Option<NumericValues>,
) -> PyResult<ScatterConfig> {
    if let Some(identity) = identity {
        return with_nd_identity(&values.view(), identity, |projection| {
            let x = x.ok_or_else(|| {
                PyTypeError::new_err("row-position mode requires a column collection")
            })?;
            let adapter = nd::Scatter::from_data(projection, x, 0usize).map_err(super::nd_error)?;
            let (native, metadata) = adapter.into_parts();
            Ok(Config {
                native,
                origin: Origin::Nd(metadata),
            })
        });
    }
    let adapter = if let Some(x) = x {
        if rank == 1 {
            nd::Scatter::new(x, array::column_values(values))
        } else {
            nd::Scatter::new(x, values.view())
        }
    } else {
        nd::Scatter::indexed(values.view())
    }
    .map_err(super::nd_error)?;
    let adapter = if rank == 2 {
        adapter
            .column_labels(names.to_vec())
            .map_err(super::nd_error)?
    } else {
        adapter
    };
    let (native, metadata) = adapter.into_parts();
    Ok(Config {
        native,
        origin: Origin::Nd(metadata),
    })
}

pub(super) fn pl_direct_projection<'columns>(
    polars_frame_rows: Option<usize>,
    names: &'columns [&'columns str],
    columns: &'columns [Column],
) -> PyResult<pl::ColumnProjection<'columns>> {
    match polars_frame_rows {
        Some(rows) => pl::ColumnProjection::new(rows, names, columns),
        None => pl::ColumnProjection::from_direct(names, columns),
    }
    .map_err(super::pl_error)
}

fn own_numeric_columns(columns: &mut Vec<Column>) -> PyResult<()> {
    // Numeric role admission already ran in the PL protocol above. Rehome its
    // ordinary f64 normalization/validity into Rust-owned buffers before x can
    // execute Python. An imported Arrow buffer alone is not a value snapshot.
    let owned = columns
        .iter()
        .map(|column| {
            let values = array::from_polars(column, Role::Numeric)?.numeric()?;
            let scratch = Scratch::F64(values);
            let names = [column.name().to_string()];
            Ok(scratch.columns(&names).remove(0))
        })
        .collect::<PyResult<Vec<_>>>()?;
    *columns = owned;
    Ok(())
}

fn prepare_y(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    discovery: bool,
    polars: bool,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<Y> {
    prepare_series(source, value, discovery, polars, roots, aliases, "y")
}

pub(super) fn prepare_series(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    discovery: bool,
    polars: bool,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
    role: &'static str,
) -> PyResult<Y> {
    let selected = if discovery {
        Some(source.expect("discovery requires data").discover()?)
    } else {
        source
            .map(|source| source.resolve(value))
            .transpose()?
            .flatten()
    };
    if let Some(selected) = selected {
        let source = source.expect("a selection was resolved against data");
        return selected_series(source, selected, discovery, roots, aliases);
    }
    let mut direct = Direct::new(value, Role::Numeric, role)?;
    direct.bind_name(value.py(), roots, aliases)?;
    if polars {
        // Namespace metadata only; unrelated main-source values stay unbound.
        let mut names = match source {
            Some(Source::Polars { names, .. }) => names.clone(),
            _ => Vec::new(),
        };
        if direct.polars_frame_rows.is_some() {
            for name in &direct.names {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        Ok(Y::PlDirect {
            columns: direct.columns(),
            rank: direct.rank,
            names,
            polars_frame_rows: direct.polars_frame_rows,
        })
    } else {
        if direct.pandas_name.is_none() && direct.rank == 1 && direct.names[0] != role {
            aliases.push((role.to_owned(), direct.names[0].clone()));
        }
        Ok(Y::NdDirect {
            values: direct.numeric()?,
            rank: direct.rank,
            identity: direct.pandas_name.as_ref().map(|_| direct.names[0].clone()),
            names: direct.names,
        })
    }
}

fn selected_series(
    source: &Source<'_>,
    selected: Selected,
    discovery: bool,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<Y> {
    if let Source::Polars { names, shape, .. } = source {
        let columns = selected
            .indices
            .iter()
            .map(|index| {
                let column = source.polars_column(*index, Role::Numeric)?;
                if selected
                    .discovered
                    .iter()
                    .any(|(candidate, dtype)| *candidate == *index && dtype != column.dtype())
                {
                    return Err(crate::DataError::new_err(
                        "Polars discovery logical dtype changed during import",
                    ));
                }
                Ok(column)
            })
            .collect::<PyResult<Vec<_>>>()?;
        if discovery {
            let frame = DataFrame::new(shape.0, columns.clone())
                .map_err(|error| crate::DataError::new_err(error.to_string()))?;
            let expected = columns
                .iter()
                .map(|column| column.name().as_str())
                .collect::<Vec<_>>();
            if pl::ColumnProjection::numeric_names(&frame) != expected {
                return Err(crate::DataError::new_err(
                    "Polars discovery dtype changed during import",
                ));
            }
        }
        return Ok(Y::PlSelected {
            selected,
            columns,
            rows: shape.0,
            names: names.clone(),
        });
    }
    let columns = NdColumns::prepare(source, &selected.indices, Role::Numeric, roots, aliases)?;
    Ok(Y::NdSelected { selected, columns })
}

pub(super) fn fresh_series(
    roots: &mut InputRoots,
    value: &Bound<'_, PyAny>,
    polars: bool,
    aliases: &mut Vec<(String, String)>,
    role: &'static str,
) -> PyResult<Y> {
    if let Some((source, selected)) = fresh(roots, value)? {
        selected_series(&source, selected, false, roots, aliases)
    } else {
        prepare_series(None, value, false, polars, roots, aliases, role)
    }
}

fn prepare_x(
    source: Option<&Source<'_>>,
    value: &Bound<'_, PyAny>,
    polars: bool,
    copies: usize,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<NumericValues> {
    if let Some(selected) = source
        .map(|source| source.resolve(value))
        .transpose()?
        .flatten()
    {
        if selected.collection {
            return Err(PyTypeError::new_err(
                "x requires one variable, not a collection",
            ));
        }
        let source = source.expect("a selection was resolved against data");
        if matches!(source, Source::Polars { .. }) {
            let column = source.polars_column(selected.indices[0], Role::Numeric)?;
            return pl::NumericInput::bind_numeric_repeated(&column, None, "x", copies)
                .map_err(super::pl_error);
        }
        let columns = NdColumns::prepare(source, &selected.indices, Role::Numeric, roots, aliases)?;
        return columns.with_projection(|projection| {
            nd::NumericInput::bind_numeric_repeated(
                selected.indices[0],
                Some(&projection),
                "x",
                copies,
            )
            .map_err(super::nd_error)
        });
    }
    let mut direct = Direct::new(value, Role::Numeric, "x")?;
    if direct.rank != 1 {
        return Err(PyTypeError::new_err(
            "x requires a one-dimensional variable",
        ));
    }
    direct.bind_name(value.py(), roots, aliases)?;
    if polars {
        let columns = direct.columns();
        pl::NumericInput::bind_numeric_repeated(&columns[0], None, "x", copies)
            .map_err(super::pl_error)
    } else {
        if direct.pandas_name.is_some() {
            let values = direct.numeric()?;
            return with_nd_identity(&values.view(), &direct.names[0], |projection| {
                nd::NumericInput::bind_numeric_repeated(0usize, Some(&projection), "x", copies)
                    .map_err(super::nd_error)
            });
        }
        if direct.names[0] != "x" {
            aliases.push(("x".to_owned(), direct.names[0].clone()));
        }
        let values = direct.numeric()?;
        nd::NumericInput::bind_numeric_repeated(array::column_values(&values), None, "x", copies)
            .map_err(super::nd_error)
    }
}

pub(super) fn defaults<K: kind::Marker>(
    config: Config<K>,
    aliases: Vec<(String, String)>,
) -> Config<K> {
    if aliases.is_empty() {
        return config;
    }
    let mut labels = config.native.get_labels().clone();
    for (raw, display) in aliases {
        let authored = labels
            .get_aliases()
            .is_some_and(|mut aliases| aliases.any(|(key, _)| key == raw));
        if !authored {
            labels = labels.alias(raw, display);
        }
    }
    config.labels(labels)
}

pub(super) fn flat_native(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    if !super::native_sequence(value) {
        return Ok(false);
    }
    Ok(!super::entries(value, "input")?
        .first()
        .is_some_and(super::native_sequence))
}

pub(crate) fn scatter(
    data: Option<&Bound<'_, PyAny>>,
    x: &Bound<'_, PyAny>,
    y: &Bound<'_, PyAny>,
    hue: Option<&Bound<'_, PyAny>>,
    by_row_position: bool,
    numeric_only: bool,
    unset: &Bound<'_, PyAny>,
) -> PyResult<Prepared<kind::Scatter>> {
    // Presence and scientific call form are decided before any source/value work.
    let data = data.filter(|data| !data.is_none());
    if by_row_position {
        if !x.is(unset) {
            return Err(PyTypeError::new_err("by_row_position excludes explicit x"));
        }
    } else if x.is(unset) {
        return Err(PyTypeError::new_err("ordinary scatter requires x"));
    }
    if numeric_only {
        if !by_row_position || data.is_none() || !y.is(unset) {
            return Err(PyTypeError::new_err(
                "numeric_only requires data, by_row_position and omitted y",
            ));
        }
    } else if y.is(unset) {
        return Err(PyTypeError::new_err("scatter requires y"));
    }
    if data.is_none() && !by_row_position && flat_native(x)? && flat_native(y)? {
        // P1's exact extraction order and delayed alignment remain unchanged.
        let x = super::numeric(x, "x")?;
        let y = super::numeric(y, "y")?;
        let mut roots = InputRoots::default();
        let mut aliases = Vec::new();
        let hue = hue
            .filter(|hue| !hue.is_none())
            .map(|value| core_category(value, &mut roots, &mut aliases))
            .transpose()?;
        let mut native = Plot::scatter(x, y);
        if let Some(hue) = hue {
            native = native.hue(hue);
        }
        return Ok(Prepared {
            config: defaults(
                Config {
                    native,
                    origin: Origin::Core,
                },
                aliases,
            ),
            roots,
        });
    }
    let source = data.map(Source::new).transpose()?;
    let mut roots = InputRoots {
        source: data.map(|data| data.clone().unbind()),
        keys: None,
    };
    // This uses real container types only, never x key/dtype/value processing.
    let polars = matches!(source, Some(Source::Polars { .. }))
        || (source.is_none()
            && (super::instance(y, "polars", "Series")?
                || super::instance(y, "polars", "DataFrame")?
                || super::instance(x, "polars", "Series")?
                || super::instance(x, "polars", "DataFrame")?));
    let mut aliases = Vec::new();
    let mut y = prepare_y(
        source.as_ref(),
        y,
        numeric_only,
        polars,
        &mut roots,
        &mut aliases,
    )?;
    if by_row_position && !y.collection() {
        return Err(PyTypeError::new_err(
            "by_row_position requires a column collection or direct rectangle",
        ));
    }
    // Original authority: y bind -> x bind/repeat -> length -> Expansion.
    // The native protocols, not a Python row-expansion loop, produce owned x.
    let copies = y.copies()?;
    let x = if by_row_position {
        None
    } else {
        Some(prepare_x(
            source.as_ref(),
            x,
            polars,
            copies,
            &mut roots,
            &mut aliases,
        )?)
    };
    let config = y.construct(x)?;
    if let Some(source) = &source {
        source.verify()?;
    }
    let mut prepared = Prepared {
        config: defaults(config, aliases),
        roots,
    };
    if let Some(value) = hue.filter(|value| !value.is_none()) {
        prepared = self::hue(&prepared.config, &prepared.roots, value)?;
    }
    Ok(prepared)
}

fn core_category(
    value: &Bound<'_, PyAny>,
    roots: &mut InputRoots,
    aliases: &mut Vec<(String, String)>,
) -> PyResult<CategoryValues> {
    if super::native_sequence(value) {
        return super::category(value, "hue");
    }
    let mut direct = Direct::new(value, Role::Category, "hue")?;
    direct.bind_name(value.py(), roots, aliases)?;
    if let Data::Polars(columns) = &direct.data {
        return pl::CategoryInput::bind_category(&columns[0], None, "hue").map_err(super::pl_error);
    }
    let values = direct.category()?;
    CategoryValues::nullable(&direct.names[0], values).map_err(crate::native_error)
}

pub(super) fn fresh<'py>(
    roots: &InputRoots,
    value: &Bound<'py, PyAny>,
) -> PyResult<Option<(Source<'py>, Selected)>> {
    if direct_container(value)? {
        return Ok(None);
    }
    let source = roots
        .source
        .as_ref()
        .ok_or_else(|| PyTypeError::new_err("a column selector requires a retained data source"))?;
    let source = Source::new(source.bind(value.py()))?;
    source
        .resolve(value)?
        .map(|selected| (source, selected))
        .map_or_else(
            || {
                Err(PyTypeError::new_err(
                    "expected a direct variable or an exact column selector",
                ))
            },
            |selection| Ok(Some(selection)),
        )
}

pub(crate) fn hue<K: kind::Hue>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    let mut roots = roots.stage(value.py())?;
    let selected = fresh(&roots, value)?;
    let mut aliases = Vec::new();
    let config = match (&config.origin, selected) {
        (Origin::Core, _) => {
            let values = core_category(value, &mut roots, &mut aliases)?;
            config.clone().map_native(|native| native.hue(values))
        }
        (Origin::Nd(metadata), Some((source, selected))) => {
            if selected.collection {
                return Err(PyTypeError::new_err(
                    "hue requires one variable, not a collection",
                ));
            }
            let columns = NdColumns::prepare(
                &source,
                &selected.indices,
                Role::Category,
                &mut roots,
                &mut aliases,
            )?;
            columns.with_projection(|projection| {
                let adapter = nd::PlotAdapter::from_parts_with_data(
                    config.native.clone(),
                    metadata.clone(),
                    projection,
                )
                .map_err(super::nd_error)?
                .hue(selected.indices[0])
                .map_err(super::nd_error)?;
                let (native, metadata) = adapter.into_parts();
                Ok(Config {
                    native,
                    origin: Origin::Nd(metadata),
                })
            })?
        }
        (Origin::Polars(metadata), Some((source, selected))) => {
            if selected.collection {
                return Err(PyTypeError::new_err(
                    "hue requires one variable, not a collection",
                ));
            }
            let column = source.polars_column(selected.indices[0], Role::Category)?;
            let adapter = pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                .hue(&column)
                .map_err(super::pl_error)?;
            let (native, metadata) = adapter.into_parts();
            Config {
                native,
                origin: Origin::Polars(metadata),
            }
        }
        (Origin::Nd(metadata), None) => {
            let mut direct = Direct::new(value, Role::Category, "hue")?;
            direct.bind_name(value.py(), &mut roots, &mut aliases)?;
            nd_direct_category(config.native.clone(), metadata.clone(), &direct)?
        }
        (Origin::Polars(metadata), None) => {
            let mut direct = Direct::new(value, Role::Category, "hue")?;
            direct.bind_name(value.py(), &mut roots, &mut aliases)?;
            let columns = direct.columns();
            let adapter = pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                .hue(&columns[0])
                .map_err(super::pl_error)?;
            let (native, metadata) = adapter.into_parts();
            Config {
                native,
                origin: Origin::Polars(metadata),
            }
        }
    };
    Ok(Prepared {
        config: defaults(config, aliases),
        roots,
    })
}

fn nd_direct_category<K: kind::Hue>(
    native: Plot<K>,
    metadata: nd::BindingMetadata<K>,
    direct: &Direct,
) -> PyResult<Config<K>> {
    let values = direct.category()?;
    let values = Array2::from_shape_vec((values.len(), 1), values)
        .map_err(|error| crate::DataError::new_err(error.to_string()))?;
    let view = values.view();
    let entries: [(usize, &dyn nd::ArraySource); 1] = [(0, &view)];
    let identities = [(0, direct.names[0].as_str())];
    // A named direct Series carries its own field metadata, but installs no
    // parent lookup source. Repetition/alignment still run in the original ND pair.
    let projection = nd::ColumnProjection::new(values.dim(), &entries)
        .map_err(super::nd_error)?
        .with_identities(&identities)
        .map_err(super::nd_error)?;
    let adapter = nd::PlotAdapter::from_parts_with_data(native, metadata, projection)
        .map_err(super::nd_error)?
        .hue(0usize)
        .map_err(super::nd_error)?;
    let (native, metadata) = adapter.into_parts();
    Ok(Config {
        native,
        origin: Origin::Nd(metadata),
    })
}

fn id_values(data: &Scratch) -> PyResult<Array2<u64>> {
    let source = data.view();
    let (rows, width) = data.shape();
    let columns = (0..width)
        .map(|index| source.id_column(index, "source rows"))
        .collect::<Result<Vec<_>, _>>()
        .map_err(super::nd_error)?;
    let mut values = Vec::new();
    for row in 0..rows {
        for column in &columns {
            values.push(
                u64::try_from(column[row])
                    .map_err(|_| crate::DataError::new_err("source-row ID exceeds UInt64"))?,
            );
        }
    }
    Array2::from_shape_vec((rows, width), values)
        .map_err(|error| crate::DataError::new_err(error.to_string()))
}

pub(crate) fn source_rows<K: kind::SourceRows>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    rows(config, roots, value, false)
}

pub(crate) fn source_rows_cells<K: kind::SourceRows>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    rows(config, roots, value, true)
}

fn rows<K: kind::SourceRows>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
    cells: bool,
) -> PyResult<Prepared<K>> {
    let mut roots = roots.stage(value.py())?;
    let selected = fresh(&roots, value)?;
    let mut aliases = Vec::new();
    let config = match (&config.origin, selected) {
        (Origin::Nd(metadata), Some((source, selected))) => {
            let columns = NdColumns::prepare(
                &source,
                &selected.indices,
                Role::Ids,
                &mut roots,
                &mut aliases,
            )?;
            columns.with_projection(|projection| {
                let adapter = nd::PlotAdapter::from_parts_with_data(
                    config.native.clone(),
                    metadata.clone(),
                    projection,
                )
                .map_err(super::nd_error)?;
                let adapter = if cells && selected.collection {
                    adapter.source_rows_cells(selected.indices.as_slice())
                } else if cells {
                    adapter.source_rows_cells(selected.indices[0])
                } else if selected.collection {
                    adapter.source_rows(selected.indices.as_slice())
                } else {
                    adapter.source_rows(selected.indices[0])
                }
                .map_err(super::nd_error)?;
                let (native, metadata) = adapter.into_parts();
                Ok(Config {
                    native,
                    origin: Origin::Nd(metadata),
                })
            })?
        }
        (Origin::Polars(metadata), Some((source, selected))) => {
            let Source::Polars { names, shape, .. } = &source else {
                unreachable!()
            };
            let columns = selected
                .indices
                .iter()
                .map(|index| source.polars_column(*index, Role::Ids))
                .collect::<PyResult<Vec<_>>>()?;
            let adapter = pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone());
            let adapter = if selected.collection {
                let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                let projection = pl::ColumnProjection::new(shape.0, &names, &columns)
                    .map_err(super::pl_error)?;
                if cells {
                    adapter.source_rows_cells(&projection)
                } else {
                    adapter.source_rows(&projection)
                }
                .map_err(super::pl_error)?
            } else {
                if cells {
                    adapter.source_rows_cells(&columns[0])
                } else {
                    adapter.source_rows(&columns[0])
                }
                .map_err(super::pl_error)?
            };
            let (native, metadata) = adapter.into_parts();
            Config {
                native,
                origin: Origin::Polars(metadata),
            }
        }
        (_, None) => attach_ids(
            config,
            &Direct::new(value, Role::Ids, "source rows")?,
            cells,
        )?,
        (Origin::Core, Some(_)) => {
            return Err(PyTypeError::new_err(
                "this core-origin plot has no lookup source",
            ));
        }
    };
    Ok(Prepared {
        config: defaults(config, aliases),
        roots,
    })
}

fn attach_ids<K: kind::SourceRows>(
    config: &Config<K>,
    direct: &Direct,
    cells: bool,
) -> PyResult<Config<K>> {
    if let Origin::Polars(metadata) = &config.origin {
        let columns = direct.columns();
        let adapter = pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone());
        let adapter = if cells && direct.rank == 1 {
            adapter.source_rows_cells(&columns[0])
        } else if direct.rank == 1 {
            adapter.source_rows(&columns[0])
        } else {
            let columns = columns.iter().collect::<Vec<_>>();
            if cells {
                adapter.source_rows_cells(columns.as_slice())
            } else {
                adapter.source_rows(columns.as_slice())
            }
        }
        .map_err(super::pl_error)?;
        let (native, metadata) = adapter.into_parts();
        return Ok(Config {
            native,
            origin: Origin::Polars(metadata),
        });
    }
    match &config.origin {
        Origin::Core => {
            if direct.rank != 1 {
                return Err(PyTypeError::new_err(
                    "source rows requires one variable for this plot",
                ));
            }
            if let Data::Polars(columns) = &direct.data {
                let rows = pl::SourceRowsInput::bind_source_rows(&columns[0], None, "source rows")
                    .map_err(super::pl_error)?;
                return Ok(config.clone().map_native(|native| native.source_rows(rows)));
            }
            let values = match &direct.data {
                Data::Nd(values) => id_values(values)?,
                Data::NdColumns(_) => unreachable!("a direct rectangle has rank two"),
                Data::Polars(_) => unreachable!("handled above"),
            };
            let rows = values
                .iter()
                .map(|value| {
                    usize::try_from(*value)
                        .map_err(|_| crate::DataError::new_err("source-row ID does not fit usize"))
                })
                .collect::<PyResult<Vec<_>>>()?;
            let rows = SourceRows::new(rows).map_err(crate::native_error)?;
            Ok(config.clone().map_native(|native| native.source_rows(rows)))
        }
        Origin::Nd(metadata) => match &direct.data {
            Data::Nd(values) => nd_direct_ids(
                config.native.clone(),
                metadata.clone(),
                values,
                direct.rank,
                cells,
            ),
            Data::NdColumns(columns) => nd_column_ids(
                config.native.clone(),
                metadata.clone(),
                columns,
                direct.rank,
                cells,
            ),
            Data::Polars(columns) => {
                let values = columns
                    .iter()
                    .map(|column| array::from_polars(column, Role::Ids))
                    .collect::<PyResult<Vec<_>>>()?;
                nd_column_ids(
                    config.native.clone(),
                    metadata.clone(),
                    &values,
                    direct.rank,
                    cells,
                )
            }
        },
        Origin::Polars(_) => unreachable!("handled above"),
    }
}

fn nd_direct_ids<K: kind::SourceRows>(
    native: Plot<K>,
    metadata: nd::BindingMetadata<K>,
    values: &Scratch,
    rank: usize,
    cells: bool,
) -> PyResult<Config<K>> {
    macro_rules! bind {
        ($values:expr) => {{
            let adapter = nd::PlotAdapter::from_parts(native, metadata);
            let adapter = if cells && rank == 1 {
                adapter.source_rows_cells($values.column(0))
            } else if cells {
                adapter.source_rows_cells($values.view())
            } else if rank == 1 {
                adapter.source_rows($values.column(0))
            } else {
                adapter.source_rows($values.view())
            }
            .map_err(super::nd_error)?;
            let (native, metadata) = adapter.into_parts();
            Ok(Config {
                native,
                origin: Origin::Nd(metadata),
            })
        }};
    }
    // Feed typed foreign views, not native SourceRows. The original adapter runs
    // cell-shape, no-row-repeat, exact-ID and uniqueness checks in its own order.
    match values {
        Scratch::I8(values) => bind!(values),
        Scratch::I16(values) => bind!(values),
        Scratch::I32(values) => bind!(values),
        Scratch::I64(values) => bind!(values),
        Scratch::I128(values) => bind!(values),
        Scratch::U8(values) => bind!(values),
        Scratch::U16(values) => bind!(values),
        Scratch::U32(values) => bind!(values),
        Scratch::U64(values) => bind!(values),
        Scratch::U128(values) => bind!(values),
        Scratch::F32(values) => bind!(values),
        Scratch::F64(values) => bind!(values),
        Scratch::Bool(values) => bind!(values),
        Scratch::String(values) => bind!(values),
    }
}

fn nd_column_ids<K: kind::SourceRows>(
    native: Plot<K>,
    metadata: nd::BindingMetadata<K>,
    values: &[Scratch],
    rank: usize,
    cells: bool,
) -> PyResult<Config<K>> {
    let Some(first) = values.first() else {
        return Err(crate::DataError::new_err("select at least one ID column"));
    };
    if rank == 1 {
        return nd_direct_ids(native, metadata, first, rank, cells);
    }
    let views = values.iter().map(Scratch::view).collect::<Vec<_>>();
    let entries = views
        .iter()
        .enumerate()
        .map(|(index, view)| (index, view.as_ref()))
        .collect::<Vec<_>>();
    let projection = nd::ColumnProjection::new((first.shape().0, values.len()), &entries)
        .map_err(super::nd_error)?;
    let indices = (0..values.len()).collect::<Vec<_>>();
    let adapter = nd::PlotAdapter::from_parts_with_data(native, metadata, projection)
        .map_err(super::nd_error)?;
    let adapter = if cells {
        adapter.source_rows_cells(indices.as_slice())
    } else {
        adapter.source_rows(indices.as_slice())
    }
    .map_err(super::nd_error)?;
    let (native, metadata) = adapter.into_parts();
    Ok(Config {
        native,
        origin: Origin::Nd(metadata),
    })
}
