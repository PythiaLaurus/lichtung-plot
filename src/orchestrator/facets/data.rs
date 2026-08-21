//! Facet selector construction, row extraction, and anchor validation.

use crate::Error;
use polars::prelude::{DataFrame, IdxCa, IdxSize};

use crate::builder::Chart;
use crate::layout::PanelBox;
use crate::spec::{LineMode, PlotKind};

type FacetCell = Vec<(String, String)>;
type PositionedFacetCell = (usize, usize, FacetCell);

pub(super) struct FacetSelectors {
    cells: Vec<PositionedFacetCell>,
    pub(super) rows: usize,
    pub(super) columns: usize,
}

pub(super) struct FacetPanel {
    pub(super) panel_index: usize,
    pub(super) row: usize,
    pub(super) column: usize,
    pub(super) cell: FacetCell,
    pub(super) data: DataFrame,
    pub(super) source_rows: Vec<usize>,
    pub(super) panel: PanelBox,
}

pub(super) fn facet_selectors(chart: &Chart<'_>) -> Result<FacetSelectors, Error> {
    let source_rows = chart.configured_source_rows()?;
    let data = source_rows.as_deref().map_or_else(
        || crate::data::PlotData::new(chart.data()),
        |rows| crate::data::PlotData::with_source_rows(chart.data(), rows),
    );
    let (row_column, column_column, wrap_column, wrap_columns) = chart.facet_columns();
    let variables = [row_column, column_column, wrap_column]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    chart.resolve_label_domain("facet variables", variables.iter().copied())?;
    let cells = if let Some(wrap) = wrap_column {
        let levels = stable_levels(crate::data::strict_strings(data, wrap, "facet")?);
        chart.resolve_label_domain("facet levels", levels.iter().map(String::as_str))?;
        let columns = wrap_columns.unwrap_or(1);
        levels
            .into_iter()
            .enumerate()
            .map(|(index, level)| {
                (
                    index / columns,
                    index % columns,
                    vec![(wrap.to_owned(), level)],
                )
            })
            .collect::<Vec<_>>()
    } else {
        let row_levels = row_column.map_or_else(
            || Ok(vec![String::new()]),
            |column| crate::data::strict_strings(data, column, "facet").map(stable_levels),
        )?;
        let column_levels = column_column.map_or_else(
            || Ok(vec![String::new()]),
            |column| crate::data::strict_strings(data, column, "facet").map(stable_levels),
        )?;
        if let Some(column) = row_column {
            chart.resolve_label_domain(
                &format!("facet levels for {column:?}"),
                row_levels.iter().map(String::as_str),
            )?;
        }
        if let Some(column) = column_column {
            chart.resolve_label_domain(
                &format!("facet levels for {column:?}"),
                column_levels.iter().map(String::as_str),
            )?;
        }
        let mut cells = Vec::new();
        for (row_index, row_level) in row_levels.iter().enumerate() {
            for (column_index, column_level) in column_levels.iter().enumerate() {
                let mut cell = Vec::new();
                if let Some(column) = row_column {
                    cell.push((column.to_owned(), row_level.clone()));
                }
                if let Some(column) = column_column {
                    cell.push((column.to_owned(), column_level.clone()));
                }
                cells.push((row_index, column_index, cell));
            }
        }
        cells
    };
    if cells.is_empty() {
        return Err(Error::Data(
            "faceting requires at least one observed level".into(),
        ));
    }
    let rows = cells.iter().map(|(row, _, _)| *row).max().unwrap_or(0) + 1;
    let columns = cells
        .iter()
        .map(|(_, column, _)| *column)
        .max()
        .unwrap_or(0)
        + 1;
    Ok(FacetSelectors {
        cells,
        rows,
        columns,
    })
}

pub(super) fn base_chart<'a>(
    chart: &Chart<'a>,
    kind: PlotKind,
    has_horizontal_axis_label: bool,
    has_vertical_axis_label: bool,
) -> Result<Chart<'a>, Error> {
    let source_rows = chart.configured_source_rows()?;
    let data = source_rows.as_deref().map_or_else(
        || crate::data::PlotData::new(chart.data()),
        |rows| crate::data::PlotData::with_source_rows(chart.data(), rows),
    );
    let mut base = chart.clone().without_facets();
    if has_horizontal_axis_label {
        base = base.x_label("");
    }
    if has_vertical_axis_label {
        base = base.y_label("");
    }
    if let Some(group_column) = chart.group_col() {
        let levels = stable_levels(crate::data::strict_strings(
            data,
            group_column,
            "facet graphical group",
        )?);
        base = base.with_global_group_levels(levels);
    }
    if kind == PlotKind::Line && chart.line_mode_spec() == LineMode::Trajectory {
        if let Some(x_column) = chart.x_col() {
            let source = chart.data().column(x_column).map_err(|error| {
                Error::Data(format!("line trajectory: column {x_column:?}: {error}"))
            })?;
            if !source.dtype().is_numeric() {
                let levels = stable_levels(crate::data::strict_normalized_strings(
                    data,
                    x_column,
                    "line trajectory",
                    "condition",
                )?);
                base = base.with_global_line_condition_levels(levels);
            }
        }
    }
    Ok(base)
}

pub(super) fn extract_panels(
    chart: &Chart<'_>,
    selectors: &FacetSelectors,
    provisional: &[PanelBox],
) -> Result<Vec<FacetPanel>, Error> {
    let configured_source_rows = chart.configured_source_rows()?;
    let mapped_data = configured_source_rows.as_deref().map_or_else(
        || crate::data::PlotData::new(chart.data()),
        |rows| crate::data::PlotData::with_source_rows(chart.data(), rows),
    );
    let mut panels = Vec::new();
    for (panel_index, (row, column, cell)) in selectors.cells.iter().enumerate() {
        let columns_data = cell
            .iter()
            .map(|(name, _)| crate::data::strict_strings(mapped_data, name, "facet"))
            .collect::<Result<Vec<_>, _>>()?;
        let source_indices = (0..chart.data().height())
            .filter(|index| {
                cell.iter().enumerate().all(|(selector_index, (_, level))| {
                    columns_data[selector_index][*index] == *level
                })
            })
            .collect::<Vec<_>>();
        let indices = source_indices
            .iter()
            .copied()
            .map(IdxSize::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::Data("facet row index exceeds backend limits".into()))?;
        if indices.is_empty() {
            continue;
        }
        let source_rows = source_indices
            .iter()
            .map(|index| {
                configured_source_rows
                    .as_ref()
                    .map_or(*index, |rows| rows[*index])
            })
            .collect();
        let data = chart
            .data()
            .take(&IdxCa::from_vec("facet".into(), indices))?;
        let panel = provisional[*row * selectors.columns + *column];
        panels.push(FacetPanel {
            panel_index,
            row: *row,
            column: *column,
            cell: cell.clone(),
            data,
            source_rows,
            panel,
        });
    }
    Ok(panels)
}

pub(super) fn validate_comparison_facet_anchors(
    chart: &Chart<'_>,
    panels: &[FacetPanel],
) -> Result<(), Error> {
    for comparison in chart.comparisons() {
        let anchored = panels.iter().any(|panel| {
            panel.cell.iter().all(|(variable, level)| {
                comparison
                    .facet
                    .iter()
                    .any(|facet| facet.variable == *variable && facet.level == *level)
            })
        });
        if !anchored {
            return Err(Error::Render(format!(
                "faceted significance comparison {:?} versus {:?} does not identify an observed facet panel",
                comparison.endpoint_a.category, comparison.endpoint_b.category
            )));
        }
    }
    Ok(())
}

pub(super) fn stable_levels(values: Vec<String>) -> Vec<String> {
    let mut levels = Vec::new();
    for value in values {
        if !levels.contains(&value) {
            levels.push(value);
        }
    }
    levels
}
