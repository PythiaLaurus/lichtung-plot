//! Complete facet replacements and cache-preserving scales-only edits.

use lichtung_plot::{kind, FacetScales, FacetSpec};
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pl;
use pyo3::prelude::*;

use super::array::Role;
use super::channels::{no_input, one_category};
use super::columns::Scalar;
use super::families::{self, from_nd, from_pl};
use super::{Config, InputRoots, Origin, Prepared};

macro_rules! one_facet {
    ($function:ident, $native:ident) => {
        pub(crate) fn $function<K: kind::Facet>(
            config: &Config<K>,
            roots: &InputRoots,
            value: &Bound<'_, PyAny>,
        ) -> PyResult<Prepared<K>> {
            one_category(
                config,
                roots,
                value,
                stringify!($function),
                |plot, values| {
                    Ok(plot.facet(FacetSpec::$native(values.category(stringify!($function))?)))
                },
                |adapter, input| adapter.$function(input),
                |adapter, input| adapter.$function(input),
            )
        }
    };
}
one_facet!(facet_row, row);
one_facet!(facet_column, column);
one_facet!(facet_row_cells, row);
one_facet!(facet_column_cells, column);

macro_rules! wrapped_facet {
    ($function:ident) => {
        pub(crate) fn $function<K: kind::Facet>(
            config: &Config<K>,
            roots: &InputRoots,
            value: &Bound<'_, PyAny>,
            columns: usize,
        ) -> PyResult<Prepared<K>> {
            one_category(
                config,
                roots,
                value,
                "facet wrap",
                |plot, values| {
                    Ok(plot.facet(
                        FacetSpec::wrap(values.category("facet wrap")?, columns)
                            .map_err(crate::native_error)?,
                    ))
                },
                |adapter, input| adapter.$function(input, columns),
                |adapter, input| adapter.$function(input, columns),
            )
        }
    };
}
wrapped_facet!(facet_wrap);
wrapped_facet!(facet_wrap_cells);

#[derive(Clone, Copy)]
enum GridScope {
    Rows,
    RowCells,
    ColumnCells,
    Cells,
}

macro_rules! grid_entry {
    ($function:ident, $scope:ident) => {
        pub(crate) fn $function<K: kind::Facet>(
            config: &Config<K>,
            roots: &InputRoots,
            row: &Bound<'_, PyAny>,
            column: &Bound<'_, PyAny>,
        ) -> PyResult<Prepared<K>> {
            grid(config, roots, row, column, GridScope::$scope)
        }
    };
}
grid_entry!(facet_grid, Rows);
grid_entry!(facet_grid_row_cells, RowCells);
grid_entry!(facet_grid_column_cells, ColumnCells);
grid_entry!(facet_grid_cells, Cells);

fn grid<K: kind::Facet>(
    config: &Config<K>,
    roots: &InputRoots,
    row: &Bound<'_, PyAny>,
    column: &Bound<'_, PyAny>,
    scope: GridScope,
) -> PyResult<Prepared<K>> {
    families::required(&[("facet row", row), ("facet column", column)])?;
    let mut roots = roots.stage(row.py())?;
    let mut aliases = Vec::new();
    let row = Scalar::fresh(&mut roots, row, Role::Category, "facet row", &mut aliases)?;
    let column = Scalar::fresh(
        &mut roots,
        column,
        Role::Category,
        "facet column",
        &mut aliases,
    )?;
    let config = match &config.origin {
        Origin::Core => {
            let facet =
                FacetSpec::grid(row.category("facet row")?, column.category("facet column")?)
                    .map_err(crate::native_error)?;
            config.clone().map_native(|native| native.facet(facet))
        }
        Origin::Nd(metadata) => match scope {
            GridScope::Rows => {
                // Both row-scoped columns have the same actual observation
                // domain. Projection validation rejects unequal shapes; it
                // never invents a padded column or infers expanded scope.
                let row_view = row.values.view();
                let column_view = column.values.view();
                let entries = [(0, row_view.as_ref()), (1, column_view.as_ref())];
                let identities = [(0, row.name.as_str()), (1, column.name.as_str())];
                let projection = nd::ColumnProjection::new((row.values.shape().0, 2), &entries)
                    .map_err(super::nd_error)?
                    .with_identities(&identities)
                    .map_err(super::nd_error)?;
                let adapter = nd::PlotAdapter::from_parts_with_data(
                    config.native.clone(),
                    metadata.clone(),
                    projection,
                )
                .map_err(super::nd_error)?
                .facet_grid(0usize, 1usize)
                .map_err(super::nd_error)?;
                from_nd(adapter)
            }
            GridScope::RowCells => {
                // Only the explicitly cell-scoped side is prebound. The
                // row-scoped side still enters the original repeat producer
                // as an ordinary selector, never native_slots=true.
                let row = row.category("facet row")?;
                column.with_nd(|projection| {
                    Ok(from_nd(
                        nd::PlotAdapter::from_parts_with_data(
                            config.native.clone(),
                            metadata.clone(),
                            projection,
                        )
                        .map_err(super::nd_error)?
                        .facet_grid_row_cells(row, 0usize)
                        .map_err(super::nd_error)?,
                    ))
                })?
            }
            GridScope::ColumnCells => {
                let column = column.category("facet column")?;
                row.with_nd(|projection| {
                    Ok(from_nd(
                        nd::PlotAdapter::from_parts_with_data(
                            config.native.clone(),
                            metadata.clone(),
                            projection,
                        )
                        .map_err(super::nd_error)?
                        .facet_grid_column_cells(0usize, column)
                        .map_err(super::nd_error)?,
                    ))
                })?
            }
            GridScope::Cells => from_nd(
                nd::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                    .facet_grid_cells(row.category("facet row")?, column.category("facet column")?)
                    .map_err(super::nd_error)?,
            ),
        },
        Origin::Polars(metadata) => {
            let row = row.column();
            let column = column.column();
            let adapter = pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone());
            let adapter = match scope {
                GridScope::Rows => adapter.facet_grid(&row, &column),
                GridScope::RowCells => adapter.facet_grid_row_cells(&row, &column),
                GridScope::ColumnCells => adapter.facet_grid_column_cells(&row, &column),
                GridScope::Cells => adapter.facet_grid_cells(&row, &column),
            }
            .map_err(super::pl_error)?;
            from_pl(adapter)
        }
    };
    Ok(families::finish(config, roots, aliases))
}

macro_rules! generated_facet {
    ($function:ident) => {
        pub(crate) fn $function<K: kind::Facet>(
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
generated_facet!(facet_columns_row);
generated_facet!(facet_columns_column);

pub(crate) fn facet_columns_wrap<K: kind::Facet>(
    config: &Config<K>,
    roots: &InputRoots,
    columns: usize,
    py: Python<'_>,
) -> PyResult<Prepared<K>> {
    no_input(
        config,
        roots,
        py,
        |adapter| adapter.facet_columns_wrap(columns),
        |adapter| adapter.facet_columns_wrap(columns),
        "this plot has no generated wide-column dimension",
    )
}

macro_rules! generated_grid {
    ($function:ident) => {
        pub(crate) fn $function<K: kind::Facet>(
            config: &Config<K>,
            roots: &InputRoots,
            other: &Bound<'_, PyAny>,
        ) -> PyResult<Prepared<K>> {
            if config.is_core() {
                return Err(crate::DataError::new_err(
                    "this plot has no generated wide-column dimension",
                ));
            }
            one_category(
                config,
                roots,
                other,
                "facet",
                |_, _| {
                    Err(crate::DataError::new_err(
                        "this plot has no generated wide-column dimension",
                    ))
                },
                |adapter, input| adapter.$function(input),
                |adapter, input| adapter.$function(input),
            )
        }
    };
}
generated_grid!(facet_grid_columns_row);
generated_grid!(facet_grid_columns_column);
generated_grid!(facet_grid_columns_row_cells);
generated_grid!(facet_grid_columns_column_cells);

pub(crate) fn facet_scales<K: kind::Facet>(
    config: &Config<K>,
    roots: &InputRoots,
    scales: FacetScales,
    py: Python<'_>,
) -> PyResult<Prepared<K>> {
    let roots = roots.stage(py)?;
    let config = match &config.origin {
        Origin::Core => {
            let facet = config
                .native
                .get_facet()
                .ok_or_else(|| {
                    crate::DataError::new_err("select a facet layout before setting its scales")
                })?
                .clone()
                .scales(scales);
            config.clone().map_native(|native| native.facet(facet))
        }
        Origin::Nd(metadata) => from_nd(
            nd::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                .facet_scales(scales)
                .map_err(super::nd_error)?,
        ),
        Origin::Polars(metadata) => from_pl(
            pl::PlotAdapter::from_parts(config.native.clone(), metadata.clone())
                .facet_scales(scales)
                .map_err(super::pl_error)?,
        ),
    };
    Ok(Prepared { config, roots })
}
