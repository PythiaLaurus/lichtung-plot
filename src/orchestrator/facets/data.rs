//! Facet selector construction, row extraction, and anchor validation.

use crate::Error;

use crate::builder::Plot;
use crate::layout::PanelBox;
use crate::spec::PlotKind;

#[derive(Clone)]
pub(super) struct FacetSelector {
    pub(super) column: &'static str,
    pub(super) variable: String,
    pub(super) level: String,
}

pub(super) type FacetCell = Vec<FacetSelector>;
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
    pub(super) rows: Vec<usize>,
    pub(super) source_rows: Vec<usize>,
    pub(super) panel: PanelBox,
}

impl FacetPanel {
    pub(super) fn render_scope(&self) -> String {
        let selectors = self
            .cell
            .iter()
            .map(|selector| format!("{}={}", selector.variable, selector.level))
            .collect::<Vec<_>>()
            .join(", ");
        if selectors.is_empty() {
            format!("facet-{}", self.panel_index)
        } else {
            format!("facet {selectors}")
        }
    }

    pub(super) fn significance_selectors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.cell
            .iter()
            .map(|selector| (selector.variable.as_str(), selector.level.as_str()))
    }
}

pub(super) fn facet_selectors(chart: &Plot) -> Result<FacetSelectors, Error> {
    let source_rows = chart.configured_source_rows()?;
    let data = source_rows.map_or_else(|| chart.data(), |rows| chart.data().map_source_rows(rows));
    let (row_column, column_column, wrap_column, wrap_columns) = chart.facet_columns();
    let variables = [row_column, column_column, wrap_column]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let variable_names = variables
        .iter()
        .map(|column| data.column(column).map(crate::data::DataColumn::name))
        .collect::<Result<Vec<_>, _>>()?;
    chart.resolve_label_domain("facet variables", variable_names.iter().copied())?;
    let cells = if let Some(wrap) = wrap_column {
        let levels = stable_levels(crate::data::strict_strings(data, wrap, "facet")?);
        chart.resolve_label_domain("facet levels", levels.iter().map(String::as_str))?;
        let columns = wrap_columns.unwrap_or(1);
        let variable = data.column(wrap)?.name().to_owned();
        levels
            .into_iter()
            .enumerate()
            .map(|(index, level)| {
                (
                    index / columns,
                    index % columns,
                    vec![FacetSelector {
                        column: wrap,
                        variable: variable.clone(),
                        level,
                    }],
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
                    cell.push(FacetSelector {
                        column,
                        variable: data.column(column)?.name().to_owned(),
                        level: row_level.clone(),
                    });
                }
                if let Some(column) = column_column {
                    cell.push(FacetSelector {
                        column,
                        variable: data.column(column)?.name().to_owned(),
                        level: column_level.clone(),
                    });
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

pub(super) fn base_chart(
    chart: &Plot,
    kind: PlotKind,
    has_horizontal_axis_label: bool,
    has_vertical_axis_label: bool,
) -> Result<Plot, Error> {
    let source_rows = chart.configured_source_rows()?;
    let data = source_rows.map_or_else(|| chart.data(), |rows| chart.data().map_source_rows(rows));
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
    if kind == PlotKind::Trajectory {
        if let Some(x_column) = chart.x_col() {
            let source = chart.data().column(x_column).map_err(|error| {
                Error::Data(format!("trajectory: column {x_column:?}: {error}"))
            })?;
            if !source.is_numeric() {
                let levels = stable_levels(crate::data::strict_normalized_strings(
                    data,
                    x_column,
                    "trajectory",
                    "condition",
                )?);
                base = base.with_global_line_condition_levels(levels);
            }
        }
    }
    Ok(base)
}

pub(super) fn extract_panels(
    chart: &Plot,
    selectors: &FacetSelectors,
    provisional: &[PanelBox],
) -> Result<Vec<FacetPanel>, Error> {
    let configured_source_rows = chart.configured_source_rows()?;
    let mapped_data = configured_source_rows
        .map_or_else(|| chart.data(), |rows| chart.data().map_source_rows(rows));
    let mut panels = Vec::new();
    for (panel_index, (row, column, cell)) in selectors.cells.iter().enumerate() {
        let columns_data = cell
            .iter()
            .map(|selector| crate::data::strict_strings(mapped_data, selector.column, "facet"))
            .collect::<Result<Vec<_>, _>>()?;
        let source_indices = (0..chart.row_count())
            .filter(|index| {
                cell.iter().enumerate().all(|(selector_index, selector)| {
                    columns_data[selector_index][*index] == selector.level
                })
            })
            .collect::<Vec<_>>();
        if source_indices.is_empty() {
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
        let panel = provisional[*row * selectors.columns + *column];
        panels.push(FacetPanel {
            panel_index,
            row: *row,
            column: *column,
            cell: cell.clone(),
            rows: source_indices,
            source_rows,
            panel,
        });
    }
    Ok(panels)
}

pub(super) fn validate_significance_facet_anchors(
    chart: &Plot,
    panels: &[FacetPanel],
) -> Result<(), Error> {
    let dimensions = panels.first().map_or(0, |panel| panel.cell.len());
    for annotation in chart.significance_annotations() {
        let Some(cell) = annotation.facet_cell() else {
            return Err(Error::Render(
                "faceted significance annotation must identify one FacetCell; annotations are never replicated across facets".into(),
            ));
        };
        if cell.dimensions() != dimensions {
            return Err(Error::Render(
                "significance FacetCell shape does not match the configured facet layout".into(),
            ));
        }
        let anchored = panels
            .iter()
            .any(|panel| cell.matches(panel.cell.iter().map(|selector| selector.level.as_str())));
        if !anchored {
            return Err(Error::Render(
                "faceted significance annotation does not identify an observed facet panel".into(),
            ));
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
