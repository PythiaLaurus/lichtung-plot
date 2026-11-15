//! Constructors and edits whose single/many/grid shape is adapter-owned.

use lichtung_plot::{self as core, kind, CategoryValues, NumericValues, Plot};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use super::array::{self, Role};
use super::binding::{self, Y};
use super::columns::{self, Scalar};
use super::families::{
    self, BoxplotArgs, HeatmapArgs, LineArgs, MatrixArgs, RaincloudArgs, ViolinArgs,
};
use super::{Config, InputRoots, Origin, Prepared};

// The first role has already gone through the original row-repeat protocol.
// The actual series/collection remains ordinary, with native_slots=false.
macro_rules! wide_config {
    ($function:ident, $kind:ident, $first:ty, $indexed:ident, $indexed_from:ident) => {
        fn $function(series: &Y, first: Option<$first>) -> PyResult<Config<kind::$kind>> {
            match series {
                Y::NdSelected { selected, columns } => columns.with_projection(|projection| {
                    let adapter = if let Some(first) = first {
                        if selected.collection {
                            nd::$kind::from_data(projection, first, selected.indices.as_slice())
                        } else {
                            nd::$kind::from_data(projection, first, selected.indices[0])
                        }
                    } else {
                        nd::$kind::$indexed_from(projection, selected.indices.as_slice())
                    }
                    .map_err(super::nd_error)?;
                    Ok(families::from_nd(adapter))
                }),
                Y::NdDirect {
                    values,
                    rank,
                    names,
                    identity,
                } => {
                    if let Some(identity) = identity {
                        return binding::with_nd_identity(&values.view(), identity, |projection| {
                            let first = first.ok_or_else(|| {
                                PyTypeError::new_err(
                                    "generated-position mode requires a column collection",
                                )
                            })?;
                            Ok(families::from_nd(
                                nd::$kind::from_data(projection, first, 0usize)
                                    .map_err(super::nd_error)?,
                            ))
                        });
                    }
                    let adapter = if let Some(first) = first {
                        if *rank == 1 {
                            nd::$kind::new(first, array::column_values(values))
                        } else {
                            nd::$kind::new(first, values.view())
                        }
                    } else {
                        nd::$kind::$indexed(values.view())
                    }
                    .map_err(super::nd_error)?;
                    let adapter = if *rank == 2 {
                        adapter
                            .column_labels(names.clone())
                            .map_err(super::nd_error)?
                    } else {
                        adapter
                    };
                    Ok(families::from_nd(adapter))
                }
                Y::PlSelected {
                    selected,
                    columns,
                    rows,
                    names,
                } => {
                    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                    let projection = pl::ColumnProjection::new(*rows, &names, columns)
                        .map_err(super::pl_error)?;
                    let adapter = if let Some(first) = first {
                        if selected.collection {
                            pl::$kind::new(first, &projection)
                        } else {
                            pl::$kind::new(first, &columns[0])
                        }
                    } else {
                        pl::$kind::$indexed(&projection)
                    }
                    .map_err(super::pl_error)?;
                    Ok(families::from_pl(adapter))
                }
                Y::PlDirect {
                    columns,
                    rank,
                    names,
                    polars_frame_rows,
                } => {
                    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                    let projection =
                        binding::pl_direct_projection(*polars_frame_rows, &names, columns)?;
                    let adapter = if let Some(first) = first {
                        if *rank == 1 {
                            pl::$kind::new(first, &columns[0])
                        } else {
                            pl::$kind::new(first, &projection)
                        }
                    } else {
                        pl::$kind::$indexed(&projection)
                    }
                    .map_err(super::pl_error)?;
                    Ok(families::from_pl(adapter))
                }
            }
        }
    };
}
wide_config!(line_config, Line, NumericValues, indexed, indexed_from_data);
wide_config!(
    boxplot_config,
    Boxplot,
    CategoryValues,
    columns,
    columns_from_data
);
wide_config!(
    violin_config,
    Violin,
    CategoryValues,
    columns,
    columns_from_data
);
wide_config!(
    raincloud_config,
    Raincloud,
    CategoryValues,
    columns,
    columns_from_data
);

fn mode(
    data: Option<&Bound<'_, PyAny>>,
    first: &Bound<'_, PyAny>,
    values: &Bound<'_, PyAny>,
    generated: bool,
    discovery: bool,
    unset: &Bound<'_, PyAny>,
    first_name: &str,
) -> PyResult<()> {
    if generated {
        if families::present(first, unset) {
            return Err(PyTypeError::new_err(format!(
                "generated-position mode excludes {first_name}"
            )));
        }
    } else if !families::present(first, unset) {
        return Err(PyTypeError::new_err(format!(
            "ordinary form requires {first_name}"
        )));
    } else {
        families::required(&[(first_name, first)])?;
    }
    if discovery {
        if !generated || data.is_none_or(Bound::is_none) || families::present(values, unset) {
            return Err(PyTypeError::new_err(
                "numeric_only requires data, explicit generated-position mode and omitted variables",
            ));
        }
    } else if !families::present(values, unset) || values.is_none() {
        return Err(PyTypeError::new_err(
            "a non-None variable input is required",
        ));
    }
    Ok(())
}

pub(crate) fn line(
    data: Option<&Bound<'_, PyAny>>,
    args: LineArgs<'_, '_>,
) -> PyResult<Prepared<kind::Line>> {
    mode(
        data,
        args.x,
        args.y,
        args.by_row_position,
        args.numeric_only,
        args.unset,
        "x",
    )?;
    let (source, mut roots) = families::context(data)?;
    let polars = columns::polars_route(source.as_ref(), &[args.y, args.x])?;
    let mut aliases = Vec::new();
    let mut y = binding::prepare_series(
        source.as_ref(),
        args.y,
        args.numeric_only,
        polars,
        &mut roots,
        &mut aliases,
        "y",
    )?;
    if args.by_row_position && !y.collection() {
        return Err(PyTypeError::new_err(
            "by_row_position requires a column collection or rectangle",
        ));
    }
    let copies = y.copies_for("y")?;
    let x = if args.by_row_position {
        None
    } else {
        Some(
            Scalar::prepare(
                source.as_ref(),
                args.x,
                Role::Numeric,
                "x",
                &mut roots,
                &mut aliases,
            )?
            .numeric_repeated("x", copies)?,
        )
    };
    let config = line_config(&y, x)?;
    if let Some(source) = &source {
        source.verify()?;
    }
    let mut prepared = families::finish(config, roots, aliases);
    if let Some(value) = args.hue.filter(|value| !value.is_none()) {
        prepared = super::hue(&prepared.config, &prepared.roots, value)?;
    }
    if let Some(value) = args.group.filter(|value| !value.is_none()) {
        prepared = super::group(&prepared.config, &prepared.roots, value)?;
    }
    Ok(prepared)
}

macro_rules! distribution {
    ($function:ident, $args:ident, $kind:ident, $config:ident, $native:expr) => {
        pub(crate) fn $function(
            data: Option<&Bound<'_, PyAny>>,
            args: $args<'_, '_>,
        ) -> PyResult<Prepared<kind::$kind>> {
            mode(
                data,
                args.category,
                args.value,
                args.by_column_name,
                args.numeric_only,
                args.unset,
                "category",
            )?;
            distribution_samples(
                data,
                args.category,
                args.value,
                args.series,
                args.by_column_name,
                args.numeric_only,
                $config,
                $native,
            )
        }
    };
}

#[expect(
    clippy::too_many_arguments,
    reason = "shared sample form keeps both explicit modes"
)]
fn distribution_samples<K: kind::Series>(
    data: Option<&Bound<'_, PyAny>>,
    category: &Bound<'_, PyAny>,
    value: &Bound<'_, PyAny>,
    series: Option<&Bound<'_, PyAny>>,
    by_column_name: bool,
    numeric_only: bool,
    construct: fn(&Y, Option<CategoryValues>) -> PyResult<Config<K>>,
    native: fn(CategoryValues, NumericValues) -> Plot<K>,
) -> PyResult<Prepared<K>> {
    let core = data.is_none_or(Bound::is_none)
        && !by_column_name
        && binding::flat_native(category)?
        && binding::flat_native(value)?;
    let (source, mut roots) = families::context(data)?;
    let polars = columns::polars_route(source.as_ref(), &[value, category])?;
    let mut aliases = Vec::new();
    let mut values = binding::prepare_series(
        source.as_ref(),
        value,
        numeric_only,
        polars,
        &mut roots,
        &mut aliases,
        "value",
    )?;
    if by_column_name && !values.collection() {
        return Err(PyTypeError::new_err(
            "by_column_name requires a column collection or rectangle",
        ));
    }
    let copies = values.copies_for("value")?;
    let category = if by_column_name {
        None
    } else {
        Some(
            Scalar::prepare(
                source.as_ref(),
                category,
                Role::Category,
                "category",
                &mut roots,
                &mut aliases,
            )?
            .category_repeated("category", copies)?,
        )
    };
    let config = if core {
        let matrix = values.numeric_matrix()?;
        Config {
            native: native(
                category.expect("ordinary sample form"),
                nd::NumericInput::bind_numeric(array::column_values(&matrix), None, "value")
                    .map_err(super::nd_error)?,
            ),
            origin: Origin::Core,
        }
    } else {
        construct(&values, category)?
    };
    if let Some(source) = &source {
        source.verify()?;
    }
    let mut prepared = families::finish(config, roots, aliases);
    if let Some(value) = series.filter(|value| !value.is_none()) {
        prepared = super::series(&prepared.config, &prepared.roots, value)?;
    }
    Ok(prepared)
}

distribution!(boxplot, BoxplotArgs, Boxplot, boxplot_config, Plot::boxplot);
distribution!(violin, ViolinArgs, Violin, violin_config, Plot::violin);

pub(super) fn raincloud_samples(
    data: Option<&Bound<'_, PyAny>>,
    args: &RaincloudArgs<'_, '_>,
) -> PyResult<Prepared<kind::Raincloud>> {
    mode(
        data,
        args.category,
        args.value,
        args.by_column_name,
        args.numeric_only,
        args.unset,
        "category",
    )?;
    distribution_samples(
        data,
        args.category,
        args.value,
        args.series,
        args.by_column_name,
        args.numeric_only,
        raincloud_config,
        |category, values| Plot::raincloud(core::RaincloudInput::samples(category, values)),
    )
}

macro_rules! collection_config {
    ($function:ident, $kind:ident, $labels:ident) => {
        fn $function(series: &Y, aliases: &[(String, String)]) -> PyResult<Config<kind::$kind>> {
            match series {
                Y::NdSelected { selected, columns } => columns.with_projection(|projection| {
                    let adapter = nd::$kind::from_data(projection, selected.indices.as_slice())
                        .map_err(super::nd_error)?;
                    let names = columns
                        .names(&selected.indices)
                        .into_iter()
                        .map(|name| {
                            aliases
                                .iter()
                                .find(|(raw, _)| *raw == name)
                                .map_or(name.clone(), |(_, display)| display.clone())
                        })
                        .collect::<Vec<_>>();
                    Ok(families::from_nd(
                        adapter.$labels(names).map_err(super::nd_error)?,
                    ))
                }),
                Y::NdDirect { values, names, .. } => {
                    let adapter = nd::$kind::new(values.view())
                        .map_err(super::nd_error)?
                        .$labels(names.clone())
                        .map_err(super::nd_error)?;
                    Ok(families::from_nd(adapter))
                }
                Y::PlSelected {
                    columns,
                    rows,
                    names,
                    ..
                } => {
                    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                    let projection = pl::ColumnProjection::new(*rows, &names, columns)
                        .map_err(super::pl_error)?;
                    Ok(families::from_pl(
                        pl::$kind::new(&projection).map_err(super::pl_error)?,
                    ))
                }
                Y::PlDirect {
                    columns,
                    names,
                    polars_frame_rows,
                    ..
                } => {
                    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
                    let projection =
                        binding::pl_direct_projection(*polars_frame_rows, &names, columns)?;
                    Ok(families::from_pl(
                        pl::$kind::new(&projection).map_err(super::pl_error)?,
                    ))
                }
            }
        }
    };
}
collection_config!(heatmap_grid, Heatmap, x_labels);
collection_config!(matrix_config, Matrix, column_labels);

pub(crate) fn heatmap(
    data: Option<&Bound<'_, PyAny>>,
    args: HeatmapArgs<'_, '_>,
) -> PyResult<Prepared<kind::Heatmap>> {
    let grid = families::present(args.grid, args.unset);
    let records = [args.x, args.y, args.fill].map(|value| families::present(value, args.unset));
    if (grid && records.iter().any(|present| *present))
        || (!grid && !records.iter().all(|present| *present))
    {
        return Err(PyTypeError::new_err(
            "heatmap requires exactly grid or complete x, y, fill records",
        ));
    }
    if grid {
        families::required(&[("grid", args.grid)])?;
    } else {
        families::required(&[("x", args.x), ("y", args.y), ("fill", args.fill)])?;
    }
    let (source, mut roots) = families::context(data)?;
    let mut aliases = Vec::new();
    let polars = if grid {
        columns::polars_route(source.as_ref(), &[args.grid])?
    } else {
        columns::polars_route(source.as_ref(), &[args.x, args.y, args.fill])?
    };
    let config = if grid {
        let values = binding::prepare_series(
            source.as_ref(),
            args.grid,
            false,
            polars,
            &mut roots,
            &mut aliases,
            "grid",
        )?;
        if !values.collection() {
            return Err(PyTypeError::new_err(
                "grid requires a rectangle or ordered column collection",
            ));
        }
        heatmap_grid(&values, &aliases)?
    } else {
        let x = Scalar::prepare(
            source.as_ref(),
            args.x,
            Role::Category,
            "x",
            &mut roots,
            &mut aliases,
        )?
        .category("x")?;
        let y = Scalar::prepare(
            source.as_ref(),
            args.y,
            Role::Category,
            "y",
            &mut roots,
            &mut aliases,
        )?
        .category("y")?;
        let fill = Scalar::prepare(
            source.as_ref(),
            args.fill,
            Role::Numeric,
            "fill",
            &mut roots,
            &mut aliases,
        )?
        .numeric("fill")?;
        if data.is_none_or(Bound::is_none)
            && binding::flat_native(args.x)?
            && binding::flat_native(args.y)?
            && binding::flat_native(args.fill)?
        {
            Config {
                native: Plot::heatmap(x, y, fill),
                origin: Origin::Core,
            }
        } else if polars {
            families::from_pl(pl::Heatmap::records(x, y, fill).map_err(super::pl_error)?)
        } else {
            families::from_nd(nd::Heatmap::records(x, y, fill).map_err(super::nd_error)?)
        }
    };
    if let Some(source) = &source {
        source.verify()?;
    }
    Ok(families::finish(config, roots, aliases))
}

pub(crate) fn matrix(
    data: Option<&Bound<'_, PyAny>>,
    args: MatrixArgs<'_, '_>,
) -> PyResult<Prepared<kind::Matrix>> {
    if args.numeric_only {
        if data.is_none_or(Bound::is_none) || families::present(args.variables, args.unset) {
            return Err(PyTypeError::new_err(
                "numeric_only requires data and omitted variables",
            ));
        }
    } else if !families::present(args.variables, args.unset) || args.variables.is_none() {
        return Err(PyTypeError::new_err(
            "matrix requires variables or explicit numeric discovery",
        ));
    }
    let (source, mut roots) = families::context(data)?;
    let polars = columns::polars_route(source.as_ref(), &[args.variables])?;
    let mut aliases = Vec::new();
    let values = binding::prepare_series(
        source.as_ref(),
        args.variables,
        args.numeric_only,
        polars,
        &mut roots,
        &mut aliases,
        "variables",
    )?;
    if !values.collection() {
        return Err(PyTypeError::new_err(
            "matrix requires a rectangle or ordered column collection",
        ));
    }
    let config = matrix_config(&values, &aliases)?;
    if let Some(source) = &source {
        source.verify()?;
    }
    let mut prepared = families::finish(config, roots, aliases);
    if let Some(value) = args.hue.filter(|value| !value.is_none()) {
        prepared = super::hue(&prepared.config, &prepared.roots, value)?;
    }
    if let Some(value) = args.group.filter(|value| !value.is_none()) {
        prepared = super::group(&prepared.config, &prepared.roots, value)?;
    }
    Ok(prepared)
}

pub(crate) fn line_band(
    config: &Config<kind::Line>,
    roots: &InputRoots,
    lower: &Bound<'_, PyAny>,
    upper: &Bound<'_, PyAny>,
) -> PyResult<Prepared<kind::Line>> {
    families::required(&[("lower", lower), ("upper", upper)])?;
    let mut roots = roots.stage(lower.py())?;
    let mut aliases = Vec::new();
    let polars = matches!(config.origin, Origin::Polars(_));
    let mut lower = binding::fresh_series(&mut roots, lower, polars, &mut aliases, "lower")?;
    lower.copies_for("lower")?;
    let mut upper = binding::fresh_series(&mut roots, upper, polars, &mut aliases, "upper")?;
    upper.copies_for("upper")?;
    let lower_many = lower.collection();
    let upper_many = upper.collection();
    let lower = lower.numeric_matrix()?;
    let upper = upper.numeric_matrix()?;
    let config = match &config.origin {
        Origin::Core => {
            if lower_many || upper_many {
                return Err(crate::DataError::new_err(
                    "bounds must match the line's single-variable shape",
                ));
            }
            let lower = nd::NumericInput::bind_numeric(array::column_values(&lower), None, "lower")
                .map_err(super::nd_error)?;
            let upper = nd::NumericInput::bind_numeric(array::column_values(&upper), None, "upper")
                .map_err(super::nd_error)?;
            config
                .clone()
                .map_native(|native| native.band(core::Bounds::new(lower, upper)))
        }
        Origin::Nd(metadata) => {
            let adapter = nd::Line::from_parts(config.native.clone(), metadata.clone());
            let adapter = match (lower_many, upper_many) {
                (false, false) => {
                    adapter.band(array::column_values(&lower), array::column_values(&upper))
                }
                (true, false) => adapter.band(lower.view(), array::column_values(&upper)),
                (false, true) => adapter.band(array::column_values(&lower), upper.view()),
                (true, true) => adapter.band(lower.view(), upper.view()),
            }
            .map_err(super::nd_error)?;
            families::from_nd(adapter)
        }
        Origin::Polars(metadata) => {
            let lower_names = (0..lower.ncols())
                .map(|index| format!("lower_{index}"))
                .collect::<Vec<_>>();
            let upper_names = (0..upper.ncols())
                .map(|index| format!("upper_{index}"))
                .collect::<Vec<_>>();
            let lower = array::Scratch::F64(lower).columns(&lower_names);
            let upper = array::Scratch::F64(upper).columns(&upper_names);
            let lower_refs = lower.iter().collect::<Vec<_>>();
            let upper_refs = upper.iter().collect::<Vec<_>>();
            let adapter = pl::Line::from_parts(config.native.clone(), metadata.clone());
            let adapter = match (lower_many, upper_many) {
                (false, false) => adapter.band(&lower[0], &upper[0]),
                (true, false) => adapter.band(lower_refs.as_slice(), &upper[0]),
                (false, true) => adapter.band(&lower[0], upper_refs.as_slice()),
                (true, true) => adapter.band(lower_refs.as_slice(), upper_refs.as_slice()),
            }
            .map_err(super::pl_error)?;
            families::from_pl(adapter)
        }
    };
    Ok(families::finish(config, roots, aliases))
}
