//! Capability-gated input edits. No function borrows a Python owner.

use lichtung_plot::{self as core, kind, Plot};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use ndarray::Array2;
use polars_core::prelude::{Column, DataType};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict};

use super::array::{self, Role, Scratch};
use super::binding::{self, Data, Direct};
use super::columns::Scalar;
use super::families::{self, from_nd, from_pl};
use super::{Config, InputRoots, Origin, Prepared};

pub(super) fn one_category<K: kind::Marker>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
    role: &'static str,
    native: impl FnOnce(Plot<K>, &Scalar) -> PyResult<Plot<K>>,
    ndarray: impl for<'s> FnOnce(
        nd::PlotAdapter<'s, K>,
        usize,
    ) -> Result<nd::PlotAdapter<'s, K>, nd::AdapterError>,
    polars: impl FnOnce(
        pl::PlotAdapter<'static, K>,
        &Column,
    ) -> Result<pl::PlotAdapter<'static, K>, pl::AdapterError>,
) -> PyResult<Prepared<K>> {
    let mut roots = roots.stage(value.py())?;
    let mut aliases = Vec::new();
    let values = Scalar::fresh(&mut roots, value, Role::Category, role, &mut aliases)?;
    let config = match &config.origin {
        Origin::Core => Config {
            native: native(config.native.clone(), &values)?,
            origin: Origin::Core,
        },
        Origin::Nd(metadata) => values.with_nd(|projection| {
            let adapter = nd::PlotAdapter::from_parts_with_data(
                config.native.clone(),
                metadata.clone(),
                projection,
            )
            .map_err(super::nd_error)?;
            Ok(from_nd(ndarray(adapter, 0usize).map_err(super::nd_error)?))
        })?,
        Origin::Polars(metadata) => from_pl(
            polars(
                pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone()),
                &values.column(),
            )
            .map_err(super::pl_error)?,
        ),
    };
    Ok(families::finish(config, roots, aliases))
}

macro_rules! category_channel {
    ($function:ident, $capability:ident, $native:ident) => {
        pub(crate) fn $function<K: kind::$capability>(
            config: &Config<K>,
            roots: &InputRoots,
            value: &Bound<'_, PyAny>,
        ) -> PyResult<Prepared<K>> {
            one_category(
                config,
                roots,
                value,
                stringify!($native),
                |plot, values| Ok(plot.$native(values.category(stringify!($native))?)),
                |adapter, input| adapter.$function(input),
                |adapter, input| adapter.$function(input),
            )
        }
    };
}
category_channel!(group, Group, group);
category_channel!(series, Series, series);
category_channel!(hue_cells, Hue, hue);
category_channel!(group_cells, Group, group);
category_channel!(series_cells, Series, series);

pub(super) fn no_input<K: kind::Marker>(
    config: &Config<K>,
    roots: &InputRoots,
    py: Python<'_>,
    ndarray: impl FnOnce(
        nd::PlotAdapter<'static, K>,
    ) -> Result<nd::PlotAdapter<'static, K>, nd::AdapterError>,
    polars: impl FnOnce(
        pl::PlotAdapter<'static, K>,
    ) -> Result<pl::PlotAdapter<'static, K>, pl::AdapterError>,
    core_error: &str,
) -> PyResult<Prepared<K>> {
    let roots = roots.stage(py)?;
    let config = match &config.origin {
        Origin::Core => return Err(crate::DataError::new_err(core_error.to_owned())),
        Origin::Nd(metadata) => from_nd(
            ndarray(nd::PlotAdapter::from_parts(
                config.native.clone(),
                metadata.clone(),
            ))
            .map_err(super::nd_error)?,
        ),
        Origin::Polars(metadata) => from_pl(
            polars(pl::PlotAdapter::from_parts(
                config.native.clone(),
                metadata.clone(),
            ))
            .map_err(super::pl_error)?,
        ),
    };
    Ok(Prepared { config, roots })
}

macro_rules! generated_channel {
    ($function:ident, $capability:ident) => {
        pub(crate) fn $function<K: kind::$capability>(
            config: &Config<K>,
            roots: &InputRoots,
            py: Python<'_>,
        ) -> PyResult<Prepared<K>> {
            no_input(
                config,
                roots,
                py,
                |adapter| adapter.$function(),
                |adapter| adapter.$function(),
                "this plot has no generated wide-column dimension",
            )
        }
    };
}
generated_channel!(hue_columns, Hue);
generated_channel!(group_columns, Group);
generated_channel!(series_columns, Series);

pub(crate) fn column_labels<K: kind::Marker>(
    config: &Config<K>,
    roots: &InputRoots,
    labels: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    let py = labels.py();
    let labels = strings(labels, "column labels")?;
    no_input(
        config,
        roots,
        py,
        |adapter| adapter.column_labels(labels.clone()),
        |adapter| adapter.column_labels(labels.clone()),
        "no generated column dimension",
    )
}

pub(super) fn strings(value: &Bound<'_, PyAny>, role: &str) -> PyResult<Vec<String>> {
    super::entries(value, role)?
        .iter()
        .map(pyo3::types::PyAnyMethods::extract::<String>)
        .collect()
}

macro_rules! heatmap_labels {
    ($function:ident) => {
        pub(crate) fn $function(
            config: &Config<kind::Heatmap>,
            roots: &InputRoots,
            labels: &Bound<'_, PyAny>,
        ) -> PyResult<Prepared<kind::Heatmap>> {
            let py = labels.py();
            let labels = strings(labels, stringify!($function))?;
            no_input(
                config,
                roots,
                py,
                |adapter| adapter.$function(labels.clone()),
                |adapter| adapter.$function(labels.clone()),
                "axis metadata requires a rectangular heatmap",
            )
        }
    };
}
heatmap_labels!(x_labels);
heatmap_labels!(y_labels);

pub(crate) fn row_labels(
    config: &Config<kind::Heatmap>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<kind::Heatmap>> {
    one_category(
        config,
        roots,
        value,
        "row labels",
        |_, values| {
            let _ = values.category("row labels")?;
            Err(crate::DataError::new_err(
                "axis metadata requires a rectangular heatmap",
            ))
        },
        |adapter, input| adapter.row_labels(input),
        |adapter, input| adapter.row_labels(input),
    )
}

/// Strict Boolean preparation without extending the frozen importer's Role enum.
/// Native lists get the one explicit Boolean role; typed arrays retain their dtype.
pub(super) struct Booleans {
    values: Scratch,
    rank: usize,
    name: String,
}

impl Booleans {
    #[allow(
        clippy::too_many_lines,
        reason = "Boolean scalar/grid admission preserves each four-route shape"
    )]
    pub(super) fn prepare(
        roots: &mut InputRoots,
        value: &Bound<'_, PyAny>,
        scalar: bool,
        aliases: &mut Vec<(String, String)>,
    ) -> PyResult<Self> {
        if let Some((source, selected)) = binding::fresh(roots, value)? {
            if scalar && selected.collection {
                return Err(PyTypeError::new_err(
                    "estimate status requires one Boolean variable",
                ));
            }
            let mut columns = Vec::new();
            for index in &selected.indices {
                columns.push(if matches!(&source, binding::Source::Polars { .. }) {
                    Self::polars_column(&source.polars_column(*index, Role::Numeric)?)?
                } else {
                    source.numpy_column(*index, Role::Numeric)?
                });
            }
            let name = match &source {
                binding::Source::Pandas(table) if !selected.collection => {
                    binding::key_identity(table, selected.indices[0], roots, aliases)?
                }
                binding::Source::Polars { names, .. } if !selected.collection => {
                    names[selected.indices[0]].clone()
                }
                _ => "Boolean input".to_owned(),
            };
            return Self::columns(columns, if selected.collection { 2 } else { 1 }, name);
        }
        if super::native_sequence(value) {
            let entries = super::entries(value, "Boolean input")?;
            let matrix = entries.first().is_some_and(super::native_sequence);
            if scalar && matrix {
                return Err(PyTypeError::new_err(
                    "estimate status requires one Boolean variable",
                ));
            }
            let rows = entries.len();
            let width = if matrix {
                super::entries(&entries[0], "Boolean row")?.len()
            } else {
                1
            };
            let mut values = Vec::new();
            for entry in entries {
                let cells = if matrix {
                    super::entries(&entry, "Boolean row")?
                } else {
                    vec![entry]
                };
                if cells.len() != width {
                    return Err(crate::DataError::new_err(
                        "Boolean matrix rows must have equal widths",
                    ));
                }
                for cell in cells {
                    values.push(if cell.is_none() {
                        None
                    } else if cell.is_instance_of::<PyBool>() {
                        Some(cell.extract::<bool>()?)
                    } else {
                        return Err(PyTypeError::new_err("Boolean inputs require bool or None"));
                    });
                }
            }
            let values = Array2::from_shape_vec((rows, width), values)
                .map_err(|error| crate::DataError::new_err(error.to_string()))?;
            return Ok(Self {
                values: Scratch::Bool(values),
                rank: if matrix { 2 } else { 1 },
                name: "Boolean input".to_owned(),
            });
        }
        // A direct pandas rectangle must not run the numeric rectangle coercion.
        if super::instance(value, "pandas", "DataFrame")? {
            if scalar {
                return Err(PyTypeError::new_err(
                    "estimate status requires one Boolean variable",
                ));
            }
            let source = binding::Source::new(value)?;
            let columns = (0..source.shape().1)
                .map(|index| source.numpy_column(index, Role::Numeric))
                .collect::<PyResult<Vec<_>>>()?;
            return Self::columns(columns, 2, "Boolean input".to_owned());
        }
        let mut direct = Direct::new(value, Role::Numeric, "Boolean input")?;
        if scalar && direct.rank != 1 {
            return Err(PyTypeError::new_err(
                "estimate status requires one Boolean variable",
            ));
        }
        direct.bind_name(value.py(), roots, aliases)?;
        let columns = match direct.data {
            Data::Nd(values) => vec![values],
            Data::NdColumns(values) => values,
            Data::Polars(columns) => columns
                .iter()
                .map(Self::polars_column)
                .collect::<PyResult<Vec<_>>>()?,
        };
        Self::columns(
            columns,
            direct.rank,
            direct
                .names
                .first()
                .cloned()
                .unwrap_or_else(|| "Boolean input".to_owned()),
        )
    }

    fn polars_column(column: &Column) -> PyResult<Scratch> {
        if column.dtype() == &DataType::Null {
            // The native Boolean role explicitly admits Null. This is not axis
            // inference or a numeric-to-Boolean coercion.
            Ok(Scratch::Bool(Array2::from_elem((column.len(), 1), None)))
        } else {
            array::from_polars(column, Role::Numeric)
        }
    }

    fn columns(columns: Vec<Scratch>, rank: usize, name: String) -> PyResult<Self> {
        let rows = columns.first().map_or(0, |column| column.shape().0);
        let mut boolean = Vec::new();
        for column in columns {
            let Scratch::Bool(values) = column else {
                return Err(PyTypeError::new_err(
                    "Boolean input requires a Boolean logical dtype",
                ));
            };
            if values.nrows() != rows {
                return Err(crate::DataError::new_err(
                    "Boolean columns have different row counts",
                ));
            }
            boolean.push(values);
        }
        let width = boolean.iter().map(Array2::ncols).sum();
        let mut cells = Vec::new();
        for row in 0..rows {
            for column in &boolean {
                cells.extend(column.row(row).iter().copied());
            }
        }
        let values = Array2::from_shape_vec((rows, width), cells)
            .map_err(|error| crate::DataError::new_err(error.to_string()))?;
        Ok(Self {
            values: Scratch::Bool(values),
            rank,
            name,
        })
    }

    fn bools(&self) -> &Array2<Option<bool>> {
        let Scratch::Bool(values) = &self.values else {
            unreachable!("Boolean preparation is typed")
        };
        values
    }
}

pub(crate) fn heatmap_highlight(
    config: &Config<kind::Heatmap>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<kind::Heatmap>> {
    let mut roots = roots.stage(value.py())?;
    let mut aliases = Vec::new();
    let values = Booleans::prepare(&mut roots, value, false, &mut aliases)?;
    let config = match &config.origin {
        Origin::Core => {
            if values.rank != 1 {
                return Err(crate::DataError::new_err(
                    "record heatmap highlights require one variable",
                ));
            }
            let values =
                core::BooleanValues::nullable(&values.name, values.bools().iter().copied());
            config
                .clone()
                .map_native(|native| native.heatmap_highlight(values))
        }
        Origin::Nd(metadata) => {
            let adapter = nd::Heatmap::from_parts(config.native.clone(), metadata.clone());
            let adapter = if values.rank == 1 {
                adapter.heatmap_highlight(values.bools().column(0))
            } else {
                adapter.heatmap_highlight(values.bools().view())
            }
            .map_err(super::nd_error)?;
            from_nd(adapter)
        }
        Origin::Polars(metadata) => {
            let names = (0..values.bools().ncols())
                .map(|index| format!("Boolean {index}"))
                .collect::<Vec<_>>();
            let columns = values.values.columns(&names);
            let refs = columns.iter().collect::<Vec<_>>();
            let adapter = pl::Heatmap::from_parts(config.native.clone(), metadata.clone());
            let adapter = if values.rank == 1 {
                adapter.heatmap_highlight(&columns[0])
            } else {
                adapter.heatmap_highlight(refs.as_slice())
            }
            .map_err(super::pl_error)?;
            from_pl(adapter)
        }
    };
    Ok(families::finish(config, roots, aliases))
}

pub(crate) fn heatmap_highlight_cells(
    config: &Config<kind::Heatmap>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<kind::Heatmap>> {
    // Both native methods deliberately share the same grid-shape producer.
    heatmap_highlight(config, roots, value)
}

fn status(value: &Bound<'_, PyAny>) -> PyResult<core::EstimateStatus> {
    match value.extract::<String>()?.as_str() {
        "estimated" => Ok(core::EstimateStatus::Estimated),
        "not_estimable" => Ok(core::EstimateStatus::NotEstimable),
        _ => Err(PyTypeError::new_err(
            "status must be estimated or not_estimable",
        )),
    }
}

pub(crate) fn estimate_status<K: kind::EstimateStatus>(
    config: &Config<K>,
    roots: &InputRoots,
    values: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    let roots = roots.stage(values.py())?;
    let values = super::entries(values, "estimate statuses")?
        .iter()
        .map(status)
        .collect::<PyResult<Vec<_>>>()?;
    Ok(Prepared {
        config: config
            .clone()
            .map_native(|native| native.estimate_status(values)),
        roots,
    })
}

pub(crate) fn estimate_status_boolean<K: kind::EstimateStatus>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    let mut roots = roots.stage(value.py())?;
    let mut aliases = Vec::new();
    let values = Booleans::prepare(&mut roots, value, true, &mut aliases)?;
    let config = match &config.origin {
        Origin::Core => {
            let statuses = values
                .bools()
                .iter()
                .enumerate()
                .map(|(slot, value)| match value {
                    Some(true) => Ok(core::EstimateStatus::Estimated),
                    Some(false) => Ok(core::EstimateStatus::NotEstimable),
                    None => Err(crate::DataError::new_err(format!(
                        "missing Boolean at slot {slot}"
                    ))),
                })
                .collect::<PyResult<Vec<_>>>()?;
            config
                .clone()
                .map_native(|native| native.estimate_status(statuses))
        }
        Origin::Nd(metadata) => {
            // A scalar selector has no grid shape; its dtype is still Boolean.
            let scalar = Scalar {
                values: values.values,
                name: values.name,
                named: true,
            };
            scalar.with_nd(|projection| {
                let adapter = nd::PlotAdapter::from_parts_with_data(
                    config.native.clone(),
                    metadata.clone(),
                    projection,
                )
                .map_err(super::nd_error)?
                .estimate_status_boolean(0usize)
                .map_err(super::nd_error)?;
                Ok(from_nd(adapter))
            })?
        }
        Origin::Polars(metadata) => {
            let column = values.values.columns(&[values.name]).remove(0);
            from_pl(
                pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                    .estimate_status_boolean(&column)
                    .map_err(super::pl_error)?,
            )
        }
    };
    Ok(families::finish(config, roots, aliases))
}

pub(crate) fn estimate_status_categorical<K: kind::EstimateStatus>(
    config: &Config<K>,
    roots: &InputRoots,
    value: &Bound<'_, PyAny>,
    mapping: &Bound<'_, PyAny>,
) -> PyResult<Prepared<K>> {
    let entries = if let Ok(mapping) = mapping.cast::<PyDict>() {
        mapping.iter().collect::<Vec<_>>()
    } else {
        let entries = if super::native_sequence(mapping) {
            super::entries(mapping, "status mapping")?
        } else {
            let abc = PyModule::import(mapping.py(), "collections.abc")?;
            if !mapping.is_instance(&abc.getattr("Mapping")?)? {
                return Err(PyTypeError::new_err(
                    "status mapping requires a Mapping or ordered pairs",
                ));
            }
            mapping
                .call_method0("items")?
                .try_iter()?
                .collect::<PyResult<Vec<_>>>()?
        };
        entries
            .iter()
            .map(|entry| {
                let pair = super::entries(entry, "status mapping pair")?;
                if pair.len() != 2 {
                    return Err(PyTypeError::new_err(
                        "status mapping entries need two values",
                    ));
                }
                Ok((pair[0].clone(), pair[1].clone()))
            })
            .collect::<PyResult<Vec<_>>>()?
    };
    let mapping = entries
        .iter()
        .map(|(key, value)| Ok((key.extract::<String>()?, status(value)?)))
        .collect::<PyResult<Vec<_>>>()?;
    one_category(
        config,
        roots,
        value,
        "estimate status",
        |plot, values| {
            let (_, values) = values.with_nd(|projection| {
                nd::CategoryInput::raw_category(0usize, Some(&projection), "estimate status")
                    .map_err(super::nd_error)
            })?;
            let statuses = values
                .into_iter()
                .enumerate()
                .map(|(slot, value)| {
                    let key = value.ok_or_else(|| {
                        crate::DataError::new_err(format!("missing category at slot {slot}"))
                    })?;
                    let mut matches = mapping.iter().filter(|(raw, _)| raw == &key);
                    let result = matches
                        .next()
                        .ok_or_else(|| {
                            crate::DataError::new_err(format!(
                                "unmapped category {key:?} at slot {slot}"
                            ))
                        })?
                        .1;
                    if matches.next().is_some() {
                        return Err(crate::DataError::new_err(format!(
                            "duplicate mapping for {key:?}"
                        )));
                    }
                    Ok(result)
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(plot.estimate_status(statuses))
        },
        |adapter, input| adapter.estimate_status_categorical(input, mapping.clone()),
        |adapter, input| adapter.estimate_status_categorical(input, mapping.clone()),
    )
}
