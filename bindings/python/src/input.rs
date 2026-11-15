//! Binding-time input snapshots; foreign work never borrows a Python owner.

mod array;
mod binding;
mod channels;
mod columns;
mod facets;
mod families;
mod geometry;
mod pandas;
pub(crate) mod polars;
mod selectors;
mod wide;

pub(crate) use binding::{hue, scatter, source_rows, source_rows_cells};
pub(crate) use channels::{
    column_labels, estimate_status, estimate_status_boolean, estimate_status_categorical, group,
    group_cells, group_columns, heatmap_highlight, heatmap_highlight_cells, hue_cells, hue_columns,
    row_labels, series, series_cells, series_columns, x_labels, y_labels,
};
pub(crate) use columns::bootstrap_units;
pub(crate) use facets::{
    facet_column, facet_column_cells, facet_columns_column, facet_columns_row, facet_columns_wrap,
    facet_grid, facet_grid_cells, facet_grid_column_cells, facet_grid_columns_column,
    facet_grid_columns_column_cells, facet_grid_columns_row, facet_grid_columns_row_cells,
    facet_grid_row_cells, facet_row, facet_row_cells, facet_scales, facet_wrap, facet_wrap_cells,
};
pub(crate) use families::{
    bar, beeswarm, coefficient, density, dumbbell, forest, horizontal_bar, interaction, line_path,
    marginal, point_interval, qq, regression, stacked_bar, stacked_horizontal_bar, trajectory,
    waterfall, BarArgs, BeeswarmArgs, BoxplotArgs, CoefficientArgs, DensityArgs, DumbbellArgs,
    ForestArgs, HeatmapArgs, HistogramArgs, HorizontalBarArgs, InteractionArgs, LineArgs,
    LinePathArgs, MarginalArgs, MatrixArgs, PointIntervalArgs, QqArgs, RaincloudArgs,
    RegressionArgs, StackedBarArgs, StackedHorizontalBarArgs, TrajectoryArgs, ViolinArgs,
    WaterfallArgs,
};
pub(crate) use geometry::{composite_precomputed_kde, histogram, raincloud};
pub(crate) use selectors::register;
pub(crate) use wide::{boxplot, heatmap, line, line_band, matrix, violin};

use lichtung_plot::{kind, Labels, Plot};
use lichtung_plot::{AxisTick, CategoryValues, NumericValues};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyTuple};
use pyo3::{PyTraverseError, PyVisit};

#[derive(Clone)]
pub(crate) struct Config<K: kind::Marker> {
    pub(crate) native: Plot<K>,
    pub(crate) origin: Origin<K>,
}

#[derive(Clone)]
pub(crate) enum Origin<K: kind::Marker> {
    Core,
    Nd(nd::BindingMetadata<K>),
    Polars(pl::BindingMetadata<K>),
}

impl<K: kind::Marker> Config<K> {
    pub(crate) fn map_native(self, edit: impl FnOnce(Plot<K>) -> Plot<K>) -> Self {
        Self {
            native: edit(self.native),
            origin: self.origin,
        }
    }

    pub(crate) fn labels(self, labels: Labels) -> Self {
        match self.origin {
            Origin::Core => Self {
                native: self.native.labels(labels),
                origin: Origin::Core,
            },
            Origin::Nd(metadata) => {
                let adapter = nd::PlotAdapter::from_parts(self.native, metadata).labels(labels);
                let (native, metadata) = adapter.into_parts();
                Self {
                    native,
                    origin: Origin::Nd(metadata),
                }
            }
            Origin::Polars(metadata) => {
                let adapter = pl::PlotAdapter::from_parts(self.native, metadata).labels(labels);
                let (native, metadata) = adapter.into_parts();
                Self {
                    native,
                    origin: Origin::Polars(metadata),
                }
            }
        }
    }

    pub(crate) fn is_core(&self) -> bool {
        matches!(self.origin, Origin::Core)
    }
}

/// Actual owner-local Python edges. The dict maps opaque raw IDs to original keys;
/// it contains no Labels, cached observations or native configuration.
#[derive(Default)]
pub(crate) struct InputRoots {
    source: Option<Py<PyAny>>,
    keys: Option<Py<PyDict>>,
}

impl InputRoots {
    pub(crate) fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            source: self.source.as_ref().map(|source| source.clone_ref(py)),
            keys: self.keys.as_ref().map(|keys| keys.clone_ref(py)),
        }
    }

    pub(crate) fn traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Some(source) = &self.source {
            visit.call(source)?;
        }
        if let Some(keys) = &self.keys {
            // The dict, not this Rust object, owns/traverses its key/value edges.
            visit.call(keys)?;
        }
        Ok(())
    }

    fn stage(&self, py: Python<'_>) -> PyResult<Self> {
        // This is deliberately separate from the borrow-safe INCREF-only snapshot.
        Ok(Self {
            source: self.source.as_ref().map(|source| source.clone_ref(py)),
            keys: self
                .keys
                .as_ref()
                .map(|keys| keys.bind(py).copy().map(Bound::unbind))
                .transpose()?,
        })
    }
}

pub(crate) struct Prepared<K: kind::Marker> {
    pub(crate) config: Config<K>,
    pub(crate) roots: InputRoots,
}

/// Optional containers are recognized through their already-loaded public module.
/// Native inputs do not import NumPy, pandas, Polars or PyArrow as a prerequisite.
fn instance(value: &Bound<'_, PyAny>, module: &str, class: &str) -> PyResult<bool> {
    let modules = PyModule::import(value.py(), "sys")?.getattr("modules")?;
    let Some(module) = modules.cast::<pyo3::types::PyDict>()?.get_item(module)? else {
        return Ok(false);
    };
    // Inspect the real container type, not an arbitrary object's __class__
    // property. Type-only route selection must not run x-role code before y.
    value.get_type().is_subclass(&module.getattr(class)?)
}

pub(crate) fn native_sequence(value: &Bound<'_, PyAny>) -> bool {
    value.is_instance_of::<PyList>() || value.is_instance_of::<PyTuple>()
}

fn nd_error(error: lichtung_plot_ndarray::AdapterError) -> PyErr {
    use lichtung_plot_ndarray::AdapterError;
    match error {
        AdapterError::Native(error) => crate::native_error(error),
        error @ AdapterError::UnsupportedElement { .. } => PyTypeError::new_err(error.to_string()),
        error => crate::DataError::new_err(error.to_string()),
    }
}

fn pl_error(error: lichtung_plot_polars::AdapterError) -> PyErr {
    use lichtung_plot_polars::AdapterError;
    match error {
        AdapterError::Native { source, .. } | AdapterError::Plot(source) => {
            crate::native_error(source)
        }
        error @ AdapterError::UnsupportedDtype { .. } => PyTypeError::new_err(error.to_string()),
        error => crate::DataError::new_err(error.to_string()),
    }
}

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
